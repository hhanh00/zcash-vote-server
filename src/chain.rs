use anyhow::Result;
use rusqlite::params;
use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};
use zcash_vote::{
    as_byte256,
    db::store_dnf,
    election::{Election, BALLOT_VK},
};

use orchard::vote::{Ballot, Frontier, OrchardHash};
use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use tendermint_abci::Application;
use tendermint_proto::abci::{RequestFinalizeBlock, ResponseCommit, ResponseFinalizeBlock};

use crate::{
    db::{check_cmx_root, get_election, store_ballot},
    routes::Tx,
};

pub enum Command {
    Stop,
    Ballot(String, Ballot, Sender<String>),
    Commit,
}

#[derive(Clone)]
pub struct VoteChain {
    cmd_tx: Sender<Command>,
}

impl VoteChain {
    pub fn new(connection: PooledConnection<SqliteConnectionManager>) -> (Self, VoteChainRunner) {
        let (cmd_tx, cmd_rx) = channel::<Command>();
        let s = Self { cmd_tx };
        let r = VoteChainRunner { connection, cmd_rx };
        (s, r)
    }
}

impl Application for VoteChain {
    fn query(
        &self,
        _request: tendermint_proto::abci::RequestQuery,
    ) -> tendermint_proto::abci::ResponseQuery {
        Default::default()
    }

    fn finalize_block(&self, request: RequestFinalizeBlock) -> ResponseFinalizeBlock {
        println!("finalize_block");

        for tx in request.txs.iter() {
            let Tx { id, ballot } = bincode::deserialize(&tx).unwrap();
            println!("{}", hex::encode(&ballot.data.domain));
            let (tx_result, rx_result) = channel::<String>();
            self.cmd_tx
                .send(Command::Ballot(id, ballot, tx_result))
                .map_err(anyhow::Error::msg)
                .unwrap();
            let sighash = rx_result.recv().unwrap();
            println!("SIGHASH {}", sighash);
            // TODO: report tx_results in ResponseFinalizeBlock
        }
        println!("end finalize_block");
        Default::default()
    }

    fn commit(&self) -> ResponseCommit {
        println!("commit");
        self.cmd_tx
            .send(Command::Commit)
            .map_err(anyhow::Error::msg)
            .unwrap();
        Default::default()
    }
}

pub struct VoteChainRunner {
    connection: PooledConnection<SqliteConnectionManager>,
    cmd_rx: Receiver<Command>,
}

impl VoteChainRunner {
    pub fn run(mut self) -> Result<()> {
        loop {
            let cmd = self.cmd_rx.recv().map_err(anyhow::Error::msg)?;
            match cmd {
                Command::Stop => return Ok(()),
                Command::Ballot(id, ballot, result) => {
                    println!(
                        "VoteChainRunner -> {}",
                        hex::encode(&ballot.data.sighash().unwrap())
                    );
                    let connection = &self.connection;
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
                    println!("Validated");

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
                    println!("cmx_root  {}", hex::encode(cmx_root));
                    let cmx_frontier = serde_json::to_string(&cmx_frontier)?;
                    connection.execute(
                        "INSERT INTO cmx_frontiers(election, height, frontier)
                    VALUES (?1, ?2, ?3)",
                        params![id_election, height + 1, &cmx_frontier],
                    )?;
                    let height = crate::db::get_num_ballots(connection, id_election)?;
                    println!("{height}");
                    store_ballot(connection, id_election, height + 1, &ballot, &cmx_root)?;
                    let sighash = hex::encode(data.sighash()?);
                    println!("{id_election} {sighash}");

                    println!("Ballot finalized");
                    result.send(sighash).unwrap();
                }
                Command::Commit => {
                    let connection = &self.connection;
                    let _ = connection.execute("COMMIT", []);
                }
            }
        }
    }
}
