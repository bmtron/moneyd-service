use crate::{
    service::api::{POST, api_call_requires_body},
    utils::logintransporter::{LoginRequest, LoginResponse},
};

pub async fn login(login: LoginRequest, api_key: &String) -> LoginResponse {
    let endpoint = String::from("http://localhost:8085/auth/login");
    let resp = api_call_requires_body::<LoginRequest, POST>(endpoint, login, None, api_key)
        .await
        .unwrap();

    println!("response: {}", resp);

    let result: LoginResponse = serde_json::from_str(&resp).unwrap();

    result
}
