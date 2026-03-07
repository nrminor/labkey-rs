---
name: labkey-rs
description: Unofficial Rust client for the LabKey Server REST API. Use when writing Rust code that queries or manages data on a LabKey server, including selecting rows, executing SQL, inserting or updating data, managing security and permissions, working with assays, experiments, domains, specimens, or any other LabKey module. Also use when the user mentions LabKey, laboratory information management systems (LIMS), or scientific data management in a Rust context.
license: Apache-2.0 OR MIT
metadata:
  repository: https://github.com/nicminor/labkey-rs
---

# labkey-rs

Typed, async Rust client for the LabKey Server REST API. Ported from the official `@labkey/api` JavaScript client (v1.48.0), supplemented by the Java client for endpoint coverage.

The crate exposes a single `LabkeyClient` struct. Every LabKey endpoint is an async method on this struct that takes an options struct and returns `Result<Response, LabkeyError>`.

## Client construction

Build a `ClientConfig`, then pass it to `LabkeyClient::new`:

```rust
use labkey_rs::{ClientConfig, Credential, LabkeyClient};

let api_key = std::env::var("LABKEY_API_KEY")?;

let client = LabkeyClient::new(ClientConfig::new(
    "https://labkey.example.com/labkey",   // base URL including context path
    Credential::ApiKey(api_key),
    "/MyProject/MyFolder",                 // container path, must start with /
))?;
```

The four credential types:

```rust
// API key from environment variable or secrets manager (recommended)
Credential::ApiKey(std::env::var("LABKEY_API_KEY")?)

// Basic auth (email + password — avoid hardcoding these)
Credential::Basic { email: "user@example.com".into(), password: "secret".into() }

// Read from a ~/.netrc file (handy for managing multiple servers)
Credential::from_netrc("labkey.example.com")?

// Anonymous access (only works if the server allows guest reads)
Credential::Guest
```

Never hardcode API keys or passwords as string literals. Use environment variables, a secrets manager, or a `.netrc` file.

`ClientConfig` has optional builder methods chained before passing to `LabkeyClient::new`:

```rust
let config = ClientConfig::new(base_url, credential, container_path)
    .with_accept_self_signed_certs(true)  // for dev servers
    .with_proxy_url("http://proxy:8080")
    .with_csrf_token("token-value")       // rarely needed with API keys
    .with_user_agent("my-app/1.0");
```

## The builder pattern

Every endpoint method takes an options struct built with `bon::Builder`. The pattern is always the same:

```rust
let opts = SomeOptions::builder()
    .required_field("value")       // non-Option<T> fields — must be set
    .optional_field("value")       // Option<T> fields — omit to use default
    .build();

let response = client.some_method(opts).await?;
```

Required fields are the non-`Option` struct fields. Optional fields are `Option<T>` and can simply be omitted from the builder chain. All options structs are `#[non_exhaustive]`, so always use the builder rather than struct literal syntax.

## Querying data

### select_rows

The primary read endpoint. Returns rows from a LabKey query or table:

```rust
use labkey_rs::query::SelectRowsOptions;
use labkey_rs::filter::{Filter, FilterType};
use labkey_rs::sort::QuerySort;

let response = client.select_rows(
    SelectRowsOptions::builder()
        .schema_name("lists")
        .query_name("Samples")
        .filter_array(vec![
            Filter::new("Status", FilterType::Equal, "Active"),
        ])
        .sort(QuerySort::parse("Name,-Created"))
        .max_rows(100)
        .offset(0)
        .build(),
).await?;

for row in &response.rows {
    // Each cell is a CellValue, not a raw JSON value
    if let Some(cell) = row.data.get("Name") {
        println!("{}", cell.value); // the raw serde_json::Value
    }
}
```

### execute_sql

Run arbitrary LabKey SQL:

```rust
use labkey_rs::query::ExecuteSqlOptions;

let response = client.execute_sql(
    ExecuteSqlOptions::builder()
        .schema_name("core")
        .sql("SELECT UserId, Email FROM core.Users WHERE Active = true")
        .max_rows(50)
        .build(),
).await?;
```

The SQL dialect is LabKey SQL, not standard SQL. Column and table references use LabKey's schema-qualified naming.

### Pagination

Use `max_rows` and `offset` together. The response includes `row_count` (total matching rows, if `include_total_count` is set) and the actual `rows` vector:

```rust
let opts = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("LargeTable")
    .max_rows(500)
    .offset(0)
    .include_total_count(true)
    .build();
```

For very large filter sets or many columns, use `RequestMethod::Post` to avoid URL length limits:

```rust
use labkey_rs::query::{SelectRowsOptions, RequestMethod};

let opts = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("WideTable")
    .method(RequestMethod::Post)
    .build();
```

### Mutations

Insert, update, and delete follow the same builder pattern. Each takes `schema_name`, `query_name`, and a `rows` vector of `serde_json::Map<String, Value>`:

```rust
use labkey_rs::query::InsertRowsOptions;
use serde_json::{Map, Value};

let mut row = Map::new();
row.insert("Name".into(), Value::String("New Sample".into()));
row.insert("Status".into(), Value::String("Active".into()));

let response = client.insert_rows(
    InsertRowsOptions::builder()
        .schema_name("lists")
        .query_name("Samples")
        .rows(vec![row])
        .build(),
).await?;
```

