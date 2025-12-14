use crate::Env;
use crate::ingestion::TransactionBatchHolder;
use crate::service::statementservice::create_statement;
use crate::utils::logintransporter::LoginResponse;
use crate::utils::statementtransporter::StatementTransport;
use crate::{
    service::transactionservice::create_transactions,
    utils::transactiontransporter::TransactionTransport,
};
use chrono::{DateTime, TimeZone, Utc};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::{env, fs};

const HASH_PATH: &str = "./config/existing-hashes.txt";

pub fn parse_ofx_date(date_str: &str) -> Result<String, Box<dyn std::error::Error>> {
    if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%Y%m%d%H%M%S") {
        let dt_utc = Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap());
        return Ok(dt_utc.to_rfc3339());
    }

    // amex adds weird timezone nonsense to the end
    // if the first parse failed then we know this should work
    let end = date_str
        .find('.')
        .or_else(|| date_str.find('['))
        .unwrap_or(date_str.len());
    let dt_str = &date_str[..end]; // "20251211000000"

    if let Ok(dt) = chrono::NaiveDate::parse_from_str(dt_str, "%Y%m%d%H%M%S") {
        let dt_utc = Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap());
        return Ok(dt_utc.to_rfc3339());
    }

    // If parsing fails, return original string
    Ok(date_str.to_string())
}
pub fn parse_and_format_date(date_str: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Try to parse as ISO 8601 format
    if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
        return Ok(dt.to_rfc3339());
    }

    // Try to parse as yyyy-MM-dd format
    if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
        // Convert to datetime at midnight UTC
        let dt_utc = Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap());
        return Ok(dt_utc.to_rfc3339());
    }

    // Try to parse as MM/dd/yyyy format
    if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%m/%d/%Y") {
        let dt_utc = Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap());
        return Ok(dt_utc.to_rfc3339());
    }

    if let Ok(dt) = chrono::NaiveDate::parse_from_str(date_str, "%Y%m%d%H%M%S") {
        let dt_utc = Utc.from_utc_datetime(&dt.and_hms_opt(0, 0, 0).unwrap());
        return Ok(dt_utc.to_rfc3339());
    }

    // If parsing fails, return original string
    Ok(date_str.to_string())
}
pub struct AuthorizationData {
    pub auth_token: String,
    pub api_key: String,
}

pub fn get_env_vars() -> Env {
    let mut api_key: Option<String> = None;
    let mut base_url: Option<String> = None;
    for (key, value) in env::vars() {
        if key.eq("API_KEY") {
            api_key = Some(value);
        } else if key.eq("BASE_URL") {
            base_url = Some(value);
        }
    }
    let envs: Env = Env { api_key, base_url };
    envs
}
pub async fn post_statements_and_transactions(
    mut transaction_batch_data: Vec<TransactionBatchHolder>,
    login_data: &LoginResponse,
    auth_data: &AuthorizationData,
) -> Result<(), Box<dyn std::error::Error>> {
    for institution_batch_holder in transaction_batch_data.iter_mut() {
        let statement_data: StatementTransport = StatementTransport {
            banking_user_id: login_data.user.id,
            institution_id: institution_batch_holder.institution_id,
            // TODO: gotta figure out these date formats from the new file
            // deal with it later. can't be too hard...
            period_start: parse_and_format_date("2025-11-01").unwrap(),
            period_end: parse_and_format_date("2025-11-17").unwrap(),
        };

        for batch in institution_batch_holder.transaction_batches.iter_mut() {
            if !batch.all_transactions_exist {
                let stmt = create_statement(&statement_data, &auth_data).await.unwrap();
                for t in batch.transactions.iter_mut() {
                    t.statement_id = Some(stmt.statement_id);
                }
                let txn_batch_result =
                    create_transactions(&batch.transactions, stmt.statement_id, &auth_data).await;
                match txn_batch_result {
                    Ok(_) => {
                        add_multiple_hashes(HASH_PATH, &batch.hashes);
                    }
                    _ => {}
                }
            }
        }
    }
    Ok(())
}

// "./config/existing-hashes.txt"
pub fn hash_transaction_data(txn: &TransactionTransport) -> String {
    let mut hasher = Sha256::new();

    // Add each string property
    hasher.update(txn.description.as_bytes());
    hasher.update(txn.transaction_date.as_bytes());
    // Add more string fields as needed

    // Optionally, add integer fields as bytes
    hasher.update(&txn.amount.to_le_bytes());
    hasher.update(&txn.refnum.as_bytes());

    // Finalize and format as hex
    let result = hasher.finalize();
    format!("{:x}", result)
}

pub fn get_transaction_hashes(path: &str) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let hashes: HashSet<String> = fs::read_to_string(path)?
        .lines()
        .map(|s| s.to_string())
        .collect();

    Ok(hashes)
}
pub fn add_multiple_hashes(
    path: &str,
    new_hashes: &HashSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut existing_hashes = get_transaction_hashes(path).unwrap();

    for hash in new_hashes.iter() {
        if !existing_hashes.contains(hash) {
            existing_hashes.insert(hash.to_string());
        }
    }

    let hash_string = existing_hashes
        .iter()
        .map(|s| s.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    fs::write(path, hash_string)?;
    Ok(())
}
pub fn add_hash(path: &str, new_hash: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut hashes = get_transaction_hashes(path).unwrap();
    if !hashes.contains(new_hash) {
        hashes.insert(new_hash.to_string());
        let hash_string = hashes
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(path, hash_string)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    const HASH_PATH: &str = "./config/existing-hashes-test.txt";
    const HASH_TEMPLATE_PATH: &str = "./config/existing-hashes-test-template.txt";
    const NEW_HASH: &str = "HASHTESTDATANEW1";
    #[test]
    fn test_get_hashes_works() {
        let hashes = get_transaction_hashes(HASH_PATH);
        let res = match hashes {
            Ok(h) => h,
            _ => HashSet::new(),
        };
        assert!(res.len() > 0);
        assert!(res.get("TESTHASHDATA1").is_some());
    }

    #[test]
    fn test_add_hashes_works() {
        let _ = add_hash(HASH_PATH, NEW_HASH);
        let hashes = match get_transaction_hashes(HASH_PATH) {
            Ok(h) => h,
            _ => HashSet::new(),
        };

        assert!(hashes.len() > 0);
        assert_eq!(hashes.len(), 3);
        assert!(hashes.get(NEW_HASH).is_some());
        clean_test_file(HASH_PATH);
    }

    fn clean_test_file(path: &str) {
        let template_data = fs::read_to_string(HASH_TEMPLATE_PATH).unwrap();
        let _ = fs::write(path, template_data);
    }
}
