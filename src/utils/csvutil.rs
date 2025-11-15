use csv;
use serde::Deserialize;
use std::{fs::File, io::Read};

use crate::utils::globalutil::AmexData;

pub fn tst() {
    println!("called from utils.");
}

pub fn ingest_amex(file_path: String) -> Result<Vec<AmexData>, Box<dyn std::error::Error>> {
    let mut rdr = csv::Reader::from_path(file_path)?;
    let mut amex_records: Vec<AmexData> = Vec::new();
    for result in rdr.deserialize() {
        let record: AmexData = result?;
        amex_records.push(record);
    }
    Ok(amex_records)
}
