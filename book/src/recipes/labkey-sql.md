# LabKey SQL

When `select_rows` isn't expressive enough — you need joins, aggregations, subqueries, or computed columns — use `execute_sql` to send LabKey SQL directly. This recipe covers common patterns and the differences from standard SQL.

## Basic usage

`execute_sql` takes a schema name (the execution context) and a SQL string:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::query::ExecuteSqlOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let options = ExecuteSqlOptions::builder()
    .schema_name("lists")
    .sql("SELECT Name, Department FROM Participants WHERE Status = 'Active'")
    .build();

let response = client.execute_sql(options).await?;
for row in &response.rows {
    println!("{:?}", row.data);
}
# Ok(())
# }
```

The response is the same `SelectRowsResponse` type returned by `select_rows`, so you work with `rows`, `row_count`, and `meta_data` in exactly the same way.

## Aggregations

```rust,no_run
# use labkey_rs::query::ExecuteSqlOptions;
let options = ExecuteSqlOptions::builder()
    .schema_name("lists")
    .sql("SELECT Department, COUNT(*) AS Total, AVG(Age) AS AvgAge \
          FROM Participants \
          GROUP BY Department \
          HAVING COUNT(*) > 5 \
          ORDER BY Total DESC")
    .build();
```

LabKey SQL supports `COUNT`, `SUM`, `AVG`, `MIN`, `MAX`, `GROUP BY`, `HAVING`, and `ORDER BY`.

## Joins

You can join tables within the same schema or across schemas:

```rust,no_run
# use labkey_rs::query::ExecuteSqlOptions;
let options = ExecuteSqlOptions::builder()
    .schema_name("lists")
    .sql("SELECT p.Name, d.DepartmentName, d.Building \
          FROM Participants p \
          INNER JOIN Departments d ON p.DepartmentId = d.RowId \
          WHERE d.Building = 'Main Campus'")
    .build();
```

To reference a table in a different schema, use the fully qualified name (e.g., `core.Users`).

## Subqueries

```rust,no_run
# use labkey_rs::query::ExecuteSqlOptions;
let options = ExecuteSqlOptions::builder()
    .schema_name("lists")
    .sql("SELECT Name, Department FROM Participants \
          WHERE DepartmentId IN (SELECT RowId FROM Departments WHERE Active = true)")
    .build();
```

## Pagination with SQL

`execute_sql` supports `max_rows` and `offset` just like `select_rows`:

```rust,no_run
# use labkey_rs::query::ExecuteSqlOptions;
let options = ExecuteSqlOptions::builder()
    .schema_name("lists")
    .sql("SELECT * FROM Participants ORDER BY RowId")
    .max_rows(100)
    .offset(200)
    .build();
```

## Experimental compact SQL endpoint (large-table sync)

For very large result sets, LabKey also exposes an experimental SQL endpoint that returns a lean compact payload with less display-oriented metadata. In `labkey-rs`, this API is feature-gated and exposed through an extension trait.

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
use labkey_rs::query::experimental::{ExperimentalQueryExt, SqlExecuteOptions};

# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let response = client
    .experimental_sql_execute(
        SqlExecuteOptions::builder()
            .schema("lists")
            .sql("SELECT RowId, Name, Score FROM Participants ORDER BY RowId")
            .build(),
    )
    .await?;

let columnar = response.clone().into_columns();
let scores = response.column_f64("Score")?;

println!("{} rows", response.row_count());
println!("{} columns in column-major form", columnar.columns.len());
println!("first score = {:?}", scores.first());
# Ok(())
# }
```

This is particularly useful for local-first synchronization flows where throughput matters more than UI-style metadata. Since it is experimental, keep usage isolated behind your own abstraction so you can adapt quickly if the wire contract changes.

## Security considerations

`execute_sql` and `experimental_sql_execute` both run raw SQL text. That makes them powerful, but it also means you should treat them as higher-risk interfaces than fixed query calls.

- Do not build SQL with direct string interpolation from untrusted input.
- Prefer predefined SQL templates with constrained inputs.
- Use service credentials with the minimum LabKey permissions needed.
- Apply conservative limits (`max_rows`, paging, request throttling) in interactive systems.

The client's automatic WAF encoding is for compatibility with web-application-firewall deployments; it does not provide SQL injection protection for application code.

## Differences from standard SQL

LabKey SQL is close to standard SQL but has some differences worth knowing about:

- Table names are LabKey query names, not database table names. Use the names you see in the LabKey UI or from `get_queries`.
- String literals use single quotes. Double quotes are for identifiers (column names with spaces or reserved words).
- The `LIMIT` keyword is not supported — use `max_rows` on the options instead.
- LabKey adds some functions not in standard SQL, and some standard functions may not be available. See LabKey's [SQL Reference](https://www.labkey.org/Documentation/wiki-page.view?name=labkeySql) for the full syntax.

## WAF encoding

LabKey servers often sit behind a Web Application Firewall (WAF) that blocks SQL-like strings in request parameters. The client handles WAF encoding automatically — your SQL is encoded before sending and the server decodes it transparently. You don't need to do anything special.

## When to use SQL vs. select_rows

Use `select_rows` when you're reading from a single table with simple filters and sorts. It's more ergonomic, the filters are type-safe, and the server can optimize the query path.

Use `execute_sql` when you need joins, aggregations, computed columns, subqueries, or anything else that goes beyond what `select_rows` supports. The trade-off is that your SQL is a raw string — typos and schema mismatches become runtime errors rather than compile-time errors.
