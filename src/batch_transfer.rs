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
use std::str::FromStr;
use structopt::StructOpt;
use subxt::{
    ext::sp_core::{sr25519::Pair, Pair as _},
    tx::PairSigner,
};

const ACCT_SEED: &str =
    "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice";

pub async fn batch_transfer() -> Result<()> {
    let recipients = [
        (
            "5HostXbk5r1smXVvYUYVNiQCsg39NqRwix4FdVVFLB5UdTSz",
            1_000_000_000_000_000_000u64,
        ),
        (
            "5ESFX6vCcR4vRieHdPYEUi3MKDyNs9FXhhHWgYhJErCoeas7",
            1_000_000_000_000_000_000u64,
        ),
        (
            "5FeZ9qQcknWLexEuo3nXXERcXNzmxQDgF4kG5e2foDCXvXwz",
            1_000_000_000_000_000_000u64,
        ),
        (
            "5Fuedf79TqB6mMWzhu8aazzfPX1mawedb7rLuHpv6iYK2Z6c",
            1_000_000_000_000_000_000u64,
        ),
        (
            "5EFTSpRN2nMZDLjkniBYdmMxquMNm5CLVsrX2V3HHue6QFFF",
            1_000_000_000_000_000_000u64,
        ),
        (
            "5GzpMhmk7o4WfJzmm5DxGTmiioTScQbeUdC2Que5ZfnH511g",
            1_000_000_000_000_000_000u64,
        ),
        (
            "5F7ckS72TB3ftao2BMWxM2YcAAM9WD7tGnwbXWreUtn2t42f",
            1_000_000_000_000_000_000u64,
        ),
    ];

    let args = Opts::from_args();
    let client = build_client(args.ws, false).await?;

    let extrinsic_params = AvailExtrinsicParams::new_with_app_id(0.into());
    let pair_a = Pair::from_string_with_seed(ACCT_SEED, None)?;
    let signer = PairSigner::<AvailConfig, Pair>::new(pair_a.0);

    let calls: Result<Vec<_>> = recipients
        .iter()
        .map(|(recipient, amt)| {
            let account_id = AccountId32::from_str(recipient)?;
            Ok(Call::Balances(BalanceCall::transfer_keep_alive {
                dest: MultiAddress::Id(account_id.clone()),
                value: (*amt).saturated_into(),
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
