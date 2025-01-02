use anyhow::Error;
use rocket::{serde::json::Json, State};
use serde_json::Value;
use zcash_vote::{
    as_byte256, ballot::Ballot, db::{load_prop, store_dnf, store_prop}, election::{Frontier, OrchardHash}, Election
};

use crate::{context::Context, db::{check_cmx_root, get_election, store_ballot}};

#[rocket::get("/election/<id>")]
pub fn get_election_by_id(id: String, state: &State<Context>) -> Result<Json<Value>, String> {
    (|| {
        let connection = state.pool.get()?;
        let (_, election) = get_election(&connection, &id)?;
        let election = serde_json::from_str::<Value>(&election)?;
        Ok::<_, Error>(Json(election))
    })()
    .map_err(|e| e.to_string())
}

#[rocket::get("/election/<id>/ballot/height/<height>")]
pub fn get_ballot_height(id: String, height: u32, state: &State<Context>) -> Result<Json<Value>, String> {
    (|| {
        let connection = state.pool.get()?;
        let (id_election, _) = get_election(&connection, &id)?;
        let ballot = crate::db::get_ballot_height(&connection, id_election, height)?;
        let ballot = serde_json::from_str::<Value>(&ballot)?;
        Ok::<_, Error>(Json(ballot))
    })().map_err(|e| e.to_string())
}

#[rocket::get("/election/<id>/num_ballots")]
pub fn get_num_ballots(id: String, state: &State<Context>) -> Result<String, String> {
    (|| {
        let connection = state.pool.get()?;
        let (id_election, _) = get_election(&connection, &id)?;
        let n = crate::db::get_num_ballots(&connection, id_election)?;
        Ok::<_, Error>(n.to_string())
    })().map_err(|e| e.to_string())
}

#[rocket::post("/election/<id>/ballot", format = "json", data = "<ballot>")]
pub fn post_ballot(id: String, ballot: Json<Ballot>, state: &State<Context>) -> Result<String, String> {
    let res = || {
        let pool = &state.pool;
        let mut connection = pool.get()?;
        let transaction = connection.transaction()?;
        let (id_election, election) = get_election(&transaction, &id)?;
        let election = serde_json::from_str::<Election>(&election)?;

        if &ballot.data.anchors.nf != &election.nf.0 {
            anyhow::bail!("Incorrect nullifier root");
        }
        check_cmx_root(&transaction, id_election, &ballot.data.anchors.cmx)?;
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
        let height = crate::db::get_num_ballots(&transaction, id_election)?;
        store_ballot(&transaction, id_election, height + 1, &ballot, &cmx_root)?;
        let sighash = hex::encode(ballot.data.sighash()?);
        zcash_vote::validate::validate_ballot(ballot.into_inner(), election.signature_required)?;
        transaction.commit()?;
        Ok::<_, Error>(sighash)
    };
    res().map_err(|e| e.to_string())
}
