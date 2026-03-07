# Bulk Import

For importing large datasets, `import_data` is more efficient than `insert_rows`. It accepts data in several formats and can run asynchronously as a pipeline job. This recipe walks through the common patterns.

## Importing CSV text

The simplest approach is to pass CSV content directly as a string:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::{ImportDataOptions, ImportDataSource};
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let csv = "\
Name,Department,StartDate
Alice,Engineering,2024-01-15
Bob,Research,2024-02-01
Charlie,Operations,2024-03-10";

let options = ImportDataOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .source(ImportDataSource::Text(csv.to_string()))
    .build();

let response = client.import_data(options).await?;
println!("success: {}, rows: {:?}", response.success, response.row_count);
# Ok(())
# }
```

The server auto-detects CSV vs. TSV based on the content. You can also set `.format("tsv".to_string())` explicitly if needed.

## Uploading a file

To upload a file from disk as a multipart form:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::{ImportDataOptions, ImportDataSource};
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let bytes = std::fs::read("data/participants.csv")?;

let options = ImportDataOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .source(ImportDataSource::File {
        file_name: "participants.csv".to_string(),
        bytes,
        mime_type: Some("text/csv".to_string()),
    })
    .build();

let response = client.import_data(options).await?;
println!("success: {}, rows: {:?}", response.success, response.row_count);
# Ok(())
# }
```

## Merging updates

By default, `import_data` inserts new rows. If you want to update existing rows (matching on the primary key) while also inserting new ones, use the `Merge` insert option:

```rust,no_run
# use labkey_rs::query::{ImportDataOptions, ImportDataSource, InsertOption};
let options = ImportDataOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .source(ImportDataSource::Text("Key,Name\n1,Alice Updated\n999,New Person".to_string()))
    .insert_option(InsertOption::Merge)
    .build();
```

The server matches rows by primary key — existing keys are updated, new keys are inserted.

## Async pipeline jobs

For very large imports that might take a while, you can queue the work as an asynchronous pipeline job:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::{ImportDataOptions, ImportDataSource};
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let bytes = std::fs::read("data/large_dataset.csv")?;

let options = ImportDataOptions::builder()
    .schema_name("lists")
    .query_name("Measurements")
    .source(ImportDataSource::File {
        file_name: "large_dataset.csv".to_string(),
        bytes,
        mime_type: Some("text/csv".to_string()),
    })
    .use_async(true)
    .build();

let response = client.import_data(options).await?;
if let Some(job_id) = &response.job_id {
    println!("Import queued as pipeline job: {job_id}");
}
# Ok(())
# }
```

When `use_async` is set, the server returns immediately with a `job_id`. The actual import runs in the background as a pipeline job. You can check job status through LabKey's pipeline UI or API.

## Server-side file paths

If the data file is already on the server (for example, uploaded via WebDAV), you can reference it by path instead of uploading it again:

```rust,no_run
# use labkey_rs::query::{ImportDataOptions, ImportDataSource};
let options = ImportDataOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .source(ImportDataSource::Path("@files/imports/participants.csv".to_string()))
    .build();
```

The path is relative to the container's WebDAV root.

## import_data vs. insert_rows

Use `import_data` when you have tabular data (CSV, TSV, or a file) and want the server to handle parsing. Use `insert_rows` when you're constructing row payloads programmatically as JSON objects. `import_data` is generally faster for large datasets because it sends the data in a more compact format and the server can optimize the import path.
