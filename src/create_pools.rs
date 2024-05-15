#![allow(dead_code)]
use anyhow::Result;
use avail_core::AppId;
use avail_subxt::{
    api::{self},
    primitives::new_params_from_app_id,
    utils::{AccountId32, MultiAddress},
    AvailClient, Opts,
};
use sp_arithmetic::traits::SaturatedConversion;
use std::str::FromStr;
use structopt::StructOpt;
use subxt_signer::{bip39::Mnemonic, sr25519::Keypair};

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
    let client = AvailClient::new(args.ws).await?;

    for seed in seeds {
        let extrinsic_params = new_params_from_app_id(AppId(0));
        let mnemonic = Mnemonic::parse(seed)?;
        let signer = Keypair::from_phrase(&mnemonic, None)?;
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
            .sign_and_submit(&tx, &signer, extrinsic_params)
            .await?;

        println!("Pool creation completed with hash: {:?}", h);
    }

    Ok(())
}
