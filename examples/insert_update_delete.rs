//! Insert, update, and delete rows example.
//!
//! Demonstrates the full CRUD lifecycle: inserting a new row, updating it, and
//! then deleting it. Each mutation endpoint takes rows as `serde_json::Value`
//! objects, so you build them with `serde_json::json!` or by constructing a
//! `serde_json::Map` directly.
//!
//! # Environment variables
//!
//! - `LABKEY_BASE_URL` — Server base URL (e.g., `http://localhost:8080/labkey`)
//! - `LABKEY_API_KEY` — API key for authentication
//! - `LABKEY_CONTAINER` — Container path (e.g., `/MyProject`)
//! - `LABKEY_SCHEMA` — Schema name (e.g., `lists`)
//! - `LABKEY_QUERY` — Query/table name (e.g., `People`)
//!
//! The target table must be writable and have at least a `Name` text column
//! and an auto-increment primary key column (typically `Key` or `RowId`).
//! Adjust the field names below to match your table's schema.
//!
//! # Usage
//!
//! ```sh
//! export LABKEY_BASE_URL=http://localhost:8080/labkey
//! export LABKEY_API_KEY=your-api-key
//! export LABKEY_CONTAINER=/MyProject
//! export LABKEY_SCHEMA=lists
//! export LABKEY_QUERY=People
//! cargo run --example insert_update_delete
//! ```

use serde_json::json;

use labkey_rs::query::{DeleteRowsOptions, InsertRowsOptions, UpdateRowsOptions};
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

    // --- Insert ---
    // Row payloads are plain JSON objects. Field names must match the target
    // table's column names. The server returns the inserted rows with any
    // auto-generated fields (like RowId) filled in.
    let insert_result = client
        .insert_rows(
            InsertRowsOptions::builder()
                .schema_name(schema.clone())
                .query_name(query.clone())
                .rows(vec![json!({"Name": "Alice Example"})])
                .build(),
        )
        .await?;

    println!(
        "Inserted {} row(s), command: {}",
        insert_result.rows_affected, insert_result.command
    );

    // Extract the primary key from the server's response so we can update
    // and delete the same row. The key field name varies by table — adjust
    // "Key" to match yours (common alternatives: "RowId", "EntityId").
    let inserted_row = insert_result
        .rows
        .first()
        .expect("server should return the inserted row");
    let row_key = inserted_row
        .get("Key")
        .or_else(|| inserted_row.get("RowId"))
        .expect("inserted row should contain a primary key field");
    println!("Inserted row key: {row_key}");

    // --- Update ---
    // Include the primary key so the server knows which row to update.
    let update_result = client
        .update_rows(
            UpdateRowsOptions::builder()
                .schema_name(schema.clone())
                .query_name(query.clone())
                .rows(vec![json!({"Key": row_key, "Name": "Alice Updated"})])
                .build(),
        )
        .await?;

    println!(
        "Updated {} row(s), command: {}",
        update_result.rows_affected, update_result.command
    );

    // --- Delete ---
    // Only the primary key is required to identify the row to delete.
    let delete_result = client
        .delete_rows(
            DeleteRowsOptions::builder()
                .schema_name(schema.clone())
                .query_name(query.clone())
                .rows(vec![json!({"Key": row_key})])
                .build(),
        )
        .await?;

    println!(
        "Deleted {} row(s), command: {}",
        delete_result.rows_affected, delete_result.command
    );

    Ok(())
}
