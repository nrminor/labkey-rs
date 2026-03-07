# Querying Data

This guide covers reading data from a LabKey server. The primary method is `select_rows`, which maps to LabKey's `selectRows` API. There are also methods for running LabKey SQL, fetching distinct values, and introspecting schemas and queries.

All examples assume you already have a `LabkeyClient` configured. If not, see [Getting Started](./getting-started.md).

## select_rows

`select_rows` is the workhorse for reading data. You specify a schema and query name, and the server returns matching rows:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::SelectRowsOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let options = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .build();

let response = client.select_rows(options).await?;
println!("{} rows returned", response.row_count);
# Ok(())
# }
```

`schema_name` and `query_name` are the only required fields. Everything else is optional and defaults to the server's behavior when omitted.

### Selecting specific columns

By default, the server returns the columns defined in the query's default view. To request specific columns, pass a `columns` vector:

```rust,no_run
# use labkey_rs::query::SelectRowsOptions;
let options = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .columns(vec![
        "ParticipantId".to_string(),
        "Name".to_string(),
        "EnrollmentDate".to_string(),
    ])
    .build();
```

You can include lookup columns using the slash notation that LabKey expects (for example, `"Department/Name"` to follow a foreign key into the Department table and return its Name column).

### Limiting rows and pagination

Use `max_rows` and `offset` to control how many rows come back and where to start:

```rust,no_run
# use labkey_rs::query::SelectRowsOptions;
// First page: rows 0-99
let page1 = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .max_rows(100)
    .offset(0)
    .build();

// Second page: rows 100-199
let page2 = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .max_rows(100)
    .offset(100)
    .build();
```

To request all rows at once, pass a negative `max_rows` value (like `-1`). Be cautious with this on large tables. The [Paginate Results](../recipes/paginate-results.md) recipe shows a complete pagination loop.

### Including the total count

By default, the response's `row_count` field reflects only the number of rows returned in this response. To get the total number of matching rows (useful for pagination), set `include_total_count`:

```rust,no_run
# use labkey_rs::query::SelectRowsOptions;
let options = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .max_rows(100)
    .include_total_count(true)
    .build();
```

### Container path override

Every request uses the client's default container path unless you override it:

```rust,no_run
# use labkey_rs::query::SelectRowsOptions;
let options = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .container_path("/MyProject/SubFolder".to_string())
    .build();
```

### Filters and sorts

`select_rows` accepts filters and sorts through the `filter_array` and `sort` fields. These are covered in depth in the [Filters and Sorts](./filters-and-sorts.md) guide.

### Using POST for large requests

By default, `select_rows` sends parameters as URL query strings via GET. If your request has many filters or a long column list, the URL can exceed server limits. Set the `method` field to use POST instead:

```rust,no_run
# use labkey_rs::query::{SelectRowsOptions, RequestMethod};
let options = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .method(RequestMethod::Post)
    .build();
```

## The response

`select_rows` returns a `SelectRowsResponse`. The key fields are:

- **`row_count`** (`i64`) — the number of rows in this response (or the total count if `include_total_count` was set)
- **`rows`** (`Vec<Row>`) — the result rows
- **`meta_data`** (`Option<ResponseMetadata>`) — column metadata, included by default

### Rows and CellValue

Each `Row` has a `data` field that maps column names to `CellValue` structs. A `CellValue` wraps the raw JSON value with optional display metadata:

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
    if let Some(cell) = row.data.get("Name") {
        // The raw value — a serde_json::Value (string, number, bool, or null)
        println!("value: {}", cell.value);

        // For lookup columns, the human-readable display text
        if let Some(display) = &cell.display_value {
            println!("display: {display}");
        }

        // For columns with server-side formatting
        if let Some(formatted) = &cell.formatted_value {
            println!("formatted: {formatted}");
        }
    }
}
# Ok(())
# }
```

The `value` field is always present. `display_value` appears for lookup columns where the stored value is a foreign key but you want the human-readable label. `formatted_value` appears when the column has a display format configured on the server. There are also `url`, `mv_value`, and `mv_indicator` fields for links and missing-value indicators.

### Column metadata

