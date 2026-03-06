//! Data Integration (DI) transform endpoints.
//!
//! These endpoints correspond to the Java client's `di` package
//! (`BaseTransformCommand`, `RunTransformCommand`,
//! `ResetTransformStateCommand`, `UpdateTransformConfigurationCommand`).
//! There is no JS client equivalent.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{client::LabkeyClient, error::LabkeyError};

/// Selector for identifying a DI transform.
///
/// Both variants serialize as the `transformId` wire key, matching the Java
/// client's `BaseTransformCommand` which accepts a single string parameter
/// that the server interprets as either a numeric id or a name.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum TransformSelector {
    /// Identify a transform by numeric id.
    Id(i64),
    /// Identify a transform by name.
    Name(String),
}

/// Options for [`LabkeyClient::run_transform`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct RunTransformOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Transform selector (sent as `transformId` on the wire).
    pub selector: TransformSelector,
}

/// Response payload from [`LabkeyClient::run_transform`].
///
/// Matches the Java `RunTransformResponse` which reads flat top-level keys
/// `success`, `jobId`, `pipelineURL`, and `status`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RunTransformResponse {
    /// Whether the transform was queued successfully.
    #[serde(default)]
    pub success: Option<bool>,
    /// Pipeline job id for the queued transform.
    #[serde(default)]
    pub job_id: Option<String>,
    /// URL for the pipeline status page for this job.
    #[serde(default, rename = "pipelineURL")]
    pub pipeline_url: Option<String>,
    /// Job status: `"success"`, `"complete"`, `"error"`, or `"no work"`.
    #[serde(default)]
    pub status: Option<String>,
    /// Additional server-provided fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Options for [`LabkeyClient::reset_transform_state`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct ResetTransformStateOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Transform selector (sent as `transformId` on the wire).
    pub selector: TransformSelector,
}

/// Response payload from [`LabkeyClient::reset_transform_state`].
///
/// Matches the Java `ResetTransformStateResponse` which only exposes the
/// `success` field inherited from `BaseTransformResponse`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResetTransformStateResponse {
    /// Whether the state was reset successfully.
    #[serde(default)]
    pub success: Option<bool>,
    /// Additional server-provided fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Options for [`LabkeyClient::update_transform_configuration`].
///
/// Matches the Java `UpdateTransformConfigurationCommand` which sends
/// `transformId` plus optional top-level `enabled` and `verboseLogging`.
/// When neither optional is set, the endpoint acts as a read-only query
/// returning the current configuration.
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct UpdateTransformConfigurationOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Transform selector (sent as `transformId` on the wire).
    pub selector: TransformSelector,
    /// Set to `true` to enable the transform's scheduled runs.
    pub enabled: Option<bool>,
    /// Set to `true` to enable verbose logging for the transform.
    pub verbose_logging: Option<bool>,
}

/// Nested result object inside [`UpdateTransformConfigurationResponse`].
///
/// Matches the Java `UpdateTransformConfigurationResponse` accessors that
/// read from the `result` sub-object.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TransformConfigurationResult {
    /// Whether the transform is enabled for scheduled runs.
    #[serde(default)]
    pub enabled: Option<bool>,
    /// Whether verbose logging is enabled.
    #[serde(default)]
    pub verbose_logging: Option<bool>,
    /// State saved after the last transform run (row counts, filter values,
    /// persisted stored-procedure parameters, etc.).
    #[serde(default)]
    pub state: Option<serde_json::Value>,
    /// When the transform last checked for work.
    #[serde(default)]
    pub last_checked: Option<String>,
    /// The transform name / description id.
    #[serde(default)]
    pub description_id: Option<String>,
    /// Additional server-provided fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response payload from [`LabkeyClient::update_transform_configuration`].
