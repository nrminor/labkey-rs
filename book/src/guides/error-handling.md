# Error Handling

All client methods return `Result<T, LabkeyError>`. This guide covers the error variants, how to match on them, and some LabKey-specific quirks to be aware of.

## The LabkeyError enum

`LabkeyError` has six variants, each representing a different failure mode:

**`Http(reqwest::Error)`** — a network-level failure. Connection refused, DNS resolution failed, TLS handshake error, request timeout. The inner `reqwest::Error` has methods like `is_timeout()` and `is_connect()` for further classification.

**`InvalidInput(String)`** — the client rejected the request before sending it. This happens when required fields are missing, values are blank, or the input is otherwise invalid. For example, calling `create_group` with an empty group name, or `Credential::from_netrc` when no `.netrc` file exists.

**`Api { status, body }`** — the server returned a non-success HTTP status with a structured error body. The `status` is a `reqwest::StatusCode` and the `body` is an `ApiErrorBody` containing the server's error message, Java exception class, and optional per-field errors.

**`UnexpectedResponse { status, text }`** — the server returned a non-success status but the body couldn't be parsed as a LabKey error. This can happen with proxy errors, load balancer pages, or server misconfiguration. The raw response text is preserved for diagnostics.

**`Deserialization(serde_json::Error)`** — the HTTP request succeeded (2xx status) but the response body couldn't be deserialized into the expected Rust type. This usually indicates a server version mismatch or an unexpected response format.

**`Url(url::ParseError)`** — the base URL passed to `LabkeyClient::new` couldn't be parsed. This only happens during client construction.

## Matching on errors

For simple scripts, the `?` operator and `Box<dyn Error>` are fine. When you need more control, match on the variants:

```rust,no_run
use labkey_rs::{LabkeyClient, LabkeyError};
use labkey_rs::query::SelectRowsOptions;

async fn query(client: &LabkeyClient) -> Result<(), LabkeyError> {
    let options = SelectRowsOptions::builder()
        .schema_name("lists")
        .query_name("Participants")
        .build();

    match client.select_rows(options).await {
        Ok(response) => {
            println!("{} rows", response.row_count);
        }
        Err(LabkeyError::Api { status, body }) => {
            eprintln!("Server error (HTTP {status}): {body}");
            for field_err in &body.errors {
                if let Some(msg) = &field_err.msg {
                    eprintln!("  field {:?}: {msg}", field_err.id);
                }
            }
        }
        Err(LabkeyError::Http(e)) if e.is_timeout() => {
            eprintln!("Request timed out — try increasing the timeout or narrowing the query");
        }
        Err(e) => {
            eprintln!("Unexpected error: {e}");
        }
    }

    Ok(())
}
```

## Structured API errors

The `ApiErrorBody` struct gives you access to the server's error details:

- **`exception`** (`Option<String>`) — the human-readable error message, like `"Query 'nonexistent' in schema 'core' doesn't exist."`
- **`exception_class`** (`Option<String>`) — the Java exception class, like `"org.labkey.api.query.QueryParseException"`
- **`errors`** (`Vec<FieldError>`) — per-field errors, each with an optional `id` (field name) and `msg` (error message)

The `Display` implementation on `ApiErrorBody` formats the message and class together, so printing the body directly gives you a useful one-liner.

## API version errors

LabKey servers can reject requests when the API version doesn't match. The `is_api_version_error()` method on `LabkeyError` checks for this specific case:

```rust,no_run
# use labkey_rs::LabkeyError;
fn handle_error(err: &LabkeyError) {
    if err.is_api_version_error() {
        eprintln!("API version mismatch — check that your server supports API version 17.1");
    }
}
```

This checks whether the error is an `Api` variant whose `exception_class` is `org.labkey.api.action.ApiVersionException`.

## The 200-with-embedded-exception pattern

One LabKey quirk worth knowing about: the server sometimes returns HTTP 200 (success) but embeds an exception in the JSON response body. The client detects this pattern automatically and converts it into a `LabkeyError::Api` error, so you don't need to check for it yourself. But if you're debugging unexpected behavior and see a 200 status in your server logs alongside an error from the client, this is why.

## Retry strategies

`LabkeyError` doesn't implement retry logic, but the variants give you enough information to decide when retrying makes sense:

- **`Http` errors** with `is_timeout()` or `is_connect()` are often transient and worth retrying
- **`Api` errors** with 5xx status codes may be transient server issues
- **`Api` errors** with 4xx status codes are usually permanent (bad input, missing permissions) and retrying won't help
- **`InvalidInput`** errors are always permanent — the request was rejected before it was sent
- **`UnexpectedResponse`** errors may be transient (proxy hiccups) or permanent (server misconfiguration)

## The error_handling example

The [`error_handling` example](https://github.com/nrminor/labkey-rs/tree/main/examples/error_handling.rs) in the repository demonstrates these patterns in a runnable program, including matching on specific variants and extracting structured error details.
