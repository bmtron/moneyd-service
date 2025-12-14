use crate::{
    service::api::{POST, api_call_requires_body},
    utils::{
        globalutil::AuthorizationData,
        transactiontransporter::{TransactionResponse, TransactionTransport},
    },
};

pub async fn create_transactions(
    mut txns: &Vec<TransactionTransport>,
    statement_id: i32,
    auth_data: &AuthorizationData,
) -> Result<Vec<TransactionResponse>, Box<dyn std::error::Error>> {
    // need to not hard code the endpoint TODO
    let endpoint = String::from("http://localhost:8085/api/transactions/batch");
    let some_auth_token: Option<String> = Some(auth_data.auth_token.clone());
    let api_result = api_call_requires_body::<Vec<TransactionTransport>, POST>(
        endpoint,
        &txns,
        some_auth_token,
        &auth_data.api_key,
    )
    .await
    .unwrap();

    let txn: Vec<TransactionResponse> = serde_json::from_str(&api_result)?;

    Ok(txn)
}
