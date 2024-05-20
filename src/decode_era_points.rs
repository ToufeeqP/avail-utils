use serde::Deserialize;
use codec::Decode;
use sp_core::H256;
use subxt::utils::AccountId32;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use sp_storage::StorageChangeSet;

#[derive(Debug, Deserialize)]
struct ResultData {
    result: Vec<StorageChangeSet<H256>>,
}

#[derive(Debug, PartialEq, Decode)]
pub struct EraRewardPoints {
    /// Total number of points. Equals the sum of reward points for each validator.
    pub total: u32,
    /// The reward points earned by a given validator.
    pub individual: BTreeMap<AccountId32, u32>,
}

fn decode_era_reward_points(data: &[u8]) -> EraRewardPoints {
    let mut input = &data[..];
    EraRewardPoints::decode(&mut input).expect("Failed to decode EraRewardPoints")
}

pub fn main() {
    let path = Path::new("era1_points.json");
    let mut file = File::open(&path).expect("Failed to open file");
    let mut json_data = String::new();
    file.read_to_string(&mut json_data).expect("Failed to read file");

    let data: ResultData = serde_json::from_str(&json_data).expect("Failed to parse JSON");

    for result in data.result {
        let block_hash = result.block;
        for (_storage_key, storage_value_opt) in result.changes {
            if let Some(ref storage_value) = storage_value_opt {
                // println!("storage_key:  {:?}, storage_value_opt:  {:?}", storage_key, storage_value_opt);
                let era_reward_points: EraRewardPoints = decode_era_reward_points(&storage_value.0.as_ref());
                // block_hash, total_points
                println!("{}, {}", block_hash, era_reward_points.total);
            }
        }
    }
}
