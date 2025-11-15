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

    let mut statement: StatementTransport = StatementTransport {
        banking_user_id: 1,
        institution_id: 1,
        period_start: String::from("2025-09-16T00:00:00Z"),
        period_end: String::from("2025-10-16T00:00:00Z"),
    };

    let api_result = apiCall::<StatementTransport>(
        String::from("http://localhost:8085/api/statements"),
        statement,
    )
    .await?;

    println!("api result: {}", api_result);

    Ok(())
}
