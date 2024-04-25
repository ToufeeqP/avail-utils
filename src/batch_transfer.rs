#![allow(dead_code)]
use crate::account_utils::{Account, ACCOUNT_PATH};
use anyhow::{anyhow, Result};
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
use csv::Reader;
use sp_arithmetic::traits::SaturatedConversion;
use std::{fs, str::FromStr};
use structopt::StructOpt;
use subxt_signer::{
    bip39::Mnemonic,
    sr25519::{dev, Keypair},
};
use tokio::time::Duration;

// NOTE: Update your mnemonic
const ACCT_SEED: &str = "submit drum tennis scheme worry keen gold dirt pepper walk mystery sphere";
const AVAIL: u128 = 1_000_000_000_000_000_000;
const DELAY_SECS: u64 = 20;
pub const TRANSFERS_CSV_PATH: &str = "transfers.csv";

// amount in AVAIL
pub async fn batch_transfer(amount: u128) -> Result<()> {
    // Read accounts from the file
    let accounts_json = fs::read_to_string(ACCOUNT_PATH)?;
    let accounts: Vec<Account> = serde_json::from_str(&accounts_json)?;

    let args = Opts::from_args();
    let client = AvailClient::new(args.ws).await?;

    let amount = amount
        .checked_mul(AVAIL)
        .ok_or(anyhow!("Error in conversion to AVAIL"))?;
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

// CSV structure
// address,amount
// 5FYdW32tnHE1NGumkEQ1u8tYTUbzmt7vFZQuYxxvGk7WFjaJ,17
// Reads CSV and processes batch transfers
pub async fn batch_transfer_from_csv(
    max_calls_per_batch: usize,
    retry_attempts: usize,
) -> Result<()> {
    let args = Opts::from_args();
    let client = AvailClient::new(args.ws).await?;

    // CSV reader setup
    let mut csv_reader = Reader::from_path(TRANSFERS_CSV_PATH)?;
    let mut transfers = vec![];

    // Collect all valid transfers from CSV
    for (index, record) in csv_reader.records().enumerate() {
        let record = record?;
        if record.len() != 2 {
            println!(
                "Warning: Record {} does not have 2 columns. Skipping.",
                index + 1
            );
            continue;
        }

        let address_str = &record[0];
        let amount_str = &record[1];

        let amount: u128 = amount_str.parse().map_err(|_| {
            println!("Error parsing amount for record {}. Skipping.", index + 1);
            anyhow!("Invalid amount in record {}.", index + 1)
        })?;

        let amount_in_avail = amount.checked_mul(AVAIL).ok_or(anyhow!(
            "Error in conversion to AVAIL in record {}",
            index + 1
        ))?;

        let account_id = AccountId32::from_str(address_str).map_err(|_| {
            println!(
                "Error parsing account ID for record {}. Skipping.",
                index + 1
            );
            anyhow!("Invalid address in record {}", index + 1)
        })?;

        transfers.push((account_id, amount_in_avail));
    }

    if transfers.is_empty() {
        return Err(anyhow!("No valid transfers in CSV."));
    }

    let mnemonic = Mnemonic::parse(ACCT_SEED)?;
    let signer = Keypair::from_phrase(&mnemonic, None)?;
    let signer_id = signer.public_key().into();
    let mut nonce = client.tx().account_nonce(&signer_id).await?;

    let mut successful_txs = vec![];
    let mut failed_txs = vec![];

    // Split transfers into batches of the specified size
    for (batch_counter, chunk) in transfers.chunks(max_calls_per_batch).enumerate() {
        let calls: Vec<Call> = chunk
            .iter()
            .map(|(account_id, amount)| {
                Call::Balances(BalanceCall::transfer_keep_alive {
                    dest: MultiAddress::Id(account_id.clone()),
                    value: *amount,
                })
            })
            .collect();

        // Attempt to send the batch with retry logic
        for attempt in 0..=retry_attempts {
            let batch_tx = api::tx().utility().batch_all(calls.clone());
            let tx_result = tx::send_with_nonce(&client, &batch_tx, &signer, 0, nonce).await;

            match tx_result {
                Ok(result) => {
                    let ext_hash = result.extrinsic_hash();
                    println!(
                        "Batch transfer #{} submitted with hash: {:?}",
                        batch_counter + 1,
                        ext_hash
                    );
                    successful_txs.push((batch_counter + 1, ext_hash));
                    break; // Exit retry loop on success
                }
                Err(e) => {
                    println!(
                        "Attempt #{} for batch #{} failed. Error: {:?}",
                        attempt + 1,
                        batch_counter + 1,
                        e
                    );
                    // Fetch the nonce again
                    nonce = client.tx().account_nonce(&signer_id).await?;
                    if attempt == retry_attempts {
                        println!(
                            "Exceeded maximum retry attempts for batch #{}.",
                            batch_counter + 1
                        );
                        failed_txs.push((batch_counter + 1, e));
                    }
                    // Add a delay between iterations
                    tokio::time::sleep(Duration::from_secs(DELAY_SECS)).await;
                }
            }
        }
        nonce += 1;
    }

    // Output successful and failed transactions
    if !successful_txs.is_empty() {
        println!("Successful Transactions:");
        for (batch_num, ext_hash) in &successful_txs {
            println!(" - Batch #{} with Tx Hash: {:?}", batch_num, ext_hash);
        }
    }

    if !failed_txs.is_empty() {
        println!("Failed Transactions:");
        for (batch_num, error) in &failed_txs {
            println!(" - Batch #{} Error: {:?}", batch_num, error);
        }
    }

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
