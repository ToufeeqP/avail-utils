use avail_subxt::{avail, build_client, submit_data, AvailConfig, Opts};
use structopt::StructOpt;
use subxt::{
    ext::sp_core::{sr25519::Pair, Pair as _},
    tx::PairSigner,
};
use tokio::time::Duration;

// Note: padded_block_length is the limiting factor for AVAIL chain, even though BlockLength may be 5MB or higher, padded_block_length needs to be considered to check how much length a block can have.
const BLOCK_SIZE: usize = 2 * 1024 * 1024;
const TX_SIZE: usize = 100 * 1024;
const NUM_CHUNKS: usize = BLOCK_SIZE / TX_SIZE;
const DELAY_SECS: u64 = 20;

const ACCT_SEED: &str =
    "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice";

pub async fn submit_txs() -> anyhow::Result<()> {
    let args = Opts::from_args();
    let (client, _) = build_client(args.ws, args.validate_codegen).await?;

    println!(
        "{} txs of size {} will be submitted in every {} seconds",
        NUM_CHUNKS, TX_SIZE, DELAY_SECS
    );
    let pair_a = Pair::from_string_with_seed(ACCT_SEED, None)?;
    let signer = PairSigner::<AvailConfig, avail::Pair>::new(pair_a.0);

    loop {
        for i in 1..=NUM_CHUNKS {
            let data = vec![(i & 255) as u8; TX_SIZE];
            let _h = submit_data(&client, &signer, data, 1).await?;
            // println!("hash #{i}: {:?}", h);
        }

        // Add a delay between iterations
        tokio::time::sleep(Duration::from_secs(DELAY_SECS)).await;
    }

}
