#![allow(dead_code)]
use anyhow::Error;
use anyhow::Result;
use avail_subxt::{api, build_client, Opts};
use codec::Encode;
use prettytable::{row, Table};
use serde::Serialize;
use sp_arithmetic::Perbill;
use structopt::StructOpt;
use subxt::{
    config::Header,
    ext::sp_core::{crypto::Ss58Codec, sr25519::Public},
};

pub const VALIDATOR_PATH: &str = "validators.json";

#[derive(Serialize)]
struct Address {
    address: String,
}

/// A utility to fetch last n blocks using RPC
pub async fn fetch_blocks(n: usize) -> Result<(), Error> {
    let args = Opts::from_args();
    let (client, _) = build_client(args.ws, args.validate_codegen).await?;
    let latest_header = client
        .rpc()
        .header(None)
        .await?
        .expect("Best block always exists .qed");
    let latest_block_number = latest_header.number();

    let mut tasks: Vec<tokio::task::JoinHandle<Result<(), Error>>> = Vec::new();
    for i in 0..n {
        let block_number = latest_block_number - i as u32;
        let client_clone = client.clone();
        tasks.push(tokio::spawn(async move {
            let result = async {
                let block_hash = client_clone
                    .rpc()
                    .block_hash(Some(block_number.into()))
                    .await?
                    .ok_or_else(|| Error::msg("Failed to get block hash"))?;

                let block_header = client_clone
                    .rpc()
                    .block(Some(block_hash))
                    .await?
                    .ok_or_else(|| Error::msg("Failed to get block header"))?;

                println!(
                    "Block {}: {}",
                    block_number,
                    block_header.block.header.hash()
                );
                Ok(())
            };

            result.await
        }));
    }
    // Wait for all tasks to complete
    for task in tasks {
        task.await??;
    }
    Ok(())
}

pub async fn fetch_validator_rewards(
    start_era: u32,
    end_era: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let args = Opts::from_args();
    let (client, _) = build_client(args.ws, args.validate_codegen).await?;

    for era in start_era..=end_era {
        let era_points_query = api::storage().staking().eras_reward_points(era);
        let era_rewards_query = api::storage().staking().eras_validator_reward(era);
        let era_points = client
            .storage()
            .at_latest()
            .await?
            .fetch(&era_points_query)
            .await?
            .unwrap();
        let era_rewards = client
            .storage()
            .at_latest()
            .await?
            .fetch(&era_rewards_query)
            .await?
            .unwrap();
        let era_total_points = era_points.total;
        let era_validators = era_points.individual;

        println!(
            "Era: {}, total_points: {}, total_rewards: {}",
            era, era_total_points, era_rewards
        );

        // Create the table
        let mut table = Table::new();
        table.set_format(*prettytable::format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

        // Add table header
        table.add_row(row![
            "Validator AccountId",
            "Identity",
            "Reward Points",
            "Rewards",
            "Rewards %"
        ]);

        for (validator_account_id, reward_points) in era_validators {
            let id_query = api::storage().identity().identity_of(&validator_account_id);
            let id = client.storage().at_latest().await?.fetch(&id_query).await?;

            let id_bytes = match id {
                Some(id) => id.info.display.encode()[1..].to_vec(),
                None => vec![],
            };
            let validator_id = String::from_utf8(id_bytes).unwrap_or_default();
            // Calculate rewards percent
            let rewards_percent = Perbill::from_rational(reward_points, era_total_points);

            // Calculate rewards
            let rewards = rewards_percent * era_rewards;

            // Add row to the table
            table.add_row(row![
                validator_account_id,
                &validator_id,
                reward_points,
                rewards,
                format!("{:#?}", rewards_percent)
            ]);
        }

        // Print the table
        table.printstd();
    }
    Ok(())
}

/// Fetches current validator_set fom chain & write them into required json format  
pub async fn dump_validators() -> Result<(), Box<dyn std::error::Error>> {
    let args = Opts::from_args();
    let (client, _) = build_client(args.ws, args.validate_codegen).await?;

    // Fetch validators directly as SS58-encoded addresses
    let validators = client
        .storage()
        .at_latest()
        .await?
        .fetch(&api::storage().session().validators())
        .await?
        .unwrap_or_default()
        .into_iter()
        .map(|f| Public::from_raw(*f.as_ref()).to_ss58check().to_string())
        .collect::<Vec<String>>();

    // Serialize to JSON format
    let json_data: Vec<Address> = validators
        .into_iter()
        .map(|address| Address { address })
        .collect();

    // Serialize to JSON format
    let json_string = serde_json::to_string_pretty(&json_data)?;

    // Write the JSON data to the file
    std::fs::write(VALIDATOR_PATH, json_string)?;

    println!("Validators written to file: {}", VALIDATOR_PATH);
    Ok(())
}
