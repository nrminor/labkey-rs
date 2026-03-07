# Getting Started

This guide walks through adding labkey-rs to your project, constructing a client, making your first query, and inspecting the response.

## Add the dependency

labkey-rs is an async library built on [reqwest](https://docs.rs/reqwest) and [tokio](https://docs.rs/tokio). Add both to your `Cargo.toml`:

```toml
[dependencies]
labkey-rs = "0.1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

You'll also want `serde_json` if you plan to construct row payloads for insert/update/delete operations, but it's not needed for read-only queries.

## Configure the client

Every interaction with a LabKey server goes through a `LabkeyClient`. To construct one, you need three things: the server's base URL, credentials, and a default container path.

```rust,no_run
use labkey_rs::{ClientConfig, Credential, LabkeyClient};

let api_key = std::env::var("LABKEY_API_KEY")
    .expect("LABKEY_API_KEY must be set");

let config = ClientConfig::new(
    "https://labkey.example.com/labkey",  // base URL
    Credential::ApiKey(api_key),           // credentials
    "/MyProject",                          // default container path
);

let client = LabkeyClient::new(config)
    .expect("valid client configuration");
```

The **base URL** is the root of your LabKey server, including any context path. If you access your server at `https://labkey.example.com/labkey/project/begin.view`, the base URL is `https://labkey.example.com/labkey`.

The **container path** is the LabKey folder where your data lives, like `/MyProject` or `/MyProject/SubFolder`. Individual requests can override this, but the default is used when no override is specified. See [How LabKey Works](./how-labkey-works.md) for more on containers.

The **credentials** determine how the client authenticates. API keys are the most common choice. The [Authentication](./authentication.md) guide covers all four credential types in detail.

> **Never hardcode API keys or passwords in source code.** Use environment variables, a secrets manager, or a `.netrc` file.

## Make your first query

The most common operation is reading data with `select_rows`. You specify a schema and query (table) name, and the server returns the matching rows:

```rust,no_run
use labkey_rs::{ClientConfig, Credential, LabkeyClient};
use labkey_rs::query::SelectRowsOptions;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let api_key = std::env::var("LABKEY_API_KEY")?;
let config = ClientConfig::new(
    "https://labkey.example.com/labkey",
    Credential::ApiKey(api_key),
    "/MyProject",
);
let client = LabkeyClient::new(config)?;

let options = SelectRowsOptions::builder()
    .schema_name("core")
    .query_name("Users")
    .max_rows(10)
    .build();

let response = client.select_rows(options).await?;

println!("Returned {} row(s)", response.row_count);
# Ok(())
# }
```

Options are constructed with a builder pattern. `schema_name` and `query_name` are required; everything else is optional and defaults to the server's behavior when omitted. The builder is provided by the [bon](https://docs.rs/bon) crate, so setter methods take owned `String` values (not `&str`).

## Inspect the response

`select_rows` returns a `SelectRowsResponse` containing a `Vec<Row>`. Each `Row` has a `data` field that maps column names to `CellValue` structs:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::SelectRowsOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
# let response = client.select_rows(SelectRowsOptions::builder()
#     .schema_name("core").query_name("Users").build()).await?;
for row in &response.rows {
    for (column_name, cell) in &row.data {
        // cell.value is a serde_json::Value (string, number, bool, or null)
        // cell.display_value is Some(...) for lookup columns
        println!("{column_name}: {}", cell.value);
    }
}
# Ok(())
# }
```

The `CellValue` struct wraps the raw JSON value with optional metadata. The `value` field is always present and is a `serde_json::Value`. For lookup columns, `display_value` contains the human-readable text. For columns with custom formatting, `formatted_value` contains the server-formatted string.

## Handle errors

All client methods return `Result<T, LabkeyError>`. For a quick start, the `?` operator and `Box<dyn Error>` work fine:

```rust,no_run
use labkey_rs::{LabkeyClient, LabkeyError};
use labkey_rs::query::SelectRowsOptions;

async fn query(client: &LabkeyClient) -> Result<(), LabkeyError> {
    let options = SelectRowsOptions::builder()
        .schema_name("core")
        .query_name("Users")
        .build();

    let response = client.select_rows(options).await?;
    println!("{} rows", response.row_count);
    Ok(())
}
```

When you need more control, you can match on specific error variants. The [Error Handling](./error-handling.md) guide covers this in depth, and the [`error_handling` example](https://github.com/nrminor/labkey-rs/tree/main/examples/error_handling.rs) demonstrates the pattern.

## Next steps

Now that you can connect and query, here are some directions to explore:

- **[How LabKey Works](./how-labkey-works.md)** — understand the concepts behind containers, schemas, and queries
- **[Querying Data](./querying-data.md)** — `select_rows` in depth, plus `execute_sql` and schema introspection
- **[Filters and Sorts](./filters-and-sorts.md)** — narrow your queries with filter operators and sort specifications
- **[Modifying Data](./modifying-data.md)** — insert, update, and delete rows
- **[Controller-to-module map](../introduction.md#controller-to-module-map)** — find the right module for a LabKey controller
