//! Assay models and API endpoints for assay listing and `NAb` queries.

use std::{collections::HashMap, time::Duration};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    client::{LabkeyClient, RequestOptions},
    common::opt,
    domain::DomainDesign,
    error::LabkeyError,
    experiment::{Run, RunGroup},
    filter::{Filter, encode_filters},
};

/// Link keys returned in [`AssayDesign::links`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[non_exhaustive]
pub enum AssayLink {
    /// Link to assay batches.
    #[serde(rename = "batches")]
    Batches,
    /// Link to the assay begin page.
    #[serde(rename = "begin")]
    Begin,
    /// Link to copy the assay design.
    #[serde(rename = "designCopy")]
    DesignCopy,
    /// Link to edit the assay design.
    #[serde(rename = "designEdit")]
    DesignEdit,
    /// Link to import data.
    #[serde(rename = "import")]
    Import,
    /// Link to a single result.
    #[serde(rename = "result")]
    Result,
    /// Link to assay results.
    #[serde(rename = "results")]
    Results,
    /// Link to assay runs.
    #[serde(rename = "runs")]
    Runs,
}

/// Fit type values used by `NAb` graph endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[non_exhaustive]
pub enum FitType {
    /// Five-parameter curve fit.
    #[serde(rename = "FIVE_PARAMETER")]
    FiveParameter,
    /// Four-parameter curve fit.
    #[serde(rename = "FOUR_PARAMETER")]
    FourParameter,
    /// Polynomial curve fit.
    #[serde(rename = "POLYNOMIAL")]
    Polynomial,
}

/// Assay design metadata returned by [`LabkeyClient::get_assays`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AssayDesign {
    /// The path to the container where the assay is defined.
    #[serde(default)]
    pub container_path: Option<String>,
    /// Assay description.
    #[serde(default)]
    pub description: Option<String>,
    /// Mapping from domain type to domain name.
    #[serde(default)]
    pub domain_types: HashMap<String, String>,
    /// Domain metadata keyed by domain name.
    #[serde(default)]
    pub domains: HashMap<String, serde_json::Value>,
    /// Assay id.
    pub id: i64,
    /// Import action name.
    #[serde(default)]
    pub import_action: Option<String>,
    /// Import controller name.
    #[serde(default)]
    pub import_controller: Option<String>,
    /// Assay links keyed by server link name.
    #[serde(default)]
    pub links: HashMap<String, String>,
    /// Assay name.
    pub name: String,
    /// Plate template name, if this assay is plate-based.
    #[serde(default)]
    pub plate_template: Option<String>,
    /// Whether this assay is project-level.
    #[serde(default)]
    pub project_level: Option<bool>,
    /// Protocol schema name.
    #[serde(default)]
    pub protocol_schema_name: Option<String>,
    /// URL to the assay import template.
    #[serde(default)]
    pub template_link: Option<String>,
    /// Assay type name.
    #[serde(rename = "type")]
    #[serde(default)]
    pub type_: Option<String>,
}

/// Options for [`LabkeyClient::get_assays`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetAssaysOptions {
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Filter assays by id.
    pub id: Option<i64>,
    /// Filter assays by name.
    pub name: Option<String>,
    /// Filter assays by plate-enabled flag.
    pub plate_enabled: Option<bool>,
    /// Filter assays by status.
    pub status: Option<String>,
    /// Filter assays by assay type.
    pub type_: Option<String>,
}

/// Options for [`LabkeyClient::get_nab_runs`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetNabRunsOptions {
    /// `NAb` assay design name.
    pub assay_name: String,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Whether neutralization should be calculated on the server.
    pub calculate_neut: Option<bool>,
    /// Filters to apply using existing query-style filter encoding.
    pub filter_array: Option<Vec<Filter>>,
    /// Whether curve fit parameters should be included.
    pub include_fit_parameters: Option<bool>,
    /// Whether run statistics should be included.
    pub include_stats: Option<bool>,
    /// Whether well-level data should be included.
    pub include_wells: Option<bool>,
    /// Maximum rows to return. Use a negative value for all rows.
    pub max_rows: Option<i32>,
    /// Row offset for pagination.
    pub offset: Option<i64>,
    /// Sort expression.
    pub sort: Option<String>,
    /// Optional request timeout.
    pub timeout: Option<Duration>,
}

/// Response payload from [`LabkeyClient::get_study_nab_graph_url`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StudyNabGraphUrlResponse {
    /// Object ids that were graphed.
    #[serde(default)]
    pub object_ids: Vec<serde_json::Value>,
    /// URL of the generated graph image.
    pub url: String,
}

/// Options for [`LabkeyClient::get_study_nab_graph_url`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetStudyNabGraphUrlOptions {
    /// Object ids for `NAb` summary rows copied to study.
    pub object_ids: Vec<String>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Optional chart title.
    pub chart_title: Option<String>,
    /// Optional fit type for curve generation.
    pub fit_type: Option<FitType>,
    /// Optional graph height in pixels.
    pub height: Option<i32>,
    /// Optional request timeout.
    pub timeout: Option<Duration>,
    /// Optional graph width in pixels.
    pub width: Option<i32>,
}

/// Options for [`LabkeyClient::get_study_nab_runs`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetStudyNabRunsOptions {
    /// Object ids for `NAb` summary rows copied to study.
    pub object_ids: Vec<String>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Whether neutralization should be calculated on the server.
    pub calculate_neut: Option<bool>,
    /// Whether fit parameters should be included.
    pub include_fit_parameters: Option<bool>,
    /// Whether run statistics should be included.
    pub include_stats: Option<bool>,
    /// Whether well-level data should be included.
    pub include_wells: Option<bool>,
    /// Optional request timeout.
    pub timeout: Option<Duration>,
}

/// Assay protocol definition used by Java-style protocol endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AssayProtocol {
    /// Protocol id.
    #[serde(default)]
    pub protocol_id: Option<i64>,
    /// Protocol name.
    pub name: String,
    /// Protocol description.
    #[serde(default)]
    pub description: Option<String>,
    /// Provider name (for example, `General`).
    pub provider_name: String,
    /// Domain definitions attached to the protocol.
    #[serde(default)]
    pub domains: Vec<DomainDesign>,
    /// Whether background upload can be enabled.
    #[serde(default)]
    pub allow_background_upload: Option<bool>,
    /// Whether background upload is enabled.
    #[serde(default)]
    pub background_upload: Option<bool>,
    /// Whether editable results can be enabled.
    #[serde(default)]
    pub allow_editable_results: Option<bool>,
    /// Whether results are editable.
    #[serde(default)]
    pub editable_results: Option<bool>,
    /// Whether runs are editable.
    #[serde(default)]
    pub editable_runs: Option<bool>,
    /// Whether transform script files should be saved.
    #[serde(default)]
    pub save_script_files: Option<bool>,
    /// Whether QC states can be enabled.
    #[serde(default)]
    pub allow_qc_states: Option<bool>,
    /// Whether QC is enabled.
    #[serde(default)]
    pub qc_enabled: Option<bool>,
    /// Whether spaces in path are allowed.
    #[serde(default)]
    pub allow_spaces_in_path: Option<bool>,
    /// Whether transformation scripts are allowed.
    #[serde(default)]
    pub allow_transformation_script: Option<bool>,
    /// Auto-copy target container id.
    #[serde(default)]
    pub auto_copy_target_container_id: Option<String>,
    /// Available detection methods.
    #[serde(default)]
    pub available_detection_methods: Vec<String>,
    /// Selected detection method.
    #[serde(default)]
    pub selected_detection_method: Option<String>,
    /// Available metadata input formats.
    #[serde(default)]
    pub available_metadata_input_formats: HashMap<String, String>,
    /// Selected metadata input format.
    #[serde(default)]
    pub selected_metadata_input_format: Option<String>,
    /// Available plate templates.
    #[serde(default)]
    pub available_plate_templates: Vec<String>,
    /// Selected plate template.
    #[serde(default)]
    pub selected_plate_template: Option<String>,
    /// Whether plate metadata can be enabled.
    #[serde(default)]
    pub allow_plate_metadata: Option<bool>,
    /// Whether plate metadata is enabled.
    #[serde(default)]
    pub plate_metadata: Option<bool>,
    /// Protocol parameter map.
    #[serde(default)]
    pub protocol_parameters: HashMap<String, String>,
    /// Protocol transform script names.
    #[serde(default)]
    pub protocol_transform_scripts: Vec<String>,
    /// Additional server-provided keys preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl AssayProtocol {
    /// Create a protocol payload with required fields and empty defaults.
    #[must_use]
    pub fn new(name: String, provider_name: String) -> Self {
        Self {
            protocol_id: None,
            name,
            description: None,
            provider_name,
            domains: Vec::new(),
            allow_background_upload: None,
            background_upload: None,
            allow_editable_results: None,
            editable_results: None,
            editable_runs: None,
            save_script_files: None,
            allow_qc_states: None,
            qc_enabled: None,
            allow_spaces_in_path: None,
            allow_transformation_script: None,
            auto_copy_target_container_id: None,
            available_detection_methods: Vec::new(),
            selected_detection_method: None,
            available_metadata_input_formats: HashMap::new(),
            selected_metadata_input_format: None,
            available_plate_templates: Vec::new(),
            selected_plate_template: None,
            allow_plate_metadata: None,
            plate_metadata: None,
            protocol_parameters: HashMap::new(),
            protocol_transform_scripts: Vec::new(),
            extra: HashMap::new(),
        }
    }
}

/// Identifier for [`LabkeyClient::get_protocol`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProtocolIdentifier {
    /// Fetch a protocol by provider name.
    ByProvider(String),
    /// Fetch a protocol by protocol id.
    ById {
        /// Protocol id.
        id: i64,
        /// Optional copy behavior.
        copy: Option<bool>,
    },
}

