//! Permission-focused security endpoints.

use std::collections::HashMap;

use reqwest::StatusCode;
use serde::Deserialize;

use crate::{
    client::LabkeyClient,
    common::opt,
    error::LabkeyError,
    security::{Group, Role, RolePermission, SecurableResource},
};

/// Options for [`LabkeyClient::get_group_permissions`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetGroupPermissionsOptions {
    /// Optional container override for this request.
    pub container_path: Option<String>,
    /// Include groups with no effective permissions (server default is `true`).
    pub include_empty_perm_groups: Option<bool>,
    /// Include descendant containers.
    pub include_subfolders: Option<bool>,
}

/// Recursive container permission summary.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PermissionsContainer {
    /// Child containers when recursion is enabled.
    #[serde(default)]
    pub children: Vec<PermissionsContainer>,
    /// Group assignments in this container.
    #[serde(default)]
    pub groups: Vec<Group>,
    /// Container id.
    pub id: String,
    /// Whether this container inherits permissions from a parent.
    #[serde(default)]
    pub is_inheriting_perms: Option<bool>,
    /// Container display name.
    pub name: String,
    /// Container path.
    pub path: String,
}

/// Response type for [`LabkeyClient::get_group_permissions`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GroupPermissionsResponse {
    /// Container permissions summary.
    pub container: PermissionsContainer,
}

/// Options for [`LabkeyClient::get_user_permissions`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetUserPermissionsOptions {
    /// Optional container override for this request.
    pub container_path: Option<String>,
    /// Include descendant containers.
    pub include_subfolders: Option<bool>,
    /// User email to query. Ignored when `user_id` is provided.
    pub user_email: Option<String>,
    /// User id to query.
    pub user_id: Option<i64>,
}

/// User summary returned from [`GetUserPermissionsResponse`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PermissionUser {
    /// User display name.
    #[serde(default)]
    pub display_name: Option<String>,
    /// User id.
    pub user_id: i64,
}

/// User permissions container details.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct UserPermissionsContainer {
    /// Child containers when recursion is enabled.
    #[serde(default)]
    pub children: Vec<UserPermissionsContainer>,
    /// Group assignments in this container.
    #[serde(default)]
    pub groups: Vec<Group>,
    /// Container id.
    pub id: String,
    /// Whether this container inherits permissions from a parent.
    #[serde(default)]
    pub is_inheriting_perms: Option<bool>,
    /// Container display name.
    pub name: String,
    /// Container path.
    pub path: String,
    /// Effective permission unique names.
    #[serde(default)]
    pub effective_permissions: Vec<String>,
    /// Deprecated integer permission bitset.
    #[serde(default)]
    pub permissions: Option<i64>,
    /// Deprecated role token.
    #[serde(default)]
    pub role: Option<String>,
    /// Deprecated user-visible role label.
    #[serde(default)]
    pub role_label: Option<String>,
    /// Role unique names assigned in this container.
    #[serde(default)]
    pub roles: Vec<String>,
}

/// Response type for [`LabkeyClient::get_user_permissions`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GetUserPermissionsResponse {
    /// Container permissions information.
    pub container: UserPermissionsContainer,
    /// Queried user information.
    pub user: PermissionUser,
}

