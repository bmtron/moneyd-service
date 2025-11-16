use csv;
use serde::Deserialize;
use std::{fs::File, io::Read};

use crate::utils::{
    globalutil::{AmexData, ConvertToTransaction},
    transactiontransporter::TransactionTransport,
};

pub fn tst() {
    println!("called from utils.");
}

pub fn ingest_amex(
    file_path: String,
) -> Result<Vec<TransactionTransport>, Box<dyn std::error::Error>> {
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut amex_records: Vec<TransactionTransport> = Vec::new();
    for result in rdr.deserialize() {
        let record: AmexData = result?;
        let txn = record.to_txn();
        amex_records.push(txn);
    }
    Ok(amex_records)
}
