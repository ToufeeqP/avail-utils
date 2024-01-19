use anyhow::Result;
pub mod account_utils;
pub mod batch_transfer;
mod create_pools;
mod staking_utils;
mod validator_rewards;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // account_utils::generate_accounts(10)?;
    // batch_transfer::batch_transfer(10_000_000_000_000_000_000u64).await?;
    // staking_utils::bond_and_nominate_batch(9_000_000_000_000_000_000u64).await?;
    let execution_start = std::time::Instant::now();
    validator_rewards::fetch_blocks(256).await?;
    println!("Time elapsed: {:?}", execution_start.elapsed());
    Ok(())
}
