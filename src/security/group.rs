//! Group-focused security endpoints.

use serde::{Deserialize, Serialize};

use crate::{client::LabkeyClient, error::LabkeyError};

/// Options for [`LabkeyClient::add_group_members`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct AddGroupMembersOptions {
    /// Group id to modify.
    pub group_id: i64,
    /// Principal ids to add to the group.
    pub principal_ids: Vec<i64>,
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

/// Response type for [`LabkeyClient::add_group_members`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AddGroupMembersResponse {
    /// Principal ids successfully added.
    #[serde(default)]
    pub added: Vec<i64>,
}

/// Options for [`LabkeyClient::create_group`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct CreateGroupOptions {
    /// Group name to create.
    pub group_name: String,
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

/// Response type for [`LabkeyClient::create_group`].
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct CreateGroupResponse {
    /// Created group id.
    pub id: i64,
    /// Created group name.
    pub name: String,
}

/// Options for [`LabkeyClient::delete_group`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct DeleteGroupOptions {
    /// Group id to delete.
    pub group_id: i64,
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

/// Response type for [`LabkeyClient::delete_group`].
#[derive(Debug, Clone, Deserialize)]
#[non_exhaustive]
pub struct DeleteGroupResponse {
    /// Number of groups deleted.
    pub deleted: i64,
}

/// Options for [`LabkeyClient::rename_group`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct RenameGroupOptions {
    /// Group id to rename.
    pub group_id: i64,
    /// New group name.
    pub new_name: String,
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

/// Response type for [`LabkeyClient::rename_group`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RenameGroupResponse {
    /// New group name.
    pub new_name: String,
    /// Previous group name.
    pub old_name: String,
    /// Group id that was renamed.
    pub renamed: i64,
    /// Whether rename succeeded.
    pub success: bool,
}

/// Options for [`LabkeyClient::remove_group_members`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct RemoveGroupMembersOptions {
    /// Group id to modify.
    pub group_id: i64,
    /// Principal ids to remove from the group.
    pub principal_ids: Vec<i64>,
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

/// Response type for [`LabkeyClient::remove_group_members`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RemoveGroupMembersResponse {
    /// Principal ids successfully removed.
    #[serde(default)]
    pub removed: Vec<i64>,
}

/// Options for [`LabkeyClient::get_groups_for_current_user`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetGroupsForCurrentUserOptions {
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

/// Group summary entry in [`GetGroupsForCurrentUserResponse`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GroupForCurrentUser {
    /// Group id.
    pub id: i64,
    /// Group name.
    pub name: String,
    /// Whether this is a project-scoped group.
    #[serde(default)]
    pub is_project_group: Option<bool>,
    /// Whether this is a system-scoped group.
    #[serde(default)]
    pub is_system_group: Option<bool>,
}

