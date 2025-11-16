use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct StatementTransport {
    pub banking_user_id: i32,
    pub institution_id: i32,
    pub period_start: String,
    pub period_end: String,
}
#[derive(Debug, Deserialize)]
pub struct StatementResponse {
    pub statement_id: i32,
    pub banking_user_id: i32,
    pub institution_id: i32,
    pub period_start: String,
    pub period_end: String,
    pub date_added: String,
}
