use crate::{
    service::{
        api::{POST, api_call_requires_body},
        loginservice::login,
    },
    ui::loginwindow::build_login_window,
    utils::{
        csvutil,
        globalutil::{self, AuthorizationData, StmntTxnData, get_env_vars},
        logintransporter::LoginRequest,
        statementtransporter::StatementTransport,
    },
};
use clap;
use cursive::{
    CursiveRunnable,
    reexports::log,
    views::{Dialog, TextView},
};
use dotenv::dotenv;
use std::env;
mod ingestion;
mod service;
mod ui;
mod utils;

struct Env {
    api_key: Option<String>,
    base_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let env_vars = get_env_vars();

    let mut siv = build_login_window();
    siv.run();
    let login_request = get_user_data_from_cursive(&mut siv);

    let mut auth_token: String = String::new();
    let login_res = login(
        login_request,
        &env_vars.api_key.expect("API_KEY not set. Panicking."),
    )
    .await;
    auth_token = login_res.token;

    // login handled, begin ingestion

    let res = ingestion::ingestinator()?;
    println!("{}", auth_token);
    Ok(())
}

async fn temp_ingest_and_send(
    auth_token: String,
    api_key: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let ingestion_result = csvutil::ingest_amex(String::from("./examples/october2025_amex.csv"))?;
    println!("{:?}", &ingestion_result);

    // temp, hardcoded for now TODO
    let std: StmntTxnData = StmntTxnData {
        user_id: 1,
        institution_id: 1,
    };
    let auth_data: AuthorizationData = AuthorizationData {
        auth_token: auth_token,
        api_key: api_key,
    };
    let txn_result =
        globalutil::add_statement_and_transaction_data(std, auth_data, ingestion_result)
            .await
            .unwrap();

    println!("{:?}", txn_result);

    Ok(())
}

fn get_user_data_from_cursive(siv: &mut CursiveRunnable) -> LoginRequest {
    let user_data: LoginRequest = siv.take_user_data().unwrap();
    let login_request: LoginRequest = LoginRequest {
        email: user_data.email,
        password: user_data.password,
    };
    login_request
}
