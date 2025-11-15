use reqwest::header::{HeaderMap, HeaderValue};
use serde::Serialize;
use serde_json;

// TODO: probably rework this at some point. This is messy and contains duplicated code. Too tired rn.
// also need to implement .env gathering
pub async fn apiCall<T: Serialize>(
    endpoint: String,
    payload: T,
    auth_token: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let json_payload = serde_json::to_string(&payload)?;
    let client = reqwest::Client::new();
    let resp = client
        .post(endpoint)
        .headers(build_headers("GET_TOKEN_FROM_ENV"))
        .bearer_auth(auth_token)
        .body(json_payload)
        .send()
        .await?;
    let res_bytes = resp.bytes().await?.to_vec();

    let e = String::from_utf8(res_bytes)?;
    Ok(e)
}

pub async fn apiBatchCall<T: Serialize>(
    endpoint: String,
    payload: Vec<T>,
    auth_token: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let json_payload = serde_json::to_string(&payload)?;
    let client = reqwest::Client::new();
    let resp = client
        .post(endpoint)
        .headers(build_headers("GET_TOKEN_FROM_ENV"))
        .bearer_auth(auth_token)
        .body(json_payload)
        .send()
        .await?;
    let res_bytes = resp.bytes().await?.to_vec();

    let e = String::from_utf8(res_bytes)?;
    Ok(e)
}

fn build_headers(api_token: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let api_token: HeaderValue = HeaderValue::from_static(api_token);

    headers.append("X-API-Key", api_token);

    headers
}
