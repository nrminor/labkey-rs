//! Policy-focused security endpoints.

use std::collections::HashMap;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{client::LabkeyClient, error::LabkeyError, security::Policy};

/// Options for [`LabkeyClient::get_policy`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetPolicyOptions {
    /// Unique id of the securable resource.
    pub resource_id: String,
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

/// Response type for [`LabkeyClient::get_policy`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GetPolicyResponse {
    /// Requested or inherited policy in effect.
    pub policy: Policy,
    /// Roles relevant to this policy context.
    #[serde(default)]
    pub relevant_roles: Vec<String>,
}

/// Options for [`LabkeyClient::save_policy`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SavePolicyOptions {
    /// Policy object to persist.
    pub policy: serde_json::Value,
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

/// Response type for [`LabkeyClient::save_policy`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SavePolicyResponse {
    /// Server-provided success flag.
    #[serde(default)]
    pub success: Option<bool>,
    /// Additional endpoint-specific fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Options for [`LabkeyClient::delete_policy`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct DeletePolicyOptions {
    /// Unique id of the securable resource.
    pub resource_id: String,
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

/// Response type for [`LabkeyClient::delete_policy`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DeletePolicyResponse {
    /// Server-provided success flag.
    #[serde(default)]
    pub success: Option<bool>,
    /// Additional endpoint-specific fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ResourceIdBody {
    resource_id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetPolicyEnvelope {
    policy: Policy,
    #[serde(default)]
    relevant_roles: Vec<String>,
}

fn validate_non_empty(field_name: &str, value: &str) -> Result<(), LabkeyError> {
    if value.trim().is_empty() {
        return Err(LabkeyError::InvalidInput(format!(
            "{field_name} cannot be empty"
        )));
    }

    Ok(())
}

fn extract_policy(response: &serde_json::Value) -> Result<GetPolicyEnvelope, LabkeyError> {
    serde_json::from_value::<GetPolicyEnvelope>(response.clone()).map_err(|_| {
        LabkeyError::UnexpectedResponse {
            status: StatusCode::OK,
            text: format!("invalid getPolicy response: {response}"),
        }
    })
}

fn normalize_save_policy_body(policy: serde_json::Value) -> serde_json::Value {
    match policy {
        serde_json::Value::Object(mut object)
            if object.len() == 1 && object.get("policy").is_some() =>
        {
            object
                .remove("policy")
                .unwrap_or(serde_json::Value::Object(object))
        }
        serde_json::Value::Object(object) => serde_json::Value::Object(object),
        other => other,
    }
}

