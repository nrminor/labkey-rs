//! Execute SQL example.
//!
//! Runs a LabKey SQL query and prints the results. LabKey SQL is similar to
//! standard SQL but uses schema-qualified table names and has some differences
//! in function names and syntax. See the LabKey documentation for details:
//! <https://www.labkey.org/Documentation/wiki-page.view?name=labkeySql>
//!
//! # Environment variables
//!
//! - `LABKEY_BASE_URL` — Server base URL (e.g., `http://localhost:8080/labkey`)
//! - `LABKEY_API_KEY` — API key for authentication
//! - `LABKEY_CONTAINER` — Container path (e.g., `/MyProject`)
//! - `LABKEY_SCHEMA` — Schema to execute against (e.g., `core`)
//!
//! # Usage
//!
//! ```sh
//! export LABKEY_BASE_URL=http://localhost:8080/labkey
//! export LABKEY_API_KEY=your-api-key
//! export LABKEY_CONTAINER=/MyProject
//! export LABKEY_SCHEMA=core
//! cargo run --example execute_sql
//! ```

use labkey_rs::query::ExecuteSqlOptions;
use labkey_rs::{ClientConfig, Credential, LabkeyClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::var("LABKEY_BASE_URL")?;
    let api_key = std::env::var("LABKEY_API_KEY")?;
    let container = std::env::var("LABKEY_CONTAINER")?;
    let schema = std::env::var("LABKEY_SCHEMA")?;

    let config = ClientConfig::new(&base_url, Credential::ApiKey(api_key), &container);
    let client = LabkeyClient::new(config)?;

    // LabKey SQL uses schema-qualified table names. The SQL is automatically
    // WAF-encoded by the client before sending.
    let sql = format!("SELECT * FROM {schema}.Users LIMIT 10");

    let options = ExecuteSqlOptions::builder()
        .schema_name(schema.clone())
        .sql(sql.clone())
        .build();

    let response = client.execute_sql(options).await?;

    println!("SQL: {sql}");
    println!("Returned {} row(s)", response.row_count);
    println!();

    // The response has the same shape as select_rows: rows of CellValue maps.
    for (i, row) in response.rows.iter().enumerate() {
        println!("Row {i}:");
        for (column, cell) in &row.data {
            println!("  {column}: {}", cell.value);
        }
    }

    Ok(())
}
