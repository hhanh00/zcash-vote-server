use anyhow::Result;
use rusqlite::{params, Connection};
use zcash_vote::Election;

pub fn create_schema(connection: &Connection) -> Result<()> {
    connection.execute(
        "CREATE TABLE IF NOT EXISTS elections(
            id_election INTEGER PRIMARY KEY,
            id TEXT NOT NULL UNIQUE,
            definition TEXT NOT NULL)", [])?;

    Ok(())
}

pub fn store_election(connection: &Connection, election: &Election) -> Result<()> {
    connection.execute(
        "INSERT INTO elections(id, definition)
        VALUES (?1, ?2)
        ON CONFLICT DO UPDATE SET
        definition = excluded.definition",
        params![election.id, serde_json::to_string(&election)?])?;
    Ok(())
}
