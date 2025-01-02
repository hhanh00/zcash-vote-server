use anyhow::Error;
use rocket::{serde::json::Json, State};
use serde_json::Value;
use zcash_vote::{
    as_byte256, ballot::Ballot, db::{load_prop, store_dnf, store_prop}, election::{Frontier, OrchardHash}, Election
};

use crate::{context::Context, db::{get_election, store_ballot, store_cmx}};

#[rocket::get("/election/<id>")]
pub fn get_election_by_id(id: String, state: &State<Context>) -> Result<Json<Value>, String> {
    (|| {
        let connection = state.pool.get()?;
        let election = connection.query_row(
            "SELECT definition FROM elections WHERE id = ?1",
            [id],
            |r| r.get::<_, String>(0),
        )?;
        let election = serde_json::from_str::<Value>(&election)?;
        Ok::<_, Error>(Json(election))
    })()
    .map_err(|e| e.to_string())
}

// #[rocket::get("/election/<id>/ballot/height/<height>")]
// pub fn get_ballot_height(id: String, height: u32, state: &State<Context>) -> Result<Json<Value>, String> {
//     (|| {
//         let connection = state.pool.get()?;
//         Ok::<_, Error>(Json(election))
//     })().map_err(|e| e.to_string())
// }

#[rocket::post("/election/<id>/ballot", format = "json", data = "<ballot>")]
pub fn post_ballot(id: String, ballot: Json<Ballot>, state: &State<Context>) -> Result<(), String> {
    let res = || {
        // TODO: Check ballot validity
        let pool = &state.pool;
        let mut connection = pool.get()?;
        let transaction = connection.transaction()?;
        let (id_election, election) = get_election(&transaction, &id)?;
        let election = serde_json::from_str::<Election>(&election)?;
        let cmx_frontier = load_prop(&transaction, "cmx_frontier")?;
        let cmx_frontier = cmx_frontier.map(|f| serde_json::from_str::<Frontier>(&f).unwrap());
        let mut cmx_frontier = cmx_frontier.unwrap_or(election.cmx_frontier.clone().unwrap());
        for action in ballot.data.actions.iter() {
            cmx_frontier.append(OrchardHash(as_byte256(&action.cmx)));
            store_dnf(&transaction, id_election, &action.nf)?;
        }
        let cmx_root = cmx_frontier.root();
        println!("cmx_root  {}", hex::encode(&cmx_root));
        store_prop(
            &transaction,
            "cmx_frontier",
            &serde_json::to_string(&cmx_frontier).unwrap(),
        )?;
        store_ballot(&transaction, id_election, &ballot, &cmx_root)?;
        zcash_vote::validate::validate_ballot(ballot.into_inner(), election.signature_required)?;
        transaction.commit()?;
        Ok::<_, Error>(())
    };
    res().map_err(|e| e.to_string())
}
