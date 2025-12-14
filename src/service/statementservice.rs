use crate::{
    service::api::{POST, api_call_requires_body},
    utils::{
        globalutil::AuthorizationData,
        statementtransporter::{StatementResponse, StatementTransport},
    },
};

pub async fn create_statement(
    statement_xport: &StatementTransport,
    auth_data: &AuthorizationData,
) -> Result<StatementResponse, Box<dyn std::error::Error>> {
    // should load base url from .env
    // check errors here, panicking on 404 TODO
    let endpoint = String::from("http://localhost:8085/api/statements");
    let some_auth_token: Option<String> = Some(auth_data.auth_token.clone());
    let api_result = api_call_requires_body::<StatementTransport, POST>(
        endpoint,
        statement_xport,
        some_auth_token,
        &auth_data.api_key.clone(),
    )
    .await?;

    let stmnt: StatementResponse = serde_json::from_str(&api_result)?;

    Ok(stmnt)
}
