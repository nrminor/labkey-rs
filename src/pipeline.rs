//! Pipeline models and API endpoints.

use std::time::Duration;

use serde::Deserialize;

use crate::{
    client::{LabkeyClient, RequestOptions},
    common::opt,
    error::LabkeyError,
};

const DEFAULT_LONG_TIMEOUT: Duration = Duration::from_millis(60_000_000);

/// Response payload from [`LabkeyClient::get_file_status`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GetFileStatusResponse {
    /// File status objects returned by the server.
    #[serde(default)]
    pub files: Vec<serde_json::Value>,
    /// Action that would be performed (for example `Retry` or `Analyze`).
    #[serde(default)]
    pub submit_type: Option<String>,
}

/// Options for [`LabkeyClient::get_file_status`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetFileStatusOptions {
    /// Names of files within the pipeline path.
    pub files: Vec<String>,
    /// Relative path from the folder's pipeline root.
    pub path: String,
    /// Name of the analysis protocol.
    pub protocol_name: String,
    /// Identifier for the pipeline task.
    pub task_id: String,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Optional request timeout.
    pub timeout: Option<Duration>,
}

/// Response payload from [`LabkeyClient::get_pipeline_container`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PipelineContainerResponse {
    /// The container path where the pipeline is defined.
    #[serde(default)]
    pub container_path: Option<String>,
    /// The `WebDAV` URL for the pipeline root.
    #[serde(default)]
    #[serde(rename = "webDavURL")]
    pub web_dav_url: Option<String>,
}

/// Options for [`LabkeyClient::get_pipeline_container`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetPipelineContainerOptions {
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Response payload from [`LabkeyClient::get_protocols`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GetProtocolsResponse {
    /// Saved protocols for the requested pipeline task.
    #[serde(default)]
    pub protocols: Vec<serde_json::Value>,
    /// Default protocol name selected by the server.
    #[serde(default)]
    pub default_protocol_name: Option<String>,
}

/// Options for [`LabkeyClient::get_protocols`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetProtocolsOptions {
    /// Relative path from the folder's pipeline root.
    pub path: String,
    /// Identifier for the pipeline task.
    pub task_id: String,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Whether protocols from child workbooks should be included.
    pub include_workbooks: Option<bool>,
}

/// Options for [`LabkeyClient::start_analysis`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct StartAnalysisOptions {
    /// Data ids used as pipeline inputs.
    pub file_ids: Vec<i64>,
    /// File names within the selected pipeline path.
    pub files: Vec<String>,
    /// Relative path from the folder's pipeline root.
    pub path: String,
    /// Name of the analysis protocol.
    pub protocol_name: String,
    /// Identifier for the pipeline task.
    pub task_id: String,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
    /// Whether non-existent files are allowed.
    pub allow_non_existent_files: Option<bool>,
    /// JSON protocol description; this is encoded to a JSON string as `configureJson`.
    pub json_parameters: Option<serde_json::Value>,
    /// XML protocol description mapped to `configureXml`.
    pub xml_parameters: Option<String>,
    /// Description displayed in pipeline UI.
    pub pipeline_description: Option<String>,
    /// Description of the protocol.
    pub protocol_description: Option<String>,
    /// Whether to save protocol definition for reuse. Defaults to `true` when omitted.
    pub save_protocol: Option<bool>,
    /// Optional request timeout.
    pub timeout: Option<Duration>,
}

