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
    let client = LabkeyClient::new(ClientConfig::new(
        "https://labkey.example.com/labkey",
        Credential::ApiKey("your-api-key".into()),
        "/MyProject",
    ))?;

    let response = client.select_rows(
        SelectRowsOptions::builder()
            .schema_name("core")
            .query_name("Users")
            .filters(vec![
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

The client supports three credential types:

```rust,no_run
use labkey_rs::Credential;

// API key (recommended for programmatic access)
let cred = Credential::ApiKey("your-api-key".into());

// Basic auth
let cred = Credential::Basic {
    email: "user@example.com".into(),
    password: "secret".into(),
};

// Read credentials from ~/.netrc
let cred = Credential::from_netrc("labkey.example.com")
    .expect("credentials found in .netrc");

// Guest (anonymous, no auth header)
let cred = Credential::Guest;
```

API keys are the standard choice for scripts and automation. The `from_netrc` constructor reads `~/.netrc` (or `~/_netrc` on Windows), which is useful for keeping credentials out of source code. Guest access works only when the LabKey server permits anonymous reads.

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
use labkey_rs::LabkeyError;
# async fn example(client: labkey_rs::LabkeyClient) {

match client.select_rows(/* ... */
#   labkey_rs::query::SelectRowsOptions::builder()
#       .schema_name("core").query_name("Users").build(),
).await {
    Ok(response) => println!("{} rows", response.row_count.unwrap_or(0)),
    Err(LabkeyError::Api { status, body }) => {
        eprintln!("Server error ({}): {:?}", status, body.exception);
    }
    Err(e) => eprintln!("Other error: {e}"),
}
# }
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