Update and delete work identically but require the row's primary key field(s) to be present in each row map.

## Filters and sorts

Filters use the `Filter` struct and `FilterType` enum:

```rust
use labkey_rs::filter::{Filter, FilterType};

let filters = vec![
    Filter::new("Age", FilterType::GreaterThanOrEqual, "18"),
    Filter::new("Status", FilterType::In, "Active;Pending"),  // semicolon-separated
    Filter::new("Name", FilterType::DoesNotStartWith, "Test"),
    Filter::new("Notes", FilterType::IsNotBlank, ""),          // no value needed
];
```

`FilterType` has variants for all LabKey operators: comparison, set membership, string matching, range, null/blank checks, date-specific variants, and more. The enum is `#[non_exhaustive]`.

Sorts use `QuerySort`, which parses LabKey's comma-separated format where `-` prefix means descending:

```rust
use labkey_rs::sort::QuerySort;

let sort = QuerySort::parse("LastName,-Created,Status");
```

Both are passed to query options via the builder.

## Error handling

All methods return `Result<T, LabkeyError>`. The error variants:

- `LabkeyError::Http` — network/connection failures (wraps `reqwest::Error`)
- `LabkeyError::Api { status, body }` — server returned an error with a structured `ApiErrorBody`
- `LabkeyError::UnexpectedResponse { status, text }` — non-success status but body wasn't parseable as a LabKey error
- `LabkeyError::Deserialization { source, body }` — response JSON didn't match the expected type
- `LabkeyError::InvalidInput(String)` — client-side validation failure before the request was sent
- `LabkeyError::Url` — invalid URL (wraps `url::ParseError`)

LabKey sometimes returns HTTP 200 with an exception embedded in the JSON body instead of a proper error status. The client detects this automatically and returns `LabkeyError::Api` rather than a confusing deserialization error.

```rust
use labkey_rs::LabkeyError;

match client.select_rows(opts).await {
    Ok(response) => { /* use response */ }
    Err(LabkeyError::Api { status, body }) => {
        eprintln!("Server error ({}): {:?}", status, body.exception);
    }
    Err(e) => eprintln!("Other error: {e}"),
}
```

## Response structure

Query responses use the LabKey 17.1 response format (sent on every request via `apiVersion=17.1`). Each row is a `Row` struct containing a `data: HashMap<String, CellValue>` map. `CellValue` is not a raw JSON value — it wraps the value with optional metadata:

```rust
pub struct CellValue {
    pub value: serde_json::Value,          // the actual data
    pub display_value: Option<String>,     // for lookup columns
    pub formatted_value: Option<String>,   // server-formatted string
    pub url: Option<String>,               // link for lookup columns
    pub mv_value: Option<String>,          // missing-value indicator
    pub mv_indicator: Option<String>,      // missing-value code
}
```

Always access `cell.value` for the raw data. Use `cell.display_value` when you need the human-readable form of a lookup column.

## Module map

Every endpoint is an async method on `LabkeyClient`. The methods are organized by LabKey controller:

| Module | LabKey controller | Key operations |
|--------|-------------------|----------------|
| `query` | query | select_rows, execute_sql, insert/update/delete_rows, get_schemas, get_queries, truncate_table |
| `security` | security | users, groups, containers, permissions, policies, impersonation |
| `experiment` | experiment | lineage, run groups, entity sequences, batches, materials |
| `assay` | assay | assay designs, runs, batches, NAb graphs, import |
| `domain` | property | domain designs, property usages, name expression validation |
| `specimen` | specimen | repositories, requests, vial operations |
| `visualization` | visualization | saved visualizations, measures, dimensions |
| `report` | reports | R/Python report sessions, execution |
| `pipeline` | pipeline-status | pipeline status, file status, protocols |
| `storage` | storage | freezer storage items (create, update, delete) |
| `di` | di | data integration transform runs |
| `list` | list | list creation with shorthand field definitions |
| `message` | announcements | message board threads |
| `participant_group` | participant-group | participant group sessions |

Supporting types live in `filter` (query filters), `sort` (sort specifications), `error` (error types), and `common` (shared enums like `ContainerFilter` and `AuditBehavior`).

## Common mistakes

**Forgetting the leading `/` on container paths.** Container paths like `"/MyProject/MyFolder"` must start with `/`. The client does not add it for you.

**Treating row data as raw JSON.** Query responses use `CellValue` wrappers, not bare `serde_json::Value`. Access the actual data through `cell.value`.

**Not handling the `Option` wrappers on response fields.** Fields like `response.row_count` are `Option<i64>` because the server only includes them when requested (e.g., `include_total_count(true)`).

**Using struct literals instead of builders.** All options structs are `#[non_exhaustive]`, so `SelectRowsOptions { schema_name: ..., .. }` won't compile. Always use `SelectRowsOptions::builder()`.

**Assuming standard SQL.** `execute_sql` uses LabKey SQL, which has its own syntax for schema-qualified names, lookups, and special columns. See the LabKey SQL Reference in the server documentation.

## Per-request container override

Most options structs have an optional `container_path` field that overrides the client's default for that single request:

```rust
let opts = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Samples")
    .container_path("/DifferentProject/SubFolder")
    .build();
```

This is useful when a single client needs to query across multiple LabKey folders.
