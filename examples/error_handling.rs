//! Error handling example.
//!
//! Demonstrates how to match on [`LabkeyError`] variants for diagnostics and
//! recovery. This example intentionally triggers errors by querying a
//! nonexistent schema and table, then shows how to inspect the structured
//! error body returned by the server.
//!
//! # Environment variables
//!
//! - `LABKEY_BASE_URL` — Server base URL (e.g., `http://localhost:8080/labkey`)
//! - `LABKEY_API_KEY` — API key for authentication
//! - `LABKEY_CONTAINER` — Container path (e.g., `/MyProject`)
//!
//! # Usage
//!
//! ```sh
//! export LABKEY_BASE_URL=http://localhost:8080/labkey
//! export LABKEY_API_KEY=your-api-key
//! export LABKEY_CONTAINER=/MyProject
//! cargo run --example error_handling
//! ```

use labkey_rs::query::SelectRowsOptions;
use labkey_rs::{ClientConfig, Credential, LabkeyClient, LabkeyError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let base_url = std::env::var("LABKEY_BASE_URL")?;
    let api_key = std::env::var("LABKEY_API_KEY")?;
    let container = std::env::var("LABKEY_CONTAINER")?;

    let config = ClientConfig::new(&base_url, Credential::ApiKey(api_key), &container);
    let client = LabkeyClient::new(config)?;

    // Intentionally query a nonexistent schema/table to trigger a server error.
    let options = SelectRowsOptions::builder()
        .schema_name("nonexistent_schema".to_owned())
        .query_name("nonexistent_table".to_owned())
        .build();

    match client.select_rows(options).await {
        Ok(response) => {
            println!("Unexpected success: {} rows", response.row_count);
        }
        Err(LabkeyError::Api { status, body }) => {
            // Structured API errors include the HTTP status, a human-readable
            // exception message, and the Java exception class from the server.
            println!("LabKey API error (HTTP {status}):");
            if let Some(msg) = &body.exception {
                println!("  Message: {msg}");
            }
            if let Some(class) = &body.exception_class {
                println!("  Exception class: {class}");
            }
            for field_err in &body.errors {
                println!(
                    "  Field error: {} — {}",
                    field_err.id.as_deref().unwrap_or("(no field)"),
                    field_err.msg.as_deref().unwrap_or("(no message)")
                );
            }

            // Helper methods provide quick checks for common error categories.
            if body
                .exception_class
                .as_deref()
                .is_some_and(|c| c.contains("NotFoundException"))
            {
                println!("  → This looks like a 'not found' error.");
            }
        }
        Err(LabkeyError::Http(err)) => {
            // Network-level failures: connection refused, DNS resolution, timeouts.
            println!("HTTP transport error: {err}");
            if err.is_connect() {
                println!("  → Could not connect to the server. Is it running?");
            }
            if err.is_timeout() {
                println!("  → The request timed out.");
            }
        }
        Err(LabkeyError::UnexpectedResponse { status, text }) => {
            // The server returned a non-success status but the body wasn't a
            // structured LabKey error (e.g., an HTML error page from a reverse
            // proxy).
            println!("Unexpected response (HTTP {status}):");
            println!("  Body: {}", &text[..text.len().min(200)]);
        }
        Err(other) => {
            // Covers InvalidInput, Deserialization, and Url variants.
            println!("Other error: {other}");
        }
    }

    Ok(())
}
