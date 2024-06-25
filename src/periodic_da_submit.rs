#![allow(dead_code)]
use avail_core::AppId;
use avail_subxt::{submit::submit_data_with_nonce, AvailClient, Opts};
use sp_arithmetic::traits::One;
use structopt::StructOpt;
use subxt_signer::sr25519::dev;
use tokio::time::Duration;

// Note: padded_block_length is the limiting factor for AVAIL chain, even though BlockLength may be 5MB or higher, padded_block_length needs to be considered to check how much length a block can have.
const BLOCK_SIZE: usize = 2 * 1024 * 1024;
const TX_SIZE: usize = 100 * 1024;
const NUM_CHUNKS: usize = BLOCK_SIZE / TX_SIZE;
const DELAY_SECS: u64 = 20;

const ACCT_SEED: &str =
    "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice";

pub async fn submit_txs() -> anyhow::Result<()> {
    let args = Opts::from_args();
    let client = AvailClient::new(args.ws).await?;

    println!(
        "{} txs of size {} will be submitted in every {} seconds",
        NUM_CHUNKS, TX_SIZE, DELAY_SECS
    );
    // let mnemonic = Mnemonic::parse(ACCT_SEED)?;
    // let signer = Keypair::from_phrase(&mnemonic, None)?;
    let signer = dev::alice();
    let signer_id = signer.public_key().into();
    let mut nonce = client.tx().account_nonce(&signer_id).await?;
    loop {
        for i in 1..=NUM_CHUNKS {
            let data = vec![(i & 255) as u8; TX_SIZE];
            let _ = submit_data_with_nonce(&client, &signer, data, AppId(1), nonce).await?;
            nonce = nonce.saturating_add(One::one());
        }

        // Add a delay between iterations
        tokio::time::sleep(Duration::from_secs(DELAY_SECS)).await;
    }
}
