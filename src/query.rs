//! Query endpoints and response types for the `LabKey` REST API.
//!
//! This module provides [`SelectRowsOptions`] and [`ExecuteSqlOptions`] for
//! the two primary query endpoints, along with the response types that model
//! the 17.1 response format. Both endpoints return a [`SelectRowsResponse`]
//! containing typed rows where each cell is a [`CellValue`] with the raw
//! value and optional display/formatting metadata.

use std::collections::HashMap;

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::{
    client::LabkeyClient,
    common::AuditBehavior,
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

/// Response from row mutation endpoints.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ModifyRowsResults {
    /// Command name returned by the server (for example, `"insert"`).
    pub command: String,
    /// Per-row or per-field errors returned by the server.
    #[serde(default)]
    pub errors: Vec<serde_json::Value>,
    /// Optional field identifier associated with the error.
    #[serde(default)]
    pub field: Option<String>,
    /// Query name included in the response.
    pub query_name: String,
    /// Returned row payloads in the same order as the request rows.
    #[serde(default)]
    pub rows: Vec<serde_json::Value>,
    /// Number of affected rows.
    pub rows_affected: i64,
    /// Schema name included in the response.
    pub schema_name: String,
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

/// Options for [`LabkeyClient::insert_rows`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct InsertRowsOptions {
    /// The schema containing the target query.
    pub schema_name: String,
    /// The query or table name.
    pub query_name: String,
    /// Row payloads to insert.
    pub rows: Vec<serde_json::Value>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Whether all inserts should execute in a single transaction.
    pub transacted: Option<bool>,
    /// Optional extra context passed to server-side scripts.
    pub extra_context: Option<serde_json::Value>,
    /// Audit behavior override for this mutation.
    pub audit_behavior: Option<AuditBehavior>,
    /// Optional audit details to record.
    pub audit_details: Option<serde_json::Value>,
    /// Optional user comment for audit records.
    pub audit_user_comment: Option<String>,
    /// Whether the server can skip detailed row reselection.
    pub skip_reselect_rows: Option<bool>,
}

/// Options for [`LabkeyClient::update_rows`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct UpdateRowsOptions {
    /// The schema containing the target query.
    pub schema_name: String,
    /// The query or table name.
    pub query_name: String,
    /// Row payloads to update.
    pub rows: Vec<serde_json::Value>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Whether all updates should execute in a single transaction.
    pub transacted: Option<bool>,
    /// Optional extra context passed to server-side scripts.
    pub extra_context: Option<serde_json::Value>,
    /// Audit behavior override for this mutation.
    pub audit_behavior: Option<AuditBehavior>,
    /// Optional audit details to record.
    pub audit_details: Option<serde_json::Value>,
    /// Optional user comment for audit records.
    pub audit_user_comment: Option<String>,
    /// Whether the server can skip detailed row reselection.
    pub skip_reselect_rows: Option<bool>,
}

/// Options for [`LabkeyClient::delete_rows`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct DeleteRowsOptions {
    /// The schema containing the target query.
    pub schema_name: String,
    /// The query or table name.
    pub query_name: String,
    /// Row payloads identifying rows to delete.
    pub rows: Vec<serde_json::Value>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Whether all deletes should execute in a single transaction.
    pub transacted: Option<bool>,
    /// Optional extra context passed to server-side scripts.
    pub extra_context: Option<serde_json::Value>,
    /// Audit behavior override for this mutation.
    pub audit_behavior: Option<AuditBehavior>,
    /// Optional audit details to record.
    pub audit_details: Option<serde_json::Value>,
    /// Optional user comment for audit records.
    pub audit_user_comment: Option<String>,
    /// Whether the server can skip detailed row reselection.
    pub skip_reselect_rows: Option<bool>,
}

/// Options for [`LabkeyClient::truncate_table`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct TruncateTableOptions {
    /// The schema containing the target query.
    pub schema_name: String,
    /// The query or table name.
    pub query_name: String,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Whether truncation should execute in a single transaction.
    pub transacted: Option<bool>,
    /// Optional extra context passed to server-side scripts.
    pub extra_context: Option<serde_json::Value>,
    /// Audit behavior override for this mutation.
    pub audit_behavior: Option<AuditBehavior>,
    /// Optional audit details to record.
    pub audit_details: Option<serde_json::Value>,
    /// Optional user comment for audit records.
    pub audit_user_comment: Option<String>,
    /// Whether the server can skip detailed row reselection.
    pub skip_reselect_rows: Option<bool>,
}