impl LabkeyClient {
    /// Get pipeline file status for a protocol and task.
    ///
    /// Sends a POST request to `pipeline-analysis-getFileStatus.api` with input
    /// values encoded as query parameters.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, or the response body cannot be deserialized.
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
    /// use labkey_rs::pipeline::GetFileStatusOptions;
    ///
    /// let status = client
    ///     .get_file_status(
    ///         GetFileStatusOptions::builder()
    ///             .files(vec!["run1.tsv".to_string()])
    ///             .path("imports".to_string())
    ///             .protocol_name("RNAseq".to_string())
    ///             .task_id("pipeline-task".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{} files", status.files.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_file_status(
        &self,
        options: GetFileStatusOptions,
    ) -> Result<GetFileStatusResponse, LabkeyError> {
        validate_get_file_status_options(&options)?;
        let url = self.build_url(
            "pipeline-analysis",
            "getFileStatus.api",
            options.container_path.as_deref(),
        );
        let params = build_get_file_status_params(&options);
        let request_options = RequestOptions {
            timeout: options.timeout.or(Some(DEFAULT_LONG_TIMEOUT)),
            ..RequestOptions::default()
        };
        self.post_with_params_with_options(url, &params, &request_options)
            .await
    }

    /// Get the container where the pipeline is defined.
    ///
    /// Sends a GET request to `pipeline-getPipelineContainer.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, or the response body cannot be deserialized.
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
    /// use labkey_rs::pipeline::GetPipelineContainerOptions;
    ///
    /// let pipeline_container = client
    ///     .get_pipeline_container(GetPipelineContainerOptions::builder().build())
    ///     .await?;
    ///
    /// println!("{:?}", pipeline_container.container_path);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_pipeline_container(
        &self,
        options: GetPipelineContainerOptions,
    ) -> Result<PipelineContainerResponse, LabkeyError> {
        let url = self.build_url(
            "pipeline",
            "getPipelineContainer.api",
            options.container_path.as_deref(),
        );
        self.get(url, &[]).await
    }

    /// Get saved protocols for a pipeline task.
    ///
    /// Sends a POST request to `pipeline-analysis-getSavedProtocols.api` with
    /// inputs encoded as query parameters.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, or the response body cannot be deserialized.
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
    /// use labkey_rs::pipeline::GetProtocolsOptions;
    ///
    /// let protocols = client
    ///     .get_protocols(
    ///         GetProtocolsOptions::builder()
    ///             .path("imports".to_string())
    ///             .task_id("pipeline-task".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{} protocols", protocols.protocols.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_protocols(
        &self,
        options: GetProtocolsOptions,
    ) -> Result<GetProtocolsResponse, LabkeyError> {
        validate_get_protocols_options(&options)?;
        let url = self.build_url(
            "pipeline-analysis",
            "getSavedProtocols.api",
            options.container_path.as_deref(),
        );
        let params = build_get_protocols_params(&options);
        self.post_with_params_with_options(url, &params, &RequestOptions::default())
            .await
    }

    /// Start pipeline analysis for provided files and protocol settings.
    ///
    /// Sends a POST request to `pipeline-analysis-startAnalysis.api` with
    /// inputs encoded as query parameters.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, or the response body cannot be deserialized.
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
    /// use labkey_rs::pipeline::StartAnalysisOptions;
    ///
    /// let response = client
    ///     .start_analysis(
    ///         StartAnalysisOptions::builder()
    ///             .file_ids(vec![101, 102])
    ///             .files(vec!["run1.tsv".to_string(), "run2.tsv".to_string()])
    ///             .path("imports".to_string())
    ///             .protocol_name("RNAseq".to_string())
    ///             .task_id("pipeline-task".to_string())
    ///             .json_parameters(serde_json::json!({ "version": 1 }))
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", response["success"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn start_analysis(
        &self,
        options: StartAnalysisOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        validate_start_analysis_options(&options)?;

        let url = self.build_url(
            "pipeline-analysis",
            "startAnalysis.api",
            options.container_path.as_deref(),
        );
        let params = build_start_analysis_params(&options)?;
        let request_options = RequestOptions {
            timeout: options.timeout.or(Some(DEFAULT_LONG_TIMEOUT)),
            ..RequestOptions::default()
        };
        self.post_with_params_with_options(url, &params, &request_options)
            .await
    }
}

fn build_get_file_status_params(options: &GetFileStatusOptions) -> Vec<(String, String)> {
    let mut params = vec![
        ("path".to_string(), options.path.clone()),
        ("protocolName".to_string(), options.protocol_name.clone()),
        ("taskId".to_string(), options.task_id.clone()),
    ];
    params.extend(
        options
            .files
            .iter()
            .cloned()
            .map(|file| ("file".to_string(), file)),
    );
    params
}

