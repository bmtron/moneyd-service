// the ingestinator

use std::{
    collections::HashMap,
    fs::{self, File, canonicalize},
    io::Read,
    path::Path,
};

// scan pre-designated folders for files
// grab files
// hash and compare to list of processed files
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use toml;

use crate::utils::{
    csvutil::{ingest_amex, ingest_citizens},
    transactiontransporter::TransactionTransport,
};

#[derive(Deserialize, Debug)]
struct Config {
    folders: Folders,
}
#[derive(Deserialize, Debug)]
struct Folders {
    amex: String,
    citizens: String,
    apple: String,
    chase: String,
}

#[derive(Deserialize, Serialize)]
pub struct FileHashData {
    pub hash: String,
    pub path: String,
}
pub struct IngestionResult {
    pub amex_ingestion: Vec<Vec<TransactionTransport>>,
    pub citizens_ingestion: Vec<Vec<TransactionTransport>>,
}

pub struct IngestinatorOutput {
    pub ingestion_result: IngestionResult,
    pub file_hash_data: Vec<FileHashData>,
}

pub fn ingestinator() -> Result<IngestinatorOutput, Box<dyn std::error::Error>> {
    // Load configuration
    let config = load_config()?;

    // Load existing file hashes
    let (hash_list, mut file_hash_data) = get_hash_list()?;

    // Process files from both directories
    let mut amex_transactions = Vec::new();
    let mut citizens_transactions = Vec::new();

    process_directory(
        &config.folders.amex,
        &hash_list,
        &mut file_hash_data,
        &mut amex_transactions,
        process_amex_file,
    )?;

    process_directory(
        &config.folders.citizens,
        &hash_list,
        &mut file_hash_data,
        &mut citizens_transactions,
        process_citizens_file,
    )?;

    // Return final result
    let ingestion_result = IngestionResult {
        amex_ingestion: amex_transactions,
        citizens_ingestion: citizens_transactions,
    };

    Ok(IngestinatorOutput {
        ingestion_result,
        file_hash_data,
    })
}

fn load_config() -> Result<Config, Box<dyn std::error::Error>> {
    let config_path = Path::new("./config/moneyd-config.toml");
    let absolute_config = canonicalize(config_path)?;
    let mut config_file = File::open(absolute_config)?;
    let mut config_contents = String::new();
    config_file.read_to_string(&mut config_contents)?;

    Ok(toml::from_str(&config_contents)?)
}

fn get_hash_list()
-> Result<(HashMap<String, String>, Vec<FileHashData>), Box<dyn std::error::Error>> {
    let hash_json_path = Path::new("./config/consumed-files.json");
    let absolute_hash_path = canonicalize(hash_json_path)?;
    let mut hash_json = File::open(absolute_hash_path)?;
    let mut hash_contents = String::new();
    hash_json.read_to_string(&mut hash_contents)?;

    let file_hashes: Vec<FileHashData> = serde_json::from_str(&hash_contents)?;

    let mut file_hashes_map: HashMap<String, String> = HashMap::new();

    for hash in &file_hashes {
        file_hashes_map.insert(hash.hash.clone(), hash.path.clone());
    }

    Ok((file_hashes_map, file_hashes))
}

