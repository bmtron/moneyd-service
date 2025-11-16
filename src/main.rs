use crate::utils::{
    csvutil,
    globalutil::{self, AuthorizationData, StmntTxnData},
    statementtransporter::StatementTransport,
};
use dotenv::dotenv;
use std::env;
mod service;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let env_vars = get_env_vars();
    let api_key = env_vars.0.expect("API_KEY value not found.");

    // temp, auth token should be provided by API after successful login
    // TODO: implement cli frontend to handle logins and stuff
    let auth_key = env_vars.1.expect("AUTH_TOKEN value not found.");

    let ingestion_result = csvutil::ingest_amex(String::from("./examples/october2025_amex.csv"))?;
    println!("{:?}", &ingestion_result);

    // temp, hardcoded for now TODO
    let std: StmntTxnData = StmntTxnData {
        user_id: 1,
        institution_id: 1,
    };
    let auth_data: AuthorizationData = AuthorizationData {
        auth_token: auth_key,
        api_key: api_key,
    };
    let txn_result =
        globalutil::add_statement_and_transaction_data(std, auth_data, ingestion_result)
            .await
            .unwrap();

    println!("{:?}", txn_result);
    Ok(())
}

fn get_env_vars() -> (Option<String>, Option<String>) {
    let mut auth_token: Option<String> = None;
    let mut api_key: Option<String> = None;
    for (key, value) in env::vars() {
        if key.eq("API_KEY") {
            api_key = Some(value);
        } else if key.eq("AUTH_TOKEN") {
            // TODO: remove this later, get it from login endpoint
            auth_token = Some(value);
        }
    }
    (api_key, auth_token)
}
