use crate::{
    service::api::{DELETE, GET, POST, PUT, api_call_requires_body},
    utils::statementtransporter::{StatementResponse, StatementTransport},
};

pub async fn create_statement(
    user_id: i32,
    institution_id: i32,
    auth_token: &String,
    api_key: &String,
) -> Result<StatementResponse, Box<dyn std::error::Error>> {
    let mut statement: StatementTransport = StatementTransport {
        banking_user_id: user_id,
        institution_id: institution_id,
        // TODO: hardcoded, need to dynamically add statement period
        period_start: String::from("2025-09-21T00:00:00Z"),
        period_end: String::from("2025-10-21T00:00:00Z"),
    };

    // should load base url from .env
    // check errors here, panicking on 404 TODO
    let endpoint = String::from("http://localhost:8085/api/statements");

    let api_result = api_call_requires_body::<StatementTransport, POST>(
        endpoint, statement, auth_token, api_key,
    )
    .await?;

    let stmnt: StatementResponse = serde_json::from_str(&api_result)?;

    Ok(stmnt)
}
