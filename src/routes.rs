use anyhow::Error;
use orchard::vote::Ballot;
use rocket::{http::Status, response::status::Custom, serde::json::Json, State};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tendermint_abci::ClientBuilder;
use tendermint_proto::abci::RequestFinalizeBlock;

use crate::{
    context::Context,
    db::get_election,
};

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
    // TODO: query block at height
    // client.query(RequestQuery {
    //         data: "test-key".into(),
    //         path: "".to_string(),
    //         height: 0,
    //         prove: false,
    //     })
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
    // TODO: get current height
    (|| {
        let connection = state.pool.get()?;
        let (id_election, _, _) = get_election(&connection, &id)?;
        let n = crate::db::get_num_ballots(&connection, id_election)?;
        Ok::<_, Error>(n.to_string())
    })()
    .map_err(|e| Custom(Status::InternalServerError, e.to_string()))
}

#[rocket::post("/election/<id>/ballot", format = "json", data = "<ballot>")]
pub fn post_ballot(
    id: String,
    ballot: Json<Ballot>,
    state: &State<Context>,
) -> Result<String, Custom<String>> {
    let res = || {
        let comet_bft = state.comet_bft;
        println!("Ballot received");
        let tx = Tx { id, ballot: ballot.into_inner() };
        let tx_bytes = bincode::serialize(&tx).unwrap();

        let mut bft_client = ClientBuilder::default().connect(format!("127.0.0.1:{}", comet_bft)).unwrap();
        bft_client.finalize_block(RequestFinalizeBlock {
            txs: vec![tx_bytes.into()],
            ..Default::default()
        })?;

        Ok::<_, Error>("".to_string())
    };
    res().map_err(|e| Custom(Status::InternalServerError, e.to_string()))
}