impl LabkeyClient {
    /// Retrieve the security policy for a resource id.
    ///
    /// Sends a POST request to `security-getPolicy.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError::InvalidInput`] when `resource_id` is empty.
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, or the response is missing policy envelope fields.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), labkey_rs::LabkeyError> {
    /// # let config = labkey_rs::ClientConfig::new(
    /// #     "https://labkey.example.com/labkey",
    /// #     labkey_rs::Credential::ApiKey("key".into()),
    /// #     "/MyProject",
    /// # );
    /// # let client = labkey_rs::LabkeyClient::new(config)?;
    /// use labkey_rs::security::GetPolicyOptions;
    ///
    /// let response = client
    ///     .get_policy(
    ///         GetPolicyOptions::builder()
    ///             .resource_id("resource-1".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Requested resource: {:?}", response.policy.requested_resource_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_policy(
        &self,
        options: GetPolicyOptions,
    ) -> Result<GetPolicyResponse, LabkeyError> {
        validate_non_empty("get_policy resource_id", &options.resource_id)?;

        let request_resource_id = options.resource_id;
        let url = self.build_url(
            "security",
            "getPolicy.api",
            options.container_path.as_deref(),
        );
        let body = ResourceIdBody {
            resource_id: request_resource_id.clone(),
        };

        let response: serde_json::Value = self.post(url, &body).await?;
        let envelope = extract_policy(&response)?;

        let mut policy = envelope.policy;
        policy.requested_resource_id = Some(request_resource_id);

        Ok(GetPolicyResponse {
            policy,
            relevant_roles: envelope.relevant_roles,
        })
    }

    /// Save a security policy.
    ///
    /// Sends a POST request to `security-savePolicy.api`.
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
    /// #     "/MyProject",
    /// # );
    /// # let client = labkey_rs::LabkeyClient::new(config)?;
    /// use labkey_rs::security::SavePolicyOptions;
    ///
    /// let _ = client
    ///     .save_policy(
    ///         SavePolicyOptions::builder()
    ///             .policy(serde_json::json!({
    ///                 "resourceId": "resource-1",
    ///                 "assignments": []
    ///             }))
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_policy(
        &self,
        options: SavePolicyOptions,
    ) -> Result<SavePolicyResponse, LabkeyError> {
        let url = self.build_url(
            "security",
            "savePolicy.api",
            options.container_path.as_deref(),
        );
        let body = normalize_save_policy_body(options.policy);
        self.post(url, &body).await
    }

    /// Delete the explicit security policy for a resource.
    ///
    /// Sends a POST request to `security-deletePolicy.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError::InvalidInput`] when `resource_id` is empty.
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
    /// #     "/MyProject",
    /// # );
    /// # let client = labkey_rs::LabkeyClient::new(config)?;
    /// use labkey_rs::security::DeletePolicyOptions;
    ///
    /// let _ = client
    ///     .delete_policy(
    ///         DeletePolicyOptions::builder()
    ///             .resource_id("resource-1".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_policy(
        &self,
        options: DeletePolicyOptions,
    ) -> Result<DeletePolicyResponse, LabkeyError> {
        validate_non_empty("delete_policy resource_id", &options.resource_id)?;

        let url = self.build_url(
            "security",
            "deletePolicy.api",
            options.container_path.as_deref(),
        );
        let body = ResourceIdBody {
            resource_id: options.resource_id,
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
    fn security_policy_endpoint_urls_match_expected_actions() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url("security", "getPolicy.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-getPolicy.api"
        );
        assert_eq!(
            client
                .build_url("security", "savePolicy.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-savePolicy.api"
        );
        assert_eq!(
            client
                .build_url("security", "deletePolicy.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-deletePolicy.api"
        );
    }

    #[test]
    fn resource_id_body_serializes_camel_case() {
        let body = ResourceIdBody {
            resource_id: "resource-1".to_string(),
        };

        let value = serde_json::to_value(body).expect("body should serialize");
        assert_eq!(
            value.get("resourceId"),
            Some(&serde_json::json!("resource-1"))
        );
    }

    #[test]
    fn get_policy_envelope_deserializes() {
        let value = serde_json::json!({
            "policy": {
                "resourceId": "resource-from-server",
                "assignments": []
            },
            "relevantRoles": ["org.labkey.security.roles.EditorRole"]
        });

        let envelope = extract_policy(&value).expect("policy envelope should parse");
        assert_eq!(
            envelope.policy.resource_id.as_deref(),
            Some("resource-from-server")
        );
        assert_eq!(envelope.relevant_roles.len(), 1);
    }

    #[test]
    fn extract_policy_rejects_missing_policy_field() {
        let value = serde_json::json!({"relevantRoles": []});

        let error = extract_policy(&value).expect_err("missing policy field should fail");
        match error {
            LabkeyError::UnexpectedResponse { status, text } => {
                assert_eq!(status, StatusCode::OK);
                assert!(text.contains("getPolicy"));
            }
            other => panic!("expected UnexpectedResponse, got {other:?}"),
        }
    }

    #[test]
    fn normalize_save_policy_body_unwraps_policy_key() {
        let wrapped = serde_json::json!({
            "policy": {
                "resourceId": "resource-1",
                "assignments": []
            }
        });

        let normalized = normalize_save_policy_body(wrapped);
        assert_eq!(
            normalized,
            serde_json::json!({
                "resourceId": "resource-1",
                "assignments": []
            })
        );
    }

    #[test]
    fn normalize_save_policy_body_preserves_plain_policy_object() {
        let plain = serde_json::json!({
            "resourceId": "resource-1",
            "assignments": []
        });

        let normalized = normalize_save_policy_body(plain.clone());
        assert_eq!(normalized, plain);
    }

    #[test]
    fn normalize_save_policy_body_preserves_objects_with_additional_fields() {
        let wrapped = serde_json::json!({
            "policy": {
                "resourceId": "resource-1",
                "assignments": []
            },
            "auditComment": "keep me"
        });

        let normalized = normalize_save_policy_body(wrapped.clone());
        assert_eq!(normalized, wrapped);
    }

    #[test]
    fn policy_validation_rejects_empty_resource_id() {
        let error = validate_non_empty("get_policy resource_id", "   ")
            .expect_err("empty resource id should fail");
        assert!(matches!(
            error,
            LabkeyError::InvalidInput(message)
                if message == "get_policy resource_id cannot be empty"
        ));
    }
}
