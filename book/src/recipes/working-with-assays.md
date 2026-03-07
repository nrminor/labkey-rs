# Working with Assays

LabKey assays are organized into a hierarchy: an assay design (protocol) defines the schema, batches group related imports, and each batch contains runs with result data. This recipe covers the most common assay operations.

## Listing assay designs

Use `get_assays` to discover what assay designs are available in a container:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::assay::GetAssaysOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let designs = client
    .get_assays(GetAssaysOptions::builder().build())
    .await?;

for design in &designs {
    println!("{}: {} (id: {})", design.name, design.type_, design.id);
}
# Ok(())
# }
```

You can filter by name, id, type, or status using the builder fields. The response is a `Vec<AssayDesign>` with fields for the design's name, id, type, description, domains, and links.

## Importing run data

`import_run` is the primary way to add data to an assay. You need the assay's numeric id and a data source:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::assay::{ImportRunOptions, ImportRunSource};
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
# let assay_id = 1;
// Import from inline row data
let response = client
    .import_run(
        ImportRunOptions::builder()
            .assay_id(assay_id)
            .source(ImportRunSource::DataRows(vec![
                serde_json::json!({"SampleId": "S001", "Value": 42.5}),
                serde_json::json!({"SampleId": "S002", "Value": 38.1}),
            ]))
            .name("Experiment 2024-03-15".to_string())
            .build(),
    )
    .await?;

println!("success: {}, run_id: {:?}", response.success, response.run_id);
# Ok(())
# }
```

`ImportRunSource` has three variants:

- **`DataRows(Vec<serde_json::Value>)`** — inline JSON row data
- **`File { data, filename }`** — upload file bytes (CSV, TSV, Excel)
- **`RunFilePath(String)`** — reference a file already on the server

You can also set run-level and batch-level properties, attach the run to an existing batch, and control whether the import runs asynchronously.

## Uploading a file

```rust,no_run
# use labkey_rs::assay::{ImportRunOptions, ImportRunSource};
# let assay_id = 1;
let bytes = std::fs::read("results.csv").unwrap();

let options = ImportRunOptions::builder()
    .assay_id(assay_id)
    .source(ImportRunSource::File {
        data: bytes,
        filename: "results.csv".to_string(),
    })
    .build();
```

## Reading assay results

Assay results are stored in regular LabKey queries under the `assay` schema. You can read them with `select_rows` or `execute_sql` like any other data:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::SelectRowsOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
// The schema name follows the pattern: assay.{provider}.{design name}
let options = SelectRowsOptions::builder()
    .schema_name("assay.General.My Assay Design")
    .query_name("Data")
    .max_rows(50)
    .build();

let response = client.select_rows(options).await?;
for row in &response.rows {
    println!("{:?}", row.data);
}
# Ok(())
# }
```

The schema name for assay results follows the pattern `assay.{ProviderName}.{DesignName}`. The query name is typically `Data` for result rows, `Runs` for run metadata, or `Batches` for batch metadata.

## NAb assays

For Neutralizing Antibody (NAb) assays, there's a dedicated `get_nab_runs` method that returns NAb-specific data including dilution curves and neutralization calculations:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::assay::GetNabRunsOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let runs = client
    .get_nab_runs(
        GetNabRunsOptions::builder()
            .assay_name("My NAb Assay".to_string())
            .include_stats(true)
            .include_fit_parameters(true)
            .build(),
    )
    .await?;

println!("{} NAb runs", runs.len());
# Ok(())
# }
```

The response is a `Vec<serde_json::Value>` because NAb run data has a complex, variable structure that depends on the assay configuration.

## Further reading

The `assay` module also provides `save_assay_batch` for creating or updating batches, `save_assay_runs` for saving run metadata, and `get_assay_batch` for fetching a specific batch. See the [API reference](https://docs.rs/labkey-rs/latest/labkey_rs/assay/index.html) for the full set of assay methods.
