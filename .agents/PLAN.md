# Implementation Plan: labkey-rs Feature Catalog and Commit Plan

This document is the comprehensive reference for implementing the remaining features of labkey-rs. It catalogs every endpoint and type from both the upstream JS client (`@labkey/api` v1.48.0) and the Java client (`labkey-api-java`), organized into commit-sized work blocks with enough detail that each Ralph loop iteration can execute without re-reading upstream source files.

The feature set is a right-join of the JS client onto the Java client: everything the JS client supports, everything both clients support, and Java-only features that are valuable for Rust users.


## What's Already Implemented

Three commits on `main` cover the foundation:

**Error types** (`src/error.rs`): `FieldError`, `ApiErrorBody`, `LabkeyError` (variants: `Http`, `Api`, `UnexpectedResponse`, `Deserialization`, `Url`).

**Client** (`src/client.rs`): `Credential` (Basic, ApiKey), `ClientConfig`, `LabkeyClient` with `build_url`, `prepare_request`, `get`, `post`, `handle_response`. Container path segments are percent-encoded individually via `encode_container_path`. The `X-Requested-With: XMLHttpRequest` header is set on all requests.

**Filter system** (`src/filter.rs`): `FilterType` (42 variants), `Filter`, `FilterValue`, `ContainerFilter` (10 variants), `encode_filters`. Multi-value filters use semicolon or comma separators with `{json:[...]}` fallback when values contain the separator.

**Query read endpoints** (`src/query.rs`): `SelectRowsOptions` and `ExecuteSqlOptions` (both with bon builders), `waf_encode`, response types (`CellValue`, `Row`, `QueryColumn`, `ResponseMetadata`, `SelectRowsResponse`), and the `select_rows`/`execute_sql` methods. Helper functions `container_filter_to_string` and `opt` are `pub(crate)` utilities. The `ExecuteSqlBody` struct handles JSON serialization with `skip_serializing_if`.

**Dependencies**: reqwest (json, query features), serde + serde_json, url, thiserror, base64, urlencoding, bon. Dev: tokio.

**Conventions**: bon builders on option structs, `#[non_exhaustive]` on option structs, declarative `Option::map` + `flatten` for URL param construction, `pub(crate)` on internal helpers, `#[allow(clippy::...)]` requires justification comment.


## Cross-Cutting Patterns

These patterns recur across multiple modules and should be established early so that later commits can reuse them.

### Pattern 1: Audit Behavior

The `insertRows`, `updateRows`, `deleteRows`, `saveRows`, `moveRows`, and several domain endpoints accept audit configuration. This should be a shared enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum AuditBehavior {
    #[serde(rename = "NONE")]
    None,
    #[serde(rename = "SUMMARY")]
    Summary,
    #[serde(rename = "DETAILED")]
    Detailed,
}
```

This belongs in `src/common.rs` (a narrow shared module for types used by multiple API modules). Do NOT create a `src/types.rs` grab-bag. Only types that are genuinely cross-cutting go here — `AuditBehavior` qualifies because both query and domain use it. `CommandType` does NOT belong here; it stays in `query.rs` since only query endpoints use it. Similarly, `opt()` and `container_filter_to_string` stay in `query.rs` unless a second module actually needs them, at which point they move to `common.rs`.

### Pattern 2: Mutation Request Base

`insertRows`, `updateRows`, `deleteRows`, and `truncateTable` share an almost identical request shape. The shared fields are: `schemaName`, `queryName`, `rows` (as `Vec<serde_json::Value>`), `containerPath`, `auditBehavior`, `auditDetails`, `auditUserComment`, `extraContext`, `transacted`, `skipReselectRows`, `timeout`. Each endpoint gets its own options type (separate thin structs, not a god struct), but they all serialize to the same JSON body shape via a shared `MutateRowsBody` struct with `#[derive(Serialize)]` and `#[serde(skip_serializing_if)]`. The `CommandType` enum (Insert, Update, Delete) stays in `query.rs` since only `save_rows` uses it and it is not cross-cutting.

### Pattern 3: Mutation Response

`insertRows`, `updateRows`, `deleteRows`, `truncateTable`, and `moveRows` all return a `ModifyRowsResults` shape:

```rust
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "camelCase")]
pub struct ModifyRowsResults {
    pub command: Option<String>,         // "insert", "update", "delete"
    pub schema_name: Option<String>,
    pub query_name: Option<String>,
    pub rows: Vec<serde_json::Value>,
    pub rows_affected: i64,
    #[serde(default)]
    pub errors: Vec<FieldError>,
}
```

All response structs throughout the crate must be `#[non_exhaustive]` to prevent users from constructing them via struct literals (which would break when we add fields). This applies to every response type in every module.

`moveRows` returns a `MoveRowsResponse` that extends this with `success`, `container_path`, `error`, `update_counts`. `saveRows` returns `SaveRowsResponse` with `committed`, `error_count`, `result: Vec<ModifyRowsResults>`.

### Pattern 4: Per-Request Options (Timeout, Redirect Handling, Accepted Statuses)

Several endpoints need per-request control over timeout, redirect behavior, or accepted status codes. Rather than proliferating `get_with_timeout`/`post_with_timeout` helpers, use a single internal abstraction:

```rust
/// Per-request options that modify how the client sends and handles a request.
/// This is `pub(crate)` — not part of the public API.
#[derive(Debug, Default)]
pub(crate) struct RequestOptions {
    /// Per-request timeout, applied via `reqwest::RequestBuilder::timeout()`.
    pub timeout: Option<std::time::Duration>,
    /// If true, build a one-off reqwest::Client with redirect policy disabled.
    /// Used by `stop_impersonating` which treats 302 as success.
    pub no_follow_redirects: bool,
    /// Additional status codes to treat as success (beyond 2xx).
    /// Used by `stop_impersonating` which returns 302.
    pub accepted_statuses: Vec<reqwest::StatusCode>,
}
```

The existing `get`/`post` methods gain an optional `RequestOptions` parameter (or we add `get_with_options`/`post_with_options` variants that the plain `get`/`post` delegate to with `RequestOptions::default()`). Endpoint option structs expose `timeout: Option<Duration>` using `std::time::Duration`, not reqwest's type.

When `no_follow_redirects` is true, the one-off `reqwest::Client` must be built with the same configuration as the base client (user agent, proxy, cert policy) except for the redirect policy. To support this, `LabkeyClient` should store the configuration values needed to rebuild a client (or store a `reqwest::ClientBuilder` factory closure). The simplest approach: store the `ClientConfig` fields that affect client construction and rebuild when needed.

### Pattern 5: Container Path Override

Every endpoint accepts an optional `container_path` that overrides the client's default. This is already handled by `build_url`'s `container_override` parameter.

### Pattern 6: FormData / Multipart Upload

`insertRows` and `updateRows` support `autoFormFileData` which converts row data containing `File` objects into multipart `FormData`. `saveRows` has a similar mechanism via `bindSaveRowsData`. The Java client's `ImportDataCommand` uses multipart for file upload, text data, or server-side path references. In Rust, we'll support multipart via reqwest's `multipart::Form`. This requires adding a `post_multipart` helper to `LabkeyClient` that accepts a `reqwest::multipart::Form` and optional `RequestOptions`, and designing the Rust-side API for specifying file data (likely accepting `Vec<u8>` or `impl Read` rather than filesystem paths, since we're a library). The `post_multipart` helper should also accept `RequestOptions` for timeout support.

### Pattern 7: Shared Security Types

The security module defines several types used across multiple endpoints: `Container`, `ContainerHierarchy`, `User`, `Group`, `Role`, `SecurableResource`, `Policy`. These should all live in a `security` module (or `security/types.rs` if the module gets large enough to split).

### Pattern 8: Experiment/Assay Shared Types

The experiment and assay modules share types like `RunGroup`, `Run`, `Data`, `Material`, `ExpObject`, `LineageNode`, `LineageEdge`. These should live in an `experiment` module with the assay module importing from it as needed.


### Pattern 9: Nested Response Extraction

Many LabKey endpoints wrap their payload in an envelope object (e.g., `{ "runs": [...] }`, `{ "batch": {...} }`, `{ "data": [...] }`). The JS client silently extracts the inner field and returns it. We handle this with private envelope structs:

```rust
/// Private envelope — never exposed in the public API.
#[derive(Deserialize)]
struct LoadBatchEnvelope {
    batch: Option<RunGroup>,
}
```

The endpoint method deserializes the envelope, then extracts the inner field. If the expected field is absent, return `LabkeyError::UnexpectedResponse` with a descriptive message. Do NOT return the envelope struct to users — always unwrap to the inner type. This keeps the public API clean while handling the server's inconsistent wrapping.

### Pattern 10: Enum Exhaustiveness Policy

New enums that represent server-defined vocabularies (values the server may add to in future versions) must be `#[non_exhaustive]`. This includes `AuditBehavior`, `DomainKind`, `StorageType`, `IconType`, `FitType`, `SeqType`, `ExpType`, `InsertOption`, `DataViewType`, `RecipientType`, `ContentType`, `AssayLink`, and any other enum whose variants come from the server.

The existing `FilterType` and `ContainerFilter` enums are currently exhaustive. Adding variants later is a breaking change. The decision: migrate both to `#[non_exhaustive]` in the first commit that touches them (commit 5), as a one-time semver break before 1.0. This is the right time to do it.

### Pattern 11: Trait Derivation Policy

Response types: `Debug, Clone, Deserialize`. Add `Serialize` only when there is a concrete use case (e.g., the type is also used as input to another endpoint). Do NOT blanket-derive `Serialize` on responses.

C-like enums (no data variants): `Debug, Clone, Copy, PartialEq, Eq`. Add `Hash` only when the enum will be used as a map key. Add `Serialize` and/or `Deserialize` as needed for wire format.

Option structs (bon builders): `Debug, Clone`. They get `#[non_exhaustive]` and bon `#[builder]`. They do NOT derive `Serialize` or `Deserialize` — they are input-only types.

### Pattern 12: Crate Root Re-exports

Keep the crate root (`src/lib.rs`) minimal. Only re-export: `LabkeyClient`, `ClientConfig`, `Credential`, `LabkeyError`. All other types stay namespaced under their modules (`labkey_rs::query::SelectRowsOptions`, `labkey_rs::security::Container`, etc.). Users access them via `use labkey_rs::query::*` or similar. This prevents the root namespace from becoming unwieldy and avoids semver hazards from re-export changes.

### Pattern 13: `serde_json::Value` Fields

Some response fields have complex or variable shapes that we don't want to type immediately. Using `serde_json::Value` is acceptable short-term, but be aware that changing a field from `Value` to a typed struct later is a breaking change. When using `Value`, prefer adding typed accessor methods alongside the `Value` field rather than replacing it. New typed fields can be added additively.


### Pattern 14: Client-Side Input Validation

Several endpoints require "exactly one of X/Y" or "at least one of X/Y" (e.g., `import_data` requires exactly one source, `get_protocol` requires either `provider_name` or `protocol_id`, `impersonate_user` requires either `user_id` or `email`). We enforce these constraints at the type level where possible using enums:

```rust
pub enum ImportDataSource {
    Text(String),
    File { data: Vec<u8>, filename: String },
    Path(String),
    ModuleResource { path: String, module: String },
}
```

When an enum isn't ergonomic (e.g., `impersonate_user` where both fields are simple scalars), validate in the method body and return `LabkeyError::InvalidInput(String)` with a descriptive message. Add the `InvalidInput` variant to `LabkeyError` in commit 5 (alongside the other semver housekeeping).

Affected endpoints and their constraints:
- `import_data`: exactly one of Text/File/Path/ModuleResource (use enum)
- `import_run`: exactly one of file/run_file_path/data_rows (use enum)
- `get_protocol`: exactly one of provider_name/protocol_id (use enum: `ProtocolIdentifier::ByProvider(String)` / `ProtocolIdentifier::ById { id: i64, copy: Option<bool> }`)
- `impersonate_user`: exactly one of user_id/email (use enum: `ImpersonateTarget::UserId(i64)` / `ImpersonateTarget::Email(String)`)
- `create_hidden_run_group`: exactly one of run_ids/selection_key (use enum)
- `rename_container`: at least one of name/title (validate in method body)


## Module-by-Module Feature Catalog

### Module: Query (src/query.rs)

Already implemented: `select_rows`, `execute_sql`, response types.

#### Remaining Query Endpoints

**insert_rows** — POST `query-insertRows.api`
- Required: `schema_name`, `query_name`, `rows: Vec<serde_json::Value>`
- Optional: `container_path`, `audit_behavior: AuditBehavior`, `audit_details: serde_json::Value`, `audit_user_comment: String`, `extra_context: serde_json::Value`, `transacted: bool` (default true), `skip_reselect_rows: bool`, `timeout: Duration`
- File upload: the JS client supports `autoFormFileData` which converts rows with `File` objects to multipart. In Rust, we defer multipart support for `insert_rows`/`update_rows` to a future enhancement. For now, these endpoints always send JSON. The `import_data` endpoint (commit 12) and `import_run` (commit 19) cover the multipart file upload use cases.
- Response: `ModifyRowsResults`
- JSON body fields: `schemaName`, `queryName`, `rows`, `transacted`, `extraContext`, `auditBehavior`, `auditDetails`, `auditUserComment`, `skipReselectRows`

**update_rows** — POST `query-updateRows.api`
- Same shape as `insert_rows` (same `QueryRequestOptions` base).
- Response: `ModifyRowsResults`

**delete_rows** — POST `query-deleteRows.api`
- Same base shape. Rows need only contain primary key fields.
- No file upload support.
- Response: `ModifyRowsResults`

**truncate_table** — POST `query-truncateTable.api`
- Same base shape. Rows is typically empty/omitted.
- No file upload support.
- Response: `ModifyRowsResults`

**move_rows** — POST `query-moveRows.api`
- Required: `schema_name`, `query_name`, `target_container_path: String`
- Optional: `rows`, `container_path`, `audit_behavior`, `audit_details`, `audit_user_comment`, `extra_context`, `data_region_selection_key: String`, `use_snapshot_selection: bool`
- Response: `MoveRowsResponse` (extends `ModifyRowsResults` with `success: bool`, `container_path`, `error`, `update_counts: HashMap<String, i64>`)
- JSON body: `targetContainerPath`, `schemaName`, `queryName`, `rows`, `auditBehavior`, `auditDetails`, `auditUserComment`, `dataRegionSelectionKey`, `useSnapshotSelection`, `extraContext`

