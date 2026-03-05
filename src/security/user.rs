//! User-focused security endpoints.

use serde::{Deserialize, Serialize};

use crate::{client::LabkeyClient, common::opt, error::LabkeyError, security::User};

/// Options for [`LabkeyClient::create_new_user`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct CreateNewUserOptions {
    /// Email address, or semicolon-separated email addresses, for user creation.
    pub email: String,
    /// Optional container override for this request.
    pub container_path: Option<String>,
    /// Optional message included in the welcome email.
    pub optional_message: Option<String>,
    /// Optional flag controlling whether welcome email is sent.
    pub send_email: Option<bool>,
}

/// Individual user entry returned from [`LabkeyClient::create_new_user`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CreatedUser {
    /// User email address.
    pub email: String,
    /// Whether this user account was newly created.
    pub is_new: bool,
    /// Server-provided status message.
    pub message: String,
    /// Numeric user id.
    pub user_id: i64,
}

/// Response type for [`LabkeyClient::create_new_user`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CreateNewUserResponse {
    /// Primary email value for single-user create flows.
    #[serde(default)]
    pub email: Option<String>,
    /// HTML errors returned by partial/multi-user create attempts.
    #[serde(default)]
    pub html_errors: Vec<String>,
    /// Server-provided status message.
    #[serde(default)]
    pub message: Option<String>,
    /// Whether user creation succeeded.
    pub success: bool,
    /// Primary user id for single-user create flows.
    #[serde(default)]
    pub user_id: Option<i64>,
    /// User entries returned for created users.
    #[serde(default)]
    pub users: Vec<CreatedUser>,
}

/// Options for [`LabkeyClient::ensure_login`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct EnsureLoginOptions {
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

/// Response type for [`LabkeyClient::ensure_login`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EnsureLoginResponse {
    /// Current authenticated user information.
    pub current_user: User,
}

/// Options for [`LabkeyClient::get_users`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetUsersOptions {
    /// Optional container override for this request.
    pub container_path: Option<String>,
    /// Filter by group name when `group_id` is unset.
    pub group: Option<String>,
    /// Filter by group id.
    pub group_id: Option<i64>,
    /// Optional user name prefix filter.
    pub name: Option<String>,
    /// Include users from nested groups.
    pub all_members: Option<bool>,
    /// Filter by activity status.
    pub active: Option<bool>,
    /// Optional permissions filter. Multiple values are treated as an AND by the server.
    pub permissions: Option<Vec<String>>,
}

/// Options for [`LabkeyClient::get_users_with_permissions`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetUsersWithPermissionsOptions {
    /// One or more permissions required for returned users.
    pub permissions: Vec<String>,
    /// Optional container override for this request.
    pub container_path: Option<String>,
    /// Filter by group name when `group_id` is unset.
    pub group: Option<String>,
    /// Filter by group id.
    pub group_id: Option<i64>,
    /// Optional user name prefix filter.
    pub name: Option<String>,
    /// Include users from nested groups.
    pub all_members: Option<bool>,
    /// Filter by activity status.
    pub active: Option<bool>,
    /// Include inactive users when supported by the selected API version.
    pub include_inactive: Option<bool>,
    /// Optional endpoint API version (for example, `23.11`).
    pub required_version: Option<String>,
}

/// Response type for [`LabkeyClient::get_users`] and [`LabkeyClient::get_users_with_permissions`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GetUsersResponse {
    /// Requested container path.
    #[serde(default)]
    pub container: Option<String>,
    /// Echoed name filter when provided.
    #[serde(default)]
    pub name: Option<String>,
    /// Matching users.
    #[serde(default)]
    pub users: Vec<User>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateNewUserBody {
    email: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    optional_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    send_email: Option<bool>,
}

fn add_permissions_params(params: &mut Vec<(String, String)>, permissions: &[String]) {
    params.extend(
        permissions
            .iter()
            .cloned()
            .map(|permission| ("permissions".to_string(), permission)),
    );
}

fn validate_non_empty(field_name: &str, value: &str) -> Result<(), LabkeyError> {
    if value.trim().is_empty() {
        return Err(LabkeyError::InvalidInput(format!(
            "{field_name} cannot be empty"
        )));
    }

    Ok(())
}

fn validate_permissions(field_name: &str, permissions: &[String]) -> Result<(), LabkeyError> {
    if permissions.is_empty() {
        return Err(LabkeyError::InvalidInput(format!(
            "{field_name} requires at least one permission"
        )));
    }

    if permissions
        .iter()
        .any(|permission| permission.trim().is_empty())
    {
        return Err(LabkeyError::InvalidInput(format!(
            "{field_name} permissions cannot be empty"
        )));
    }

    Ok(())
}

