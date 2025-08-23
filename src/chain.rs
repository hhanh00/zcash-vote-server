use anyhow::Result;
use bincode::Options;
use blake2b_simd::Params;
use rusqlite::{params, OptionalExtension};
use std::{
    collections::{hash_map::Entry, HashMap, HashSet},
    sync::mpsc::{channel, Receiver, Sender},
};
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
    ExecTxResult, RequestCheckTx, RequestFinalizeBlock, RequestInfo, RequestPrepareProposal,
    RequestQuery, ResponseCheckTx, ResponseCommit, ResponseFinalizeBlock, ResponseInfo,
    ResponsePrepareProposal, ResponseQuery,
};

use crate::{
    db::{check_cmx_root, get_election, store_ballot, AppState},
    routes::Tx,
};

pub enum Command {
    Stop,
    Info(Sender<AppState>),
    CheckBallot(String, Ballot, Sender<Result<String, String>>),
    PrepareProposal(String, Ballot, Sender<Option<String>>),
    FinalizeBallot(String, Ballot, Sender<Result<String, String>>),
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
            check_cache: HashMap::new(),
            dnfs: HashSet::new(),
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

    fn check_tx(&self, request: RequestCheckTx) -> ResponseCheckTx {
        tracing::info!(
            "check_tx --> {} TYPE {}",
            hex::encode(&request.tx[0..16]),
            request.r#type
        );

        let Tx { id, ballot } = bincode::options()
            .with_little_endian()
            .deserialize(&request.tx).unwrap();
        let (tx_result, rx_result) = channel();
        self.cmd_tx
            .send(Command::CheckBallot(id, ballot, tx_result))
            .map_err(anyhow::Error::msg)
            .unwrap();

        let res = rx_result.recv().unwrap();
        match res {
            Ok(hash) => {
                tracing::info!("check_tx ok: {}", hash);
                ResponseCheckTx {
                    code: 0,
                    data: hash.into(),
                    ..Default::default()
                }
            }

            Err(message) => {
                tracing::error!("check_tx failed: {}", message);
                ResponseCheckTx {
                    code: 1,
                    data: message.into(),
                    ..Default::default()
                }
            }
        }
    }

    fn prepare_proposal(&self, request: RequestPrepareProposal) -> ResponsePrepareProposal {
        let mut filtered_txs = vec![];
        for tx in request.txs.into_iter() {
            let Tx { id, ballot } = bincode::options()
                .with_little_endian()
                .deserialize(&tx).unwrap();
            let sighash = hex::encode(&ballot.data.sighash().unwrap());
            let (tx_result, rx_result) = channel();
            self.cmd_tx
                .send(Command::PrepareProposal(id, ballot, tx_result))
                .map_err(anyhow::Error::msg)
                .unwrap();
            let res = rx_result.recv().unwrap();
            match res {
                None => {
                    tracing::info!("prepare_proposal: {}", sighash);
                    filtered_txs.push(tx)
                }
                Some(error) => tracing::error!("prepare_proposal: {}", error),
            }
        }
        ResponsePrepareProposal { txs: filtered_txs }
    }

