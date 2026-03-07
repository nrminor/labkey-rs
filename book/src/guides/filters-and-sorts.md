# Filters and Sorts

Filters narrow which rows a query returns. Sorts control the order. Both are passed as options to `select_rows`, `select_distinct_rows`, and other query methods. This guide covers how to construct them.

## Building filters

A filter has three parts: a column name, an operator (`FilterType`), and a value (`FilterValue`). The `Filter::new` constructor takes all three:

```rust
use labkey_rs::filter::{Filter, FilterType, FilterValue};

// Age greater than 30
let f = Filter::new("Age", FilterType::GreaterThan, FilterValue::Single("30".into()));
```

For the common case of equality, there's a shorthand:

```rust
use labkey_rs::filter::Filter;

let f = Filter::equal("Status", "Active");
```

### Filter values

The `FilterValue` enum has three variants:

- **`FilterValue::None`** — for operators that don't take a value, like `IsBlank` or `HasAnyValue`
- **`FilterValue::Single(String)`** — a single value, used by most operators
- **`FilterValue::Multi(Vec<String>)`** — multiple values, used by `In`, `NotIn`, `Between`, `ContainsOneOf`, and similar operators

All values are strings regardless of the column's data type. The server handles type coercion.

```rust
use labkey_rs::filter::{Filter, FilterType, FilterValue};

// Null check — no value needed
let blank = Filter::new("Notes", FilterType::IsBlank, FilterValue::None);

// IN filter — multiple values
let departments = Filter::new(
    "Department",
    FilterType::In,
    FilterValue::Multi(vec![
        "Engineering".into(),
        "Research".into(),
        "Operations".into(),
    ]),
);

// BETWEEN filter — exactly two values
let age_range = Filter::new(
    "Age",
    FilterType::Between,
    FilterValue::Multi(vec!["18".into(), "65".into()]),
);
```

### Available operators

The `FilterType` enum has operators for comparison (`Equal`, `GreaterThan`, `LessThan`, etc.), string matching (`Contains`, `StartsWith`, `ContainsOneOf`), set membership (`In`, `NotIn`), ranges (`Between`, `NotBetween`), null checks (`IsBlank`, `IsNotBlank`), missing-value indicators (`HasMissingValue`, `DoesNotHaveMissingValue`), date-specific variants (`DateEqual`, `DateGreaterThan`, etc.), and array operators (`ArrayContainsAll`, `ArrayContainsAny`, etc.).

The full list is in the [API reference](https://docs.rs/labkey-rs/latest/labkey_rs/filter/enum.FilterType.html). The date-specific variants have the same semantics as their non-date counterparts but use different URL suffixes that tell the server to apply date-aware comparison logic.

### Passing filters to queries

Filters are passed via the `filter_array` field on query options:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::SelectRowsOptions;
# use labkey_rs::filter::{Filter, FilterType, FilterValue};
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let options = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .filter_array(vec![
        Filter::equal("Status", "Active"),
        Filter::new("Age", FilterType::GreaterThanOrEqual, FilterValue::Single("18".into())),
    ])
    .build();

let response = client.select_rows(options).await?;
# Ok(())
# }
```

Multiple filters are combined with AND logic — all filters must match for a row to be included.

## Building sorts

Sorts are represented by the `QuerySort` type. The easiest way to create one is by parsing a comma-separated string, which is the same format LabKey uses on the wire:

```rust
use labkey_rs::sort::QuerySort;

// Sort by Name ascending, then Created descending
let sort = QuerySort::parse("Name,-Created");
```

A `-` prefix means descending; no prefix means ascending. You can also build sorts programmatically:

```rust
use labkey_rs::sort::{ColumnSort, QuerySort};

let sort = QuerySort::from(vec![
    ColumnSort::ascending("Name"),
    ColumnSort::descending("Created"),
]);

// Display produces the wire format
assert_eq!(sort.to_string(), "Name,-Created");
```

### Passing sorts to queries

Sorts are passed via the `sort` field:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::SelectRowsOptions;
# use labkey_rs::sort::QuerySort;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let options = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .sort(QuerySort::parse("-Created"))
    .build();

let response = client.select_rows(options).await?;
# Ok(())
# }
```

## Combining filters and sorts

Filters and sorts compose naturally — just set both fields:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::SelectRowsOptions;
# use labkey_rs::filter::{Filter, FilterType, FilterValue};
# use labkey_rs::sort::QuerySort;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let options = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .filter_array(vec![
        Filter::equal("Status", "Active"),
        Filter::new("Department", FilterType::In, FilterValue::Multi(vec![
            "Engineering".into(),
            "Research".into(),
        ])),
    ])
    .sort(QuerySort::parse("Name"))
    .max_rows(50)
    .build();

let response = client.select_rows(options).await?;
# Ok(())
# }
```

The `select_distinct_rows` method also accepts `filter_array` and `sort`, so you can filter and sort distinct value lists the same way.

## Dynamic filter construction

If you're building filters from user input or configuration, `FilterType` provides `from_name` and `from_url_suffix` methods for looking up operators by their programmatic name or URL suffix:

```rust
use labkey_rs::filter::FilterType;

// Look up by programmatic name (case-insensitive)
let op = FilterType::from_name("eq").unwrap();
assert_eq!(op, FilterType::Equal);

// Look up by URL suffix
let op = FilterType::from_url_suffix("gte").unwrap();
assert_eq!(op, FilterType::GreaterThanOrEqual);
```

Each `FilterType` also has a `programmatic_name()` method that returns the canonical name, and a `display_text()` method that returns a human-readable description like "Is Equal To" or "Is Greater Than Or Equal To".
