//! Query endpoints and response types for the `LabKey` REST API.
//!
//! This module provides [`SelectRowsOptions`] and [`ExecuteSqlOptions`] for
//! the two primary query endpoints, along with the response types that model
//! the 17.1 response format. Both endpoints return a [`SelectRowsResponse`]
//! containing typed rows where each cell is a [`CellValue`] with the raw
//! value and optional display/formatting metadata.

use std::collections::HashMap;

use base64::Engine;
use serde::Deserialize;

use crate::{
    client::LabkeyClient,
    error::LabkeyError,
    filter::{ContainerFilter, Filter, encode_filters},
};

/// A cell value in a query response row.
///
/// In the 17.1 response format, each column value is an object with at
/// minimum a `value` field, plus optional display and formatting metadata.
/// Multi-value columns return an array of these objects, but we deserialize
/// those as `serde_json::Value` for now since they're uncommon.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellValue {
    /// The raw value (string, number, boolean, null, or array for multi-value).
    pub value: serde_json::Value,
    /// Display value for lookup columns.
    #[serde(default)]
    pub display_value: Option<String>,
    /// Value formatted according to the column's display format (17.1+).
    #[serde(default)]
    pub formatted_value: Option<String>,
    /// URL for lookup columns.
    #[serde(default)]
    pub url: Option<String>,
    /// Raw missing-value indicator value.
    #[serde(default)]
    pub mv_value: Option<String>,
    /// Missing-value indicator code (e.g., `"Q"` for quality control).
    #[serde(default)]
    pub mv_indicator: Option<String>,
}

/// A single row in a query response.
///
/// Each row contains a `data` map keyed by column name, where each value
/// is a [`CellValue`]. The optional `links` field contains server-generated
/// URLs for detail/update views.
#[derive(Debug, Clone, Deserialize)]
pub struct Row {
    /// Column values keyed by column name.
    pub data: HashMap<String, CellValue>,
    /// Server-generated links (detail, update, etc.), if present.
    #[serde(default)]
    pub links: Option<serde_json::Value>,
}

/// Column metadata returned in query responses.
///
/// Not all fields are always present — the server omits fields that have
/// default values. Fields are added here as needed; the full set in the JS
/// client's `MetadataField` interface has many more.
// The LabKey server defines these boolean fields on column metadata; we
// can't reduce them without losing fidelity to the response format.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryColumn {
    /// Internal column name.
    pub name: String,
    /// Field key (can be a string or an array for nested lookups).
    pub field_key: serde_json::Value,
    /// Human-readable column caption.
    #[serde(default)]
    pub caption: Option<String>,
    /// Short caption for compact displays.
    #[serde(default)]
    pub short_caption: Option<String>,
    /// JSON type name (e.g., `"int"`, `"string"`, `"float"`).
    #[serde(default)]
    pub json_type: Option<String>,
    /// SQL type name (e.g., `"INTEGER"`, `"VARCHAR"`).
    #[serde(default)]
    pub sql_type: Option<String>,
    /// Whether this column is hidden by default.
    #[serde(default)]
    pub hidden: bool,
    /// Whether this column allows null values.
    #[serde(default)]
    pub nullable: bool,
    /// Whether this column is read-only.
    #[serde(default)]
    pub read_only: bool,
    /// Whether this column is editable by users.
    #[serde(default)]
    pub user_editable: bool,
    /// Whether this column auto-increments.
    #[serde(default)]
    pub auto_increment: bool,
    /// Whether this column is a primary key field.
    #[serde(default)]
    pub key_field: bool,
    /// Whether missing-value indicators are enabled for this column.
    #[serde(default)]
    pub mv_enabled: bool,
}

/// Metadata block in a query response.
#[derive(Debug, Clone, Deserialize)]
pub struct ResponseMetadata {
    /// Column definitions.
    pub fields: Vec<QueryColumn>,
    /// Name of the primary key column.
    #[serde(default)]
    pub id: Option<String>,
    /// Name of the property containing rows (typically `"rows"`).
    #[serde(default)]
    pub root: Option<String>,
    /// Title of the underlying query.
    #[serde(default)]
    pub title: Option<String>,
    /// Description of the underlying query.
    #[serde(default)]
    pub description: Option<String>,
}

