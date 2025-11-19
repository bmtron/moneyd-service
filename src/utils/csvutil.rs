use csv;

use crate::utils::{
    globalutil::{AmexData, CitizensData, ConvertToTransaction},
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
    let mut count = 0;
    for result in rdr.records() {
        let str_rec = result;
        if count > 0 {
            let record: AmexData = str_rec.unwrap().deserialize(None).unwrap();
            let txn = record.to_txn();
            amex_records.push(txn);
        }
        count += 1;
    }
    Ok(amex_records)
}

pub fn ingest_citizens(
    file_path: String,
) -> Result<Vec<TransactionTransport>, Box<dyn std::error::Error>> {
    let mut rdr = csv::ReaderBuilder::new().from_path(file_path)?;
    let mut citizens_records: Vec<TransactionTransport> = Vec::new();

    // Citizens has annoying headers, just skip the first row.
    let mut count = 0;
    for result in rdr.records() {
        let str_rec = result;
        if count > 0 {
            let record: CitizensData = str_rec.unwrap().deserialize(None).unwrap();
            let txn = record.to_txn();
            citizens_records.push(txn);
        }
        count += 1;
    }

    Ok(citizens_records)
}
