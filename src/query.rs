//! Query endpoints and response types for the `LabKey` REST API.
//!
//! This module provides [`SelectRowsOptions`] and [`ExecuteSqlOptions`] for
//! the two primary query endpoints, along with the response types that model
//! the 17.1 response format. Both endpoints return a [`SelectRowsResponse`]
//! containing typed rows where each cell is a [`CellValue`] with the raw
//! value and optional display/formatting metadata.

use std::collections::HashMap;

use base64::Engine;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    client::LabkeyClient,
    common::{AuditBehavior, container_filter_to_string, opt},
    error::LabkeyError,
    filter::{ContainerFilter, Filter, encode_filters},
};

/// Prefix used by `LabKey` for URL-valued hidden columns.
pub const URL_COLUMN_PREFIX: &str = "_labkeyurl_";

/// HTTP method for query read endpoints like [`LabkeyClient::select_rows`].
///
/// Defaults to [`Get`](RequestMethod::Get). The `Post` variant sends parameters
/// as an `application/x-www-form-urlencoded` body instead of URL query string
/// parameters, which avoids URL length limits for requests with complex filters
/// or many columns. This follows the JS client convention; the Java client
/// always POSTs with a JSON body, but we match JS since our parameter encoding
/// is already JS-compatible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum RequestMethod {
    /// Send parameters as URL query string (default).
    #[default]
    Get,
    /// Send parameters as a form-encoded request body.
    Post,
}

/// Controls which rows the server returns for query read endpoints.
///
/// When set to anything other than [`Paginated`](ShowRows::Paginated), the
/// `max_rows` and `offset` options on [`SelectRowsOptions`] are ignored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ShowRows {
    /// Return all rows, ignoring pagination.
    All,
    /// Return no data rows (metadata only).
    None,
    /// Honor `max_rows` and `offset` for pagination (default when omitted).
    Paginated,
    /// Return only rows in the current grid selection (requires `selection_key`).
    Selected,
    /// Return only rows NOT in the current grid selection (requires `selection_key`).
    Unselected,
}

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
#[non_exhaustive]
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
    #[serde(default, alias = "isHidden")]
    pub hidden: bool,
    /// Whether this column allows null values.
    #[serde(default, alias = "isNullable")]
    pub nullable: bool,
    /// Whether this column is read-only.
    #[serde(default, alias = "isReadOnly")]
    pub read_only: bool,
    /// Whether this column is editable by users.
    #[serde(default, alias = "isUserEditable")]
    pub user_editable: bool,
    /// Whether this column auto-increments.
    #[serde(default, alias = "isAutoIncrement")]
    pub auto_increment: bool,
    /// Whether this column is a primary key field.
    #[serde(default, alias = "isKeyField")]
    pub key_field: bool,
    /// Whether missing-value indicators are enabled for this column.
    #[serde(default, alias = "isMvEnabled")]
    pub mv_enabled: bool,
    /// Foreign-key lookup metadata for this column, if it references another table.
    #[serde(default)]
    pub lookup: Option<QueryLookup>,
    /// Whether this column is selectable in views.
    #[serde(default, alias = "isSelectable")]
    pub selectable: bool,
    /// Whether this column is a version/timestamp field.
    #[serde(default, alias = "isVersionField")]
    pub version_field: bool,
}

/// Metadata block in a query response.
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
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
    /// Server-provided message about data import behavior, if any.
    #[serde(default, rename = "importMessage")]
    pub import_message: Option<String>,
    /// Available import templates for the underlying query.
    #[serde(default, rename = "importTemplates")]
    pub import_templates: Vec<QueryImportTemplate>,
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

/// Response from [`LabkeyClient::truncate_table`].
///
/// The `LabKey` server returns `deletedRows` (not `rowsAffected`) for
/// truncation operations. All fields are optional to match the Java
/// client's `getProperty` null-return pattern.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TruncateTableResponse {
    /// Number of rows deleted by the truncation.
    #[serde(default)]
    pub deleted_rows: Option<i64>,
    /// Schema name affected by the command.
    #[serde(default)]
    pub schema_name: Option<String>,
    /// Query name affected by the command.
    #[serde(default)]
    pub query_name: Option<String>,
    /// Command name returned by the server.
    #[serde(default)]
    pub command: Option<String>,
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
    /// Prefix for query-region parameters (e.g., filters, sorts, columns).
    /// Defaults to `"query"` when omitted, matching the JS client's
    /// `ensureRegionName` behavior.
    pub data_region_name: Option<String>,
    /// HTTP method for this request. Defaults to [`RequestMethod::Get`].
    /// Use [`RequestMethod::Post`] to send parameters as a form-encoded body
    /// instead of URL query string, which avoids URL length limits for
    /// requests with complex filters or many columns.
    pub method: Option<RequestMethod>,
    /// Controls which rows the server returns. When set to anything other
    /// than [`ShowRows::Paginated`], `max_rows` and `offset` are ignored.
    /// When omitted, the server uses paginated mode by default.
    pub show_rows: Option<ShowRows>,
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
    /// Whether to include the details column in the response.
    pub include_details_column: Option<bool>,
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

/// Options for [`LabkeyClient::select_distinct_rows`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SelectDistinctOptions {
    /// The schema containing the target query.
    pub schema_name: String,
    /// The query or table name.
    pub query_name: String,
    /// A single column for which to request distinct values.
    pub column: String,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Prefix for query-region parameters. Defaults to `"query"` when omitted.
    pub data_region_name: Option<String>,
    /// Filters to apply to the query.
    pub filter_array: Option<Vec<Filter>>,
    /// Sort specification (for example, `"Name"` or `"-Created"`).
    pub sort: Option<String>,
    /// Named view to use.
    pub view_name: Option<String>,
    /// Maximum number of rows to return. Use `-1` to request all rows.
    pub max_rows: Option<i32>,
    /// Container filter scope.
    pub container_filter: Option<ContainerFilter>,
    /// Whether to ignore the selected view's default filters.
    pub ignore_filter: Option<bool>,
    /// Parameters for parameterized queries.
    pub parameters: Option<HashMap<String, String>>,
    /// HTTP method for the request. When set to [`RequestMethod::Post`], the
    /// request is sent as a POST with form-encoded body instead of the default
    /// GET with query parameters. This avoids URL length limits for requests
    /// with complex filters.
    pub method: Option<RequestMethod>,
}

/// Response from [`LabkeyClient::select_distinct_rows`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SelectDistinctResponse {
    /// Query name included in the response.
    pub query_name: String,
    /// Schema name included in the response.
    pub schema_name: String,
    /// Distinct values returned for the requested column.
    pub values: Vec<serde_json::Value>,
}

/// Options for [`LabkeyClient::get_query_details`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetQueryDetailsOptions {
    /// The schema containing the target query.
    pub schema_name: String,
    /// The query or table name.
    pub query_name: String,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Include custom view details for the provided view names.
    pub view_name: Option<Vec<String>>,
    /// Include only metadata for the provided field keys.
    pub fields: Option<Vec<String>>,
    /// Include only columns from the specified foreign key query.
    pub fk: Option<String>,
    /// Initialize the view from default if it does not exist.
    pub initialize_missing_view: Option<bool>,
    /// Include trigger metadata in the response.
    pub include_triggers: Option<bool>,
    /// Include suggested columns from related tables (Java-only parameter).
    pub include_suggested_query_columns: Option<bool>,
}

/// Options for [`LabkeyClient::get_queries`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetQueriesOptions {
    /// The schema for which available queries should be listed.
    pub schema_name: String,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Include column metadata for each query.
    pub include_columns: Option<bool>,
    /// Include `LabKey` system-defined queries.
    pub include_system_queries: Option<bool>,
    /// Include custom query titles.
    pub include_title: Option<bool>,
    /// Include user-defined queries.
    pub include_user_queries: Option<bool>,
    /// Include view-data URLs in each query record.
    pub include_view_data_url: Option<bool>,
    /// Include detailed query column metadata.
    pub query_detail_columns: Option<bool>,
}

/// Options for [`LabkeyClient::get_schemas`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetSchemasOptions {
    /// Optional API version string sent to the server.
    pub api_version: Option<String>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Include hidden schemas in the result.
    pub include_hidden: Option<bool>,
    /// Filter the response to one schema name.
    pub schema_name: Option<String>,
}

/// Source type for [`GetDataSource`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GetDataSourceType {
    /// Read rows from a schema/query source.
    Query,
    /// Read rows from a SQL source.
    Sql,
}

/// Source configuration for [`LabkeyClient::get_data`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum GetDataSource {
    /// Query-backed getData source.
    Query {
        /// Schema containing the query.
        schema_name: String,
        /// Query name to execute.
        query_name: String,
    },
    /// SQL-backed getData source.
    Sql {
        /// Schema used as SQL execution context.
        schema_name: String,
        /// SQL text to execute.
        sql: String,
    },
}

impl GetDataSource {
    /// Return the source discriminator for this value.
    #[must_use]
    pub fn source_type(&self) -> GetDataSourceType {
        match self {
            Self::Query { .. } => GetDataSourceType::Query,
            Self::Sql { .. } => GetDataSourceType::Sql,
        }
    }
}

/// Sort direction for [`GetDataSort`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum GetDataSortDirection {
    /// Ascending sort.
    Asc,
    /// Descending sort.
    Desc,
}

/// Sort descriptor for [`LabkeyClient::get_data`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetDataSort {
    /// Field key parts of the column to sort.
    pub field_key: Vec<String>,
    /// Sort direction. Defaults to server behavior when omitted.
    pub dir: Option<GetDataSortDirection>,
}

/// Filter descriptor for [`GetDataTransform`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetDataFilter {
    /// Field key parts targeted by the filter.
    pub field_key: Vec<String>,
    /// Filter type URL suffix.
    pub type_: String,
    /// Optional filter value payload.
    pub value: Option<serde_json::Value>,
}

/// Aggregate descriptor for [`GetDataTransform`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetDataAggregate {
    /// Field key parts targeted by the aggregate.
    pub field_key: Vec<String>,
    /// Aggregate type.
    pub type_: String,
}

/// Transform descriptor for [`LabkeyClient::get_data`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetDataTransform {
    /// Transform type (for example, `aggregate`).
    pub type_: Option<String>,
    /// Group-by field keys.
    pub group_by: Option<Vec<Vec<String>>>,
    /// Transform-level filters.
    pub filters: Option<Vec<GetDataFilter>>,
    /// Transform-level aggregates.
    pub aggregates: Option<Vec<GetDataAggregate>>,
}

/// Pivot descriptor for [`LabkeyClient::get_data`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetDataPivot {
    /// Field key parts used as the pivot axis.
    pub by: Vec<String>,
    /// Field keys to pivot.
    pub columns: Vec<Vec<String>>,
}

/// Options for [`LabkeyClient::get_data`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetDataOptions {
    /// Source configuration for query/sql mode.
    pub source: GetDataSource,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Renderer column field keys.
    pub columns: Option<Vec<Vec<String>>>,
    /// Include details-link pseudo column when supported.
    pub include_details_column: Option<bool>,
    /// Maximum number of rows to return.
    pub max_rows: Option<i32>,
    /// Row offset used for paging.
    pub offset: Option<i64>,
    /// Sort descriptors.
    pub sort: Option<Vec<GetDataSort>>,
    /// Optional transforms applied server-side.
    pub transforms: Option<Vec<GetDataTransform>>,
    /// Optional pivot configuration.
    pub pivot: Option<GetDataPivot>,
}

