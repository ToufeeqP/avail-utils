#![allow(dead_code)]
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{self};
use subxt::ext::sp_core::{crypto::Ss58Codec, sr25519, Pair as _};

pub const ACCOUNT_PATH: &str = "accounts.json";

#[derive(Serialize, Deserialize)]
pub struct Account {
    pub address: String,
    pub mnemonic: String,
}

pub fn generate_accounts(n: u32) -> Result<(), Box<dyn std::error::Error>> {
    let mut accounts: Vec<Account> = Vec::new();

    // Generate and store accounts
    for _ in 0..n {
        let (private, mnemonic, _) = sr25519::Pair::generate_with_phrase(None);
        let address = private.public().to_ss58check();

        // Store account information in the vector
        accounts.push(Account { address, mnemonic });
    }
    write_accounts_to_json(&accounts)?;
    println!("{n} accounts generated & written to file: {}", ACCOUNT_PATH);
    Ok(())
}

fn write_accounts_to_json(accounts: &Vec<Account>) -> io::Result<()> {
    // Open the file in write mode
    let _file = File::create(ACCOUNT_PATH)?;

    // Serialize the accounts vector to JSON
    let json_data = serde_json::to_string_pretty(accounts)?;

    // Write the JSON data to the file
    std::fs::write(ACCOUNT_PATH, json_data)?;

    Ok(())
}
