# labkey-rs

Unofficial Rust client for the [LabKey Server](https://www.labkey.com/) REST API.

This crate provides typed, async access to LabKey's HTTP endpoints for querying data, managing security, working with assays and experiments, and more. It is a port of the official [`@labkey/api`](https://github.com/LabKey/labkey-api-js) JavaScript/TypeScript client (v1.48.0), supplemented by the [Java client](https://github.com/LabKey/labkey-api-java) for endpoint coverage. It is not affiliated with or endorsed by LabKey Corporation.

## Quick start

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
labkey-rs = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Then construct a client and query some data:

```rust,no_run
use labkey_rs::{ClientConfig, Credential, LabkeyClient};
use labkey_rs::query::SelectRowsOptions;
use labkey_rs::filter::{Filter, FilterType};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("LABKEY_API_KEY")?;

    let client = LabkeyClient::new(ClientConfig::new(
        "https://labkey.example.com/labkey",
        Credential::ApiKey(api_key),
        "/MyProject",
    ))?;

    let response = client.select_rows(
        SelectRowsOptions::builder()
            .schema_name("core")
            .query_name("Users")
            .filter_array(vec![
                Filter::new("Email", FilterType::Contains, "example.com"),
            ])
            .build(),
    ).await?;

    println!("Found {} users", response.row_count.unwrap_or(0));
    for row in &response.rows {
        println!("  {row:?}");
    }

    Ok(())
}
```

## Authentication

The client supports four credential types:

```rust,no_run
use labkey_rs::Credential;

// API key from an environment variable (recommended)
let cred = Credential::ApiKey(
    std::env::var("LABKEY_API_KEY").expect("LABKEY_API_KEY must be set"),
);

// Basic auth (email + password — avoid hardcoding these)
let cred = Credential::Basic {
    email: "user@example.com".into(),
    password: "secret".into(),
};

// Read credentials from a ~/.netrc file
let cred = Credential::from_netrc("labkey.example.com")
    .expect("credentials found in .netrc");

// Guest (anonymous, no auth header)
let cred = Credential::Guest;
```

API keys are the standard LabKey authentication mechanism — just make sure the key comes from an environment variable, a secrets manager, or a credential file rather than being hardcoded in source. The `from_netrc` constructor reads `~/.netrc` (or `~/_netrc` on Windows), matching the Java client's `NetrcCredentialsProvider`, which is handy if you manage credentials for multiple servers. Guest access works only when the LabKey server permits anonymous reads.

## Modules

Every endpoint is an async method on `LabkeyClient`. The methods are organized by the LabKey controller they target:

| Module | Description | Endpoints |
|--------|-------------|-----------|
| [`query`] | Select, insert, update, delete rows; execute SQL; manage saved views | 21 |
| [`security`] | Users, groups, containers, permissions, policies, impersonation | 30 |
| [`experiment`] | Lineage queries, run groups, entity sequences, data objects | 12 |
| [`assay`] | Assay designs, runs, batches, NAb study graphs, import | 11 |
| [`domain`] | Domain designs, property usages, name expression validation | 11 |
| [`specimen`] | Specimen repositories, request management, vial operations | 11 |
| [`visualization`] | Saved visualizations and chart configurations | 7 |
| [`report`] | Report creation, execution, and management | 5 |
| [`pipeline`] | Pipeline status, file status, protocols | 4 |
| [`storage`] | Freezer storage items (create, update, delete) | 3 |
| [`di`] | Data integration transform runs and configuration | 3 |
| [`list`] | List creation with shorthand fields | 1 |
| [`message`] | Message board threads | 1 |
| [`participant_group`] | Participant group sessions | 1 |

[`query`]: https://docs.rs/labkey-rs/latest/labkey_rs/query/index.html
[`security`]: https://docs.rs/labkey-rs/latest/labkey_rs/security/index.html
[`experiment`]: https://docs.rs/labkey-rs/latest/labkey_rs/experiment/index.html
[`assay`]: https://docs.rs/labkey-rs/latest/labkey_rs/assay/index.html
[`domain`]: https://docs.rs/labkey-rs/latest/labkey_rs/domain/index.html
[`specimen`]: https://docs.rs/labkey-rs/latest/labkey_rs/specimen/index.html
[`visualization`]: https://docs.rs/labkey-rs/latest/labkey_rs/visualization/index.html
[`report`]: https://docs.rs/labkey-rs/latest/labkey_rs/report/index.html
[`pipeline`]: https://docs.rs/labkey-rs/latest/labkey_rs/pipeline/index.html
[`storage`]: https://docs.rs/labkey-rs/latest/labkey_rs/storage/index.html
[`di`]: https://docs.rs/labkey-rs/latest/labkey_rs/di/index.html
[`list`]: https://docs.rs/labkey-rs/latest/labkey_rs/list/index.html
[`message`]: https://docs.rs/labkey-rs/latest/labkey_rs/message/index.html
[`participant_group`]: https://docs.rs/labkey-rs/latest/labkey_rs/participant_group/index.html

Supporting types live in [`filter`], [`sort`], [`error`], and [`common`].

[`filter`]: https://docs.rs/labkey-rs/latest/labkey_rs/filter/index.html
[`sort`]: https://docs.rs/labkey-rs/latest/labkey_rs/sort/index.html
[`error`]: https://docs.rs/labkey-rs/latest/labkey_rs/error/index.html
[`common`]: https://docs.rs/labkey-rs/latest/labkey_rs/common/index.html

## Filters and sorts

LabKey queries support a rich set of filter operators. Filters are built with the `Filter` type and the `FilterType` enum:

```rust
use labkey_rs::filter::{Filter, FilterType};

let filters = vec![
    Filter::new("Age", FilterType::GreaterThanOrEqual, "18"),
    Filter::new("Status", FilterType::In, "Active;Pending"),
    Filter::new("Name", FilterType::DoesNotStartWith, "Test"),
];
```

Sort specifications use the `QuerySort` type, which parses LabKey's comma-separated sort format:

```rust
use labkey_rs::sort::QuerySort;

// Parse LabKey's sort string format: column names, "-" prefix for descending
let sort = QuerySort::parse("Name,-Created");
assert_eq!(sort.to_string(), "Name,-Created");
```

Both filters and sorts are passed to query methods through their options builders.

## Error handling

All client methods return `Result<T, LabkeyError>`. The error type covers network failures, structured API errors, deserialization problems, and unexpected responses:

```rust,no_run
use labkey_rs::{LabkeyClient, LabkeyError};
use labkey_rs::query::SelectRowsOptions;

async fn handle_query(client: &LabkeyClient) -> Result<(), LabkeyError> {
    let options = SelectRowsOptions::builder()
        .schema_name("core")
        .query_name("Users")
        .build();

    match client.select_rows(options).await {
        Ok(response) => println!("{} rows", response.row_count.unwrap_or(0)),
        Err(LabkeyError::Api { status, body }) => {
            eprintln!("Server error ({}): {:?}", status, body.exception);
        }
        Err(e) => eprintln!("Other error: {e}"),
    }
    Ok(())
}
```

LabKey sometimes returns HTTP 200 with an embedded exception in the JSON body instead of a proper error status code. The client detects this and returns `LabkeyError::Api` rather than a confusing deserialization error.

## Compatibility

This crate targets LabKey Server's API version 17.1, which is the response format used by all modern LabKey Server releases. The `apiVersion=17.1` parameter is sent on every request. All response types are modeled against this format.

The crate is pre-1.0 and the API may change. It has not yet been published to crates.io.

## LabKey documentation

For background on the server-side concepts this client interacts with:

- [LabKey Client APIs overview](https://www.labkey.org/Documentation/wiki-page.view?name=viewApis) — hub page for all client API languages
- [HTTP Interface](https://www.labkey.org/Documentation/wiki-page.view?name=remoteAPIs) — the raw REST endpoints this crate wraps
- [LabKey SQL Reference](https://www.labkey.org/Documentation/wiki-page.view?name=labkeySql) — SQL dialect used by `execute_sql`
- [Filtering Expressions](https://www.labkey.org/Documentation/wiki-page.view?name=filteringExpressions) — filter operator reference
- [API Resources](https://www.labkey.org/Documentation/wiki-page.view?name=apiResources) — general API documentation

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT), at your option.

The upstream JavaScript client is licensed under Apache 2.0.
