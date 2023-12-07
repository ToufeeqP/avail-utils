use anyhow::Result;
use avail_subxt::{api, build_client, Opts};
use structopt::StructOpt;
use sp_arithmetic::Perbill;
use prettytable::{row, Table};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let start_era = 28;
	let end_era = 29;

	let args = Opts::from_args();
	let client = build_client(args.ws, args.validate_codegen).await?;

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
        table.add_row(row!["Validator AccountId", "Reward Points", "Rewards", "Rewards %"]);

        for (validator_account_id, reward_points) in era_validators {
            // Calculate rewards percent
            let rewards_percent = Perbill::from_rational(reward_points, era_total_points);

            // Calculate rewards
            let rewards = rewards_percent * era_rewards;

            // Add row to the table
            table.add_row(row![
                validator_account_id,
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
