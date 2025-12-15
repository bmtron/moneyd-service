// the ingestinator
use crate::{
    quickbooks::parser::parse_ofx_with_fallback,
    utils::{
        globalutil::{get_transaction_hashes, hash_transaction_data},
        transactiontransporter::TransactionTransport,
    },
};
use serde::Deserialize;
use std::{
    collections::HashSet,
    fs::{self, File, canonicalize},
    io::Read,
    path::Path,
};
use toml;

#[derive(Deserialize, Debug)]
struct Config {
    directory: Vec<Directory>,
}
#[derive(Deserialize, Debug)]
struct Directory {
    name: String,
    path: String,
}

pub struct TransactionBatchHolder {
    pub transaction_batches: Vec<TransactionBatch>,
    pub institution_id: i32,
}

pub struct TransactionBatch {
    pub transactions: Vec<TransactionTransport>,
    pub hashes: HashSet<String>,
    pub all_transactions_exist: bool,
}

impl TransactionBatch {
    pub fn new() -> Self {
        TransactionBatch {
            transactions: Vec::new(),
            hashes: HashSet::new(),
            all_transactions_exist: false,
        }
    }
}

const HASH_PATH: &str = "./config/existing-hashes.txt";
const AMEX_INSITUTION_ID: i32 = 1;
const CITIZENS_INSTITUTION_ID: i32 = 2;
const CAPITAL_ONE_INSITUTION_ID: i32 = 3;
const APPLE_INSITUTION_ID: i32 = 4;
const CHASE_INSTITUTION_ID: i32 = 5;

pub fn ingestinator() -> Result<Vec<TransactionBatchHolder>, Box<dyn std::error::Error>> {
    // Load configuration
    let config = load_config()?;

    // Load existing file hashes
    let mut existing_hashes = get_transaction_hashes(HASH_PATH)?;

    let mut master_transaction_batch_holder: Vec<TransactionBatchHolder> = Vec::new();
    for dir in config.directory.iter() {
        let institution_id = match dir.name.as_str() {
            "amex" => AMEX_INSITUTION_ID,
            "citizens" => CITIZENS_INSTITUTION_ID,
            "capitalone" => CAPITAL_ONE_INSITUTION_ID,
            "apple" => APPLE_INSITUTION_ID,
            "chase" => CHASE_INSTITUTION_ID,
            _ => 0,
        };
        let processing_result_batch = process_directory(&dir.path, &mut existing_hashes).unwrap();
        let trans_holder = TransactionBatchHolder {
            transaction_batches: processing_result_batch,
            institution_id,
        };
        master_transaction_batch_holder.push(trans_holder);
    }

    // i think the hashes need to be separated by
    // transaction set.
    Ok(master_transaction_batch_holder)
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = Path::new("./config/moneyd-config.toml");
    let absolute_config = canonicalize(config_path)?;
    let mut config_file = File::open(absolute_config)?;
    let mut config_contents = String::new();
    config_file.read_to_string(&mut config_contents)?;

    Ok(toml::from_str(&config_contents)?)
}

fn process_directory(
    directory_path: &str,
    hash_set: &mut HashSet<String>,
) -> Result<Vec<TransactionBatch>, Box<dyn std::error::Error>> {
    let dir = fs::read_dir(directory_path)?;

    let mut batches = Vec::new();
    for entry in dir {
        let file_path = entry?
            .path()
            .canonicalize()?
            .as_os_str()
            .to_str()
            .expect("Invalid file path")
            .to_string();
        let file_name = &file_path.as_str();
        let file_content = fs::read_to_string(&file_path)?;

        let txns = parse_ofx_with_fallback(file_content.as_str(), file_name);
        let mut new_hashes: HashSet<String> = HashSet::new();
        let mut batch: TransactionBatch = TransactionBatch::new();
        // if we got this far, the parsing worked.
        // probably
        let mut txn_transports: Vec<TransactionTransport> = Vec::new();
        for txn in txns.iter() {
            let xport = txn.to_transport();
            let hashed_xport = hash_transaction_data(&xport);
            if !hash_set.contains(&hashed_xport) {
                txn_transports.push(xport);
                new_hashes.insert(hashed_xport);
            }
        }

        if new_hashes.len() == 0 {
            batch.all_transactions_exist = true;
        }
        batch.transactions = txn_transports;
        batch.hashes = new_hashes;
        batches.push(batch);
    }

    Ok(batches)
}