/// Response from [`LabkeyClient::select_rows`] or [`LabkeyClient::execute_sql`].
///
/// Both endpoints return the same response shape. The `format_version` field
/// reflects the requested API version (this crate always requests 17.1).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelectRowsResponse {
    /// Schema name (string, or array for nested schemas).
    pub schema_name: serde_json::Value,
    /// Query name, if applicable.
    #[serde(default)]
    pub query_name: Option<String>,
    /// Response format version (should be 17.1 when using this crate).
    #[serde(default)]
    pub format_version: Option<f64>,
    /// Number of rows returned.
    pub row_count: i64,
    /// The result rows.
    pub rows: Vec<Row>,
    /// Column metadata, if requested.
    #[serde(default)]
    pub meta_data: Option<ResponseMetadata>,
}

/// Options for [`LabkeyClient::select_rows`].
///
/// Only `schema_name` and `query_name` are required. All other fields
/// default to `None`, which means the server will use its own defaults.
/// Use the builder to construct:
///
/// ```
/// use labkey_rs::query::SelectRowsOptions;
///
/// let opts = SelectRowsOptions::builder()
///     .schema_name("lists")
///     .query_name("People")
///     .max_rows(100)
///     .build();
/// ```
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SelectRowsOptions {
    /// The schema containing the query (e.g., `"lists"`, `"core"`).
    pub schema_name: String,
    /// The query or table name.
    pub query_name: String,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Columns to include in the response. If `None`, the default view's
    /// columns are returned.
    pub columns: Option<Vec<String>>,
    /// Filters to apply to the query.
    pub filter_array: Option<Vec<Filter>>,
    /// Sort specification (e.g., `"Name"` or `"-Created"` for descending).
    pub sort: Option<String>,
    /// Named view to use.
    pub view_name: Option<String>,
    /// Maximum number of rows to return. Use a negative value to return
    /// all rows (the server sends `showRows=all`).
    pub max_rows: Option<i32>,
    /// Row offset for pagination.
    pub offset: Option<i64>,
    /// Container filter scope.
    pub container_filter: Option<ContainerFilter>,
    /// Whether to include the total row count in the response.
    pub include_total_count: Option<bool>,
    /// Whether to include column metadata in the response.
    pub include_metadata: Option<bool>,
    /// Whether to ignore the view's default filters.
    pub ignore_filter: Option<bool>,
    /// Whether to include a column with links to the details view.
    pub include_details_column: Option<bool>,
    /// Whether to include a column with links to the update view.
    pub include_update_column: Option<bool>,
    /// Whether to include style information in the response.
    pub include_style: Option<bool>,
    /// Selection key for managing grid selections.
    pub selection_key: Option<String>,
    /// Parameters for parameterized queries.
    pub parameters: Option<HashMap<String, String>>,
}

/// Options for [`LabkeyClient::execute_sql`].
///
/// Only `schema_name` and `sql` are required. Use the builder to construct:
///
/// ```
/// use labkey_rs::query::ExecuteSqlOptions;
///
/// let opts = ExecuteSqlOptions::builder()
///     .schema_name("core")
///     .sql("SELECT * FROM core.users")
///     .build();
/// ```
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct ExecuteSqlOptions {
    /// The schema to execute the SQL against.
    pub schema_name: String,
    /// The SQL query to execute. This will be WAF-encoded before sending.
    pub sql: String,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Maximum number of rows to return.
    pub max_rows: Option<i32>,
    /// Row offset for pagination.
    pub offset: Option<i64>,
    /// Sort specification.
    pub sort: Option<String>,
    /// Container filter scope.
    pub container_filter: Option<ContainerFilter>,
    /// Whether to include the total row count in the response.
    pub include_total_count: Option<bool>,
    /// Whether to include column metadata in the response.
    pub include_metadata: Option<bool>,
    /// Whether to save the query in the server session.
    pub save_in_session: Option<bool>,
    /// Whether to include style information in the response.
    pub include_style: Option<bool>,
    /// Parameters for parameterized queries.
    pub parameters: Option<HashMap<String, String>>,
}

/// Encode a string to avoid web application firewall false positives.
///
/// `LabKey` endpoints that accept SQL or script content use this encoding to
/// prevent WAFs from rejecting legitimate content. The encoding is
/// URL-encode first, then base64-encode, then prepend a magic prefix that
/// tells the server how to decode it.
///
/// This matches the JS client's `wafEncode` function in `Utils.ts`.
pub(crate) fn waf_encode(value: &str) -> String {
    let url_encoded = urlencoding::encode(value);
    let b64 = base64::engine::general_purpose::STANDARD.encode(url_encoded.as_bytes());
    format!("/*{{{{base64/x-www-form-urlencoded/wafText}}}}*/{b64}")
}

