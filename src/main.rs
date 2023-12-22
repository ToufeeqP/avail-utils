use anyhow::Result;
pub mod account_utils;
pub mod batch_transfer;
mod create_pools;
mod validator_rewards;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
   account_utils::generate_accounts(10)?;
    batch_transfer::batch_transfer(10_000_000_000_000_000_000u64).await?;
    Ok(())
}
