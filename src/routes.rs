use anyhow::Error;
use rocket::{serde::json::Json, State};
use serde_json::Value;

use crate::context::Context;


#[rocket::get("/election/<id>")]
pub fn get_election(id: String, state: &State<Context>) -> Result<Json<Value>, String> {
    (|| {
        let connection = state.pool.get()?;
        let election = connection.query_row(
            "SELECT definition FROM elections WHERE id = ?1", [id],
            |r| r.get::<_, String>(0))?;
        let election = serde_json::from_str::<Value>(&election)?;
        Ok::<_, Error>(Json(election))
    })().map_err(|e| e.to_string())
}