/// Response type for [`LabkeyClient::get_groups_for_current_user`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GetGroupsForCurrentUserResponse {
    /// Groups for the current user.
    #[serde(default)]
    pub groups: Vec<GroupForCurrentUser>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AddOrRemoveGroupMembersBody {
    group_id: i64,
    principal_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
struct CreateGroupBody {
    name: String,
}

#[derive(Debug, Serialize)]
struct DeleteGroupBody {
    id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RenameGroupBody {
    id: i64,
    new_name: String,
}

fn validate_non_empty(field_name: &str, value: &str) -> Result<(), LabkeyError> {
    if value.trim().is_empty() {
        return Err(LabkeyError::InvalidInput(format!(
            "{field_name} cannot be empty"
        )));
    }

    Ok(())
}

fn validate_principal_ids(field_name: &str, principal_ids: &[i64]) -> Result<(), LabkeyError> {
    if principal_ids.is_empty() {
        return Err(LabkeyError::InvalidInput(format!(
            "{field_name} requires at least one principal id"
        )));
    }

    Ok(())
}

impl LabkeyClient {
    /// Add principals to an existing group.
    ///
    /// Sends a POST request to `security-addGroupMember.api`.
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
    /// use labkey_rs::security::AddGroupMembersOptions;
    ///
    /// let response = client
    ///     .add_group_members(
    ///         AddGroupMembersOptions::builder()
    ///             .group_id(101)
    ///             .principal_ids(vec![202, 303])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Added principals: {}", response.added.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_group_members(
        &self,
        options: AddGroupMembersOptions,
    ) -> Result<AddGroupMembersResponse, LabkeyError> {
        validate_principal_ids("add_group_members", &options.principal_ids)?;

        let url = self.build_url(
            "security",
            "addGroupMember.api",
            options.container_path.as_deref(),
        );
        let body = AddOrRemoveGroupMembersBody {
            group_id: options.group_id,
            principal_ids: options.principal_ids,
        };
        self.post(url, &body).await
    }

    /// Create a new group.
    ///
    /// Sends a POST request to `security-createGroup.api`.
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
    /// use labkey_rs::security::CreateGroupOptions;
    ///
    /// let group = client
    ///     .create_group(
    ///         CreateGroupOptions::builder()
    ///             .group_name("Research Analysts".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Created group id: {}", group.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_group(
        &self,
        options: CreateGroupOptions,
    ) -> Result<CreateGroupResponse, LabkeyError> {
        validate_non_empty("create_group group_name", &options.group_name)?;

        let url = self.build_url(
            "security",
            "createGroup.api",
            options.container_path.as_deref(),
        );
        let body = CreateGroupBody {
            name: options.group_name,
        };
        self.post(url, &body).await
    }

    /// Delete an existing group.
    ///
    /// Sends a POST request to `security-deleteGroup.api`.
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
    /// use labkey_rs::security::DeleteGroupOptions;
    ///
    /// let response = client
    ///     .delete_group(DeleteGroupOptions::builder().group_id(101).build())
    ///     .await?;
    ///
    /// println!("Deleted groups: {}", response.deleted);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_group(
        &self,
        options: DeleteGroupOptions,
    ) -> Result<DeleteGroupResponse, LabkeyError> {
        let url = self.build_url(
            "security",
            "deleteGroup.api",
            options.container_path.as_deref(),
        );
        let body = DeleteGroupBody {
            id: options.group_id,
        };
        self.post(url, &body).await
    }

    /// Rename an existing group.
    ///
    /// Sends a POST request to `security-renameGroup.api`.
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
    /// use labkey_rs::security::RenameGroupOptions;
    ///
    /// let response = client
    ///     .rename_group(
    ///         RenameGroupOptions::builder()
    ///             .group_id(101)
    ///             .new_name("Senior Analysts".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Rename success: {}", response.success);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rename_group(
        &self,
        options: RenameGroupOptions,
    ) -> Result<RenameGroupResponse, LabkeyError> {
        validate_non_empty("rename_group new_name", &options.new_name)?;

        let url = self.build_url(
            "security",
            "renameGroup.api",
            options.container_path.as_deref(),
        );
        let body = RenameGroupBody {
            id: options.group_id,
            new_name: options.new_name,
        };
        self.post(url, &body).await
    }

    /// Remove principals from an existing group.
    ///
    /// Sends a POST request to `security-removeGroupMember.api`.
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
    /// use labkey_rs::security::RemoveGroupMembersOptions;
    ///
    /// let response = client
    ///     .remove_group_members(
    ///         RemoveGroupMembersOptions::builder()
    ///             .group_id(101)
    ///             .principal_ids(vec![202])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Removed principals: {}", response.removed.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_group_members(
        &self,
        options: RemoveGroupMembersOptions,
    ) -> Result<RemoveGroupMembersResponse, LabkeyError> {
        validate_principal_ids("remove_group_members", &options.principal_ids)?;

        let url = self.build_url(
            "security",
            "removeGroupMember.api",
            options.container_path.as_deref(),
        );
        let body = AddOrRemoveGroupMembersBody {
            group_id: options.group_id,
            principal_ids: options.principal_ids,
        };
        self.post(url, &body).await
    }

    /// Retrieve groups for the current authenticated user.
    ///
    /// Sends a GET request to `security-getGroupsForCurrentUser.api`.
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
    /// use labkey_rs::security::GetGroupsForCurrentUserOptions;
    ///
    /// let response = client
    ///     .get_groups_for_current_user(GetGroupsForCurrentUserOptions::builder().build())
    ///     .await?;
    ///
    /// println!("Current-user groups: {}", response.groups.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_groups_for_current_user(
        &self,
        options: GetGroupsForCurrentUserOptions,
    ) -> Result<GetGroupsForCurrentUserResponse, LabkeyError> {
        let url = self.build_url(
            "security",
            "getGroupsForCurrentUser.api",
            options.container_path.as_deref(),
        );
        self.get(url, &[]).await
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
    fn security_group_endpoint_urls_match_expected_actions() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url("security", "addGroupMember.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-addGroupMember.api"
        );
        assert_eq!(
            client
                .build_url("security", "createGroup.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-createGroup.api"
        );
        assert_eq!(
            client
                .build_url("security", "deleteGroup.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-deleteGroup.api"
        );
        assert_eq!(
            client
                .build_url(
                    "security",
                    "getGroupsForCurrentUser.api",
                    Some("/Alt/Container")
                )
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-getGroupsForCurrentUser.api"
        );
        assert_eq!(
            client
                .build_url("security", "removeGroupMember.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-removeGroupMember.api"
        );
        assert_eq!(
            client
                .build_url("security", "renameGroup.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-renameGroup.api"
        );
    }

    #[test]
    fn add_group_members_body_uses_group_id_and_principal_ids() {
        let body = AddOrRemoveGroupMembersBody {
            group_id: 101,
            principal_ids: vec![202, 303],
        };

        let value = serde_json::to_value(body).expect("should serialize");
        assert_eq!(value.get("groupId"), Some(&serde_json::json!(101)));
        assert_eq!(
            value.get("principalIds"),
            Some(&serde_json::json!([202, 303]))
        );
    }

    #[test]
    fn create_group_rejects_empty_group_name() {
        let error = validate_non_empty("create_group group_name", " ")
            .expect_err("empty group name should fail");
        assert!(matches!(
            error,
            LabkeyError::InvalidInput(message)
                if message == "create_group group_name cannot be empty"
        ));
    }

    #[test]
    fn rename_group_rejects_empty_new_name() {
        let error = validate_non_empty("rename_group new_name", "\t")
            .expect_err("empty new name should fail");
        assert!(matches!(
            error,
            LabkeyError::InvalidInput(message) if message == "rename_group new_name cannot be empty"
        ));
    }

    #[test]
    fn add_group_members_rejects_empty_principal_ids() {
        let error = validate_principal_ids("add_group_members", &[])
            .expect_err("empty principal ids should fail");
        assert!(matches!(
            error,
            LabkeyError::InvalidInput(message)
                if message == "add_group_members requires at least one principal id"
        ));
    }

    #[test]
    fn remove_group_members_rejects_empty_principal_ids() {
        let error = validate_principal_ids("remove_group_members", &[])
            .expect_err("empty principal ids should fail");
        assert!(matches!(
            error,
            LabkeyError::InvalidInput(message)
                if message == "remove_group_members requires at least one principal id"
        ));
    }

    #[test]
    fn create_group_body_serializes_name_key_not_group_name() {
        let body = CreateGroupBody {
            name: "Research Analysts".to_string(),
        };

        let value = serde_json::to_value(body).expect("should serialize");
        assert_eq!(
            value,
            serde_json::json!({
                "name": "Research Analysts"
            })
        );
    }

    #[test]
    fn rename_group_body_serializes_expected_wire_keys() {
        let body = RenameGroupBody {
            id: 44,
            new_name: "Senior Analysts".to_string(),
        };

        let value = serde_json::to_value(body).expect("should serialize");
        assert_eq!(value.get("id"), Some(&serde_json::json!(44)));
        assert_eq!(
            value.get("newName"),
            Some(&serde_json::json!("Senior Analysts"))
        );
    }

    #[test]
    fn delete_group_response_deserializes() {
        let value = serde_json::json!({"deleted": 1});
        let response: DeleteGroupResponse =
            serde_json::from_value(value).expect("delete group response should parse");
        assert_eq!(response.deleted, 1);
    }

    #[test]
    fn rename_group_response_deserializes() {
        let value = serde_json::json!({
            "newName": "Senior Analysts",
            "oldName": "Research Analysts",
            "renamed": 44,
            "success": true
        });
        let response: RenameGroupResponse =
            serde_json::from_value(value).expect("rename group response should parse");
        assert!(response.success);
        assert_eq!(response.renamed, 44);
        assert_eq!(response.new_name, "Senior Analysts");
    }

    #[test]
    fn get_groups_for_current_user_response_deserializes_minimal_group_entries() {
        let value = serde_json::json!({
            "groups": [
                {
                    "id": 10,
                    "name": "Administrators",
                    "isProjectGroup": true,
                    "isSystemGroup": false
                }
            ]
        });
        let response: GetGroupsForCurrentUserResponse =
            serde_json::from_value(value).expect("groups response should parse");
        assert_eq!(response.groups.len(), 1);
        assert_eq!(response.groups[0].id, 10);
        assert_eq!(response.groups[0].name, "Administrators");
        assert_eq!(response.groups[0].is_project_group, Some(true));
    }
}
