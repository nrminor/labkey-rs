//! Experimental compact SQL example.
//!
//! Demonstrates the feature-gated experimental SQL API for leaner bulk reads.
//! This endpoint returns compact row data and includes helper methods for
//! row/column-oriented access.
//!
//! # Security note
//!
//! Keep SQL text under application control. Do not concatenate untrusted user
//! input directly into SQL strings.
//!
//! # Environment variables
//!
//! - `LABKEY_BASE_URL` — Server base URL (e.g., `http://localhost:8080/labkey`)
//! - `LABKEY_API_KEY` — API key for authentication
//! - `LABKEY_CONTAINER` — Container path (e.g., `/MyProject`)
//! - `LABKEY_SCHEMA` — Schema to execute against (e.g., `core`)
//! - `LABKEY_QUERY` — Query/table name to read from (e.g., `Users`)
//!
//! # Usage
//!
//! ```sh
//! export LABKEY_BASE_URL=http://localhost:8080/labkey
//! export LABKEY_API_KEY=your-api-key
//! export LABKEY_CONTAINER=/MyProject
//! export LABKEY_SCHEMA=core
//! export LABKEY_QUERY=Users
//! cargo run --example experimental_sql --features experimental
//! ```

use labkey_rs::query::experimental::{ExperimentalQueryExt, SqlExecuteOptions};
use labkey_rs::{ClientConfig, Credential, LabkeyClient};

fn validate_identifier(value: &str, name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let valid = !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_');

    if valid {
        Ok(())
    } else {
        Err(format!("{name} must contain only ASCII letters, numbers, or underscores").into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::var("LABKEY_BASE_URL")?;
    let api_key = std::env::var("LABKEY_API_KEY")?;
    let container = std::env::var("LABKEY_CONTAINER")?;
    let schema = std::env::var("LABKEY_SCHEMA")?;
    let query = std::env::var("LABKEY_QUERY")?;

    validate_identifier(&schema, "LABKEY_SCHEMA")?;
    validate_identifier(&query, "LABKEY_QUERY")?;

    let config = ClientConfig::new(&base_url, Credential::ApiKey(api_key), &container);
    let client = LabkeyClient::new(config)?;

    // Keep SQL template-controlled and static in structure.
    let sql = format!("SELECT * FROM {schema}.{query}");

    let response = client
        .experimental_sql_execute(
            SqlExecuteOptions::builder()
                .schema(schema.clone())
                .sql(sql.clone())
                .build(),
        )
        .await?;

    println!("SQL: {sql}");
    println!(
        "Returned {} row(s), {} column(s)",
        response.row_count(),
        response.column_count()
    );
    println!("Columns: {:?}", response.names);

    // Iterate row-major without allocating per-row maps.
    for (index, row) in response.iter_rows().take(5).enumerate() {
        println!("row[{index}] = {row:?}");
    }

    // Column-major projection is often convenient for analytics pipelines.
    let columnar = response.clone().into_columns();
    if let Some(first_column) = columnar.columns.first() {
        println!("First column has {} value(s)", first_column.len());
    }

    Ok(())
}
