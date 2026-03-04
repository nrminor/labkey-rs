//! HTTP client and URL construction for the `LabKey` REST API.
//!
//! The [`LabkeyClient`] struct is the main entry point for interacting with a
//! `LabKey` server. It holds a [`reqwest::Client`], the server's base URL, a
//! default container path, and authentication credentials. Every API endpoint
//! method is an async method on this struct.

use url::Url;

use crate::error::{ApiErrorBody, LabkeyError};

/// Authentication credentials for a `LabKey` server.
#[derive(Debug, Clone)]
pub enum Credential {
    /// HTTP Basic authentication with an email and password.
    Basic {
        /// The user's email address.
        email: String,
        /// The user's password.
        password: String,
    },
    /// `LabKey` API key, sent as basic auth with username `"apikey"` and the
    /// key as the password, per `LabKey` convention.
    ApiKey(
        /// The API key string.
        String,
    ),
}

/// Configuration for constructing a [`LabkeyClient`].
///
/// # Example
///
/// ```no_run
/// use labkey_rs::{ClientConfig, Credential, LabkeyClient};
///
/// let client = LabkeyClient::new(ClientConfig {
///     base_url: "https://labkey.example.com/labkey".into(),
///     credential: Credential::ApiKey("my-api-key".into()),
///     container_path: "/MyProject/MyFolder".into(),
/// }).expect("valid configuration");
/// ```
pub struct ClientConfig {
    /// The base URL of the `LabKey` server (e.g., `"https://labkey.example.com/labkey"`).
    pub base_url: String,
    /// Authentication credentials.
    pub credential: Credential,
    /// Default container path (e.g., `"/MyProject/MyFolder"`).
    /// Individual requests can override this.
    pub container_path: String,
}

/// Percent-encode each segment of a container path individually.
///
/// Container names in `LabKey` can contain spaces and special characters.
/// We split on `/`, encode each segment, and rejoin — matching the JS
/// client's `encodePath` function in `ActionURL.ts`.
fn encode_container_path(path: &str) -> String {
    path.trim_matches('/')
        .split('/')
        .map(|segment| urlencoding::encode(segment))
        .collect::<Vec<_>>()
        .join("/")
}

/// Async client for the `LabKey` Server REST API.
///
/// Construct one via [`LabkeyClient::new`], then call endpoint methods like
/// [`select_rows`](Self::select_rows) or [`execute_sql`](Self::execute_sql).
pub struct LabkeyClient {
    http: reqwest::Client,
    base_url: Url,
    container_path: String,
    credential: Credential,
}

impl LabkeyClient {
    /// Create a new client from the given configuration.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError::Url`] if `config.base_url` is not a valid URL.
    pub fn new(config: ClientConfig) -> Result<Self, LabkeyError> {
        let base_url = Url::parse(&config.base_url)?;
        let http = reqwest::Client::new();
        Ok(Self {
            http,
            base_url,
            container_path: config.container_path,
            credential: config.credential,
        })
    }

    /// Build a `LabKey` action URL.
    ///
    /// `LabKey` URLs follow the pattern
    /// `{base_url}/{container_path}/{controller}-{action}` where `action`
    /// includes the extension (e.g., `"getQuery.api"`).
    ///
    /// Container path segments are percent-encoded individually so that
    /// folder names containing spaces or special characters produce valid
    /// URLs. This matches the JS client's `encodePath` behavior.
    ///
    /// If `container_override` is `None`, the client's default container path
    /// is used.
    pub(crate) fn build_url(
        &self,
        controller: &str,
        action: &str,
        container_override: Option<&str>,
    ) -> Url {
        let container = container_override.unwrap_or(&self.container_path);
        let encoded_container = encode_container_path(container);

        let base_path = self.base_url.path().trim_end_matches('/');
        let path = if encoded_container.is_empty() {
            format!("{base_path}/{controller}-{action}")
        } else {
            format!("{base_path}/{encoded_container}/{controller}-{action}")
        };

        let mut url = self.base_url.clone();
        url.set_path(&path);
        url
    }

    /// Apply standard headers and authentication to a request builder.
    ///
    /// Sets `X-Requested-With: XMLHttpRequest` (which `LabKey` servers expect
    /// on API requests) and the appropriate authentication credentials.
    fn prepare_request(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let builder = builder.header("X-Requested-With", "XMLHttpRequest");
        match &self.credential {
            Credential::Basic { email, password } => builder.basic_auth(email, Some(password)),
            Credential::ApiKey(key) => builder.basic_auth("apikey", Some(key)),
        }
    }

