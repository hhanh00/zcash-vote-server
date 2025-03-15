use std::{
    fs::{read_dir, File},
    io::BufReader,
};

use anyhow::Result;
use zcash_vote::election::Election;
use ff::PrimeField as _;

pub fn scan_data_dir(data_dir: &str) -> Result<Vec<Election>> {
    let mut elections = vec![];
    let dir = read_dir(data_dir)?.flatten(); // Simplify the iterator

    for entry in dir {
        let p = entry.path();
        if p.is_file() {
            if let Ok(election) =
                serde_json::from_reader::<_, Election>(BufReader::new(File::open(&p)?))
            {
                tracing::info!("Election ID: {}", election.id());
                elections.push(election);
            }
        }
    }

    Ok(elections)
}