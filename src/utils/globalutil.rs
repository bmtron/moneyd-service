use std::env;
use std::fs::{File, canonicalize};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::str::FromStr;

use crate::Env;
use crate::ingestion::{FileHashData, IngestionResult};
use crate::service::statementservice::create_statement;
use crate::utils::logintransporter::LoginResponse;
use crate::utils::statementtransporter::StatementTransport;
use crate::{
    service::transactionservice::create_transactions,
    utils::transactiontransporter::TransactionTransport,
};

use chrono::{DateTime, TimeZone, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::{self, Decimal};
use serde::{Deserialize, Serialize};
pub enum FinancialEstablishment {
    Amex,
    Citizens,
    CapitalOne,
}
pub trait ConvertToTransaction {
    fn to_txn(&self) -> TransactionTransport;
}
#[derive(Debug, Deserialize, Serialize)]
pub struct AmexData {
    pub date: String,
    pub description: String,
    pub amount: Decimal,
}

impl ConvertToTransaction for AmexData {
    fn to_txn(&self) -> TransactionTransport {
        let formatted_date = match parse_and_format_date(&self.date) {
            Ok(date_str) => date_str,
            Err(_) => self.date.clone(), // Fallback to original if parsing fails
        };

        let amount_as_i32 = self
            .amount
            .checked_mul(rust_decimal::Decimal::from(100))
            .and_then(|dec| dec.to_i32())
            .unwrap_or(0);

        let txntp = TransactionTransport {
            statement_id: 0,
            description: self.description.clone(),
            amount: amount_as_i32,
            transaction_date: formatted_date,
        };

        txntp
    }
}
#[derive(Debug, Deserialize)]
pub struct CitizensData {
    transaction_type: String,
    date: String,
    account_type: String,
    description: String,
    amount: String,
    reference_no: String,
    debits: String,
    credits: String,
}

impl ConvertToTransaction for CitizensData {
    fn to_txn(&self) -> TransactionTransport {
        let formatted_date = match parse_and_format_date(&self.date) {
            Ok(date_str) => date_str,
            Err(_) => self.date.clone(), // Fallback to original if parsing fails
        };

        let mut is_negative = false;
        let mut sliced_amount = String::from(self.amount.as_str());
        if self.amount.contains("-") {
            is_negative = true;
            sliced_amount = sliced_amount.replace("-", "");
        }

        let amount_as_decimal = Decimal::from_str(&sliced_amount.as_str()).unwrap();
        let mut amount_as_i32 = amount_as_decimal
            .checked_mul(rust_decimal::Decimal::from(100))
            .and_then(|dec| dec.to_i32())
            .unwrap_or(0);

        if is_negative {
            amount_as_i32 = amount_as_i32 * -1;
        }

        let txntp = TransactionTransport {
            statement_id: 0,
            description: self.description.clone(),
            amount: amount_as_i32,
            transaction_date: formatted_date,
        };

        txntp
    }
}
fn parse_and_format_date(date_str: &str) -> Result<String, Box<dyn std::error::Error>> {
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
    ingest_data: IngestionResult,
    login_data: LoginResponse,
    auth_data: AuthorizationData,
) -> Result<(), Box<dyn std::error::Error>> {
    for c in ingest_data.citizens_ingestion {
        // hardcoding institution id's for now and statment period data, TODO
        let citizens_statement_data: StatementTransport = StatementTransport {
            banking_user_id: login_data.user.id,
            institution_id: 2, // cb
            period_start: parse_and_format_date("2025-11-01").unwrap(),
            period_end: parse_and_format_date("2025-11-17").unwrap(),
        };
        let stmt = create_statement(citizens_statement_data, &auth_data)
            .await
            .unwrap();

        // not using the response for anything here, consider doing...something...TODO
        let txn_batch_result = create_transactions(c, stmt.statement_id, &auth_data)
            .await
            .unwrap();
    }

    for a in ingest_data.amex_ingestion {
        // hardcoding institution id's for now and statment period data, TODO
        let amex_statement_data: StatementTransport = StatementTransport {
            banking_user_id: login_data.user.id,
            institution_id: 1, // amex
            period_start: parse_and_format_date("2025-11-01").unwrap(),
            period_end: parse_and_format_date("2025-11-17").unwrap(),
        };
        let stmt = create_statement(amex_statement_data, &auth_data)
            .await
            .unwrap();

        // not using the response for anything here, consider doing...something...TODO
        let txn_batch_result = create_transactions(a, stmt.statement_id, &auth_data)
            .await
            .unwrap();
    }

    Ok(())
}

pub fn update_hashes(hash_data: Vec<FileHashData>) -> Result<(), Box<dyn std::error::Error>> {
    let file_hash_str = serde_json::to_string(&hash_data).unwrap();
    let hash_json_path = Path::new("./config/consumed-files.json");
    let absolute_hash_path = canonicalize(hash_json_path)?;

    let file = File::create(absolute_hash_path).unwrap();
    let mut writer = BufWriter::new(file);
    writer.write_all(&file_hash_str.as_bytes())?;
    writer.flush()?;
    Ok(())
}
