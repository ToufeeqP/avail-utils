use crate::block_author::BlockId;
use anyhow::Result;
use avail_subxt::Opts;
use codec::{Decode, Encode};
use sp_consensus_babe::{digests::PreDigest, AuthorityId};
use sp_core::{
    crypto::{Ss58AddressFormat, Ss58Codec},
    ByteArray,
};
use sp_runtime::{generic::Header, traits::Header as OtherHeader, AccountId32};
use structopt::StructOpt;
use subxt::{
    backend::{legacy::LegacyRpcMethods, rpc::RpcClient},
    config::substrate::DigestItem,
    OnlineClient, PolkadotConfig,
};

pub type BlockNumber = u32;
pub type HeaderT = Header<BlockNumber, sp_runtime::traits::BlakeTwo256>;

// Generate an interface that we can use from the node's metadata.
#[subxt::subxt(runtime_metadata_path = "./polkadot_metadata_full.scale")]
pub mod polkadot {}

/// The `ConsensusEngineId` of BABE.
pub const BABE_ENGINE_ID: [u8; 4] = *b"BABE";

pub async fn verify_seal_and_session(block_id: Option<BlockId>) -> Result<()> {
    use crate::polkadot_utils::polkadot::runtime_types::sp_core::crypto::KeyTypeId;
    use sp_consensus_babe::digests::*;
    use sp_runtime::RuntimeAppPublic;

    let mut signing_auth: Option<AuthorityId> = None;
    // First, create a raw RPC client:
    let args = Opts::from_args();
    let rpc_client = RpcClient::from_url(args.ws).await?;

    // Use this to construct our RPC methods:
    let rpc = LegacyRpcMethods::<PolkadotConfig>::new(rpc_client.clone());
    let client = OnlineClient::<PolkadotConfig>::from_rpc_client(rpc_client.clone()).await?;

    let block = match block_id {
        None => client.blocks().at_latest().await?,
        Some(id) => match id {
            BlockId::Hash(block_hash) => client.blocks().at(block_hash).await?,
            BlockId::Number(block_number) => {
                let block_hash = rpc
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

    // get PreDigest
    let digests = header.digest.logs.clone();
    let digest = digests
        .first()
        .expect("Every block will have a PreDigest qed");
    if let DigestItem::PreRuntime(id, data) = digest {
        if id == &BABE_ENGINE_ID {
            let pre_digest: PreDigest = PreDigest::decode(&mut data.as_ref()).expect("Lets see");
            let auth_index = authority_index(pre_digest);
            // fetch the session validators list
            let vals_query = polkadot::storage().session().validators();
            let validators = client
                .storage()
                .at(header.parent_hash)
                .fetch(&vals_query)
                .await?
                .unwrap();
            if let Some(validator) = validators.get(auth_index as usize) {
                let session_account: AccountId32 =
                    AccountId32::from_slice(validator.as_ref()).expect("Should be able recover");
                session_author =
                    session_account.to_ss58check_with_version(Ss58AddressFormat::custom(0));
                // session_author = validator.to_string();
            }

            // fetch the babe authority list
            let babe_query = polkadot::storage().babe().authorities();
            let babe_auths = client
                .storage()
                .at(block.hash())
                .fetch(&babe_query)
                .await?
                .unwrap()
                .0;
            if let Some((validator, _)) = babe_auths.get(auth_index as usize) {
                let raw_bytes = *b"babe";
                let babe_key_type = KeyTypeId::decode(&mut raw_bytes.as_ref()).unwrap();

                let key_bytes = validator.0 .0;
                let vals_query = polkadot::storage()
                    .session()
                    .key_owner(babe_key_type, key_bytes);
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
                    let babe_account: AccountId32 =
                        AccountId32::from_slice(validator_stash.as_ref())
                            .expect("Should be able recover");
                    babe_author =
                        babe_account.to_ss58check_with_version(Ss58AddressFormat::custom(0));

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
        "{:<8} | {:<50} | {:<50} | {:<6} | {:<6}",
        block_number, babe_author, session_author, authors_match, seal_verification_result
    );

    Ok(())
}

fn authority_index(digest: PreDigest) -> u32 {
    match digest {
        PreDigest::Primary(primary) => primary.authority_index,
        PreDigest::SecondaryPlain(secondary) => secondary.authority_index,
        PreDigest::SecondaryVRF(secondary) => secondary.authority_index,
    }
}
