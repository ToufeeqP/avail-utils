#![allow(dead_code)]
use crate::account_utils::{Account, ACCOUNT_PATH};
use anyhow::Result;
use avail_subxt::{
    api::{
        runtime_types::pallet_balances::pallet::Call as BalanceCall,
        {self},
    },
    primitives::new_params_from_app_id,
    tx,
    utils::{AccountId32, MultiAddress},
    AvailClient, Call, Opts,
};
use sp_arithmetic::traits::SaturatedConversion;
use std::{fs, str::FromStr};
use structopt::StructOpt;
use subxt_signer::{
    bip39::Mnemonic,
    sr25519::{dev, Keypair},
};

const ACCT_SEED: &str =
    "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice";

pub async fn batch_transfer(amount: u128) -> Result<()> {
    // Read accounts from the file
    let accounts_json = fs::read_to_string(ACCOUNT_PATH)?;
    let accounts: Vec<Account> = serde_json::from_str(&accounts_json)?;

    let args = Opts::from_args();
    let client = AvailClient::new(args.ws).await?;

    // let signer = dev::bob();
    let mnemonic = Mnemonic::parse(ACCT_SEED)?;
    let signer = Keypair::from_phrase(&mnemonic, None)?;

    let calls: Result<Vec<_>> = accounts
        .iter()
        .map(|account| {
            let account_id = AccountId32::from_str(&account.address)?;
            Ok(Call::Balances(BalanceCall::transfer_keep_alive {
                dest: MultiAddress::Id(account_id.clone()),
                value: amount.saturated_into(),
            }))
        })
        .collect();

    let calls = calls?;

    let batch_tx = api::tx().utility().batch(calls);
    let h = tx::send_then_in_block(&client, &batch_tx, &signer, 0)
        .await?
        .block_hash();
    println!("Batch transfer completed with hash: {:?}", h);

    Ok(())
}

/// Transfer given amount from  above seed to all the accounts without batching
pub async fn individual_transfers(amount: u64) -> Result<()> {
    // Read accounts from the file
    let accounts_json = fs::read_to_string(ACCOUNT_PATH)?;
    let accounts: Vec<Account> = serde_json::from_str(&accounts_json)?;

    let args = Opts::from_args();
    let client = AvailClient::new(args.ws).await?;

    let signer = dev::bob();

    for account in accounts {
        let account_id = AccountId32::from_str(&account.address)?;

        // TODO: implement clone on ExtrinsicParams in avail-subxt and use it outside loop
        let extrinsic_params = new_params_from_app_id(0);
        let tx = api::tx().balances().transfer_keep_alive(
            MultiAddress::Id(account_id.clone()),
            amount.saturated_into(),
        );
        let h = client
            .tx()
            .sign_and_submit(&tx, &signer, extrinsic_params)
            .await?;

        println!(
            "Transfer completed for account {:?} with hash: {:?}",
            account_id, h
        );
    }

    Ok(())
}
