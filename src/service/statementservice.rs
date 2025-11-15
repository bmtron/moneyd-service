use crate::{
    service::api::apiCall,
    utils::transport::{StatementResponse, StatementTransport},
};

pub async fn create_statement(
    user_id: i32,
    institution_id: i32,
) -> Result<StatementResponse, Box<dyn std::error::Error>> {
    let mut statement: StatementTransport = StatementTransport {
        banking_user_id: 1,
        institution_id: 1,
        period_start: String::from("2025-09-16T00:00:00Z"),
        period_end: String::from("2025-10-16T00:00:00Z"),
    };

    let api_result = apiCall::<StatementTransport>(
        String::from("http://localhost:8085/api/statements"),
        statement,
    )
    .await?;

    let stmnt: StatementResponse = serde_json::from_str(&api_result)?;

    Ok(stmnt)
}
