//! Data Integration (DI) transform endpoints.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{client::LabkeyClient, error::LabkeyError};

/// Transform configuration payload used by update/read APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TransformConfig {
    /// Optional transform description.
    #[serde(default)]
    pub description: Option<String>,
    /// Optional enabled flag.
    #[serde(default)]
    pub enabled: Option<bool>,
    /// Arbitrary transform properties.
    #[serde(default)]
    pub properties: Option<serde_json::Value>,
    /// Additional server-provided fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl TransformConfig {
    /// Create an empty transform configuration.
    #[must_use]
    pub fn new() -> Self {
        Self {
            description: None,
            enabled: None,
            properties: None,
            extra: HashMap::new(),
        }
    }
}

impl Default for TransformConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Selector for identifying a DI transform.
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
    /// Transform selector.
    pub selector: TransformSelector,
    /// Optional transform configuration override.
    pub transform_config: Option<TransformConfig>,
}

/// Response payload from [`LabkeyClient::run_transform`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RunTransformResponse {
    /// Optional job id for asynchronous execution.
    #[serde(default)]
    pub job_id: Option<i64>,
    /// Optional server message.
    #[serde(default)]
    pub message: Option<String>,
    /// Success flag when provided by the server.
    #[serde(default)]
    pub success: Option<bool>,
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
    /// Transform selector.
    pub selector: TransformSelector,
}

/// Response payload from [`LabkeyClient::reset_transform_state`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResetTransformStateResponse {
    /// Optional server message.
    #[serde(default)]
    pub message: Option<String>,
    /// Success flag when provided by the server.
    #[serde(default)]
    pub success: Option<bool>,
    /// Additional server-provided fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Options for [`LabkeyClient::update_transform_configuration`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct UpdateTransformConfigurationOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Transform selector.
    pub selector: TransformSelector,
    /// Updated transform configuration.
    pub transform_config: TransformConfig,
}

/// Response payload from [`LabkeyClient::update_transform_configuration`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UpdateTransformConfigurationResponse {
    /// Optional updated configuration.
    #[serde(default)]
    pub transform_config: Option<TransformConfig>,
    /// Optional server message.
    #[serde(default)]
    pub message: Option<String>,
    /// Success flag when provided by the server.
    #[serde(default)]
    pub success: Option<bool>,
    /// Additional server-provided fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RunTransformBody {
    #[serde(rename = "transformId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    #[serde(rename = "transformName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(rename = "transformConfig")]
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<TransformConfig>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResetTransformStateBody {
    #[serde(rename = "transformId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    #[serde(rename = "transformName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateTransformConfigurationBody {
    #[serde(rename = "transformId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    id: Option<i64>,
    #[serde(rename = "transformName")]
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(rename = "transformConfig")]
    #[serde(skip_serializing_if = "Option::is_none")]
    config: Option<TransformConfig>,
}

