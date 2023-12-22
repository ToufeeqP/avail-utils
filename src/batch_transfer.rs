#![allow(dead_code)]
use anyhow::Result;
use avail_subxt::{
    api::{self, runtime_types::pallet_balances::pallet::Call as BalanceCall},
    build_client,
    primitives::AvailExtrinsicParams,
    utils::{AccountId32, MultiAddress},
    AvailConfig, Call, Opts,
};
use sp_arithmetic::traits::SaturatedConversion;
use std::fs;
use std::str::FromStr;
use structopt::StructOpt;
use subxt::{
    ext::sp_core::{sr25519::Pair, Pair as _},
    tx::PairSigner,
};

use crate::account_utils::{Account, ACCOUNT_PATH};

const ACCT_SEED: &str =
    "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice";

pub async fn batch_transfer(amount: u64) -> Result<()> {
    // Read accounts from the file
    let accounts_json = fs::read_to_string(ACCOUNT_PATH)?;
    let accounts: Vec<Account> = serde_json::from_str(&accounts_json)?;

    let args = Opts::from_args();
    let client = build_client(args.ws, false).await?;

    let extrinsic_params = AvailExtrinsicParams::new_with_app_id(0.into());
    let pair_a = Pair::from_string_with_seed(ACCT_SEED, None)?;
    let signer = PairSigner::<AvailConfig, Pair>::new(pair_a.0);

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
    let h = client
        .tx()
        .sign_and_submit(&batch_tx, &signer, extrinsic_params.clone())
        .await?;
    println!("Batch transfer completed with hash: {:?}", h);

    Ok(())
}