When `include_metadata` is not explicitly set to `false`, the response includes a `meta_data` field with column definitions:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::SelectRowsOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
# let response = client.select_rows(SelectRowsOptions::builder()
#     .schema_name("core").query_name("Users").build()).await?;
if let Some(meta) = &response.meta_data {
    for col in &meta.fields {
        println!(
            "{}: type={:?}, nullable={}, read_only={}",
            col.name,
            col.json_type.as_deref().unwrap_or("unknown"),
            col.nullable,
            col.read_only,
        );
    }
}
# Ok(())
# }
```

Each `QueryColumn` includes the column name, JSON and SQL type information, whether it's nullable or read-only, whether it's a key field, and lookup metadata if it references another table.

## execute_sql

When you need more control than `select_rows` provides — joins, aggregations, subqueries — use `execute_sql` to send LabKey SQL directly:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::ExecuteSqlOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let options = ExecuteSqlOptions::builder()
    .schema_name("lists")
    .sql("SELECT Department, COUNT(*) AS Total FROM Participants GROUP BY Department")
    .build();

let response = client.execute_sql(options).await?;
for row in &response.rows {
    println!("{:?}", row.data);
}
# Ok(())
# }
```

`execute_sql` returns the same `SelectRowsResponse` type as `select_rows`, so you work with the results in exactly the same way. The `schema_name` provides the execution context for resolving table names in your SQL. LabKey SQL is similar to standard SQL but has some extensions and limitations — see LabKey's [SQL Reference](https://www.labkey.org/Documentation/wiki-page.view?name=labkeySql) for the full syntax.

The [LabKey SQL](../recipes/labkey-sql.md) recipe has more examples of what you can do with `execute_sql`.

## select_distinct_rows

To get the unique values of a single column, use `select_distinct_rows`:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::SelectDistinctOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let options = SelectDistinctOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .column("Department")
    .build();

let response = client.select_distinct_rows(options).await?;
for value in &response.values {
    println!("{value}");
}
# Ok(())
# }
```

The response is a `SelectDistinctResponse` with a `values` field containing a `Vec<serde_json::Value>`. This is useful for populating dropdowns or validating input against known values. You can apply filters and sorts just like with `select_rows`.

## Schema introspection

Two methods help you discover what data is available on a server.

### get_schemas

`get_schemas` returns the schemas available in a container. The response is a `serde_json::Value` (the server returns a nested object keyed by schema name, which doesn't map cleanly to a single typed struct):

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::GetSchemasOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let schemas = client
    .get_schemas(GetSchemasOptions::builder().build())
    .await?;

println!("{schemas:#}");
# Ok(())
# }
```

### get_queries

`get_queries` lists the queries (tables and views) available within a schema:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::GetQueriesOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let response = client
    .get_queries(
        GetQueriesOptions::builder()
            .schema_name("lists")
            .build(),
    )
    .await?;

for query in &response.queries {
    println!("{}", query.name);
}
# Ok(())
# }
```

The response is a `GetQueriesResponse` with a `queries` field containing `Vec<QueryInfo>`. Each `QueryInfo` has the query's `name`, optional `title` and `description`, and optionally column metadata if you set `include_columns(true)`.

### get_query_details

For detailed metadata about a specific query — its columns, types, key fields, and lookup relationships — use `get_query_details`:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::GetQueryDetailsOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let details = client
    .get_query_details(
        GetQueryDetailsOptions::builder()
            .schema_name("lists")
            .query_name("Participants")
            .build(),
    )
    .await?;

println!("{details:#?}");
# Ok(())
# }
```

This is particularly useful when you need to know a table's primary key column before constructing update or delete payloads, or when you want to understand the lookup relationships between tables.

## What's next

Now that you can read data, the natural next steps are:

- **[Filters and Sorts](./filters-and-sorts.md)** — narrow your queries with filter operators and sort specifications
- **[Modifying Data](./modifying-data.md)** — insert, update, and delete rows
- **[Paginate Results](../recipes/paginate-results.md)** — loop through large result sets page by page
- **[LabKey SQL](../recipes/labkey-sql.md)** — more examples of `execute_sql` for complex queries
