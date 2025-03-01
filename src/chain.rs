use anyhow::Result;
use blake2b_simd::Params;
use rusqlite::params;
use std::sync::mpsc::{channel, Receiver, Sender};
use zcash_vote::{
    as_byte256,
    db::{load_prop, store_dnf, store_prop},
    election::{Election, BALLOT_VK},
};

use orchard::vote::{Ballot, Frontier, OrchardHash};
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use tendermint_abci::Application;
use tendermint_proto::abci::{
    ExecTxResult, RequestFinalizeBlock, RequestInfo, RequestQuery, ResponseCommit,
    ResponseFinalizeBlock, ResponseInfo, ResponseQuery,
};

use crate::{
    db::{check_cmx_root, get_election, store_ballot, AppState},
    routes::Tx,
};

pub enum Command {
    Stop,
    Info(Sender<AppState>),
    Ballot(String, Ballot, Sender<Result<String, String>>),
    Commit(Sender<AppState>),
}

#[derive(Clone)]
pub struct VoteChain {
    cmd_tx: Sender<Command>,
}

impl VoteChain {
    pub fn new(connection: PooledConnection<SqliteConnectionManager>) -> (Self, VoteChainRunner) {
        let (cmd_tx, cmd_rx) = channel::<Command>();
        let s = Self { cmd_tx };
        let r = VoteChainRunner {
            connection,
            cmd_rx,
        };
        (s, r)
    }
}

impl Application for VoteChain {
    fn info(&self, _request: RequestInfo) -> ResponseInfo {
        let (tx_result, rx_result) = channel();
        self.cmd_tx
            .send(Command::Info(tx_result))
            .map_err(anyhow::Error::msg)
            .unwrap();
        let app_state = rx_result.recv().unwrap();
        tracing::info!("INFO {:?}", app_state);

        ResponseInfo {
            data: "zcash-vote-bft".to_string(),
            version: "0.1.0".to_string(),
            app_version: 1,
            last_block_height: app_state.height as i64,
            last_block_app_hash: hex::decode(&app_state.hash).unwrap().into(),
        }
    }

    fn query(&self, _request: RequestQuery) -> ResponseQuery {
        Default::default()
    }

    fn finalize_block(&self, request: RequestFinalizeBlock) -> ResponseFinalizeBlock {
        let mut tx_results = vec![];
        for tx in request.txs.iter() {
            let Tx { id, ballot } = bincode::deserialize(&tx).unwrap();
            let (tx_result, rx_result) = channel();
            self.cmd_tx
                .send(Command::Ballot(id, ballot, tx_result))
                .map_err(anyhow::Error::msg)
                .unwrap();
            let res = rx_result.recv().unwrap();

            let tx_result = match res {
                Ok(_) => ExecTxResult {
                    code: 0,
                    log: "Validated".to_string(),
                    ..Default::default()
                },
                Err(err) => ExecTxResult {
                    code: 1,
                    log: format!("Validation failed: {}", err),
                    ..Default::default()
                },
            };
            tx_results.push(tx_result);
        }

        let (tx_result, rx_result) = channel();
        self.cmd_tx
            .send(Command::Info(tx_result))
            .map_err(anyhow::Error::msg)
            .unwrap();
        let app_state = rx_result.recv().unwrap();

        ResponseFinalizeBlock {
            tx_results,
            app_hash: hex::decode(&app_state.hash).unwrap().into(),
            ..Default::default()
        }
    }

    fn commit(&self) -> ResponseCommit {
        let (tx_result, rx_result) = channel();
        self.cmd_tx
            .send(Command::Commit(tx_result))
            .map_err(anyhow::Error::msg)
            .unwrap();
        let app_state = rx_result.recv().unwrap();
        ResponseCommit {
            retain_height: (app_state.height - 1) as i64,
        };

        Default::default()
    }
}

pub struct VoteChainRunner {
    connection: PooledConnection<SqliteConnectionManager>,
    cmd_rx: Receiver<Command>,
}

impl VoteChainRunner {
    fn get_state(connection: &PooledConnection<SqliteConnectionManager>) -> AppState {
        let s = load_prop(connection, "state").unwrap().unwrap();
        let app_state = serde_json::from_str::<AppState>(&s).unwrap();
        app_state
    }