fn build_get_protocols_params(options: &GetProtocolsOptions) -> Vec<(String, String)> {
    [
        opt(
            "includeWorkbooks",
            Some(options.include_workbooks.unwrap_or(false)),
        ),
        opt("path", Some(options.path.as_str())),
        opt("taskId", Some(options.task_id.as_str())),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn build_start_analysis_params(
    options: &StartAnalysisOptions,
) -> Result<Vec<(String, String)>, LabkeyError> {
    let mut params = [
        opt("allowNonExistentFiles", options.allow_non_existent_files),
        opt("path", Some(options.path.as_str())),
        opt(
            "pipelineDescription",
            options.pipeline_description.as_deref(),
        ),
        opt(
            "protocolDescription",
            options.protocol_description.as_deref(),
        ),
        opt("protocolName", Some(options.protocol_name.as_str())),
        opt("saveProtocol", Some(options.save_protocol.unwrap_or(true))),
        opt("taskId", Some(options.task_id.as_str())),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    params.extend(
        options
            .files
            .iter()
            .cloned()
            .map(|file| ("file".to_string(), file)),
    );
    params.extend(
        options
            .file_ids
            .iter()
            .map(|id| ("fileIds".to_string(), id.to_string())),
    );

    if let Some(xml_parameters) = &options.xml_parameters {
        params.push(("configureXml".to_string(), xml_parameters.clone()));
    } else if let Some(json_parameters) = &options.json_parameters {
        params.push((
            "configureJson".to_string(),
            serde_json::to_string(json_parameters)?,
        ));
    }

    Ok(params)
}

fn validate_start_analysis_options(options: &StartAnalysisOptions) -> Result<(), LabkeyError> {
    validate_required_non_blank("protocol_name", &options.protocol_name, "start_analysis")?;
    validate_required_non_blank("path", &options.path, "start_analysis")?;
    validate_required_non_blank("task_id", &options.task_id, "start_analysis")?;

    if options.files.is_empty() {
        return Err(LabkeyError::InvalidInput(
            "start_analysis requires at least one file".to_string(),
        ));
    }
    if options.files.iter().any(|file| file.trim().is_empty()) {
        return Err(LabkeyError::InvalidInput(
            "start_analysis files cannot contain blank values".to_string(),
        ));
    }
    if options.file_ids.is_empty() {
        return Err(LabkeyError::InvalidInput(
            "start_analysis requires at least one file_id".to_string(),
        ));
    }

    Ok(())
}

fn validate_get_file_status_options(options: &GetFileStatusOptions) -> Result<(), LabkeyError> {
    validate_required_non_blank("path", &options.path, "get_file_status")?;
    validate_required_non_blank("protocol_name", &options.protocol_name, "get_file_status")?;
    validate_required_non_blank("task_id", &options.task_id, "get_file_status")?;

    if options.files.is_empty() {
        return Err(LabkeyError::InvalidInput(
            "get_file_status requires at least one file".to_string(),
        ));
    }
    if options.files.iter().any(|file| file.trim().is_empty()) {
        return Err(LabkeyError::InvalidInput(
            "get_file_status files cannot contain blank values".to_string(),
        ));
    }

    Ok(())
}

fn validate_get_protocols_options(options: &GetProtocolsOptions) -> Result<(), LabkeyError> {
    validate_required_non_blank("path", &options.path, "get_protocols")?;
    validate_required_non_blank("task_id", &options.task_id, "get_protocols")?;

    Ok(())
}

fn validate_required_non_blank(
    field_name: &str,
    value: &str,
    endpoint: &str,
) -> Result<(), LabkeyError> {
    if value.trim().is_empty() {
        return Err(LabkeyError::InvalidInput(format!(
            "{endpoint} requires a non-empty {field_name}"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClientConfig, Credential};

    fn test_client(base_url: &str, container_path: &str) -> LabkeyClient {
        LabkeyClient::new(ClientConfig::new(
            base_url,
            Credential::ApiKey("test-key".to_string()),
            container_path,
        ))
        .expect("valid client config")
    }

    #[test]
    fn pipeline_endpoint_urls_match_expected_actions() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url(
                    "pipeline-analysis",
                    "getFileStatus.api",
                    Some("/Alt/Container")
                )
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/pipeline-analysis-getFileStatus.api"
        );
        assert_eq!(
            client
                .build_url(
                    "pipeline",
                    "getPipelineContainer.api",
                    Some("/Alt/Container")
                )
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/pipeline-getPipelineContainer.api"
        );
        assert_eq!(
            client
                .build_url(
                    "pipeline-analysis",
                    "getSavedProtocols.api",
                    Some("/Alt/Container")
                )
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/pipeline-analysis-getSavedProtocols.api"
        );
        assert_eq!(
            client
                .build_url(
                    "pipeline-analysis",
                    "startAnalysis.api",
                    Some("/Alt/Container")
                )
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/pipeline-analysis-startAnalysis.api"
        );
    }

    #[test]
    fn start_analysis_params_use_configure_keys_and_json_string_encoding() {
        let options = StartAnalysisOptions::builder()
            .file_ids(vec![10, 11])
            .files(vec!["input1.tsv".to_string(), "input2.tsv".to_string()])
            .path("pipeline/path".to_string())
            .protocol_name("RNAseq".to_string())
            .task_id("task-1".to_string())
            .json_parameters(serde_json::json!({"alpha": 1, "beta": true}))
            .build();

        let params = build_start_analysis_params(&options).expect("params should build");

        assert!(params.contains(&(
            "configureJson".to_string(),
            "{\"alpha\":1,\"beta\":true}".to_string()
        )));
        assert!(!params.iter().any(|(key, _)| key == "jsonParameters"));
        assert!(params.contains(&("file".to_string(), "input1.tsv".to_string())));
        assert!(params.contains(&("fileIds".to_string(), "10".to_string())));
    }

    #[test]
    fn start_analysis_params_prefer_configure_xml_when_both_config_formats_are_set() {
        let options = StartAnalysisOptions::builder()
            .file_ids(vec![1])
            .files(vec!["input1.tsv".to_string()])
            .path("pipeline/path".to_string())
            .protocol_name("RNAseq".to_string())
            .task_id("task-1".to_string())
            .xml_parameters("<bioml />".to_string())
            .json_parameters(serde_json::json!({"alpha": 1}))
            .build();

        let params = build_start_analysis_params(&options).expect("params should build");

        assert!(params.contains(&("configureXml".to_string(), "<bioml />".to_string())));
        assert!(!params.iter().any(|(key, _)| key == "configureJson"));
    }

    #[test]
    fn get_protocols_params_include_workbooks_defaults_to_false() {
        let options = GetProtocolsOptions::builder()
            .path("imports".to_string())
            .task_id("task-1".to_string())
            .build();

        let params = build_get_protocols_params(&options);

        assert!(params.contains(&("includeWorkbooks".to_string(), "false".to_string())));
        assert!(params.contains(&("path".to_string(), "imports".to_string())));
        assert!(params.contains(&("taskId".to_string(), "task-1".to_string())));
    }

    #[test]
    fn pipeline_container_response_deserializes_happy_path() {
        let response: PipelineContainerResponse = serde_json::from_value(serde_json::json!({
            "containerPath": "/Home/Project",
            "webDavURL": "https://labkey.example.com/_webdav/Home/Project"
        }))
        .expect("response should deserialize");

        assert_eq!(response.container_path.as_deref(), Some("/Home/Project"));
        assert_eq!(
            response.web_dav_url.as_deref(),
            Some("https://labkey.example.com/_webdav/Home/Project")
        );
    }

    #[test]
    fn pipeline_container_response_deserializes_minimal_path() {
        let response: PipelineContainerResponse =
            serde_json::from_value(serde_json::json!({})).expect("response should deserialize");

        assert!(response.container_path.is_none());
        assert!(response.web_dav_url.is_none());
    }

    #[test]
    fn start_analysis_rejects_blank_protocol_name() {
        let options = StartAnalysisOptions::builder()
            .file_ids(vec![1])
            .files(vec!["input1.tsv".to_string()])
            .path("pipeline/path".to_string())
            .protocol_name("  ".to_string())
            .task_id("task-1".to_string())
            .build();

        let error = validate_start_analysis_options(&options).expect_err("should fail");
        assert!(matches!(error, LabkeyError::InvalidInput(_)));
    }

    #[test]
    fn start_analysis_rejects_missing_files_and_file_ids() {
        let options = StartAnalysisOptions::builder()
            .file_ids(vec![])
            .files(vec![])
            .path("pipeline/path".to_string())
            .protocol_name("RNAseq".to_string())
            .task_id("task-1".to_string())
            .build();

        let error = validate_start_analysis_options(&options).expect_err("should fail");
        assert!(matches!(error, LabkeyError::InvalidInput(_)));
    }

    #[test]
    fn get_file_status_validation_rejects_blank_path_and_empty_files() {
        let options = GetFileStatusOptions::builder()
            .files(vec![])
            .path("  ".to_string())
            .protocol_name("RNAseq".to_string())
            .task_id("task-1".to_string())
            .build();

        let error = validate_get_file_status_options(&options).expect_err("should fail");
        assert!(matches!(error, LabkeyError::InvalidInput(_)));
    }

    #[test]
    fn get_protocols_validation_rejects_blank_task_id() {
        let options = GetProtocolsOptions::builder()
            .path("imports".to_string())
            .task_id("  ".to_string())
            .build();

        let error = validate_get_protocols_options(&options).expect_err("should fail");
        assert!(matches!(error, LabkeyError::InvalidInput(_)));
    }
}
