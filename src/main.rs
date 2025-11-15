use crate::{
    service::api::apiCall,
    utils::{csvutil, globalutil, transport::StatementTransport},
};

mod ingestion;
mod service;
mod utils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");
    let ingestion_result = csvutil::ingest_amex(String::from("./examples/october2025_amex.csv"))?;
    println!("{:?}", ingestion_result);

    Ok(())
}
