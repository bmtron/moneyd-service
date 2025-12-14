use crate::{
    service::api::{GET, api_call_no_body},
    utils::transactiontransporter::TransactionResponse,
    AuthorizationData
};

pub async fn analyze_data(auth_data: &AuthorizationData, institution_id: &i32, user_id: &i32) {
    let endpoint = format!("http://localhost:8085/api/transactions/by_institution/user/{user_id}/institution/{institution_id}");
    let txn_resp = 
        api_call_no_body::<Vec<TransactionResponse>, GET>(endpoint, &auth_data.auth_token, &auth_data.api_key)
            .await
            .unwrap_or(String::from("Invalid data returned to analyze."));
    let txns: Vec<TransactionResponse> = serde_json::from_str(&txn_resp).unwrap_or(Vec::new());

    println!("{:?}", txns);


}