/// Serialize a [`ContainerFilter`] to its string representation for use
/// as a query parameter value.
fn container_filter_to_string(cf: ContainerFilter) -> String {
    // ContainerFilter's serde Serialize produces a JSON string like
    // "CurrentAndSubfolders". We strip the quotes to get the bare value.
    serde_json::to_value(cf)
        .ok()
        .and_then(|v| v.as_str().map(String::from))
        .unwrap_or_default()
}

/// Shorthand for building an optional query parameter pair.
fn opt<V: ToString>(key: impl Into<String>, value: Option<V>) -> Option<(String, String)> {
    value.map(|v| (key.into(), v.to_string()))
}

/// JSON body for the `executeSql.api` endpoint. Using a struct with
/// `skip_serializing_if` replaces the imperative `if let Some` pattern.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecuteSqlBody {
    schema_name: String,
    sql: String,
    api_version: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_rows: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    container_filter: Option<ContainerFilter>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_total_count: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_metadata: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    save_in_session: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_style: Option<bool>,
}

impl LabkeyClient {
    /// Select rows from a `LabKey` query.
    ///
    /// Sends a GET request to the `query-getQuery.api` endpoint with the
    /// specified filters, sort, pagination, and column options encoded as
    /// URL query parameters. Always requests the 17.1 response format.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the HTTP request fails, the server returns
    /// an error response, or the response body cannot be deserialized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), labkey_rs::LabkeyError> {
    /// # let config = labkey_rs::ClientConfig::new(
    /// #     "https://labkey.example.com/labkey",
    /// #     labkey_rs::Credential::ApiKey("key".into()),
    /// #     "/",
    /// # );
    /// # let client = labkey_rs::LabkeyClient::new(config)?;
    /// use labkey_rs::query::SelectRowsOptions;
    ///
    /// let response = client.select_rows(
    ///     SelectRowsOptions::builder()
    ///         .schema_name("lists")
    ///         .query_name("People")
    ///         .build(),
    /// ).await?;
    ///
    /// println!("Got {} rows", response.row_count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn select_rows(
        &self,
        options: SelectRowsOptions,
    ) -> Result<SelectRowsResponse, LabkeyError> {
        let url = self.build_url("query", "getQuery.api", options.container_path.as_deref());
        let dr = "query";

        let mut params: Vec<(String, String)> = [
            Some(("schemaName".into(), options.schema_name)),
            Some((format!("{dr}.queryName"), options.query_name)),
            Some(("apiVersion".into(), "17.1".into())),
            options
                .columns
                .map(|c| (format!("{dr}.columns"), c.join(","))),
            opt(format!("{dr}.sort"), options.sort),
            opt(format!("{dr}.offset"), options.offset),
            opt(format!("{dr}.viewName"), options.view_name),
            opt(format!("{dr}.selectionKey"), options.selection_key),
            opt(
                "containerFilter",
                options.container_filter.map(container_filter_to_string),
            ),
            opt("includeTotalCount", options.include_total_count),
            opt("includeMetadata", options.include_metadata),
            opt("includeDetailsColumn", options.include_details_column),
            opt("includeUpdateColumn", options.include_update_column),
            opt("includeStyle", options.include_style),
            options
                .ignore_filter
                .and_then(|v| v.then(|| (format!("{dr}.ignoreFilter"), "1".into()))),
            match options.max_rows {
                Some(max) if max < 0 => Some((format!("{dr}.showRows"), "all".into())),
                Some(max) => Some((format!("{dr}.maxRows"), max.to_string())),
                None => None,
            },
        ]
        .into_iter()
        .flatten()
        .collect();

        if let Some(filters) = &options.filter_array {
            params.extend(encode_filters(filters, dr));
        }
        if let Some(parameters) = &options.parameters {
            for (k, v) in parameters {
                params.push((format!("{dr}.param.{k}"), v.clone()));
            }
        }

        self.get(url, &params).await
    }

    /// Execute arbitrary `LabKey` SQL.
    ///
    /// Sends a POST request to the `query-executeSql.api` endpoint with the
    /// SQL in the JSON body (WAF-encoded to avoid firewall false positives).
    /// Sort and parameterized query parameters are sent as URL query params.
    /// Always requests the 17.1 response format.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the HTTP request fails, the server returns
    /// an error response, or the response body cannot be deserialized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), labkey_rs::LabkeyError> {
    /// # let config = labkey_rs::ClientConfig::new(
    /// #     "https://labkey.example.com/labkey",
    /// #     labkey_rs::Credential::ApiKey("key".into()),
    /// #     "/",
    /// # );
    /// # let client = labkey_rs::LabkeyClient::new(config)?;
    /// use labkey_rs::query::ExecuteSqlOptions;
    ///
    /// let response = client.execute_sql(
    ///     ExecuteSqlOptions::builder()
    ///         .schema_name("core")
    ///         .sql("SELECT DisplayName, Email FROM core.users")
    ///         .build(),
    /// ).await?;
    ///
    /// println!("Got {} rows", response.row_count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_sql(
        &self,
        options: ExecuteSqlOptions,
    ) -> Result<SelectRowsResponse, LabkeyError> {
        let mut url = self.build_url("query", "executeSql.api", options.container_path.as_deref());

        // Sort and parameterized query params go on the URL, not in the body.
        let mut url_params: Vec<(String, String)> = [opt("query.sort", options.sort)]
            .into_iter()
            .flatten()
            .collect();
        if let Some(parameters) = &options.parameters {
            for (k, v) in parameters {
                url_params.push((format!("query.param.{k}"), v.clone()));
            }
        }
        if !url_params.is_empty() {
            let mut pairs = url.query_pairs_mut();
            for (k, v) in &url_params {
                pairs.append_pair(k, v);
            }
        }

        let body = ExecuteSqlBody {
            schema_name: options.schema_name,
            sql: waf_encode(&options.sql),
            api_version: 17.1,
            max_rows: options.max_rows,
            offset: options.offset,
            container_filter: options.container_filter,
            include_total_count: options.include_total_count,
            include_metadata: options.include_metadata,
            save_in_session: options.save_in_session,
            include_style: options.include_style,
        };

        self.post(url, &body).await
    }
}

