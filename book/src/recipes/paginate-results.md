# Paginate Results

When a query returns more rows than you want to fetch at once, use `max_rows` and `offset` to page through the results. This recipe shows a complete pagination loop.

## The pattern

Set `max_rows` to your page size and `include_total_count(true)` so the response tells you the total number of matching rows. Then increment `offset` by the page size on each iteration:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::SelectRowsOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let page_size = 100;
let mut offset: i64 = 0;

loop {
    let options = SelectRowsOptions::builder()
        .schema_name("lists")
        .query_name("Participants")
        .max_rows(page_size)
        .offset(offset)
        .include_total_count(true)
        .build();

    let response = client.select_rows(options).await?;

    for row in &response.rows {
        // Process each row
        println!("{:?}", row.data.keys().collect::<Vec<_>>());
    }

    // row_count here is the total count (because include_total_count is true)
    let fetched = offset + response.rows.len() as i64;
    if fetched >= response.row_count || response.rows.is_empty() {
        break;
    }

    offset = fetched;
}
# Ok(())
# }
```

## Detecting the last page

There are two ways to know you've reached the end:

1. **Total count**: When `include_total_count` is set, `response.row_count` reflects the total number of matching rows across all pages. Compare your running offset against this total.

2. **Empty page**: If `response.rows` is empty, there are no more rows. This is a safe fallback even without `include_total_count`.

The loop above uses both checks for robustness.

## Sorting for stable pagination

If the underlying data can change between page requests, rows might shift between pages. Adding a sort on a unique column (like the primary key) ensures stable ordering:

```rust,no_run
# use labkey_rs::query::SelectRowsOptions;
# use labkey_rs::sort::QuerySort;
let options = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .max_rows(100)
    .offset(0)
    .sort(QuerySort::parse("RowId"))
    .include_total_count(true)
    .build();
```

## Fetching all rows at once

If you know the result set is small enough to fit in memory, pass a negative `max_rows` to skip pagination entirely:

```rust,no_run
# use labkey_rs::query::SelectRowsOptions;
let options = SelectRowsOptions::builder()
    .schema_name("lists")
    .query_name("Participants")
    .max_rows(-1)
    .build();
```

Be cautious with this on large tables — it loads everything into memory in a single HTTP response.
