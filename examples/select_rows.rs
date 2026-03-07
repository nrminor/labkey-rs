//! Basic `select_rows` example.
//!
//! Connects to a LabKey server, queries a table, and prints the results.
//!
//! # Environment variables
//!
//! - `LABKEY_BASE_URL` — Server base URL (e.g., `http://localhost:8080/labkey`)
//! - `LABKEY_API_KEY` — API key for authentication
//! - `LABKEY_CONTAINER` — Container path (e.g., `/MyProject`)
//! - `LABKEY_SCHEMA` — Schema name (e.g., `core`)
//! - `LABKEY_QUERY` — Query/table name (e.g., `Users`)
//!
//! # Usage
//!
//! ```sh
//! export LABKEY_BASE_URL=http://localhost:8080/labkey
//! export LABKEY_API_KEY=your-api-key
//! export LABKEY_CONTAINER=/MyProject
//! export LABKEY_SCHEMA=core
//! export LABKEY_QUERY=Users
//! cargo run --example select_rows
//! ```

use labkey_rs::query::SelectRowsOptions;
use labkey_rs::{ClientConfig, Credential, LabkeyClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::var("LABKEY_BASE_URL")?;
    let api_key = std::env::var("LABKEY_API_KEY")?;
    let container = std::env::var("LABKEY_CONTAINER")?;
    let schema = std::env::var("LABKEY_SCHEMA")?;
    let query = std::env::var("LABKEY_QUERY")?;

    let config = ClientConfig::new(&base_url, Credential::ApiKey(api_key), &container);
    let client = LabkeyClient::new(config)?;

    let options = SelectRowsOptions::builder()
        .schema_name(schema.clone())
        .query_name(query.clone())
        .max_rows(10)
        .build();

    let response = client.select_rows(options).await?;

    println!(
        "Query: {}.{} — {} row(s) returned",
        schema, query, response.row_count
    );
    println!();

    for (i, row) in response.rows.iter().enumerate() {
        println!("Row {i}:");
        for (column, cell) in &row.data {
            // CellValue wraps the raw JSON value; display_value is populated
            // for lookup columns.
            let display = cell
                .display_value
                .as_deref()
                .map(|dv| format!(" (display: {dv})"))
                .unwrap_or_default();
            println!("  {column}: {}{display}", cell.value);
        }
    }

    Ok(())
}