///
/// Matches the Java `UpdateTransformConfigurationResponse` which reads
/// `success` at the top level and configuration fields from a nested
/// `result` object.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UpdateTransformConfigurationResponse {
    /// Whether the operation succeeded.
    #[serde(default)]
    pub success: Option<bool>,
    /// Nested configuration result.
    #[serde(default)]
    pub result: Option<TransformConfigurationResult>,
    /// Additional server-provided fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

// ---------------------------------------------------------------------------
// Internal request body structs
// ---------------------------------------------------------------------------

/// Body for `runTransform` and `resetTransformState`. Java's
/// `BaseTransformCommand.getJsonObject()` sends exactly `{"transformId": "..."}`.
#[derive(Debug, Serialize)]
struct TransformIdBody {
    #[serde(rename = "transformId")]
    transform_id: String,
}

/// Body for `UpdateTransformConfiguration`. Java's override of
/// `getJsonObject()` adds optional `enabled` and `verboseLogging` alongside
/// the base `transformId`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateTransformConfigurationBody {
    transform_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    verbose_logging: Option<bool>,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn validate_transform_selector(
    selector: &TransformSelector,
    operation: &str,
) -> Result<(), LabkeyError> {
    match selector {
        TransformSelector::Id(value) => {
            if *value <= 0 {
                return Err(LabkeyError::InvalidInput(format!(
                    "{operation} requires transform_id > 0"
                )));
            }
        }
        TransformSelector::Name(value) => {
            if value.trim().is_empty() {
                return Err(LabkeyError::InvalidInput(format!(
                    "{operation} requires non-blank transform_name"
                )));
            }
        }
    }
    Ok(())
}

/// Convert a selector to the wire string. Java's `BaseTransformCommand`
/// stores the transform id as a `String` regardless of whether it was
/// originally a numeric id or a name.
fn selector_to_wire_string(selector: TransformSelector) -> String {
    match selector {
        TransformSelector::Id(value) => value.to_string(),
        TransformSelector::Name(value) => value,
    }
}

impl LabkeyClient {
    /// Run a DI transform through `dataintegration-runTransform`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if input validation fails, the request fails, or
    /// the response cannot be parsed.
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
    /// use labkey_rs::di::{RunTransformOptions, TransformSelector};
    ///
    /// let _ = client
    ///     .run_transform(
    ///         RunTransformOptions::builder()
    ///             .selector(TransformSelector::Name("LoadFromStaging".to_string()))
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run_transform(
        &self,
        options: RunTransformOptions,
    ) -> Result<RunTransformResponse, LabkeyError> {
        validate_transform_selector(&options.selector, "run_transform")?;
        let url = self.build_url(
            "dataintegration",
            "runTransform",
            options.container_path.as_deref(),
        );
        let body = TransformIdBody {
            transform_id: selector_to_wire_string(options.selector),
        };
        self.post(url, &body).await
    }

    /// Reset a DI transform state through `dataintegration-resetTransformState`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if input validation fails, the request fails, or
    /// the response cannot be parsed.
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
    /// use labkey_rs::di::{ResetTransformStateOptions, TransformSelector};
    ///
    /// let _ = client
    ///     .reset_transform_state(
    ///         ResetTransformStateOptions::builder()
    ///             .selector(TransformSelector::Id(42))
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn reset_transform_state(
        &self,
        options: ResetTransformStateOptions,
    ) -> Result<ResetTransformStateResponse, LabkeyError> {
        validate_transform_selector(&options.selector, "reset_transform_state")?;
        let url = self.build_url(
            "dataintegration",
            "resetTransformState",
            options.container_path.as_deref(),
        );
        let body = TransformIdBody {
            transform_id: selector_to_wire_string(options.selector),
        };
        self.post(url, &body).await
    }