#[cfg(test)]
mod tests {
    use base64::Engine;

    use super::*;

    const WAF_PREFIX: &str = "/*{{base64/x-www-form-urlencoded/wafText}}*/";

    /// Strip the WAF prefix and decode back to the original string.
    /// Panics if the prefix is missing or decoding fails.
    fn waf_decode(encoded: &str) -> String {
        let after_prefix = encoded
            .strip_prefix(WAF_PREFIX)
            .expect("encoded value should start with exactly one WAF prefix");
        let decoded_bytes = base64::engine::general_purpose::STANDARD
            .decode(after_prefix)
            .expect("should be valid base64");
        let url_encoded = String::from_utf8(decoded_bytes).expect("should be valid UTF-8");
        urlencoding::decode(&url_encoded)
            .expect("should be valid URL encoding")
            .into_owned()
    }

    #[test]
    fn waf_encode_round_trips() {
        // Our URL encoding is slightly more aggressive than JS's
        // encodeURIComponent (e.g., we encode `*` as `%2A`), but the
        // round-trip produces the same original string.
        let sql = "SELECT * FROM core.users WHERE x > 1";
        assert_eq!(waf_decode(&waf_encode(sql)), sql);
    }

    #[test]
    fn waf_encode_empty_string() {
        let encoded = waf_encode("");
        assert_eq!(encoded, WAF_PREFIX);
        assert_eq!(waf_decode(&encoded), "");
    }

    #[test]
    fn waf_encode_special_characters() {
        let sql = "SELECT * FROM t WHERE name = 'O''Brien' AND val < 100";
        assert_eq!(waf_decode(&waf_encode(sql)), sql);
    }

    #[test]
    fn waf_encode_unicode() {
        let sql = "SELECT * FROM t WHERE label = '日本語' OR note LIKE '%café%'";
        assert_eq!(waf_decode(&waf_encode(sql)), sql);
    }

    #[test]
    fn container_filter_to_string_produces_variant_name() {
        assert_eq!(
            container_filter_to_string(ContainerFilter::CurrentAndSubfolders),
            "CurrentAndSubfolders"
        );
        assert_eq!(
            container_filter_to_string(ContainerFilter::AllFolders),
            "AllFolders"
        );
    }

