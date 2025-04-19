use anyhow::Error;
use base64::{prelude::BASE64_STANDARD, Engine as _};
use orchard::vote::Ballot;
use rocket::{http::Status, response::status::Custom, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{context::Context, db::get_election};

#[derive(Serialize, Deserialize)]
pub struct Tx {
    pub id: String,
    pub ballot: Ballot,
}

#[rocket::get("/election/<id>")]
pub fn get_election_by_id(id: String, state: &State<Context>) -> Result<Json<Value>, String> {
    (|| {
        let connection = state.pool.get()?;
        let (_, election, _) = get_election(&connection, &id)?;
        let election = serde_json::from_str::<Value>(&election)?;
        Ok::<_, Error>(Json(election))
    })()
    .map_err(|e| e.to_string())
}

#[rocket::get("/election/<id>/ballot/height/<height>")]
pub fn get_ballot_height(
    id: String,
    height: u32,
    state: &State<Context>,
) -> Result<Json<Value>, Custom<String>> {
    (|| {
        let connection = state.pool.get()?;
        let (id_election, _, _) = get_election(&connection, &id)?;
        let ballot = crate::db::get_ballot_height(&connection, id_election, height)?;
        let ballot = serde_json::from_str::<Value>(&ballot)?;
        Ok::<_, Error>(Json(ballot))
    })()
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))
}

#[rocket::get("/election/<id>/num_ballots")]
pub fn get_num_ballots(id: String, state: &State<Context>) -> Result<String, Custom<String>> {
    (|| {
        let connection = state.pool.get()?;
        let (id_election, _, _) = get_election(&connection, &id)?;
        let n = crate::db::get_num_ballots(&connection, id_election)?;
        Ok::<_, Error>(n.to_string())
    })()
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))
}

#[rocket::post("/election/<id>/ballot", format = "json", data = "<ballot>")]
pub async fn post_ballot(
    id: String,
    ballot: Json<Ballot>,
    state: &State<Context>,
) -> Result<String, Custom<String>> {
    let res = async {
        let comet_bft = state.comet_bft;
        tracing::info!("Ballot received");
        let tx = Tx {
            id,
            ballot: ballot.into_inner(),
        };
        let tx_bytes = bincode::serialize(&tx).unwrap();

        let rpc_port = comet_bft - 1;
        let tx_data = BASE64_STANDARD.encode(&tx_bytes);
        let req_body = serde_json::json!({
            "id": "",
            "method": "broadcast_tx_sync",
            "params": [tx_data]
        });
        let url = format!("http://127.0.0.1:{rpc_port}/v1");
        tracing::info!("Post to {}", url);
        let client = reqwest::Client::new();
        let rep = client
            .post(&url)
            .json(&req_body)
            .send()
            .await?
            .error_for_status()?;
        let json_rep: Value = rep.json().await?;
        tracing::info!("post ballot rep: {:?}", json_rep);
        if let Some(error_msg) = json_rep.pointer("/error/data") {
            anyhow::bail!(error_msg.as_str().unwrap().to_string());
        }
        let result = &json_rep
            .pointer("/result/hash")
            .map(|v| v.as_str().unwrap().to_string())
            .unwrap_or_default();

        Ok::<_, Error>(result.clone())
    };
    res.await.map_err(|e| {
        let e = e.root_cause();
        let err_string = e.to_string();
        Custom(Status::InternalServerError, err_string)
    })
}