    /// Send a GET request and deserialize the JSON response.
    pub(crate) async fn get<T: serde::de::DeserializeOwned>(
        &self,
        url: Url,
        params: &[(String, String)],
    ) -> Result<T, LabkeyError> {
        let builder = self.http.get(url).query(params);
        let builder = self.prepare_request(builder);
        let response = builder.send().await?;
        self.handle_response(response).await
    }

    /// Send a POST request with a JSON body and deserialize the JSON response.
    pub(crate) async fn post<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        url: Url,
        body: &B,
    ) -> Result<T, LabkeyError> {
        let builder = self.http.post(url).json(body);
        let builder = self.prepare_request(builder);
        let response = builder.send().await?;
        self.handle_response(response).await
    }

    /// Check the response status and either deserialize the success body or
    /// construct an appropriate error.
    ///
    /// On non-success status codes, the body is read as text and we attempt to
    /// parse it as [`ApiErrorBody`]. If that fails, we return
    /// [`LabkeyError::UnexpectedResponse`] with the raw text.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
    ) -> Result<T, LabkeyError> {
        let status = response.status();
        if status.is_success() {
            let body = response.json::<T>().await?;
            Ok(body)
        } else {
            let text = response.text().await.unwrap_or_default();
            match serde_json::from_str::<ApiErrorBody>(&text) {
                Ok(api_error) => Err(LabkeyError::Api {
                    status,
                    body: api_error,
                }),
                Err(_) => Err(LabkeyError::UnexpectedResponse { status, text }),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_client(base_url: &str, container_path: &str) -> LabkeyClient {
        LabkeyClient::new(ClientConfig {
            base_url: base_url.into(),
            credential: Credential::ApiKey("test-key".into()),
            container_path: container_path.into(),
        })
        .expect("valid test config")
    }

    #[test]
    fn build_url_basic() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");
        let url = client.build_url("query", "getQuery.api", None);
        assert_eq!(
            url.as_str(),
            "https://labkey.example.com/labkey/MyProject/MyFolder/query-getQuery.api"
        );
    }

    #[test]
    fn build_url_with_container_override() {
        let client = test_client("https://labkey.example.com/labkey", "/Default");
        let url = client.build_url("query", "executeSql.api", Some("/Other/Container"));
        assert_eq!(
            url.as_str(),
            "https://labkey.example.com/labkey/Other/Container/query-executeSql.api"
        );
    }

    #[test]
    fn build_url_strips_extra_slashes() {
        let client = test_client("https://labkey.example.com/labkey/", "//MyProject/");
        let url = client.build_url("query", "getQuery.api", None);
        assert_eq!(
            url.as_str(),
            "https://labkey.example.com/labkey/MyProject/query-getQuery.api"
        );
    }

    #[test]
    fn build_url_no_context_path() {
        let client = test_client("https://labkey.example.com", "/MyProject");
        let url = client.build_url("security", "getContainers.api", None);
        assert_eq!(
            url.as_str(),
            "https://labkey.example.com/MyProject/security-getContainers.api"
        );
    }

    #[test]
    fn build_url_bare_container() {
        let client = test_client("https://labkey.example.com/labkey", "Home");
        let url = client.build_url("project", "begin.api", None);
        assert_eq!(
            url.as_str(),
            "https://labkey.example.com/labkey/Home/project-begin.api"
        );
    }

    #[test]
    fn build_url_root_container() {
        let client = test_client("https://labkey.example.com/labkey", "/");
        let url = client.build_url("query", "getQuery.api", None);
        assert_eq!(
            url.as_str(),
            "https://labkey.example.com/labkey/query-getQuery.api"
        );
    }

    #[test]
    fn build_url_encodes_special_characters_in_container() {
        let client = test_client(
            "https://labkey.example.com/labkey",
            "/My Project/Sub Folder",
        );
        let url = client.build_url("query", "getQuery.api", None);
        assert_eq!(
            url.as_str(),
            "https://labkey.example.com/labkey/My%20Project/Sub%20Folder/query-getQuery.api"
        );
    }

    #[test]
    fn build_url_encodes_ampersand_in_container() {
        let client = test_client("https://labkey.example.com/labkey", "/R&D/Tests");
        let url = client.build_url("query", "getQuery.api", None);
        assert_eq!(
            url.as_str(),
            "https://labkey.example.com/labkey/R%26D/Tests/query-getQuery.api"
        );
    }

    #[test]
    fn new_rejects_invalid_url() {
        let result = LabkeyClient::new(ClientConfig {
            base_url: "not a url".into(),
            credential: Credential::ApiKey("key".into()),
            container_path: "/".into(),
        });
        assert!(matches!(result, Err(crate::error::LabkeyError::Url(_))));
    }
}
