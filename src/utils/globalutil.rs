use chrono::NaiveDate;
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
