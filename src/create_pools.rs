#![allow(dead_code)]
use anyhow::Result;
use avail_subxt::{
    api, build_client,
    primitives::AvailExtrinsicParams,
    utils::{AccountId32, MultiAddress},
    AvailConfig, Opts,
};
use sp_arithmetic::traits::SaturatedConversion;
use std::str::FromStr;
use structopt::StructOpt;
use subxt::{
    ext::sp_core::{sr25519::Pair, Pair as _},
    tx::PairSigner,
};

pub async fn create_pools() -> Result<()> {
    let seeds = vec![
        "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice",
        "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Bob",
        "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Charlie",
        "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Dave",
        "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Eve",
        "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Ferdie",
    ];
    let args = Opts::from_args();
    let client = build_client(args.ws, false).await?;

    let extrinsic_params = AvailExtrinsicParams::new_with_app_id(0.into());
    for seed in seeds {
        let pair_a = Pair::from_string_with_seed(seed, None)?;
        let signer = PairSigner::<AvailConfig, Pair>::new(pair_a.0);
        let account_id = AccountId32::from_str("5HpG9w8EBLe5XCrbczpwq5TSXvedjrBGCwqxK1iQ7qUsSWFc")?;
        let multi_address = MultiAddress::Id(account_id);
        let tx = api::tx().nomination_pools().create(
            10_000_000_000_000_000_000u64.saturated_into(),
            multi_address.clone(),
            multi_address.clone(),
            multi_address.clone(),
        );
        let h = client
            .tx()
            .sign_and_submit(&tx, &signer, extrinsic_params.clone())
            .await?;

        println!("Pool creation completed with hash: {:?}", h);
    }

    Ok(())
}
