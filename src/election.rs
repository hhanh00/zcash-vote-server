use std::{
    fs::{read_dir, File},
    io::BufReader,
};

use anyhow::Result;
use zcash_vote::Election;

pub fn scan_data_dir(data_dir: &str) -> Result<Vec<Election>> {
    let mut elections = vec![];
    let dir = read_dir(data_dir)?;
    for entry in dir {
        if let Ok(entry) = entry {
            let p = entry.path();
            if p.is_file() {
                if let Ok(election) = serde_json::from_reader::<_, Election>(BufReader::new(File::open(&p)?)) {
                    elections.push(election);
                }
            }
        }
    }

    Ok(elections)
}
