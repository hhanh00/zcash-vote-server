use anyhow::Result;
use rusqlite::Connection;

pub fn create_schema(connection: &Connection) -> Result<()> {
    connection.execute(
        "CREATE TABLE IF NOT EXISTS elections(
            id_election INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            definition TEXT NOT NULL)", [])?;

    Ok(())
}

