#![allow(dead_code)]
use anyhow::Result;
use avail_core::header::Header as DaHeader;
use avail_subxt::{
    api::{
        self,
        runtime_types::{sp_consensus_babe::digests::PreDigest, sp_core::crypto::KeyTypeId},
    },
    AvailClient, Opts,
};
use avail_subxt::api::runtime_types::sp_consensus_slots::Slot;
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

/// Returns the slot of the pre digest.
fn slot(digest: PreDigest) -> Slot {
    match digest {
        PreDigest::Primary(primary) => primary.slot,
        PreDigest::SecondaryPlain(secondary) => secondary.slot,
        PreDigest::SecondaryVRF(secondary) => secondary.slot,
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

pub async fn verify_seal_and_session(block_id: Option<BlockId>) -> Result<()> {
    use avail_subxt::api::runtime_types::sp_consensus_babe::digests::PreDigest as AvailPreDigest;
    use sp_consensus_babe::digests::*;
    use sp_runtime::RuntimeAppPublic;

    let mut signing_auth: Option<AuthorityId> = None;
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
    let block_number = header.number;

    // Initialize variables for table output
    let mut babe_author = "No babe author".to_string();
    let mut session_author = "No session author".to_string();
    let mut seal_verification_result = "❌".to_string();
    let mut authors_match = "❌";
    let mut babe_slot: Slot = Slot(0);

    // get PreDigest
    let digests = header.digest.logs.clone();
    let digest = digests
        .first()
        .expect("Every block will have a PreDigest qed");
    if let DigestItem::PreRuntime(id, data) = digest {
        if id == &BABE_ENGINE_ID {
            let pre_digest: AvailPreDigest =
                AvailPreDigest::decode(&mut data.as_ref()).expect("Lets see");
            let auth_index = authority_index(pre_digest.clone());
            // fetch the session validators list
            let vals_query = api::storage().session().validators();
            let validators = client
                .storage()
                .at(header.parent_hash)
                .fetch(&vals_query)
                .await?
                .unwrap();
            if let Some(validator) = validators.get(auth_index as usize) {
                session_author = validator.to_string();
            }

            // fetch the babe authority list
            let babe_query = api::storage().babe().authorities();
            let babe_auths = client
                .storage()
                .at(block.hash())
                .fetch(&babe_query)
                .await?
                .unwrap()
                .0;
            if let Some((validator, _)) = babe_auths.get(auth_index as usize) {
                let key_bytes = *b"babe";
                let babe_key_type = KeyTypeId::decode(&mut key_bytes.as_ref()).unwrap();

                let key_bytes = validator.0 .0;
                let vals_query = api::storage().session().key_owner(babe_key_type, key_bytes);
                if let Some(validator_stash) = client
                    .storage()
                    .at(header.parent_hash)
                    .fetch(&vals_query)
                    .await?
                {
                    signing_auth = Some(
                        AuthorityId::from_slice(&key_bytes)
                            .expect("Should be able recover public key from bytes"),
                    );
                    babe_author = validator_stash.to_string();
                    babe_slot = slot(pre_digest);
                    // Check if session and BABE authors match
                    if babe_author == session_author {
                        authors_match = "✅";
                    }
                }
            }
        }
    }

    let verify_seal_signature = |mut header: HeaderT, signer: &AuthorityId| {
        let seal = header.digest_mut().pop()?.as_babe_seal()?;
        let pre_hash = header.hash();

        if !signer.verify(&pre_hash.as_ref(), &seal) {
            return None;
        }

        Some(())
    };

    let local_header = HeaderT::decode(&mut header.encode().as_ref()).unwrap();

    if let Some(signer_key) = signing_auth {
        if verify_seal_signature(local_header, &signer_key).is_some() {
            seal_verification_result = "✅".to_string();
        } else {
            seal_verification_result = "❌".to_string();
        }
    }

    // Print the table row
    println!(
        "{:<8} | {:<10} | {:<50} | {:<50} | {:<6} | {:<6}",
        block_number, babe_slot.0, babe_author, session_author, authors_match, seal_verification_result
    );

    Ok(())
}