fn build_get_users_params(options: &GetUsersOptions) -> Vec<(String, String)> {
    let mut params: Vec<(String, String)> = [
        opt("name", options.name.as_deref()),
        opt("allMembers", options.all_members),
        opt("active", options.active),
    ]
    .into_iter()
    .flatten()
    .collect();

    if let Some(group_id) = options.group_id {
        params.push(("groupId".to_string(), group_id.to_string()));
    } else if let Some(group) = options.group.as_ref().filter(|value| !value.is_empty()) {
        params.push(("group".to_string(), group.clone()));
    }

    if let Some(permissions) = options.permissions.as_ref() {
        add_permissions_params(&mut params, permissions);
    }

    params
}

fn build_get_users_with_permissions_params(
    options: &GetUsersWithPermissionsOptions,
) -> Vec<(String, String)> {
    let mut params: Vec<(String, String)> = [
        opt("name", options.name.as_deref()),
        opt("allMembers", options.all_members),
        opt("active", options.active),
        opt("includeInactive", options.include_inactive),
        opt("apiVersion", options.required_version.as_deref()),
    ]
    .into_iter()
    .flatten()
    .collect();

    if let Some(group_id) = options.group_id {
        params.push(("groupId".to_string(), group_id.to_string()));
    } else if let Some(group) = options.group.as_ref().filter(|value| !value.is_empty()) {
        params.push(("group".to_string(), group.clone()));
    }

    add_permissions_params(&mut params, &options.permissions);
    params
}

