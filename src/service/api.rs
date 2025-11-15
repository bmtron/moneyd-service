use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{Client, RequestBuilder};
use serde::Serialize;
use serde_json;

pub struct GET;
pub struct POST;
pub struct PUT;
pub struct DELETE;

trait ApiRequestBuildable {
    fn build(client: &Client, endpoint: &str) -> RequestBuilder;
    fn has_body() -> bool;
}

impl ApiRequestBuildable for GET {
    fn build(client: &Client, endpoint: &str) -> RequestBuilder {
        client.get(endpoint)
    }

    fn has_body() -> bool {
        false
    }
}

impl ApiRequestBuildable for POST {
    fn build(client: &Client, endpoint: &str) -> RequestBuilder {
        client.post(endpoint)
    }

    fn has_body() -> bool {
        true
    }
}

impl ApiRequestBuildable for PUT {
    fn build(client: &Client, endpoint: &str) -> RequestBuilder {
        client.put(endpoint)
    }

    fn has_body() -> bool {
        true
    }
}

impl ApiRequestBuildable for DELETE {
    fn build(client: &Client, endpoint: &str) -> RequestBuilder {
        client.delete(endpoint)
    }

    fn has_body() -> bool {
        false
    }
}
// TODO: likely split these into RequiresBody and ForbiddenBody calls
pub async fn apiCall<T: Serialize, K: ApiRequestBuildable>(
    endpoint: String,
    payload: T,
    auth_token: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = Client::new();
    let mut builder = K::build(&client, endpoint.as_str());
    let request_has_body = K::has_body();

    builder = builder
        .headers(build_headers("GET_TOKEN_FROM_ENV"))
        .bearer_auth(auth_token);
    if request_has_body {
        builder = builder.json(&payload);
    }
    let resp_bytes = builder.send().await?.bytes().await?.to_vec();

    let e = String::from_utf8(resp_bytes)?;
    Ok(e)
}

fn build_headers(api_token: &'static str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let api_token: HeaderValue = HeaderValue::from_static(api_token);

    headers.append("X-API-Key", api_token);

    headers
}