**save_rows** — POST `query-saveRows.api`
- Required: `commands: Vec<SaveRowsCommand>`
- Optional: `container_path`, `api_version: String`, `audit_details: serde_json::Value`, `extra_context: serde_json::Value`, `transacted: bool` (default true), `validate_only: bool`, `timeout: Duration`
- `SaveRowsCommand` struct: `command: CommandType` (enum: Insert, Update, Delete), `schema_name`, `query_name`, `rows: Vec<serde_json::Value>`, `container_path`, `extra_context`, `audit_behavior`, `audit_details`, `audit_user_comment`, `skip_reselect_rows`
- File upload: deferred (same rationale as `insert_rows`). `save_rows` always sends JSON for now.
- Response: `SaveRowsResponse` — `committed: bool`, `error_count: i64`, `result: Vec<ModifyRowsResults>`

**select_distinct_rows** — GET `query-selectDistinct.api` (the JS client allows POST via an option, but we use GET; all params go in the query string)
- Required: `schema_name`, `query_name`, `column: String`
- Optional: `container_path`, `container_filter`, `data_region_name: String` (default "query"), `filter_array`, `ignore_filter: bool`, `max_rows: i32` (default 100000, -1 for all), `sort`, `view_name`, `parameters: HashMap<String, String>`
- Response: `SelectDistinctResponse` — `schema_name: String`, `query_name: String`, `values: Vec<serde_json::Value>`

**get_query_details** — GET `query-getQueryDetails.api` (the JS client allows POST via an option, but we use GET; all params go in the query string)
- Required: `schema_name`, `query_name`
- Optional: `container_path`, `fields: Vec<String>`, `fk: String`, `include_triggers: bool`, `initialize_missing_view: bool`, `view_name: Vec<String>` (or single string; use `"*"` for all views)
- Response: `QueryDetailsResponse` — extensive type with `name`, `schema_name`, `title`, `title_column`, `description`, `can_edit`, `can_edit_shared_views`, `is_inherited`, `is_metadata_overrideable`, `is_temporary`, `is_user_defined`, `columns: Vec<QueryDetailsColumn>`, `default_view`, `edit_definition_url`, `import_templates`, `indices`, `target_containers`, `view_data_url`, `views: Vec<QueryView>`, plus optional `audit_history_url`, `exception`, `icon_url`, `import_message`, `import_url`, `insert_url`, `module_name`, `warning`
- Supporting types: `QueryDetailsColumn` extends the response-level `QueryColumn` we already have with many additional metadata fields. Strategy: define a typed core of commonly-used fields (all the fields from `QueryColumn` plus: `align: Option<String>`, `calculated: Option<bool>`, `concept_uri: Option<String>`, `default_scale: Option<String>`, `dimension: Option<bool>`, `display_field: Option<serde_json::Value>`, `faceting_behavior_type: Option<String>`, `field_key_array: Option<Vec<String>>`, `field_key_path: Option<String>`, `friendly_type: Option<String>`, `input_type: Option<String>`, `lookup: Option<QueryLookup>`, `measure: Option<bool>`, `multi_value: Option<bool>`, `name_expression: Option<String>`, `phi: Option<String>`, `range_uri: Option<String>`, `recommended_variable: Option<bool>`, `required: Option<bool>`, `selectable: Option<bool>`, `shown_in_details_view: Option<bool>`, `shown_in_insert_view: Option<bool>`, `shown_in_update_view: Option<bool>`, `sortable: Option<bool>`, `type_name: Option<String>`, `type_uri: Option<String>`, `value_expression: Option<String>`, `version_field: Option<bool>`). Use `#[serde(flatten)] pub extra: HashMap<String, serde_json::Value>` to capture any remaining server-added fields without breaking deserialization. `QueryLookup` (`container: Option<String>`, `container_path: Option<String>`, `display_column: Option<String>`, `is_public: Option<bool>`, `junction_lookup: Option<String>`, `key_column: Option<String>`, `multi_valued: Option<String>`, `query_name: Option<String>`, `schema: Option<String>`, `schema_name: Option<String>`, `table: Option<String>`), `QueryView` (`columns: Option<Vec<QueryViewColumn>>`, `container_filter: Option<String>`, `container_path: Option<String>`, `default: Option<bool>`, `deletable: Option<bool>`, `editable: Option<bool>`, `fields: Option<Vec<serde_json::Value>>`, `filter: Option<Vec<QueryViewFilter>>`, `hidden: Option<bool>`, `inherit: Option<bool>`, `label: Option<String>`, `name: Option<String>`, `owner: Option<serde_json::Value>`, `revertable: Option<bool>`, `savable: Option<bool>`, `session: Option<bool>`, `shared: Option<bool>`, `sort: Option<Vec<QueryViewSort>>`, `view_data_url: Option<String>`), `QueryViewColumn` (`field_key: String`, `title: Option<String>`), `QueryViewFilter` (`field_key: String`, `op: String`, `value: String`), `QueryViewSort` (`field_key: String`, `dir: String`), `QueryImportTemplate` (`label: String`, `url: String`), `QueryIndex` (`column_names: Vec<String>`, `unique: bool`, `type_: Option<String>`)

**get_queries** — GET `query-getQueries.api`
- Required: `schema_name`
- Optional: `container_path`, `include_columns: bool` (default true), `include_system_queries: bool` (default true), `include_title: bool` (default true), `include_user_queries: bool` (default true), `include_view_data_url: bool` (default true), `query_detail_columns: bool` (default false)
- Response: `GetQueriesResponse` — `schema_name: String`, `queries: Vec<QueryInfo>` where `QueryInfo` has `can_edit`, `can_edit_shared_views`, `columns: Vec<QueryDetailsColumn>`, `description`, `hidden`, `inherit`, `is_inherited`, `is_metadata_overrideable`, `is_user_defined`, `name`, `snapshot`, `title`, `view_data_url`

**get_schemas** — GET `query-getSchemas.api`
- Optional: `container_path`, `api_version`, `schema_name`, `include_hidden: bool`
- Response: `serde_json::Value` (the server returns a JSON object keyed by schema name; the shape varies by API version and is not strongly typed in the JS client)

**get_query_views** — GET `query-getQueryViews.api`
- Optional: `container_path`, `schema_name`, `query_name`, `view_name`, `metadata: serde_json::Value`, `exclude_session_view: bool`
- Response: `serde_json::Value` (not strongly typed in JS client)

**save_query_views** — POST `query-saveQueryViews.api`
- Optional: `container_path`, `schema_name`, `query_name`, `views: serde_json::Value`, `shared: bool`, `session: bool`, `hidden: bool`
- JSON body: `{ schemaName, queryName, views, shared, session, hidden }` (booleans only sent if true)
- Response: `serde_json::Value`

**save_session_view** — POST `query-saveSessionView.api`
- Optional: `container_path`, `schema_name`, `query_name`, `view_name`, `new_name: String`, `shared: bool`, `inherit: bool`, `hidden: bool`, `replace: bool`
- JSON body: `{ schemaName, "query.queryName": queryName, "query.viewName": viewName, newName, shared, inherit, hidden, replace }` (note the `query.` prefix on queryName and viewName; booleans only sent if true)
- Response: `serde_json::Value`

**delete_query_view** — POST `query-deleteView.api`
- Required: `schema_name`, `query_name`
- Optional: `container_path`, `view_name`, `revert: bool`
- JSON body: `{ schemaName, queryName, viewName, complete: !revert }` (complete only sent if revert is explicitly set)
- Response: `serde_json::Value`

**get_data_views** — POST `reports-browseData.api` (note: `reports` controller, not `query`)
- Optional: `container_path`, `data_types: Vec<DataViewType>` (enum: Datasets, Queries, Reports), `timeout: Duration`
- JSON body: `{ includeData: true, includeMetadata: false, dataTypes }` (dataTypes only if provided)
- Response: `serde_json::Value` (the server returns `{ data: [...] }` and the JS client unwraps to just the inner array)

**get_server_date** — GET `query-getServerDate.api`
- No parameters (no container path either — the JS client calls `buildURL('query', 'getServerDate.api')` with no container argument)
- Response: `GetServerDateResponse` — `date: String` (ISO date string; we return the string and let callers parse it with chrono or similar)

**validate_query** — GET `query-validateQuery.api` (or `query-validateQueryMetadata.api` when `validate_query_metadata` is true); all params go in the query string
- Optional: `container_path`, `schema_name`, `query_name`, `validate_query_metadata: bool` (if true, uses the `validateQueryMetadata.api` action instead)
- Response: `ValidateQueryResponse` — `valid: bool`

**import_data** (Java-only) — POST `query-import.api` (multipart/form-data)
- Required: `schema_name`, `query_name`, plus one of: `text: String` (TSV/CSV data), `file: Vec<u8>` (uploaded file), `path: String` (server-side webdav path), `module_resource: String` (module resource path)
- Optional: `container_path`, `module: String` (for module_resource), `format: String` (e.g., "tsv", "csv"), `insert_option: InsertOption` (enum: Import, Merge), `use_async: bool` (file-only), `save_to_pipeline: bool` (file-only), `import_identity: bool`, `import_lookup_by_alternate_key: bool`
- Multipart form fields: each parameter as a text part, file as binary part with content type `application/octet-stream`
- Response: `ImportDataResponse` — `success: bool`, `row_count: i64`, `job_id: Option<String>` (present when async)

**get_data** (aka `getRawData`) — POST `query-getData` (note: NO `.api` suffix — this is intentional and matches the JS client)
- Required: `source: GetDataSource` (struct with `type_: GetDataSourceType` (enum: Query, Sql), `schema_name`, `query_name` (if Query), `sql` (if Sql), `container_path`)
- Optional: `columns: Vec<Vec<String>>` (field key arrays), `include_details_column: bool`, `max_rows: i32` (default 100000, -1 for all), `offset: i64`, `sort: Vec<GetDataSort>` (struct with `field_key: Vec<String>`, `dir: Option<String>` — "ASC" or "DESC"), `transforms: Vec<GetDataTransform>` (struct with `type_: Option<String>`, `filters: Vec<GetDataFilter>`, `group_by: Vec<Vec<String>>`, `aggregates: Vec<GetDataAggregate>`), `pivot: GetDataPivot` (struct with `by: Vec<String>`, `columns: Vec<Vec<String>>`)
- Supporting types: `GetDataFilter` (struct with `field_key: Vec<String>`, `type_: String` (filter type name), `value: Option<serde_json::Value>`), `GetDataAggregate` (struct with `field_key: Vec<String>`, `type_: String`), `GetDataSourceType` enum (Query, Sql)
- JSON body: the entire config is serialized as `{ source: { type, schemaName, queryName, sql, containerPath }, renderer: { type: "json", columns, includeDetailsColumn, maxRows, offset, sort }, transforms, pivot }`
- Response: `SelectRowsResponse` (same as `select_rows` — the server returns the standard query response format via the `Response` class)
- Note: this is a complex endpoint with nested config. The JS client constructs a `renderer` object internally with `type: "json"`. The Rust implementation should build this structure in the method body, not expose `renderer` to users.

#### Query Utility Types and Functions

**sql_date_literal**, **sql_date_time_literal**, **sql_string_literal** — These are string formatting helpers from the JS client's `Utils.ts`. They wrap values in LabKey SQL literal syntax: `{d 'YYYY-MM-DD'}`, `{ts 'YYYY-MM-DD HH:MM:SS'}`, `{s 'escaped string'}`. Simple to implement as standalone functions.

**URL_COLUMN_PREFIX** — The constant `"_labkeyurl_"` (no second underscore between "labkey" and "url") used to identify URL columns in response data. Should be a `pub const`.


### Module: Security (src/security/ — directory module with submodules)

This is the largest module by endpoint count. All types defined here are used across multiple security endpoints.

#### Shared Security Types

**Container** (response type used by `create_container`, `rename_container`, `get_containers`):
```
active_modules: Vec<String>, effective_permissions: Option<Vec<String>>,
folder_type: String, formats: ContainerFormats, has_restricted_active_module: bool,
icon_href: String, id: String, is_archived: bool, is_container_tab: bool,
is_workbook: bool, name: String, parent_id: String, parent_path: String,
path: String, sort_order: i64, start_url: String, title: String,
type_: String  // "type" is reserved in Rust, use #[serde(rename = "type")]
```

**ContainerFormats**: `date_format: String, date_time_format: String, number_format: String, time_format: String`

**ContainerHierarchy** (extends Container): adds `children: Vec<ContainerHierarchy>`, `module_properties: Vec<ModuleProperty>`, `user_permissions: Option<i64>` (deprecated)

**ModuleProperty**: `effective_value: serde_json::Value, module: String, name: String, value: serde_json::Value`

**User** (from security types): `display_name: String, email: String, user_id: i64`, plus optional fields that vary by endpoint

**Group**: `id: i64, name: String, type_: String, is_project_group: bool, is_system_group: bool, effective_permissions: Vec<String>, groups: Vec<Group>, roles: Vec<String>`

**Role**: `name: String, description: String, unique_name: String, source_module: String, excluded_principals: Vec<i64>, permissions: Vec<RolePermission>`

**RolePermission**: `name: String, description: String, unique_name: String, source_module: String`

**SecurableResource**: `id: String, name: String, description: String, resource_class: String, parent_id: Option<String>, parent_container_path: Option<String>, effective_permissions: Option<Vec<String>>, children: Vec<SecurableResource>`

**Policy**: `resource_id: String, assignments: Vec<PolicyAssignment>, modified: Option<String>, modified_millis: Option<i64>`

**PolicyAssignment**: `role: String, user_id: i64`

**FolderType**: `name: String, label: String, description: String, default_module: String, active_modules: Vec<String>, preferred_web_parts: Vec<FolderTypeWebPart>, required_web_parts: Vec<FolderTypeWebPart>, workbook_type: bool`

**FolderTypeWebPart**: `name: String, properties: serde_json::Value`

#### Container Endpoints

**create_container** — POST `core-createContainer.api`
- Required: `name: String`
- Optional: `container_path`, `description: String`, `folder_type: String`, `is_workbook: bool`, `title: String`
- JSON body: `{ name, description, folderType, isWorkbook, title }`
- Response: `Container`

**delete_container** — POST `core-deleteContainer.api`
- Optional: `container_path`, `comment: String`
- JSON body: `{ comment }`
- Response: `serde_json::Value` (generic success)

**rename_container** — POST `admin-renameContainer.api`
- Required: at least one of `name` or `title`
- Optional: `container_path`, `name: String`, `title: String`, `add_alias: bool`
- JSON body: `{ name, title, addAlias }`
- Response: `Container`