/// Command type for [`SaveRowsCommand`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum CommandType {
    /// Delete rows identified in the command payload.
    #[serde(rename = "delete")]
    Delete,
    /// Insert new rows from the command payload.
    #[serde(rename = "insert")]
    Insert,
    /// Update existing rows from the command payload.
    #[serde(rename = "update")]
    Update,
}

/// A single command in [`SaveRowsOptions`].
#[derive(Debug, Clone, bon::Builder, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SaveRowsCommand {
    /// Name of the command to perform.
    pub command: CommandType,
    /// The schema containing the target query.
    pub schema_name: String,
    /// The query or table name.
    pub query_name: String,
    /// Row payloads for this command.
    pub rows: Vec<serde_json::Value>,
    /// Override the default command-level container path.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_path: Option<String>,
    /// Optional extra context passed to server-side scripts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_context: Option<serde_json::Value>,
    /// Audit behavior override for this command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_behavior: Option<AuditBehavior>,
    /// Optional audit details to record for this command.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_details: Option<serde_json::Value>,
    /// Optional user comment for this command's audit records.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audit_user_comment: Option<String>,
    /// Whether the server can skip detailed row reselection.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reselect_rows: Option<bool>,
}

/// API version override for [`SaveRowsOptions`].
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum SaveRowsApiVersion {
    /// String API version representation (for example, `"17.1"`).
    String(String),
    /// Numeric API version representation (for example, `17.1`).
    Number(f64),
}

/// Options for [`LabkeyClient::move_rows`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct MoveRowsOptions {
    /// The schema containing the target query.
    pub schema_name: String,
    /// The query or table name.
    pub query_name: String,
    /// Target container path where rows should be moved.
    pub target_container_path: String,
    /// Row payloads identifying the rows to move.
    pub rows: Option<Vec<serde_json::Value>>,
    /// Override the client's default source container path for this request.
    pub container_path: Option<String>,
    /// Audit behavior override for this move operation.
    pub audit_behavior: Option<AuditBehavior>,
    /// Optional audit details to record.
    pub audit_details: Option<serde_json::Value>,
    /// Optional user comment for audit records.
    pub audit_user_comment: Option<String>,
    /// Data region selection key when rows are selected server-side.
    pub data_region_selection_key: Option<String>,
    /// Whether to use the snapshot selection with data region selection key.
    pub use_snapshot_selection: Option<bool>,
    /// Optional extra context passed to server-side scripts.
    pub extra_context: Option<serde_json::Value>,
}

/// Response from [`LabkeyClient::move_rows`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MoveRowsResponse {
    /// Core row-mutation response fields.
    #[serde(flatten)]
    pub result: ModifyRowsResults,
    /// Indicates if the move operation succeeded.
    pub success: bool,
    /// The container path where rows were moved.
    #[serde(default)]
    pub container_path: Option<String>,
    /// A summary error string, if present.
    #[serde(default)]
    pub error: Option<String>,
    /// Counts of moved entities keyed by category.
    #[serde(default)]
    pub update_counts: Option<HashMap<String, i64>>,
}

/// Options for [`LabkeyClient::save_rows`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SaveRowsOptions {
    /// Commands to execute as one save request.
    pub commands: Vec<SaveRowsCommand>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Optional audit details for the overall request.
    pub audit_details: Option<serde_json::Value>,
    /// Optional API version override accepted by the server.
    pub api_version: Option<SaveRowsApiVersion>,
    /// Optional extra context passed to server-side scripts.
    pub extra_context: Option<serde_json::Value>,
    /// Whether all commands should run in a single transaction.
    pub transacted: Option<bool>,
    /// Whether to validate commands without committing changes.
    pub validate_only: Option<bool>,
}

