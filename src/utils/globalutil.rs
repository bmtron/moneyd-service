use crate::service::statementservice::create_statement;
use crate::utils::transactiontransporter::TransactionResponse;
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

#[derive(Debug, Deserialize, Serialize)]
pub struct AmexData {
    pub date: String,
    pub description: String,
    pub amount: Decimal,
}

pub trait ConvertToTransaction {
    fn to_txn(&self) -> TransactionTransport;
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
pub struct StmntTxnData {
    pub user_id: i32,
    pub institution_id: i32,
}

pub struct AuthorizationData {
    pub auth_token: String,
    pub api_key: String,
}

pub async fn add_statement_and_transaction_data(
    stmn_txn_data: StmntTxnData,
    auth_data: AuthorizationData,
    txn_batch: Vec<TransactionTransport>,
) -> Result<Vec<TransactionResponse>, Box<dyn std::error::Error>> {
    let statement = create_statement(
        stmn_txn_data.user_id,
        stmn_txn_data.institution_id,
        &auth_data.auth_token,
        &auth_data.api_key,
    )
    .await
    .unwrap();

    let txns = create_transactions(
        txn_batch,
        statement.statement_id,
        &auth_data.auth_token,
        &auth_data.api_key,
    )
    .await
    .unwrap();

    Ok(txns)
}
