//! Participant group session management for LabKey studies.
//!
//! Participant groups define subsets of study participants for analysis. This
//! module provides [`UpdateParticipantGroupOptions`] for updating the session-
//! level participant group selection, which controls which participants are
//! visible in study views and reports.

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{client::LabkeyClient, error::LabkeyError};

/// Options for [`LabkeyClient::update_participant_group`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct UpdateParticipantGroupOptions {
    /// Identifier of the participant group to update.
    pub row_id: i64,
    /// Set of participant ids to be members of the group.
    pub participant_ids: Option<Vec<String>>,
    /// Participant ids to ensure are members.
    pub ensure_participant_ids: Option<Vec<String>>,
    /// Participant ids to delete from the group.
    pub delete_participant_ids: Option<Vec<String>>,
    /// Updated group label.
    pub label: Option<String>,
    /// Updated group description.
    pub description: Option<String>,
    /// Optional filters payload forwarded to the server.
    pub filters: Option<serde_json::Value>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateParticipantGroupBody {
    row_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    participant_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ensure_participant_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delete_participant_ids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateParticipantGroupEnvelope {
    #[serde(default)]
    group: Option<serde_json::Value>,
}

impl LabkeyClient {
    /// Update an existing participant group.
    ///
    /// Sends a POST request to `participant-group-updateParticipantGroup.api`
    /// and returns the unwrapped `response.group` payload.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, the response envelope is missing `group`, or the body
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
    /// use labkey_rs::participant_group::UpdateParticipantGroupOptions;
    ///
    /// let group = client
    ///     .update_participant_group(
    ///         UpdateParticipantGroupOptions::builder()
    ///             .row_id(101)
    ///             .label("Responders".to_string())
    ///             .participant_ids(vec!["PT-1".to_string(), "PT-2".to_string()])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", group["rowId"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_participant_group(
        &self,
        options: UpdateParticipantGroupOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "participant-group",
            "updateParticipantGroup.api",
            options.container_path.as_deref(),
        );
        let body = UpdateParticipantGroupBody {
            row_id: options.row_id,
            participant_ids: options.participant_ids,
            ensure_participant_ids: options.ensure_participant_ids,
            delete_participant_ids: options.delete_participant_ids,
            label: options.label,
            description: options.description,
            filters: options.filters,
        };
        let response: UpdateParticipantGroupEnvelope = self.post(url, &body).await?;
        extract_group(response)
    }
}

fn extract_group(
    response: UpdateParticipantGroupEnvelope,
) -> Result<serde_json::Value, LabkeyError> {
    match response.group {
        Some(group) => Ok(group),
        None => Err(LabkeyError::UnexpectedResponse {
            status: StatusCode::OK,
            text: "update_participant_group response missing group envelope".to_string(),
        }),
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
    fn participant_group_endpoint_url_matches_expected_action() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url(
                    "participant-group",
                    "updateParticipantGroup.api",
                    Some("/Alt/Study"),
                )
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Study/participant-group-updateParticipantGroup.api"
        );
    }

    #[test]
    fn participant_group_body_omits_unset_optionals() {
        let body = UpdateParticipantGroupBody {
            row_id: 101,
            participant_ids: None,
            ensure_participant_ids: None,
            delete_participant_ids: None,
            label: Some("A".to_string()),
            description: None,
            filters: None,
        };

        let json = serde_json::to_value(body).expect("body should serialize");
        assert_eq!(json["rowId"], serde_json::json!(101));
        assert_eq!(json["label"], serde_json::json!("A"));
        assert!(json.get("participantIds").is_none());
        assert!(json.get("ensureParticipantIds").is_none());
        assert!(json.get("deleteParticipantIds").is_none());
        assert!(json.get("description").is_none());
        assert!(json.get("filters").is_none());
    }

    #[test]
    fn extract_group_returns_group_and_errors_when_missing() {
        let group = extract_group(UpdateParticipantGroupEnvelope {
            group: Some(serde_json::json!({ "rowId": 5 })),
        })
        .expect("group should extract");
        assert_eq!(group["rowId"], serde_json::json!(5));

        let error = extract_group(UpdateParticipantGroupEnvelope { group: None })
            .expect_err("missing group should fail");
        assert!(matches!(error, LabkeyError::UnexpectedResponse { .. }));
    }
}