**get_containers** — GET `project-getContainers.api`
- Optional: `container: Vec<String>` (container IDs or paths; when multiple, sets `multipleContainers=true`), `container_path`, `depth: i32`, `include_effective_permissions: bool` (default true), `include_inheritable_formats: bool`, `include_standard_properties: bool` (default true), `include_subfolders: bool` (default false), `include_workbook_children: bool` (default true), `module_properties: Vec<String>` (use `"*"` for all)
- Response: always `Vec<ContainerHierarchy>`. When the server returns a single container (no `multipleContainers` param), wrap it in a one-element `Vec`. When the server returns `{ containers: [...] }` (multi mode), extract the inner array. This gives callers a uniform return type regardless of how many containers were requested.

**get_readable_containers** — GET `project-getReadableContainers.api`
- Optional: `container: Vec<String>`, `container_path`, `depth: i32`, `include_subfolders: bool`
- Response: `Vec<String>` (container paths, extracted from `response.containers`)

**get_folder_types** — POST `core-getFolderTypes.api`
- Optional: `container_path`
- Response: `HashMap<String, FolderType>`

**get_modules** — POST `admin-getModules.api`
- Optional: `container_path`
- Response: `GetModulesResponse` — `folder_type: String`, `modules: Vec<ModuleInfo>` where `ModuleInfo` has `name`, `active`, `enabled`, `require_site_permission`, `required`, `tab_name`

**move_container** — POST `core-moveContainer.api`
- Required: `container` (ID or path) AND `parent` (destination parent path)
- Optional: `container_path`, `add_alias: bool` (default true)
- JSON body: `{ addAlias, container, parent }`
- Response: `serde_json::Value` (generic success)

#### User Endpoints

**create_new_user** — POST `security-createNewUser.api`
- Required: `email: String` (semicolon-separated for multiple)
- Optional: `container_path`, `send_email: bool`, `optional_message: String`
- JSON body: `{ email, sendEmail, optionalMessage }`
- Response: `CreateNewUserResponse` — `success: bool`, `email: Option<String>`, `user_id: Option<i64>`, `message: Option<String>`, `html_errors: Vec<String>`, `users: Vec<CreatedUser>` where `CreatedUser` has `email`, `is_new`, `message`, `user_id`

**get_users** — GET `user-getUsers.api`
- Optional: `container_path`, `group_id: i64`, `group: String`, `name: String`, `all_members: bool`, `active: bool`, `permissions: Vec<String>`
- Response: `GetUsersResponse` — `container: String`, `name: Option<String>`, `users: Vec<User>`

**get_users_with_permissions** — GET `user-getUsersWithPermissions.api`
- Required: `permissions: Vec<String>`
- Optional: same as `get_users` plus `include_inactive: bool`, `api_version: f64`
- Response: same shape as `GetUsersResponse`

**ensure_login** — GET `security-ensureLogin.api`
- Optional: `force: bool`
- Response: `EnsureLoginResponse` — `current_user: User`
- Note: The JS client's `useSiteLoginPage` option is browser-only (redirects to login page). We skip that parameter.

#### Group Endpoints

**create_group** — POST `security-createGroup.api`
- Required: `group_name: String`
- Optional: `container_path`
- JSON body: `{ name: groupName }`
- Response: `CreateGroupResponse` — `id: i64`, `name: String`

**delete_group** — POST `security-deleteGroup.api`
- Required: `group_id: i64`
- Optional: `container_path`
- JSON body: `{ id: groupId }`
- Response: `DeleteGroupResponse` — `deleted: i64`

**rename_group** — POST `security-renameGroup.api`
- Required: `group_id: i64`, `new_name: String`
- Optional: `container_path`
- JSON body: `{ id: groupId, newName }`
- Response: `RenameGroupResponse` — `renamed: i64`, `success: bool`, `old_name: String`, `new_name: String`

**add_group_members** — POST `security-addGroupMember.api`
- Required: `group_id: i64`, `principal_ids: Vec<i64>`
- Optional: `container_path`
- JSON body: `{ groupId, principalIds: [...] }`
- Response: `AddGroupMembersResponse` — `added: Vec<i64>`

**remove_group_members** — POST `security-removeGroupMember.api`
- Required: `group_id: i64`, `principal_ids: Vec<i64>`
- Optional: `container_path`
- JSON body: `{ groupId, principalIds: [...] }`
- Response: `RemoveGroupMembersResponse` — `removed: Vec<i64>`

**get_groups_for_current_user** — GET `security-getGroupsForCurrentUser.api`
- Optional: `container_path`
- Response: `GetGroupsResponse` — `groups: Vec<GroupSummary>` where `GroupSummary` has `id`, `name`, `is_project_group`, `is_system_group`

#### Permission Endpoints

**get_group_permissions** — GET `security-getGroupPerms.api`
- Optional: `container_path`, `include_subfolders: bool`, `include_empty_perm_groups: bool`
- Response: `GroupPermissionsResponse` — `container: PermissionsContainer` where `PermissionsContainer` has `id`, `name`, `path`, `is_inheriting_perms`, `groups: Vec<Group>`, `children: Option<Vec<PermissionsContainer>>`

**get_user_permissions** — GET `security-getUserPerms.api`
- Optional: `container_path`, `user_id: i64`, `user_email: String`, `include_subfolders: bool`
- Response: `UserPermissionsResponse` — `container: UserPermissionsContainer` (extends PermissionsContainer with `effective_permissions`, `roles`), `user: UserSummary` (`display_name`, `user_id`)

**get_roles** — GET `security-getRoles.api`
- Optional: `container_path`
- Response: `Vec<Role>`

**get_securable_resources** — GET `security-getSecurableResources.api`
- Optional: `container_path`, `include_subfolders: bool`, `include_effective_permissions: bool`
- Response: `GetSecurableResourcesResponse` — `resources: SecurableResource`

#### Policy Endpoints

**get_policy** — POST `security-getPolicy.api` (despite being conceptually a read, the JS client uses `jsonData` which implies POST)
- Required: `resource_id: String`
- Optional: `container_path`
- JSON body: `{ resourceId }`
- Response: extract `policy` and `relevant_roles` from the response object. Return `GetPolicyResponse` — `policy: Policy`, `relevant_roles: Vec<String>`. The JS client also sets `policy.requestedResourceId = config.resourceId` on the response; we should do the same.

**save_policy** — POST `security-savePolicy.api`
- Required: `policy: Policy`
- Optional: `container_path`
- JSON body: the Policy object
- Response: `serde_json::Value` (generic success)

**delete_policy** — POST `security-deletePolicy.api`
- Required: `resource_id: String`
- Optional: `container_path`
- JSON body: `{ resourceId }`
- Response: `serde_json::Value` (generic success; causes resource to inherit parent policy)

#### Java-Only Security Endpoints

**logout** — POST `login-logout`
- No parameters
- Ends the current HTTP session. No JSON body is sent.
- Response: `serde_json::Value` (generic success)

**who_am_i** — GET `login-whoami.api`
- No parameters
- Response: `WhoAmIResponse` — `id: i64`, `email: String`, `display_name: String`, `impersonated: bool`, `csrf: String`

**delete_user** — POST `security-deleteUser` (note: no `.api` suffix)
- Required: `user_id: i64`
- JSON body: `{ id: userId }`
- Response: `serde_json::Value` (generic success)

**impersonate_user** — POST `user-impersonateUser.api`
- Required: one of `user_id: i64` or `email: String`
- Parameters sent as query params (not JSON body)
- Response: `serde_json::Value` (generic success)

**stop_impersonating** — POST `login-stopImpersonating.api`
- No parameters
- Special handling: success is HTTP 302 (redirect), not 200. Need to disable redirect following for this request and treat 302 as success.
- Response: `serde_json::Value` (generic success)


### Module: Domain (src/domain.rs — new file)

All endpoints use the `property` controller.

#### Shared Domain Types

**DomainDesign**: `domain_id: Option<i64>`, `domain_uri: Option<String>`, `name: Option<String>`, `description: Option<String>`, `container: Option<String>`, `fields: Option<Vec<DomainField>>`, `indices: Option<Vec<DomainIndex>>`, `allow_attachment_properties: Option<bool>`, `allow_file_link_properties: Option<bool>`, `allow_flag_properties: Option<bool>`, `default_default_value_type: Option<String>`, `default_value_options: Option<Vec<String>>`, `instructions: Option<String>`, `show_default_value_settings: Option<bool>`, `template_description: Option<String>`, `query_name: Option<String>`, `schema_name: Option<String>`

**DomainField**: This is a large type with many optional fields. Strategy: use a thin typed wrapper with `#[serde(flatten)]` for extensibility. Define the commonly-used fields as typed (`name: Option<String>`, `range_uri: Option<String>`, `label: Option<String>`, `description: Option<String>`, `property_id: Option<i64>`, `property_uri: Option<String>`, `concept_uri: Option<String>`, `type_editable: Option<bool>`, `dimension: Option<bool>`, `measure: Option<bool>`, `recommended_variable: Option<bool>`, `default_scale: Option<String>`, `phi: Option<String>`, `lookup: Option<serde_json::Value>`, `format: Option<String>`, `hidden: Option<bool>`, `mv_enabled: Option<bool>`, `required: Option<bool>`, `locked_type: Option<String>`), plus `#[serde(flatten)] pub extra: HashMap<String, serde_json::Value>` for everything else. This type needs both `Serialize` and `Deserialize` since it's used as input to `save_domain`/`create_domain` and as output from `get_domain_details`.

**DomainIndex**: `column_names: Vec<String>`, `unique: bool`

**DomainKind** (enum): `DataClass`, `IntList`, `SampleSet` (alias: SampleType), `StudyDatasetDate`, `StudyDatasetVisit`, `Unknown`, `VarList`

#### Domain Endpoints

**create_domain** — POST `property-createDomain.api`
- Optional: `kind: DomainKind` (or `domain_kind`), `domain_design: DomainDesign`, `options: serde_json::Value` (kind-specific), `container_path`, `domain_group: String`, `domain_template: String`, `module: String`, `create_domain: bool` (default true), `import_data: bool` (default true), `timeout: Duration`
- JSON body: the entire options object
- Response: `serde_json::Value`

**get_domain** — GET `property-getDomain.api` (deprecated, use `get_domain_details`)
- Optional: `schema_name`, `query_name`, `domain_id: i64`, `container_path`
- Response: `DomainDesign`

**get_domain_details** — GET `property-getDomainDetails.api`
- Optional: `schema_name`, `query_name`, `domain_id: i64`, `domain_kind: String`, `container_path`
- Response: `GetDomainDetailsResponse` — `domain_design: DomainDesign`, `domain_kind_name: String`, `options: Option<serde_json::Value>`

**save_domain** — POST `property-saveDomain.api`
- Optional: `schema_name`, `query_name`, `domain_id: i64`, `domain_design: DomainDesign`, `options: serde_json::Value`, `container_path`, `include_warnings: bool`, `audit_user_comment: String`
- JSON body: `{ domainDesign, schemaName, queryName, domainId, includeWarnings, auditUserComment, options }`
- Response: `serde_json::Value`

**drop_domain** — POST `property-deleteDomain.api`
- Required: `schema_name`, `query_name`
- Optional: `container_path`, `domain_design: DomainDesign`, `audit_user_comment: String`
- JSON body: `{ domainDesign, schemaName, queryName, auditUserComment }`
- Response: `serde_json::Value`

**update_domain** — POST `property-updateDomain.api`
- Required: `domain_id: i64`
- Optional: `create_fields: Vec<serde_json::Value>`, `update_fields: Vec<serde_json::Value>`, `delete_fields: Vec<i64>`, `include_warnings: bool`, `container_path`
- JSON body: `{ domainId, includeWarnings, createFields, updateFields, deleteFields }`
- Response: `serde_json::Value`

**list_domains** — GET `property-listDomains.api`
- Optional: `domain_kinds: Vec<String>`, `include_fields: bool`, `include_project_and_shared: bool`, `container_path`
- Response: `ListDomainsResponse` — `data: Vec<DomainDesign>`, `success: bool`

**validate_name_expressions** — POST `property-validateNameExpressions.api`
- Optional: `domain_design: DomainDesign`, `options: serde_json::Value`, `kind: DomainKind`, `container_path`, `include_name_preview: bool`
- JSON body: `{ domainDesign, options, kind }`
- Response: `serde_json::Value`

**get_domain_name_previews** — GET `property-getDomainNamePreviews.api`
- Optional: `schema_name`, `query_name`, `domain_id: i64`, `container_path`
- Response: `serde_json::Value`

**get_properties** — POST `property-getProperties.api`
- Optional: `domain_ids: Vec<i64>`, `domain_kinds: Vec<String>`, `filters: Vec<Filter>`, `max_rows: i32`, `offset: i64`, `property_ids: Vec<i64>`, `property_uris: Vec<String>`, `search: String`, `sort: String`, `container_path`
- JSON body: all params
- Response: `serde_json::Value`

**get_property_usages** — GET `property-propertyUsages.api`
- Optional: `property_ids: Vec<i64>`, `property_uris: Vec<String>`, `max_usage_count: i32`, `container_path`
- Response: `Vec<PropertyUsage>` (extracted from `response.data`) where `PropertyUsage` has `property_id: i64`, `property_uri: String`, `usage_count: i64`, `objects: Vec<serde_json::Value>`


### Module: Experiment (src/experiment.rs — new file)

#### Shared Experiment Types

**ExpObject** (base type): `id: Option<i64>`, `row_id: Option<i64>`, `lsid: Option<String>`, `name: Option<String>`, `comment: Option<String>`, `created: Option<String>`, `created_by: Option<String>`, `modified: Option<String>`, `modified_by: Option<String>`, `properties: Option<serde_json::Value>`

**RunGroup** (extends ExpObject): `batch_protocol_id: Option<i64>`, `hidden: Option<bool>`, `runs: Option<Vec<Run>>`

**Run** (extends ExpObject): `data_inputs: Vec<ExpData>`, `data_outputs: Vec<ExpData>`, `data_rows: Vec<serde_json::Value>`, `experiments: Option<serde_json::Value>`, `file_path_root: Option<String>`, `material_inputs: Vec<Material>`, `material_outputs: Vec<Material>`, `object_properties: Option<serde_json::Value>`, `protocol: Option<serde_json::Value>`

**ExpData** (extends ExpObject): `data_type: Option<String>`, `data_file_url: Option<String>`, `pipeline_path: Option<String>`, `role: Option<String>`, `data_class: Option<DataClassRef>` where `DataClassRef` has `id: i64`, `name: String`

