use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct TransactionTransport {
    pub statement_id: Option<i32>,
    pub description: String,
    pub amount: i32,
    pub transaction_date: String,
    pub refnum: String,
    pub transaction_type_lookup_code: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TransactionResponse {
    pub transaction_id: i32,
    pub statement_id: i32,
    pub description: String,
    pub amount: i32,
    pub transaction_date: String,
}
