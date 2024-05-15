#![allow(dead_code)]
use avail_core::{header::Header as DaHeader, OpaqueExtrinsic};
// Note: This util requires local updation in the sc_client_db to work until we fork this crate
use sc_client_db::{ BlocksPruning, DatabaseSource, PruningMode};
use std::path::PathBuf;
use anyhow::Error;
use anyhow::anyhow;
use codec::Decode;
use hex_literal::hex;
use sc_client_api::{
    backend::Backend as ApiBackend,
    blockchain::{Backend as BlockchainBackend, HeaderBackend},
};
use sc_client_db::{Backend, Database, DbExtrinsic};
use sp_core::H256;
use sp_runtime::traits::{BlakeTwo256, Hash};
use std::sync::Arc;
// use sp_runtime::traits::{Block as BlockT, Header as HeaderT};

pub type BlockNumber = u32;
pub type Header = DaHeader<BlockNumber, BlakeTwo256>;
pub type NodeBlock = sp_runtime::generic::Block<Header, OpaqueExtrinsic>;
pub const CANONICALIZATION_DELAY: u64 = 4096;
pub const KEY_LOOKUP: u32 = 3;
pub const TRANSACTION: u32 = 11;
pub const BODY_INDEX: u32 = 12;

pub fn run() {
    // let db_config = kvdb_rocksdb::DatabaseConfig::with_columns(12);
    // let path_str = "../avail/tmp/turing/chains/avail_turing_network/paritydb/full/full";
    // let path_str = "/Users/toufeeqpasha/avail-project/avail/tmp/alice/chains/avail_development_network/paritydb/full";
    // let path_str = "/Users/toufeeqpasha/Downloads/paritydb/full";
    let path_str = "/Users/toufeeqpasha/Downloads/paritydb_full/full";
    let path = PathBuf::from(path_str);
    let db_config = sc_client_db::DatabaseSettings {
        trie_cache_maximum_size: Some(2 * 1024 * 1024 * 1024),
        state_pruning: Some(PruningMode::Constrained(sc_state_db::Constraints {
            max_blocks: Some(256),
        })),
        // state_pruning: Some(PruningMode::ArchiveAll),
        // To use rocksdb, enable rocksdb feature in sc-client-db
        // source: DatabaseSource::RocksDb {
        //     path,
        //     cache_size: 2,
        // },
        source: DatabaseSource::ParityDb { path },
        blocks_pruning: BlocksPruning::KeepAll,
    };

    let db_backend = sc_client_db::Backend::<NodeBlock>::new(db_config, CANONICALIZATION_DELAY)
        .expect("Hope it works");
    let backend: Arc<Backend<NodeBlock>> = Arc::new(db_backend);
    // let info = backend.blockchain().info();
    // println!("chain info: {:?}", info);
    // println!();
    // let block_hash = H256(hex!(
    //     "404c93c17dac274f8efdffe805bba9f79e9c01a09019675a331a3882f6feb850"
    // ));
    let block_hash = H256(hex!(
        "3b98e7a2a76784cb11b8377ea9feaeb7860f6bec1011fd3e2e708cc8d6181f51"
    ));
    // let indexed_tx = backend
    //     .blockchain()
    //     .has_indexed_transaction(block_hash)
    //     .unwrap();
    // println!("does_block_has_indexed_tx: {:?}", indexed_tx);
    // println!();
    // let indexed_body = backend
    //     .blockchain()
    //     .block_indexed_body(block_hash)
    //     .unwrap();
    // println!("indexed_body: {:x?}", indexed_body);
    // println!();

    // Following lines fetches indexed_body directly from the Db without decoding them into Extrinsics
    // This will when using block_hash as the key
    // let lookup_key = backend.blockchain().db.get(KEY_LOOKUP, block_hash.as_ref()).expect("block_hash should have a valid lookup_key");
    // println!("lookup_key: {:?}", lookup_key);
    // let index_body = backend.blockchain().db.get(BODY_INDEX, lookup_key.as_ref());
    // println!("index_body: {:?}", index_body);
    // println!();

    // let lookup_key = H256::from_slice(lookup_key.as_ref());
    // println!("lookup_key: {:?}", lookup_key);

    // blocks are keyed by number + hash.
    let lookup_key = sc_client_db::utils::number_and_hash_to_lookup_key(149709u32, block_hash)
        .expect("Hoping for good");
    // let lookup_key = sc_client_db::utils::number_and_hash_to_lookup_key(149712u32, block_hash)
    //     .expect("Hoping for good");
    // get indexed_body directly from the backend using the lookup_key
    let index_body = backend.blockchain().db.get(BODY_INDEX, lookup_key.as_ref());
    // println!("index_body: {:?}", index_body);
    // println!();

    let decoded_txs = decode_indexed_txs(index_body.unwrap(), backend.blockchain().db.clone()).unwrap();
    println!("decoded_txs: {:?}", decoded_txs.clone().unwrap()[0]);
    println!();

    let tx_hash = BlakeTwo256::hash(&decoded_txs.unwrap()[0]);
    let tx_hash = H256::from_slice(tx_hash.as_ref());
    println!("tx_hash: {:?}", tx_hash);
    println!();
    let tx = backend.blockchain().db.get(TRANSACTION, tx_hash.as_ref()).unwrap();
    println!("tx: {:?}", tx);
    println!();

    // let body = backend
    //     .blockchain()
    //     .body(block_hash)
    //     .unwrap();
    // println!("body: {:x?}", body);
    // println!();

    // let tx_hash = BlakeTwo256::hash(&indexed_body.unwrap()[0]);
    // let tx_hash = H256::from_slice(tx_hash.as_ref());
    // println!("tx_hash: {:?}", tx_hash);
    // println!();
    // let tx = backend.blockchain().db.get(TRANSACTION, tx_hash.as_ref()).unwrap();
    // println!("tx: {:?}", tx);
    // println!();
    // let tx = backend.blockchain().db.get(TRANSACTION, lookup_key.as_ref()).unwrap();
    // println!("tx: {:?}", tx);

    // let tx_hex = hex!(
    //     "590e8400722ca7c04a7d922036068f14d6f3de12f10b70fc48287b30bc6f3a889fc4114a010e475d47df81d1557fa45b69236a4f1012adf8362c0f225d70685d691f65a112d7f46a1ba6b0011eccf537fc6f8d5f8a04616044679548d9c88fc33fc952128000611800301d01a90c376232323438363137333638323233613232333933393635333133303339333233383631363533363636333436353636333033383332363233393337333133313339333633303336333633393631363336313232326332323534373837333232336135623762323234383631373336383232336132323331333633303330333336343338363536343334363333333334333836323334333833363337333633373331333336363632363333333336333933323335333232323263323234393664363136373635346434343335323233613232343634323435333134313434343433383334333933333335333733383332333433393333333033333330343634363333333333353334333733353434333833313232326332323537363536393637363837343232336133313330333032633232353036313732363136643733323233613562333032633332356437643263376232323438363137333638323233613232333736353334363336333635333836353338333133373334333436343331333833383332363633383632363133383634363433383331333733353631363236313232326332323439366436313637363534643434333532323361323234363432343533313431343434343338333433393333333533373338333233343339333333303333333034363436333333333335333433373335343433383331323232633232353736353639363736383734323233613331333033303263323235303631373236313664373332323361356233303263333235643764326337623232343836313733363832323361323233313339333036353331333233303330363233373333333433343631333736363631333733333339363433353635333833333338333333343331333033393631323232633232343936643631363736353464343433353232336132323436343234353331343134343434333833343339333333353337333833323334333933333330333333303436343633333333333533343337333534343338333132323263323235373635363936373638373432323361333133303330326332323530363137323631366437333232336135623330326333323564376435643764"
    // );
    // let tx_hash = BlakeTwo256::hash(&tx_hex);
    // println!("tx_hash: {:?}", tx_hash);
    // let tx_hash = H256::from_slice(tx_hash.as_ref());
    // println!("tx_hash: {:?}", tx_hash);
    // let tx = backend.blockchain().db.get(TRANSACTION, tx_hash.as_ref()).unwrap();
    // println!("tx: {:?}", tx);
}

fn decode_indexed_txs(
    body: Vec<u8>,
    db: Arc<dyn Database<sp_core::H256>>,
) -> Result<Option<Vec<Vec<u8>>>, Error> {
    match Vec::<DbExtrinsic<NodeBlock>>::decode(&mut &body[..]) {
        Ok(index) => {
            let mut transactions = Vec::new();
            for ex in index.into_iter() {
                println!("ext {:?}", ex);
                if let DbExtrinsic::Indexed { hash, .. } = ex {
                    match db.get(TRANSACTION, hash.as_ref()) {
                        Some(t) => transactions.push(t),
                        None => return Err(anyhow!("Missing indexed transaction {:?}", hash)),
                    }
                }
            }
            Ok(Some(transactions))
        }
        Err(err) => Err(anyhow!("Error decoding body list: {}", err)),
    }
}