    #[test]
    fn select_rows_response_deserializes_minimal() {
        let json = serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "formatVersion": 17.1,
            "rowCount": 1,
            "rows": [{
                "data": {
                    "Name": { "value": "Alice" },
                    "Age": { "value": 30 }
                }
            }]
        });
        let response: SelectRowsResponse =
            serde_json::from_value(json).expect("should deserialize");
        assert_eq!(response.row_count, 1);
        assert_eq!(response.rows.len(), 1);
        let row = &response.rows[0];
        assert_eq!(
            row.data["Name"].value,
            serde_json::Value::String("Alice".into())
        );
        assert_eq!(row.data["Age"].value, serde_json::json!(30));
    }

    #[test]
    fn select_rows_response_deserializes_with_metadata() {
        let json = serde_json::json!({
            "schemaName": "lists",
            "rowCount": 0,
            "rows": [],
            "metaData": {
                "fields": [{
                    "name": "RowId",
                    "fieldKey": "RowId",
                    "caption": "Row Id",
                    "jsonType": "int",
                    "keyField": true,
                    "autoIncrement": true
                }],
                "id": "RowId",
                "root": "rows",
                "title": "People"
            }
        });
        let response: SelectRowsResponse =
            serde_json::from_value(json).expect("should deserialize");
        let meta = response.meta_data.expect("should have metadata");
        assert_eq!(meta.fields.len(), 1);
        assert_eq!(meta.fields[0].name, "RowId");
        assert!(meta.fields[0].key_field);
        assert!(meta.fields[0].auto_increment);
        assert_eq!(meta.id, Some("RowId".into()));
        assert_eq!(meta.title, Some("People".into()));
    }

    #[test]
    fn cell_value_deserializes_with_all_fields() {
        let json = serde_json::json!({
            "value": 42,
            "displayValue": "forty-two",
            "formattedValue": "42.00",
            "url": "/labkey/list/details.view?id=42",
            "mvValue": "Q",
            "mvIndicator": "Q"
        });
        let cell: CellValue = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(cell.value, serde_json::json!(42));
        assert_eq!(cell.display_value.as_deref(), Some("forty-two"));
        assert_eq!(cell.formatted_value.as_deref(), Some("42.00"));
        assert_eq!(cell.url.as_deref(), Some("/labkey/list/details.view?id=42"));
        assert_eq!(cell.mv_value.as_deref(), Some("Q"));
        assert_eq!(cell.mv_indicator.as_deref(), Some("Q"));
    }

    #[test]
    fn cell_value_deserializes_minimal() {
        let json = serde_json::json!({ "value": null });
        let cell: CellValue = serde_json::from_value(json).expect("should deserialize");
        assert!(cell.value.is_null());
        assert!(cell.display_value.is_none());
        assert!(cell.formatted_value.is_none());
        assert!(cell.url.is_none());
        assert!(cell.mv_value.is_none());
        assert!(cell.mv_indicator.is_none());
    }

    #[test]
    fn row_deserializes_with_links() {
        let json = serde_json::json!({
            "data": {
                "Name": { "value": "Bob" }
            },
            "links": {
                "details": "/labkey/list/details.view?id=1"
            }
        });
        let row: Row = serde_json::from_value(json).expect("should deserialize");
        assert!(row.links.is_some());
        assert_eq!(
            row.data["Name"].value,
            serde_json::Value::String("Bob".into())
        );
    }

    #[test]
    fn select_rows_response_with_empty_rows() {
        let json = serde_json::json!({
            "schemaName": ["core", "nested"],
            "rowCount": 0,
            "rows": []
        });
        let response: SelectRowsResponse =
            serde_json::from_value(json).expect("should deserialize");
        assert_eq!(response.row_count, 0);
        assert!(response.rows.is_empty());
        // schemaName can be an array for nested schemas
        assert!(response.schema_name.is_array());
    }

    #[test]
    fn query_column_deserializes_with_defaults() {
        let json = serde_json::json!({
            "name": "Status",
            "fieldKey": "Status"
        });
        let col: QueryColumn = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(col.name, "Status");
        assert!(!col.hidden);
        assert!(!col.nullable);
        assert!(!col.read_only);
        assert!(!col.user_editable);
        assert!(!col.auto_increment);
        assert!(!col.key_field);
        assert!(!col.mv_enabled);
        assert!(col.caption.is_none());
        assert!(col.json_type.is_none());
    }
}
