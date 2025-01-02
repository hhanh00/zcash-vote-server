use anyhow::Result;
use zcash_vote::{as_byte256, ballot::Ballot, election::{Frontier, OrchardHash}};

pub fn compute_new_cmx_root(old_frontier: &mut Frontier, ballot: &Ballot) -> Result<()> {
    for action in ballot.data.actions.iter() {
        old_frontier.append(OrchardHash(as_byte256(&action.cmx)));
    }
    Ok(())
}