    fn finalize_block(&self, request: RequestFinalizeBlock) -> ResponseFinalizeBlock {
        let mut tx_results = vec![];
        for tx in request.txs.iter() {
            let Tx { id, ballot } = bincode::options()
                .with_little_endian()
                .deserialize(&tx).unwrap();
            let (tx_result, rx_result) = channel();
            self.cmd_tx
                .send(Command::FinalizeBallot(id, ballot, tx_result))
                .map_err(anyhow::Error::msg)
                .unwrap();
            let res = rx_result.recv().unwrap();
            tracing::info!("finalize_block: {:?}", res);

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
    check_cache: HashMap<String, Result<String, String>>,
    dnfs: HashSet<String>,
}

impl VoteChainRunner {
    fn get_state(connection: &PooledConnection<SqliteConnectionManager>) -> AppState {
        let s = load_prop(connection, "state").unwrap().unwrap();
        let app_state = serde_json::from_str::<AppState>(&s).unwrap();
        app_state
    }

    fn process_command(&mut self, cmd: &Command) -> Result<()> {
        match cmd {
            Command::Stop => return Ok(()), // handled by caller
            Command::Info(result) => {
                let app_state = Self::get_state(&self.connection);
                result.send(app_state).unwrap();
            }
            Command::CheckBallot(id, ballot, result) => {
                let sighash = hex::encode(ballot.data.sighash().unwrap());
                let r = match self.check_cache.entry(sighash.clone()) {
                    Entry::Occupied(r) => r.get().clone(),
                    Entry::Vacant(ve) => {
                        let res = || {
                            let connection = &self.connection;
                            let (id_election, election, closed) =
                                get_election(connection, &id).map_err(|e| e.to_string())?;
                            if closed {
                                return Err("Election is closed".to_string());
                            }
                            let election = serde_json::from_str::<Election>(&election)
                                .map_err(|e| e.to_string())?;
                            // check ballot zkp, and signatures
                            let data = orchard::vote::validate_ballot(
                                ballot.clone(),
                                election.signature_required,
                                &BALLOT_VK,
                            )
                            .map_err(|e| e.to_string())?;
                            tracing::info!("Checking ballot {}", sighash);

                            // check that the public data matches with the election params
                            // nf_root & cmx_root
                            if data.anchors.nf != election.nf.0 {
                                return Err("Incorrect nullifier root".to_string());
                            }
                            check_cmx_root(connection, id_election, &data.anchors.cmx)
                                .map_err(|e| e.to_string())?;

                            // check that we are not double spending a previous note
                            for action in data.actions.iter() {
                                let dnf = &action.nf;
                                let exists = connection
                                    .query_row(
                                        "SELECT 1 FROM dnfs WHERE election = ?1 AND hash = ?2",
                                        params![&election.id(), dnf],
                                        |_| Ok(()),
                                    )
                                    .optional()
                                    .unwrap()
                                    .is_some();
                                if exists {
                                    return Err("Duplicate nullifier: double spend".to_string());
                                }
                            }
                            Ok::<_, String>(sighash.clone())
                        };
                        let r = res();
                        ve.insert_entry(r.clone());
                        r
                    }
                };

                result.send(r).unwrap();
            }
            Command::PrepareProposal(_, ballot, sender) => {
                for a in ballot.data.actions.iter() {
                    let dnf = hex::encode(&a.nf);
                    let new_spend = self.dnfs.insert(dnf);
                    if !new_spend {
                        sender.send(Some("Double spend".to_string()))?;
                    }
                    else {
                        sender.send(None)?;
                    }
                }
            }
            Command::FinalizeBallot(id, ballot, result) => {
                let connection = &self.connection;
                let mut res = || {
                    let _ = connection.execute("ROLLBACK", []); // Ignore error
                    connection.execute("BEGIN TRANSACTION", [])?;

                    let (id_election, _, closed) = get_election(connection, &id)?;
                    if closed {
                        anyhow::bail!("Election is closed");
                    }

                    // election id, ballot zkp, signatures and
                    // double spends were checked in check_tx
                    let data = &ballot.data;

                    let height = connection.query_row(
                        "SELECT MAX(height) FROM cmx_frontiers WHERE election = ?1",
                        [id_election],
                        |r| r.get::<_, u32>(0),
                    )?;

                    let cmx_frontier = {
                        // calculate the new cmx_frontier
                        let cmx_frontier = connection.query_row(
                            "SELECT frontier FROM cmx_frontiers WHERE election = ?1 AND height = ?2",
                            params![id_election, height],
                            |r| r.get::<_, String>(0),
                        )?;
                        let mut cmx_frontier = serde_json::from_str::<Frontier>(&cmx_frontier)?;
                        for action in data.actions.iter() {
                            cmx_frontier.append(OrchardHash(as_byte256(&action.cmx)));
                            store_dnf(connection, id_election, &action.nf).map_err(|_| {
                                anyhow::anyhow!("Duplicate nullifier: double spend")
                            })?;
                        }
                        cmx_frontier
                    };

                    let cmx_root = cmx_frontier.root();
                    {
                        // store the new cmx_frontier
                        let cmx_frontier = serde_json::to_string(&cmx_frontier)?;
                        connection.execute(
                            "INSERT INTO cmx_frontiers(election, height, frontier)
                            VALUES (?1, ?2, ?3)",
                            params![id_election, height + 1, &cmx_frontier],
                        )?;
                    }

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

                    self.check_cache.remove(&sighash);
                    self.dnfs.clear();
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

        Ok(())
    }

    pub fn run(mut self) -> Result<()> {
        loop {
            let cmd = self.cmd_rx.recv().map_err(anyhow::Error::msg)?;
            match self.process_command(&cmd) {
                Ok(_) => {}
                Err(e) => {
                    tracing::error!("Error processing command: {}", e);
                }
            }
            if let Command::Stop = cmd {
                break;
            }
        }

        Ok(())
    }
}

const PERSO_VOTE_BFT: &[u8] = b"Zcash_Vote_CmBFT";
