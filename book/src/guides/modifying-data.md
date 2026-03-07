# Modifying Data

This guide covers writing data to a LabKey server: inserting new rows, updating existing ones, deleting rows, and bulk operations. All mutation methods require appropriate permissions on the target container — typically "Editor" or above.

All examples assume you already have a `LabkeyClient` configured. If not, see [Getting Started](./getting-started.md).

## Constructing row payloads

Row payloads are `serde_json::Value` objects. Each object is a JSON map where keys are column names and values are the data to write. You'll use `serde_json::json!` for this:

```rust,no_run
use serde_json::json;

let row = json!({
    "Name": "Alice",
    "Department": "Engineering",
    "StartDate": "2024-01-15"
});
```

Column names must match the server's column names exactly (they're case-sensitive). For lookup columns, you typically provide the foreign key value, not the display value. Use `get_query_details` (described in [Querying Data](./querying-data.md)) to discover the exact column names and types for a table.

## insert_rows

`insert_rows` adds new rows to a table:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::InsertRowsOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
use serde_json::json;

let options = InsertRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .rows(vec![
        json!({"Name": "Alice", "Department": "Engineering"}),
        json!({"Name": "Bob", "Department": "Research"}),
    ])
    .build();

let result = client.insert_rows(options).await?;
println!("{} rows inserted", result.rows_affected);
# Ok(())
# }
```

The response is a `ModifyRowsResults` with `rows_affected` (the count of inserted rows) and `rows` (the inserted rows as returned by the server, which may include auto-generated fields like primary keys).

### Transactional inserts

By default, each row is inserted independently — if the third row fails, the first two are still committed. To make all rows succeed or fail together, set `transacted`:

```rust,no_run
# use labkey_rs::query::InsertRowsOptions;
# use serde_json::json;
let options = InsertRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .rows(vec![
        json!({"Name": "Alice"}),
        json!({"Name": "Bob"}),
    ])
    .transacted(true)
    .build();
```

## update_rows

`update_rows` modifies existing rows. Each row payload must include the primary key column so the server knows which row to update:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::UpdateRowsOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
use serde_json::json;

let options = UpdateRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .rows(vec![
        json!({"Key": 1, "Department": "Data Science"}),
        json!({"Key": 2, "Department": "Data Science"}),
    ])
    .build();

let result = client.update_rows(options).await?;
println!("{} rows updated", result.rows_affected);
# Ok(())
# }
```

Only the columns you include in the payload are updated — omitted columns keep their existing values. The primary key column name varies by table (it might be `Key`, `RowId`, `ParticipantId`, etc.). Use `get_query_details` to find it.

## delete_rows

`delete_rows` removes rows. Like `update_rows`, each row payload must include the primary key:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::DeleteRowsOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
use serde_json::json;

let options = DeleteRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .rows(vec![
        json!({"Key": 1}),
        json!({"Key": 2}),
    ])
    .build();

let result = client.delete_rows(options).await?;
println!("{} rows deleted", result.rows_affected);
# Ok(())
# }
```

## save_rows: mixed operations in one request

When you need to insert, update, and delete rows in a single round trip, `save_rows` lets you batch multiple commands together:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::{SaveRowsOptions, SaveRowsCommand, CommandType};
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
use serde_json::json;

let options = SaveRowsOptions::builder()
    .commands(vec![
        SaveRowsCommand::builder()
            .command(CommandType::Insert)
            .schema_name("lists")
            .query_name("Participants")
            .rows(vec![json!({"Name": "Charlie"})])
            .build(),
        SaveRowsCommand::builder()
            .command(CommandType::Update)
            .schema_name("lists")
            .query_name("Participants")
            .rows(vec![json!({"Key": 1, "Name": "Alice Updated"})])
            .build(),
        SaveRowsCommand::builder()
            .command(CommandType::Delete)
            .schema_name("lists")
            .query_name("Participants")
            .rows(vec![json!({"Key": 99})])
            .build(),
    ])
    .build();

let response = client.save_rows(options).await?;
println!("committed: {}, errors: {}", response.committed, response.error_count);
for result in &response.result {
    println!("{}: {} rows affected", result.command, result.rows_affected);
}
# Ok(())
# }
```

