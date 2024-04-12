#![allow(dead_code)]
use anyhow::Result;
use avail_subxt::{
    api::{
        runtime_types::frame_system::pallet::Call as SystemCall,
        {self},
    },
    tx,
    AvailClient, Call, Opts,
};
use structopt::StructOpt;
use subxt_signer::{
    // bip39::Mnemonic,
    sr25519::dev,
    // sr25519::Keypair,
};

const ACCT_SEED: &str =
    "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice";

pub async fn batch_submit() -> Result<()> {
    let args = Opts::from_args();
    let client = AvailClient::new(args.ws).await?;

    // let mnemonic = Mnemonic::parse(ACCT_SEED)?;
    // let signer = Keypair::from_phrase(&mnemonic, None)?;
    let signer = dev::bob();
    let signer_id = signer.public_key().into();
    let nonce = client.tx().account_nonce(&signer_id).await?;

    let mut calls: Vec<_> = vec![];
    for _ in 0..8000 {
        calls.push(Call::System(SystemCall::remark_with_event {
            remark: b"H".to_vec(),
        }))
    }

    let batch_tx = api::tx().utility().batch(calls);
    tx::send_with_nonce(&client, &batch_tx, &signer, 0, nonce).await?;
    tx::send_with_nonce(&client, &batch_tx, &signer, 0, nonce + 1 as u64).await?;
    tx::send_with_nonce(&client, &batch_tx, &signer, 0, nonce + 2 as u64).await?;
    tx::send_with_nonce(&client, &batch_tx, &signer, 0, nonce + 3 as u64).await?;
    tx::send_with_nonce(&client, &batch_tx, &signer, 0, nonce + 4 as u64).await?;
    tx::send_with_nonce(&client, &batch_tx, &signer, 0, nonce + 5 as u64).await?;
    tx::send_with_nonce(&client, &batch_tx, &signer, 0, nonce + 6 as u64).await?;
    tx::send_with_nonce(&client, &batch_tx, &signer, 0, nonce + 7 as u64).await?;
    tx::send_with_nonce(&client, &batch_tx, &signer, 0, nonce + 8 as u64).await?;

    Ok(())
}
