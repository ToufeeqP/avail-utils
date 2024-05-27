#![allow(dead_code)]
use anyhow::Result;
use avail_core::header::Header as DaHeader;
use avail_subxt::{
    api::{self, runtime_types::sp_consensus_babe::digests::PreDigest},
    AvailClient, Opts,
};
use codec::{Decode, Encode};
use hex_literal::hex;
use sp_consensus_babe::AuthorityId;
use sp_core::{ByteArray, H256};
use sp_runtime::{traits::Header, RuntimeDebug};
use structopt::StructOpt;
use subxt::config::substrate::DigestItem;

pub type BlockNumber = u32;
pub type HeaderT = DaHeader<BlockNumber, sp_runtime::traits::BlakeTwo256>;

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
                client.blocks().at(block_hash).await?
            }
        },
    };

    let header = block.header();

    let digests = header.digest.logs.clone().into_iter().collect::<Vec<_>>();
    for digest in digests {
        if let DigestItem::PreRuntime(id, data) = digest {
            if id == BABE_ENGINE_ID {
                let pre_digest: PreDigest =
                    PreDigest::decode(&mut data.as_ref()).expect("Lets see");
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
                    Some(validator) => {
                        println!("author of block #{} is {}", header.number, validator)
                    }
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

pub async fn verify_seal(block_id: Option<BlockId>) -> Result<()> {
    use sp_consensus_babe::digests::*;
    use sp_runtime::RuntimeAppPublic;

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
                client.blocks().at(block_hash).await?
            }
        },
    };

    let header = block.header();
    let verify_seal_signature = |mut header: HeaderT, signer: &AuthorityId| {
        let seal = header.digest_mut().pop()?.as_babe_seal()?;
        let pre_hash = header.hash();

        if !signer.verify(&pre_hash.as_ref(), &seal) {
            return None;
        }

        Some(())
    };

    let local_header = HeaderT::decode(&mut header.encode().as_ref()).unwrap();

    // creating public key from bytes
    let key_bytes = hex!("5eb6d4f14715a25cbb43415a559ffcf9a197e6875315d9ef438b9bbdcec9ee6b");
    let signer_key =
        AuthorityId::from_slice(&key_bytes).expect("Failed to get public key froom bytes");
    verify_seal_signature(local_header, &signer_key).unwrap();
    println!("Seal verification successful");
    Ok(())
}
