use crate::{
    service::loginservice::login,
    ui::loginwindow::build_login_window,
    utils::{
        globalutil::{
            self, AuthorizationData, get_env_vars, post_statements_and_transactions, update_hashes,
        },
        logintransporter::LoginRequest,
    },
};

use cursive::CursiveRunnable;
use dotenv::dotenv;

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

    let api_key = &env_vars.api_key.expect("API_KEY not set. Panicking.");
    let mut auth_token: String = String::new();
    let login_res = login(login_request, api_key).await;

    auth_token = login_res.token.clone();
    let auth_data: AuthorizationData = AuthorizationData {
        auth_token: auth_token,
        api_key: api_key.clone(),
    };

    // login handled, begin ingestion
    let ingestion_res = ingestion::ingestinator()?;

    post_statements_and_transactions(ingestion_res.ingestion_result, login_res, auth_data)
        .await
        .unwrap();

    update_hashes(ingestion_res.file_hash_data).unwrap();

    println!("Execution successful. Data uploaded.");
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