/// Options for [`LabkeyClient::get_protocol`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetProtocolOptions {
    /// Protocol identifier mode.
    pub identifier: ProtocolIdentifier,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::save_protocol`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SaveProtocolOptions {
    /// Protocol payload to save.
    pub protocol: AssayProtocol,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::get_assay_run`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetAssayRunOptions {
    /// Run LSID to fetch.
    pub lsid: String,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Source payload variants for [`LabkeyClient::import_run`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum ImportRunSource {
    /// Upload run data as file bytes.
    #[non_exhaustive]
    File {
        /// Raw file bytes.
        data: Vec<u8>,
        /// File name sent to the server.
        filename: String,
    },
    /// Import from a server-side run file path.
    RunFilePath(String),
    /// Import inline row data.
    DataRows(Vec<serde_json::Value>),
}

/// Options for [`LabkeyClient::import_run`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct ImportRunOptions {
    /// Assay id to import into.
    pub assay_id: i64,
    /// Exactly one run input mode.
    pub source: ImportRunSource,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Optional run name.
    pub name: Option<String>,
    /// Optional run comment.
    pub comment: Option<String>,
    /// Optional existing batch id.
    pub batch_id: Option<i64>,
    /// Optional rerun id.
    pub re_run_id: Option<i64>,
    /// Optional save-data-as-file flag.
    pub save_data_as_file: Option<bool>,
    /// Optional save-only-matching-columns flag.
    pub save_matching_column_data_only: Option<bool>,
    /// Optional async job description.
    pub job_description: Option<String>,
    /// Optional async job notification provider.
    pub job_notification_provider: Option<String>,
    /// Optional force-async flag.
    pub force_async: Option<bool>,
    /// Optional cross-run file-input flag.
    pub allow_cross_run_file_inputs: Option<bool>,
    /// Optional workflow task id.
    pub workflow_task: Option<i64>,
    /// Optional alternate-key lookup flag.
    pub allow_lookup_by_alternate_key: Option<bool>,
    /// Optional audit user comment.
    pub audit_user_comment: Option<String>,
    /// Optional structured audit details.
    pub audit_details: Option<serde_json::Value>,
    /// Optional run-level properties.
    pub properties: Option<HashMap<String, serde_json::Value>>,
    /// Optional batch-level properties.
    pub batch_properties: Option<HashMap<String, serde_json::Value>>,
    /// Optional plate metadata payload.
    pub plate_metadata: Option<serde_json::Value>,
    /// Optional JSON payload mode. Defaults to `false` when unset.
    pub use_json: Option<bool>,
}

/// Response from [`LabkeyClient::import_run`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ImportRunResponse {
    /// Whether the import request was accepted.
    pub success: bool,
    /// Imported run id when available.
    #[serde(default)]
    pub run_id: Option<i64>,
    /// Imported batch id when available.
    #[serde(default)]
    pub batch_id: Option<i64>,
    /// Pipeline job id for async imports.
    #[serde(default)]
    pub job_id: Option<String>,
    /// Redirect URL provided on successful import.
    #[serde(default, rename = "successurl")]
    pub success_url: Option<String>,
    /// Assay id associated with the import.
    #[serde(default)]
    pub assay_id: Option<i64>,
}

/// An assay batch, which groups related runs under a single experiment run
/// group. This is a type alias for [`RunGroup`] since the wire format is
/// identical — `RunGroup` already contains `ExpObject` fields plus
/// `runs: Vec<Run>`.
pub type Batch = RunGroup;

/// Identifies an assay for [`LabkeyClient::save_assay_batch`].
///
/// The server accepts either an integer assay ID or a protocol name string,
/// but not both. This enum makes the mutual exclusivity explicit.
#[derive(Debug, Clone)]
pub enum BatchIdentifier {
    /// Identify the assay by its numeric ID.
    ByAssayId(i64),
    /// Identify the assay by its protocol name. This mode supports
    /// non-assay-backed runs (e.g., `"Sample Derivation Protocol"`).
    ByProtocolName(String),
}