fn validate_transform_identity(
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

fn selector_parts(selector: TransformSelector) -> (Option<i64>, Option<String>) {
    match selector {
        TransformSelector::Id(value) => (Some(value), None),
        TransformSelector::Name(value) => (None, Some(value)),
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
        validate_transform_identity(&options.selector, "run_transform")?;
        let (transform_id, transform_name) = selector_parts(options.selector);

        let url = self.build_url(
            "dataintegration",
            "runTransform",
            options.container_path.as_deref(),
        );
        let body = RunTransformBody {
            id: transform_id,
            name: transform_name,
            config: options.transform_config,
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
        validate_transform_identity(&options.selector, "reset_transform_state")?;
        let (transform_id, transform_name) = selector_parts(options.selector);

        let url = self.build_url(
            "dataintegration",
            "resetTransformState",
            options.container_path.as_deref(),
        );
        let body = ResetTransformStateBody {
            id: transform_id,
            name: transform_name,
        };

        self.post(url, &body).await
    }

    /// Update DI transform configuration through `dataintegration-UpdateTransformConfiguration`.
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
    /// use labkey_rs::di::{TransformConfig, TransformSelector, UpdateTransformConfigurationOptions};
    ///
    /// let mut config = TransformConfig::new();
    /// config.description = Some("updated".to_string());
    /// config.enabled = Some(true);
    ///
    /// let _ = client
    ///     .update_transform_configuration(
    ///         UpdateTransformConfigurationOptions::builder()
    ///             .selector(TransformSelector::Name("LoadFromStaging".to_string()))
    ///             .transform_config(config)
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_transform_configuration(
        &self,
        options: UpdateTransformConfigurationOptions,
    ) -> Result<UpdateTransformConfigurationResponse, LabkeyError> {
        validate_transform_identity(&options.selector, "update_transform_configuration")?;
        let (transform_id, transform_name) = selector_parts(options.selector);

        let url = self.build_url(
            "dataintegration",
            "UpdateTransformConfiguration",
            options.container_path.as_deref(),
        );
        let body = UpdateTransformConfigurationBody {
            id: transform_id,
            name: transform_name,
            config: Some(options.transform_config),
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

    #[test]
    fn run_transform_response_deserializes_happy_and_minimal_shapes() {
        let happy: RunTransformResponse = serde_json::from_value(serde_json::json!({
            "success": true,
            "message": "queued",
            "jobId": 123
        }))
        .expect("happy response should deserialize");
        assert_eq!(happy.success, Some(true));
        assert_eq!(happy.message.as_deref(), Some("queued"));
        assert_eq!(happy.job_id, Some(123));

        let minimal: RunTransformResponse = serde_json::from_value(serde_json::json!({}))
            .expect("minimal response should deserialize");
        assert_eq!(minimal.success, None);
        assert_eq!(minimal.message, None);
        assert_eq!(minimal.job_id, None);
    }

    #[test]
    fn di_body_serialization_uses_expected_wire_keys_and_omits_absent_fields() {
        let run_body = RunTransformBody {
            id: Some(7),
            name: None,
            config: Some(TransformConfig {
                description: Some("config".to_string()),
                enabled: Some(true),
                properties: None,
                extra: HashMap::new(),
            }),
        };
        let run_json = serde_json::to_value(run_body).expect("run body should serialize");
        assert_eq!(run_json["transformId"], serde_json::json!(7));
        assert!(run_json.get("transformName").is_none());
        assert!(run_json.get("transformConfig").is_some());

        let reset_body = ResetTransformStateBody {
            id: None,
            name: Some("TransformA".to_string()),
        };
        let reset_json = serde_json::to_value(reset_body).expect("reset body should serialize");
        assert_eq!(reset_json["transformName"], serde_json::json!("TransformA"));
        assert!(reset_json.get("transformId").is_none());

        let update_body = UpdateTransformConfigurationBody {
            id: Some(11),
            name: None,
            config: Some(TransformConfig {
                description: None,
                enabled: Some(false),
                properties: Some(serde_json::json!({"mode": "full"})),
                extra: HashMap::new(),
            }),
        };
        let update_json = serde_json::to_value(update_body).expect("update body should serialize");
        assert_eq!(update_json["transformId"], serde_json::json!(11));
        assert!(update_json.get("transformName").is_none());
        assert_eq!(
            update_json["transformConfig"]["enabled"],
            serde_json::json!(false)
        );
    }

    #[test]
    fn reset_transform_state_response_deserializes_happy_and_minimal_shapes() {
        let happy: ResetTransformStateResponse = serde_json::from_value(serde_json::json!({
            "success": true,
            "message": "reset"
        }))
        .expect("happy response should deserialize");
        assert_eq!(happy.success, Some(true));
        assert_eq!(happy.message.as_deref(), Some("reset"));

        let minimal: ResetTransformStateResponse = serde_json::from_value(serde_json::json!({}))
            .expect("minimal response should deserialize");
        assert_eq!(minimal.success, None);
        assert_eq!(minimal.message, None);
    }

    #[test]
    fn update_transform_configuration_response_deserializes_happy_and_minimal_shapes() {
        let happy: UpdateTransformConfigurationResponse =
            serde_json::from_value(serde_json::json!({
                "success": true,
                "message": "updated",
                "transformConfig": {
                    "description": "new config",
                    "enabled": true,
                    "properties": { "batchSize": 100 }
                }
            }))
            .expect("happy response should deserialize");
        assert_eq!(happy.success, Some(true));
        assert_eq!(happy.message.as_deref(), Some("updated"));
        assert_eq!(
            happy
                .transform_config
                .as_ref()
                .and_then(|value| value.enabled),
            Some(true)
        );

        let minimal: UpdateTransformConfigurationResponse =
            serde_json::from_value(serde_json::json!({}))
                .expect("minimal response should deserialize");
        assert_eq!(minimal.success, None);
        assert_eq!(minimal.message, None);
        assert!(minimal.transform_config.is_none());
    }

    #[test]
    fn di_validation_rejects_invalid_selector_values() {
        assert!(matches!(
            validate_transform_identity(&TransformSelector::Id(0), "run_transform"),
            Err(LabkeyError::InvalidInput(message)) if message.contains("transform_id > 0")
        ));
        assert!(matches!(
            validate_transform_identity(&TransformSelector::Name("  \t  ".to_string()), "run_transform"),
            Err(LabkeyError::InvalidInput(message)) if message.contains("non-blank")
        ));
        assert!(validate_transform_identity(&TransformSelector::Id(1), "run_transform").is_ok());
        assert!(
            validate_transform_identity(
                &TransformSelector::Name("TransformA".to_string()),
                "run_transform"
            )
            .is_ok()
        );
    }
}