/// Data view categories supported by [`LabkeyClient::get_data_views`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DataViewType {
    /// Include dataset-backed data views.
    #[serde(rename = "datasets")]
    Datasets,
    /// Include query-backed data views.
    #[serde(rename = "queries")]
    Queries,
    /// Include report-backed data views.
    #[serde(rename = "reports")]
    Reports,
}

/// Options for [`LabkeyClient::get_data_views`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetDataViewsOptions {
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Restrict the browse-data response to one or more data-view categories.
    pub data_types: Option<Vec<DataViewType>>,
}

/// Options for [`LabkeyClient::validate_query`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct ValidateQueryOptions {
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Schema containing the query to validate.
    pub schema_name: Option<String>,
    /// Query name to validate.
    pub query_name: Option<String>,
    /// Optional SQL payload to validate.
    pub sql: Option<String>,
    /// Optional saved view name to include in validation.
    pub view_name: Option<String>,
    /// Validate query metadata and custom views in addition to parse/execute.
    pub validate_query_metadata: Option<bool>,
}

/// Response from [`LabkeyClient::validate_query`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ValidateQueryResponse {
    /// Indicates whether the query validated successfully.
    pub valid: bool,
    /// Additional server-provided response fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response from [`LabkeyClient::get_server_date`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GetServerDateResponse {
    /// Server-local current date/time value.
    pub date: String,
}

/// Insert strategy used by [`LabkeyClient::import_data`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum InsertOption {
    /// Import new rows only.
    #[serde(rename = "IMPORT")]
    Import,
    /// Import rows and merge updates when supported by the target table.
    #[serde(rename = "MERGE")]
    Merge,
}

/// Data source variants accepted by [`LabkeyClient::import_data`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ImportDataSource {
    /// Inline delimited text payload (for example, CSV or TSV content).
    Text(String),
    /// Upload file bytes as a multipart `file` part.
    File {
        /// Filename sent for the multipart file part.
        file_name: String,
        /// Raw file bytes uploaded to the server.
        bytes: Vec<u8>,
        /// Optional MIME type for the file part.
        mime_type: Option<String>,
    },
    /// Server-relative path from the webdav root.
    Path(String),
    /// Module resource source requiring both module and resource path.
    ModuleResource {
        /// Module name that owns the resource.
        module: String,
        /// Resource path inside the module.
        module_resource: String,
    },
}

/// Options for [`LabkeyClient::import_data`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct ImportDataOptions {
    /// The schema containing the target query.
    pub schema_name: String,
    /// The query or table name.
    pub query_name: String,
    /// Exactly one import payload source.
    pub source: ImportDataSource,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Explicit format hint (for example, `csv` or `tsv`).
    pub format: Option<String>,
    /// Insert behavior override.
    pub insert_option: Option<InsertOption>,
    /// Queue the import as an asynchronous pipeline job.
    pub use_async: Option<bool>,
    /// Save uploaded files to pipeline root.
    pub save_to_pipeline: Option<bool>,
    /// Use import identity behavior when supported by the target table.
    pub import_identity: Option<bool>,
    /// Match lookup rows using alternate key columns.
    pub import_lookup_by_alternate_key: Option<bool>,
    /// Optional user comment attached to import audit records.
    pub audit_user_comment: Option<String>,
    /// Optional structured details attached to import audit records.
    pub audit_details: Option<serde_json::Value>,
}

/// Response from [`LabkeyClient::import_data`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ImportDataResponse {
    /// Whether the import request was accepted by the server.
    pub success: bool,
    /// Number of rows imported when available.
    #[serde(default)]
    pub row_count: Option<i64>,
    /// Pipeline job id for asynchronous imports.
    #[serde(default)]
    pub job_id: Option<String>,
}

/// Options for [`LabkeyClient::get_query_views`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetQueryViewsOptions {
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Exclude the current session-scoped view when listing views.
    pub exclude_session_view: Option<bool>,
    /// Optional metadata payload echoed by the server.
    pub metadata: Option<serde_json::Value>,
    /// Query name to fetch views for.
    pub query_name: Option<String>,
    /// Schema name containing the query.
    pub schema_name: Option<String>,
    /// Filter to a specific view name.
    pub view_name: Option<String>,
}

/// Options for [`LabkeyClient::save_query_views`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SaveQueryViewsOptions {
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Marks the saved view(s) as hidden when true.
    pub hidden: Option<bool>,
    /// Optional metadata payload forwarded to the server.
    pub metadata: Option<serde_json::Value>,
    /// Query name for the view save operation.
    pub query_name: Option<String>,
    /// Schema name for the view save operation.
    pub schema_name: Option<String>,
    /// Marks the saved view(s) as session scoped when true.
    pub session: Option<bool>,
    /// Marks the saved view(s) as shared when true.
    pub shared: Option<bool>,
    /// View definitions to create or update.
    pub views: Option<serde_json::Value>,
}

/// Options for [`LabkeyClient::save_session_view`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SaveSessionViewOptions {
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Marks the saved view as hidden when true.
    pub hidden: Option<bool>,
    /// Makes the saved view available to child containers when true.
    pub inherit: Option<bool>,
    /// New non-session view name to save as.
    pub new_name: Option<String>,
    /// Query name copied from the session view.
    pub query_name: Option<String>,
    /// Replaces an existing target view when true.
    pub replace: Option<bool>,
    /// Schema name containing the query.
    pub schema_name: Option<String>,
    /// Marks the saved view as shared when true.
    pub shared: Option<bool>,
    /// Session view name to persist.
    pub view_name: Option<String>,
}

/// Options for [`LabkeyClient::delete_query_view`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct DeleteQueryViewOptions {
    /// Schema name containing the query.
    pub schema_name: String,
    /// Query name containing the view.
    pub query_name: String,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// View name to delete or revert.
    pub view_name: Option<String>,
    /// Revert mode flag from `LabKey`'s delete-view API.
    pub revert: Option<bool>,
}

/// Query metadata entry in [`GetQueriesResponse`].
// This endpoint exposes several independent capability flags in one object;
// preserving bool fields keeps parity with server payload semantics.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QueryInfo {
    /// Whether the query is editable.
    #[serde(default)]
    pub can_edit: bool,
    /// Whether shared views are editable.
    #[serde(default)]
    pub can_edit_shared_views: bool,
    /// Query columns in this listing response.
    ///
    /// This field is intentionally untyped because `LabKey` varies column
    /// payload detail based on request flags (for example,
    /// `queryDetailColumns`).
    #[serde(default)]
    pub columns: Vec<serde_json::Value>,
    /// Query description.
    #[serde(default)]
    pub description: Option<String>,
    /// Whether the query is hidden.
    #[serde(default)]
    pub hidden: bool,
    /// Whether the query is inherited from a parent container.
    #[serde(default)]
    pub inherit: bool,
    /// Whether inherited metadata is currently active.
    #[serde(default)]
    pub is_inherited: bool,
    /// Whether metadata is overrideable.
    #[serde(default)]
    pub is_metadata_overrideable: bool,
    /// Whether the query is user-defined.
    #[serde(default)]
    pub is_user_defined: bool,
    /// Query name.
    pub name: String,
    /// Whether the query is a snapshot query.
    #[serde(default)]
    pub snapshot: bool,
    /// Query title.
    #[serde(default)]
    pub title: Option<String>,
    /// URL for viewing query data.
    #[serde(default)]
    pub view_data_url: Option<String>,
}

/// Response from [`LabkeyClient::get_queries`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GetQueriesResponse {
    /// Query list returned for the schema.
    pub queries: Vec<QueryInfo>,
    /// Schema name included in the response.
    pub schema_name: String,
}

/// Lookup metadata attached to a query column.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QueryLookup {
    /// Optional container identifier.
    #[serde(default)]
    pub container: Option<String>,
    /// Optional container path.
    #[serde(default)]
    pub container_path: Option<String>,
    /// Display column for lookup resolution.
    #[serde(default)]
    pub display_column: Option<String>,
    /// Whether the lookup is marked public in server metadata.
    ///
    /// Accepts both `isPublic` (from `camelCase` rename) and bare `public`
    /// wire keys; the server may send either form depending on version.
    #[serde(default, alias = "public")]
    pub is_public: Option<bool>,
    /// Key column in the lookup table.
    #[serde(default)]
    pub key_column: Option<String>,
    /// Query name for the lookup target.
    #[serde(default)]
    pub query_name: Option<String>,
    /// Schema name for the lookup target (canonical form).
    #[serde(default)]
    pub schema_name: Option<String>,
    /// Schema name alias — some server versions send a bare `schema` key
    /// alongside `schemaName`. Both carry the same value in practice.
    #[serde(default)]
    pub schema: Option<String>,
    /// Table name for the lookup target (may differ from `query_name`).
    #[serde(default)]
    pub table: Option<String>,
    /// Multi-valued lookup mode (e.g., `"junction"`).
    #[serde(default)]
    pub multi_valued: Option<String>,
    /// Junction lookup name for multi-valued relationships.
    #[serde(default)]
    pub junction_lookup: Option<String>,
    /// Filter groups for the lookup, if any.
    #[serde(default)]
    pub filter_groups: Option<serde_json::Value>,
}

/// Column metadata returned by [`LabkeyClient::get_query_details`].
// The LabKey server response exposes many independent boolean attributes for
// query-detail columns; preserving the wire shape keeps parity and avoids
// collapsing semantics into custom enums.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QueryDetailsColumn {
    /// Internal column name.
    pub name: String,
    /// Field key.
    pub field_key: serde_json::Value,
    /// Human-readable caption.
    #[serde(default)]
    pub caption: Option<String>,
    /// Short caption for compact displays.
    #[serde(default)]
    pub short_caption: Option<String>,
    /// JSON type name.
    #[serde(default)]
    pub json_type: Option<String>,
    /// SQL type name.
    #[serde(default)]
    pub sql_type: Option<String>,
    /// Whether the column is hidden.
    #[serde(default)]
    pub hidden: bool,
    /// Whether the column allows null values.
    #[serde(default)]
    pub nullable: bool,
    /// Whether the column is read-only.
    #[serde(default)]
    pub read_only: bool,
    /// Whether the column is user-editable.
    #[serde(default)]
    pub user_editable: bool,
    /// Lookup metadata for the column, when present.
    #[serde(default)]
    pub lookup: Option<QueryLookup>,
    /// Additional server-provided fields not modeled explicitly.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// View column entry in [`QueryView`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QueryViewColumn {
    /// Field key for the view column.
    pub field_key: String,
    /// Internal key for the view column.
    pub key: String,
    /// Name of the view column.
    pub name: String,
}

/// View filter entry in [`QueryView`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QueryViewFilter {
    /// Field key used by the filter.
    pub field_key: String,
    /// Filter operator.
    pub op: String,
    /// Filter value.
    pub value: String,
}

