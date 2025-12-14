use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, RequestBuilder};
use serde::Serialize;

pub struct GET;
pub struct POST;
pub struct PUT;
pub struct DELETE;

pub trait ApiRequestBuildable {
    fn build(client: &Client, endpoint: &str) -> RequestBuilder;
}

pub trait RequiresBody {}
pub trait ForbiddenBody {}

impl RequiresBody for POST {}
impl RequiresBody for PUT {}
impl ForbiddenBody for GET {}
impl ForbiddenBody for DELETE {}

impl ApiRequestBuildable for GET {
    fn build(client: &Client, endpoint: &str) -> RequestBuilder {
        client.get(endpoint)
    }
}

impl ApiRequestBuildable for POST {
    fn build(client: &Client, endpoint: &str) -> RequestBuilder {
        client.post(endpoint)
    }
}

impl ApiRequestBuildable for PUT {
    fn build(client: &Client, endpoint: &str) -> RequestBuilder {
        client.put(endpoint)
    }
}

impl ApiRequestBuildable for DELETE {
    fn build(client: &Client, endpoint: &str) -> RequestBuilder {
        client.delete(endpoint)
    }
}

pub async fn api_call_no_body<T: Serialize, K: ApiRequestBuildable + ForbiddenBody>(
    endpoint: String,
    auth_token: &String,
    api_key: &String,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut builder = K::build(&client, endpoint.as_str());
    builder = builder
        .headers(build_headers(api_key.as_str()))
        .bearer_auth(auth_token);
    let resp_bytes = builder.send().await?.bytes().await?.to_vec();
    let result = String::from_utf8(resp_bytes)?;
    Ok(result)
}

pub async fn api_call_requires_body<T: Serialize, K: ApiRequestBuildable + RequiresBody>(
    endpoint: String,
    payload: &T,
    auth_token: Option<String>,
    api_key: &String,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut builder = K::build(&client, endpoint.as_str());

    builder = builder.headers(build_headers(&api_key));
    // login endpoint does not require auth token (duh)
    if let Some(auth_token_exists) = auth_token {
        builder = builder.bearer_auth(auth_token_exists);
    }
    builder = builder.json(&payload);

    let resp_bytes = builder.send().await?.bytes().await?.to_vec();
    let result = String::from_utf8(resp_bytes)?;
    Ok(result)
}

fn build_headers(api_key: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let api_token: HeaderValue = HeaderValue::from_str(api_key).expect("Invalid header value.");

    headers.append("X-API-Key", api_token);

    headers
}
