//! Filtering and sorting example.
//!
//! Demonstrates building queries with multiple filters and sort specifications.
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
//! cargo run --example filters_and_sorts
//! ```

use labkey_rs::filter::{Filter, FilterType, FilterValue};
use labkey_rs::query::SelectRowsOptions;
use labkey_rs::sort::QuerySort;
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

    // Filters: multiple filters are AND'd together by the server.
    let filters = vec![
        // Equality filter using the convenience constructor.
        Filter::equal("Status", "Active"),
        // Greater-than filter with an explicit FilterType.
        Filter::new(
            "Age",
            FilterType::GreaterThan,
            FilterValue::Single("21".into()),
        ),
        // IN filter with multiple values.
        Filter::new(
            "Country",
            FilterType::In,
            FilterValue::Multi(vec!["US".into(), "CA".into(), "UK".into()]),
        ),
    ];

    // Sort: comma-separated string where `-` prefix means descending.
    // "Name" ascending, then "Created" descending.
    let sort = QuerySort::parse("Name,-Created");

    let options = SelectRowsOptions::builder()
        .schema_name(schema.clone())
        .query_name(query.clone())
        .filter_array(filters)
        .sort(sort)
        .max_rows(25)
        .build();

    let response = client.select_rows(options).await?;

    println!(
        "Filtered query: {}.{} — {} row(s)",
        schema, query, response.row_count
    );
    for row in &response.rows {
        // Print just the column names and values in a compact format.
        let cols: Vec<String> = row
            .data
            .iter()
            .map(|(k, v)| format!("{k}={}", v.value))
            .collect();
        println!("  {}", cols.join(", "));
    }

    Ok(())
}