/// View sort entry in [`QueryView`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QueryViewSort {
    /// Sort direction.
    pub dir: String,
    /// Field key being sorted.
    pub field_key: String,
}

/// Import template metadata in [`QueryDetailsResponse`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QueryImportTemplate {
    /// Display label for the template.
    pub label: String,
    /// URL to download the template.
    pub url: String,
}

/// Index metadata in [`QueryDetailsResponse`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QueryIndex {
    /// Column names in this index.
    pub columns: Vec<String>,
    /// Index type string.
    #[serde(rename = "type")]
    pub type_: String,
}

/// Saved-view metadata in [`QueryDetailsResponse`].
// The LabKey server view model contains several independent flags that are
// intentionally modeled as booleans for response fidelity.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QueryView {
    /// Analytics providers for this view.
    #[serde(default)]
    pub analytics_providers: Vec<serde_json::Value>,
    /// Explicit columns in this view.
    #[serde(default)]
    pub columns: Vec<QueryViewColumn>,
    /// Container filter associated with this view.
    #[serde(default)]
    pub container_filter: Option<serde_json::Value>,
    /// Container path where this view is stored.
    #[serde(default)]
    pub container_path: Option<String>,
    /// Whether this is the default view.
    #[serde(rename = "default", default)]
    pub is_default: bool,
    /// Whether the view can be deleted.
    #[serde(default)]
    pub deletable: bool,
    /// Whether the view can be edited.
    #[serde(default)]
    pub editable: bool,
    /// Field metadata for this view.
    #[serde(default)]
    pub fields: Vec<QueryDetailsColumn>,
    /// Filters applied by this view.
    #[serde(default)]
    pub filter: Vec<QueryViewFilter>,
    /// Whether the view is hidden.
    #[serde(default)]
    pub hidden: bool,
    /// Whether the view is inherited.
    #[serde(default)]
    pub inherit: bool,
    /// View label.
    #[serde(default)]
    pub label: Option<String>,
    /// View name.
    #[serde(default)]
    pub name: Option<String>,
    /// View owner display name.
    #[serde(default)]
    pub owner: Option<String>,
    /// Whether the view can be reverted.
    #[serde(default)]
    pub revertable: bool,
    /// Whether the view can be saved.
    #[serde(default)]
    pub savable: bool,
    /// Whether this is a session view.
    #[serde(default)]
    pub session: bool,
    /// Whether the view is shared.
    #[serde(default)]
    pub shared: bool,
    /// Sort descriptors for this view.
    #[serde(default)]
    pub sort: Vec<QueryViewSort>,
    /// URL for opening data with this view.
    #[serde(default)]
    pub view_data_url: Option<String>,
}

/// Default-view block in [`QueryDetailsResponse`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QueryDefaultView {
    /// Columns included in the default view.
    #[serde(default)]
    pub columns: Vec<QueryDetailsColumn>,
}

/// Response from [`LabkeyClient::get_query_details`].
// The query-details payload includes multiple independent capability flags from
// the server; bool fields preserve those semantics directly.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QueryDetailsResponse {
    /// Whether the query is editable.
    #[serde(default)]
    pub can_edit: bool,
    /// Whether shared views are editable.
    #[serde(default)]
    pub can_edit_shared_views: bool,
    /// Columns for this query.
    #[serde(default)]
    pub columns: Vec<QueryDetailsColumn>,
    /// The query default view details.
    #[serde(default)]
    pub default_view: Option<QueryDefaultView>,
    /// Query description.
    #[serde(default)]
    pub description: Option<String>,
    /// URL for editing query definition.
    #[serde(default)]
    pub edit_definition_url: Option<String>,
    /// Import templates exposed by the query.
    #[serde(default)]
    pub import_templates: Vec<QueryImportTemplate>,
    /// Index metadata keyed by index name.
    #[serde(default)]
    pub indices: HashMap<String, QueryIndex>,
    /// Whether the query is inherited.
    #[serde(default)]
    pub is_inherited: bool,
    /// Whether metadata is overrideable.
    #[serde(default)]
    pub is_metadata_overrideable: bool,
    /// Whether the query is temporary.
    #[serde(default)]
    pub is_temporary: bool,
    /// Whether the query is user-defined.
    #[serde(default)]
    pub is_user_defined: bool,
    /// Query name.
    pub name: String,
    /// Schema name.
    pub schema_name: String,
    /// Query title.
    #[serde(default)]
    pub title: Option<String>,
    /// Title column field key.
    #[serde(default)]
    pub title_column: Option<String>,
    /// URL for loading view data.
    #[serde(default)]
    pub view_data_url: Option<String>,
    /// Saved views available for this query.
    #[serde(default)]
    pub views: Vec<QueryView>,
    /// Additional server-provided fields not modeled explicitly.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Encode a string to avoid web application firewall false positives.
///
/// `LabKey` endpoints that accept SQL or script content use this encoding to
/// prevent WAFs from rejecting legitimate content. The encoding is
/// URL-encode first, then base64-encode, then prepend a magic prefix that
/// tells the server how to decode it.
///
/// This matches the JS client's `wafEncode` function in `Utils.ts`.
///
/// WAF encoding must be applied whenever user-provided SQL is placed into a
/// POST request body field. It is not needed for SQL sent as GET query
/// parameters (those are URL-encoded by the HTTP client automatically).
/// Currently the two POST-body SQL paths are `execute_sql` and `get_data`
/// in SQL source mode. `validate_query` sends SQL as a query parameter so
/// it is excluded.
pub(crate) fn waf_encode(value: &str) -> String {
    let url_encoded = urlencoding::encode(value);
    let b64 = base64::engine::general_purpose::STANDARD.encode(url_encoded.as_bytes());
    format!("/*{{{{base64/x-www-form-urlencoded/wafText}}}}*/{b64}")
}

fn insert_option_to_string(insert_option: InsertOption) -> &'static str {
    match insert_option {
        InsertOption::Import => "IMPORT",
        InsertOption::Merge => "MERGE",
    }
}

/// Convert a value to a SQL string literal with escaped single quotes.
///
/// Empty strings return `NULL`.
#[must_use]
pub fn sql_string_literal(value: &str) -> String {
    if value.is_empty() {
        return "NULL".to_string();
    }
    format!("'{}'", value.replace('\'', "''"))
}

/// Convert a date-like value to a `LabKey` SQL date literal.
///
/// Empty strings return `NULL`.
#[must_use]
pub fn sql_date_literal(value: &str) -> String {
    if value.is_empty() {
        return "NULL".to_string();
    }
    format!("{{d {}}}", sql_string_literal(value))
}

