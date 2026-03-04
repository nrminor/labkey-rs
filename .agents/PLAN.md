# Implementation Plan: Core Abstractions

This document covers the foundational pieces that must be built by hand before the remaining ~60 endpoint functions can be implemented mechanically (likely via Ralph loops). Each section includes a code sketch for review, and the final section describes how the work maps to commits.

## 1. Error types (`src/error.rs`)

The error type is the first thing every module depends on. LabKey's server returns JSON error bodies with a specific shape when something goes wrong, so we need a variant for that in addition to the usual reqwest/serde plumbing.

A LabKey API error response looks like this (observed from the JS client's `getCallbackWrapper` handling):

```json
{
  "exception": "Query 'nonexistent' in schema 'core' doesn't exist.",
  "exceptionClass": "org.labkey.api.query.QueryParseException",
  "errors": [{ "id": "some_field", "msg": "Detailed error message" }]
}
```

```rust
use thiserror::Error;

/// Individual field-level error returned by the LabKey server.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct FieldError {
    /// The field this error relates to, if any.
    #[serde(default)]
    pub id: Option<String>,
    /// The error message.
    #[serde(default)]
    pub msg: Option<String>,
}

/// Structured error body returned by LabKey API endpoints.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiErrorBody {
    /// Human-readable error message.
    #[serde(default)]
    pub exception: Option<String>,
    /// Java exception class name from the server.
    #[serde(rename = "exceptionClass", default)]
    pub exception_class: Option<String>,
    /// Per-field errors, if any.
    #[serde(default)]
    pub errors: Vec<FieldError>,
}

#[derive(Debug, Error)]
pub enum LabkeyError {
    /// HTTP-level error from reqwest (connection failures, timeouts, etc.).
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// The server returned a non-success status code with a structured error body.
    #[error("LabKey API error (HTTP {status}): {body}")]
    Api {
        status: reqwest::StatusCode,
        body: ApiErrorBody,
    },

    /// The server returned a non-success status code but the body wasn't
    /// parseable as a LabKey error.
    #[error("HTTP {status}: {text}")]
    UnexpectedResponse {
        status: reqwest::StatusCode,
        text: String,
    },

    /// JSON deserialization failed on an otherwise successful response.
    #[error("Failed to deserialize response: {0}")]
    Deserialization(#[from] serde_json::Error),

    /// URL construction failed.
    #[error("Invalid URL: {0}")]
    Url(#[from] url::ParseError),
}

// ApiErrorBody needs Display for the #[error] attribute on the Api variant.
impl std::fmt::Display for ApiErrorBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.exception {
            Some(msg) => write!(f, "{msg}"),
            None => write!(f, "(no message)"),
        }
    }
}
```

The key design decision here is the `Api` variant: rather than using reqwest's `error_for_status()` (which discards the response body), we'll read the body first, try to parse it as `ApiErrorBody`, and fall back to `UnexpectedResponse` if it's not valid JSON. This gives callers access to the structured error information the server provides.

## 2. Client and URL construction (`src/client.rs`)

This is where the JS client's `ActionURL.ts`, `Ajax.ts`, and `constants.ts` collapse into a single Rust struct. The main design decisions are:

- Authentication: support basic auth and API keys (the two most common non-browser auth methods). Session-based auth can come later.
- Container path: a default is set at construction time, but every request can override it.
- The internal `request` helper handles the common pattern of "send request, check status, try to parse error body on failure, deserialize success body."

LabKey URLs follow the pattern `{base_url}/{container_path}/{controller}-{action}`, where the action usually ends in `.api` (e.g., `query-getQuery.api`). The JS client's `buildURL` appends `.view` if no extension is present, but for the Rust client we only care about `.api` endpoints.

```rust
use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
use url::Url;

/// Authentication credentials for a LabKey server.
#[derive(Debug, Clone)]
pub enum Credential {
    /// HTTP Basic authentication.
    Basic { email: String, password: String },
    /// LabKey API key (sent as basic auth with the key as the password
    /// and "apikey" as the username, per LabKey convention).
    ApiKey(String),
}

/// Configuration for constructing a [`LabkeyClient`].
pub struct ClientConfig {
    pub base_url: String,
    pub credential: Credential,
    /// Default container path (e.g., "/MyProject/MyFolder").
    /// Individual requests can override this.
    pub container_path: String,
}

/// Async client for the LabKey Server REST API.
pub struct LabkeyClient {
    http: reqwest::Client,
    base_url: Url,
    container_path: String,
    credential: Credential,
}

impl LabkeyClient {
    /// Create a new client from the given configuration.
    pub fn new(config: ClientConfig) -> Result<Self, crate::error::LabkeyError> {
        let base_url = Url::parse(&config.base_url)?;
        let http = reqwest::Client::new();
        Ok(Self {
            http,
            base_url,
            container_path: config.container_path,
            credential: config.credential,
        })
    }

    /// Build a LabKey action URL.
    ///
    /// LabKey URLs follow the pattern:
    /// `{base_url}/{container_path}/{controller}-{action}`
    ///
    /// The `action` should include the extension (e.g., `"getQuery.api"`).
    /// If `container_override` is `None`, the client's default container path is used.
    pub(crate) fn build_url(
        &self,
        controller: &str,
        action: &str,
        container_override: Option<&str>,
    ) -> Result<Url, crate::error::LabkeyError> {
        let container = container_override.unwrap_or(&self.container_path);

        // Normalize: ensure container starts and ends with '/'
        let container = format!(
            "/{}/",
            container.trim_matches('/')
        );

        let path = format!(
            "{}{}{}-{}",
            self.base_url.path().trim_end_matches('/'),
            container,
            controller,
            action,
        );

        let mut url = self.base_url.clone();
        url.set_path(&path);
        Ok(url)
    }

    /// Apply authentication headers to a request builder.
    fn authenticate(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        match &self.credential {
            Credential::Basic { email, password } => {
                builder.basic_auth(email, Some(password))
            }
            Credential::ApiKey(key) => {
                builder.basic_auth("apikey", Some(key))
            }
        }
    }

    /// Internal helper: send a GET request and deserialize the response.
    pub(crate) async fn get<T: serde::de::DeserializeOwned>(
        &self,
        url: Url,
        params: &[(String, String)],
    ) -> Result<T, crate::error::LabkeyError> {
        let builder = self.http.get(url).query(params);
        let builder = self.authenticate(builder);
        let response = builder.send().await?;
        self.handle_response(response).await
    }

    /// Internal helper: send a POST request with a JSON body and deserialize the response.
    pub(crate) async fn post<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        url: Url,
        body: &B,
    ) -> Result<T, crate::error::LabkeyError> {
        let builder = self.http.post(url).json(body);
        let builder = self.authenticate(builder);
        let response = builder.send().await?;
        self.handle_response(response).await
    }

    /// Check the response status and either deserialize the success body
    /// or construct an appropriate error.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, crate::error::LabkeyError> {
        let status = response.status();
        if status.is_success() {
            let body = response.json::<T>().await?;
            Ok(body)
        } else {
            let text = response.text().await.unwrap_or_default();
            match serde_json::from_str::<crate::error::ApiErrorBody>(&text) {
                Ok(api_error) => Err(crate::error::LabkeyError::Api {
                    status,
                    body: api_error,
                }),
                Err(_) => Err(crate::error::LabkeyError::UnexpectedResponse {
                    status,
                    text,
                }),
            }
        }
    }
}
```

A few things worth noting about this sketch:

The `build_url` method is intentionally simpler than the JS `buildURL`. The JS version has to deal with extracting the context path and container from the browser's current URL, which we don't need. We take both as explicit configuration.

The `get` and `post` helpers are `pub(crate)` — they're the internal workhorses that every endpoint method will call, but they're not part of the public API. The public API is the endpoint methods themselves (e.g., `select_rows`, `execute_sql`).

The `handle_response` method is where we diverge from the JS client's `getCallbackWrapper`. Instead of callbacks, we return `Result`. Instead of `error_for_status()` (which throws away the body), we read the body and try to parse it as a LabKey error.

## 3. Filter system (`src/filter.rs`)

The filter system is the most intricate piece of pure logic in the port. The JS client defines ~40 filter operators, each with a URL suffix, display text, optional multi-value separator, and various metadata. In Rust, this maps to an enum with associated data.

The key insight from the JS source is that filters are encoded as URL query parameters in the form `{dataRegionName}.{columnName}~{urlSuffix}={value}`. Multi-valued filters (like `IN`) join their values with a separator (`;` or `,`), and if any value contains the separator character, the whole thing gets wrapped in `{json:[...]}` syntax.

```rust
use std::fmt;

/// The operator for a LabKey query filter.
///
/// Each variant corresponds to a URL suffix that the server recognizes.
/// For example, `FilterType::Equal` has suffix `"eq"`, so a filter on
/// column "Age" with value "25" would be encoded as `query.Age~eq=25`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FilterType {
    Equal,
    NotEqual,
    NotEqualOrNull,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    In,
    NotIn,
    Contains,
    DoesNotContain,
    StartsWith,
    DoesNotStartWith,
    ContainsOneOf,
    ContainsNoneOf,
    Between,
    NotBetween,
    IsBlank,
    IsNotBlank,
    HasAnyValue,
    MemberOf,
    HasMissingValue,
    DoesNotHaveMissingValue,

    // Date-specific variants (same operators but different URL suffixes)
    DateEqual,
    DateNotEqual,
    DateGreaterThan,
    DateGreaterThanOrEqual,
    DateLessThan,
    DateLessThanOrEqual,

    // Array operators
    ArrayContainsAll,
    ArrayContainsAny,
    ArrayContainsExact,
    ArrayContainsNotExact,
    ArrayContainsNone,
    ArrayIsEmpty,
    ArrayIsNotEmpty,

    // Table-wise search
    Q,

    // Ontology operators
    OntologyInSubtree,
    OntologyNotInSubtree,

    // Lineage operators
    ExpChildOf,
    ExpParentOf,
    ExpLineageOf,
}

impl FilterType {
    /// The URL suffix used when encoding this filter as a query parameter.
    #[must_use]
    pub fn url_suffix(self) -> &'static str {
        match self {
            Self::Equal => "eq",
            Self::NotEqual => "neq",
            Self::NotEqualOrNull => "neqornull",
            Self::GreaterThan => "gt",
            Self::GreaterThanOrEqual => "gte",
            Self::LessThan => "lt",
            Self::LessThanOrEqual => "lte",
            Self::In => "in",
            Self::NotIn => "notin",
            Self::Contains => "contains",
            Self::DoesNotContain => "doesnotcontain",
            Self::StartsWith => "startswith",
            Self::DoesNotStartWith => "doesnotstartwith",
            Self::ContainsOneOf => "containsoneof",
            Self::ContainsNoneOf => "containsnoneof",
            Self::Between => "between",
            Self::NotBetween => "notbetween",
            Self::IsBlank => "isblank",
            Self::IsNotBlank => "isnonblank",
            Self::HasAnyValue => "",
            Self::MemberOf => "memberof",
            Self::HasMissingValue => "hasmvvalue",
            Self::DoesNotHaveMissingValue => "nomvvalue",
            Self::DateEqual => "dateeq",
            Self::DateNotEqual => "dateneq",
            Self::DateGreaterThan => "dategt",
            Self::DateGreaterThanOrEqual => "dategte",
            Self::DateLessThan => "datelt",
            Self::DateLessThanOrEqual => "datelte",
            Self::ArrayContainsAll => "arraycontainsall",
            Self::ArrayContainsAny => "arraycontainsany",
            Self::ArrayContainsExact => "arraymatches",
            Self::ArrayContainsNotExact => "arraynotmatches",
            Self::ArrayContainsNone => "arraycontainsnone",
            Self::ArrayIsEmpty => "arrayisempty",
            Self::ArrayIsNotEmpty => "arrayisnotempty",
            Self::Q => "q",
            Self::OntologyInSubtree => "concept:insubtree",
            Self::OntologyNotInSubtree => "concept:notinsubtree",
            Self::ExpChildOf => "exp:childof",
            Self::ExpParentOf => "exp:parentof",
            Self::ExpLineageOf => "exp:lineageof",
        }
    }

    /// Whether this filter type requires a data value.
    /// Filters like `IsBlank` and `HasAnyValue` do not.
    #[must_use]
    pub fn requires_value(self) -> bool {
        !matches!(
            self,
            Self::IsBlank
                | Self::IsNotBlank
                | Self::HasAnyValue
                | Self::ArrayIsEmpty
                | Self::ArrayIsNotEmpty
        )
    }

    /// Whether this filter type accepts multiple values.
    #[must_use]
    pub fn is_multi_valued(self) -> bool {
        self.separator().is_some()
    }

    /// The separator character for multi-valued filters, if applicable.
    #[must_use]
    pub fn separator(self) -> Option<char> {
        match self {
            Self::In | Self::NotIn | Self::ContainsOneOf | Self::ContainsNoneOf
            | Self::ArrayContainsAll | Self::ArrayContainsAny | Self::ArrayContainsExact
            | Self::ArrayContainsNotExact | Self::ArrayContainsNone => Some(';'),
            Self::Between | Self::NotBetween | Self::ExpLineageOf => Some(','),
            _ => None,
        }
    }

    /// Look up a `FilterType` by its URL suffix.
    #[must_use]
    pub fn from_url_suffix(suffix: &str) -> Option<Self> {
        // This could be a static map, but there are only ~40 variants
        // and this is not a hot path.
        Self::ALL.iter().copied().find(|ft| ft.url_suffix() == suffix)
    }

    /// All filter type variants.
    const ALL: &[Self] = &[
        Self::Equal, Self::NotEqual, Self::NotEqualOrNull,
        Self::GreaterThan, Self::GreaterThanOrEqual,
        Self::LessThan, Self::LessThanOrEqual,
        Self::In, Self::NotIn,
        Self::Contains, Self::DoesNotContain,
        Self::StartsWith, Self::DoesNotStartWith,
        Self::ContainsOneOf, Self::ContainsNoneOf,
        Self::Between, Self::NotBetween,
        Self::IsBlank, Self::IsNotBlank, Self::HasAnyValue,
        Self::MemberOf,
        Self::HasMissingValue, Self::DoesNotHaveMissingValue,
        Self::DateEqual, Self::DateNotEqual,
        Self::DateGreaterThan, Self::DateGreaterThanOrEqual,
        Self::DateLessThan, Self::DateLessThanOrEqual,
        Self::ArrayContainsAll, Self::ArrayContainsAny,
        Self::ArrayContainsExact, Self::ArrayContainsNotExact,
        Self::ArrayContainsNone, Self::ArrayIsEmpty, Self::ArrayIsNotEmpty,
        Self::Q,
        Self::OntologyInSubtree, Self::OntologyNotInSubtree,
        Self::ExpChildOf, Self::ExpParentOf, Self::ExpLineageOf,
    ];
}

/// A filter to apply to a LabKey query.
///
/// Filters are encoded as URL query parameters in the form
/// `{dataRegionName}.{columnName}~{urlSuffix}={value}`.
#[derive(Debug, Clone)]
pub struct Filter {
    column_name: String,
    filter_type: FilterType,
    value: FilterValue,
}

/// The value(s) for a filter.
#[derive(Debug, Clone)]
pub enum FilterValue {
    /// No value (for filters like `IsBlank`).
    None,
    /// A single value.
    Single(String),
    /// Multiple values (for `In`, `Between`, etc.).
    Multi(Vec<String>),
}

impl Filter {
    /// Create a new filter.
    #[must_use]
    pub fn new(column_name: impl Into<String>, filter_type: FilterType, value: FilterValue) -> Self {
        Self {
            column_name: column_name.into(),
            filter_type,
            value,
        }
    }

    /// Convenience: create an equality filter.
    #[must_use]
    pub fn equal(column_name: impl Into<String>, value: impl Into<String>) -> Self {
        Self::new(column_name, FilterType::Equal, FilterValue::Single(value.into()))
    }

    /// The URL parameter name for this filter (e.g., `"query.Age~eq"`).
    #[must_use]
    pub fn url_param_name(&self, data_region_name: &str) -> String {
        format!(
            "{}.{}~{}",
            data_region_name,
            self.column_name,
            self.filter_type.url_suffix()
        )
    }

    /// The URL parameter value for this filter.
    ///
    /// For multi-valued filters, values are joined with the appropriate separator.
    /// If any value contains the separator, the `{json:[...]}` encoding is used.
    #[must_use]
    pub fn url_param_value(&self) -> String {
        match (&self.value, self.filter_type.separator()) {
            (FilterValue::None, _) => String::new(),
            (FilterValue::Single(v), _) => v.clone(),
            (FilterValue::Multi(values), Some(sep)) => {
                let sep_str = String::from(sep);
                let needs_json = values.iter().any(|v| v.contains(sep));
                if needs_json {
                    // {json:["val1","val2"]}
                    let json_array = serde_json::to_string(values)
                        .unwrap_or_default();
                    format!("{{json:{json_array}}}")
                } else {
                    values.join(&sep_str)
                }
            }
            (FilterValue::Multi(values), None) => {
                // Shouldn't happen for well-formed filters, but handle gracefully
                values.join(";")
            }
        }
    }
}

/// Encode an array of filters into query parameter key-value pairs.
pub fn encode_filters(filters: &[Filter], data_region_name: &str) -> Vec<(String, String)> {
    filters
        .iter()
        .filter(|f| {
            // Skip no-op filters (value-required filter with no value)
            if f.filter_type.requires_value() {
                !matches!(f.value, FilterValue::None)
            } else {
                true
            }
        })
        .map(|f| (f.url_param_name(data_region_name), f.url_param_value()))
        .collect()
}

/// Container filter scope for queries.
///
/// Controls which containers' data is included in query results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ContainerFilter {
    #[serde(rename = "AllFolders")]
    AllFolders,
    #[serde(rename = "AllInProject")]
    AllInProject,
    #[serde(rename = "AllInProjectPlusShared")]
    AllInProjectPlusShared,
    #[serde(rename = "Current")]
    Current,
    #[serde(rename = "CurrentAndFirstChildren")]
    CurrentAndFirstChildren,
    #[serde(rename = "CurrentAndParents")]
    CurrentAndParents,
    #[serde(rename = "CurrentAndSubfolders")]
    CurrentAndSubfolders,
    #[serde(rename = "CurrentAndSubfoldersPlusShared")]
    CurrentAndSubfoldersPlusShared,
    #[serde(rename = "CurrentPlusProject")]
    CurrentPlusProject,
    #[serde(rename = "CurrentPlusProjectAndShared")]
    CurrentPlusProjectAndShared,
}
```

## 4. Query endpoints: `select_rows` and `execute_sql` (`src/query.rs`)

These are the proof-of-concept endpoints that force the response type system to be built out. The JS client's `SelectRows.ts` and `ExecuteSql.ts` are the reference.

The response types are the most important part here. The JS client supports multiple response format versions (`requiredVersion` 8.3, 9.1, 13.2, 16.2, 17.1), but we'll only support 17.1 (the latest). In 17.1, each row is an object with a `data` map where each column value is an object with `value`, optionally `displayValue`, `formattedValue`, `url`, `mvValue`, `mvIndicator`, etc.

The `wafEncode` function from `Utils.ts` is needed for `execute_sql` — it BASE64-encodes the SQL string with a magic prefix to avoid WAF false positives.

```rust
use serde::{Deserialize, Serialize};
use crate::client::LabkeyClient;
use crate::error::LabkeyError;
use crate::filter::{ContainerFilter, Filter, encode_filters};

// -- Response types --

/// A cell value in a query response row (requiredVersion 17.1).
///
/// In the 17.1 response format, each column value is an object with
/// at minimum a `value` field, plus optional display/formatting metadata.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellValue {
    pub value: serde_json::Value,
    #[serde(default)]
    pub display_value: Option<String>,
    #[serde(default)]
    pub formatted_value: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub mv_value: Option<String>,
    #[serde(default)]
    pub mv_indicator: Option<String>,
}

/// A single row in a query response.
#[derive(Debug, Clone, Deserialize)]
pub struct Row {
    pub data: std::collections::HashMap<String, CellValue>,
    #[serde(default)]
    pub links: Option<serde_json::Value>,
}

/// Column metadata returned in query responses.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryColumn {
    pub name: String,
    pub field_key: serde_json::Value, // Can be string or array
    pub caption: String,
    pub short_caption: String,
    pub json_type: Option<String>,
    pub sql_type: Option<String>,
    #[serde(default)]
    pub hidden: bool,
    #[serde(default)]
    pub nullable: bool,
    #[serde(default)]
    pub read_only: bool,
    #[serde(default)]
    pub user_editable: bool,
    #[serde(default)]
    pub auto_increment: bool,
    #[serde(default)]
    pub key_field: bool,
    #[serde(default)]
    pub mv_enabled: bool,
    // ... many more optional fields exist; we can add them as needed
}

/// Metadata block in a query response.
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseMetadata {
    pub fields: Vec<QueryColumn>,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub root: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

/// Response from `select_rows` or `execute_sql`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectRowsResponse {
    pub schema_name: serde_json::Value, // Can be string or array
    pub query_name: Option<String>,
    pub format_version: Option<f64>,
    pub row_count: i64,
    pub rows: Vec<Row>,
    #[serde(default)]
    pub meta_data: Option<ResponseMetadata>,
}

// -- Request options --

/// Options for `select_rows`.
#[derive(Debug, Default)]
pub struct SelectRowsOptions {
    pub schema_name: String,
    pub query_name: String,
    pub container_path: Option<String>,
    pub columns: Option<Vec<String>>,
    pub filter_array: Option<Vec<Filter>>,
    pub sort: Option<String>,
    pub view_name: Option<String>,
    pub max_rows: Option<i32>,
    pub offset: Option<i64>,
    pub container_filter: Option<ContainerFilter>,
    pub include_total_count: Option<bool>,
    pub include_metadata: Option<bool>,
    pub ignore_filter: Option<bool>,
    pub parameters: Option<std::collections::HashMap<String, String>>,
}

/// Options for `execute_sql`.
#[derive(Debug)]
pub struct ExecuteSqlOptions {
    pub schema_name: String,
    pub sql: String,
    pub container_path: Option<String>,
    pub max_rows: Option<i32>,
    pub offset: Option<i64>,
    pub sort: Option<String>,
    pub container_filter: Option<ContainerFilter>,
    pub include_total_count: Option<bool>,
    pub include_metadata: Option<bool>,
    pub save_in_session: Option<bool>,
    pub parameters: Option<std::collections::HashMap<String, String>>,
}

// -- WAF encoding --

/// Encode a SQL string to avoid WAF false positives.
///
/// LabKey endpoints that accept SQL use this encoding to prevent web
/// application firewalls from rejecting legitimate SQL content.
/// The encoding is: `/*{{base64/x-www-form-urlencoded/wafText}}*/` + base64(url_encode(sql)).
fn waf_encode(sql: &str) -> String {
    use base64::Engine;
    let encoded = urlencoding::encode(sql);
    let b64 = base64::engine::general_purpose::STANDARD.encode(encoded.as_bytes());
    format!("/*{{{{base64/x-www-form-urlencoded/wafText}}}}*/{b64}")
}

// -- Endpoint implementations --

impl LabkeyClient {
    /// Select rows from a LabKey query.
    pub async fn select_rows(
        &self,
        options: SelectRowsOptions,
    ) -> Result<SelectRowsResponse, LabkeyError> {
        let url = self.build_url(
            "query",
            "getQuery.api",
            options.container_path.as_deref(),
        )?;

        let data_region = "query";
        let mut params: Vec<(String, String)> = vec![
            ("schemaName".into(), options.schema_name.clone()),
            (format!("{data_region}.queryName"), options.query_name.clone()),
            ("apiVersion".into(), "17.1".into()),
        ];

        // Filters
        if let Some(filters) = &options.filter_array {
            params.extend(encode_filters(filters, data_region));
        }

        // Sort
        if let Some(sort) = &options.sort {
            params.push((format!("{data_region}.sort"), sort.clone()));
        }

        // Columns
        if let Some(cols) = &options.columns {
            params.push((format!("{data_region}.columns"), cols.join(",")));
        }

        // Pagination
        if let Some(max) = options.max_rows {
            if max < 0 {
                params.push((format!("{data_region}.showRows"), "all".into()));
            } else {
                params.push((format!("{data_region}.maxRows"), max.to_string()));
            }
        }
        if let Some(offset) = options.offset {
            params.push((format!("{data_region}.offset"), offset.to_string()));
        }

        // View
        if let Some(view) = &options.view_name {
            params.push((format!("{data_region}.viewName"), view.clone()));
        }

        // Container filter
        if let Some(cf) = &options.container_filter {
            let cf_str = serde_json::to_value(cf)
                .ok()
                .and_then(|v| v.as_str().map(String::from))
                .unwrap_or_default();
            params.push(("containerFilter".into(), cf_str));
        }

        // Optional booleans
        if let Some(v) = options.include_total_count {
            params.push(("includeTotalCount".into(), v.to_string()));
        }
        if let Some(v) = options.include_metadata {
            params.push(("includeMetadata".into(), v.to_string()));
        }
        if let Some(true) = options.ignore_filter {
            params.push((format!("{data_region}.ignoreFilter"), "1".into()));
        }

        // Parameterized query parameters
        if let Some(parameters) = &options.parameters {
            for (k, v) in parameters {
                params.push((format!("{data_region}.param.{k}"), v.clone()));
            }
        }

        self.get(url, &params).await
    }

    /// Execute arbitrary LabKey SQL.
    pub async fn execute_sql(
        &self,
        options: ExecuteSqlOptions,
    ) -> Result<SelectRowsResponse, LabkeyError> {
        // URL params (sort and parameterized query params go on the URL)
        let mut url_params: Vec<(String, String)> = Vec::new();
        if let Some(sort) = &options.sort {
            url_params.push(("query.sort".into(), sort.clone()));
        }
        if let Some(parameters) = &options.parameters {
            for (k, v) in parameters {
                url_params.push((format!("query.param.{k}"), v.clone()));
            }
        }

        let mut url = self.build_url(
            "query",
            "executeSql.api",
            options.container_path.as_deref(),
        )?;

        // Append URL params
        if !url_params.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (k, v) in &url_params {
                pairs.append_pair(k, v);
            }
        }

        // JSON body
        let mut body = serde_json::json!({
            "schemaName": options.schema_name,
            "sql": waf_encode(&options.sql),
            "apiVersion": 17.1,
        });

        if let Some(max) = options.max_rows {
            body["maxRows"] = serde_json::json!(max);
        }
        if let Some(offset) = options.offset {
            body["offset"] = serde_json::json!(offset);
        }
        if let Some(cf) = &options.container_filter {
            body["containerFilter"] = serde_json::to_value(cf).unwrap_or_default();
        }
        if let Some(v) = options.include_total_count {
            body["includeTotalCount"] = serde_json::json!(v);
        }
        if let Some(v) = options.include_metadata {
            body["includeMetadata"] = serde_json::json!(v);
        }
        if let Some(v) = options.save_in_session {
            body["saveInSession"] = serde_json::json!(v);
        }

        self.post(url, &body).await
    }
}
```

Note that `waf_encode` uses `urlencoding::encode` — we'd need to add the `urlencoding` crate, or we could use the `url` crate's `form_urlencoded` module which we already depend on. That's a detail to resolve during implementation.

## 5. Public API surface (`src/lib.rs`)

The lib.rs ties everything together with module declarations and re-exports. This is straightforward:

````rust
//! Unofficial Rust client for the `LabKey` Server REST API.
//!
//! # Example
//!
//! ```no_run
//! use labkey_rs::{LabkeyClient, ClientConfig, Credential};
//! use labkey_rs::query::SelectRowsOptions;
//!
//! # async fn example() -> Result<(), labkey_rs::LabkeyError> {
//! let client = LabkeyClient::new(ClientConfig {
//!     base_url: "https://labkey.example.com/labkey".into(),
//!     credential: Credential::BasicAuth {
//!         email: "user@example.com".into(),
//!         password: "secret".into(),
//!     },
//!     container_path: "/MyProject/MyFolder".into(),
//! })?;
//!
//! let response = client.select_rows(SelectRowsOptions {
//!     schema_name: "lists".into(),
//!     query_name: "People".into(),
//!     ..Default::default()
//! }).await?;
//!
//! println!("Got {} rows", response.row_count);
//! # Ok(())
//! # }
//! ```

pub mod client;
pub mod error;
pub mod filter;
pub mod query;

// Re-export the most commonly used types at the crate root.
pub use client::{ClientConfig, Credential, LabkeyClient};
pub use error::LabkeyError;
````

## Commit strategy

The work above should be organized into three commits. Each commit builds, passes all checks, and is a coherent unit of review.

**Commit 1: Error types, client struct, and URL construction**

This commit introduces `error.rs` and `client.rs`, plus the module declarations in `lib.rs`. After this commit, the crate compiles and has a `LabkeyClient` that can be constructed and can build URLs, but can't do anything useful yet. Tests in this commit cover URL construction (the most logic-dense part) and error type construction/display. This is the commit where we make and lock in the authentication design, the error handling strategy, and the URL building approach.

Files: `src/error.rs`, `src/client.rs`, `src/lib.rs`, `.gitignore` updates.

**Commit 2: Filter system**

This commit introduces `filter.rs` with the `FilterType` enum, `Filter` struct, `FilterValue`, `ContainerFilter`, and the `encode_filters` function. It's a self-contained module with no I/O, so it can be thoroughly unit-tested: URL suffix round-tripping, multi-value encoding, the `{json:...}` escape path, `from_url_suffix` lookups, and the no-op filter skipping in `encode_filters`. This is the commit where we validate that the filter encoding matches what the JS client produces.

Files: `src/filter.rs`, `src/lib.rs` update, `.gitignore` updates.

**Commit 3: `select_rows`, `execute_sql`, and response types**

This commit introduces `query.rs` with the response types (`CellValue`, `Row`, `QueryColumn`, `ResponseMetadata`, `SelectRowsResponse`), the request option structs (`SelectRowsOptions`, `ExecuteSqlOptions`), the `waf_encode` helper, and the two endpoint methods on `LabkeyClient`. Tests here cover `waf_encode` (we can compare against the JS test fixtures), request parameter construction, and response deserialization from fixture JSON. We can't easily test the actual HTTP round-trip without a LabKey server, but we can test everything up to and including the serialized request shape and the deserialized response shape.

Files: `src/query.rs`, `src/lib.rs` update, `.gitignore` updates. Possibly a new dev-dependency if we need `urlencoding` or decide to use `form_urlencoded` from the `url` crate.

After these three commits, the pattern is fully established and the remaining ~60 endpoint functions can be implemented one or a few at a time in Ralph loop iterations. Each Ralph story would be something like "implement `insert_rows`, `update_rows`, and `delete_rows`" — define the request/response structs, write the method, add tests for serialization.
