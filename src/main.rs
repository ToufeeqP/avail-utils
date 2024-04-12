use anyhow::Result;
pub mod account_utils;
pub mod batch_transfer;
mod batch_txs;
mod create_pools;
mod periodic_da_submit;
mod scale_encode;
mod staking_utils;
mod validator_rewards;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // account_utils::generate_accounts(10)?;
    let execution_start = std::time::Instant::now();
    // batch_transfer::batch_transfer(10_000_000_000_000_000_000_000u128).await?;
    // staking_utils::bond_and_nominate_batch(1_000_000_000_000_000_000_000u128).await?;
    // validator_rewards::fetch_blocks(10).await?;
    // validator_rewards::fetch_validator_rewards(1, 3).await?;
    // validator_rewards::dump_validators().await?;
    // periodic_da_submit::submit_txs().await?;
    // scale_encode::encode();
    batch_txs::batch_submit().await?;
    println!("Time elapsed: {:?}", execution_start.elapsed());
    Ok(())
}