/// Convert a date-time-like value to a `LabKey` SQL timestamp literal.
///
/// Empty strings return `NULL`.
#[must_use]
pub fn sql_date_time_literal(value: &str) -> String {
    if value.is_empty() {
        return "NULL".to_string();
    }
    format!("{{ts {}}}", sql_string_literal(value))
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
    #[serde(skip_serializing_if = "Option::is_none")]
    include_details_column: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetDataBody {
    source: GetDataBodySource,
    renderer: GetDataRenderer,
    #[serde(skip_serializing_if = "Option::is_none")]
    transforms: Option<Vec<GetDataBodyTransform>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pivot: Option<GetDataBodyPivot>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetDataBodySource {
    #[serde(rename = "type")]
    type_: String,
    schema_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sql: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetDataRenderer {
    #[serde(rename = "type")]
    type_: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    columns: Option<Vec<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_details_column: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_rows: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<Vec<GetDataBodySort>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetDataBodySort {
    field_key: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dir: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetDataBodyTransform {
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    type_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_by: Option<Vec<Vec<String>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<Vec<GetDataBodyFilter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    aggregates: Option<Vec<GetDataBodyAggregate>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetDataBodyFilter {
    field_key: Vec<String>,
    #[serde(rename = "type")]
    type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<serde_json::Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetDataBodyAggregate {
    field_key: Vec<String>,
    #[serde(rename = "type")]
    type_: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetDataBodyPivot {
    by: Vec<String>,
    columns: Vec<Vec<String>>,
}

/// Request body for the `browseData.api` endpoint.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GetDataViewsBody {
    include_data: bool,
    include_metadata: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data_types: Option<Vec<DataViewType>>,
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

/// Request body for the `saveQueryViews.api` endpoint.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveQueryViewsBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    views: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shared: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    session: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hidden: Option<bool>,
}

/// Request body for the `saveSessionView.api` endpoint.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveSessionViewBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_name: Option<String>,
    #[serde(rename = "query.queryName", skip_serializing_if = "Option::is_none")]
    query_query_name: Option<String>,
    #[serde(rename = "query.viewName", skip_serializing_if = "Option::is_none")]
    query_view_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shared: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inherit: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hidden: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replace: Option<bool>,
}

/// Request body for the `deleteView.api` endpoint.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DeleteQueryViewBody {
    schema_name: String,
    query_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    view_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    complete: Option<bool>,
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

    fn build_get_data_source(source: GetDataSource) -> Result<GetDataBodySource, LabkeyError> {
        match source {
            GetDataSource::Query {
                schema_name,
                query_name,
            } => {
                if query_name.trim().is_empty() {
                    return Err(LabkeyError::InvalidInput(
                        "get_data source.type=query requires non-empty query_name".to_string(),
                    ));
                }
                Ok(GetDataBodySource {
                    type_: "query".to_string(),
                    schema_name,
                    query_name: Some(query_name),
                    sql: None,
                })
            }
            GetDataSource::Sql { schema_name, sql } => {
                if sql.trim().is_empty() {
                    return Err(LabkeyError::InvalidInput(
                        "get_data source.type=sql requires non-empty sql".to_string(),
                    ));
                }
                Ok(GetDataBodySource {
                    type_: "sql".to_string(),
                    schema_name,
                    query_name: None,
                    sql: Some(waf_encode(&sql)),
                })
            }
        }
    }

    fn map_get_data_sorts(sorts: Option<Vec<GetDataSort>>) -> Option<Vec<GetDataBodySort>> {
        sorts.map(|items| {
            items
                .into_iter()
                .map(|sort| GetDataBodySort {
                    field_key: sort.field_key,
                    dir: sort.dir.map(|d| match d {
                        GetDataSortDirection::Asc => "ASC".to_string(),
                        GetDataSortDirection::Desc => "DESC".to_string(),
                    }),
                })
                .collect()
        })
    }

    fn map_get_data_transforms(
        transforms: Option<Vec<GetDataTransform>>,
    ) -> Option<Vec<GetDataBodyTransform>> {
        transforms.map(|items| {
            items
                .into_iter()
                .map(|transform| GetDataBodyTransform {
                    type_: transform.type_,
                    group_by: transform.group_by,
                    filters: transform.filters.map(|filters| {
                        filters
                            .into_iter()
                            .map(|filter| GetDataBodyFilter {
                                field_key: filter.field_key,
                                type_: filter.type_,
                                value: filter.value,
                            })
                            .collect()
                    }),
                    aggregates: transform.aggregates.map(|aggregates| {
                        aggregates
                            .into_iter()
                            .map(|aggregate| GetDataBodyAggregate {
                                field_key: aggregate.field_key,
                                type_: aggregate.type_,
                            })
                            .collect()
                    }),
                })
                .collect()
        })
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
        let dr = options
            .data_region_name
            .unwrap_or_else(|| "query".to_string());
        let method = options.method.unwrap_or_default();

        let mut params: Vec<(String, String)> = [
            Some(("dataRegionName".into(), dr.clone())),
            Some(("schemaName".into(), options.schema_name)),
            Some((format!("{dr}.queryName"), options.query_name)),
            Some(("apiVersion".into(), "17.1".into())),
            options
                .columns
                .map(|c| (format!("{dr}.columns"), c.join(","))),
            opt(format!("{dr}.sort"), options.sort),
            opt(format!("{dr}.viewName"), options.view_name),
            opt(format!("{dr}.selectionKey"), options.selection_key),
            opt(
                "containerFilter",
                options.container_filter.map(container_filter_to_string),
            ),
            opt("includeTotalCount", options.include_total_count),
            opt("includeMetadata", options.include_metadata),
            options
                .include_details_column
                .and_then(|v| v.then(|| ("includeDetailsColumn".into(), "true".into()))),
            options
                .include_update_column
                .and_then(|v| v.then(|| ("includeUpdateColumn".into(), "true".into()))),
            options
                .include_style
                .and_then(|v| v.then(|| ("includeStyle".into(), "true".into()))),
            options
                .ignore_filter
                .and_then(|v| v.then(|| (format!("{dr}.ignoreFilter"), "1".into()))),
        ]
        .into_iter()
        .flatten()
        .collect();

        // showRows / maxRows / offset interaction (JS SelectRows.ts:134-148):
        // When showRows is absent or Paginated, honor maxRows and offset.
        // When showRows is All/Selected/Unselected/None, send showRows directly
        // and skip maxRows/offset entirely.
        match options.show_rows {
            None | Some(ShowRows::Paginated) => {
                if let Some(offset) = options.offset {
                    params.push((format!("{dr}.offset"), offset.to_string()));
                }
                match options.max_rows {
                    Some(max) if max < 0 => {
                        params.push((format!("{dr}.showRows"), "all".into()));
                    }
                    Some(max) => {
                        params.push((format!("{dr}.maxRows"), max.to_string()));
                    }
                    None => {}
                }
            }
            Some(show) => {
                let value = match show {
                    ShowRows::All => "all",
                    ShowRows::None => "none",
                    ShowRows::Selected => "selected",
                    ShowRows::Unselected => "unselected",
                    ShowRows::Paginated => unreachable!(),
                };
                params.push((format!("{dr}.showRows"), value.into()));
            }
        }

        if let Some(filters) = &options.filter_array {
            params.extend(encode_filters(filters, &dr));
        }
        if let Some(parameters) = &options.parameters {
            for (k, v) in parameters {
                params.push((format!("{dr}.param.{k}"), v.clone()));
            }
        }

        match method {
            RequestMethod::Get => self.get(url, &params).await,
            RequestMethod::Post => self.post_form(url, &params).await,
        }
    }

    /// Select distinct values for a query column.
    ///
    /// Sends a request to `query-selectDistinct.api` and returns distinct
    /// values for one column. The request defaults to a query region prefix of
    /// `query`, matching `LabKey`'s JS client behavior.
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
    /// use labkey_rs::query::SelectDistinctOptions;
    ///
    /// let response = client
    ///     .select_distinct_rows(
    ///         SelectDistinctOptions::builder()
    ///             .schema_name("lists".to_string())
    ///             .query_name("People".to_string())
    ///             .column("Gender".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{} distinct values", response.values.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn select_distinct_rows(
        &self,
        options: SelectDistinctOptions,
    ) -> Result<SelectDistinctResponse, LabkeyError> {
        let url = self.build_url(
            "query",
            "selectDistinct.api",
            options.container_path.as_deref(),
        );
        let use_post = options.method == Some(RequestMethod::Post);
        let dr = options
            .data_region_name
            .unwrap_or_else(|| "query".to_string());

        let mut params: Vec<(String, String)> = [
            Some(("dataRegionName".into(), dr.clone())),
            Some(("schemaName".into(), options.schema_name)),
            Some((format!("{dr}.queryName"), options.query_name)),
            Some((format!("{dr}.columns"), options.column)),
            opt(format!("{dr}.sort"), options.sort),
            opt(format!("{dr}.viewName"), options.view_name),
            opt(
                "containerFilter",
                options.container_filter.map(container_filter_to_string),
            ),
            options
                .ignore_filter
                .and_then(|v| v.then(|| (format!("{dr}.ignoreFilter"), "true".into()))),
            match options.max_rows {
                Some(max) if max < 0 => Some((format!("{dr}.showRows"), "all".into())),
                Some(max) => Some(("maxRows".into(), max.to_string())),
                None => None,
            },
        ]
        .into_iter()
        .flatten()
        .collect();

        if let Some(filters) = &options.filter_array {
            params.extend(encode_filters(filters, dr.as_str()));
        }
        if let Some(parameters) = &options.parameters {
            for (k, v) in parameters {
                params.push((format!("{dr}.param.{k}"), v.clone()));
            }
        }

        if use_post {
            self.post_form(url, &params).await
        } else {
            self.get(url, &params).await
        }
    }

    /// Fetch schema/query metadata and view details.
    ///
    /// Sends a GET request to `query-getQueryDetails.api`.
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
    /// use labkey_rs::query::GetQueryDetailsOptions;
    ///
    /// let details = client
    ///     .get_query_details(
    ///         GetQueryDetailsOptions::builder()
    ///             .schema_name("lists".to_string())
    ///             .query_name("People".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Query: {}.{}", details.schema_name, details.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_query_details(
        &self,
        options: GetQueryDetailsOptions,
    ) -> Result<QueryDetailsResponse, LabkeyError> {
        let url = self.build_url(
            "query",
            "getQueryDetails.api",
            options.container_path.as_deref(),
        );

        let mut params: Vec<(String, String)> = [
            Some(("schemaName".into(), options.schema_name)),
            Some(("queryName".into(), options.query_name)),
            opt("fk", options.fk),
            opt("initializeMissingView", options.initialize_missing_view),
            opt("includeTriggers", options.include_triggers),
            opt(
                "includeSuggestedQueryColumns",
                options.include_suggested_query_columns,
            ),
        ]
        .into_iter()
        .flatten()
        .collect();

        if let Some(fields) = &options.fields {
            params.extend(fields.iter().cloned().map(|field| ("fields".into(), field)));
        }
        if let Some(view_names) = &options.view_name {
            params.extend(
                view_names
                    .iter()
                    .cloned()
                    .map(|view_name| ("viewName".into(), view_name)),
            );
        }

        self.get(url, &params).await
    }

    /// List available queries for a schema.
    ///
    /// Sends a GET request to `query-getQueries.api`.
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
    /// use labkey_rs::query::GetQueriesOptions;
    ///
    /// let response = client
    ///     .get_queries(
    ///         GetQueriesOptions::builder()
    ///             .schema_name("lists".to_string())
    ///             .include_columns(true)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Found {} queries", response.queries.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_queries(
        &self,
        options: GetQueriesOptions,
    ) -> Result<GetQueriesResponse, LabkeyError> {
        let url = self.build_url("query", "getQueries.api", options.container_path.as_deref());
        let params: Vec<(String, String)> = [
            Some(("schemaName".into(), options.schema_name)),
            opt("includeColumns", options.include_columns),
            opt("includeSystemQueries", options.include_system_queries),
            opt("includeTitle", options.include_title),
            opt("includeUserQueries", options.include_user_queries),
            opt("includeViewDataUrl", options.include_view_data_url),
            opt("queryDetailColumns", options.query_detail_columns),
        ]
        .into_iter()
        .flatten()
        .collect();

        self.get(url, &params).await
    }

    /// List schemas available in a container.
    ///
    /// Sends a GET request to `query-getSchemas.api`.
    ///
    /// Returns the raw JSON object keyed by schema name to preserve wire-level
    /// compatibility with server variations.
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
    /// use labkey_rs::query::GetSchemasOptions;
    ///
    /// let response = client
    ///     .get_schemas(
    ///         GetSchemasOptions::builder()
    ///             .include_hidden(false)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Schema keys returned: {}", response.as_object().map_or(0, |v| v.len()));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_schemas(
        &self,
        options: GetSchemasOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url("query", "getSchemas.api", options.container_path.as_deref());
        let params: Vec<(String, String)> = [
            opt("apiVersion", options.api_version),
            opt("schemaName", options.schema_name),
            opt("includeHidden", options.include_hidden),
        ]
        .into_iter()
        .flatten()
        .collect();

        self.get(url, &params).await
    }

    /// List available views for a query.
    ///
    /// Sends a GET request to `query-getQueryViews.api`.
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
    /// use labkey_rs::query::GetQueryViewsOptions;
    ///
    /// let views = client
    ///     .get_query_views(
    ///         GetQueryViewsOptions::builder()
    ///             .schema_name("lists".to_string())
    ///             .query_name("People".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("View payload keys: {}", views.as_object().map_or(0, |v| v.len()));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_query_views(
        &self,
        options: GetQueryViewsOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "query",
            "getQueryViews.api",
            options.container_path.as_deref(),
        );
        let params: Vec<(String, String)> = [
            opt("schemaName", options.schema_name),
            opt("queryName", options.query_name),
            opt("viewName", options.view_name),
            opt("metadata", options.metadata.map(|v| v.to_string())),
            options
                .exclude_session_view
                .and_then(|v| v.then(|| ("excludeSessionView".to_string(), "true".to_string()))),
        ]
        .into_iter()
        .flatten()
        .collect();

        self.get(url, &params).await
    }

    /// Create or update query views.
    ///
    /// Sends a POST request to `query-saveQueryViews.api`.
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
    /// use labkey_rs::query::SaveQueryViewsOptions;
    ///
    /// let response = client
    ///     .save_query_views(
    ///         SaveQueryViewsOptions::builder()
    ///             .schema_name("lists".to_string())
    ///             .query_name("People".to_string())
    ///             .views(serde_json::json!([{"name": "All"}]))
    ///             .shared(true)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Save payload keys: {}", response.as_object().map_or(0, |v| v.len()));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_query_views(
        &self,
        options: SaveQueryViewsOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "query",
            "saveQueryViews.api",
            options.container_path.as_deref(),
        );
        let body = SaveQueryViewsBody {
            schema_name: options.schema_name,
            query_name: options.query_name,
            metadata: options.metadata,
            views: options.views,
            shared: options.shared.and_then(|v| v.then_some(true)),
            session: options.session.and_then(|v| v.then_some(true)),
            hidden: options.hidden.and_then(|v| v.then_some(true)),
        };

        self.post(url, &body).await
    }

    /// Persist a session view as a named non-session view.
    ///
    /// Sends a POST request to `query-saveSessionView.api`.
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
    /// use labkey_rs::query::SaveSessionViewOptions;
    ///
    /// let response = client
    ///     .save_session_view(
    ///         SaveSessionViewOptions::builder()
    ///             .schema_name("lists".to_string())
    ///             .query_name("People".to_string())
    ///             .view_name("Session".to_string())
    ///             .new_name("My Saved View".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Session save response: {}", response);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_session_view(
        &self,
        options: SaveSessionViewOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "query",
            "saveSessionView.api",
            options.container_path.as_deref(),
        );
        let body = SaveSessionViewBody {
            schema_name: options.schema_name,
            query_query_name: options.query_name,
            query_view_name: options.view_name,
            new_name: options.new_name,
            shared: options.shared.and_then(|v| v.then_some(true)),
            inherit: options.inherit.and_then(|v| v.then_some(true)),
            hidden: options.hidden.and_then(|v| v.then_some(true)),
            replace: options.replace.and_then(|v| v.then_some(true)),
        };

        self.post(url, &body).await
    }

    /// Delete or revert a query view.
    ///
    /// Sends a POST request to `query-deleteView.api`.
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
    /// use labkey_rs::query::DeleteQueryViewOptions;
    ///
    /// let response = client
    ///     .delete_query_view(
    ///         DeleteQueryViewOptions::builder()
    ///             .schema_name("lists".to_string())
    ///             .query_name("People".to_string())
    ///             .view_name("My Saved View".to_string())
    ///             .revert(false)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Delete response: {}", response);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_query_view(
        &self,
        options: DeleteQueryViewOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url("query", "deleteView.api", options.container_path.as_deref());
        let body = DeleteQueryViewBody {
            schema_name: options.schema_name,
            query_name: options.query_name,
            view_name: options.view_name,
            complete: options.revert.map(|revert| !revert),
        };

        self.post(url, &body).await
    }

    /// List report/query/dataset data views and return the inner `data` payload.
    ///
    /// Sends a POST request to `reports-browseData.api` with `LabKey`'s required
    /// defaults `includeData: true` and `includeMetadata: false`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the HTTP request fails, the server returns
    /// an error response, the response body cannot be deserialized, or the
    /// expected `data` envelope field is missing.
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
    /// use labkey_rs::query::{DataViewType, GetDataViewsOptions};
    ///
    /// let data = client
    ///     .get_data_views(
    ///         GetDataViewsOptions::builder()
    ///             .data_types(vec![DataViewType::Queries, DataViewType::Reports])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Top-level keys: {}", data.as_object().map_or(0, |v| v.len()));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_data_views(
        &self,
        options: GetDataViewsOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "reports",
            "browseData.api",
            options.container_path.as_deref(),
        );
        let body = GetDataViewsBody {
            include_data: true,
            include_metadata: false,
            data_types: options.data_types,
        };
        let response: serde_json::Value = self.post(url, &body).await?;

        let data =
            response
                .get("data")
                .cloned()
                .ok_or_else(|| LabkeyError::UnexpectedResponse {
                    status: StatusCode::OK,
                    text: format!("missing `data` field in browseData response: {response}"),
                })?;

        if data.is_null() || !(data.is_object() || data.is_array()) {
            return Err(LabkeyError::UnexpectedResponse {
                status: StatusCode::OK,
                text: format!("invalid `data` field in browseData response: {response}"),
            });
        }

        Ok(data)
    }

    /// Validate a query payload against server-side parsing and execution rules.
    ///
    /// Sends a GET request to `query-validateQuery.api` by default. When
    /// `validate_query_metadata` is true, the method targets
    /// `query-validateQueryMetadata.api`.
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
    /// use labkey_rs::query::ValidateQueryOptions;
    ///
    /// let result = client
    ///     .validate_query(
    ///         ValidateQueryOptions::builder()
    ///             .schema_name("lists".to_string())
    ///             .query_name("People".to_string())
    ///             .validate_query_metadata(true)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Valid: {}", result.valid);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate_query(
        &self,
        options: ValidateQueryOptions,
    ) -> Result<ValidateQueryResponse, LabkeyError> {
        let action = if options.validate_query_metadata.unwrap_or(false) {
            "validateQueryMetadata.api"
        } else {
            "validateQuery.api"
        };

        let url = self.build_url("query", action, options.container_path.as_deref());
        let params: Vec<(String, String)> = [
            opt("schemaName", options.schema_name),
            opt("queryName", options.query_name),
            opt("sql", options.sql),
            opt("viewName", options.view_name),
        ]
        .into_iter()
        .flatten()
        .collect();

        self.get(url, &params).await
    }

    /// Return the current date/time from the `LabKey` server.
    ///
    /// Sends a GET request to `query-getServerDate.api` with no query
    /// parameters and no container path segment.
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
    /// let server_date = client.get_server_date().await?;
    /// println!("Server date: {}", server_date.date);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_server_date(&self) -> Result<GetServerDateResponse, LabkeyError> {
        let url = self.build_url("query", "getServerDate.api", Some(""));
        self.get(url, &[]).await
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
    /// if let Some(count) = result.deleted_rows {
    ///     println!("Deleted {count} rows");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn truncate_table(
        &self,
        options: TruncateTableOptions,
    ) -> Result<TruncateTableResponse, LabkeyError> {
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

        let url = self.build_url(
            "query",
            "truncateTable.api",
            options.container_path.as_deref(),
        );
        self.post(url, &body).await
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

    /// Import query data from text, uploaded file bytes, server path, or module resource.
    ///
    /// Sends a multipart POST request to `query-import.api` with required
    /// `schemaName` and `queryName` parts plus source-specific multipart parts.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the HTTP request fails, the server returns
    /// an error response, the response body cannot be deserialized, or a
    /// provided file MIME type is invalid.
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
    /// use labkey_rs::query::{ImportDataOptions, ImportDataSource, InsertOption};
    ///
    /// let response = client
    ///     .import_data(
    ///         ImportDataOptions::builder()
    ///             .schema_name("lists".to_string())
    ///             .query_name("People".to_string())
    ///             .source(ImportDataSource::Text("Name,Age\nAlice,30".to_string()))
    ///             .format("csv".to_string())
    ///             .insert_option(InsertOption::Import)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Import success: {}", response.success);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn import_data(
        &self,
        options: ImportDataOptions,
    ) -> Result<ImportDataResponse, LabkeyError> {
        let url = self.build_url("query", "import.api", options.container_path.as_deref());
        let mut form = reqwest::multipart::Form::new()
            .text("schemaName", options.schema_name)
            .text("queryName", options.query_name);

        match options.source {
            ImportDataSource::Text(text) => {
                form = form.text("text", text);
            }
            ImportDataSource::File {
                file_name,
                bytes,
                mime_type,
            } => {
                let part = reqwest::multipart::Part::bytes(bytes).file_name(file_name);
                let part = if let Some(mime_type) = mime_type {
                    part.mime_str(&mime_type).map_err(|error| {
                        LabkeyError::InvalidInput(format!(
                            "invalid MIME type for import data file part: {error}"
                        ))
                    })?
                } else {
                    part
                };
                form = form.part("file", part);
            }
            ImportDataSource::Path(path) => {
                form = form.text("path", path);
            }
            ImportDataSource::ModuleResource {
                module,
                module_resource,
            } => {
                form = form
                    .text("module", module)
                    .text("moduleResource", module_resource);
            }
        }

        if let Some(format) = options.format {
            form = form.text("format", format);
        }
        if let Some(insert_option) = options.insert_option {
            form = form.text(
                "insertOption",
                insert_option_to_string(insert_option).to_string(),
            );
        }
        if let Some(use_async) = options.use_async {
            form = form.text("useAsync", use_async.to_string());
        }
        if let Some(save_to_pipeline) = options.save_to_pipeline {
            form = form.text("saveToPipeline", save_to_pipeline.to_string());
        }
        if let Some(import_identity) = options.import_identity {
            form = form.text("importIdentity", import_identity.to_string());
        }
        if let Some(import_lookup_by_alternate_key) = options.import_lookup_by_alternate_key {
            form = form.text(
                "importLookupByAlternateKey",
                import_lookup_by_alternate_key.to_string(),
            );
        }
        if let Some(audit_user_comment) = options.audit_user_comment {
            form = form.text("auditUserComment", audit_user_comment);
        }
        if let Some(audit_details) = options.audit_details {
            form = form.text("auditDetails", audit_details.to_string());
        }

        self.post_multipart(url, form, &crate::client::RequestOptions::default())
            .await
    }

    /// Execute a typed `getData` request and return JSON-rendered rows.
    ///
    /// Sends a POST request to `query-getData` (no `.api` suffix) and always
    /// sets `renderer.type` to `"json"`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, the response body cannot be deserialized, or required
    /// source fields are missing for the selected source type.
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
    /// use labkey_rs::query::{GetDataOptions, GetDataSource};
    ///
    /// let response = client
    ///     .get_data(
    ///         GetDataOptions::builder()
    ///             .source(GetDataSource::Query {
    ///                 schema_name: "lists".to_string(),
    ///                 query_name: "People".to_string(),
    ///             })
    ///             .max_rows(250)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("getData row count: {}", response.row_count);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_data(
        &self,
        options: GetDataOptions,
    ) -> Result<SelectRowsResponse, LabkeyError> {
        let body_source = Self::build_get_data_source(options.source)?;

        let body = GetDataBody {
            source: body_source,
            renderer: GetDataRenderer {
                type_: "json",
                columns: options.columns,
                include_details_column: options.include_details_column,
                max_rows: options.max_rows,
                offset: options.offset,
                sort: Self::map_get_data_sorts(options.sort),
            },
            transforms: Self::map_get_data_transforms(options.transforms),
            pivot: options.pivot.map(|pivot| GetDataBodyPivot {
                by: pivot.by,
                columns: pivot.columns,
            }),
        };

        let url = self.build_url("query", "getData", options.container_path.as_deref());
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

        // JS ExecuteSql.ts:111-115 omits maxRows when negative and offset
        // when zero or falsy, so we mirror that to avoid sending sentinel
        // values that change server behavior.
        let max_rows = options.max_rows.filter(|&m| m >= 0);
        let offset = options.offset.filter(|&o| o > 0);

        let body = ExecuteSqlBody {
            schema_name: options.schema_name,
            sql: waf_encode(&options.sql),
            api_version: 17.1,
            max_rows,
            offset,
            container_filter: options.container_filter,
            include_total_count: options.include_total_count,
            include_metadata: options.include_metadata,
            save_in_session: options.save_in_session,
            include_style: options.include_style,
            include_details_column: options.include_details_column,
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
    fn select_distinct_response_deserializes() {
        let json = serde_json::json!({
            "queryName": "People",
            "schemaName": "lists",
            "values": ["F", "M", null]
        });

        let response: SelectDistinctResponse =
            serde_json::from_value(json).expect("should deserialize");
        assert_eq!(response.query_name, "People");
        assert_eq!(response.schema_name, "lists");
        assert_eq!(response.values.len(), 3);
    }

    #[test]
    fn query_details_column_deserializes_optional_fields_present() {
        let json = serde_json::json!({
            "name": "Project",
            "fieldKey": "Project",
            "caption": "Project",
            "shortCaption": "Project",
            "jsonType": "string",
            "sqlType": "VARCHAR",
            "nullable": false,
            "readOnly": true,
            "lookup": {
                "schemaName": "core",
                "queryName": "Projects",
                "keyColumn": "RowId"
            },
            "customProp": "extra"
        });

        let column: QueryDetailsColumn = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(column.name, "Project");
        assert_eq!(column.field_key, serde_json::json!("Project"));
        assert_eq!(column.caption.as_deref(), Some("Project"));
        assert!(column.read_only);
        assert_eq!(
            column
                .lookup
                .as_ref()
                .and_then(|lookup| lookup.schema_name.as_deref()),
            Some("core")
        );
        assert_eq!(
            column.extra.get("customProp"),
            Some(&serde_json::Value::String("extra".to_string()))
        );
    }

    #[test]
    fn query_details_column_deserializes_optional_fields_absent() {
        let json = serde_json::json!({
            "name": "RowId",
            "fieldKey": "RowId"
        });

        let column: QueryDetailsColumn = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(column.name, "RowId");
        assert_eq!(column.field_key, serde_json::json!("RowId"));
        assert!(column.caption.is_none());
        assert!(column.short_caption.is_none());
        assert!(column.lookup.is_none());
        assert!(column.extra.is_empty());
    }

    #[test]
    fn query_details_response_deserializes_fixture() {
        let fixture = include_str!("../tests/fixtures/query_details.json");
        let response: QueryDetailsResponse =
            serde_json::from_str(fixture).expect("fixture should deserialize");

        assert_eq!(response.schema_name, "lists");
        assert_eq!(response.name, "People");
        assert_eq!(response.columns.len(), 2);
        assert_eq!(response.import_templates.len(), 1);
        assert_eq!(response.indices.len(), 1);
        assert_eq!(response.views.len(), 1);
        assert_eq!(
            response
                .columns
                .first()
                .and_then(|column| column.extra.get("facetingBehaviorType")),
            Some(&serde_json::Value::String("automatic".to_string()))
        );
        assert_eq!(
            response.extra.get("moduleName"),
            Some(&serde_json::Value::String("lists".to_string()))
        );
    }

    #[test]
    fn query_details_response_collects_unknown_top_level_fields_in_extra() {
        let json = serde_json::json!({
            "schemaName": "lists",
            "name": "People",
            "customMetadata": {"enabled": true}
        });

        let response: QueryDetailsResponse =
            serde_json::from_value(json).expect("should deserialize");
        assert_eq!(
            response.extra.get("customMetadata"),
            Some(&serde_json::json!({"enabled": true}))
        );
    }

    #[test]
    fn get_queries_response_deserializes_with_nested_queries() {
        let json = serde_json::json!({
            "schemaName": "lists",
            "queries": [
                {
                    "name": "People",
                    "title": "People List",
                    "columns": [{"name": "RowId", "caption": "Row Id"}],
                    "canEdit": true,
                    "canEditSharedViews": false,
                    "hidden": false,
                    "inherit": true,
                    "isInherited": false,
                    "isMetadataOverrideable": true,
                    "isUserDefined": true,
                    "snapshot": false,
                    "viewDataUrl": "/list-grid.view?name=People"
                }
            ]
        });

        let response: GetQueriesResponse =
            serde_json::from_value(json).expect("should deserialize");
        assert_eq!(response.schema_name, "lists");
        assert_eq!(response.queries.len(), 1);
        assert_eq!(response.queries[0].name, "People");
        assert_eq!(response.queries[0].columns.len(), 1);
        assert_eq!(response.queries[0].title.as_deref(), Some("People List"));
    }

    #[test]
    fn get_schemas_fixture_deserializes_to_keyed_object() {
        let fixture = include_str!("../tests/fixtures/get_schemas.json");
        let value: serde_json::Value =
            serde_json::from_str(fixture).expect("fixture should deserialize");

        let schemas = value
            .as_object()
            .expect("getSchemas payload should be object keyed by schema name");
        assert!(schemas.contains_key("core"));
        assert!(schemas.contains_key("lists"));
    }

    #[test]
    fn query_endpoint_urls_match_expected_actions() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url("query", "selectDistinct.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-selectDistinct.api"
        );
        assert_eq!(
            client
                .build_url("query", "getQueryDetails.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-getQueryDetails.api"
        );
        assert_eq!(
            client
                .build_url("query", "getQueries.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-getQueries.api"
        );
        assert_eq!(
            client
                .build_url("query", "getSchemas.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-getSchemas.api"
        );
        assert_eq!(
            client
                .build_url("query", "getQueryViews.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-getQueryViews.api"
        );
        assert_eq!(
            client
                .build_url("query", "saveQueryViews.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-saveQueryViews.api"
        );
        assert_eq!(
            client
                .build_url("query", "saveSessionView.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-saveSessionView.api"
        );
        assert_eq!(
            client
                .build_url("query", "deleteView.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-deleteView.api"
        );

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
        assert_eq!(
            client
                .build_url("query", "import.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-import.api"
        );
    }

    #[test]
    fn query_misc_endpoint_urls_match_expected_actions() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url("reports", "browseData.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/reports-browseData.api"
        );
        assert_eq!(
            client
                .build_url("query", "validateQuery.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-validateQuery.api"
        );
        assert_eq!(
            client
                .build_url("query", "validateQueryMetadata.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-validateQueryMetadata.api"
        );
        assert_eq!(
            client
                .build_url("query", "getServerDate.api", Some(""))
                .as_str(),
            "https://labkey.example.com/labkey/query-getServerDate.api"
        );
        assert_eq!(
            client
                .build_url("query", "getData", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/query-getData"
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
    fn data_view_type_serializes_exact_wire_values() {
        assert_eq!(
            serde_json::to_string(&DataViewType::Datasets).expect("should serialize"),
            "\"datasets\""
        );
        assert_eq!(
            serde_json::to_string(&DataViewType::Queries).expect("should serialize"),
            "\"queries\""
        );
        assert_eq!(
            serde_json::to_string(&DataViewType::Reports).expect("should serialize"),
            "\"reports\""
        );
    }

    #[test]
    fn data_view_type_deserializes_exact_wire_values() {
        let datasets: DataViewType =
            serde_json::from_str("\"datasets\"").expect("should deserialize");
        let queries: DataViewType =
            serde_json::from_str("\"queries\"").expect("should deserialize");
        let reports: DataViewType =
            serde_json::from_str("\"reports\"").expect("should deserialize");

        assert_eq!(datasets, DataViewType::Datasets);
        assert_eq!(queries, DataViewType::Queries);
        assert_eq!(reports, DataViewType::Reports);
    }

    #[test]
    fn data_view_type_rejects_unknown_wire_value() {
        let err = serde_json::from_str::<DataViewType>("\"charts\"")
            .expect_err("unknown data view type should fail to deserialize");
        assert!(err.is_data());
    }

    fn data_view_type_variant_count(value: DataViewType) -> usize {
        match value {
            DataViewType::Datasets | DataViewType::Queries | DataViewType::Reports => 3,
        }
    }

    #[test]
    fn data_view_type_variant_count_regression() {
        assert_eq!(data_view_type_variant_count(DataViewType::Datasets), 3);
    }

    #[test]
    fn insert_option_serializes_exact_wire_values() {
        assert_eq!(
            serde_json::to_string(&InsertOption::Import).expect("should serialize"),
            "\"IMPORT\""
        );
        assert_eq!(
            serde_json::to_string(&InsertOption::Merge).expect("should serialize"),
            "\"MERGE\""
        );
    }

    #[test]
    fn insert_option_deserializes_exact_wire_values() {
        let import: InsertOption = serde_json::from_str("\"IMPORT\"").expect("should deserialize");
        let merge: InsertOption = serde_json::from_str("\"MERGE\"").expect("should deserialize");

        assert_eq!(import, InsertOption::Import);
        assert_eq!(merge, InsertOption::Merge);
    }

    #[test]
    fn insert_option_rejects_unknown_wire_value() {
        let err = serde_json::from_str::<InsertOption>("\"UPSERT\"")
            .expect_err("unknown insert option should fail to deserialize");
        assert!(err.is_data());
    }

    fn insert_option_variant_count(value: InsertOption) -> usize {
        match value {
            InsertOption::Import | InsertOption::Merge => 2,
        }
    }

    #[test]
    fn insert_option_variant_count_regression() {
        assert_eq!(insert_option_variant_count(InsertOption::Import), 2);
    }

    #[test]
    fn get_data_source_type_variant_count_regression() {
        let count = match GetDataSourceType::Query {
            GetDataSourceType::Query | GetDataSourceType::Sql => 2,
        };
        assert_eq!(count, 2);
    }

    #[test]
    fn get_data_sort_direction_variant_count_regression() {
        let count = match GetDataSortDirection::Asc {
            GetDataSortDirection::Asc | GetDataSortDirection::Desc => 2,
        };
        assert_eq!(count, 2);
    }

    #[test]
    fn import_data_response_deserializes_with_job_id() {
        let json = serde_json::json!({
            "success": true,
            "rowCount": 4,
            "jobId": "job-123"
        });

        let response: ImportDataResponse =
            serde_json::from_value(json).expect("should deserialize");
        assert!(response.success);
        assert_eq!(response.row_count, Some(4));
        assert_eq!(response.job_id.as_deref(), Some("job-123"));
    }

    #[test]
    fn import_data_response_deserializes_without_job_id() {
        let json = serde_json::json!({
            "success": true,
            "rowCount": 2
        });

        let response: ImportDataResponse =
            serde_json::from_value(json).expect("should deserialize");
        assert!(response.success);
        assert_eq!(response.row_count, Some(2));
        assert!(response.job_id.is_none());
    }

    #[tokio::test]
    async fn import_data_rejects_invalid_file_mime_type() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        let result = client
            .import_data(
                ImportDataOptions::builder()
                    .schema_name("lists".to_string())
                    .query_name("People".to_string())
                    .source(ImportDataSource::File {
                        file_name: "rows.csv".to_string(),
                        bytes: b"Name,Age\nAlice,30".to_vec(),
                        mime_type: Some("not/a valid mime type".to_string()),
                    })
                    .build(),
            )
            .await;

        match result {
            Err(LabkeyError::InvalidInput(message)) => {
                assert!(message.contains("invalid MIME type for import data file part"));
            }
            other => panic!("expected LabkeyError::InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn get_data_views_body_serializes_required_flags_and_optional_data_types() {
        let body = GetDataViewsBody {
            include_data: true,
            include_metadata: false,
            data_types: Some(vec![DataViewType::Queries, DataViewType::Reports]),
        };

        let value = serde_json::to_value(body).expect("should serialize");
        let obj = value.as_object().expect("body should be object");

        assert_eq!(obj.get("includeData"), Some(&serde_json::json!(true)));
        assert_eq!(obj.get("includeMetadata"), Some(&serde_json::json!(false)));
        assert_eq!(
            obj.get("dataTypes"),
            Some(&serde_json::json!(["queries", "reports"]))
        );
    }

    #[test]
    fn get_data_views_body_omits_data_types_when_absent() {
        let body = GetDataViewsBody {
            include_data: true,
            include_metadata: false,
            data_types: None,
        };

        let value = serde_json::to_value(body).expect("should serialize");
        let obj = value.as_object().expect("body should be object");

        assert_eq!(obj.get("includeData"), Some(&serde_json::json!(true)));
        assert_eq!(obj.get("includeMetadata"), Some(&serde_json::json!(false)));
        assert!(!obj.contains_key("dataTypes"));
    }

    #[test]
    fn get_data_body_serializes_query_source_and_optional_sections() {
        let body = GetDataBody {
            source: GetDataBodySource {
                type_: "query".to_string(),
                schema_name: "lists".to_string(),
                query_name: Some("People".to_string()),
                sql: None,
            },
            renderer: GetDataRenderer {
                type_: "json",
                columns: Some(vec![vec!["Name".to_string()]]),
                include_details_column: Some(true),
                max_rows: Some(10),
                offset: Some(5),
                sort: Some(vec![GetDataBodySort {
                    field_key: vec!["Name".to_string()],
                    dir: Some("ASC".to_string()),
                }]),
            },
            transforms: Some(vec![GetDataBodyTransform {
                type_: Some("aggregate".to_string()),
                group_by: Some(vec![vec!["Department".to_string()]]),
                filters: Some(vec![GetDataBodyFilter {
                    field_key: vec!["Status".to_string()],
                    type_: "eq".to_string(),
                    value: Some(serde_json::json!("Active")),
                }]),
                aggregates: Some(vec![GetDataBodyAggregate {
                    field_key: vec!["Amount".to_string()],
                    type_: "sum".to_string(),
                }]),
            }]),
            pivot: Some(GetDataBodyPivot {
                by: vec!["Department".to_string()],
                columns: vec![vec!["Amount".to_string()]],
            }),
        };

        let value = serde_json::to_value(body).expect("should serialize get_data body");
        let obj = value.as_object().expect("body should be object");

        assert_eq!(obj["source"]["type"], serde_json::json!("query"));
        assert_eq!(obj["source"]["schemaName"], serde_json::json!("lists"));
        assert_eq!(obj["source"]["queryName"], serde_json::json!("People"));
        assert!(obj["source"].get("sql").is_none());
        assert_eq!(obj["renderer"]["type"], serde_json::json!("json"));
        assert_eq!(obj["renderer"]["columns"], serde_json::json!([["Name"]]));
        assert_eq!(
            obj["renderer"]["includeDetailsColumn"],
            serde_json::json!(true)
        );
        assert_eq!(obj["renderer"]["maxRows"], serde_json::json!(10));
        assert_eq!(obj["renderer"]["offset"], serde_json::json!(5));
        assert_eq!(
            obj["renderer"]["sort"][0]["fieldKey"],
            serde_json::json!(["Name"])
        );
        assert_eq!(obj["renderer"]["sort"][0]["dir"], serde_json::json!("ASC"));
        assert_eq!(obj["transforms"][0]["type"], serde_json::json!("aggregate"));
        assert_eq!(
            obj["transforms"][0]["filters"][0]["fieldKey"],
            serde_json::json!(["Status"])
        );
        assert_eq!(
            obj["transforms"][0]["filters"][0]["type"],
            serde_json::json!("eq")
        );
        assert_eq!(
            obj["transforms"][0]["aggregates"][0]["fieldKey"],
            serde_json::json!(["Amount"])
        );
        assert_eq!(
            obj["transforms"][0]["aggregates"][0]["type"],
            serde_json::json!("sum")
        );
        assert_eq!(obj["pivot"]["by"], serde_json::json!(["Department"]));
        assert_eq!(obj["pivot"]["columns"], serde_json::json!([["Amount"]]));
    }

    #[test]
    fn get_data_body_omits_absent_optional_fields() {
        let body = GetDataBody {
            source: GetDataBodySource {
                type_: "sql".to_string(),
                schema_name: "core".to_string(),
                query_name: None,
                sql: Some(waf_encode("SELECT * FROM core.Users")),
            },
            renderer: GetDataRenderer {
                type_: "json",
                columns: None,
                include_details_column: None,
                max_rows: None,
                offset: None,
                sort: None,
            },
            transforms: None,
            pivot: None,
        };

        let value = serde_json::to_value(body).expect("should serialize get_data body");
        let obj = value.as_object().expect("body should be object");
        let source = obj
            .get("source")
            .and_then(serde_json::Value::as_object)
            .expect("source should be object");
        let renderer = obj
            .get("renderer")
            .and_then(serde_json::Value::as_object)
            .expect("renderer should be object");

        assert_eq!(source.get("type"), Some(&serde_json::json!("sql")));
        assert_eq!(source.get("schemaName"), Some(&serde_json::json!("core")));
        assert!(source.get("queryName").is_none());
        assert!(source.get("sql").is_some());
        assert_eq!(renderer.get("type"), Some(&serde_json::json!("json")));
        assert!(!renderer.contains_key("columns"));
        assert!(!renderer.contains_key("includeDetailsColumn"));
        assert!(!renderer.contains_key("maxRows"));
        assert!(!renderer.contains_key("offset"));
        assert!(!renderer.contains_key("sort"));
        assert!(!obj.contains_key("transforms"));
        assert!(!obj.contains_key("pivot"));
    }

    #[test]
    fn get_data_source_reports_expected_source_type() {
        assert_eq!(
            GetDataSource::Query {
                schema_name: "lists".to_string(),
                query_name: "People".to_string(),
            }
            .source_type(),
            GetDataSourceType::Query
        );
        assert_eq!(
            GetDataSource::Sql {
                schema_name: "core".to_string(),
                sql: "SELECT * FROM core.Users".to_string(),
            }
            .source_type(),
            GetDataSourceType::Sql
        );
    }

    #[tokio::test]
    async fn get_data_query_source_with_empty_query_name_returns_invalid_input() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");
        let result = client
            .get_data(
                GetDataOptions::builder()
                    .source(GetDataSource::Query {
                        schema_name: "lists".to_string(),
                        query_name: "   ".to_string(),
                    })
                    .build(),
            )
            .await;

        match result {
            Err(LabkeyError::InvalidInput(message)) => {
                assert_eq!(
                    message,
                    "get_data source.type=query requires non-empty query_name"
                );
            }
            other => panic!("expected LabkeyError::InvalidInput, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn get_data_sql_source_with_empty_sql_returns_invalid_input() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");
        let result = client
            .get_data(
                GetDataOptions::builder()
                    .source(GetDataSource::Sql {
                        schema_name: "core".to_string(),
                        sql: "  ".to_string(),
                    })
                    .build(),
            )
            .await;

        match result {
            Err(LabkeyError::InvalidInput(message)) => {
                assert_eq!(message, "get_data source.type=sql requires non-empty sql");
            }
            other => panic!("expected LabkeyError::InvalidInput, got {other:?}"),
        }
    }

    #[test]
    fn validate_query_response_deserializes_with_extra_fields() {
        let json = serde_json::json!({
            "valid": true,
            "queryName": "People",
            "schemaName": "lists"
        });

        let response: ValidateQueryResponse =
            serde_json::from_value(json).expect("should deserialize");
        assert!(response.valid);
        assert_eq!(
            response.extra.get("queryName"),
            Some(&serde_json::json!("People"))
        );
    }

    #[test]
    fn get_server_date_response_deserializes() {
        let json = serde_json::json!({"date": "2026-03-04T17:34:00.000Z"});
        let response: GetServerDateResponse =
            serde_json::from_value(json).expect("should deserialize");
        assert_eq!(response.date, "2026-03-04T17:34:00.000Z");
    }

    #[test]
    fn sql_literal_helpers_escape_single_quotes_and_format_wrappers() {
        assert_eq!(sql_string_literal("O'Brien"), "'O''Brien'");
        assert_eq!(sql_date_literal("2026-03-04"), "{d '2026-03-04'}");
        assert_eq!(
            sql_date_time_literal("2026-03-04 17:35:30"),
            "{ts '2026-03-04 17:35:30'}"
        );
    }

    #[test]
    fn sql_literal_helpers_return_null_for_empty_input() {
        assert_eq!(sql_string_literal(""), "NULL");
        assert_eq!(sql_date_literal(""), "NULL");
        assert_eq!(sql_date_time_literal(""), "NULL");
    }

    #[test]
    fn url_column_prefix_regression() {
        assert_eq!(URL_COLUMN_PREFIX, "_labkeyurl_");
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

    #[test]
    fn save_session_view_body_uses_flat_query_dot_keys() {
        let body = SaveSessionViewBody {
            schema_name: Some("lists".to_string()),
            query_query_name: Some("People".to_string()),
            query_view_name: Some("Session".to_string()),
            new_name: Some("Saved".to_string()),
            shared: Some(true),
            inherit: None,
            hidden: None,
            replace: None,
        };

        let value = serde_json::to_value(body).expect("should serialize save session body");
        let obj = value.as_object().expect("body should be object");

        assert_eq!(obj.get("schemaName"), Some(&serde_json::json!("lists")));
        assert_eq!(
            obj.get("query.queryName"),
            Some(&serde_json::json!("People"))
        );
        assert_eq!(
            obj.get("query.viewName"),
            Some(&serde_json::json!("Session"))
        );
        assert_eq!(obj.get("newName"), Some(&serde_json::json!("Saved")));
        assert_eq!(obj.get("shared"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn delete_query_view_body_complete_only_emits_when_revert_is_set() {
        let body_without_revert = DeleteQueryViewBody {
            schema_name: "lists".to_string(),
            query_name: "People".to_string(),
            view_name: Some("MyView".to_string()),
            complete: None,
        };
        let without_revert =
            serde_json::to_value(body_without_revert).expect("should serialize without revert");
        let without_revert_obj = without_revert.as_object().expect("body should be object");
        assert!(!without_revert_obj.contains_key("complete"));

        let body_revert_true = DeleteQueryViewBody {
            schema_name: "lists".to_string(),
            query_name: "People".to_string(),
            view_name: Some("MyView".to_string()),
            complete: Some(false),
        };
        let revert_true =
            serde_json::to_value(body_revert_true).expect("should serialize revert true");
        assert_eq!(revert_true["complete"], serde_json::json!(false));

        let body_revert_false = DeleteQueryViewBody {
            schema_name: "lists".to_string(),
            query_name: "People".to_string(),
            view_name: Some("MyView".to_string()),
            complete: Some(true),
        };
        let revert_false =
            serde_json::to_value(body_revert_false).expect("should serialize revert false");
        assert_eq!(revert_false["complete"], serde_json::json!(true));
    }

    #[test]
    fn save_query_views_body_omits_false_boolean_flags() {
        let body = SaveQueryViewsBody {
            schema_name: Some("lists".to_string()),
            query_name: Some("People".to_string()),
            metadata: None,
            views: Some(serde_json::json!([{"name": "All"}])),
            shared: None,
            session: Some(true),
            hidden: None,
        };

        let value = serde_json::to_value(body).expect("should serialize save query views body");
        let obj = value.as_object().expect("body should be object");

        assert_eq!(obj.get("schemaName"), Some(&serde_json::json!("lists")));
        assert_eq!(obj.get("queryName"), Some(&serde_json::json!("People")));
        assert_eq!(obj.get("session"), Some(&serde_json::json!(true)));
        assert!(!obj.contains_key("shared"));
        assert!(!obj.contains_key("hidden"));
    }

    #[test]
    fn save_query_views_body_emits_true_flags_and_metadata() {
        let body = SaveQueryViewsBody {
            schema_name: Some("lists".to_string()),
            query_name: Some("People".to_string()),
            metadata: Some(serde_json::json!({"scope": "grid"})),
            views: Some(serde_json::json!([{"name": "All"}])),
            shared: Some(true),
            session: None,
            hidden: Some(true),
        };

        let value = serde_json::to_value(body).expect("should serialize save query views body");
        let obj = value.as_object().expect("body should be object");

        assert_eq!(
            obj.get("metadata"),
            Some(&serde_json::json!({"scope": "grid"}))
        );
        assert_eq!(obj.get("shared"), Some(&serde_json::json!(true)));
        assert_eq!(obj.get("hidden"), Some(&serde_json::json!(true)));
        assert!(!obj.contains_key("session"));
    }

    #[test]
    fn execute_sql_body_omits_negative_max_rows_and_zero_offset() {
        let body = ExecuteSqlBody {
            schema_name: "core".to_string(),
            sql: "SELECT 1".to_string(),
            api_version: 17.1,
            max_rows: Some(-1),
            offset: Some(0),
            container_filter: None,
            include_total_count: None,
            include_metadata: None,
            save_in_session: None,
            include_style: None,
            include_details_column: None,
        };

        // Negative max_rows and zero offset should still serialize here because
        // the filtering happens in execute_sql before body construction, not in
        // the body itself. This test documents the body struct's behavior.
        let value = serde_json::to_value(&body).expect("should serialize");
        assert_eq!(value["maxRows"], -1);
        assert_eq!(value["offset"], 0);
    }

    #[test]
    fn execute_sql_body_includes_details_column_when_set() {
        let body = ExecuteSqlBody {
            schema_name: "core".to_string(),
            sql: "SELECT 1".to_string(),
            api_version: 17.1,
            max_rows: None,
            offset: None,
            container_filter: None,
            include_total_count: None,
            include_metadata: None,
            save_in_session: None,
            include_style: None,
            include_details_column: Some(true),
        };

        let value = serde_json::to_value(&body).expect("should serialize");
        assert_eq!(value["includeDetailsColumn"], true);
    }

    #[test]
    fn import_data_response_deserializes_without_row_count() {
        let json = serde_json::json!({
            "success": true
        });

        let response: ImportDataResponse =
            serde_json::from_value(json).expect("should deserialize without rowCount");
        assert!(response.success);
        assert_eq!(response.row_count, None);
    }

    #[test]
    fn truncate_table_response_deserializes_java_wire_format() {
        let json = serde_json::json!({
            "deletedRows": 42,
            "schemaName": "lists",
            "queryName": "People",
            "command": "truncate"
        });

        let response: TruncateTableResponse =
            serde_json::from_value(json).expect("should deserialize");
        assert_eq!(response.deleted_rows, Some(42));
        assert_eq!(response.schema_name.as_deref(), Some("lists"));
        assert_eq!(response.query_name.as_deref(), Some("People"));
        assert_eq!(response.command.as_deref(), Some("truncate"));
    }

    #[test]
    fn truncate_table_response_deserializes_minimal() {
        let json = serde_json::json!({});

        let response: TruncateTableResponse =
            serde_json::from_value(json).expect("should deserialize minimal");
        assert_eq!(response.deleted_rows, None);
        assert_eq!(response.schema_name, None);
    }

    #[test]
    fn request_method_defaults_to_get() {
        assert_eq!(RequestMethod::default(), RequestMethod::Get);
    }

    #[test]
    fn show_rows_variants_are_distinct() {
        let variants = [
            ShowRows::All,
            ShowRows::None,
            ShowRows::Paginated,
            ShowRows::Selected,
            ShowRows::Unselected,
        ];
        for (i, a) in variants.iter().enumerate() {
            for (j, b) in variants.iter().enumerate() {
                assert_eq!(i == j, a == b, "{a:?} vs {b:?}");
            }
        }
    }

    // -- US-036 tests --

    #[test]
    fn query_column_deserializes_is_hidden_alias() {
        let json = serde_json::json!({
            "name": "Secret",
            "fieldKey": "Secret",
            "isHidden": true
        });
        let col: QueryColumn = serde_json::from_value(json).expect("should deserialize");
        assert!(col.hidden, "isHidden alias should map to hidden");
    }

    #[test]
    fn query_column_deserializes_all_is_aliases() {
        let json = serde_json::json!({
            "name": "Col",
            "fieldKey": "Col",
            "isHidden": true,
            "isNullable": true,
            "isReadOnly": true,
            "isUserEditable": true,
            "isAutoIncrement": true,
            "isKeyField": true,
            "isMvEnabled": true,
            "isSelectable": true,
            "isVersionField": true
        });
        let col: QueryColumn = serde_json::from_value(json).expect("should deserialize");
        assert!(col.hidden);
        assert!(col.nullable);
        assert!(col.read_only);
        assert!(col.user_editable);
        assert!(col.auto_increment);
        assert!(col.key_field);
        assert!(col.mv_enabled);
        assert!(col.selectable);
        assert!(col.version_field);
    }

    #[test]
    fn query_column_deserializes_camel_case_booleans() {
        // The primary camelCase keys should still work.
        let json = serde_json::json!({
            "name": "Col",
            "fieldKey": "Col",
            "hidden": true,
            "nullable": true,
            "readOnly": true,
            "userEditable": true,
            "autoIncrement": true,
            "keyField": true,
            "mvEnabled": true,
            "selectable": true,
            "versionField": true
        });
        let col: QueryColumn = serde_json::from_value(json).expect("should deserialize");
        assert!(col.hidden);
        assert!(col.nullable);
        assert!(col.read_only);
        assert!(col.user_editable);
        assert!(col.auto_increment);
        assert!(col.key_field);
        assert!(col.mv_enabled);
        assert!(col.selectable);
        assert!(col.version_field);
    }

    #[test]
    fn query_column_booleans_default_false_when_absent() {
        let json = serde_json::json!({
            "name": "Col",
            "fieldKey": "Col"
        });
        let col: QueryColumn = serde_json::from_value(json).expect("should deserialize");
        assert!(!col.hidden);
        assert!(!col.nullable);
        assert!(!col.read_only);
        assert!(!col.user_editable);
        assert!(!col.auto_increment);
        assert!(!col.key_field);
        assert!(!col.mv_enabled);
        assert!(!col.selectable);
        assert!(!col.version_field);
        assert!(col.lookup.is_none());
    }

    #[test]
    fn query_column_deserializes_nested_lookup() {
        let json = serde_json::json!({
            "name": "CreatedBy",
            "fieldKey": "CreatedBy",
            "lookup": {
                "queryName": "Users",
                "schemaName": "core",
                "keyColumn": "UserId",
                "table": "Users",
                "multiValued": "junction"
            }
        });
        let col: QueryColumn = serde_json::from_value(json).expect("should deserialize");
        let lookup = col.lookup.expect("lookup should be Some");
        assert_eq!(lookup.query_name.as_deref(), Some("Users"));
        assert_eq!(lookup.schema_name.as_deref(), Some("core"));
        assert_eq!(lookup.key_column.as_deref(), Some("UserId"));
        assert_eq!(lookup.table.as_deref(), Some("Users"));
        assert_eq!(lookup.multi_valued.as_deref(), Some("junction"));
    }

    #[test]
    fn query_column_lookup_absent_is_none() {
        let json = serde_json::json!({
            "name": "Name",
            "fieldKey": "Name"
        });
        let col: QueryColumn = serde_json::from_value(json).expect("should deserialize");
        assert!(col.lookup.is_none());
    }

    #[test]
    fn response_metadata_deserializes_import_fields() {
        let json = serde_json::json!({
            "fields": [{
                "name": "Col",
                "fieldKey": "Col"
            }],
            "importMessage": "Use TSV format",
            "importTemplates": [
                { "label": "TSV Template", "url": "/template.tsv" }
            ]
        });
        let meta: ResponseMetadata = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(meta.import_message.as_deref(), Some("Use TSV format"));
        assert_eq!(meta.import_templates.len(), 1);
        assert_eq!(meta.import_templates[0].label, "TSV Template");
        assert_eq!(meta.import_templates[0].url, "/template.tsv");
    }

    #[test]
    fn response_metadata_import_fields_default_when_absent() {
        let json = serde_json::json!({
            "fields": [{
                "name": "Col",
                "fieldKey": "Col"
            }]
        });
        let meta: ResponseMetadata = serde_json::from_value(json).expect("should deserialize");
        assert!(meta.import_message.is_none());
        assert!(meta.import_templates.is_empty());
    }

    #[test]
    fn query_lookup_deserializes_new_fields() {
        let json = serde_json::json!({
            "queryName": "Users",
            "schemaName": "core",
            "schema": "core",
            "keyColumn": "UserId",
            "table": "Users",
            "multiValued": "junction",
            "junctionLookup": "MemberOf",
            "filterGroups": [{"name": "active"}]
        });
        let lookup: QueryLookup = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(lookup.schema.as_deref(), Some("core"));
        assert_eq!(lookup.table.as_deref(), Some("Users"));
        assert_eq!(lookup.multi_valued.as_deref(), Some("junction"));
        assert_eq!(lookup.junction_lookup.as_deref(), Some("MemberOf"));
        let fg = lookup.filter_groups.expect("filter_groups should be Some");
        let arr = fg.as_array().expect("filter_groups should be an array");
        assert_eq!(arr.len(), 1);
        assert_eq!(arr[0]["name"], "active");
    }

    #[test]
    fn query_lookup_new_fields_default_none_when_absent() {
        let json = serde_json::json!({
            "queryName": "Users",
            "schemaName": "core"
        });
        let lookup: QueryLookup = serde_json::from_value(json).expect("should deserialize");
        assert!(lookup.schema.is_none());
        assert!(lookup.table.is_none());
        assert!(lookup.multi_valued.is_none());
        assert!(lookup.junction_lookup.is_none());
        assert!(lookup.filter_groups.is_none());
    }

    #[test]
    fn query_lookup_accepts_public_alias_for_is_public() {
        // The server sometimes sends "public" instead of "isPublic".
        let json = serde_json::json!({
            "queryName": "Users",
            "schemaName": "core",
            "public": true
        });
        let lookup: QueryLookup = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(lookup.is_public, Some(true));
    }

    #[test]
    fn query_lookup_accepts_is_public_primary_key() {
        let json = serde_json::json!({
            "queryName": "Users",
            "schemaName": "core",
            "isPublic": false
        });
        let lookup: QueryLookup = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(lookup.is_public, Some(false));
    }

    #[test]
    fn select_distinct_options_includes_method_field() {
        let opts = SelectDistinctOptions::builder()
            .schema_name("core".to_string())
            .query_name("Users".to_string())
            .column("Name".to_string())
            .method(RequestMethod::Post)
            .build();
        assert_eq!(opts.method, Some(RequestMethod::Post));
    }

    #[test]
    fn select_distinct_options_method_defaults_to_none() {
        let opts = SelectDistinctOptions::builder()
            .schema_name("core".to_string())
            .query_name("Users".to_string())
            .column("Name".to_string())
            .build();
        assert!(opts.method.is_none());
    }
}