    /// Update DI transform configuration through
    /// `dataintegration-UpdateTransformConfiguration`.
    ///
    /// When neither `enabled` nor `verbose_logging` is set, the endpoint acts
    /// as a read-only query returning the current configuration in the
    /// `result` envelope.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if input validation fails, the request fails, or
    /// the response cannot be parsed.
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
    /// use labkey_rs::di::{TransformSelector, UpdateTransformConfigurationOptions};
    ///
    /// let response = client
    ///     .update_transform_configuration(
    ///         UpdateTransformConfigurationOptions::builder()
    ///             .selector(TransformSelector::Name("LoadFromStaging".to_string()))
    ///             .enabled(true)
    ///             .verbose_logging(false)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// if let Some(result) = &response.result {
    ///     println!("enabled: {:?}", result.enabled);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_transform_configuration(
        &self,
        options: UpdateTransformConfigurationOptions,
    ) -> Result<UpdateTransformConfigurationResponse, LabkeyError> {
        validate_transform_selector(&options.selector, "update_transform_configuration")?;
        let url = self.build_url(
            "dataintegration",
            "UpdateTransformConfiguration",
            options.container_path.as_deref(),
        );
        let body = UpdateTransformConfigurationBody {
            transform_id: selector_to_wire_string(options.selector),
            enabled: options.enabled,
            verbose_logging: options.verbose_logging,
        };
        self.post(url, &body).await
    }
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
    fn di_endpoint_urls_match_expected_actions_without_api_suffix() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url("dataintegration", "runTransform", Some("/Alt/Container"),)
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/dataintegration-runTransform"
        );
        assert_eq!(
            client
                .build_url(
                    "dataintegration",
                    "resetTransformState",
                    Some("/Alt/Container"),
                )
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/dataintegration-resetTransformState"
        );
        assert_eq!(
            client
                .build_url(
                    "dataintegration",
                    "UpdateTransformConfiguration",
                    Some("/Alt/Container"),
                )
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/dataintegration-UpdateTransformConfiguration"
        );
    }

    // -- Request body serialization tests --
    // These verify the exact JSON shape that Java's BaseTransformCommand and
    // UpdateTransformConfigurationCommand produce.

    #[test]
    fn transform_id_body_sends_string_transform_id_for_numeric_selector() {
        let body = TransformIdBody {
            transform_id: selector_to_wire_string(TransformSelector::Id(42)),
        };
        let json = serde_json::to_value(body).expect("should serialize");
        assert_eq!(json, serde_json::json!({"transformId": "42"}));
    }

    #[test]
    fn transform_id_body_sends_string_transform_id_for_name_selector() {
        let body = TransformIdBody {
            transform_id: selector_to_wire_string(TransformSelector::Name(
                "LoadFromStaging".to_string(),
            )),
        };
        let json = serde_json::to_value(body).expect("should serialize");
        assert_eq!(json, serde_json::json!({"transformId": "LoadFromStaging"}));
    }

    #[test]
    fn update_body_sends_only_transform_id_when_no_optionals_set() {
        let body = UpdateTransformConfigurationBody {
            transform_id: "MyETL".to_string(),
            enabled: None,
            verbose_logging: None,
        };
        let json = serde_json::to_value(body).expect("should serialize");
        assert_eq!(json, serde_json::json!({"transformId": "MyETL"}));
    }

    #[test]
    fn update_body_includes_enabled_and_verbose_logging_when_set() {
        let body = UpdateTransformConfigurationBody {
            transform_id: "MyETL".to_string(),
            enabled: Some(true),
            verbose_logging: Some(false),
        };
        let json = serde_json::to_value(body).expect("should serialize");
        assert_eq!(
            json,
            serde_json::json!({
                "transformId": "MyETL",
                "enabled": true,
                "verboseLogging": false
            })
        );
    }

    #[test]
    fn update_body_includes_only_enabled_when_verbose_logging_unset() {
        let body = UpdateTransformConfigurationBody {
            transform_id: "MyETL".to_string(),
            enabled: Some(false),
            verbose_logging: None,
        };
        let json = serde_json::to_value(body).expect("should serialize");
        assert_eq!(
            json,
            serde_json::json!({"transformId": "MyETL", "enabled": false})
        );
    }

    // -- Response deserialization tests --

    #[test]
    fn run_transform_response_deserializes_all_java_fields() {
        let response: RunTransformResponse = serde_json::from_value(serde_json::json!({
            "success": true,
            "jobId": "123",
            "pipelineURL": "/labkey/pipeline-status/showList.view",
            "status": "success"
        }))
        .expect("should deserialize");
        assert_eq!(response.success, Some(true));
        assert_eq!(response.job_id.as_deref(), Some("123"));
        assert_eq!(
            response.pipeline_url.as_deref(),
            Some("/labkey/pipeline-status/showList.view")
        );
        assert_eq!(response.status.as_deref(), Some("success"));
    }

    #[test]
    fn run_transform_response_deserializes_minimal() {
        let response: RunTransformResponse =
            serde_json::from_value(serde_json::json!({})).expect("should deserialize");
        assert_eq!(response.success, None);
        assert_eq!(response.job_id, None);
        assert_eq!(response.pipeline_url, None);
        assert_eq!(response.status, None);
    }

    #[test]
    fn reset_transform_state_response_deserializes_success() {
        let response: ResetTransformStateResponse =
            serde_json::from_value(serde_json::json!({"success": true}))
                .expect("should deserialize");
        assert_eq!(response.success, Some(true));
    }

    #[test]
    fn reset_transform_state_response_deserializes_minimal() {
        let response: ResetTransformStateResponse =
            serde_json::from_value(serde_json::json!({})).expect("should deserialize");
        assert_eq!(response.success, None);
    }

    #[test]
    fn update_transform_configuration_response_deserializes_with_result_envelope() {
        let response: UpdateTransformConfigurationResponse =
            serde_json::from_value(serde_json::json!({
                "success": true,
                "result": {
                    "enabled": true,
                    "verboseLogging": false,
                    "state": {"rowCount": 42},
                    "lastChecked": "2024-01-15T10:30:00Z",
                    "descriptionId": "LoadFromStaging"
                }
            }))
            .expect("should deserialize");
        assert_eq!(response.success, Some(true));
        let result = response.result.expect("result should be present");
        assert_eq!(result.enabled, Some(true));
        assert_eq!(result.verbose_logging, Some(false));
        assert_eq!(result.state, Some(serde_json::json!({"rowCount": 42})));
        assert_eq!(result.last_checked.as_deref(), Some("2024-01-15T10:30:00Z"));
        assert_eq!(result.description_id.as_deref(), Some("LoadFromStaging"));
    }

    #[test]
    fn update_transform_configuration_response_deserializes_minimal() {
        let response: UpdateTransformConfigurationResponse =
            serde_json::from_value(serde_json::json!({})).expect("should deserialize");
        assert_eq!(response.success, None);
        assert!(response.result.is_none());
    }

    // -- Validation tests --

    #[test]
    fn validation_rejects_non_positive_id() {
        assert!(matches!(
            validate_transform_selector(&TransformSelector::Id(0), "run_transform"),
            Err(LabkeyError::InvalidInput(message)) if message.contains("transform_id > 0")
        ));
        assert!(matches!(
            validate_transform_selector(&TransformSelector::Id(-1), "run_transform"),
            Err(LabkeyError::InvalidInput(message)) if message.contains("transform_id > 0")
        ));
    }

    #[test]
    fn validation_rejects_blank_name() {
        assert!(matches!(
            validate_transform_selector(&TransformSelector::Name("  \t  ".to_string()), "run_transform"),
            Err(LabkeyError::InvalidInput(message)) if message.contains("non-blank")
        ));
    }

    #[test]
    fn validation_accepts_valid_selectors() {
        assert!(validate_transform_selector(&TransformSelector::Id(1), "run_transform").is_ok());
        assert!(
            validate_transform_selector(
                &TransformSelector::Name("TransformA".to_string()),
                "run_transform"
            )
            .is_ok()
        );
    }
}
