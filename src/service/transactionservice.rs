use crate::{
    service::{
        api::{POST, api_call_requires_body},
        transactionservice,
    },
    utils::transactiontransporter::{TransactionResponse, TransactionTransport},
};

pub async fn create_transactions(
    mut txns: Vec<TransactionTransport>,
    statement_id: i32,
    auth_token: &String,
    api_key: &String,
) -> Result<Vec<TransactionResponse>, Box<dyn std::error::Error>> {
    for transaction in txns.iter_mut() {
        // Process each transaction
        // transaction is of type &mut TransactionTransport
        transaction.statement_id = statement_id;
    }
    let endpoint = String::from("http://localhost:8085/api/transactions/batch");
    let some_auth_token: Option<String> = Some(auth_token.clone());
    let api_result = api_call_requires_body::<Vec<TransactionTransport>, POST>(
        endpoint,
        txns,
        some_auth_token,
        api_key,
    )
    .await
    .unwrap();

    let txn: Vec<TransactionResponse> = serde_json::from_str(&api_result)?;

    Ok(txn)
}
