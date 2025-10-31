use anyhow::Result;
use blake2b_simd::Params;
use orchard::vote::Ballot;
use rusqlite::{params, Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use zcash_vote::{db::{load_prop, store_cmx_root, store_prop}, election::Election};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppState {
    pub height: u32,
    pub hash: String,
}

pub fn create_schema(connection: &Connection) -> Result<()> {
    zcash_vote::db::create_schema(connection)?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS properties(
        id_property INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        value TEXT NOT NULL)",
        [],
    )?;

    connection.execute(
        "CREATE TABLE IF NOT EXISTS elections(
            id_election INTEGER PRIMARY KEY,
            id TEXT NOT NULL UNIQUE,
            definition TEXT NOT NULL,
            closed BOOLEAN NOT NULL)",
        [],
    )?;

    if load_prop(connection, "state")?.is_none() {
        let hash = Params::new().hash_length(32).personal(b"Zcash_Vote_CmBFT").to_state().finalize();
        let hash = hex::encode(hash.as_bytes());

        let initial_state= AppState {
            height: 0,
            hash,
        };
        store_prop(connection, "state", &serde_json::to_string(&initial_state).unwrap())?;
    }

    Ok(())
}

pub fn get_election(connection: &Connection, id: &str) -> Result<(u32, String, bool)> {
    let res = connection.query_row(
        "SELECT id_election, definition, closed FROM elections WHERE id = ?1",
        [id],
        |r| {
            let id_election = r.get::<_, u32>(0)?;
            let election = r.get::<_, String>(1)?;
            let closed = r.get::<_, bool>(2)?;
            Ok((id_election, election, closed))
        },
    )?;
    Ok(res)
}

pub fn store_election(connection: &Connection, election: &Election, closed: bool) -> Result<u32> {
    let id_election = connection.query_row(
        "INSERT INTO elections(id, definition, closed)
        VALUES (?1, ?2, ?3)
        ON CONFLICT DO UPDATE SET
        definition = excluded.definition,
        closed = excluded.closed
        RETURNING id_election",
        params![&election.id(), serde_json::to_string(&election)?, closed],
        |r| r.get::<_, u32>(0),
    )?;
    Ok(id_election)
}

pub fn check_cmx_root(connection: &Connection, id_election: u32, cmx: &[u8]) -> Result<()> {
    let r = connection
        .query_row(
            "SELECT 1 FROM cmx_roots WHERE election = ?1 AND hash = ?2",
            params![id_election, cmx],
            |_| Ok(()),
        )
        .optional()?;
    r.ok_or(anyhow::anyhow!("Invalid cmx root"))
}

pub fn store_ballot(
    connection: &Connection,
    id_election: u32,
    height: u32,
    ballot: &Ballot,
    cmx_root: &[u8],
) -> Result<u32> {
    let hash = ballot.data.sighash()?;
    connection.execute(
        "INSERT INTO ballots
        (election, height, hash, data)
        VALUES (?1, ?2, ?3, ?4)",
        params![id_election, height, &hash, serde_json::to_string(ballot)?],
    )?;
    let id_ballot = connection.last_insert_rowid() as u32;

    store_cmx_root(connection, id_election, id_ballot, cmx_root)?;
    Ok(id_ballot)
}

pub fn get_ballot_height(connection: &Connection, id_election: u32, height: u32) -> Result<String> {
    let e = connection.query_row(
        "SELECT data FROM ballots WHERE election = ?1 AND height = ?2",
        params![id_election, height],
        |r| r.get::<_, String>(0),
    )?;
    Ok(e)
}

pub fn get_num_ballots(connection: &Connection, id_election: u32) -> Result<u32> {
    let n = connection.query_row(
        "SELECT COUNT(*) FROM ballots WHERE election = ?1",
        [id_election],
        |r| r.get::<_, u32>(0),
    )?;
    Ok(n)
}