/// Response from [`LabkeyClient::save_rows`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SaveRowsResponse {
    /// Whether changes were committed.
    pub committed: bool,
    /// Total number of errors across all commands.
    pub error_count: i64,
    /// Per-command responses in request order.
    pub result: Vec<ModifyRowsResults>,
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
#[derive(Serialize)]
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

/// Shared request body for query mutation endpoints.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MutateRowsBody {
    schema_name: String,
    query_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    rows: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transacted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_context: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audit_behavior: Option<AuditBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audit_details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audit_user_comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    skip_reselect_rows: Option<bool>,
}

/// Request body for the `moveRows.api` endpoint.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct MoveRowsBody {
    target_container_path: String,
    schema_name: String,
    query_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    rows: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audit_behavior: Option<AuditBehavior>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audit_details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audit_user_comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    data_region_selection_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    use_snapshot_selection: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_context: Option<serde_json::Value>,
}

/// Request body for the `saveRows.api` endpoint.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveRowsBody {
    commands: Vec<SaveRowsCommand>,
    #[serde(skip_serializing_if = "Option::is_none")]
    container_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audit_details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    api_version: Option<SaveRowsApiVersion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    extra_context: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transacted: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    validate_only: Option<bool>,
}

impl LabkeyClient {
    async fn mutate_rows(
        &self,
        action: &str,
        container_path: Option<String>,
        body: &MutateRowsBody,
    ) -> Result<ModifyRowsResults, LabkeyError> {
        let url = self.build_url("query", action, container_path.as_deref());
        self.post(url, body).await
    }

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

