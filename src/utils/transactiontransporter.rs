use serde::{Deserialize, Serialize};

use crate::utils::globalutil::AmexData;

#[derive(Debug, Serialize)]
pub struct TransactionTransport {
    pub statement_id: i32,
    pub description: String,
    pub amount: i32,
    pub transaction_date: String,
}
#[derive(Debug, Deserialize)]
pub struct TransactionResponse {
    pub transaction_id: i32,
    pub statement_id: i32,
    pub description: String,
    pub amount: i32,
    pub transaction_date: String,
}
