#![allow(dead_code)]
use anyhow::Result;
use serde::{Deserialize, Serialize};
pub mod account_utils;
pub mod batch_transfer;
mod create_pools;
// mod db_utils;
mod block_author;
mod periodic_da_submit;
mod scale_encode;
mod staking_utils;
mod validator_rewards;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // account_utils::generate_accounts(10)?;
    let execution_start = std::time::Instant::now();
    // check_block_authors().await?;
    block_author::verify_seal(Some(block_author::BlockId::Number(8497))).await?;
    // block_author::find_author(Some(block_author::BlockId::Number(8497))).await?;
    // scale_encode::decode_justification();
    // db_utils::run();
    // csv_accounts::generate_accounts(7500)?;
    // batch_transfer::batch_transfer(10_000u128).await?;
    // batch_transfer::batch_transfer_from_csv(500, 3).await?;
    // staking_utils::bond_and_nominate_batch(1_000_000_000_000_000_000_000u128).await?;
    // validator_rewards::fetch_blocks(10).await?;
    // validator_rewards::fetch_validator_rewards(1, 3).await?;
    // validator_rewards::dump_validators().await?;
    // println!(
    //     "vote_weight: {}",
    //     validator_rewards::to_vote_weight(100_000_000_000_000_000_000_000u128).await?
    // );
    // println!(
    //     "vote_to_currency: {}",
    //     validator_rewards::to_balance(183904829544958u64).await?
    // );
    // periodic_da_submit::submit_txs().await?;
    // scale_encode::encode();
    println!("Time elapsed: {:?}", execution_start.elapsed());
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
struct BlockIds {
    nodes: Vec<u32>,
}

impl BlockIds {
    pub fn from_json(slice: &[u8]) -> Result<Self, serde_json::Error> {
        serde_json::from_slice(slice)
    }
}

async fn check_block_authors() -> Result<()> {
    // turing
    // let blocks = r#"{
    //     "nodes": [
    //                 4177,
    //                 8497,
    //                 9773,
    //                 14092,
    //                 18411,
    //                 22730,
    //                 27050,
    //                 28358,
    //                 32677,
    //                 36997,
    //                 41317,
    //                 45636,
    //                 49954,
    //                 54274,
    //                 58455,
    //                 62775,
    //                 67065,
    //                 71268,
    //                 75588,
    //                 79855,
    //                 84175,
    //                 88369,
    //                 92630,
    //                 96934,
    //                 101231,
    //                 105515,
    //                 109832,
    //                 114055,
    //                 118371,
    //                 122683,
    //                 127003,
    //                 131319,
    //                 135614,
    //                 139889,
    //                 144167,
    //                 148455,
    //                 152748,
    //                 157048,
    //                 161302,
    //                 165620,
    //                 169939,
    //                 174257,
    //                 178568,
    //                 182869,
    //                 187173,
    //                 191475,
    //                 195795,
    //                 200115,
    //                 204411,
    //                 208727,
    //                 213046,
    //                 217350,
    //                 221664,
    //                 225958,
    //                 230277,
    //                 234584
    //     ]
    // }"#;

    // hex
    let blocks = r#"{
        "nodes": [
          90691, 8641, 86373, 82057, 77739, 73421, 69102, 64787, 60469, 56150, 51830,
          47511, 4321, 43194, 38876, 34558, 30239, 25919, 21601, 17281, 12961
        ]
      }
      "#;
    let block_ids = BlockIds::from_json(blocks.as_bytes()).unwrap();
    for node in block_ids.nodes {
        block_author::find_author(Some(block_author::BlockId::Number(node))).await?;
    }
    Ok(())
}
