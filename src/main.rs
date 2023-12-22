use anyhow::Result;

mod batch_transfer;
mod create_pools;
mod validator_rewards;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    validator_rewards::fetch_validator_rewards(29, 32).await?;
    Ok(())
}
