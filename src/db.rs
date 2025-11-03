use anyhow::Result;
use blake2b_simd::Params;
use orchard::vote::Ballot;
use serde::{Deserialize, Serialize};
use sqlx::SqliteConnection;
use zcash_vote::{
    db::{load_prop, store_cmx_root, store_prop},
    election::Election,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct AppState {
    pub height: u32,
    pub hash: String,
}

pub async fn create_schema(connection: &mut SqliteConnection) -> Result<()> {
    zcash_vote::db::create_schema(connection).await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS properties(
        id_property INTEGER PRIMARY KEY,
        name TEXT NOT NULL UNIQUE,
        value TEXT NOT NULL)",
    )
    .execute(&mut *connection)
    .await?;

    sqlx::query(
        "CREATE TABLE IF NOT EXISTS elections(
            id_election INTEGER PRIMARY KEY,
            id TEXT NOT NULL UNIQUE,
            definition TEXT NOT NULL,
            closed BOOLEAN NOT NULL)",
    )
    .execute(&mut *connection)
    .await?;

    if load_prop(connection, "state").await?.is_none() {
        let hash = Params::new()
            .hash_length(32)
            .personal(b"Zcash_Vote_CmBFT")
            .to_state()
            .finalize();
        let hash = hex::encode(hash.as_bytes());

        let initial_state = AppState { height: 0, hash };
        store_prop(
            connection,
            "state",
            &serde_json::to_string(&initial_state).unwrap(),
        )
        .await?;
    }

    Ok(())
}

pub async fn get_election(
    connection: &mut SqliteConnection,
    id: &str,
) -> Result<(u32, String, bool)> {
    let res: (u32, String, bool) =
        sqlx::query_as("SELECT id_election, definition, closed FROM elections WHERE id = ?1")
            .bind(id)
            .fetch_one(&mut *connection)
            .await?;
    Ok(res)
}

pub async fn store_election(
    connection: &mut SqliteConnection,
    election: &Election,
    closed: bool,
) -> Result<u32> {
    let (id_election,): (u32,) = sqlx::query_as(
        "INSERT INTO elections(id, definition, closed)
        VALUES (?1, ?2, ?3)
        ON CONFLICT DO UPDATE SET
        definition = excluded.definition,
        closed = excluded.closed
        RETURNING id_election",
    )
    .bind(election.id())
    .bind(serde_json::to_string(&election)?)
    .bind(closed)
    .fetch_one(&mut *connection)
    .await?;
    Ok(id_election)
}

pub async fn check_cmx_root(
    connection: &mut SqliteConnection,
    id_election: u32,
    cmx: &[u8],
) -> Result<()> {
    let r = sqlx::query("SELECT 1 FROM cmx_roots WHERE election = ?1 AND hash = ?2")
        .bind(id_election)
        .bind(cmx)
        .fetch_optional(&mut *connection)
        .await?;
    r.ok_or(anyhow::anyhow!("Invalid cmx root"))?;
    Ok(())
}

pub async fn store_ballot(
    connection: &mut SqliteConnection,
    id_election: u32,
    height: u32,
    ballot: &Ballot,
    cmx_root: &[u8],
) -> Result<u32> {
    let hash = ballot.data.sighash()?;
    let r = sqlx::query(
        "INSERT INTO ballots
        (election, height, hash, data)
        VALUES (?1, ?2, ?3, ?4)",
    )
    .bind(id_election)
    .bind(height)
    .bind(&hash)
    .bind(serde_json::to_string(ballot)?)
    .execute(&mut *connection)
    .await?;
    let id_ballot = r.last_insert_rowid() as u32;

    store_cmx_root(connection, id_election, id_ballot, cmx_root).await?;
    Ok(id_ballot)
}

pub async fn get_ballot_height(
    connection: &mut SqliteConnection,
    id_election: u32,
    height: u32,
) -> Result<String> {
    let (e, ): (String, ) = sqlx::query_as(
        "SELECT data FROM ballots WHERE election = ?1 AND height = ?2")
        .bind(id_election).bind(height).fetch_one(&mut *connection).await?;
    Ok(e)
}

pub async fn get_num_ballots(connection: &mut SqliteConnection, id_election: u32) -> Result<u32> {
    let (n, ): (u32, ) = sqlx::query_as(
        "SELECT COUNT(*) FROM ballots WHERE election = ?1")
        .bind(id_election)
        .fetch_one(&mut *connection).await?;
    Ok(n)
}