**Material** (extends ExpObject): `cpas_type: Option<String>`, `sample_set: Option<SampleSetRef>` where `SampleSetRef` has `id: i64`, `name: String`

**LineageNode**: `id: i64`, `lsid: String`, `name: String`, `container: String`, `container_path: String`, `cpas_type: Option<String>`, `exp_type: String`, `created: String`, `created_by: String`, `modified: String`, `modified_by: String`, `comment: Option<String>`, `query_name: String`, `schema_name: String`, `type_: Option<String>`, `url: Option<String>`, `pk_filters: Vec<PkFilter>`, `restricted: Option<bool>`, `absolute_path: Option<String>`, `distance: Option<i64>`, `data_file_url: Option<String>`, `list_url: Option<String>`, `pipeline_path: Option<String>`, `properties: Option<serde_json::Value>`, `parents: Vec<LineageEdge>`, `children: Vec<LineageEdge>`, `steps: Option<Vec<serde_json::Value>>`, `data_inputs: Option<Vec<serde_json::Value>>`, `data_outputs: Option<Vec<serde_json::Value>>`, `material_inputs: Option<Vec<serde_json::Value>>`, `material_outputs: Option<Vec<serde_json::Value>>`

**PkFilter**: `field_key: String`, `value: serde_json::Value`

**LineageEdge**: `lsid: String`, `role: String`

#### Experiment Endpoints

