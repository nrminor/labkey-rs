# How LabKey Works

This page gives a brief orientation to the LabKey concepts you'll encounter when using this client. It is not a comprehensive guide to LabKey Server — for that, see the [official documentation](https://www.labkey.org/Documentation/wiki-page.view?name=docs). The goal here is to explain just enough that the rest of this book makes sense.

## Containers

LabKey organizes everything into a hierarchy of **containers**. A container is essentially a folder that holds data, security settings, and configuration. The hierarchy looks like this:

```text
/                          ← site root
  /MyProject               ← a project (top-level container)
    /MyProject/Lab1         ← a folder within the project
    /MyProject/Lab2         ← another folder
```

Every API request targets a specific container, identified by its path. When you construct a `LabkeyClient`, you provide a default container path. Individual requests can override it when you need to reach data in a different folder.

```rust,no_run
use labkey_rs::{ClientConfig, Credential};

let api_key = std::env::var("LABKEY_API_KEY").unwrap();

// Default container is /MyProject
let config = ClientConfig::new(
    "https://labkey.example.com/labkey",
    Credential::ApiKey(api_key),
    "/MyProject",
);
```

Projects and folders can have different security settings, module configurations, and data. When you query data, you're querying within a specific container unless you use a [container filter](../recipes/cross-folder-queries.md) to widen the scope.

For more on containers, see LabKey's [Projects and Folders](https://www.labkey.org/Documentation/wiki-page.view?name=projects) documentation.

## Schemas and queries

Within a container, data is organized into **schemas** and **queries**. A schema is a namespace that groups related tables and queries. A query is either a built-in table or a user-defined SQL view.

Some schemas you'll encounter frequently:

- **`core`** — built-in tables like `Users`, `Groups`, and `Containers`
- **`lists`** — user-created lists (simple key-value tables)
- **`study`** — study datasets, if the Study module is enabled
- **`assay`** — assay result tables, nested under the assay design name
- **`exp`** — experiment framework tables (runs, data objects, materials)

When you call `select_rows`, you specify a schema name and a query name:

```rust
use labkey_rs::query::SelectRowsOptions;

let options = SelectRowsOptions::builder()
    .schema_name("lists")       // the schema
    .query_name("Participants") // the table or query within that schema
    .build();
```

You can discover what schemas and queries are available using `get_schemas` and `get_queries`. The [Querying Data](./querying-data.md) guide covers these introspection methods.

For more on schemas and queries, see LabKey's [Queries](https://www.labkey.org/Documentation/wiki-page.view?name=simpleQuery) documentation.

## URL structure

LabKey's REST API follows a consistent URL pattern:

```text
{base_url}/{container_path}/{controller}-{action}.api
```

For example, a `select_rows` call to the `query` controller in the `/MyProject` container produces:

```text
https://labkey.example.com/labkey/MyProject/query-selectRows.api
```

You don't need to construct these URLs yourself — the client handles it. But understanding the pattern helps when reading server logs or debugging requests. Each module in this crate corresponds to one or more LabKey controllers: the `query` module wraps the `query` controller, the `security` module wraps several controllers (`security`, `core`, `project`, `login`, etc.), and so on. The [controller-to-module map](../introduction.md#controller-to-module-map) in the introduction lists all of them.

For more on the URL structure and available endpoints, see LabKey's [HTTP Interface](https://www.labkey.org/Documentation/wiki-page.view?name=remoteAPIs) documentation.

## Response format

This crate always requests API version 17.1, which is the response format used by all modern LabKey Server releases. In this format, each row is an object where column values are wrapped in a `CellValue` struct containing the raw value plus optional display metadata:

```json
{
  "rows": [
    {
      "data": {
        "Name": { "value": "Alice" },
        "Age": { "value": 30, "formattedValue": "30" },
        "Department": { "value": 3, "displayValue": "Engineering" }
      }
    }
  ]
}
```

The `value` field is always present. The `displayValue` field appears for lookup columns (where the stored value is a foreign key but you want the human-readable label). The `formattedValue` field appears when the column has a display format configured on the server.

## What's next

With these concepts in hand, the rest of the book should make sense. Start with [Getting Started](./getting-started.md) if you haven't already, or jump to [Querying Data](./querying-data.md) to learn the full range of read operations.