impl LabkeyClient {
    /// Create one or more user accounts.
    ///
    /// Sends a POST request to `security-createNewUser.api`.
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
    /// use labkey_rs::security::CreateNewUserOptions;
    ///
    /// let created = client
    ///     .create_new_user(
    ///         CreateNewUserOptions::builder()
    ///             .email("analyst@example.com".to_string())
    ///             .send_email(true)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Create succeeded: {}", created.success);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_new_user(
        &self,
        options: CreateNewUserOptions,
    ) -> Result<CreateNewUserResponse, LabkeyError> {
        validate_non_empty("create_new_user email", &options.email)?;

        let url = self.build_url(
            "security",
            "createNewUser.api",
            options.container_path.as_deref(),
        );
        let body = CreateNewUserBody {
            email: options.email,
            optional_message: options.optional_message,
            send_email: options.send_email,
        };
        self.post(url, &body).await
    }

    /// Ensure the caller is authenticated and return current user details.
    ///
    /// Sends a GET request to `security-ensureLogin.api`.
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
    /// use labkey_rs::security::EnsureLoginOptions;
    ///
    /// let login = client
    ///     .ensure_login(EnsureLoginOptions::builder().build())
    ///     .await?;
    ///
    /// println!("Current user id: {}", login.current_user.user_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn ensure_login(
        &self,
        options: EnsureLoginOptions,
    ) -> Result<EnsureLoginResponse, LabkeyError> {
        let url = self.build_url(
            "security",
            "ensureLogin.api",
            options.container_path.as_deref(),
        );
        self.get(url, &[]).await
    }

    /// Retrieve users matching filter criteria.
    ///
    /// Sends a GET request to `user-getUsers.api`.
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
    /// use labkey_rs::security::GetUsersOptions;
    ///
    /// let users = client
    ///     .get_users(
    ///         GetUsersOptions::builder()
    ///             .name("ana".to_string())
    ///             .active(true)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Matched users: {}", users.users.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_users(
        &self,
        options: GetUsersOptions,
    ) -> Result<GetUsersResponse, LabkeyError> {
        if let Some(permissions) = options.permissions.as_ref() {
            validate_permissions("get_users", permissions)?;
        }

        let url = self.build_url("user", "getUsers.api", options.container_path.as_deref());
        let params = build_get_users_params(&options);
        self.get(url, &params).await
    }

    /// Retrieve users that hold all specified permissions.
    ///
    /// Sends a GET request to `user-getUsersWithPermissions.api`.
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
    /// use labkey_rs::security::GetUsersWithPermissionsOptions;
    ///
    /// let users = client
    ///     .get_users_with_permissions(
    ///         GetUsersWithPermissionsOptions::builder()
    ///             .permissions(vec!["ReadPermission".to_string()])
    ///             .include_inactive(true)
    ///             .required_version("23.11".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Matched users: {}", users.users.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_users_with_permissions(
        &self,
        options: GetUsersWithPermissionsOptions,
    ) -> Result<GetUsersResponse, LabkeyError> {
        validate_permissions("get_users_with_permissions", &options.permissions)?;

        let url = self.build_url(
            "user",
            "getUsersWithPermissions.api",
            options.container_path.as_deref(),
        );
        let params = build_get_users_with_permissions_params(&options);
        self.get(url, &params).await
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
    fn security_user_endpoint_urls_match_expected_actions() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url("security", "createNewUser.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-createNewUser.api"
        );
        assert_eq!(
            client
                .build_url("security", "ensureLogin.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-ensureLogin.api"
        );
        assert_eq!(
            client
                .build_url("user", "getUsers.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/user-getUsers.api"
        );
        assert_eq!(
            client
                .build_url(
                    "user",
                    "getUsersWithPermissions.api",
                    Some("/Alt/Container")
                )
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/user-getUsersWithPermissions.api"
        );
    }

    #[test]
    fn create_new_user_body_serializes_expected_wire_keys() {
        let body = CreateNewUserBody {
            email: "a@example.com;b@example.com".to_string(),
            optional_message: Some("welcome".to_string()),
            send_email: Some(false),
        };

        let value = serde_json::to_value(body).expect("should serialize");
        assert_eq!(
            value.get("email"),
            Some(&serde_json::json!("a@example.com;b@example.com"))
        );
        assert_eq!(
            value.get("optionalMessage"),
            Some(&serde_json::json!("welcome"))
        );
        assert_eq!(value.get("sendEmail"), Some(&serde_json::json!(false)));
    }

    #[test]
    fn get_users_params_follow_group_precedence_and_repeat_permissions() {
        let options = GetUsersOptions::builder()
            .group_id(17)
            .group("IgnoredGroup".to_string())
            .name("ana".to_string())
            .all_members(true)
            .active(false)
            .permissions(vec![
                "ReadPermission".to_string(),
                "InsertPermission".to_string(),
            ])
            .build();

        let params = build_get_users_params(&options);

        assert!(params.contains(&("groupId".to_string(), "17".to_string())));
        assert!(!params.iter().any(|(key, _)| key == "group"));
        assert!(params.contains(&("name".to_string(), "ana".to_string())));
        assert!(params.contains(&("allMembers".to_string(), "true".to_string())));
        assert!(params.contains(&("active".to_string(), "false".to_string())));
        assert_eq!(
            params
                .iter()
                .filter(|(key, _)| key == "permissions")
                .count(),
            2
        );
    }

    #[test]
    fn get_users_with_permissions_params_include_api_version_and_include_inactive() {
        let options = GetUsersWithPermissionsOptions::builder()
            .permissions(vec!["ReadPermission".to_string()])
            .group("Developers".to_string())
            .include_inactive(true)
            .required_version("23.11".to_string())
            .build();

        let params = build_get_users_with_permissions_params(&options);

        assert!(params.contains(&("group".to_string(), "Developers".to_string())));
        assert!(params.contains(&("includeInactive".to_string(), "true".to_string())));
        assert!(params.contains(&("apiVersion".to_string(), "23.11".to_string())));
        assert!(params.contains(&("permissions".to_string(), "ReadPermission".to_string())));
    }

    #[test]
    fn create_new_user_rejects_empty_email() {
        let error =
            validate_non_empty("create_new_user email", "  ").expect_err("empty email should fail");
        assert!(matches!(
            error,
            LabkeyError::InvalidInput(message) if message == "create_new_user email cannot be empty"
        ));
    }

    #[test]
    fn get_users_with_permissions_rejects_empty_permissions() {
        let error = validate_permissions("get_users_with_permissions", &[])
            .expect_err("empty permissions should fail");
        assert!(matches!(
            error,
            LabkeyError::InvalidInput(message)
                if message == "get_users_with_permissions requires at least one permission"
        ));
    }

    #[test]
    fn get_users_rejects_blank_permission_values() {
        let error = validate_permissions(
            "get_users",
            &["ReadPermission".to_string(), "   ".to_string()],
        )
        .expect_err("blank permission should fail");
        assert!(matches!(
            error,
            LabkeyError::InvalidInput(message) if message == "get_users permissions cannot be empty"
        ));
    }

    #[test]
    fn create_new_user_response_deserializes_nested_users_and_html_errors() {
        let value = serde_json::json!({
            "email": "analyst@example.com",
            "htmlErrors": ["Already exists"],
            "message": "Created users",
            "success": true,
            "userId": 41,
            "users": [
                {
                    "email": "analyst@example.com",
                    "isNew": true,
                    "message": "created",
                    "userId": 41
                }
            ]
        });

        let response: CreateNewUserResponse =
            serde_json::from_value(value).expect("create user response should parse");
        assert!(response.success);
        assert_eq!(response.user_id, Some(41));
        assert_eq!(response.html_errors, vec!["Already exists"]);
        assert_eq!(response.users.len(), 1);
        assert!(response.users[0].is_new);
    }

    #[test]
    fn get_users_response_deserializes_happy_path() {
        let value = serde_json::json!({
            "container": "/Home/Project",
            "name": "ana",
            "users": [
                {
                    "userId": 101,
                    "email": "analyst@example.com",
                    "displayName": "Analyst",
                    "active": true
                }
            ]
        });

        let response: GetUsersResponse =
            serde_json::from_value(value).expect("get users response should parse");
        assert_eq!(response.container.as_deref(), Some("/Home/Project"));
        assert_eq!(response.name.as_deref(), Some("ana"));
        assert_eq!(response.users.len(), 1);
        assert_eq!(response.users[0].user_id, 101);
    }

    #[test]
    fn get_users_response_deserializes_minimal_fixture() {
        let value = serde_json::json!({"users": []});

        let response: GetUsersResponse =
            serde_json::from_value(value).expect("minimal get users response should parse");
        assert!(response.container.is_none());
        assert!(response.name.is_none());
        assert!(response.users.is_empty());
    }
}
