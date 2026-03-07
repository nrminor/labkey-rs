# Managing Lists

LabKey lists are simple tabular data stores with a primary key — the most common way to store custom data in LabKey. This recipe covers creating lists programmatically using the `list` module.

## Creating a list

The `create_list` method is a convenience wrapper around the domain API. You specify a name, a key field name, a key type, and optionally the fields:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::list::{CreateListOptions, ListKeyType};
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let response = client
    .create_list(
        CreateListOptions::builder()
            .name("Participants".to_string())
            .key_name("ParticipantId".to_string())
            .key_type(ListKeyType::AutoIncrementInteger)
            .build(),
    )
    .await?;

println!("Created domain: {:?}", response.name);
# Ok(())
# }
```

### Key types

The `ListKeyType` enum controls how the primary key works:

- **`IntList`** — integer key, you supply the value on insert
- **`VarList`** — string (varchar) key, you supply the value on insert
- **`AutoIncrementInteger`** — integer key, the server assigns values automatically

`AutoIncrementInteger` is the most common choice for lists where you don't need to control the key values.

## Adding fields with the shorthand syntax

For simple lists, you can define fields directly on the options using the `fields` shorthand:

```rust,no_run
# use labkey_rs::list::{CreateListOptions, ListKeyType};
# use labkey_rs::domain::DomainField;
let options = CreateListOptions::builder()
    .name("Participants".to_string())
    .key_name("ParticipantId".to_string())
    .key_type(ListKeyType::AutoIncrementInteger)
    .fields(vec![
        DomainField {
            name: Some("Name".to_string()),
            range_uri: Some("http://www.w3.org/2001/XMLSchema#string".into()),
            ..Default::default()
        },
        DomainField {
            name: Some("Age".to_string()),
            range_uri: Some("http://www.w3.org/2001/XMLSchema#int".into()),
            ..Default::default()
        },
    ])
    .description("Study participants".to_string())
    .build();
```

The `range_uri` field specifies the column type using XML Schema URIs. Common types include `#string`, `#int`, `#double`, `#dateTime`, and `#boolean`.

## Using a full DomainDesign

For more control over the domain configuration, provide a `DomainDesign` directly instead of using the shorthand fields. The two approaches are mutually exclusive — you can't use both `domain_design` and the shorthand `fields`/`description`/`indices` on the same options:

```rust,no_run
# use labkey_rs::list::{CreateListOptions, ListKeyType};
# use labkey_rs::domain::{DomainDesign, DomainField};
let domain = DomainDesign {
    name: Some("Participants".to_string()),
    description: Some("Study participants with custom configuration".to_string()),
    fields: vec![
        DomainField {
            name: Some("Name".to_string()),
            range_uri: Some("http://www.w3.org/2001/XMLSchema#string".into()),
            nullable: Some(false),
            ..Default::default()
        },
    ],
    ..Default::default()
};

let options = CreateListOptions::builder()
    .name("Participants".to_string())
    .key_name("ParticipantId".to_string())
    .key_type(ListKeyType::AutoIncrementInteger)
    .domain_design(domain)
    .build();
```

## Working with list data

Once a list exists, you interact with its data using the standard query methods. Lists live in the `lists` schema:

```rust,no_run
# use labkey_rs::query::{SelectRowsOptions, InsertRowsOptions};
// Read from a list
let read_options = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .build();

// Insert into a list
let insert_options = InsertRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .rows(vec![
        serde_json::json!({"Name": "Alice", "Age": 30}),
    ])
    .build();
```

See [Querying Data](../guides/querying-data.md) and [Modifying Data](../guides/modifying-data.md) for the full details on reading and writing data.

## Further reading

The `domain` module provides lower-level methods for creating, updating, and deleting domains of any kind — not just lists. The `list` module is a convenience layer on top of it. See the [API reference](https://docs.rs/labkey-rs/latest/labkey_rs/domain/index.html) for the full domain API.