    pub fn run(self) -> Result<()> {
        loop {
            let cmd = self.cmd_rx.recv().map_err(anyhow::Error::msg)?;
            match cmd {
                Command::Stop => return Ok(()),
                Command::Info(result) => {
                    let app_state = Self::get_state(&self.connection);
                    result.send(app_state).unwrap();
                }
                Command::Ballot(id, ballot, result) => {
                    let connection = &self.connection;
                    let res = move || {
                        let _ = connection.execute("ROLLBACK", []); // Ignore error
                        connection.execute("BEGIN TRANSACTION", [])?;

                        let (id_election, election, closed) = get_election(connection, &id)?;
                        if closed {
                            anyhow::bail!("Election is closed");
                        }
                        let election = serde_json::from_str::<Election>(&election)?;
                        let data = orchard::vote::validate_ballot(
                            ballot.clone(),
                            election.signature_required,
                            &BALLOT_VK,
                        )?;

                        if data.anchors.nf != election.nf.0 {
                            anyhow::bail!("Incorrect nullifier root");
                        }
                        check_cmx_root(connection, id_election, &data.anchors.cmx)?;
                        let height = connection.query_row(
                            "SELECT MAX(height) FROM cmx_frontiers WHERE election = ?1",
                            [id_election],
                            |r| r.get::<_, u32>(0),
                        )?;
                        let cmx_frontier = connection.query_row(
                                    "SELECT frontier FROM cmx_frontiers WHERE election = ?1 AND height = ?2",
                                    params![id_election, height],
                                    |r| r.get::<_, String>(0),
                                )?;
                        let mut cmx_frontier = serde_json::from_str::<Frontier>(&cmx_frontier)?;
                        for action in data.actions.iter() {
                            cmx_frontier.append(OrchardHash(as_byte256(&action.cmx)));
                            store_dnf(connection, id_election, &action.nf)?;
                        }
                        let cmx_root = cmx_frontier.root();
                        let cmx_frontier = serde_json::to_string(&cmx_frontier)?;
                        connection.execute(
                            "INSERT INTO cmx_frontiers(election, height, frontier)
                            VALUES (?1, ?2, ?3)",
                            params![id_election, height + 1, &cmx_frontier],
                        )?;
                        let height = crate::db::get_num_ballots(connection, id_election)?;
                        tracing::info!("ballot height: {height}");
                        store_ballot(connection, id_election, height + 1, &ballot, &cmx_root)?;
                        let sighash = hex::encode(data.sighash()?);
                        tracing::info!("election: {id_election} sighash: {sighash}");

                        let mut s = connection.prepare(
                            "SELECT t1.hash, t1.election
                            FROM cmx_roots t1
                            JOIN (
                                SELECT election, MAX(height) AS max_height
                                FROM cmx_roots
                                GROUP BY election
                            ) t2
                            ON t1.election = t2.election AND t1.height = t2.max_height",
                        )?;
                        let rows = s.query_map([], |r| {
                            Ok((r.get::<_, Vec<u8>>(0)?, r.get::<_, u32>(1)?))
                        })?;
                        let mut hasher = Params::new()
                            .hash_length(32)
                            .personal(PERSO_VOTE_BFT)
                            .to_state();
                        for r in rows {
                            let (h, _) = r?;
                            hasher.update(&h);
                        }
                        let hash = hasher.finalize();
                        let hash = hash.as_bytes().to_vec();

                        let app_state = Self::get_state(connection);
                        let app_state = AppState {
                            hash: hex::encode(&hash),
                            ..app_state
                        };
                        store_prop(
                            connection,
                            "state",
                            &serde_json::to_string(&app_state).unwrap(),
                        )?;

                        tracing::info!("Ballot finalized");

                        Ok::<_, anyhow::Error>(sighash)
                    };

                    result.send(res().map_err(|e| e.to_string())).unwrap();
                }
                Command::Commit(result) => {
                    let connection = &self.connection;
                    let _ = connection.execute("COMMIT", []);

                    let app_state = Self::get_state(connection);
                    let app_state = AppState {
                        height: app_state.height + 1,
                        ..app_state
                    };
                    store_prop(
                        connection,
                        "state",
                        &serde_json::to_string(&app_state).unwrap(),
                    )?;

                    result.send(app_state).unwrap();
                }
            }
        }
    }
}

const PERSO_VOTE_BFT: &[u8] = b"Zcash_Vote_CmBFT";