/// Options for [`LabkeyClient::get_assay_batch`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetAssayBatchOptions {
    /// Protocol name of the assay design.
    pub protocol_name: String,
    /// Server-assigned batch ID to load.
    pub batch_id: i64,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::save_assay_batch`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SaveAssayBatchOptions {
    /// Assay identifier — by numeric ID or protocol name.
    pub identifier: BatchIdentifier,
    /// The batch payload to save. If run/batch IDs are absent the server
    /// inserts new records; if present it updates existing ones.
    pub batch: Batch,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Response from [`LabkeyClient::save_assay_batch`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SaveAssayBatchResponse {
    /// The saved batch, including any server-assigned IDs.
    pub batch: Batch,
    /// The assay ID associated with the saved batch.
    #[serde(default)]
    pub assay_id: Option<i64>,
}

/// Options for [`LabkeyClient::save_assay_runs`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SaveAssayRunsOptions {
    /// Protocol name of the assay design.
    pub protocol_name: String,
    /// The runs to save. If run IDs are absent the server inserts new
    /// records; if present it updates existing ones.
    pub runs: Vec<Run>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Response from [`LabkeyClient::save_assay_runs`].
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct SaveAssayRunsResponse {
    /// The saved runs, including any server-assigned IDs.
    pub runs: Vec<Run>,
}

#[derive(Debug, Clone, Deserialize)]
struct GetAssaysResponse {
    definitions: Vec<AssayDesign>,
}

#[derive(Debug, Clone, Deserialize)]
struct NabRunsResponse {
    runs: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
struct ProtocolEnvelope {
    data: AssayProtocol,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetAssaysBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    plate_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    type_: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
struct GetAssayRunBody {
    lsid: String,
}

#[derive(Deserialize)]
struct GetAssayRunEnvelope {
    run: Run,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetAssayBatchBody {
    protocol_name: String,
    batch_id: i64,
}

#[derive(Deserialize)]
struct GetAssayBatchEnvelope {
    batch: Batch,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveAssayBatchBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    assay_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocol_name: Option<String>,
    batch: Batch,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveAssayRunsBody {
    protocol_name: String,
    runs: Vec<Run>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ImportRunPart {
    Text {
        name: String,
        value: String,
    },
    Json {
        name: String,
        value: String,
    },
    File {
        name: String,
        filename: String,
        data: Vec<u8>,
    },
}

impl LabkeyClient {
    /// Get assay designs in a container.
    ///
    /// Sends a POST request to `assay-assayList.api` with filter values as
    /// flat top-level fields in the JSON body.
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
    /// use labkey_rs::assay::GetAssaysOptions;
    ///
    /// let assays = client
    ///     .get_assays(
    ///         GetAssaysOptions::builder()
    ///             .type_("General".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{} assays", assays.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_assays(
        &self,
        options: GetAssaysOptions,
    ) -> Result<Vec<AssayDesign>, LabkeyError> {
        let url = self.build_url("assay", "assayList.api", options.container_path.as_deref());
        let body = build_get_assays_body(&options);
        let response: GetAssaysResponse = self.post(url, &body).await?;
        Ok(response.definitions)
    }

    /// Get `NAb` assay runs.
    ///
    /// Sends a GET request to `nabassay-getNabRuns.api` and encodes paging and
    /// sorting using `query.*` keys (`query.sort`, `query.offset`, and either
    /// `query.maxRows` or `query.showRows=all`).
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
    /// use labkey_rs::assay::GetNabRunsOptions;
    ///
    /// let runs = client
    ///     .get_nab_runs(
    ///         GetNabRunsOptions::builder()
    ///             .assay_name("Neutralization".to_string())
    ///             .max_rows(50)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{} runs", runs.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_nab_runs(
        &self,
        options: GetNabRunsOptions,
    ) -> Result<Vec<serde_json::Value>, LabkeyError> {
        validate_get_nab_runs_options(&options)?;
        let url = self.build_url(
            "nabassay",
            "getNabRuns.api",
            options.container_path.as_deref(),
        );
        let params = build_get_nab_runs_params(&options);
        let request_options = RequestOptions {
            timeout: options.timeout,
            ..RequestOptions::default()
        };
        let response: NabRunsResponse = self
            .get_with_options(url, &params, &request_options)
            .await?;
        Ok(response.runs)
    }

    /// Get the graph URL for study-linked `NAb` results.
    ///
    /// Sends a GET request to `nabassay-getStudyNabGraphURL.api`.
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
    /// use labkey_rs::assay::GetStudyNabGraphUrlOptions;
    ///
    /// let graph = client
    ///     .get_study_nab_graph_url(
    ///         GetStudyNabGraphUrlOptions::builder()
    ///             .object_ids(vec!["101".to_string(), "102".to_string()])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", graph.url);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_study_nab_graph_url(
        &self,
        options: GetStudyNabGraphUrlOptions,
    ) -> Result<StudyNabGraphUrlResponse, LabkeyError> {
        validate_object_ids("get_study_nab_graph_url", &options.object_ids)?;
        let url = self.build_url(
            "nabassay",
            "getStudyNabGraphURL.api",
            options.container_path.as_deref(),
        );
        let params = build_get_study_nab_graph_url_params(&options);
        let request_options = RequestOptions {
            timeout: options.timeout,
            ..RequestOptions::default()
        };
        self.get_with_options(url, &params, &request_options).await
    }

    /// Get study-linked `NAb` run details.
    ///
    /// Sends a GET request to `nabassay-getStudyNabRuns.api`.
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
    /// use labkey_rs::assay::GetStudyNabRunsOptions;
    ///
    /// let runs = client
    ///     .get_study_nab_runs(
    ///         GetStudyNabRunsOptions::builder()
    ///             .object_ids(vec!["101".to_string()])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{} runs", runs.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_study_nab_runs(
        &self,
        options: GetStudyNabRunsOptions,
    ) -> Result<Vec<serde_json::Value>, LabkeyError> {
        validate_object_ids("get_study_nab_runs", &options.object_ids)?;
        let url = self.build_url(
            "nabassay",
            "getStudyNabRuns.api",
            options.container_path.as_deref(),
        );
        let params = build_get_study_nab_runs_params(&options);
        let request_options = RequestOptions {
            timeout: options.timeout,
            ..RequestOptions::default()
        };
        let response: NabRunsResponse = self
            .get_with_options(url, &params, &request_options)
            .await?;
        Ok(response.runs)
    }

    /// Get an assay protocol definition.
    ///
    /// Sends a GET request to `assay-getProtocol` (no `.api` suffix) and extracts
    /// the protocol from `response.data`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the HTTP request fails, the server returns
    /// an error response, or the response does not include `data`.
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
    /// use labkey_rs::assay::{GetProtocolOptions, ProtocolIdentifier};
    ///
    /// let protocol = client
    ///     .get_protocol(
    ///         GetProtocolOptions::builder()
    ///             .identifier(ProtocolIdentifier::ByProvider("General".to_string()))
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", protocol.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_protocol(
        &self,
        options: GetProtocolOptions,
    ) -> Result<AssayProtocol, LabkeyError> {
        validate_get_protocol_options(&options)?;
        let url = self.build_url("assay", "getProtocol", options.container_path.as_deref());
        let params = build_get_protocol_params(&options.identifier);
        let response: serde_json::Value = self.get(url, &params).await?;
        extract_protocol_response("get_protocol", &response)
    }

    /// Save an assay protocol definition.
    ///
    /// Sends a POST request to `assay-saveProtocol` (no `.api` suffix) and extracts
    /// the saved protocol from `response.data`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the HTTP request fails, the server returns
    /// an error response, or the response does not include `data`.
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
    /// use labkey_rs::assay::{SaveProtocolOptions, AssayProtocol};
    ///
    /// let mut protocol = AssayProtocol::new("General".to_string(), "General".to_string());
    /// protocol.protocol_id = Some(10);
    ///
    /// let saved = client
    ///     .save_protocol(
    ///         SaveProtocolOptions::builder()
    ///             .protocol(protocol)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", saved.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_protocol(
        &self,
        options: SaveProtocolOptions,
    ) -> Result<AssayProtocol, LabkeyError> {
        let url = self.build_url("assay", "saveProtocol", options.container_path.as_deref());
        let response: serde_json::Value = self.post(url, &options.protocol).await?;
        extract_protocol_response("save_protocol", &response)
    }

    /// Get details for a single assay run.
    ///
    /// Sends a POST request to `assay-getAssayRun` (no `.api` suffix) with a
    /// typed body containing `lsid`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if `lsid` is blank, if the HTTP request fails,
    /// or if the server returns an error response.
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
    /// use labkey_rs::assay::GetAssayRunOptions;
    ///
    /// let run = client
    ///     .get_assay_run(
    ///         GetAssayRunOptions::builder()
    ///             .lsid("urn:lsid:labkey.com:AssayRun.Folder-1:7".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{:?}", run.exp_object.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_assay_run(&self, options: GetAssayRunOptions) -> Result<Run, LabkeyError> {
        validate_get_assay_run_options(&options)?;
        let url = self.build_url("assay", "getAssayRun", options.container_path.as_deref());
        let body = GetAssayRunBody { lsid: options.lsid };
        let envelope: GetAssayRunEnvelope = self.post(url, &body).await?;
        Ok(envelope.run)
    }

    /// Import an assay run using multipart upload modes.
    ///
    /// Sends a POST request to `assay-importRun.api` with multipart payload
    /// encoding. When `use_json` is unset or `false`, parameters are emitted as
    /// individual parts, including bracket-notation property keys
    /// (`properties['key']` and `batchProperties['key']`). When `use_json` is
    /// `true`, the request sends a `json` part with the structured payload; if
    /// the source is a [`ImportRunSource::File`], a separate `file` binary part
    /// is also included alongside the JSON part.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if multipart body construction fails, the HTTP
    /// request fails, the server returns an error response, or the response body
    /// cannot be deserialized.
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
    /// use labkey_rs::assay::{ImportRunOptions, ImportRunSource};
    ///
    /// let response = client
    ///     .import_run(
    ///         ImportRunOptions::builder()
    ///             .assay_id(42)
    ///             .source(ImportRunSource::RunFilePath(
    ///                 "/files/assays/run1.tsv".to_string(),
    ///             ))
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", response.success);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn import_run(
        &self,
        options: ImportRunOptions,
    ) -> Result<ImportRunResponse, LabkeyError> {
        validate_import_run_options(&options)?;
        let url = self.build_url("assay", "importRun.api", options.container_path.as_deref());
        let parts = build_import_run_parts(&options);
        let form = build_import_run_form(parts)?;

        self.post_multipart(url, form, &RequestOptions::default())
            .await
    }

    /// Load an assay batch by protocol name and batch ID.
    ///
    /// Sends a POST request to `assay-getAssayBatch.api` and extracts the
    /// `batch` object from the response envelope.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the protocol name is blank, the HTTP
    /// request fails, or the response cannot be deserialized.
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
    /// let batch = client
    ///     .get_assay_batch(
    ///         labkey_rs::assay::GetAssayBatchOptions::builder()
    ///             .protocol_name("General".to_string())
    ///             .batch_id(42)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{:?}", batch.exp_object.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_assay_batch(
        &self,
        options: GetAssayBatchOptions,
    ) -> Result<Batch, LabkeyError> {
        validate_get_assay_batch_options(&options)?;
        let url = self.build_url("assay", "getAssayBatch", options.container_path.as_deref());
        let body = GetAssayBatchBody {
            protocol_name: options.protocol_name,
            batch_id: options.batch_id,
        };
        let envelope: GetAssayBatchEnvelope = self.post(url, &body).await?;
        Ok(envelope.batch)
    }

    /// Save (insert or update) an assay batch.
    ///
    /// Sends a POST request to `assay-saveAssayBatch.api`. If the batch and
    /// its runs have no server-assigned IDs, the server inserts new records.
    /// If IDs are present, existing records are updated.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the identifier is invalid (blank protocol
    /// name or non-positive assay ID), the HTTP request fails, or the
    /// response cannot be deserialized.
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
    /// use labkey_rs::assay::{Batch, BatchIdentifier, SaveAssayBatchOptions};
    ///
    /// let batch: Batch = serde_json::from_value(serde_json::json!({}))?;
    /// let response = client
    ///     .save_assay_batch(
    ///         SaveAssayBatchOptions::builder()
    ///             .identifier(BatchIdentifier::ByAssayId(7))
    ///             .batch(batch)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("saved batch for assay {:?}", response.assay_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_assay_batch(
        &self,
        options: SaveAssayBatchOptions,
    ) -> Result<SaveAssayBatchResponse, LabkeyError> {
        validate_save_assay_batch_options(&options)?;
        let url = self.build_url("assay", "saveAssayBatch", options.container_path.as_deref());
        let (assay_id, protocol_name) = match options.identifier {
            BatchIdentifier::ByAssayId(id) => (Some(id), None),
            BatchIdentifier::ByProtocolName(name) => (None, Some(name)),
        };
        let body = SaveAssayBatchBody {
            assay_id,
            protocol_name,
            batch: options.batch,
        };
        self.post(url, &body).await
    }

    /// Save (insert or update) assay runs without a batch wrapper.
    ///
    /// Sends a POST request to `assay-saveAssayRuns.api`. Unlike
    /// [`save_assay_batch`](Self::save_assay_batch), this endpoint accepts
    /// runs directly without grouping them into a batch.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the protocol name is blank, the runs list
    /// is empty, the HTTP request fails, or the response cannot be
    /// deserialized.
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
    /// use labkey_rs::assay::SaveAssayRunsOptions;
    /// use labkey_rs::experiment::Run;
    ///
    /// let run: Run = serde_json::from_value(serde_json::json!({}))?;
    /// let response = client
    ///     .save_assay_runs(
    ///         SaveAssayRunsOptions::builder()
    ///             .protocol_name("General".to_string())
    ///             .runs(vec![run])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("saved {} runs", response.runs.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_assay_runs(
        &self,
        options: SaveAssayRunsOptions,
    ) -> Result<SaveAssayRunsResponse, LabkeyError> {
        validate_save_assay_runs_options(&options)?;
        let url = self.build_url("assay", "saveAssayRuns", options.container_path.as_deref());
        let body = SaveAssayRunsBody {
            protocol_name: options.protocol_name,
            runs: options.runs,
        };
        self.post(url, &body).await
    }
}

fn build_get_assays_body(options: &GetAssaysOptions) -> GetAssaysBody {
    GetAssaysBody {
        id: options.id,
        name: options.name.clone(),
        plate_enabled: options.plate_enabled,
        status: options.status.clone(),
        type_: options.type_.clone(),
    }
}

fn build_get_protocol_params(identifier: &ProtocolIdentifier) -> Vec<(String, String)> {
    match identifier {
        ProtocolIdentifier::ByProvider(provider_name) => {
            vec![("providerName".to_string(), provider_name.clone())]
        }
        ProtocolIdentifier::ById { id, copy } => [
            Some(("protocolId".to_string(), id.to_string())),
            opt("copy", *copy),
        ]
        .into_iter()
        .flatten()
        .collect(),
    }
}

fn extract_protocol_response(
    endpoint: &str,
    response: &serde_json::Value,
) -> Result<AssayProtocol, LabkeyError> {
    serde_json::from_value::<ProtocolEnvelope>(response.clone())
        .map(|envelope| envelope.data)
        .map_err(|error| LabkeyError::UnexpectedResponse {
            status: StatusCode::OK,
            text: format!("invalid {endpoint} response: {response}; parse error: {error}"),
        })
}

fn validate_get_protocol_options(options: &GetProtocolOptions) -> Result<(), LabkeyError> {
    match &options.identifier {
        ProtocolIdentifier::ByProvider(provider_name) if provider_name.trim().is_empty() => {
            Err(LabkeyError::InvalidInput(
                "get_protocol requires a non-empty provider name in ByProvider mode".to_string(),
            ))
        }
        _ => Ok(()),
    }
}

fn validate_get_nab_runs_options(options: &GetNabRunsOptions) -> Result<(), LabkeyError> {
    if options.assay_name.trim().is_empty() {
        return Err(LabkeyError::InvalidInput(
            "get_nab_runs requires a non-empty assay_name".to_string(),
        ));
    }
    Ok(())
}

fn validate_get_assay_run_options(options: &GetAssayRunOptions) -> Result<(), LabkeyError> {
    if options.lsid.trim().is_empty() {
        return Err(LabkeyError::InvalidInput(
            "get_assay_run requires a non-empty lsid".to_string(),
        ));
    }
    Ok(())
}

fn validate_get_assay_batch_options(options: &GetAssayBatchOptions) -> Result<(), LabkeyError> {
    if options.protocol_name.trim().is_empty() {
        return Err(LabkeyError::InvalidInput(
            "get_assay_batch requires a non-empty protocol_name".to_string(),
        ));
    }
    Ok(())
}

fn validate_save_assay_batch_options(options: &SaveAssayBatchOptions) -> Result<(), LabkeyError> {
    match &options.identifier {
        BatchIdentifier::ByAssayId(id) if *id <= 0 => Err(LabkeyError::InvalidInput(
            "save_assay_batch requires a positive assay_id in ByAssayId mode".to_string(),
        )),
        BatchIdentifier::ByProtocolName(name) if name.trim().is_empty() => {
            Err(LabkeyError::InvalidInput(
                "save_assay_batch requires a non-empty protocol_name in ByProtocolName mode"
                    .to_string(),
            ))
        }
        _ => Ok(()),
    }
}

fn validate_save_assay_runs_options(options: &SaveAssayRunsOptions) -> Result<(), LabkeyError> {
    if options.protocol_name.trim().is_empty() {
        return Err(LabkeyError::InvalidInput(
            "save_assay_runs requires a non-empty protocol_name".to_string(),
        ));
    }
    if options.runs.is_empty() {
        return Err(LabkeyError::InvalidInput(
            "save_assay_runs requires at least one run".to_string(),
        ));
    }
    Ok(())
}

fn validate_import_run_options(options: &ImportRunOptions) -> Result<(), LabkeyError> {
    if options.assay_id <= 0 {
        return Err(LabkeyError::InvalidInput(
            "import_run requires assay_id to be greater than zero".to_string(),
        ));
    }

    match &options.source {
        ImportRunSource::File { data, filename } => {
            if filename.trim().is_empty() {
                return Err(LabkeyError::InvalidInput(
                    "import_run file source requires a non-empty filename".to_string(),
                ));
            }
            if data.is_empty() {
                return Err(LabkeyError::InvalidInput(
                    "import_run file source requires non-empty file data".to_string(),
                ));
            }
        }
        ImportRunSource::RunFilePath(path) => {
            if path.trim().is_empty() {
                return Err(LabkeyError::InvalidInput(
                    "import_run run_file_path source requires a non-empty path".to_string(),
                ));
            }
        }
        ImportRunSource::DataRows(rows) => {
            if rows.is_empty() {
                return Err(LabkeyError::InvalidInput(
                    "import_run data_rows source requires at least one row".to_string(),
                ));
            }
        }
    }

    Ok(())
}

fn build_import_run_parts(options: &ImportRunOptions) -> Vec<ImportRunPart> {
    if options.use_json.unwrap_or(false) {
        let mut parts = vec![ImportRunPart::Json {
            name: "json".to_string(),
            value: build_import_run_json_payload(options).to_string(),
        }];

        // Java ImportRunCommand adds the file part outside the useJson branch,
        // so it is always sent as a separate binary multipart part regardless
        // of JSON mode.
        if let ImportRunSource::File { data, filename } = &options.source {
            parts.push(ImportRunPart::File {
                name: "file".to_string(),
                filename: filename.clone(),
                data: data.clone(),
            });
        }

        return parts;
    }

    let mut parts = vec![ImportRunPart::Text {
        name: "assayId".to_string(),
        value: options.assay_id.to_string(),
    }];

    append_import_run_common_parts(&mut parts, options);

    match &options.source {
        ImportRunSource::File { data, filename } => parts.push(ImportRunPart::File {
            name: "file".to_string(),
            filename: filename.clone(),
            data: data.clone(),
        }),
        ImportRunSource::RunFilePath(path) => {
            push_optional_part(&mut parts, "runFilePath", Some(path.clone()));
        }
        ImportRunSource::DataRows(rows) => {
            push_optional_part(
                &mut parts,
                "dataRows",
                Some(serde_json::Value::Array(rows.clone()).to_string()),
            );
        }
    }

    parts
}

fn append_import_run_common_parts(parts: &mut Vec<ImportRunPart>, options: &ImportRunOptions) {
    push_optional_part(parts, "name", options.name.clone());
    push_optional_part(parts, "comment", options.comment.clone());
    push_optional_part(
        parts,
        "batchId",
        options.batch_id.map(|value| value.to_string()),
    );
    push_optional_part(
        parts,
        "reRunId",
        options.re_run_id.map(|value| value.to_string()),
    );
    push_optional_part(
        parts,
        "saveDataAsFile",
        options.save_data_as_file.map(|value| value.to_string()),
    );
    push_optional_part(
        parts,
        "saveMatchingColumnDataOnly",
        options
            .save_matching_column_data_only
            .map(|value| value.to_string()),
    );
    push_optional_part(parts, "jobDescription", options.job_description.clone());
    push_optional_part(
        parts,
        "jobNotificationProvider",
        options.job_notification_provider.clone(),
    );
    push_optional_part(
        parts,
        "forceAsync",
        options.force_async.map(|value| value.to_string()),
    );
    push_optional_part(
        parts,
        "allowCrossRunFileInputs",
        options
            .allow_cross_run_file_inputs
            .map(|value| value.to_string()),
    );
    push_optional_part(
        parts,
        "workflowTask",
        options.workflow_task.map(|value| value.to_string()),
    );
    push_optional_part(
        parts,
        "allowLookupByAlternateKey",
        options
            .allow_lookup_by_alternate_key
            .map(|value| value.to_string()),
    );
    push_optional_part(
        parts,
        "auditUserComment",
        options.audit_user_comment.clone(),
    );
    push_optional_part(
        parts,
        "auditDetails",
        options
            .audit_details
            .as_ref()
            .map(serde_json::Value::to_string),
    );

    if let Some(properties) = &options.properties {
        append_property_parts(parts, "properties", properties);
    }
    if let Some(batch_properties) = &options.batch_properties {
        append_property_parts(parts, "batchProperties", batch_properties);
    }

    push_optional_part(
        parts,
        "plateMetadata",
        options
            .plate_metadata
            .as_ref()
            .map(serde_json::Value::to_string),
    );
}

fn build_import_run_json_payload(options: &ImportRunOptions) -> serde_json::Value {
    let mut payload = serde_json::Map::new();
    payload.insert(
        "assayId".to_string(),
        serde_json::Value::Number(serde_json::Number::from(options.assay_id)),
    );
    payload.insert("useJson".to_string(), serde_json::Value::Bool(true));

    insert_optional_json_string(&mut payload, "name", options.name.clone());
    insert_optional_json_string(&mut payload, "comment", options.comment.clone());
    insert_optional_json_i64(&mut payload, "batchId", options.batch_id);
    insert_optional_json_i64(&mut payload, "reRunId", options.re_run_id);
    insert_optional_json_bool(&mut payload, "saveDataAsFile", options.save_data_as_file);
    insert_optional_json_bool(
        &mut payload,
        "saveMatchingColumnDataOnly",
        options.save_matching_column_data_only,
    );
    insert_optional_json_string(
        &mut payload,
        "jobDescription",
        options.job_description.clone(),
    );
    insert_optional_json_string(
        &mut payload,
        "jobNotificationProvider",
        options.job_notification_provider.clone(),
    );
    insert_optional_json_bool(&mut payload, "forceAsync", options.force_async);
    insert_optional_json_bool(
        &mut payload,
        "allowCrossRunFileInputs",
        options.allow_cross_run_file_inputs,
    );
    insert_optional_json_i64(&mut payload, "workflowTask", options.workflow_task);
    insert_optional_json_bool(
        &mut payload,
        "allowLookupByAlternateKey",
        options.allow_lookup_by_alternate_key,
    );
    insert_optional_json_string(
        &mut payload,
        "auditUserComment",
        options.audit_user_comment.clone(),
    );
    insert_optional_json_value(&mut payload, "auditDetails", options.audit_details.clone());
    insert_optional_json_value(
        &mut payload,
        "properties",
        options
            .properties
            .clone()
            .map(|value| serde_json::Value::Object(serde_json::Map::from_iter(value))),
    );
    insert_optional_json_value(
        &mut payload,
        "batchProperties",
        options
            .batch_properties
            .clone()
            .map(|value| serde_json::Value::Object(serde_json::Map::from_iter(value))),
    );
    insert_optional_json_value(
        &mut payload,
        "plateMetadata",
        options.plate_metadata.clone(),
    );

    match &options.source {
        // File source is sent as a separate binary multipart part, not in the
        // JSON payload. See build_import_run_parts for the file part emission.
        ImportRunSource::File { .. } => {}
        ImportRunSource::RunFilePath(path) => {
            payload.insert(
                "runFilePath".to_string(),
                serde_json::Value::String(path.clone()),
            );
        }
        ImportRunSource::DataRows(rows) => {
            payload.insert(
                "dataRows".to_string(),
                serde_json::Value::Array(rows.clone()),
            );
        }
    }

    serde_json::Value::Object(payload)
}

fn build_import_run_form(
    parts: Vec<ImportRunPart>,
) -> Result<reqwest::multipart::Form, LabkeyError> {
    let mut form = reqwest::multipart::Form::new();
    for part in parts {
        match part {
            ImportRunPart::Text { name, value } => {
                form = form.part(name, reqwest::multipart::Part::text(value));
            }
            ImportRunPart::Json { name, value } => {
                let json_part = reqwest::multipart::Part::text(value)
                    .mime_str("application/json")
                    .map_err(|error| {
                        LabkeyError::InvalidInput(format!(
                            "failed to build import_run json part: {error}"
                        ))
                    })?;
                form = form.part(name, json_part);
            }
            ImportRunPart::File {
                name,
                filename,
                data,
            } => {
                let part = reqwest::multipart::Part::bytes(data)
                    .file_name(filename)
                    .mime_str("application/octet-stream")
                    .map_err(|error| {
                        LabkeyError::InvalidInput(format!(
                            "failed to build import_run file part: {error}"
                        ))
                    })?;
                form = form.part(name, part);
            }
        }
    }

    Ok(form)
}

fn append_property_parts(
    parts: &mut Vec<ImportRunPart>,
    prefix: &str,
    properties: &HashMap<String, serde_json::Value>,
) {
    for (key, value) in properties {
        if value.is_null() {
            continue;
        }

        let part_name = format!("{prefix}['{key}']");
        let part_value = property_value_to_string(value);
        parts.push(ImportRunPart::Text {
            name: part_name,
            value: part_value,
        });
    }
}

fn property_value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Bool(_)
        | serde_json::Value::Number(_)
        | serde_json::Value::Array(_)
        | serde_json::Value::Object(_)
        | serde_json::Value::Null => value.to_string(),
    }
}

fn push_optional_part(parts: &mut Vec<ImportRunPart>, name: &str, value: Option<String>) {
    if let Some(value) = value {
        parts.push(ImportRunPart::Text {
            name: name.to_string(),
            value,
        });
    }
}

fn insert_optional_json_string(
    payload: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<String>,
) {
    if let Some(value) = value {
        payload.insert(key.to_string(), serde_json::Value::String(value));
    }
}

fn insert_optional_json_bool(
    payload: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<bool>,
) {
    if let Some(value) = value {
        payload.insert(key.to_string(), serde_json::Value::Bool(value));
    }
}

fn insert_optional_json_i64(
    payload: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<i64>,
) {
    if let Some(value) = value {
        payload.insert(
            key.to_string(),
            serde_json::Value::Number(serde_json::Number::from(value)),
        );
    }
}

fn insert_optional_json_value(
    payload: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    value: Option<serde_json::Value>,
) {
    if let Some(value) = value {
        payload.insert(key.to_string(), value);
    }
}

fn validate_object_ids(endpoint: &str, object_ids: &[String]) -> Result<(), LabkeyError> {
    if object_ids.is_empty() {
        return Err(LabkeyError::InvalidInput(format!(
            "{endpoint} requires at least one object_id"
        )));
    }

    if object_ids.iter().any(|id| id.trim().is_empty()) {
        return Err(LabkeyError::InvalidInput(format!(
            "{endpoint} does not accept blank object_ids"
        )));
    }

    Ok(())
}

fn build_get_nab_runs_params(options: &GetNabRunsOptions) -> Vec<(String, String)> {
    let mut params: Vec<(String, String)> = [
        Some(("assayName".into(), options.assay_name.clone())),
        opt("calculateNeut", options.calculate_neut),
        opt("includeFitParameters", options.include_fit_parameters),
        opt("includeStats", options.include_stats),
        opt("includeWells", options.include_wells),
        opt("query.sort", options.sort.clone()),
        opt("query.offset", options.offset),
        match options.max_rows {
            Some(max) if max < 0 => Some(("query.showRows".into(), "all".into())),
            Some(max) => Some(("query.maxRows".into(), max.to_string())),
            None => None,
        },
    ]
    .into_iter()
    .flatten()
    .collect();

    if let Some(filters) = &options.filter_array {
        params.extend(encode_filters(filters, "query"));
    }

    params
}

fn build_get_study_nab_graph_url_params(
    options: &GetStudyNabGraphUrlOptions,
) -> Vec<(String, String)> {
    let mut params: Vec<(String, String)> = [
        opt("chartTitle", options.chart_title.clone()),
        opt("fitType", options.fit_type.map(fit_type_to_wire)),
        opt("height", options.height),
        opt("width", options.width),
    ]
    .into_iter()
    .flatten()
    .collect();

    for object_id in &options.object_ids {
        params.push(("id".into(), object_id.clone()));
    }

    params
}

fn build_get_study_nab_runs_params(options: &GetStudyNabRunsOptions) -> Vec<(String, String)> {
    let mut params: Vec<(String, String)> = [
        opt("calculateNeut", options.calculate_neut),
        opt("includeFitParameters", options.include_fit_parameters),
        opt("includeStats", options.include_stats),
        opt("includeWells", options.include_wells),
    ]
    .into_iter()
    .flatten()
    .collect();

    for object_id in &options.object_ids {
        params.push(("objectIds".into(), object_id.clone()));
    }

    params
}

const fn fit_type_to_wire(value: FitType) -> &'static str {
    match value {
        FitType::FiveParameter => "FIVE_PARAMETER",
        FitType::FourParameter => "FOUR_PARAMETER",
        FitType::Polynomial => "POLYNOMIAL",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{ClientConfig, Credential};

    fn test_client() -> LabkeyClient {
        let config = ClientConfig::new(
            "https://labkey.example.com/labkey",
            Credential::ApiKey("test-api-key".to_string()),
            "/Project/Folder",
        );
        LabkeyClient::new(config).expect("test client should construct")
    }

    #[test]
    fn all_assay_endpoint_urls_match_expected_routes() {
        let client = test_client();
        let cases = [
            ("assay", "assayList.api"),
            ("nabassay", "getNabRuns.api"),
            ("nabassay", "getStudyNabGraphURL.api"),
            ("nabassay", "getStudyNabRuns.api"),
            ("assay", "getProtocol"),
            ("assay", "saveProtocol"),
            ("assay", "getAssayRun"),
            ("assay", "importRun.api"),
        ];

        for (controller, action) in cases {
            let url = client.build_url(controller, action, Some("/Alt/Container"));
            assert_eq!(
                url.as_str(),
                format!("https://labkey.example.com/labkey/Alt/Container/{controller}-{action}")
            );
        }
    }

    #[test]
    fn assay_design_deserializes_with_link_keys() {
        let value = serde_json::json!({
            "containerPath": "/Project/Folder",
            "description": "A neutralization assay",
            "domainTypes": {"Result": "results"},
            "domains": {"results": []},
            "id": 42,
            "importAction": "importRun",
            "importController": "assay",
            "links": {
                "begin": "/labkey/begin.view",
                "runs": "/labkey/runs.view"
            },
            "name": "NAb",
            "protocolSchemaName": "assay.General.NAb",
            "templateLink": "/labkey/template.xlsx",
            "type": "General"
        });

        let assay: AssayDesign = serde_json::from_value(value).expect("deserialize assay design");
        assert_eq!(assay.id, 42);
        assert_eq!(assay.name, "NAb");
        assert_eq!(assay.type_.as_deref(), Some("General"));
        assert_eq!(
            assay.links.get("begin").map(String::as_str),
            Some("/labkey/begin.view")
        );
        assert_eq!(
            assay.links.get("runs").map(String::as_str),
            Some("/labkey/runs.view")
        );
    }

    #[test]
    fn assay_design_deserializes_minimal_with_defaults() {
        let value = serde_json::json!({
            "id": 1,
            "name": "Minimal"
        });

        let assay: AssayDesign = serde_json::from_value(value).expect("deserialize minimal assay");
        assert_eq!(assay.id, 1);
        assert_eq!(assay.name, "Minimal");
        assert!(assay.links.is_empty());
        assert!(assay.domain_types.is_empty());
        assert!(assay.domains.is_empty());
    }

    #[test]
    fn get_assays_body_sends_flat_fields() {
        let options = GetAssaysOptions::builder()
            .id(7)
            .name("MyAssay".to_string())
            .plate_enabled(true)
            .status("Active".to_string())
            .type_("General".to_string())
            .build();

        let body = serde_json::to_value(build_get_assays_body(&options)).expect("serialize body");
        assert_eq!(body["id"], 7);
        assert_eq!(body["name"], "MyAssay");
        assert_eq!(body["plateEnabled"], true);
        assert_eq!(body["status"], "Active");
        assert_eq!(body["type"], "General");
        assert!(body.get("parameters").is_none());
    }

    #[test]
    fn get_assays_body_omits_unset_fields() {
        let options = GetAssaysOptions::builder().build();
        let body = serde_json::to_value(build_get_assays_body(&options)).expect("serialize body");

        assert!(body.get("parameters").is_none());
        assert!(body.as_object().is_some_and(serde_json::Map::is_empty));
    }

    #[test]
    fn get_nab_runs_params_use_query_prefixes_and_filter_encoding() {
        let options = GetNabRunsOptions::builder()
            .assay_name("NAb".to_string())
            .sort("-Created".to_string())
            .offset(5)
            .max_rows(-1)
            .filter_array(vec![Filter::equal("SpecimenID", "S-1")])
            .build();

        let params = build_get_nab_runs_params(&options);
        assert!(params.contains(&("assayName".to_string(), "NAb".to_string())));
        assert!(params.contains(&("query.sort".to_string(), "-Created".to_string())));
        assert!(params.contains(&("query.offset".to_string(), "5".to_string())));
        assert!(params.contains(&("query.showRows".to_string(), "all".to_string())));
        assert!(params.contains(&("query.SpecimenID~eq".to_string(), "S-1".to_string())));
        assert!(!params.iter().any(|(k, _)| k == "query.maxRows"));
    }

    #[test]
    fn get_nab_runs_params_use_query_max_rows_for_positive_limits() {
        let options = GetNabRunsOptions::builder()
            .assay_name("NAb".to_string())
            .max_rows(25)
            .build();

        let params = build_get_nab_runs_params(&options);
        assert!(params.contains(&("query.maxRows".to_string(), "25".to_string())));
        assert!(!params.iter().any(|(k, _)| k == "query.showRows"));
    }

    #[test]
    fn get_nab_runs_params_omit_max_rows_when_not_provided() {
        let options = GetNabRunsOptions::builder()
            .assay_name("NAb".to_string())
            .build();

        let params = build_get_nab_runs_params(&options);
        assert!(!params.iter().any(|(k, _)| k == "query.maxRows"));
        assert!(!params.iter().any(|(k, _)| k == "query.showRows"));
    }

    #[test]
    fn fit_type_round_trips_all_variants() {
        let variants = [
            FitType::FiveParameter,
            FitType::FourParameter,
            FitType::Polynomial,
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).expect("serialize fit type");
            let restored: FitType = serde_json::from_str(&json).expect("deserialize fit type");
            assert_eq!(restored, variant);
        }
    }

    #[test]
    fn fit_type_serializes_exact_wire_values() {
        assert_eq!(
            serde_json::to_string(&FitType::FiveParameter).expect("serialize fit type"),
            "\"FIVE_PARAMETER\""
        );
        assert_eq!(
            serde_json::to_string(&FitType::FourParameter).expect("serialize fit type"),
            "\"FOUR_PARAMETER\""
        );
        assert_eq!(
            serde_json::to_string(&FitType::Polynomial).expect("serialize fit type"),
            "\"POLYNOMIAL\""
        );
    }

    #[test]
    fn assay_link_round_trips_all_variants() {
        let variants = [
            AssayLink::Batches,
            AssayLink::Begin,
            AssayLink::DesignCopy,
            AssayLink::DesignEdit,
            AssayLink::Import,
            AssayLink::Result,
            AssayLink::Results,
            AssayLink::Runs,
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).expect("serialize assay link");
            let restored: AssayLink = serde_json::from_str(&json).expect("deserialize assay link");
            assert_eq!(restored, variant);
        }
    }

    #[test]
    fn assay_link_serializes_exact_wire_values() {
        let pairs = [
            (AssayLink::Batches, "\"batches\""),
            (AssayLink::Begin, "\"begin\""),
            (AssayLink::DesignCopy, "\"designCopy\""),
            (AssayLink::DesignEdit, "\"designEdit\""),
            (AssayLink::Import, "\"import\""),
            (AssayLink::Result, "\"result\""),
            (AssayLink::Results, "\"results\""),
            (AssayLink::Runs, "\"runs\""),
        ];

        for (variant, expected) in pairs {
            assert_eq!(
                serde_json::to_string(&variant).expect("serialize assay link"),
                expected
            );
        }
    }

    fn fit_type_variant_count(value: FitType) -> usize {
        match value {
            FitType::FiveParameter | FitType::FourParameter | FitType::Polynomial => 3,
        }
    }

    fn assay_link_variant_count(value: AssayLink) -> usize {
        match value {
            AssayLink::Batches
            | AssayLink::Begin
            | AssayLink::DesignCopy
            | AssayLink::DesignEdit
            | AssayLink::Import
            | AssayLink::Result
            | AssayLink::Results
            | AssayLink::Runs => 8,
        }
    }

    #[test]
    fn fit_type_variant_count_regression() {
        assert_eq!(fit_type_variant_count(FitType::FiveParameter), 3);
    }

    #[test]
    fn assay_link_variant_count_regression() {
        assert_eq!(assay_link_variant_count(AssayLink::Batches), 8);
    }

    #[test]
    fn get_nab_runs_validation_rejects_blank_assay_name() {
        let error = validate_get_nab_runs_options(
            &GetNabRunsOptions::builder()
                .assay_name("   ".to_string())
                .build(),
        )
        .expect_err("blank assay name should fail");

        match error {
            LabkeyError::InvalidInput(message) => {
                assert!(message.contains("assay_name"));
            }
            other => panic!("expected invalid input error, got {other:?}"),
        }
    }

    #[test]
    fn object_id_validation_rejects_empty_and_blank_values() {
        let empty = validate_object_ids("get_study_nab_runs", &[])
            .expect_err("empty object ids should fail");
        assert!(matches!(empty, LabkeyError::InvalidInput(_)));

        let blank = validate_object_ids("get_study_nab_runs", &["  ".to_string()])
            .expect_err("blank object id should fail");
        assert!(matches!(blank, LabkeyError::InvalidInput(_)));
    }

    #[test]
    fn assay_protocol_round_trips_all_expected_fields() {
        let protocol = AssayProtocol {
            protocol_id: Some(5),
            name: "Neutralization".to_string(),
            description: Some("NAb protocol".to_string()),
            provider_name: "General".to_string(),
            domains: vec![DomainDesign {
                domain_id: Some(11),
                domain_uri: Some("urn:lsid:labkey.com:AssayDomain.Folder-1:11".to_string()),
                name: Some("Results".to_string()),
                description: Some("Result fields".to_string()),
                schema_name: Some("assay".to_string()),
                query_name: Some("results".to_string()),
                fields: None,
                indices: None,
                extra: HashMap::new(),
            }],
            allow_background_upload: Some(true),
            background_upload: Some(false),
            allow_editable_results: Some(true),
            editable_results: Some(true),
            editable_runs: Some(false),
            save_script_files: Some(true),
            allow_qc_states: Some(true),
            qc_enabled: Some(false),
            allow_spaces_in_path: Some(true),
            allow_transformation_script: Some(true),
            auto_copy_target_container_id: Some("folder-id".to_string()),
            available_detection_methods: vec!["MethodA".to_string(), "MethodB".to_string()],
            selected_detection_method: Some("MethodA".to_string()),
            available_metadata_input_formats: HashMap::from([
                ("csv".to_string(), "CSV".to_string()),
                ("xlsx".to_string(), "Excel".to_string()),
            ]),
            selected_metadata_input_format: Some("csv".to_string()),
            available_plate_templates: vec!["Template1".to_string()],
            selected_plate_template: Some("Template1".to_string()),
            allow_plate_metadata: Some(true),
            plate_metadata: Some(false),
            protocol_parameters: HashMap::from([("qcLevel".to_string(), "2".to_string())]),
            protocol_transform_scripts: vec!["transform.py".to_string()],
            extra: HashMap::new(),
        };

        let value = serde_json::to_value(&protocol).expect("serialize protocol");
        assert!(value.get("providerName").is_some());
        assert!(value.get("allowEditableResults").is_some());
        assert!(value.get("protocolTransformScripts").is_some());
        assert!(value.get("availableMetadataInputFormats").is_some());

        let restored: AssayProtocol = serde_json::from_value(value).expect("deserialize protocol");

        assert_eq!(restored.protocol_id, Some(5));
        assert_eq!(restored.name, "Neutralization");
        assert_eq!(restored.provider_name, "General");
        assert_eq!(restored.domains.len(), 1);
        assert_eq!(restored.available_detection_methods.len(), 2);
        assert_eq!(restored.protocol_transform_scripts, vec!["transform.py"]);
    }

    #[test]
    fn get_protocol_params_support_provider_and_id_modes() {
        let by_provider =
            build_get_protocol_params(&ProtocolIdentifier::ByProvider("General".to_string()));
        assert_eq!(
            by_provider,
            vec![("providerName".to_string(), "General".to_string())]
        );

        let by_id = build_get_protocol_params(&ProtocolIdentifier::ById {
            id: 42,
            copy: Some(true),
        });
        assert!(by_id.contains(&("protocolId".to_string(), "42".to_string())));
        assert!(by_id.contains(&("copy".to_string(), "true".to_string())));

        let by_id_without_copy =
            build_get_protocol_params(&ProtocolIdentifier::ById { id: 42, copy: None });
        assert!(by_id_without_copy.contains(&("protocolId".to_string(), "42".to_string())));
        assert!(!by_id_without_copy.iter().any(|(k, _)| k == "copy"));

        let by_id_copy_false = build_get_protocol_params(&ProtocolIdentifier::ById {
            id: 42,
            copy: Some(false),
        });
        assert!(by_id_copy_false.contains(&("copy".to_string(), "false".to_string())));
    }

    #[test]
    fn protocol_envelope_extraction_requires_response_data() {
        let happy = serde_json::json!({
            "success": true,
            "data": {
                "protocolId": 7,
                "name": "General",
                "providerName": "General",
                "domains": []
            }
        });

        let protocol = extract_protocol_response("get_protocol", &happy)
            .expect("data envelope should deserialize");
        assert_eq!(protocol.protocol_id, Some(7));

        let missing = serde_json::json!({"success": true});
        let error = extract_protocol_response("get_protocol", &missing)
            .expect_err("missing data should fail");
        match error {
            LabkeyError::UnexpectedResponse { text, .. } => {
                assert!(text.contains("get_protocol"));
            }
            other => panic!("expected unexpected response, got {other:?}"),
        }

        let save_error = extract_protocol_response("save_protocol", &missing)
            .expect_err("missing data should fail for save_protocol too");
        match save_error {
            LabkeyError::UnexpectedResponse { text, .. } => {
                assert!(text.contains("save_protocol"));
            }
            other => panic!("expected unexpected response, got {other:?}"),
        }
    }

    #[test]
    fn save_protocol_body_serializes_directly_from_assay_protocol() {
        let protocol = AssayProtocol {
            protocol_id: Some(9),
            name: "General".to_string(),
            description: Some("desc".to_string()),
            provider_name: "General".to_string(),
            domains: vec![],
            allow_background_upload: None,
            background_upload: None,
            allow_editable_results: None,
            editable_results: None,
            editable_runs: None,
            save_script_files: None,
            allow_qc_states: None,
            qc_enabled: None,
            allow_spaces_in_path: None,
            allow_transformation_script: None,
            auto_copy_target_container_id: None,
            available_detection_methods: vec![],
            selected_detection_method: None,
            available_metadata_input_formats: HashMap::new(),
            selected_metadata_input_format: None,
            available_plate_templates: vec![],
            selected_plate_template: None,
            allow_plate_metadata: None,
            plate_metadata: None,
            protocol_parameters: HashMap::new(),
            protocol_transform_scripts: vec![],
            extra: HashMap::new(),
        };

        let body = serde_json::to_value(&protocol).expect("serialize body");
        assert_eq!(body["protocolId"], 9);
        assert_eq!(body["name"], "General");
        assert_eq!(body["providerName"], "General");
        assert_eq!(body["description"], "desc");
        assert!(body.get("data").is_none());
        assert!(body.get("protocol").is_none());
    }

    #[test]
    fn get_assay_run_body_contains_required_lsid() {
        fn assert_get_assay_run_return_type<F>(_: F)
        where
            F: std::future::Future<Output = Result<Run, LabkeyError>>,
        {
        }

        let client = test_client();
        let future = client.get_assay_run(
            GetAssayRunOptions::builder()
                .lsid("urn:lsid:labkey.com:AssayRun.Folder-1:123".to_string())
                .build(),
        );
        assert_get_assay_run_return_type(future);

        let body = GetAssayRunBody {
            lsid: "urn:lsid:labkey.com:AssayRun.Folder-1:123".to_string(),
        };
        let value = serde_json::to_value(body).expect("serialize get_assay_run body");

        assert_eq!(value["lsid"], "urn:lsid:labkey.com:AssayRun.Folder-1:123");
    }

    #[test]
    fn get_protocol_validation_rejects_blank_provider_name() {
        let error = validate_get_protocol_options(
            &GetProtocolOptions::builder()
                .identifier(ProtocolIdentifier::ByProvider("   ".to_string()))
                .build(),
        )
        .expect_err("blank provider name should fail");

        match error {
            LabkeyError::InvalidInput(message) => assert!(message.contains("provider")),
            other => panic!("expected invalid input, got {other:?}"),
        }
    }

    #[test]
    fn get_assay_run_validation_rejects_blank_lsid() {
        let error = validate_get_assay_run_options(
            &GetAssayRunOptions::builder()
                .lsid("   ".to_string())
                .build(),
        )
        .expect_err("blank lsid should fail");

        match error {
            LabkeyError::InvalidInput(message) => assert!(message.contains("lsid")),
            other => panic!("expected invalid input, got {other:?}"),
        }
    }

    #[test]
    fn import_run_response_deserializes_happy_path() {
        let value = serde_json::json!({
            "success": true,
            "runId": 31,
            "batchId": 12,
            "jobId": "job-42"
        });

        let response: ImportRunResponse =
            serde_json::from_value(value).expect("deserialize import run response");
        assert!(response.success);
        assert_eq!(response.run_id, Some(31));
        assert_eq!(response.batch_id, Some(12));
        assert_eq!(response.job_id.as_deref(), Some("job-42"));
    }

    #[test]
    fn import_run_response_deserializes_minimal_path() {
        let value = serde_json::json!({"success": true});

        let response: ImportRunResponse =
            serde_json::from_value(value).expect("deserialize minimal import run response");
        assert!(response.success);
        assert!(response.run_id.is_none());
        assert!(response.batch_id.is_none());
        assert!(response.job_id.is_none());
        assert!(response.success_url.is_none());
        assert!(response.assay_id.is_none());
    }

    #[test]
    fn import_run_response_deserializes_success_url_lowercase_wire_key_and_assay_id() {
        let value = serde_json::json!({
            "success": true,
            "runId": 55,
            "successurl": "http://labkey.example.com/success",
            "assayId": 101
        });

        let response: ImportRunResponse =
            serde_json::from_value(value).expect("deserialize import run response with new fields");
        assert!(response.success);
        assert_eq!(response.run_id, Some(55));
        assert_eq!(
            response.success_url.as_deref(),
            Some("http://labkey.example.com/success")
        );
        assert_eq!(response.assay_id, Some(101));
    }

    #[test]
    fn import_run_non_json_mode_uses_expected_part_names_and_bracket_keys() {
        let options = ImportRunOptions::builder()
            .assay_id(7)
            .source(ImportRunSource::DataRows(vec![
                serde_json::json!({"Name": "Alice"}),
                serde_json::json!({"Name": "Bob"}),
            ]))
            .name("Run 1".to_string())
            .properties(HashMap::from([
                (
                    "qc".to_string(),
                    serde_json::Value::String("pass".to_string()),
                ),
                (
                    "metadata".to_string(),
                    serde_json::json!({"instrument": "A1"}),
                ),
            ]))
            .batch_properties(HashMap::from([
                (
                    "batchFlag".to_string(),
                    serde_json::Value::String("yes".to_string()),
                ),
                ("skip".to_string(), serde_json::Value::Null),
            ]))
            .build();

        let parts = build_import_run_parts(&options);

        assert!(parts.contains(&ImportRunPart::Text {
            name: "assayId".to_string(),
            value: "7".to_string(),
        }));
        assert!(parts.contains(&ImportRunPart::Text {
            name: "name".to_string(),
            value: "Run 1".to_string(),
        }));
        assert!(parts.contains(&ImportRunPart::Text {
            name: "properties['qc']".to_string(),
            value: "pass".to_string(),
        }));
        assert!(parts.iter().any(|part| {
            matches!(
                part,
                ImportRunPart::Text { name, value }
                if name == "properties['metadata']" && value.contains("instrument")
            )
        }));
        assert!(parts.contains(&ImportRunPart::Text {
            name: "batchProperties['batchFlag']".to_string(),
            value: "yes".to_string(),
        }));
        assert!(!parts.iter().any(|part| {
            matches!(
                part,
                ImportRunPart::Text { name, .. } if name == "batchProperties['skip']"
            )
        }));
        assert!(parts.iter().any(|part| {
            matches!(
                part,
                ImportRunPart::Text { name, value }
                if name == "dataRows" && value.contains("Alice") && value.contains("Bob")
            )
        }));
    }

    #[test]
    fn import_run_bracket_keys_handle_special_characters() {
        let properties = HashMap::from([
            (
                "emoji_\u{1F642}".to_string(),
                serde_json::Value::String("smile".to_string()),
            ),
            (
                "qu\"ot'ed\"key".to_string(),
                serde_json::Value::String("quotes\" are 'ok\"".to_string()),
            ),
            (
                "with[brackets]".to_string(),
                serde_json::Value::String("square brackets".to_string()),
            ),
            (
                "unicode_\u{00B5}g/\u{03BC}L".to_string(),
                serde_json::Value::String("micro units".to_string()),
            ),
            (
                "someInt".to_string(),
                serde_json::Value::Number(serde_json::Number::from(42)),
            ),
            ("array_primitive".to_string(), serde_json::json!([1, 2, 3])),
            ("empty_object".to_string(), serde_json::json!({})),
        ]);

        let options = ImportRunOptions::builder()
            .assay_id(1)
            .source(ImportRunSource::DataRows(vec![serde_json::json!({})]))
            .properties(properties)
            .build();

        let parts = build_import_run_parts(&options);

        assert!(parts.contains(&ImportRunPart::Text {
            name: "properties['emoji_\u{1F642}']".to_string(),
            value: "smile".to_string(),
        }));
        assert!(parts.contains(&ImportRunPart::Text {
            name: "properties['qu\"ot'ed\"key']".to_string(),
            value: "quotes\" are 'ok\"".to_string(),
        }));
        assert!(parts.contains(&ImportRunPart::Text {
            name: "properties['with[brackets]']".to_string(),
            value: "square brackets".to_string(),
        }));
        assert!(parts.contains(&ImportRunPart::Text {
            name: "properties['unicode_\u{00B5}g/\u{03BC}L']".to_string(),
            value: "micro units".to_string(),
        }));
        assert!(parts.contains(&ImportRunPart::Text {
            name: "properties['someInt']".to_string(),
            value: "42".to_string(),
        }));
        assert!(parts.contains(&ImportRunPart::Text {
            name: "properties['array_primitive']".to_string(),
            value: "[1,2,3]".to_string(),
        }));
        assert!(parts.contains(&ImportRunPart::Text {
            name: "properties['empty_object']".to_string(),
            value: "{}".to_string(),
        }));
    }

    #[test]
    fn import_run_json_mode_with_file_sends_json_and_binary_parts() {
        let options = ImportRunOptions::builder()
            .assay_id(42)
            .source(ImportRunSource::File {
                data: b"A\tB\n1\t2".to_vec(),
                filename: "run.tsv".to_string(),
            })
            .use_json(true)
            .allow_lookup_by_alternate_key(true)
            .build();

        let parts = build_import_run_parts(&options);

        // Java ImportRunCommand sends both a json text part and a file binary
        // part when useJson=true with a file source.
        assert_eq!(parts.len(), 2);

        match &parts[0] {
            ImportRunPart::Json { name, value } => {
                assert_eq!(name, "json");
                let payload: serde_json::Value =
                    serde_json::from_str(value).expect("json payload should parse");
                assert_eq!(payload["assayId"], 42);
                assert_eq!(payload["useJson"], true);
                assert_eq!(payload["allowLookupByAlternateKey"], true);
                // File data must NOT be in the JSON payload — it goes as a
                // separate binary part.
                assert!(payload.get("file").is_none());
            }
            other => panic!("expected json part first, got {other:?}"),
        }

        match &parts[1] {
            ImportRunPart::File {
                name,
                filename,
                data,
            } => {
                assert_eq!(name, "file");
                assert_eq!(filename, "run.tsv");
                assert_eq!(data, b"A\tB\n1\t2");
            }
            other => panic!("expected file part second, got {other:?}"),
        }
    }

    #[test]
    fn import_run_json_mode_with_non_file_source_sends_single_json_part() {
        let options = ImportRunOptions::builder()
            .assay_id(42)
            .source(ImportRunSource::DataRows(vec![serde_json::json!({"x": 1})]))
            .use_json(true)
            .build();

        let parts = build_import_run_parts(&options);
        assert_eq!(parts.len(), 1);

        match &parts[0] {
            ImportRunPart::Json { name, value } => {
                assert_eq!(name, "json");
                let payload: serde_json::Value =
                    serde_json::from_str(value).expect("json payload should parse");
                assert_eq!(payload["assayId"], 42);
                assert_eq!(payload["useJson"], true);
                assert!(payload.get("dataRows").is_some());
            }
            other => panic!("expected json part, got {other:?}"),
        }
    }

    #[test]
    fn import_run_source_variants_map_to_expected_part_names() {
        let file_parts = build_import_run_parts(
            &ImportRunOptions::builder()
                .assay_id(1)
                .source(ImportRunSource::File {
                    data: vec![1, 2, 3],
                    filename: "run.tsv".to_string(),
                })
                .build(),
        );
        assert!(file_parts.iter().any(|part| {
            matches!(
                part,
                ImportRunPart::File { name, filename, .. }
                if name == "file" && filename == "run.tsv"
            )
        }));

        let path_parts = build_import_run_parts(
            &ImportRunOptions::builder()
                .assay_id(1)
                .source(ImportRunSource::RunFilePath(
                    "/files/assay/run-1.tsv".to_string(),
                ))
                .build(),
        );
        assert!(path_parts.contains(&ImportRunPart::Text {
            name: "runFilePath".to_string(),
            value: "/files/assay/run-1.tsv".to_string(),
        }));

        let data_rows_parts = build_import_run_parts(
            &ImportRunOptions::builder()
                .assay_id(1)
                .source(ImportRunSource::DataRows(vec![
                    serde_json::json!({"Name": "A"}),
                ]))
                .build(),
        );
        assert!(data_rows_parts.iter().any(|part| {
            matches!(
                part,
                ImportRunPart::Text { name, value }
                if name == "dataRows" && value.contains("Name")
            )
        }));
    }

    #[test]
    fn import_run_validation_rejects_invalid_inputs() {
        let invalid_assay_id = validate_import_run_options(
            &ImportRunOptions::builder()
                .assay_id(0)
                .source(ImportRunSource::RunFilePath("/files/run.tsv".to_string()))
                .build(),
        )
        .expect_err("zero assay_id should fail");
        assert!(matches!(invalid_assay_id, LabkeyError::InvalidInput(_)));

        let blank_filename = validate_import_run_options(
            &ImportRunOptions::builder()
                .assay_id(1)
                .source(ImportRunSource::File {
                    data: vec![1],
                    filename: "   ".to_string(),
                })
                .build(),
        )
        .expect_err("blank filename should fail");
        assert!(matches!(blank_filename, LabkeyError::InvalidInput(_)));

        let empty_file_data = validate_import_run_options(
            &ImportRunOptions::builder()
                .assay_id(1)
                .source(ImportRunSource::File {
                    data: Vec::new(),
                    filename: "run.tsv".to_string(),
                })
                .build(),
        )
        .expect_err("empty file bytes should fail");
        assert!(matches!(empty_file_data, LabkeyError::InvalidInput(_)));

        let blank_path = validate_import_run_options(
            &ImportRunOptions::builder()
                .assay_id(1)
                .source(ImportRunSource::RunFilePath("   ".to_string()))
                .build(),
        )
        .expect_err("blank runFilePath should fail");
        assert!(matches!(blank_path, LabkeyError::InvalidInput(_)));

        let empty_rows = validate_import_run_options(
            &ImportRunOptions::builder()
                .assay_id(1)
                .source(ImportRunSource::DataRows(Vec::new()))
                .build(),
        )
        .expect_err("empty dataRows should fail");
        assert!(matches!(empty_rows, LabkeyError::InvalidInput(_)));
    }

    #[test]
    fn import_run_use_json_false_matches_default_multipart_mode() {
        let default_mode = build_import_run_parts(
            &ImportRunOptions::builder()
                .assay_id(1)
                .source(ImportRunSource::RunFilePath(
                    "/files/assay/run.tsv".to_string(),
                ))
                .build(),
        );

        let explicit_false = build_import_run_parts(
            &ImportRunOptions::builder()
                .assay_id(1)
                .source(ImportRunSource::RunFilePath(
                    "/files/assay/run.tsv".to_string(),
                ))
                .use_json(false)
                .build(),
        );

        assert_eq!(explicit_false, default_mode);
    }

    #[test]
    fn get_study_nab_graph_url_params_emit_repeated_id_keys() {
        let options = GetStudyNabGraphUrlOptions::builder()
            .object_ids(vec![
                "obj-1".to_string(),
                "obj-2".to_string(),
                "obj-3".to_string(),
            ])
            .chart_title("My Chart".to_string())
            .fit_type(FitType::FourParameter)
            .height(400)
            .width(600)
            .build();

        let params = build_get_study_nab_graph_url_params(&options);

        let id_params: Vec<_> = params.iter().filter(|(k, _)| k == "id").collect();
        assert_eq!(
            id_params.len(),
            3,
            "each objectId should be a separate 'id' param"
        );
        assert_eq!(id_params[0].1, "obj-1");
        assert_eq!(id_params[1].1, "obj-2");
        assert_eq!(id_params[2].1, "obj-3");

        assert!(params.contains(&("chartTitle".into(), "My Chart".into())));
        assert!(params.contains(&("fitType".into(), "FOUR_PARAMETER".into())));
        assert!(params.contains(&("height".into(), "400".into())));
        assert!(params.contains(&("width".into(), "600".into())));
    }

    #[test]
    fn get_study_nab_graph_url_params_omit_optional_fields_when_absent() {
        let options = GetStudyNabGraphUrlOptions::builder()
            .object_ids(vec!["obj-1".to_string()])
            .build();

        let params = build_get_study_nab_graph_url_params(&options);

        assert_eq!(
            params.len(),
            1,
            "only the single id param should be emitted when all optionals are None"
        );
        assert_eq!(params[0], ("id".into(), "obj-1".into()));
    }

    #[test]
    fn get_study_nab_runs_params_emit_repeated_object_ids_keys() {
        let options = GetStudyNabRunsOptions::builder()
            .object_ids(vec!["run-a".to_string(), "run-b".to_string()])
            .calculate_neut(true)
            .include_fit_parameters(false)
            .include_stats(true)
            .include_wells(false)
            .build();

        let params = build_get_study_nab_runs_params(&options);

        let obj_params: Vec<_> = params.iter().filter(|(k, _)| k == "objectIds").collect();
        assert_eq!(
            obj_params.len(),
            2,
            "each objectId should be a separate 'objectIds' param"
        );
        assert_eq!(obj_params[0].1, "run-a");
        assert_eq!(obj_params[1].1, "run-b");

        assert!(params.contains(&("calculateNeut".into(), "true".into())));
        assert!(params.contains(&("includeFitParameters".into(), "false".into())));
        assert!(params.contains(&("includeStats".into(), "true".into())));
        assert!(params.contains(&("includeWells".into(), "false".into())));
    }

    #[test]
    fn get_study_nab_runs_params_omit_optional_fields_when_absent() {
        let options = GetStudyNabRunsOptions::builder()
            .object_ids(vec!["run-a".to_string()])
            .build();

        let params = build_get_study_nab_runs_params(&options);

        assert_eq!(
            params.len(),
            1,
            "only the single objectIds param should be emitted when all optionals are None"
        );
        assert_eq!(params[0], ("objectIds".into(), "run-a".into()));
    }

    #[test]
    fn batch_type_alias_is_run_group() {
        // Batch is a type alias for RunGroup, so constructing via
        // deserialization should produce the same type.
        let json = serde_json::json!({
            "name": "Test Batch",
            "runs": [
                {"name": "Run 1"},
                {"name": "Run 2"}
            ]
        });
        let batch: Batch = serde_json::from_value(json).expect("should deserialize");
        assert_eq!(batch.exp_object.name.as_deref(), Some("Test Batch"));
        assert_eq!(batch.runs.len(), 2);
        assert_eq!(batch.runs[0].exp_object.name.as_deref(), Some("Run 1"));
    }

    #[test]
    fn get_assay_batch_body_serializes_expected_keys() {
        let body = GetAssayBatchBody {
            protocol_name: "General".to_string(),
            batch_id: 42,
        };
        let json = serde_json::to_value(&body).expect("should serialize");
        assert_eq!(json["protocolName"], "General");
        assert_eq!(json["batchId"], 42);
    }

    #[test]
    fn save_assay_batch_body_serializes_assay_id_mode() {
        let batch: Batch =
            serde_json::from_value(serde_json::json!({})).expect("empty JSON should deserialize");
        let body = SaveAssayBatchBody {
            assay_id: Some(7),
            protocol_name: None,
            batch,
        };
        let json = serde_json::to_value(&body).expect("should serialize");
        assert_eq!(json["assayId"], 7);
        assert!(json.get("protocolName").is_none());
        assert!(json.get("batch").is_some());
    }

    #[test]
    fn save_assay_batch_body_serializes_protocol_name_mode() {
        let batch: Batch =
            serde_json::from_value(serde_json::json!({})).expect("empty JSON should deserialize");
        let body = SaveAssayBatchBody {
            assay_id: None,
            protocol_name: Some("Sample Derivation Protocol".to_string()),
            batch,
        };
        let json = serde_json::to_value(&body).expect("should serialize");
        assert!(json.get("assayId").is_none());
        assert_eq!(json["protocolName"], "Sample Derivation Protocol");
        assert!(json.get("batch").is_some());
    }

    #[test]
    fn save_assay_runs_body_serializes_protocol_and_runs() {
        let run: Run =
            serde_json::from_value(serde_json::json!({"name": "R1"})).expect("should deserialize");
        let body = SaveAssayRunsBody {
            protocol_name: "General".to_string(),
            runs: vec![run],
        };
        let json = serde_json::to_value(&body).expect("should serialize");
        assert_eq!(json["protocolName"], "General");
        let runs = json["runs"].as_array().expect("runs should be an array");
        assert_eq!(runs.len(), 1);
    }

    #[test]
    fn save_assay_batch_response_deserializes_envelope() {
        let json = serde_json::json!({
            "assayId": 7,
            "batch": {
                "name": "Batch 1",
                "runs": [{"name": "Run 1"}]
            }
        });
        let response: SaveAssayBatchResponse =
            serde_json::from_value(json).expect("should deserialize");
        assert_eq!(response.assay_id, Some(7));
        assert_eq!(response.batch.exp_object.name.as_deref(), Some("Batch 1"));
        assert_eq!(response.batch.runs.len(), 1);
    }

    #[test]
    fn save_assay_runs_response_deserializes_runs_array() {
        let json = serde_json::json!({
            "runs": [
                {"name": "Run A"},
                {"name": "Run B"}
            ]
        });
        let response: SaveAssayRunsResponse =
            serde_json::from_value(json).expect("should deserialize");
        assert_eq!(response.runs.len(), 2);
        assert_eq!(response.runs[0].exp_object.name.as_deref(), Some("Run A"));
        assert_eq!(response.runs[1].exp_object.name.as_deref(), Some("Run B"));
    }

    #[test]
    fn get_assay_batch_envelope_extracts_batch() {
        let json = serde_json::json!({
            "batch": {
                "name": "Loaded Batch",
                "runs": []
            }
        });
        let envelope: GetAssayBatchEnvelope =
            serde_json::from_value(json).expect("should deserialize");
        assert_eq!(
            envelope.batch.exp_object.name.as_deref(),
            Some("Loaded Batch")
        );
        assert!(envelope.batch.runs.is_empty());
    }

    #[test]
    fn validate_get_assay_batch_rejects_blank_protocol_name() {
        let err = validate_get_assay_batch_options(&GetAssayBatchOptions {
            protocol_name: "  ".to_string(),
            batch_id: 1,
            container_path: None,
        })
        .expect_err("should reject blank protocol name");
        assert!(matches!(err, LabkeyError::InvalidInput(_)));
    }

    #[test]
    fn validate_save_assay_batch_rejects_non_positive_assay_id() {
        let batch: Batch =
            serde_json::from_value(serde_json::json!({})).expect("empty JSON should deserialize");
        let err = validate_save_assay_batch_options(&SaveAssayBatchOptions {
            identifier: BatchIdentifier::ByAssayId(0),
            batch,
            container_path: None,
        })
        .expect_err("should reject non-positive assay_id");
        assert!(matches!(err, LabkeyError::InvalidInput(_)));
    }

    #[test]
    fn validate_save_assay_batch_rejects_blank_protocol_name() {
        let batch: Batch =
            serde_json::from_value(serde_json::json!({})).expect("empty JSON should deserialize");
        let err = validate_save_assay_batch_options(&SaveAssayBatchOptions {
            identifier: BatchIdentifier::ByProtocolName(String::new()),
            batch,
            container_path: None,
        })
        .expect_err("should reject blank protocol name");
        assert!(matches!(err, LabkeyError::InvalidInput(_)));
    }

    #[test]
    fn validate_save_assay_batch_accepts_valid_inputs() {
        let batch: Batch =
            serde_json::from_value(serde_json::json!({})).expect("empty JSON should deserialize");
        validate_save_assay_batch_options(&SaveAssayBatchOptions {
            identifier: BatchIdentifier::ByAssayId(7),
            batch: batch.clone(),
            container_path: None,
        })
        .expect("positive assay_id should be valid");

        validate_save_assay_batch_options(&SaveAssayBatchOptions {
            identifier: BatchIdentifier::ByProtocolName("General".to_string()),
            batch,
            container_path: None,
        })
        .expect("non-empty protocol name should be valid");
    }

    #[test]
    fn validate_save_assay_runs_rejects_blank_protocol_name() {
        let run: Run =
            serde_json::from_value(serde_json::json!({})).expect("empty JSON should deserialize");
        let err = validate_save_assay_runs_options(&SaveAssayRunsOptions {
            protocol_name: "  ".to_string(),
            runs: vec![run],
            container_path: None,
        })
        .expect_err("should reject blank protocol name");
        assert!(matches!(err, LabkeyError::InvalidInput(_)));
    }

    #[test]
    fn validate_save_assay_runs_rejects_empty_runs() {
        let err = validate_save_assay_runs_options(&SaveAssayRunsOptions {
            protocol_name: "General".to_string(),
            runs: vec![],
            container_path: None,
        })
        .expect_err("should reject empty runs");
        assert!(matches!(err, LabkeyError::InvalidInput(_)));
    }
}
