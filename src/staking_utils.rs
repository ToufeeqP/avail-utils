#![allow(dead_code)]
use crate::account_utils::{Account, ACCOUNT_PATH};
use anyhow::Result;
use avail_subxt::{
    api::{self, runtime_types::pallet_staking::pallet::pallet::Call as StakingCall},
    build_client,
    primitives::AvailExtrinsicParams,
    utils::{AccountId32, MultiAddress},
    AvailConfig, Call, Opts,
};
use serde::{Deserialize, Serialize};
use sp_arithmetic::traits::SaturatedConversion;
use std::fs;
use std::str::FromStr;
use structopt::StructOpt;
use subxt::{
    ext::sp_core::{sr25519::Pair, Pair as _},
    tx::PairSigner,
};

#[derive(Serialize, Deserialize)]
struct Validator {
    address: String,
}

pub async fn bond_and_nominate_batch(amount: u64) -> Result<()> {
    // Read accounts from the accounts.json file
    let accounts_json = fs::read_to_string(ACCOUNT_PATH)?;
    let accounts: Vec<Account> = serde_json::from_str(&accounts_json)?;

    // Read validators from the validators.json file
    let validators_json = fs::read_to_string("validators.json")?;
    let validators: Vec<Validator> = serde_json::from_str(&validators_json)?;

    let args = Opts::from_args();
    let (client, _) = build_client(args.ws, false).await?;

    let extrinsic_params = AvailExtrinsicParams::new_with_app_id(0.into());

    for (account, validator) in accounts.iter().zip(validators.iter()) {
        let pair = Pair::from_string_with_seed(&account.mnemonic, None)?;
        let signer = PairSigner::<AvailConfig, Pair>::new(pair.0);

        let validator_id = AccountId32::from_str(&validator.address)?;

        // Bond
        let bond_call = Call::Staking(StakingCall::bond {
            value: amount.saturated_into(),
            payee: api::runtime_types::pallet_staking::RewardDestination::Stash,
        });

        // Nominate
        let nominate_call = Call::Staking(StakingCall::nominate {
            targets: vec![MultiAddress::Id(validator_id)],
        });

        // Add calls to the batch
        let batch_tx = api::tx().utility().batch(vec![bond_call, nominate_call]);
        let h = client
            .tx()
            .sign_and_submit(&batch_tx, &signer, extrinsic_params.clone())
            .await?;
        println!("Batch operation completed with hash: {:?}", h);
    }

    Ok(())
}
