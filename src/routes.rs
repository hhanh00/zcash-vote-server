use anyhow::Error;
use orchard::vote::{Ballot, Frontier, OrchardHash};
use rocket::{http::Status, response::status::Custom, serde::json::Json, State};
use rusqlite::params;
use serde_json::Value;
use zcash_vote::{
    as_byte256,
    db::store_dnf,
    election::{Election, BALLOT_VK},
};

use crate::{
    context::Context,
    db::{check_cmx_root, get_election, store_ballot},
};

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
pub fn post_ballot(
    id: String,
    ballot: Json<Ballot>,
    state: &State<Context>,
) -> Result<String, Custom<String>> {
    let res = || {
        println!("Ballot received");
        let pool = &state.pool;
        let mut connection = pool.get()?;
        let (id_election, election, closed) = get_election(&connection, &id)?;
        if closed {
            anyhow::bail!("Election is closed");
        }
        let election = serde_json::from_str::<Election>(&election)?;
        let data = orchard::vote::validate_ballot(
            ballot.clone().into_inner(),
            election.signature_required,
            &BALLOT_VK,
        )?;
        println!("Validated");

        let transaction = connection.transaction()?;
        if &data.anchors.nf != &election.nf.0 {
            anyhow::bail!("Incorrect nullifier root");
        }
        check_cmx_root(&transaction, id_election, &data.anchors.cmx)?;
        let height = transaction.query_row(
            "SELECT MAX(height) FROM cmx_frontiers WHERE election = ?1",
            [id_election],
            |r| r.get::<_, u32>(0),
        )?;
        let cmx_frontier = transaction.query_row(
            "SELECT frontier FROM cmx_frontiers WHERE election = ?1 AND height = ?2",
            params![id_election, height],
            |r| r.get::<_, String>(0),
        )?;
        let mut cmx_frontier = serde_json::from_str::<Frontier>(&cmx_frontier)?;
        for action in data.actions.iter() {
            cmx_frontier.append(OrchardHash(as_byte256(&action.cmx)));
            store_dnf(&transaction, id_election, &action.nf)?;
        }
        let cmx_root = cmx_frontier.root();
        println!("cmx_root  {}", hex::encode(&cmx_root));
        let cmx_frontier = serde_json::to_string(&cmx_frontier)?;
        transaction.execute(
            "INSERT INTO cmx_frontiers(election, height, frontier)
        VALUES (?1, ?2, ?3)",
            params![id_election, height + 1, &cmx_frontier],
        )?;
        let height = crate::db::get_num_ballots(&transaction, id_election)?;
        println!("{height}");
        store_ballot(&transaction, id_election, height + 1, &ballot, &cmx_root)?;
        let sighash = hex::encode(data.sighash()?);
        println!("{id_election} {sighash}");
        transaction.commit()?;
        println!("Commited");
        Ok::<_, Error>(sighash)
    };
    res().map_err(|e| Custom(Status::InternalServerError, e.to_string()))
}
