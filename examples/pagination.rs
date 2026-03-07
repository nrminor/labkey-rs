//! Pagination example.
//!
//! Demonstrates offset-based paging through a large result set using `max_rows`
//! and `offset`. LabKey's query endpoints use a simple offset/limit model: you
//! request a page size with `max_rows` and advance through results by
//! incrementing `offset`.
//!
//! # Environment variables
//!
//! - `LABKEY_BASE_URL` — Server base URL (e.g., `http://localhost:8080/labkey`)
//! - `LABKEY_API_KEY` — API key for authentication
//! - `LABKEY_CONTAINER` — Container path (e.g., `/MyProject`)
//! - `LABKEY_SCHEMA` — Schema name (e.g., `lists`)
//! - `LABKEY_QUERY` — Query/table name (e.g., `People`)
//!
//! # Usage
//!
//! ```sh
//! export LABKEY_BASE_URL=http://localhost:8080/labkey
//! export LABKEY_API_KEY=your-api-key
//! export LABKEY_CONTAINER=/MyProject
//! export LABKEY_SCHEMA=lists
//! export LABKEY_QUERY=People
//! cargo run --example pagination
//! ```

use labkey_rs::query::SelectRowsOptions;
use labkey_rs::{ClientConfig, Credential, LabkeyClient};

const PAGE_SIZE: i32 = 25;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::var("LABKEY_BASE_URL")?;
    let api_key = std::env::var("LABKEY_API_KEY")?;
    let container = std::env::var("LABKEY_CONTAINER")?;
    let schema = std::env::var("LABKEY_SCHEMA")?;
    let query = std::env::var("LABKEY_QUERY")?;

    let config = ClientConfig::new(&base_url, Credential::ApiKey(api_key), &container);
    let client = LabkeyClient::new(config)?;

    let mut offset: i64 = 0;
    let mut page = 1;
    let mut total_fetched: i64 = 0;

    loop {
        let options = SelectRowsOptions::builder()
            .schema_name(schema.clone())
            .query_name(query.clone())
            .max_rows(PAGE_SIZE)
            .offset(offset)
            .include_total_count(true)
            .build();

        let response = client.select_rows(options).await?;

        println!(
            "Page {page}: got {} row(s) (offset {offset})",
            response.row_count
        );

        total_fetched += response.row_count;

        // When the server returns fewer rows than requested, we've reached
        // the end. An empty page (row_count == 0) also signals completion.
        if response.row_count < i64::from(PAGE_SIZE) {
            break;
        }

        offset += i64::from(PAGE_SIZE);
        page += 1;
    }

    println!("Fetched {total_fetched} total row(s) across {page} page(s)");

    Ok(())
}