Each `SaveRowsCommand` specifies its own `CommandType` (`Insert`, `Update`, or `Delete`), schema, query, and rows. The commands can even target different tables. The response is a `SaveRowsResponse` with a `committed` flag, an `error_count`, and a `result` vector containing one `ModifyRowsResults` per command.

### Transactional save_rows

Like the individual mutation methods, `save_rows` supports `transacted(true)` to make all commands succeed or fail atomically. There's also `validate_only(true)`, which runs all validation without committing — useful for dry runs.

## import_data: bulk import

For importing large amounts of data, `import_data` is more efficient than `insert_rows`. It accepts data in several formats and can run asynchronously as a pipeline job:

### Importing inline text

The simplest approach is to pass CSV or TSV content directly:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::{ImportDataOptions, ImportDataSource};
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let csv_data = "\
Name,Department,StartDate
Alice,Engineering,2024-01-15
Bob,Research,2024-02-01
Charlie,Engineering,2024-03-10";

let options = ImportDataOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .source(ImportDataSource::Text(csv_data.to_string()))
    .build();

let response = client.import_data(options).await?;
println!("success: {}, rows imported: {:?}", response.success, response.row_count);
# Ok(())
# }
```

### Uploading a file

To upload a file as a multipart form, use `ImportDataSource::File`:

```rust,no_run
# use labkey_rs::query::{ImportDataOptions, ImportDataSource};
let bytes = std::fs::read("participants.csv")?;

let options = ImportDataOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .source(ImportDataSource::File {
        file_name: "participants.csv".to_string(),
        bytes,
        mime_type: Some("text/csv".to_string()),
    })
    .build();
# Ok::<(), Box<dyn std::error::Error>>(())
```

### Other import sources

`ImportDataSource` also supports `Path(String)` for a server-relative path from the WebDAV root, and `ModuleResource { module, module_resource }` for importing from a module's bundled resources.

### Import vs. merge

By default, `import_data` inserts new rows. To merge updates into existing rows (matching on the primary key), set the `insert_option`:

```rust,no_run
# use labkey_rs::query::{ImportDataOptions, ImportDataSource, InsertOption};
let options = ImportDataOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .source(ImportDataSource::Text("Name,Key\nAlice,1".to_string()))
    .insert_option(InsertOption::Merge)
    .build();
```

### Async import

For very large imports, you can queue the work as a pipeline job:

```rust,no_run
# use labkey_rs::query::{ImportDataOptions, ImportDataSource};
let options = ImportDataOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .source(ImportDataSource::Text("Name\nAlice\nBob".to_string()))
    .use_async(true)
    .build();
```

When `use_async` is set, the response's `job_id` field contains the pipeline job identifier. You can poll the server for job status using LabKey's pipeline APIs.

The [Bulk Import](../recipes/bulk-import.md) recipe has a more complete example.

## truncate_table

To delete all rows from a table without specifying individual keys, use `truncate_table`:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::TruncateTableOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let options = TruncateTableOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .build();

let response = client.truncate_table(options).await?;
if let Some(count) = response.deleted_rows {
    println!("{count} rows deleted");
}
# Ok(())
# }
```

The response is a `TruncateTableResponse` where `deleted_rows` is `Option<i64>` — the server may or may not report the count depending on the table type.

> **This is irreversible.** There is no confirmation prompt. Make sure you're targeting the right container and table.

## Audit options

All mutation methods support optional audit fields for compliance and traceability:

```rust,no_run
# use labkey_rs::query::InsertRowsOptions;
# use serde_json::json;
let options = InsertRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .rows(vec![json!({"Name": "Alice"})])
    .audit_user_comment("Batch import from enrollment system".to_string())
    .build();
```

The `audit_user_comment` field attaches a human-readable note to the audit log entry. There's also `audit_behavior` for controlling the audit detail level and `audit_details` for structured metadata.

## What's next

- **[Filters and Sorts](./filters-and-sorts.md)** — narrow queries before modifying data
- **[Error Handling](./error-handling.md)** — understand what happens when mutations fail
- **[Bulk Import](../recipes/bulk-import.md)** — a complete `import_data` workflow
- **[Querying Data](./querying-data.md)** — read data back after modifications