fn process_directory<F>(
    directory_path: &str,
    hash_list: &HashMap<String, String>,
    file_hash_data: &mut Vec<FileHashData>,
    transactions: &mut Vec<Vec<TransactionTransport>>,
    processor: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    F: Fn(String) -> Result<Vec<TransactionTransport>, Box<dyn std::error::Error>>,
{
    let dir = fs::read_dir(directory_path)?;

    for entry in dir {
        let file_path = entry?
            .path()
            .canonicalize()?
            .as_os_str()
            .to_str()
            .expect("Invalid file path")
            .to_string();

        let file_hash = hash_csv_data(&file_path)?;

        if !hash_list.contains_key(&file_hash) {
            let new_hash_data = FileHashData {
                hash: file_hash.clone(),
                path: file_path.clone(),
            };

            let txns = processor(file_path)?;
            transactions.push(txns);
            file_hash_data.push(new_hash_data);
        }
    }

    Ok(())
}

fn process_amex_file(
    file_path: String,
) -> Result<Vec<TransactionTransport>, Box<dyn std::error::Error>> {
    Ok(ingest_amex(file_path)?)
}

fn process_citizens_file(
    file_path: String,
) -> Result<Vec<TransactionTransport>, Box<dyn std::error::Error>> {
    Ok(ingest_citizens(file_path)?)
}

fn hash_csv_data(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

// TODO: remove this code (pre-refactor)
// saving it for now as a reference in case something breaks or whatever
// pub fn ingestinator() -> std::io::Result<IngestinatorOutput> {
//     let config_path = Path::new("./config/moneyd-config.toml");
//     let absolute_config = fs::canonicalize(config_path)?;
//     let mut config_file = File::open(absolute_config)?;
//     let mut config_contents = String::new();
//     config_file.read_to_string(&mut config_contents)?;
//     let hash_list_result = get_hash_list().unwrap();
//     let hash_list = hash_list_result.0;
//     let mut file_hash_data = hash_list_result.1;

//     let config: Config = toml::from_str(&config_contents.as_str()).unwrap();

//     let amex_dir = fs::read_dir(config.folders.amex).unwrap();
//     let citizens_dir = fs::read_dir(config.folders.citizens).unwrap();

//     let mut citizens_transactions: Vec<Vec<TransactionTransport>> = Vec::new();
//     let mut amex_transactions: Vec<Vec<TransactionTransport>> = Vec::new();

//     for path in amex_dir {
//         let file_path = path
//             .unwrap()
//             .path()
//             .canonicalize()
//             .unwrap()
//             .as_os_str()
//             .to_str()
//             .expect("Invalid file path: amex")
//             .to_string();

//         let file_hash = hash_csv_data(file_path.as_str()).unwrap();
//         if !hash_list.contains_key(&file_hash) {
//             // get file data
//             // hash file data
//             // if hash exists -> skip, else add hash to file, ingest file, write file on successful ingestion
//             let new_hash_data: FileHashData = FileHashData {
//                 hash: file_hash.clone(),
//                 path: file_path.clone(),
//             };
//             let amex_txns = ingest_amex(file_path).unwrap();
//             amex_transactions.push(amex_txns);
//             file_hash_data.push(new_hash_data);
//         }
//     }

//     for path in citizens_dir {
//         let file_path = path
//             .unwrap()
//             .path()
//             .canonicalize()
//             .unwrap()
//             .as_os_str()
//             .to_str()
//             .expect("Invalid file path: citizens")
//             .to_string();

//         let file_hash = hash_csv_data(file_path.as_str()).unwrap();
//         if !hash_list.contains_key(&file_hash) {
//             // get file data
//             // hash file data
//             // if hash exists -> skip, else add hash to file, ingest file, write file on successful ingestion
//             let new_hash_data: FileHashData = FileHashData {
//                 hash: file_hash.clone(),
//                 path: file_path.clone(),
//             };
//             let citizens_txns = ingest_citizens(file_path).unwrap();
//             citizens_transactions.push(citizens_txns);
//             // push after ingestion
//             file_hash_data.push(new_hash_data);
//         }
//     }

//     let ingestion_result: IngestionResult = IngestionResult {
//         amex_ingestion: amex_transactions,
//         citizens_ingestion: citizens_transactions,
//     };
//     let ingestinator_output: IngestinatorOutput = IngestinatorOutput {
//         ingestion_result: ingestion_result,
//         file_hash_data: file_hash_data,
//     };
//     Ok(ingestinator_output)
// }

// fn get_hash_list()
// -> Result<(HashMap<String, String>, Vec<FileHashData>), Box<dyn std::error::Error>> {
//     let hash_json_path = Path::new("./config/consumed-files.json");
//     let absolute_hash_path = canonicalize(hash_json_path).unwrap();
//     let mut hash_json = File::open(absolute_hash_path).unwrap();
//     let mut hash_contents = String::new();
//     hash_json.read_to_string(&mut hash_contents)?;

//     let file_hashes: Vec<FileHashData> = serde_json::from_str(hash_contents.as_str()).unwrap();
//     let mut file_hashes_map: HashMap<String, String> = HashMap::new();
//     for hash in &file_hashes {
//         file_hashes_map.insert(hash.hash.clone(), hash.path.clone());
//     }
//     Ok((file_hashes_map, file_hashes))
// }

// fn hash_csv_data(path: &str) -> Result<String, Box<dyn std::error::Error>> {
//     let mut file = File::open(path)?;
//     let mut hasher = Sha256::new();
//     let mut buffer = [0; 1024];

//     loop {
//         let bytes_read = file.read(&mut buffer)?;
//         if bytes_read == 0 {
//             break;
//         }
//         hasher.update(&buffer[..bytes_read]);
//     }

//     let result = hasher.finalize();
//     Ok(format!("{:x}", result))
// }
