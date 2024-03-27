#![allow(dead_code)]
use crate::account_utils::{Account, ACCOUNT_PATH};
use anyhow::Result;
use avail_subxt::{
    api::{self, runtime_types::pallet_staking::pallet::pallet::Call as StakingCall, Call},
    primitives::new_params_from_app_id,
    utils::{AccountId32, MultiAddress},
    AvailClient, Opts,
};
use serde::{Deserialize, Serialize};
use sp_arithmetic::traits::SaturatedConversion;
use std::{fs, str::FromStr};
use structopt::StructOpt;
use subxt_signer::{bip39::Mnemonic, sr25519::Keypair};

#[derive(Serialize, Deserialize)]
struct Validator {
    address: String,
}

pub async fn bond_and_nominate_batch(amount: u128) -> Result<()> {
    // Read accounts from the accounts.json file
    let accounts_json = fs::read_to_string(ACCOUNT_PATH)?;
    let accounts: Vec<Account> = serde_json::from_str(&accounts_json)?;

    // Read validators from the validators.json file
    let validators_json = fs::read_to_string("validators.json")?;
    let validators: Vec<Validator> = serde_json::from_str(&validators_json)?;

    let args = Opts::from_args();
    let client = AvailClient::new(args.ws).await?;

    for (account, validator) in accounts.iter().zip(validators.iter()) {
        let extrinsic_params = new_params_from_app_id(0);

        let mnemonic = Mnemonic::parse(&account.mnemonic)?;
        let signer = Keypair::from_phrase(&mnemonic, None)?;

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
            .sign_and_submit(&batch_tx, &signer, extrinsic_params)
            .await?;
        println!("Batch operation completed with hash: {:?}", h);
    }

    Ok(())
}
