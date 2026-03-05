//! Assay models and API endpoints for assay listing and `NAb` queries.

use std::collections::HashMap;
use std::time::Duration;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    client::{LabkeyClient, RequestOptions},
    common::opt,
    domain::DomainDesign,
    error::LabkeyError,
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
    parameters: GetAssaysParameters,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetAssaysParameters {
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

impl LabkeyClient {
    /// Get assay designs in a container.
    ///
    /// Sends a POST request to `assay-assayList.api` with all filter values
    /// nested under a top-level `parameters` object.
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
    /// println!("{}", run["name"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_assay_run(
        &self,
        options: GetAssayRunOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        validate_get_assay_run_options(&options)?;
        let url = self.build_url("assay", "getAssayRun", options.container_path.as_deref());
        let body = GetAssayRunBody { lsid: options.lsid };
        self.post(url, &body).await
    }
}

fn build_get_assays_body(options: &GetAssaysOptions) -> GetAssaysBody {
    GetAssaysBody {
        parameters: GetAssaysParameters {
            id: options.id,
            name: options.name.clone(),
            plate_enabled: options.plate_enabled,
            status: options.status.clone(),
            type_: options.type_.clone(),
        },
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
    fn get_assays_body_wraps_filters_in_parameters_object() {
        let options = GetAssaysOptions::builder()
            .id(7)
            .name("MyAssay".to_string())
            .plate_enabled(true)
            .status("Active".to_string())
            .type_("General".to_string())
            .build();

        let body = serde_json::to_value(build_get_assays_body(&options)).expect("serialize body");
        assert_eq!(body["parameters"]["id"], 7);
        assert_eq!(body["parameters"]["name"], "MyAssay");
        assert_eq!(body["parameters"]["plateEnabled"], true);
        assert_eq!(body["parameters"]["status"], "Active");
        assert_eq!(body["parameters"]["type"], "General");
        assert!(body.get("id").is_none());
        assert!(body.get("name").is_none());
    }

    #[test]
    fn get_assays_body_omits_unset_parameters() {
        let options = GetAssaysOptions::builder().build();
        let body = serde_json::to_value(build_get_assays_body(&options)).expect("serialize body");

        assert!(body.get("parameters").is_some());
        assert!(
            body["parameters"]
                .as_object()
                .is_some_and(serde_json::Map::is_empty)
        );
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
            F: std::future::Future<Output = Result<serde_json::Value, LabkeyError>>,
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
}
