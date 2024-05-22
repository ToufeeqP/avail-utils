#![allow(dead_code)]
use anyhow::Result;
use avail_subxt::{
    api, api::runtime_types::sp_consensus_babe::digests::PreDigest, AvailClient, Opts,
};
use codec::{Decode, Encode};
use sp_core::H256;
use sp_runtime::RuntimeDebug;
use structopt::StructOpt;
use subxt::config::substrate::DigestItem;

/// The `ConsensusEngineId` of BABE.
pub const BABE_ENGINE_ID: [u8; 4] = *b"BABE";

/// Something to identify a block.
#[derive(PartialEq, Eq, Clone, Encode, Decode, RuntimeDebug)]
pub enum BlockId {
    /// Identify by block header hash.
    Hash(H256),
    /// Identify by block number.
    Number(u32),
}

pub async fn find_author(block_id: Option<BlockId>) -> Result<()> {
    let args = Opts::from_args();
    let client = AvailClient::new(args.ws).await?;

    let block = match block_id {
        None => client.blocks().at_latest().await?,
        Some(id) => match id {
            BlockId::Hash(block_hash) => client.blocks().at(block_hash).await?,
            BlockId::Number(block_number) => {
                let block_hash = client
                    .legacy_rpc()
                    .chain_get_block_hash(Some(block_number.into()))
                    .await?
                    .expect("header exist");
                let block = client.blocks().at(block_hash).await?;
                block
            }
        },
    };

    let header = block.header();

    let digests = header.digest.logs.clone().into_iter().collect::<Vec<_>>();
    for digest in digests {
        if let DigestItem::PreRuntime(id, data) = digest {
            if id == BABE_ENGINE_ID {
                let pre_digest: PreDigest = PreDigest::decode(&mut data.as_ref())
                    .ok()
                    .expect("Lets see");
                let auth_index = authority_index(pre_digest);
                // fetch the validator list
                let vals_query = api::storage().session().validators();
                let validators = client
                    .storage()
                    .at(header.parent_hash)
                    .fetch(&vals_query)
                    .await?
                    .unwrap();
                let validator = validators.get(auth_index as usize);
                match validator {
                    Some(validator) => println!(
                            "author of block #{} is {}",
                            header.number,
                            validator.to_string()
                        ),
                    None => println!("No author for block #{}", header.number,),
                }
            }
        }
    }
    Ok(())
}

fn authority_index(digest: PreDigest) -> u32 {
    match digest {
        PreDigest::Primary(primary) => primary.authority_index,
        PreDigest::SecondaryPlain(secondary) => secondary.authority_index,
        PreDigest::SecondaryVRF(secondary) => secondary.authority_index,
    }
}
