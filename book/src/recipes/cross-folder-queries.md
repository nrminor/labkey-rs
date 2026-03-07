# Cross-Folder Queries

By default, a query runs against the client's default container (folder). To include data from subfolders, parent folders, or the entire project, use a `ContainerFilter`.

## Setting a container filter

Pass a `ContainerFilter` variant through the `container_filter` field on query options:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::SelectRowsOptions;
# use labkey_rs::filter::ContainerFilter;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/MyProject",
# ))?;
let options = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Samples")
    .container_filter(ContainerFilter::CurrentAndSubfolders)
    .build();

let response = client.select_rows(options).await?;
println!("{} rows across all subfolders", response.row_count);
# Ok(())
# }
```

## Available scopes

The `ContainerFilter` enum provides these scopes:

| Variant | What it includes |
|---|---|
| `Current` | The target container only (the default) |
| `CurrentAndSubfolders` | The target container and all its subfolders |
| `CurrentAndFirstChildren` | The target container and its direct children (not workbooks) |
| `CurrentAndParents` | The target container and its parent containers |
| `CurrentPlusProject` | The target container and the project that contains it |
| `CurrentPlusProjectAndShared` | The target container, its project, and shared folders |
| `AllInProject` | The entire project and all folders in it |
| `AllInProjectPlusShared` | The entire project, all folders, and the Shared project |
| `CurrentAndSubfoldersPlusShared` | The target container, subfolders, and the Shared folder |
| `AllFolders` | Every folder the user has read permission on |

The user's permissions still apply — a container filter can widen the scope, but the server only returns data from containers where the authenticated user has read access.

## Combining with container_path

You can combine `container_filter` with `container_path` to query from a different starting point than the client's default:

```rust,no_run
# use labkey_rs::query::SelectRowsOptions;
# use labkey_rs::filter::ContainerFilter;
let options = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Samples")
    .container_path("/MyProject/Lab1".to_string())
    .container_filter(ContainerFilter::CurrentAndSubfolders)
    .build();
```

This queries `Lab1` and all its subfolders, regardless of what the client's default container is.

## Limitations

Not all data types support cross-container queries. For tables that don't, the container filter is silently ignored and the query runs against the current container only. LabKey's built-in tables (like `core.Users`) generally support cross-container queries, but user-created lists may not, depending on server configuration.