**lineage** — GET `experiment-lineage.api`
- Required: `lsids: Vec<String>` (the JS client also accepts a single `lsid` but that's deprecated)
- Optional: `container_path`, `parents: bool` (default true), `children: bool` (default true), `depth: i32`, `exp_type: ExpType` (enum: Data, Material, ExperimentRun), `cpas_type: String`, `run_protocol_lsid: String`, `include_inputs_and_outputs: bool`, `include_properties: bool`, `include_run_steps: bool`
- Response: `LineageResponse` — `seed: Option<String>` (deprecated), `seeds: Vec<String>`, `nodes: HashMap<String, LineageNode>`

**resolve** — GET `experiment-resolve.api`
- Optional: `lsids: Vec<String>`, `container_path`, `include_inputs_and_outputs: bool`, `include_properties: bool`, `include_run_steps: bool`
- Response: `ResolveResponse` — `data: Vec<LineageNode>`

**create_hidden_run_group** — POST `experiment-createHiddenRunGroup.api`
- Required: one of `run_ids: Vec<i64>` or `selection_key: String` (not both)
- Optional: `container_path`
- Response: `RunGroup`

**save_batch** — POST `assay-saveAssayBatch.api`
- Required: `assay_id: i64`
- Optional: `assay_name: String`, `batch: RunGroup`, `container_path`, `protocol_name: String`, `provider_name: String`
- JSON body: `{ assayId, assayName, batches: [batch], protocolName, providerName }`
- Response: `RunGroup`

**save_batches** — POST `assay-saveAssayBatch.api` (same endpoint)
- Required: `assay_id: i64`, `batches: Vec<RunGroup>`
- Optional: same as `save_batch`
- Response: `Vec<RunGroup>`

**load_batch** — POST `assay-getAssayBatch.api`
- Required: `assay_id: i64`, `assay_name: String`, `batch_id: i64`, `provider_name: String`
- Optional: `container_path`, `protocol_name: String`
- Response: `RunGroup` (extracted from `response.batch`)

**load_batches** — POST `assay-getAssayBatches.api`
- Required: `assay_id: i64`, `assay_name: String`, `batch_ids: Vec<i64>`, `provider_name: String`
- Optional: `container_path`, `protocol_name: String`
- Response: `Vec<RunGroup>` (extracted from `response.batches`)

**load_runs** — POST `assay-getAssayRuns.api`
- Optional: `run_ids: Vec<i64>`, `lsids: Vec<String>`, `container_path`, `include_inputs_and_outputs: bool`, `include_properties: bool`, `include_run_steps: bool`
- Response: `Vec<Run>` (extracted from `response.runs`)

**save_runs** — POST `assay-saveAssayRuns.api`
- Required: `runs: Vec<serde_json::Value>`
- Optional: `assay_id: i64`, `assay_name: String`, `protocol_name: String`, `provider_name: String`, `container_path`
- Response: `Vec<Run>`

**save_materials** — wrapper around `insert_rows` with `schema_name: "Samples"`, `query_name: options.name`, `rows: options.materials`. Not a direct HTTP call. We should implement this as a convenience method that delegates to `insert_rows`.

**set_entity_sequence** — POST `experiment-setEntitySequence.api`
- Required: `seq_type: SeqType` (enum: GenId, RootSampleCount, SampleCount)
- Optional: `row_id: i64`, `kind_name: String` ("DataClass" or "SampleSet"), `new_value: i64`, `container_path`
- JSON body: `{ rowId, kindName, newValue, seqType }`
- Response: `serde_json::Value`

**get_entity_sequence** — GET `experiment-getEntitySequence.api`
- Required: `seq_type: SeqType`
- Optional: `row_id: i64`, `kind_name: String`, `container_path`
- Response: `serde_json::Value`


### Module: Assay (src/assay.rs — new file)

#### Shared Assay Types

**AssayDesign**: `id: i64`, `name: String`, `type_: String`, `description: String`, `container_path: String`, `project_level: bool`, `protocol_schema_name: String`, `template_link: String`, `import_action: String`, `import_controller: String`, `plate_template: Option<String>`, `domains: serde_json::Value`, `domain_types: serde_json::Value`, `links: HashMap<String, String>`

**AssayLink** (enum): `Batches`, `Begin`, `DesignCopy`, `DesignEdit`, `Import`, `Result`, `Results`, `Runs`

#### Assay Endpoints

**get_assays** — POST `assay-assayList.api`
- Optional: `id: i64`, `name: String`, `type_: String`, `plate_enabled: bool`, `status: String`, `container_path`
- JSON body: `{ id, name, type, plateEnabled, status }` (as `parameters` object)
- Response: `Vec<AssayDesign>` (extracted from `response.definitions`)

**get_nab_runs** — GET `nabassay-getNabRuns.api`
- Required: `assay_name: String`
- Optional: `container_path`, `filter_array: Vec<Filter>`, `sort: String`, `offset: i64`, `max_rows: i32` (-1 for all), `calculate_neut: bool`, `include_fit_parameters: bool`, `include_stats: bool`, `include_wells: bool`, `timeout: Duration`
- Params: sort → `query.sort`, offset → `query.offset`, maxRows → `query.maxRows` (or `query.showRows=all` if -1). Filters appended via `encode_filters`.
- Response: `serde_json::Value` (extracted from `response.runs`)

**get_study_nab_graph_url** — GET `nabassay-getStudyNabGraphURL.api`
- Required: `object_ids: Vec<String>`
- Optional: `container_path`, `chart_title: String`, `fit_type: FitType` (enum: FiveParameter, FourParameter, Polynomial), `height: i32`, `width: i32`, `timeout: Duration`
- Params: `objectIds` → `id` param
- Response: `GetStudyNabGraphUrlResponse` — `url: String`, `object_ids: Vec<serde_json::Value>`

**get_study_nab_runs** — GET `nabassay-getStudyNabRuns.api`
- Required: `object_ids: Vec<String>`
- Optional: `container_path`, `calculate_neut: bool`, `include_fit_parameters: bool`, `include_stats: bool`, `include_wells: bool`, `timeout: Duration`
- Response: `serde_json::Value` (extracted from `response.runs`)

#### Java-Only Assay Endpoints

**get_protocol** — GET `assay-getProtocol`
- Required: one of `provider_name: String` or `protocol_id: i64`
- Optional: `container_path`, `copy: bool` (only meaningful with `protocol_id`)
- Params: `providerName` or `protocolId` + `copy` as query params
- Response: `AssayProtocol` (extracted from `response.data`) — see `AssayProtocol` type below

**save_protocol** — POST `assay-saveProtocol`
- Required: `protocol: AssayProtocol`
- Optional: `container_path`
- JSON body: the `AssayProtocol` object serialized directly
- Response: `AssayProtocol` (extracted from `response.data`)

**import_run** — POST `assay-importRun.api` (multipart)
- Required: `assay_id: i64`, plus exactly one of: `file: Vec<u8>` (with filename), `run_file_path: String` (server-side path), `data_rows: Vec<serde_json::Value>` (inline data)
- Optional: `container_path`, `batch_id: i64`, `name: String`, `comment: String`, `properties: HashMap<String, serde_json::Value>`, `batch_properties: HashMap<String, serde_json::Value>`, `plate_metadata: serde_json::Value`, `audit_user_comment: String`, `workflow_task_id: i64`, `use_json: bool` (if true, sends all params as a single `json` multipart part instead of individual parts)
- Multipart form: when `use_json` is false (default), each param is a separate text part. `properties` and `batch_properties` use bracket notation (`properties[key]`). `dataRows` is sent as JSON text. `file` is a binary part with `application/octet-stream` content type. When `use_json` is true, everything goes into a single `json` text part.
- Response: `ImportRunResponse` — `success: bool`, `assay_id: i64`, `batch_id: i64`, `run_id: i64`, `succeeded_count: Option<i64>`

**get_assay_run** — POST `assay-getAssayRun`
- Required: `lsid: String`
- Optional: `container_path`
- JSON body: `{ lsid }`
- Response: `serde_json::Value` (the response shape is not well-documented; use `Value` initially)

#### Shared Assay Protocol Type (Java-only)

**AssayProtocol**: `protocol_id: Option<i64>`, `name: String`, `description: Option<String>`, `provider_name: String`, `domains: Vec<serde_json::Value>` (domain objects — complex, use `Value` initially), `allow_background_upload: Option<bool>`, `background_upload: Option<bool>`, `allow_editable_results: Option<bool>`, `editable_results: Option<bool>`, `editable_runs: Option<bool>`, `save_script_files: Option<bool>`, `allow_qc_states: Option<bool>`, `qc_enabled: Option<bool>`, `allow_spaces_in_path: Option<bool>`, `allow_transformation_script: Option<bool>`, `auto_copy_target_container_id: Option<String>`, `available_detection_methods: Option<Vec<String>>`, `selected_detection_method: Option<String>`, `available_metadata_input_formats: Option<HashMap<String, String>>`, `selected_metadata_input_format: Option<String>`, `available_plate_templates: Option<Vec<String>>`, `selected_plate_template: Option<String>`, `allow_plate_metadata: Option<bool>`, `plate_metadata: Option<bool>`, `protocol_parameters: Option<HashMap<String, String>>`, `protocol_transform_scripts: Option<Vec<String>>`. This type needs both `Serialize` and `Deserialize` since it's used as both input (save_protocol) and output (get_protocol). Mark `#[non_exhaustive]`.


### Module: Pipeline (src/pipeline.rs — new file)

**get_file_status** — POST `pipeline-analysis-getFileStatus.api`
- Required: `files: Vec<String>`, `path: String`, `protocol_name: String`, `task_id: String`
- Optional: `container_path`
- Params sent as query params (not JSON body)
- Timeout: very long (60000000ms in JS client)
- Response: `GetFileStatusResponse` — `files: Vec<serde_json::Value>`, `submit_type: serde_json::Value`

**get_pipeline_container** — GET `pipeline-getPipelineContainer.api`
- Optional: `container_path`
- Response: `PipelineContainerResponse` — `container_path: String`, `web_dav_url: String`

**get_protocols** — POST `pipeline-analysis-getSavedProtocols.api`
- Required: `path: String`, `task_id: String`
- Optional: `container_path`, `include_workbooks: bool`
- Params sent as query params
- Response: `GetProtocolsResponse` — `protocols: Vec<serde_json::Value>`, `default_protocol_name: String`

**start_analysis** — POST `pipeline-analysis-startAnalysis.api`
- Required: `files: Vec<String>`, `path: String`, `protocol_name: String`, `task_id: String`
- Optional: `container_path`, `file_ids: Vec<i64>`, `json_parameters: serde_json::Value`, `xml_parameters: String`, `protocol_description: String`, `pipeline_description: String`, `save_protocol: bool`, `allow_non_existent_files: bool`
- Params sent as query params. `xml_parameters` → `configureXml`, `json_parameters` → `configureJson` (JSON-encoded string)
- Timeout: very long
- Response: `serde_json::Value`


### Module: Report (src/report.rs — new file)

**create_session** — POST `reports-createSession.api`
- Required: `client_context: serde_json::Value`
- Optional: `container_path`
- JSON body: `{ clientContext }`
- Response: `CreateSessionResponse` — `report_session_id: String`

**delete_session** — POST `reports-deleteSession.api`
- Required: `report_session_id: String`
- Optional: `container_path`
- Params sent as query params: `{ reportSessionId }`
- Response: `serde_json::Value`

**execute** — POST `reports-execute.api`
- Required: one of `report_id: String` or `report_name: String`
- Optional: `container_path`, `report_session_id: String`, `schema_name: String`, `query_name: String`, `input_params: HashMap<String, serde_json::Value>`
- JSON body: `{ reportId, reportName, schemaName, queryName, reportSessionId, "inputParams[key]": value... }` (input params are flattened with bracket notation)
- Response: `ExecuteResponse` — `console: Vec<String>`, `errors: Vec<String>`, `output_params: Vec<OutputParam>` where `OutputParam` has `name`, `type_` (enum: text, json, etc.), `value: serde_json::Value` (JSON type values are decoded from string)

**execute_function** — POST `reports-execute.api` (same endpoint)
- Required: `function_name: String`
- Optional: `container_path`, `report_session_id: String`, `input_params: HashMap<String, serde_json::Value>`
- JSON body: `{ functionName, reportSessionId, "inputParams[key]": value... }`
- Response: same as `execute`

**get_sessions** — POST `reports-getSessions.api`
- Optional: `container_path`
- Response: `GetSessionsResponse` — `report_sessions: Vec<serde_json::Value>`


### Module: Message (src/message.rs — new file)

**send_message** — POST `announcements-sendMessage.api`
- Optional: `msg_from: String`, `msg_recipients: Vec<Recipient>`, `msg_content: Vec<MsgContent>`, `msg_subject: String`, `container_path`
- JSON body: `{ msgFrom, msgRecipients, msgContent, msgSubject }`
- Response: `serde_json::Value`

**Recipient**: `address: String`, `type_: RecipientType` (enum: To, Cc, Bcc)

**MsgContent**: `content: String`, `type_: ContentType` (enum: TextPlain, TextHtml — serialized as `"text/plain"`, `"text/html"`)


### Module: Specimen (src/specimen.rs — new file)

All endpoints use the `specimen-api` controller and POST method.

**add_specimens_to_request** — POST `specimen-api-addSpecimensToRequest.api`
- Required: `request_id: i64`, `specimen_hashes: Vec<serde_json::Value>`, `preferred_location: i64`
- Optional: `container_path`
- JSON body: `{ requestId, specimenHashes, preferredLocation }`
- Response: `serde_json::Value`

**add_vials_to_request** — POST `specimen-api-addVialsToRequest.api`
- Required: `request_id: i64`, `vial_ids: Vec<serde_json::Value>`
- Optional: `container_path`, `id_type: String` (default "GlobalUniqueId")
- JSON body: `{ requestId, vialIds, idType }`
- Response: `serde_json::Value`

**cancel_request** — POST `specimen-api-cancelRequest.api`
- Required: `request_id: i64`
- Optional: `container_path`
- JSON body: `{ requestId }`
- Response: `serde_json::Value`

**get_open_requests** — POST `specimen-api-getOpenRequests.api`
- Optional: `all_users: bool`, `container_path`
- JSON body: `{ allUsers }`
- Response: `serde_json::Value` (extracted from `response.requests`)

**get_providing_locations** — POST `specimen-api-getProvidingLocations.api`
- Required: `specimen_hashes: Vec<String>`
- Optional: `container_path`
- JSON body: `{ specimenHashes }`
- Response: `serde_json::Value` (extracted from `response.locations`)

**get_repositories** — POST `specimen-api-getRepositories.api`
- Optional: `container_path`
- No JSON body (but sends Content-Type: application/json header)
- Response: `serde_json::Value` (extracted from `response.repositories`)

**get_request** — POST `specimen-api-getRequest.api`
- Required: `request_id: i64`
- Optional: `container_path`
- JSON body: `{ requestId }`
- Response: `serde_json::Value` (extracted from `response.request`)

**get_specimen_web_part_groups** — POST `specimen-api-getSpecimenWebPartGroups.api`
- Optional: `container_path`
- No JSON body
- Response: `serde_json::Value`

**get_vials_by_row_id** — POST `specimen-api-getVialsByRowId.api`
- Required: `vial_row_ids: Vec<i64>`
- Optional: `container_path`
- JSON body: `{ rowIds }`
- Response: `serde_json::Value` (extracted from `response.vials`)

**get_vial_type_summary** — POST `specimen-api-getVialTypeSummary.api`
- Optional: `container_path`
- No JSON body
- Response: `serde_json::Value`

**remove_vials_from_request** — POST `specimen-api-removeVialsFromRequest` (note: no `.api` suffix in JS source)
- Required: `request_id: i64`, `vial_ids: Vec<serde_json::Value>`
- Optional: `container_path`, `id_type: String` (default "GlobalUniqueId")
- JSON body: `{ requestId, vialIds, idType }`
- Response: `serde_json::Value`


### Module: Storage (src/storage.rs — new file)

All endpoints use the `storage` controller and POST method.

**StorageType** (enum): `Canister`, `Freezer`, `PhysicalLocation`, `PrimaryStorage`, `Rack`, `Shelf`, `StorageUnitType`, `TerminalStorageLocation` — serialized as their display names (e.g., `"Physical Location"`, `"Primary Storage"`, `"Storage Unit Type"`, `"Terminal Storage Location"`)

**StorageCommandResponse**: `success: bool`, `message: Option<String>`, `data: Option<serde_json::Value>`

**create_storage_item** — POST `storage-create.api`
- Required: `type_: StorageType`, `props: serde_json::Value`
- Optional: `container_path`
- JSON body: `{ type, props }`
- Response: `StorageCommandResponse`

**update_storage_item** — POST `storage-update.api`
- Required: `type_: StorageType`, `props: serde_json::Value` (must include `rowId`)
- Optional: `container_path`
- JSON body: `{ type, props }`
- Response: `StorageCommandResponse`

**delete_storage_item** — POST `storage-delete.api`
- Required: `type_: StorageType`, `row_id: i64`
- Optional: `container_path`
- JSON body: `{ type, props: { rowId } }`
- Response: `StorageCommandResponse`


### Module: Visualization (src/visualization.rs — new file)

**get_visualization** — POST `visualization-getVisualization.api`
- Optional: `name: String`, `report_id: String`, `query_name: String`, `schema_name: String`, `container_path`
- JSON body: `{ name, reportId, queryName, schemaName }`
- Response: `VisualizationResponse` — `id: String`, `name: String`, `description: String`, `type_: String`, `created_by: i64`, `owner_id: serde_json::Value`, `shared: bool`, `inheritable: bool`, `can_edit: bool`, `can_delete: bool`, `can_share: bool`, `schema_name: String`, `query_name: String`, `report_id: String`, `report_props: serde_json::Value`, `thumbnail_url: String`, `visualization_config: serde_json::Value` (decoded from JSON string)

**get_data** (visualization) — POST `visualization-getData` (no `.api` suffix)
- Required: `measures: Vec<serde_json::Value>`
- Optional: `container_path`, `filter_query: String`, `filter_url: String`, `sorts: serde_json::Value`, `limit: i32`, `group_bys: serde_json::Value`, `meta_data_only: bool`, `join_to_first: bool` (default false), `parameters: HashMap<String, String>`, `endpoint: Option<String>`
- Default URL: `build_url("visualization", "getData", container_path)` with `visualization.param.*` params appended. When `endpoint` is `Some(url)`, use that URL verbatim (with `visualization.param.*` params appended as query string) instead of building from controller/action.
- JSON body: `{ measures, sorts, filterQuery, filterUrl, limit, groupBys, metaDataOnly, joinToFirst }`. Measures are cloned and their `filterArray` entries are converted to URL-encoded filter strings (e.g., `encodeURIComponent(paramName) + "=" + encodeURIComponent(value)`).
- Response: `serde_json::Value`

**get_measures** — GET `visualization-getMeasures.api`
- Optional: `filters: Vec<serde_json::Value>`, `date_measures: bool`, `all_columns: bool`, `show_hidden: bool`, `container_path`
- Response: `Vec<serde_json::Value>` (extracted from `response.measures`)

**get_types** — GET `visualization-getVisualizationTypes.api`
- Optional: `container_path`
- Response: `Vec<serde_json::Value>` (extracted from `response.types`)

**save_visualization** — POST `visualization-saveVisualization.api`
- Required: `name: String`, `type_: String`, `visualization_config: serde_json::Value`
- Optional: `description: String`, `replace: bool`, `shared: bool`, `thumbnail_type: IconType`, `icon_type: IconType`, `svg: String`, `schema_name: String`, `query_name: String`, `container_path`
- JSON body: `{ name, description, json: encode(visualizationConfig), replace, shared, thumbnailType, iconType, svg, type, schemaName, queryName }`
- Response: `SaveVisualizationResponse` — `name: String`, `visualization_id: i64`

**get_dimensions** — GET `visualization-getDimensions.api`
- Required: `query_name: String`, `schema_name: String`
- Optional: `container_path`, `include_demographics: bool`
- Response: `Vec<Dimension>` (extracted from `response.dimensions`) where `Dimension` has `name: String`, `label: String`, `description: String`, `query_name: String`, `schema_name: String`, `type_: String`, `is_user_defined: bool`

**get_dimension_values** — GET `visualization-getDimensionValues.api`
- Required: `name: String`, `query_name: String`, `schema_name: String`
- Optional: `container_path`
- Response: `Vec<serde_json::Value>` (extracted from `response.values` when `response.success` is true)

**Measure** (response type, same shape as Dimension): `name: String`, `label: String`, `description: String`, `query_name: String`, `schema_name: String`, `type_: String`, `is_user_defined: bool`. Used by `get_measures` response.

**IconType** (enum): `Auto`, `Custom`, `None` — serialized as `"AUTO"`, `"CUSTOM"`, `"NONE"`


### Module: ParticipantGroup (src/participant_group.rs — new file)

**update_participant_group** — POST `participant-group-updateParticipantGroup.api`
- Required: `row_id: i64`
- Optional: `container_path`, `participant_ids: Vec<String>`, `ensure_participant_ids: Vec<String>`, `delete_participant_ids: Vec<String>`, `label: String`, `description: String`, `filters: serde_json::Value`
- JSON body: `{ rowId, participantIds, ensureParticipantIds, deleteParticipantIds, label, description, filters }`
- Response: `serde_json::Value` (extracted from `response.group`)


### Module: List (convenience wrapper, in src/list.rs)

**create_list** — convenience wrapper around `create_domain`
- Required: `name: String`, `key_name: String`, `key_type: ListKeyType` (enum: IntList, VarList — maps to `DomainKind`)
- Optional: `domain_design: DomainDesign`, `options: serde_json::Value`, `container_path`
- Implementation: constructs a `CreateDomainOptions` with `kind` set based on `key_type`, sets `domain_design.name` and `options.keyName`, then delegates to `create_domain`.
- No direct HTTP call.


### Module: Data Integration (src/di.rs — new file, Java-only)

All endpoints use the `dataintegration` controller and POST method.

**run_transform** — POST `dataintegration-runTransform`
- Required: `transform_id: String`
- Optional: `container_path`
- JSON body: `{ transformId }`
- Response: `RunTransformResponse` — `success: bool`, `job_id: String`, `pipeline_url: String`, `status: String`

**reset_transform_state** — POST `dataintegration-resetTransformState`
- Required: `transform_id: String`
- Optional: `container_path`
- JSON body: `{ transformId }`
- Response: `ResetTransformStateResponse` — `success: bool`

**update_transform_configuration** — POST `dataintegration-UpdateTransformConfiguration`
- Required: `transform_id: String`
- Optional: `enabled: bool`, `verbose_logging: bool`, `container_path`
- JSON body: `{ transformId, enabled, verboseLogging }`
- Response: `UpdateTransformConfigurationResponse` — `success: bool`, `result: TransformConfig` where `TransformConfig` has `enabled: bool`, `verbose_logging: bool`, `state: serde_json::Value`, `last_checked: String`, `description_id: String`


### Client-Level Features (src/client.rs enhancements)

These features are distributed across two commits to avoid the overlap identified in review:

**Commit 6** (request infrastructure): `RequestOptions` struct, `get_with_options`/`post_with_options`/`post_multipart` helpers, per-request timeout, per-request redirect handling, accepted status codes. These are internal plumbing that later endpoints depend on.

**Commit 25** (connection configuration): `user_agent`, `proxy_url`, `accept_self_signed_certs` fields on `ClientConfig`. These are user-facing configuration that doesn't block any endpoint implementation.


## Features We Skip

These are features from the JS client that don't make sense in a Rust library:

- **MultiRequest** — client-side request batching with callbacks. Rust users can use `tokio::join!` or `futures::join_all`.
- **getFromUrl** (Visualization) — reads URL parameters from the browser's current page.
- **getHomeContainer / getSharedContainer** — reads server context from the browser page DOM.
- **exportRuns** (Experiment) — DOM-dependent, submits a hidden form.
- **ensureLogin with useSiteLoginPage** — browser redirect to login page.
- **Deprecated wrappers** — `Assay.getAll`, `getById`, `getByName`, `getByType` (all delegate to `getAssays`). We provide only `get_assays`.
- **Domain.get** — deprecated in favor of `getDomainDetails`. We implement it but mark it `#[deprecated]`.
- **getSchemaPermissions** — a thin wrapper around `getSecurableResources` with `includeEffectivePermissions: true`. We can provide it as a convenience method but it's not a separate endpoint.


## Commit Plan

The commits are ordered by dependency: shared types first, then modules that depend on them, then modules that are independent. Each commit is self-contained, builds, passes all checks, and is bisect-friendly. Review iterations are interspersed at key milestones to catch drift before it compounds.

There are 30 code commits (numbered 5–34) and 4 review gate iterations (A–D), for 34 total stories. Commits 1–4 are already on `main`.


### Commit 5: Semver housekeeping and shared types

This commit makes the one-time breaking changes we need before 1.0, and introduces the narrow shared module.

Modify `src/client.rs`:
- Add `#[non_exhaustive]` to `ClientConfig`. This is a one-time break — anyone constructing `ClientConfig` via struct literal will need to add `..` or switch to a builder. This must happen now because we will add fields (user_agent, proxy, etc.) later and those additions must not be breaking.

Modify `src/filter.rs`:
- Add `#[non_exhaustive]` to `FilterType` and `ContainerFilter`. Same rationale: the server can add new filter types and container filter scopes, and we need to be able to add variants without breaking downstream.

Modify `src/error.rs`:
- Add `InvalidInput(String)` variant to `LabkeyError` for client-side validation errors (e.g., mutual exclusivity constraints per Pattern 14).

Create `src/common.rs`:
- `AuditBehavior` enum (`#[non_exhaustive]`, derives: `Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize`)
- Nothing else goes here yet. `CommandType` stays in `query.rs`. `opt()` and `container_filter_to_string` stay in `query.rs` until a second module needs them.

Modify `src/lib.rs`:
- Add `pub mod common;`
- Do NOT re-export `AuditBehavior` at the crate root. Users access it as `labkey_rs::common::AuditBehavior`.

Tests:
- `variant_count` test for `AuditBehavior` (assert the number of variants matches expected count, so adding a variant forces a test update)
- Verify `AuditBehavior` serde round-trips for all variants: `None` ↔ `"NONE"`, `Summary` ↔ `"SUMMARY"`, `Detailed` ↔ `"DETAILED"`
- Verify `ClientConfig` can still be constructed (with `..` if needed after `#[non_exhaustive]` — actually, `#[non_exhaustive]` on a struct only prevents construction outside the defining crate, so our own tests still work fine; the test should verify this)


### Commit 6: Client request infrastructure (RequestOptions, multipart, timeout)

This commit adds the internal request plumbing that later endpoints depend on. No connection-level config changes (user_agent, proxy, self-signed certs come later).

Modify `src/client.rs`:
- Add `pub(crate) struct RequestOptions` with `timeout: Option<Duration>`, `no_follow_redirects: bool`, `accepted_statuses: Vec<reqwest::StatusCode>`. Derive `Debug, Default`.
- Add `pub(crate) async fn get_with_options<T>(&self, url: Url, params: &[(String, String)], options: RequestOptions) -> Result<T, LabkeyError>` — applies timeout via `reqwest::RequestBuilder::timeout()`, handles `no_follow_redirects` by building a one-off `reqwest::Client` with `redirect::Policy::none()`, checks `accepted_statuses` in addition to `is_success()` in response handling.
- Add `pub(crate) async fn post_with_options<B, T>(&self, url: Url, body: &B, options: RequestOptions) -> Result<T, LabkeyError>` — same pattern.
- Add `pub(crate) async fn post_multipart<T>(&self, url: Url, form: reqwest::multipart::Form, options: RequestOptions) -> Result<T, LabkeyError>` — sends multipart form data instead of JSON.
- Refactor existing `get` and `post` to delegate to `get_with_options`/`post_with_options` with `RequestOptions::default()`.
- Add `reqwest` feature `multipart` to `Cargo.toml` if not already present.

Tests:
- Unit test: `RequestOptions::default()` has `timeout: None`, `no_follow_redirects: false`, empty `accepted_statuses`
- Unit test: verify that the refactored `get`/`post` still produce the same URLs and headers (existing tests should continue passing)
- Note: we cannot unit-test actual timeout behavior or redirect handling without a mock server. That comes in the integration test commit.


### Commit 7: Integration test infrastructure

Create `tests/` directory with shared test utilities. This commit introduces the mock server layer that later commits depend on for verifying HTTP behavior (multipart, redirects, error mapping).

Create `tests/common/mod.rs`:
- `test_client(mock_server_url: &str) -> LabkeyClient` helper
- `fixture<T: DeserializeOwned>(name: &str) -> T` helper that reads from `tests/fixtures/{name}.json` and deserializes
- Re-export commonly used types

Add `wiremock` (or `mockito`) to `[dev-dependencies]` in `Cargo.toml`.

Create `tests/client_integration.rs`:
- Test: GET request sends correct URL path, query params, auth header, and `X-Requested-With` header
- Test: POST request sends correct URL path, JSON body, auth header
- Test: non-2xx response is parsed as `LabkeyError::Api` when body is valid `ApiErrorBody`
- Test: non-2xx response with non-JSON body returns `LabkeyError::UnexpectedResponse`
- Test: `RequestOptions` with timeout is applied (verify the request is sent; timeout behavior itself is hard to test)
- Test: `post_multipart` sends multipart content type and form parts

Create `tests/fixtures/` directory (empty for now; fixtures added by later commits).

Update `.gitignore` to allow `tests/` files.


### Commit 8: Query mutation endpoints (insert, update, delete, truncate)

All four endpoints share `ModifyRowsResults` and `MutateRowsBody`. They must be implemented together as a cohesive unit — no duplicate body-shaping logic per endpoint unless required by an API difference.

Add to `src/query.rs`:
- `ModifyRowsResults` response type (`#[non_exhaustive]`, derives: `Debug, Clone, Deserialize`)
- `MutateRowsBody` internal struct (`#[derive(Serialize)]`, `#[serde(skip_serializing_if)]` on optional fields) for the shared JSON body shape
- `InsertRowsOptions`, `UpdateRowsOptions`, `DeleteRowsOptions`, `TruncateTableOptions` (all with bon builders, `#[non_exhaustive]`)
- `insert_rows`, `update_rows`, `delete_rows`, `truncate_table` methods on `LabkeyClient`

Endpoint contract table:

| Method | Action | Required body keys | Optional body keys |
|--------|--------|-------------------|-------------------|
| POST | `query-insertRows.api` | `schemaName`, `queryName`, `rows` | `transacted`, `extraContext`, `auditBehavior`, `auditDetails`, `auditUserComment`, `skipReselectRows` |
| POST | `query-updateRows.api` | `schemaName`, `queryName`, `rows` | (same as insert) |
| POST | `query-deleteRows.api` | `schemaName`, `queryName`, `rows` | (same as insert) |
| POST | `query-truncateTable.api` | `schemaName`, `queryName` | (same as insert, `rows` optional) |

Tests (per-endpoint checklist):
- URL construction: verify controller/action for each of the four endpoints
- JSON body contract: assert required fields present, optional `None` fields omitted, `auditBehavior` serializes correctly when set
- `ModifyRowsResults` deserialization from realistic fixture (with rows, errors, rowsAffected)
- Edge case: empty rows array, zero rowsAffected


### Commit 9: Query mutation endpoints (move_rows, save_rows)

Add to `src/query.rs`:
- `MoveRowsOptions`, `MoveRowsResponse` (`#[non_exhaustive]`)
- `SaveRowsOptions`, `SaveRowsCommand`, `SaveRowsResponse` (`#[non_exhaustive]`)
- `CommandType` enum (Insert, Update, Delete) — `#[non_exhaustive]`, stays in `query.rs`
- `move_rows`, `save_rows` methods (both JSON-only, no multipart)

Tests:
- URL construction for both endpoints
- `MoveRowsResponse` deserialization (extends `ModifyRowsResults` with `success`, `update_counts`)
- `SaveRowsCommand` serialization: verify `command` field uses `CommandType` wire format
- `CommandType` serde round-trip for all variants, `variant_count` test
- `SaveRowsResponse` deserialization with `committed`, `error_count`, nested `result` array
- Edge case: `save_rows` with empty commands array


### Commit 10: Query read endpoints (select_distinct, get_query_details)

Add to `src/query.rs`:
- `SelectDistinctOptions`, `SelectDistinctResponse` (`#[non_exhaustive]`)
- `GetQueryDetailsOptions`, `QueryDetailsResponse` and all supporting types: `QueryDetailsColumn` (full field set), `QueryLookup`, `QueryView`, `QueryViewColumn`, `QueryViewFilter`, `QueryViewSort`, `QueryImportTemplate`, `QueryIndex` — all `#[non_exhaustive]`
- `select_distinct_rows`, `get_query_details` methods

Tests:
- URL and param construction for both endpoints
- `SelectDistinctResponse` deserialization
- `QueryDetailsResponse` deserialization from a realistic fixture (create `tests/fixtures/query_details.json` with nested columns, lookup, views)
- `QueryDetailsColumn` deserialization with all optional fields absent vs. present
- Edge case: `select_distinct` with `max_rows: -1` sends `showRows=all`


### Commit 11: Query schema and query list endpoints

This is the first of three commits splitting the query metadata surface. This commit adds the two schema/query listing endpoints.

Add to `src/query.rs`:
- `GetQueriesOptions`, `GetQueriesResponse`, `QueryInfo` (`#[non_exhaustive]`)
- `GetSchemasOptions`
- `get_queries`, `get_schemas` methods

Tests:
- URL construction for both endpoints (`query-getQueries.api`, `query-getSchemas.api`)
- `GetQueriesResponse` deserialization with nested `queries` array containing `QueryInfo` objects
- `get_queries` param construction: verify `includeColumns`, `includeSystemQueries`, `includeTitle`, `includeUserQueries`, `includeViewDataUrl`, `queryDetailColumns` param names
- `get_schemas` returns `serde_json::Value` (the server returns a JSON object keyed by schema name; verify deserialization of a representative fixture)


### Commit 12: Query view management endpoints

This commit adds the four view management endpoints. These share a thematic cluster but have different HTTP methods and body shapes, so pay close attention to the per-endpoint details.

Add to `src/query.rs`:
- `GetQueryViewsOptions`, `SaveQueryViewsOptions`, `SaveSessionViewOptions`, `DeleteQueryViewOptions`
- `get_query_views`, `save_query_views`, `save_session_view`, `delete_query_view` methods

Special semantics to implement correctly:
- `save_session_view` JSON body uses `query.queryName` and `query.viewName` as literal key names (the `query.` prefix is part of the key, not a nested object). This is the data region prefix pattern.
- `delete_query_view` JSON body includes a `complete` field whose value is `!revert`. The `complete` field is only sent when `revert` is explicitly set by the caller. When `revert` is `None`, omit `complete` entirely.

Tests:
- URL construction for all four endpoints (`query-getQueryViews.api`, `query-saveQueryViews.api`, `query-saveSessionView.api`, `query-deleteView.api`)
- `save_session_view` JSON body: verify `query.queryName` and `query.viewName` key format (the `query.` prefix)
- `delete_query_view` JSON body: verify `complete` field logic (sent as `!revert` only when `revert` is explicitly set; omitted when `revert` is `None`)
- `save_query_views` JSON body: verify booleans (`shared`, `session`, `hidden`) only sent when true


### Commit 13: Query miscellaneous endpoints and utilities

This commit adds the remaining query endpoints that don't fit the schema/view clusters, plus the SQL literal utility functions and the URL column prefix constant.

Add to `src/query.rs`:
- `GetDataViewsOptions`, `DataViewType` enum (`#[non_exhaustive]`)
- `ValidateQueryOptions`, `ValidateQueryResponse` (`#[non_exhaustive]`)
- `GetServerDateResponse` (`#[non_exhaustive]`)
- `get_data_views`, `get_server_date`, `validate_query` methods
- `sql_date_literal`, `sql_date_time_literal`, `sql_string_literal` utility functions
- `URL_COLUMN_PREFIX` constant (`"_labkeyurl_"`)

Special semantics:
- `get_data_views` JSON body must always include `includeData: true` and `includeMetadata: false` regardless of caller input. The response envelope contains a `data` array that must be extracted (the JS client unwraps `response.data`).
- `validate_query` uses `query-validateQuery.api` by default, but when `validate_query_metadata` is true, switches to `query-validateQueryMetadata.api` instead.
- `get_server_date` takes no parameters and no container path — it calls `build_url('query', 'getServerDate.api')` with no container argument.
- SQL literal escape rules: `sql_string_literal` must escape single quotes by doubling them (`'` → `''`). Format: `sql_date_literal("2024-01-15")` → `{d '2024-01-15'}`, `sql_date_time_literal("2024-01-15 10:30:00")` → `{ts '2024-01-15 10:30:00'}`, `sql_string_literal("O'Brien")` → `{s 'O''Brien'}`.

Tests:
- URL construction for all three endpoints
- `get_data_views` JSON body: verify `includeData: true`, `includeMetadata: false` are always set
- `get_data_views` response extraction from `response.data` envelope
- `validate_query` action switching based on `validate_query_metadata` flag
- SQL literal formatting with escape cases
- `DataViewType` serde round-trips, `variant_count` for `DataViewType`
- `URL_COLUMN_PREFIX` value is `"_labkeyurl_"` (regression test)


### Commit 14: Query import_data (multipart, Java-only)

This commit adds the `import_data` endpoint, which is the first endpoint to use `post_multipart`. It is Java-only (not present in the JS client).

Add to `src/query.rs`:
- `ImportDataOptions`, `ImportDataSource` enum (Text, File, Path, ModuleResource), `InsertOption` enum (Import, Merge), `ImportDataResponse` — all `#[non_exhaustive]`
- `import_data` method using `post_multipart`

`ImportDataSource` enforces the "exactly one source" constraint at the type level (Pattern 14). The four variants are:
- `Text(String)` — inline TSV/CSV data, sent as multipart part named `text`
- `File { data: Vec<u8>, filename: String }` — uploaded file, sent as binary part named `file` with content type `application/octet-stream`
- `Path(String)` — server-side webdav path, sent as text part named `path`
- `ModuleResource { path: String, module: String }` — module resource, sent as text parts named `moduleResource` and `module`

Multipart field mapping:
- `schemaName` — text part (required)
- `queryName` — text part (required)
- Source part — one of `text`, `file`, `path`, or `moduleResource`+`module` (determined by `ImportDataSource` variant)
- `format` — text part (optional, e.g., "tsv", "csv")
- `insertOption` — text part (optional, "IMPORT" or "MERGE")
- `useAsync` — text part (optional, file-only)
- `saveToPipeline` — text part (optional, file-only)
- `importIdentity` — text part (optional)
- `importLookupByAlternateKey` — text part (optional)

Tests:
- Multipart form construction: verify part names match server expectations for each `ImportDataSource` variant
- `ImportDataResponse` deserialization (with and without `job_id`)
- `InsertOption` serde round-trips
- Type-level enforcement: verify that `ImportDataSource` makes it impossible to specify multiple sources


### Commit 15: Query get_data (getRawData)

This commit adds the `get_data` endpoint (aka `getRawData`), which has complex nested JSON body construction.

Add to `src/query.rs`:
- `GetDataOptions`, `GetDataSource`, `GetDataSourceType` enum (Query, Sql), `GetDataSort`, `GetDataTransform`, `GetDataFilter`, `GetDataAggregate`, `GetDataPivot` — all `#[non_exhaustive]`
- `get_data` method (POST `query-getData` — note: NO `.api` suffix)

The JSON body has a specific nested structure that the method must construct internally:
```json
{
  "source": { "type": "query"|"sql", "schemaName": "...", "queryName": "...", "sql": "...", "containerPath": "..." },
  "renderer": { "type": "json", "columns": [...], "includeDetailsColumn": ..., "maxRows": ..., "offset": ..., "sort": [...] },
  "transforms": [...],
  "pivot": { "by": [...], "columns": [...] }
}
```
The `renderer.type` field is always `"json"` — this is set internally, not exposed to callers. The `renderer` object is constructed in the method body from the user-facing options.

Tests:
- JSON body construction: verify `renderer.type` is always `"json"`
- Source fields are correct for Query mode (`type: "query"`, `schemaName`, `queryName`) vs Sql mode (`type: "sql"`, `schemaName`, `sql`)
- URL has no `.api` suffix: verify the built URL ends with `query-getData` not `query-getData.api`
- Response is `SelectRowsResponse` (same as `select_rows`)
- Optional fields (`transforms`, `pivot`, `sort`) are omitted from body when not provided


### REVIEW ITERATION A: Query module cross-reference

This is an audit gate, not a code commit. The agent's job is to verify parity between the implemented query module and the upstream reference implementations, then report findings. Scope is strictly limited: read upstream source, compare, report, and optionally make minimal fixes for documented discrepancies.

Read the upstream JS client's query module (`src/labkey/query/Utils.ts`, `src/labkey/query/GetData.ts`, `src/labkey/Query.ts`) and the Java client's query commands (`src/org/labkey/remoteapi/query/`). Produce an endpoint-by-endpoint parity checklist verifying:
- Every endpoint in the upstream clients is either implemented or explicitly listed in "Features We Skip"
- All parameter names match the wire format (camelCase in JSON, dot-notation for query params)
- All response field names match the server's JSON keys
- No endpoints were missed

Allowed outputs: (1) a brief discrepancy report in the story notes, and if no discrepancies are found, mark the story as passing; (2) if discrepancies are found, make minimal code fixes (parameter name corrections, missing fields, etc.) and document what was changed. Forbidden scope: no new endpoints, no refactors, no feature additions beyond fixing identified mismatches. If more than 5 discrepancies are found, document them all but only fix the most critical ones, and note the remainder for a follow-up.


### Commit 16: Security module scaffolding and shared types

This commit creates the security module structure and defines all shared types. It also moves the `opt()` and `container_filter_to_string` helpers from `query.rs` to `common.rs` since the security module is the first module outside `query` that needs them. No endpoint methods are added in this commit — it is a refactor-and-types-only commit to keep the helper move isolated and bisectable.

Create `src/security/mod.rs` (security as a directory module with submodules):
- `src/security/mod.rs` — module declarations, shared type re-exports
- `src/security/types.rs` — all shared security types: `Container`, `ContainerFormats`, `ContainerHierarchy`, `ModuleProperty`, `FolderType`, `FolderTypeWebPart`, `ModuleInfo`, `User`, `Group`, `Role`, `RolePermission`, `SecurableResource`, `Policy`, `PolicyAssignment` — all `#[non_exhaustive]`

Modify `src/common.rs`:
- Move `opt()` and `container_filter_to_string` from `query.rs` to `common.rs` as `pub(crate)` functions. This must be a behavior-preserving move: same function signatures, same semantics, no logic changes.

Modify `src/query.rs`:
- Remove `opt()` and `container_filter_to_string` definitions, replace with imports from `common`. All existing query module code must continue to work identically.

Modify `src/lib.rs`:
- Add `pub mod security;`

Tests:
- `Container` deserialization (verify `#[serde(rename = "type")] type_` field)
- `ContainerHierarchy` deserialization with nested `children` (recursive structure)
- `ContainerFormats` deserialization
- `User`, `Group`, `Role`, `Policy`, `SecurableResource` deserialization from minimal fixtures
- All existing query module tests must continue to pass unchanged (verifying the helper move was behavior-preserving)


### Commit 17: Security container endpoints — batch 1

Add to `src/security/`:
- `src/security/container.rs` — container endpoint option structs and the first four container methods: `create_container`, `delete_container`, `rename_container`, `get_containers`

`rename_container` must validate that at least one of `name` or `title` is provided (Pattern 14). Since both fields are simple scalars, validate in the method body and return `LabkeyError::InvalidInput` with a descriptive message if both are `None`.

`get_containers` has special response handling: when the server returns a single container (no `multipleContainers` param), wrap it in a one-element `Vec`. When the server returns `{ containers: [...] }` (multi mode), extract the inner array. The return type is always `Vec<ContainerHierarchy>`.

Tests:
- URL construction for all four endpoints (`core-createContainer.api`, `core-deleteContainer.api`, `admin-renameContainer.api`, `project-getContainers.api`)
- `get_containers` param construction: verify `multipleContainers=true` when multiple container IDs provided
- `get_containers` response handling: test both single-container and multi-container response shapes
- `rename_container` validation: verify `InvalidInput` error when both `name` and `title` are `None`
- JSON body construction for `create_container` and `rename_container`


### Commit 18: Security container endpoints — batch 2

Add to `src/security/container.rs`:
- `get_readable_containers`, `get_folder_types`, `get_modules`, `move_container` methods and their option structs

Tests:
- URL construction for all four endpoints (`project-getReadableContainers.api`, `core-getFolderTypes.api`, `admin-getModules.api`, `core-moveContainer.api`)
- `get_readable_containers` response extraction: verify inner `containers` field is extracted as `Vec<String>`
- `GetModulesResponse` deserialization with nested `ModuleInfo` array
- `FolderType` deserialization
- `move_container` JSON body: verify `addAlias`, `container`, `parent` fields


### Commit 19: Security module — user and group endpoints

Add to `src/security/`:
- `src/security/user.rs` — `create_new_user`, `get_users`, `get_users_with_permissions`, `ensure_login`
- `src/security/group.rs` — `create_group`, `delete_group`, `rename_group`, `add_group_members`, `remove_group_members`, `get_groups_for_current_user`
- All option structs with bon builders, all response types `#[non_exhaustive]`

Tests:
- `CreateNewUserResponse` deserialization (nested `users` array with `CreatedUser`)
- `GetUsersResponse` deserialization
- URL construction for all endpoints
- JSON body construction for `create_group` (verify `name` field, not `groupName`)
- `add_group_members` JSON body: verify `groupId` and `principalIds` fields


### Commit 20: Security module — permission and policy endpoints

Add to `src/security/`:
- `src/security/permission.rs` — `get_group_permissions`, `get_user_permissions`, `get_roles`, `get_securable_resources`
- `src/security/policy.rs` — `get_policy`, `save_policy`, `delete_policy`
- All option structs with bon builders, all response types `#[non_exhaustive]`

Tests:
- `GroupPermissionsResponse` deserialization with nested `PermissionsContainer` children
- `SecurableResource` deserialization (recursive `children`)
- `get_policy`: verify it uses POST (not GET), and JSON body contains `resourceId` field
- URL construction for all seven endpoints (note varying controllers: `security`)


### Commit 21: Security module — Java-only session and impersonation endpoints

These endpoints have unusual request patterns. Pay close attention to the per-endpoint behavior matrix below.

Add to `src/security/`:
- Add Java-only endpoints to a new `src/security/session.rs` submodule:
  - `logout` — POST `login-logout`, no JSON body, no parameters
  - `who_am_i` — GET `login-whoami.api`, no parameters
  - `delete_user` — POST `security-deleteUser` (note: no `.api` suffix), JSON body `{ id: userId }`
  - `impersonate_user` — POST `user-impersonateUser.api`, parameters sent as query params (NOT JSON body)
  - `stop_impersonating` — POST `login-stopImpersonating.api`, uses `RequestOptions { no_follow_redirects: true, accepted_statuses: vec![StatusCode::FOUND] }`

Request behavior matrix:

| Endpoint | Method | Controller | Action suffix | Body | Params | Special |
|----------|--------|------------|---------------|------|--------|---------|
| `logout` | POST | `login` | no `.api` | none | none | — |
| `who_am_i` | GET | `login` | `.api` | — | none | — |
| `delete_user` | POST | `security` | no `.api` | `{ id }` | none | — |
| `impersonate_user` | POST | `user` | `.api` | none | query params | `ImpersonateTarget` enum (Pattern 14) |
| `stop_impersonating` | POST | `login` | `.api` | none | none | 302 = success, disable redirects |

`impersonate_user` uses the `ImpersonateTarget` enum to enforce the "exactly one of user_id/email" constraint: `ImpersonateTarget::UserId(i64)` or `ImpersonateTarget::Email(String)`.

Tests:
- `stop_impersonating`: integration test with wiremock returning 302, verify it's treated as success
- `impersonate_user`: verify params sent as query params, not JSON body
- `WhoAmIResponse` deserialization
- URL construction for all five endpoints (note varying controllers: `login`, `user`, `security`)
- `logout`: verify POST with no body
- `delete_user`: verify URL has no `.api` suffix


### REVIEW ITERATION B: Security module cross-reference

This is an audit gate with the same scope rules as Review A.

Read the upstream JS client's security module (`src/labkey/Security.ts`) and Java client's security commands (`src/org/labkey/remoteapi/security/`) and verify completeness. Produce an endpoint-by-endpoint parity checklist. Check that all parameter names, response field names, and URL patterns match. Report discrepancies.

Allowed outputs: discrepancy report and minimal fixes. Forbidden scope: no new endpoints, no refactors beyond fixing identified mismatches. If more than 5 discrepancies, document all but only fix the most critical.


### Commit 22: Domain module

Create `src/domain.rs`:
- All domain types: `DomainDesign` (`#[non_exhaustive]`, needs both `Serialize` and `Deserialize` since it's used as input and output), `DomainField` (struct with many optional fields, `#[non_exhaustive]`), `DomainIndex`, `DomainKind` enum (`#[non_exhaustive]`), `PropertyUsage` (`#[non_exhaustive]`)
- All domain endpoints: `create_domain`, `get_domain` (mark `#[deprecated(note = "use get_domain_details")]`), `get_domain_details`, `save_domain`, `drop_domain`, `update_domain`, `list_domains`, `validate_name_expressions`, `get_domain_name_previews`, `get_properties`, `get_property_usages`
- Response types: `GetDomainDetailsResponse`, `ListDomainsResponse` — `#[non_exhaustive]`
- Envelope extraction for `get_property_usages` (extract `response.data`)

Tests:
- `DomainDesign` round-trip serialization/deserialization
- `DomainKind` serde round-trips for all variants, `variant_count` test
- URL construction for all eleven endpoints
- `get_property_usages` response extraction from envelope
- `drop_domain` JSON body construction


### Commit 23: Experiment module — types and lineage

Create `src/experiment.rs`:
- All shared experiment types: `ExpObject`, `RunGroup`, `Run`, `ExpData`, `Material`, `LineageNode`, `LineageEdge`, `PkFilter`, `DataClassRef`, `SampleSetRef` — all `#[non_exhaustive]`
- `ExpType` enum, `SeqType` enum — `#[non_exhaustive]`, `variant_count` tests
- Lineage endpoints: `lineage`, `resolve`
- Response types: `LineageResponse`, `ResolveResponse` — `#[non_exhaustive]`
- Envelope extraction for `resolve` (extract `response.data`)

Tests:
- `LineageNode` deserialization with nested `parents`/`children` edges
- `LineageResponse` deserialization with `nodes: HashMap<String, LineageNode>`
- `lineage` param construction: verify `lsids` becomes multiple `lsid` params, `expType` serialization
- Edge case: `lineage` with single LSID (deprecated `lsid` param vs. `lsids`)


### Commit 24: Experiment module — batch/run operations

Add to `src/experiment.rs`:
- Batch endpoints: `save_batch`, `save_batches`, `load_batch`, `load_batches`
- Run endpoints: `load_runs`, `save_runs`
- `create_hidden_run_group`
- `save_materials` (convenience wrapper that delegates to `insert_rows`)
- `set_entity_sequence`, `get_entity_sequence`

Tests:
- `RunGroup` serialization/deserialization round-trip
- `save_batch` JSON body: verify `batches: [batch]` wrapping (single batch wrapped in array)
- `load_batch` response extraction from `response.batch` envelope
- `load_batches` response extraction from `response.batches` envelope
- `load_runs` response extraction from `response.runs` envelope
- `save_materials` delegates correctly (verify it calls `insert_rows` with `schema_name: "Samples"`)
- `SeqType` serde round-trips


### Commit 25: Assay module — JS endpoints

Create `src/assay.rs`:
- JS endpoints: `get_assays`, `get_nab_runs`, `get_study_nab_graph_url`, `get_study_nab_runs`
- Types: `AssayDesign`, `AssayLink` enum, `FitType` enum — all `#[non_exhaustive]`

Tests:
- `AssayDesign` deserialization
- `get_assays` JSON body: verify `parameters` wrapper object
- `get_nab_runs` param construction: verify `query.sort`, `query.offset`, `query.maxRows` / `query.showRows` prefix pattern, filter encoding
- `FitType` serde round-trips, `variant_count`
- `AssayLink` serde round-trips, `variant_count`
- URL construction for all four endpoints


### Commit 26: Assay module — Java-only non-multipart endpoints

Add to `src/assay.rs`:
- `get_protocol`, `save_protocol`, `get_assay_run` methods
- `AssayProtocol` type (needs both `Serialize` and `Deserialize`, `#[non_exhaustive]`) — this is a large type with ~25 optional fields. All fields from the Java `Protocol.java` must be represented. See the Module-by-Module Feature Catalog for the full field list.
- `ProtocolIdentifier` enum for `get_protocol`'s "exactly one of provider_name/protocol_id" constraint (Pattern 14): `ProtocolIdentifier::ByProvider(String)` / `ProtocolIdentifier::ById { id: i64, copy: Option<bool> }`

Tests:
- `AssayProtocol` round-trip serialization/deserialization (verify all ~25 fields survive a round-trip)
- `get_protocol` param construction: verify `providerName` vs `protocolId` + `copy` modes via `ProtocolIdentifier` enum
- `save_protocol` JSON body: verify `AssayProtocol` is serialized directly as the body
- `get_assay_run` JSON body: verify `lsid` field
- URL construction for all three endpoints (note: no `.api` suffix on `assay-getProtocol` and `assay-saveProtocol`)


### Commit 27: Assay module — import_run (multipart)

This commit adds the `import_run` endpoint, which has the most complex multipart logic in the crate. It supports two serialization modes controlled by the `use_json` flag.

Add to `src/assay.rs`:
- `ImportRunOptions`, `ImportRunSource` enum, `ImportRunResponse` (`#[non_exhaustive]`)
- `import_run` method using `post_multipart`

`ImportRunSource` enforces the "exactly one of file/run_file_path/data_rows" constraint (Pattern 14):
- `File { data: Vec<u8>, filename: String }` — binary part named `file` with `application/octet-stream`
- `RunFilePath(String)` — text part named `runFilePath`
- `DataRows(Vec<serde_json::Value>)` — text part named `dataRows` containing JSON-encoded array

Multipart mode switch (`use_json` field on `ImportRunOptions`, default `false`):
- When `use_json` is `false` (default): each parameter is a separate text part. `properties` and `batch_properties` use bracket notation (`properties[key]`, `batchProperties[key]`). `dataRows` is sent as JSON text. `file` is a binary part.
- When `use_json` is `true`: everything goes into a single text part named `json` containing a JSON object with all parameters.

Tests:
- Multipart construction in `use_json: false` mode: verify part names, bracket notation for properties
- Multipart construction in `use_json: true` mode: verify single `json` part contains all parameters
- `ImportRunResponse` deserialization
- All three `ImportRunSource` variants produce correct part names
- URL construction: `assay-importRun.api`


### REVIEW ITERATION C: Experiment and assay cross-reference

This is an audit gate with the same scope rules as Reviews A and B.

Read the upstream JS client's experiment module (`src/labkey/Experiment.ts`) and Java client's assay commands (`src/org/labkey/remoteapi/assay/`) and verify completeness. Pay special attention to the `Protocol` type fields (verify all ~25 fields from Java `Protocol.java` are present in `AssayProtocol`) and the `ImportRunCommand` multipart format (verify bracket notation and mode switching). Produce an endpoint-by-endpoint parity checklist. Report discrepancies.

Allowed outputs: discrepancy report and minimal fixes. Forbidden scope: no new endpoints, no refactors beyond fixing identified mismatches.


### Commit 28: Pipeline module

Create `src/pipeline.rs`:
- `GetFileStatusOptions`, `GetFileStatusResponse`, `GetPipelineContainerOptions`, `PipelineContainerResponse`, `GetProtocolsOptions`, `GetProtocolsResponse`, `StartAnalysisOptions` — all `#[non_exhaustive]`
- All four endpoint methods
- Note: `get_file_status` and `start_analysis` send params as query params (not JSON body) and use very long timeouts

Tests:
- URL construction for all four endpoints (note `pipeline-analysis-` prefix on some)
- Param construction for `start_analysis`: verify `configureXml` and `configureJson` param names
- `PipelineContainerResponse` deserialization


### Commit 29: Report module

Create `src/report.rs`:
- `CreateSessionOptions`, `CreateSessionResponse`, `DeleteSessionOptions`, `ExecuteOptions`, `ExecuteFunctionOptions`, `ExecuteResponse`, `OutputParam`, `GetSessionsOptions`, `GetSessionsResponse` — all `#[non_exhaustive]`
- All five endpoint methods
- Input params flattening logic: `input_params` HashMap entries become `inputParams[key]` in the JSON body

Tests:
- Input params flattening: `{"x": 1, "y": "hello"}` → `{"inputParams[x]": 1, "inputParams[y]": "hello"}`
- `ExecuteResponse` deserialization with JSON-decoded output params (output params of type `json` have their `value` as a JSON string that needs decoding)
- `delete_session`: verify params sent as query params, not JSON body
- URL construction for all endpoints


### Commit 30: Message, Storage, and ParticipantGroup modules

These are small modules that can be done together.

Create `src/message.rs`:
- `SendMessageOptions`, `Recipient`, `RecipientType` enum (`#[non_exhaustive]`), `MsgContent`, `ContentType` enum (`#[non_exhaustive]`)
- `send_message` method

Create `src/storage.rs`:
- `StorageType` enum (`#[non_exhaustive]`, with display-name serialization), `StorageCommandResponse` (`#[non_exhaustive]`)
- `create_storage_item`, `update_storage_item`, `delete_storage_item`

Create `src/participant_group.rs`:
- `UpdateParticipantGroupOptions`
- `update_participant_group` method (response extracted from `response.group` envelope)

Tests:
- `RecipientType` serde: `To`, `Cc`, `Bcc`
- `ContentType` serde: `TextPlain` → `"text/plain"`, `TextHtml` → `"text/html"`
- `StorageType` serde: `PhysicalLocation` → `"Physical Location"`, `TerminalStorageLocation` → `"Terminal Storage Location"`, etc.
- `variant_count` for `StorageType`, `RecipientType`, `ContentType`
- `update_participant_group` response extraction from `response.group` envelope
- URL construction for all five endpoints


### Commit 31: Specimen module

Create `src/specimen.rs` with all 11 specimen endpoints. All endpoints use the `specimen-api` controller and POST method. Most return `serde_json::Value` responses, some with envelope extraction.

The 11 endpoints (all must be implemented — this is the complete list):
1. `add_specimens_to_request` — POST `specimen-api-addSpecimensToRequest.api`
2. `add_vials_to_request` — POST `specimen-api-addVialsToRequest.api`
3. `cancel_request` — POST `specimen-api-cancelRequest.api`
4. `get_open_requests` — POST `specimen-api-getOpenRequests.api` (extract `response.requests`)
5. `get_providing_locations` — POST `specimen-api-getProvidingLocations.api` (extract `response.locations`)
6. `get_repositories` — POST `specimen-api-getRepositories.api` (extract `response.repositories`, no JSON body but sends Content-Type header)
7. `get_request` — POST `specimen-api-getRequest.api` (extract `response.request`)
8. `get_specimen_web_part_groups` — POST `specimen-api-getSpecimenWebPartGroups.api` (no JSON body)
9. `get_vials_by_row_id` — POST `specimen-api-getVialsByRowId.api` (extract `response.vials`, body key is `rowIds`)
10. `get_vial_type_summary` — POST `specimen-api-getVialTypeSummary.api` (no JSON body)
11. `remove_vials_from_request` — POST `specimen-api-removeVialsFromRequest` (note: NO `.api` suffix)

No bespoke response types — all endpoints return `serde_json::Value` (with envelope extraction where noted above).

Tests:
- URL construction for all 11 endpoints (verify `specimen-api-` controller prefix)
- `remove_vials_from_request` URL: verify no `.api` suffix
- JSON body construction for representative endpoints (`add_specimens_to_request`, `get_vials_by_row_id`, `add_vials_to_request`)
- Envelope extraction for endpoints that extract (verify extraction works and missing field produces `LabkeyError::UnexpectedResponse`)
- `get_repositories` and `get_specimen_web_part_groups`: verify POST with no JSON body


### Commit 32: Visualization module

Create `src/visualization.rs`:
- `IconType` enum (`#[non_exhaustive]`), `Measure` type, `Dimension` type, `VisualizationResponse`, `SaveVisualizationResponse` — all `#[non_exhaustive]`
- `get_visualization`, `get_data` (visualization), `get_measures`, `get_types`, `save_visualization`, `get_dimensions`, `get_dimension_values`
- Parameter handling for `get_data`: `visualization.param.*` prefix, filter conversion on measures
- Envelope extraction: `get_measures` → `response.measures`, `get_types` → `response.types`, `get_dimensions` → `response.dimensions`, `get_dimension_values` → `response.values`

Special semantics for `get_data` (visualization):
- Default URL: `build_url("visualization", "getData", container_path)` with `visualization.param.*` params appended. Note: NO `.api` suffix.
- When `endpoint` is `Some(url)`, use that URL verbatim (with `visualization.param.*` params appended as query string) instead of building from controller/action.

Tests:
- `VisualizationResponse` deserialization (note: `visualization_config` is a JSON string that gets decoded)
- `save_visualization` JSON body: verify `json` field contains encoded `visualizationConfig`
- `get_data` param construction: verify `visualization.param.*` prefix
- `get_data` URL: verify no `.api` suffix, and custom `endpoint` URL is used verbatim when provided
- `Dimension` and `Measure` deserialization
- `get_dimension_values` response extraction (only when `response.success` is true)
- `IconType` serde round-trips, `variant_count`


### Commit 33: List and Data Integration modules

Create `src/list.rs`:
- `CreateListOptions`, `ListKeyType` enum (`#[non_exhaustive]`)
- `create_list` convenience method (delegates to `create_domain`)

Create `src/di.rs`:
- `RunTransformOptions`, `RunTransformResponse`, `ResetTransformStateOptions`, `ResetTransformStateResponse`, `UpdateTransformConfigurationOptions`, `UpdateTransformConfigurationResponse`, `TransformConfig` — all `#[non_exhaustive]`
- All three endpoint methods

Tests:
- `create_list` delegation: verify it constructs correct `CreateDomainOptions` with `kind` based on `ListKeyType`
- `ListKeyType` mapping: `IntList` → `DomainKind::IntList`, `VarList` → `DomainKind::VarList`
- DI response deserialization for all three endpoints
- DI URL construction (note: `dataintegration` controller, no `.api` suffix)


### REVIEW ITERATION D: Full API surface cross-reference

This is the final audit gate. Same scope rules as Reviews A–C.

The agent should:
1. List every endpoint in the JS client's public API (from `src/labkey/Query.ts`, `src/labkey/Security.ts`, `src/labkey/Domain.ts`, `src/labkey/Experiment.ts`, etc.) and verify each is either implemented or in "Features We Skip"
2. List every command class in the Java client (`src/org/labkey/remoteapi/`) and verify the same
3. Verify all `#[non_exhaustive]` annotations are in place per the policy
4. Verify all response types have the correct derive set per Pattern 11
5. Report any gaps or policy violations

Produce a comprehensive parity checklist. If discrepancies are found, make minimal fixes and document what was changed. If more than 5 discrepancies, document all but only fix the most critical.


### Commit 34: Client configuration enhancements

Modify `src/client.rs`:
- Add `user_agent: Option<String>` to `ClientConfig`. When `None`, defaults to `"labkey-rs/{version}"` (using `env!("CARGO_PKG_VERSION")`). When `Some`, uses the provided string.
- Add `accept_self_signed_certs: bool` to `ClientConfig`. Defaults to `false` (since `ClientConfig` is `#[non_exhaustive]` and constructed within the crate for tests, we can use `Default` or document the field default).
- Add `proxy_url: Option<String>` to `ClientConfig`. When `Some`, configures `reqwest::Proxy::all(url)`.
- Update `LabkeyClient::new` to configure the `reqwest::Client::builder()` with these options: `.user_agent(...)`, `.danger_accept_invalid_certs(...)`, `.proxy(...)`. The user agent is set ONLY on the `reqwest::Client::builder()` — do NOT also set it in `prepare_request`. One source of truth.
- Store the config values needed for one-off client rebuilds (for `no_follow_redirects` in `RequestOptions`).
- Since `ClientConfig` is `#[non_exhaustive]`, external users cannot construct it via struct literal. Provide a `ClientConfig::new(base_url, credential, container_path)` constructor that sets all other fields to defaults, plus builder-style setters (e.g., `with_user_agent`, `with_proxy_url`, `with_accept_self_signed_certs`). This is the canonical construction API.

Tests:
- Client construction with `user_agent` set — verify the reqwest client sends the custom UA
- Client construction with `accept_self_signed_certs: true` — verify it doesn't panic
- Client construction with `proxy_url` set — verify it doesn't panic
- Default user agent contains crate version string
- `ClientConfig::new` sets sensible defaults for all optional fields


## Notes for Ralph Loop Agents

Each commit description above tells you exactly which file to create or modify, which types to define, which methods to implement, and what tests to write. These are mandatory conventions — not suggestions.


### Construction Patterns

**Option structs**: always use bon builders, always mark `#[non_exhaustive]`. Derive `Debug, Clone`. Do NOT derive `Serialize` or `Deserialize` on option structs.

**Response types**: always mark `#[non_exhaustive]`. Derive `Debug, Clone, Deserialize`. Use `#[serde(rename_all = "camelCase")]`. Fields that may be absent should be `Option<T>` with `#[serde(default)]`. Do NOT derive `Serialize` unless the type is also used as input to another endpoint (e.g., `DomainDesign`, `AssayProtocol`).

**C-like enums**: derive `Debug, Clone, Copy, PartialEq, Eq`. Add `Serialize` and/or `Deserialize` as needed. Add `Hash` only when the enum will be used as a map key. Always `#[non_exhaustive]` for server-defined vocabularies.

**JSON body construction** for POST endpoints: use typed structs with `#[derive(Serialize)]` and `#[serde(skip_serializing_if = "Option::is_none")]`. This is the pattern established by `ExecuteSqlBody`. Do NOT use `serde_json::json!` imperatively.

**Parameter construction** for GET endpoints: use the declarative `opt()` helper and the `Option::map` + `flatten` pattern established in `select_rows`.

**Container path override**: every endpoint method accepts `container_path: Option<String>` via the options struct and passes it to `build_url`.

**Timeout**: endpoints that accept `timeout: Option<Duration>` construct a `RequestOptions { timeout, ..Default::default() }` and pass it to the `_with_options` variant.

**Nested response extraction**: use private envelope structs (Pattern 9). Extract the inner field and return `LabkeyError::UnexpectedResponse` if the expected field is absent. Never expose envelope structs in the public API. The rule: if the feature catalog entry says "extracted from `response.X`", the endpoint MUST use an envelope struct and return the inner type. If the entry says "Response: `serde_json::Value`" with no extraction note, return the full response `Value` as-is. Every endpoint that extracts is explicitly marked in the catalog — there are no implicit extractions.


### Testing Checklist (Per Endpoint)

Every endpoint implementation must include tests covering:

1. **URL construction**: verify controller and action (e.g., `query-insertRows.api`, `security-createGroup.api`). Use the existing `build_url` test pattern.
2. **Parameter/body construction**: for GET endpoints, verify query param keys and values. For POST endpoints, verify JSON body field names (camelCase), required fields present, optional `None` fields omitted via `skip_serializing_if`.
3. **Enum wire values**: every enum variant used in params or bodies must have a serde round-trip test verifying the exact string the server expects.
4. **Response deserialization**: deserialize from a realistic JSON fixture. Test both the happy path (all fields present) and the minimal path (only required fields).
5. **Envelope extraction**: if the endpoint extracts a nested field, test that extraction works and that a missing field produces `LabkeyError::UnexpectedResponse`.
6. **Edge cases**: empty arrays, `None` for optional fields, negative `max_rows` → `showRows=all`, etc.

For enum types, always include a `variant_count` test that asserts the number of variants. This forces a test update when variants are added, preventing silent omissions.


### Fixture Strategy

Use inline `serde_json::json!` for small fixtures (< 20 lines). For large realistic response fixtures (e.g., `QueryDetailsResponse`), create `tests/fixtures/{name}.json` files and load them with the `fixture<T>` helper from `tests/common/mod.rs`. Fixture files should contain realistic data from actual LabKey server responses when possible.


### Module Registration

When creating a new module file, add `pub mod {name};` to `src/lib.rs`. Do NOT re-export types at the crate root — keep the root minimal (`LabkeyClient`, `ClientConfig`, `Credential`, `LabkeyError`). Users access module types via `labkey_rs::query::SelectRowsOptions`, `labkey_rs::security::Container`, etc.


### Documentation

Every public type and method needs a doc comment. Endpoint methods should include:
- A brief description of what the endpoint does
- `# Errors` section listing error conditions
- `# Example` section with a `no_run` example (since examples need a LabKey server)

Prefer runnable doctests where possible. Use `no_run` over `ignore`. Use `ignore` only as a last resort for pseudocode.


### The `type` Field

Several response types have a `type` field, which is a reserved word in Rust. Use `#[serde(rename = "type")] pub type_: String` (or the appropriate type).


### The `query.` Prefix Pattern

Some endpoints use `query.queryName`, `query.viewName`, `query.sort` etc. as parameter names. This is the data region prefix pattern. Use the `dr` variable pattern from `select_rows`.


### File Upload

When implementing multipart endpoints, use `reqwest::multipart::Form` and `reqwest::multipart::Part`. Text fields become `Part::text(value)`, file data becomes `Part::bytes(data).file_name(name).mime_str("application/octet-stream")`.


### Review Iterations

Review iterations (A, B, C, D) are audit gates, not code commits. They are verification steps where the agent reads upstream source files and cross-references against the implemented code. The agent produces an endpoint-by-endpoint parity checklist and may make minimal fixes for documented discrepancies. These exist to catch drift before it compounds across later commits. Scope is strictly limited: no new endpoints, no refactors, no feature additions beyond fixing identified mismatches. If more than 5 discrepancies are found, document all but only fix the most critical ones, and note the remainder for follow-up.
