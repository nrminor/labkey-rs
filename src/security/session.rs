//! Session and impersonation security endpoints.

use std::collections::HashMap;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    client::{LabkeyClient, RequestOptions},
    error::LabkeyError,
};

/// Options for [`LabkeyClient::logout`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct LogoutOptions {
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::who_am_i`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct WhoAmIOptions {
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

/// Response type for [`LabkeyClient::who_am_i`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct WhoAmIResponse {
    /// Whether the current request is authenticated.
    #[serde(default)]
    pub authenticated: Option<bool>,
    /// Current user email when available.
    #[serde(default)]
    pub email: Option<String>,
    /// Whether the current session is impersonating another user.
    #[serde(default)]
    pub impersonated: Option<bool>,
    /// Current user id when available. Accepts both `userId` (JS convention)
    /// and `id` (Java convention) from the server response.
    #[serde(default, alias = "id")]
    pub user_id: Option<i64>,
    /// Display name of the current user when available.
    #[serde(default)]
    pub display_name: Option<String>,
    /// CSRF token for the current session.
    #[serde(default, rename = "CSRF")]
    pub csrf: Option<String>,
    /// Additional endpoint-specific response fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Options for [`LabkeyClient::delete_user`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct DeleteUserOptions {
    /// User id to delete.
    pub id: i64,
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

/// Response type for [`LabkeyClient::delete_user`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DeleteUserResponse {
    /// Server-provided success flag when present.
    #[serde(default)]
    pub success: Option<bool>,
    /// Server-provided deleted flag when present.
    #[serde(default)]
    pub deleted: Option<bool>,
    /// Additional endpoint-specific response fields.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Target for [`LabkeyClient::impersonate_user`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ImpersonateTarget {
    /// Impersonate by numeric user id.
    UserId(i64),
    /// Impersonate by user email.
    Email(String),
}

/// Options for [`LabkeyClient::impersonate_user`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct ImpersonateUserOptions {
    /// Target user to impersonate.
    pub target: ImpersonateTarget,
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::stop_impersonating`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct StopImpersonatingOptions {
    /// Optional container override for this request.
    pub container_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct DeleteUserBody {
    id: i64,
}

fn validate_non_empty(field_name: &str, value: &str) -> Result<(), LabkeyError> {
    if value.trim().is_empty() {
        return Err(LabkeyError::InvalidInput(format!(
            "{field_name} cannot be empty"
        )));
    }

    Ok(())
}

fn impersonate_target_param(target: &ImpersonateTarget) -> Result<(String, String), LabkeyError> {
    match target {
        ImpersonateTarget::UserId(user_id) => Ok(("userId".to_string(), user_id.to_string())),
        ImpersonateTarget::Email(email) => {
            validate_non_empty("impersonate_user email", email)?;
            Ok(("email".to_string(), email.clone()))
        }
    }
}

impl LabkeyClient {
    /// Log out the current session.
    ///
    /// Sends a POST request to `login-logout` (no `.api` suffix) with no JSON
    /// body and no query parameters.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails or the server returns an
    /// error response.
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
    /// use labkey_rs::security::LogoutOptions;
    ///
    /// client.logout(LogoutOptions::builder().build()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn logout(&self, options: LogoutOptions) -> Result<(), LabkeyError> {
        let url = self.build_url("login", "logout", options.container_path.as_deref());
        self.post_without_body(url).await
    }

    /// Return information about the current authenticated user.
    ///
    /// Sends a GET request to `login-whoami.api` with no query parameters.
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
    /// use labkey_rs::security::WhoAmIOptions;
    ///
    /// let response = client.who_am_i(WhoAmIOptions::builder().build()).await?;
    /// println!("Current user id: {:?}", response.user_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn who_am_i(&self, options: WhoAmIOptions) -> Result<WhoAmIResponse, LabkeyError> {
        let url = self.build_url("login", "whoami.api", options.container_path.as_deref());
        self.get(url, &[]).await
    }

    /// Delete a user by id.
    ///
    /// Sends a POST request to `security-deleteUser` (no `.api` suffix) with
    /// typed JSON body `{ id }`.
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
    /// use labkey_rs::security::DeleteUserOptions;
    ///
    /// let response = client
    ///     .delete_user(DeleteUserOptions::builder().id(101).build())
    ///     .await?;
    /// println!("Delete result: {response}");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_user(
        &self,
        options: DeleteUserOptions,
    ) -> Result<DeleteUserResponse, LabkeyError> {
        let url = self.build_url("security", "deleteUser", options.container_path.as_deref());
        let body = DeleteUserBody { id: options.id };
        self.post(url, &body).await
    }

    /// Start impersonating a target user.
    ///
    /// Sends a POST request to `user-impersonateUser.api` with query
    /// parameters and an empty request body.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError::InvalidInput`] when the email target is blank.
    /// Returns [`LabkeyError`] for request and response failures.
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
    /// use labkey_rs::security::{ImpersonateTarget, ImpersonateUserOptions};
    ///
    /// client
    ///     .impersonate_user(
    ///         ImpersonateUserOptions::builder()
    ///             .target(ImpersonateTarget::Email("analyst@example.com".to_string()))
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn impersonate_user(
        &self,
        options: ImpersonateUserOptions,
    ) -> Result<(), LabkeyError> {
        let mut url = self.build_url(
            "user",
            "impersonateUser.api",
            options.container_path.as_deref(),
        );

        let (key, value) = impersonate_target_param(&options.target)?;
        url.query_pairs_mut().append_pair(&key, &value);

        self.post_without_body(url).await
    }

    /// Stop impersonating and return to the authenticated principal.
    ///
    /// Sends a POST request to `login-stopImpersonating.api` with
    /// `RequestOptions` configured to disable redirect following and treat HTTP
    /// 302 as success.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails or the server returns an
    /// unexpected status.
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
    /// use labkey_rs::security::StopImpersonatingOptions;
    ///
    /// client
    ///     .stop_impersonating(StopImpersonatingOptions::builder().build())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn stop_impersonating(
        &self,
        options: StopImpersonatingOptions,
    ) -> Result<(), LabkeyError> {
        let url = self.build_url(
            "login",
            "stopImpersonating.api",
            options.container_path.as_deref(),
        );
        let request_options = RequestOptions {
            no_follow_redirects: true,
            accepted_statuses: vec![StatusCode::FOUND],
            ..RequestOptions::default()
        };
        self.post_without_body_with_options(url, &request_options)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClientConfig, Credential};
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::*};

    fn test_client(base_url: &str, container_path: &str) -> LabkeyClient {
        LabkeyClient::new(ClientConfig::new(
            base_url,
            Credential::ApiKey("test-key".to_string()),
            container_path,
        ))
        .expect("valid client config")
    }

    #[test]
    fn security_session_endpoint_urls_match_expected_actions() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url("login", "logout", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/login-logout"
        );
        assert_eq!(
            client
                .build_url("login", "whoami.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/login-whoami.api"
        );
        assert_eq!(
            client
                .build_url("security", "deleteUser", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/security-deleteUser"
        );
        assert_eq!(
            client
                .build_url("user", "impersonateUser.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/user-impersonateUser.api"
        );
        assert_eq!(
            client
                .build_url("login", "stopImpersonating.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/login-stopImpersonating.api"
        );
    }

    #[test]
    fn delete_user_body_serializes_id_field() {
        let body = DeleteUserBody { id: 101 };
        let value = serde_json::to_value(body).expect("delete user body should serialize");
        assert_eq!(value, serde_json::json!({"id": 101}));
    }

    #[test]
    fn impersonate_target_maps_user_id_and_email_query_params() {
        let user_id_param = impersonate_target_param(&ImpersonateTarget::UserId(17))
            .expect("user id target should map");
        assert_eq!(user_id_param, ("userId".to_string(), "17".to_string()));

        let email_param =
            impersonate_target_param(&ImpersonateTarget::Email("analyst@example.com".to_string()))
                .expect("email target should map");
        assert_eq!(
            email_param,
            ("email".to_string(), "analyst@example.com".to_string())
        );
    }

    #[test]
    fn impersonate_target_rejects_blank_email() {
        let error = impersonate_target_param(&ImpersonateTarget::Email("  ".to_string()))
            .expect_err("blank email should fail");
        assert!(matches!(
            error,
            LabkeyError::InvalidInput(message) if message == "impersonate_user email cannot be empty"
        ));
    }

    #[test]
    fn who_am_i_response_deserializes_from_fixture() {
        let value: serde_json::Value =
            serde_json::from_str(include_str!("../../tests/fixtures/whoami.json"))
                .expect("fixture should parse as JSON");
        let response: WhoAmIResponse =
            serde_json::from_value(value).expect("who am i response should parse");

        assert_eq!(response.user_id, Some(101));
        assert_eq!(response.email.as_deref(), Some("analyst@example.com"));
        assert_eq!(response.authenticated, Some(true));
        assert_eq!(response.impersonated, Some(false));
        assert_eq!(response.display_name.as_deref(), Some("Analyst User"));
        assert_eq!(response.csrf.as_deref(), Some("abc123token"));
    }

    #[test]
    fn who_am_i_response_deserializes_minimal_path() {
        let response: WhoAmIResponse =
            serde_json::from_value(serde_json::json!({})).expect("minimal response should parse");

        assert!(response.user_id.is_none());
        assert!(response.email.is_none());
        assert!(response.authenticated.is_none());
        assert!(response.impersonated.is_none());
        assert!(response.display_name.is_none());
        assert!(response.csrf.is_none());
        assert!(response.extra.is_empty());
    }

    #[test]
    fn who_am_i_response_accepts_java_style_id_field() {
        let response: WhoAmIResponse = serde_json::from_value(serde_json::json!({
            "id": 42,
            "email": "admin@example.com",
            "displayName": "Admin",
            "CSRF": "token123",
            "impersonated": false
        }))
        .expect("Java-style id field should parse");

        assert_eq!(response.user_id, Some(42));
        assert_eq!(response.display_name.as_deref(), Some("Admin"));
        assert_eq!(response.csrf.as_deref(), Some("token123"));
    }

    #[test]
    fn delete_user_response_deserializes_typed_fields_and_extra() {
        let response: DeleteUserResponse = serde_json::from_value(serde_json::json!({
            "success": true,
            "deleted": true,
            "auditId": 9001
        }))
        .expect("delete user response should parse");

        assert_eq!(response.success, Some(true));
        assert_eq!(response.deleted, Some(true));
        assert_eq!(
            response.extra.get("auditId"),
            Some(&serde_json::json!(9001))
        );
    }

    #[tokio::test]
    async fn logout_posts_without_query_params_or_request_body() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/Alt/Container/login-logout"))
            .and(query_param_is_missing("id"))
            .and(query_param_is_missing("email"))
            .and(query_param_is_missing("userId"))
            .and(header("x-requested-with", "XMLHttpRequest"))
            .and(basic_auth("apikey", "test-key"))
            .and(body_string(String::new()))
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server.uri(), "/Default");
        client
            .logout(
                LogoutOptions::builder()
                    .container_path("/Alt/Container".to_string())
                    .build(),
            )
            .await
            .expect("logout should succeed");
    }
}