/// Options for [`LabkeyClient::get_roles`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetRolesOptions {
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::get_securable_resources`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetSecurableResourcesOptions {
    /// Optional container override for this request.
    pub container_path: Option<String>,
    /// Include effective permission unique names for each resource.
    pub include_effective_permissions: Option<bool>,
    /// Include descendant containers and resources.
    pub include_subfolders: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawRolePermission {
    name: String,
    unique_name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    source_module: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawRole {
    #[serde(default)]
    unique_name: Option<String>,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    excluded_principals: Vec<i64>,
    #[serde(default)]
    permissions: Vec<String>,
    #[serde(default)]
    source_module: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetRolesRawResponse {
    #[serde(default)]
    permissions: Vec<RawRolePermission>,
    #[serde(default)]
    roles: Vec<RawRole>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetSecurableResourcesEnvelope {
    resources: SecurableResource,
}

fn build_group_permissions_params(options: &GetGroupPermissionsOptions) -> Vec<(String, String)> {
    [
        opt("includeSubfolders", options.include_subfolders),
        opt("includeEmptyPermGroups", options.include_empty_perm_groups),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn build_user_permissions_params(options: &GetUserPermissionsOptions) -> Vec<(String, String)> {
    let mut params: Vec<(String, String)> = [opt("includeSubfolders", options.include_subfolders)]
        .into_iter()
        .flatten()
        .collect();

    if let Some(user_id) = options.user_id {
        params.push(("userId".to_string(), user_id.to_string()));
    } else if let Some(user_email) = options
        .user_email
        .as_ref()
        .filter(|value| !value.is_empty())
    {
        params.push(("userEmail".to_string(), user_email.clone()));
    }

    params
}

fn validate_optional_non_empty(field_name: &str, value: Option<&str>) -> Result<(), LabkeyError> {
    if value.is_some_and(|value| value.trim().is_empty()) {
        return Err(LabkeyError::InvalidInput(format!(
            "{field_name} cannot be empty"
        )));
    }

    Ok(())
}

fn build_securable_resources_params(
    options: &GetSecurableResourcesOptions,
) -> Vec<(String, String)> {
    [
        opt("includeSubfolders", options.include_subfolders),
        opt(
            "includeEffectivePermissions",
            options.include_effective_permissions,
        ),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn map_roles(response: GetRolesRawResponse) -> Result<Vec<Role>, LabkeyError> {
    let permission_map: HashMap<String, RolePermission> = response
        .permissions
        .into_iter()
        .map(|permission| {
            (
                permission.unique_name.clone(),
                RolePermission {
                    unique_name: Some(permission.unique_name),
                    name: permission.name,
                    description: permission.description,
                    source_module: permission.source_module,
                },
            )
        })
        .collect();

    response
        .roles
        .into_iter()
        .map(|role| {
            let mapped_permissions: Result<Vec<RolePermission>, LabkeyError> = role
                .permissions
                .into_iter()
                .map(|permission_name| {
                    permission_map.get(&permission_name).cloned().ok_or_else(|| {
                        LabkeyError::UnexpectedResponse {
                            status: StatusCode::OK,
                            text: format!(
                                "invalid getRoles response: unknown permission reference `{permission_name}`"
                            ),
                        }
                    })
                })
                .collect();

            Ok(Role {
                unique_name: role.unique_name,
                name: role.name,
                description: role.description,
                excluded_principals: role.excluded_principals,
                permissions: mapped_permissions?,
                source_module: role.source_module,
            })
        })
        .collect()
}

fn extract_securable_resources(
    response: &serde_json::Value,
) -> Result<SecurableResource, LabkeyError> {
    serde_json::from_value::<GetSecurableResourcesEnvelope>(response.clone())
        .map(|envelope| envelope.resources)
        .map_err(|_| LabkeyError::UnexpectedResponse {
            status: StatusCode::OK,
            text: format!("invalid getSecurableResources response: {response}"),
        })
}

impl LabkeyClient {
    /// Retrieve effective permissions by group for a container.
    ///
    /// Sends a GET request to `security-getGroupPerms.api`.
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
    /// use labkey_rs::security::GetGroupPermissionsOptions;
    ///
    /// let response = client
    ///     .get_group_permissions(
    ///         GetGroupPermissionsOptions::builder()
    ///             .include_subfolders(true)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Container: {}", response.container.path);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_group_permissions(
        &self,
        options: GetGroupPermissionsOptions,
    ) -> Result<GroupPermissionsResponse, LabkeyError> {
        let url = self.build_url(
            "security",
            "getGroupPerms.api",
            options.container_path.as_deref(),
        );
        let params = build_group_permissions_params(&options);
        self.get(url, &params).await
    }

    /// Retrieve permissions for a specific user or the current user.
    ///
    /// Sends a GET request to `security-getUserPerms.api`.
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
    /// use labkey_rs::security::GetUserPermissionsOptions;
    ///
    /// let response = client
    ///     .get_user_permissions(
    ///         GetUserPermissionsOptions::builder()
    ///             .user_email("analyst@example.com".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("User id: {}", response.user.user_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_user_permissions(
        &self,
        options: GetUserPermissionsOptions,
    ) -> Result<GetUserPermissionsResponse, LabkeyError> {
        validate_optional_non_empty(
            "get_user_permissions user_email",
            options.user_email.as_deref(),
        )?;

        let url = self.build_url(
            "security",
            "getUserPerms.api",
            options.container_path.as_deref(),
        );
        let params = build_user_permissions_params(&options);
        self.get(url, &params).await
    }

    /// Retrieve all roles defined on the server with expanded permissions.
    ///
    /// Sends a GET request to `security-getRoles.api`.
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
    /// use labkey_rs::security::GetRolesOptions;
    ///
    /// let roles = client.get_roles(GetRolesOptions::builder().build()).await?;
    ///
    /// println!("Role count: {}", roles.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_roles(&self, options: GetRolesOptions) -> Result<Vec<Role>, LabkeyError> {
        let url = self.build_url(
            "security",
            "getRoles.api",
            options.container_path.as_deref(),
        );
        let response: GetRolesRawResponse = self.get(url, &[]).await?;
        map_roles(response)
    }

    /// Retrieve securable resources for a container tree.
    ///
    /// Sends a GET request to `security-getSecurableResources.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, or the response body does not include a valid
    /// `resources` envelope.
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
    /// use labkey_rs::security::GetSecurableResourcesOptions;
    ///
    /// let resources = client
    ///     .get_securable_resources(
    ///         GetSecurableResourcesOptions::builder()
    ///             .include_subfolders(true)
    ///             .include_effective_permissions(true)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Root resource id: {}", resources.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_securable_resources(
        &self,
        options: GetSecurableResourcesOptions,
    ) -> Result<SecurableResource, LabkeyError> {
        let url = self.build_url(
            "security",
            "getSecurableResources.api",
            options.container_path.as_deref(),
        );
        let params = build_securable_resources_params(&options);
        let response: serde_json::Value = self.get(url, &params).await?;
        extract_securable_resources(&response)
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
    fn security_permission_endpoint_urls_match_expected_actions() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url("security", "getGroupPerms.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-getGroupPerms.api"
        );
        assert_eq!(
            client
                .build_url("security", "getUserPerms.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-getUserPerms.api"
        );
        assert_eq!(
            client
                .build_url("security", "getRoles.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-getRoles.api"
        );
        assert_eq!(
            client
                .build_url(
                    "security",
                    "getSecurableResources.api",
                    Some("/Alt/Container")
                )
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-getSecurableResources.api"
        );
    }

    #[test]
    fn group_permissions_response_deserializes_recursive_children() {
        let value = serde_json::json!({
            "container": {
                "id": "c1",
                "name": "Project",
                "path": "/Home/Project",
                "groups": [{ "id": 5, "name": "Readers" }],
                "children": [
                    {
                        "id": "c2",
                        "name": "Subfolder",
                        "path": "/Home/Project/Subfolder",
                        "groups": [],
                        "children": []
                    }
                ]
            }
        });

        let response: GroupPermissionsResponse =
            serde_json::from_value(value).expect("response should deserialize");
        assert_eq!(response.container.id, "c1");
        assert_eq!(response.container.groups.len(), 1);
        assert_eq!(response.container.children.len(), 1);
        assert_eq!(response.container.children[0].id, "c2");
    }

    #[test]
    fn group_permissions_response_deserializes_minimal_fixture() {
        let value = serde_json::json!({
            "container": {
                "id": "c1",
                "name": "Project",
                "path": "/Home/Project"
            }
        });

        let response: GroupPermissionsResponse =
            serde_json::from_value(value).expect("minimal response should deserialize");
        assert_eq!(response.container.id, "c1");
        assert!(response.container.groups.is_empty());
        assert!(response.container.children.is_empty());
    }

    #[test]
    fn securable_resource_deserializes_recursive_children() {
        let value = serde_json::json!({
            "id": "root",
            "name": "Root",
            "resourceClass": "org.labkey.core.project.ProjectImpl",
            "children": [
                {
                    "id": "child",
                    "name": "Child",
                    "resourceClass": "org.labkey.study.model.StudyImpl",
                    "children": []
                }
            ]
        });

        let resource: SecurableResource =
            serde_json::from_value(value).expect("resource should deserialize");
        assert_eq!(resource.id, "root");
        assert_eq!(resource.children.len(), 1);
        assert_eq!(resource.children[0].id, "child");
    }

    #[test]
    fn user_permissions_params_prefer_user_id_over_email() {
        let options = GetUserPermissionsOptions::builder()
            .user_id(101)
            .user_email("ignored@example.com".to_string())
            .include_subfolders(true)
            .build();

        let params = build_user_permissions_params(&options);
        assert!(params.contains(&("includeSubfolders".to_string(), "true".to_string())));
        assert!(params.contains(&("userId".to_string(), "101".to_string())));
        assert!(!params.iter().any(|(key, _)| key == "userEmail"));
    }

    #[test]
    fn get_roles_maps_permission_references_to_permission_objects() {
        let response = GetRolesRawResponse {
            permissions: vec![RawRolePermission {
                name: "Read".to_string(),
                unique_name: "org.labkey.api.security.permissions.ReadPermission".to_string(),
                description: Some("Can read data".to_string()),
                source_module: Some("Core".to_string()),
            }],
            roles: vec![RawRole {
                unique_name: Some("org.labkey.security.roles.ReaderRole".to_string()),
                name: "Reader".to_string(),
                description: Some("Read-only access".to_string()),
                excluded_principals: vec![],
                permissions: vec!["org.labkey.api.security.permissions.ReadPermission".to_string()],
                source_module: Some("Core".to_string()),
            }],
        };

        let roles = map_roles(response).expect("roles should map");
        assert_eq!(roles.len(), 1);
        assert_eq!(roles[0].name, "Reader");
        assert_eq!(roles[0].permissions.len(), 1);
        assert_eq!(roles[0].permissions[0].name, "Read");
    }

    #[test]
    fn get_roles_mapping_rejects_unknown_permission_reference() {
        let response = GetRolesRawResponse {
            permissions: Vec::new(),
            roles: vec![RawRole {
                unique_name: Some("org.labkey.security.roles.ReaderRole".to_string()),
                name: "Reader".to_string(),
                description: None,
                excluded_principals: vec![],
                permissions: vec!["org.labkey.api.security.permissions.ReadPermission".to_string()],
                source_module: None,
            }],
        };

        let error = map_roles(response).expect_err("unknown permission should fail");
        assert!(matches!(error, LabkeyError::UnexpectedResponse { .. }));
    }

    #[test]
    fn user_permissions_rejects_blank_user_email() {
        let error = validate_optional_non_empty("get_user_permissions user_email", Some("   "))
            .expect_err("blank email should fail");
        assert!(matches!(
            error,
            LabkeyError::InvalidInput(message)
                if message == "get_user_permissions user_email cannot be empty"
        ));
    }

    #[test]
    fn extract_securable_resources_rejects_missing_envelope() {
        let response = serde_json::json!({"container": {}});
        let error =
            extract_securable_resources(&response).expect_err("missing envelope should fail");
        match error {
            LabkeyError::UnexpectedResponse { status, text } => {
                assert_eq!(status, StatusCode::OK);
                assert!(text.contains("getSecurableResources"));
            }
            other => panic!("expected UnexpectedResponse, got {other:?}"),
        }
    }
}