    /// Insert rows into a query table.
    ///
    /// Sends a POST request to `query-insertRows.api`.
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
    /// use labkey_rs::query::InsertRowsOptions;
    ///
    /// let result = client
    ///     .insert_rows(
    ///         InsertRowsOptions::builder()
    ///             .schema_name("lists")
    ///             .query_name("People")
    ///             .rows(vec![serde_json::json!({"Name": "Alice"})])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Inserted {} rows", result.rows_affected);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn insert_rows(
        &self,
        options: InsertRowsOptions,
    ) -> Result<ModifyRowsResults, LabkeyError> {
        let body = MutateRowsBody {
            schema_name: options.schema_name,
            query_name: options.query_name,
            rows: Some(options.rows),
            transacted: options.transacted,
            extra_context: options.extra_context,
            audit_behavior: options.audit_behavior,
            audit_details: options.audit_details,
            audit_user_comment: options.audit_user_comment,
            skip_reselect_rows: options.skip_reselect_rows,
        };

        self.mutate_rows("insertRows.api", options.container_path, &body)
            .await
    }

    /// Update rows in a query table.
    ///
    /// Sends a POST request to `query-updateRows.api`.
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
    /// use labkey_rs::query::UpdateRowsOptions;
    ///
    /// let result = client
    ///     .update_rows(
    ///         UpdateRowsOptions::builder()
    ///             .schema_name("lists")
    ///             .query_name("People")
    ///             .rows(vec![serde_json::json!({"RowId": 1, "Name": "Alicia"})])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Updated {} rows", result.rows_affected);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_rows(
        &self,
        options: UpdateRowsOptions,
    ) -> Result<ModifyRowsResults, LabkeyError> {
        let body = MutateRowsBody {
            schema_name: options.schema_name,
            query_name: options.query_name,
            rows: Some(options.rows),
            transacted: options.transacted,
            extra_context: options.extra_context,
            audit_behavior: options.audit_behavior,
            audit_details: options.audit_details,
            audit_user_comment: options.audit_user_comment,
            skip_reselect_rows: options.skip_reselect_rows,
        };

        self.mutate_rows("updateRows.api", options.container_path, &body)
            .await
    }

    /// Delete rows from a query table.
    ///
    /// Sends a POST request to `query-deleteRows.api`.
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
    /// use labkey_rs::query::DeleteRowsOptions;
    ///
    /// let result = client
    ///     .delete_rows(
    ///         DeleteRowsOptions::builder()
    ///             .schema_name("lists")
    ///             .query_name("People")
    ///             .rows(vec![serde_json::json!({"RowId": 1})])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Deleted {} rows", result.rows_affected);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_rows(
        &self,
        options: DeleteRowsOptions,
    ) -> Result<ModifyRowsResults, LabkeyError> {
        let body = MutateRowsBody {
            schema_name: options.schema_name,
            query_name: options.query_name,
            rows: Some(options.rows),
            transacted: options.transacted,
            extra_context: options.extra_context,
            audit_behavior: options.audit_behavior,
            audit_details: options.audit_details,
            audit_user_comment: options.audit_user_comment,
            skip_reselect_rows: options.skip_reselect_rows,
        };

        self.mutate_rows("deleteRows.api", options.container_path, &body)
            .await
    }

    /// Delete all rows in a query table.
    ///
    /// Sends a POST request to `query-truncateTable.api`.
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
    /// use labkey_rs::query::TruncateTableOptions;
    ///
    /// let result = client
    ///     .truncate_table(
    ///         TruncateTableOptions::builder()
    ///             .schema_name("lists")
    ///             .query_name("People")
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Rows affected: {}", result.rows_affected);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn truncate_table(
        &self,
        options: TruncateTableOptions,
    ) -> Result<ModifyRowsResults, LabkeyError> {
        let body = MutateRowsBody {
            schema_name: options.schema_name,
            query_name: options.query_name,
            rows: None,
            transacted: options.transacted,
            extra_context: options.extra_context,
            audit_behavior: options.audit_behavior,
            audit_details: options.audit_details,
            audit_user_comment: options.audit_user_comment,
            skip_reselect_rows: options.skip_reselect_rows,
        };

        self.mutate_rows("truncateTable.api", options.container_path, &body)
            .await
    }

    /// Move rows to a different container.
    ///
    /// Sends a POST request to `query-moveRows.api`.
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
    /// use labkey_rs::query::MoveRowsOptions;
    ///
    /// let response = client
    ///     .move_rows(
    ///         MoveRowsOptions::builder()
    ///             .schema_name("lists".to_string())
    ///             .query_name("People".to_string())
    ///             .target_container_path("/Target/Folder".to_string())
    ///             .rows(vec![serde_json::json!({"RowId": 1})])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Move success: {}", response.success);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn move_rows(
        &self,
        options: MoveRowsOptions,
    ) -> Result<MoveRowsResponse, LabkeyError> {
        let body = MoveRowsBody {
            target_container_path: options.target_container_path,
            schema_name: options.schema_name,
            query_name: options.query_name,
            rows: options.rows,
            audit_behavior: options.audit_behavior,
            audit_details: options.audit_details,
            audit_user_comment: options.audit_user_comment,
            data_region_selection_key: options.data_region_selection_key,
            use_snapshot_selection: options.use_snapshot_selection,
            extra_context: options.extra_context,
        };

        let url = self.build_url("query", "moveRows.api", options.container_path.as_deref());
        self.post(url, &body).await
    }

    /// Save one or more insert, update, or delete commands.
    ///
    /// Sends a POST request to `query-saveRows.api`.
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
    /// use labkey_rs::query::{CommandType, SaveRowsCommand, SaveRowsOptions};
    ///
    /// let command = SaveRowsCommand::builder()
    ///     .command(CommandType::Update)
    ///     .schema_name("lists".to_string())
    ///     .query_name("People".to_string())
    ///     .rows(vec![serde_json::json!({"RowId": 1, "Name": "Alicia"})])
    ///     .build();
    ///
    /// let response = client
    ///     .save_rows(
    ///         SaveRowsOptions::builder()
    ///             .commands(vec![command])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Committed: {}", response.committed);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_rows(
        &self,
        options: SaveRowsOptions,
    ) -> Result<SaveRowsResponse, LabkeyError> {
        let url = self.build_url("query", "saveRows.api", options.container_path.as_deref());
        let body = SaveRowsBody {
            commands: options.commands,
            container_path: options.container_path,
            audit_details: options.audit_details,
            api_version: options.api_version,
            extra_context: options.extra_context,
            transacted: options.transacted,
            validate_only: options.validate_only,
        };

        self.post(url, &body).await
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

    fn test_client(base_url: &str, container_path: &str) -> LabkeyClient {
        LabkeyClient::new(crate::ClientConfig::new(
            base_url,
            crate::Credential::ApiKey("test-key".to_string()),
            container_path,
        ))
        .expect("valid client config")
    }

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

    #[test]
    fn mutation_endpoint_urls_match_expected_actions() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url("query", "insertRows.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-insertRows.api"
        );
        assert_eq!(
            client
                .build_url("query", "updateRows.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-updateRows.api"
        );
        assert_eq!(
            client
                .build_url("query", "deleteRows.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-deleteRows.api"
        );
        assert_eq!(
            client
                .build_url("query", "truncateTable.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-truncateTable.api"
        );
        assert_eq!(
            client
                .build_url("query", "moveRows.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-moveRows.api"
        );
        assert_eq!(
            client
                .build_url("query", "saveRows.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-saveRows.api"
        );
    }

    #[test]
    fn save_rows_command_serializes_command_wire_value() {
        let command = SaveRowsCommand::builder()
            .command(CommandType::Update)
            .schema_name("lists".to_string())
            .query_name("People".to_string())
            .rows(vec![serde_json::json!({"RowId": 1, "Name": "Alicia"})])
            .skip_reselect_rows(true)
            .build();

        let json = serde_json::to_value(command).expect("should serialize command");
        assert_eq!(json["command"], serde_json::json!("update"));
        assert_eq!(json["schemaName"], serde_json::json!("lists"));
        assert_eq!(json["queryName"], serde_json::json!("People"));
        assert_eq!(json["skipReselectRows"], serde_json::json!(true));
    }

    #[test]
    fn command_type_serializes_exact_wire_values() {
        assert_eq!(
            serde_json::to_string(&CommandType::Delete).expect("should serialize"),
            "\"delete\""
        );
        assert_eq!(
            serde_json::to_string(&CommandType::Insert).expect("should serialize"),
            "\"insert\""
        );
        assert_eq!(
            serde_json::to_string(&CommandType::Update).expect("should serialize"),
            "\"update\""
        );
    }

    #[test]
    fn command_type_deserializes_exact_wire_values() {
        let delete: CommandType = serde_json::from_str("\"delete\"").expect("should deserialize");
        let insert: CommandType = serde_json::from_str("\"insert\"").expect("should deserialize");
        let update: CommandType = serde_json::from_str("\"update\"").expect("should deserialize");

        assert_eq!(delete, CommandType::Delete);
        assert_eq!(insert, CommandType::Insert);
        assert_eq!(update, CommandType::Update);
    }

    #[test]
    fn command_type_rejects_unknown_wire_value() {
        let err = serde_json::from_str::<CommandType>("\"upsert\"")
            .expect_err("unknown command type should fail to deserialize");
        assert!(err.is_data());
    }

    fn command_type_variant_count(value: CommandType) -> usize {
        match value {
            CommandType::Delete | CommandType::Insert | CommandType::Update => 3,
        }
    }

    #[test]
    fn command_type_variant_count_regression() {
        assert_eq!(command_type_variant_count(CommandType::Delete), 3);
    }

    #[test]
    fn mutate_rows_body_includes_required_and_omits_absent_optional_fields() {
        let body = MutateRowsBody {
            schema_name: "lists".to_string(),
            query_name: "People".to_string(),
            rows: Some(vec![serde_json::json!({"Name": "Alice"})]),
            transacted: None,
            extra_context: None,
            audit_behavior: Some(AuditBehavior::Summary),
            audit_details: None,
            audit_user_comment: None,
            skip_reselect_rows: None,
        };

        let value = serde_json::to_value(&body).expect("should serialize body");
        let obj = value.as_object().expect("body should be object");

        assert_eq!(obj.get("schemaName"), Some(&serde_json::json!("lists")));
        assert_eq!(obj.get("queryName"), Some(&serde_json::json!("People")));
        assert_eq!(
            obj.get("rows"),
            Some(&serde_json::json!([{"Name": "Alice"}]))
        );
        assert_eq!(
            obj.get("auditBehavior"),
            Some(&serde_json::json!("SUMMARY"))
        );
        assert!(!obj.contains_key("transacted"));
        assert!(!obj.contains_key("extraContext"));
        assert!(!obj.contains_key("auditDetails"));
        assert!(!obj.contains_key("auditUserComment"));
        assert!(!obj.contains_key("skipReselectRows"));
    }

    #[test]
    fn truncate_mutate_rows_body_omits_rows() {
        let body = MutateRowsBody {
            schema_name: "lists".to_string(),
            query_name: "People".to_string(),
            rows: None,
            transacted: Some(true),
            extra_context: None,
            audit_behavior: Some(AuditBehavior::Detailed),
            audit_details: Some(serde_json::json!({"reason": "cleanup"})),
            audit_user_comment: Some("Maintenance".to_string()),
            skip_reselect_rows: Some(true),
        };

        let value = serde_json::to_value(&body).expect("should serialize body");
        let obj = value.as_object().expect("body should be object");

        assert!(!obj.contains_key("rows"));
        assert_eq!(obj.get("transacted"), Some(&serde_json::json!(true)));
        assert_eq!(
            obj.get("auditBehavior"),
            Some(&serde_json::json!("DETAILED"))
        );
        assert_eq!(
            obj.get("auditDetails"),
            Some(&serde_json::json!({"reason": "cleanup"}))
        );
        assert_eq!(
            obj.get("auditUserComment"),
            Some(&serde_json::json!("Maintenance"))
        );
        assert_eq!(obj.get("skipReselectRows"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn modify_rows_results_deserializes_happy_path() {
        let json = serde_json::json!({
            "command": "insert",
            "errors": [],
            "queryName": "People",
            "rows": [{"RowId": 42, "Name": "Alice"}],
            "rowsAffected": 1,
            "schemaName": "lists"
        });

        let result: ModifyRowsResults = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(result.command, "insert");
        assert_eq!(result.query_name, "People");
        assert_eq!(result.rows_affected, 1);
        assert_eq!(result.schema_name, "lists");
        assert_eq!(result.rows.len(), 1);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn modify_rows_results_deserializes_minimal_with_zero_rows_affected() {
        let json = serde_json::json!({
            "command": "truncate",
            "queryName": "People",
            "rowsAffected": 0,
            "schemaName": "lists"
        });

        let result: ModifyRowsResults = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(result.command, "truncate");
        assert_eq!(result.rows_affected, 0);
        assert!(result.rows.is_empty());
        assert!(result.errors.is_empty());
        assert!(result.field.is_none());
    }

    #[test]
    fn move_rows_response_deserializes_happy_path() {
        let json = serde_json::json!({
            "command": "update",
            "errors": [],
            "queryName": "People",
            "rows": [{"RowId": 1}],
            "rowsAffected": 1,
            "schemaName": "lists",
            "success": true,
            "containerPath": "/Target/Folder",
            "updateCounts": {"rows": 1}
        });

        let response: MoveRowsResponse = serde_json::from_value(json).expect("should deserialize");
        assert!(response.success);
        assert_eq!(response.result.command, "update");
        assert_eq!(response.result.rows_affected, 1);
        assert_eq!(response.container_path.as_deref(), Some("/Target/Folder"));
        assert_eq!(
            response.update_counts.as_ref().and_then(|v| v.get("rows")),
            Some(&1)
        );
    }

    #[test]
    fn move_rows_response_deserializes_minimal_without_optional_fields() {
        let json = serde_json::json!({
            "command": "update",
            "errors": [],
            "queryName": "People",
            "rows": [],
            "rowsAffected": 0,
            "schemaName": "lists",
            "success": false
        });

        let response: MoveRowsResponse = serde_json::from_value(json).expect("should deserialize");
        assert!(!response.success);
        assert_eq!(response.result.rows_affected, 0);
        assert!(response.container_path.is_none());
        assert!(response.error.is_none());
        assert!(response.update_counts.is_none());
    }

    #[test]
    fn save_rows_response_deserializes_with_empty_commands() {
        let json = serde_json::json!({
            "committed": true,
            "errorCount": 0,
            "result": []
        });

        let response: SaveRowsResponse = serde_json::from_value(json).expect("should deserialize");
        assert!(response.committed);
        assert_eq!(response.error_count, 0);
        assert!(response.result.is_empty());
    }
}
