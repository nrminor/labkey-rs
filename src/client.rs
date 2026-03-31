//! HTTP client and URL construction for the LabKey REST API.
//!
//! The [`LabkeyClient`] struct is the main entry point for interacting with a
//! LabKey server. It holds a [`reqwest::Client`], the server's base URL, a
//! default container path, and authentication credentials. Every API endpoint
//! method is an async method on this struct.

use std::io::BufRead;
use std::time::Duration;

use reqwest::StatusCode;
use url::Url;

use crate::error::{ApiErrorBody, LabkeyError};

/// Authentication credentials for a LabKey server.
#[derive(Debug, Clone)]
pub enum Credential {
    /// HTTP Basic authentication with an email and password.
    Basic {
        /// The user's email address.
        email: String,
        /// The user's password.
        password: String,
    },
    /// LabKey API key, sent as basic auth with username `"apikey"` and the
    /// key as the password, per LabKey convention.
    ApiKey(
        /// The API key string.
        String,
    ),
    /// Anonymous access with no authentication credentials.
    ///
    /// Requests are sent without an `Authorization` header, so the server
    /// grants only guest-level permissions. This matches Java's
    /// `GuestCredentialsProvider`.
    Guest,
}

impl Credential {
    /// Read credentials for `host` from the user's `~/.netrc` file.
    ///
    /// Looks for `~/.netrc` first, then `~/_netrc` (the Windows convention),
    /// matching Java's `NetrcCredentialsProvider` behavior. Returns
    /// [`Credential::Basic`] with the `login` and `password` from the
    /// matching `machine` entry.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError::InvalidInput`] if:
    /// - The home directory cannot be determined
    /// - Neither `~/.netrc` nor `~/_netrc` exists
    /// - The file cannot be read or parsed
    /// - No entry matches the given `host`
    /// - The matching entry has no password
    ///
    /// If you want to fall back to guest access when no netrc entry is found,
    /// catch the error and use [`Credential::Guest`] instead.
    pub fn from_netrc(host: &str) -> Result<Self, LabkeyError> {
        let home = home_dir().ok_or_else(|| {
            LabkeyError::InvalidInput("cannot determine home directory for .netrc lookup".into())
        })?;

        let netrc_path = home.join(".netrc");
        let alt_path = home.join("_netrc");

        let path = if netrc_path.is_file() {
            netrc_path
        } else if alt_path.is_file() {
            alt_path
        } else {
            return Err(LabkeyError::InvalidInput(format!(
                "no .netrc or _netrc file found in {}",
                home.display()
            )));
        };

        let file = std::fs::File::open(&path).map_err(|e| {
            LabkeyError::InvalidInput(format!("failed to open {}: {e}", path.display()))
        })?;
        let reader = std::io::BufReader::new(file);

        Self::from_netrc_reader(reader, host)
    }

    /// Parse credentials for `host` from a netrc-formatted byte stream.
    ///
    /// This is the testable core of [`from_netrc`](Self::from_netrc),
    /// separated so tests can supply in-memory content without touching the
    /// filesystem.
    fn from_netrc_reader(reader: impl BufRead, host: &str) -> Result<Self, LabkeyError> {
        let netrc = netrc::Netrc::parse(reader)
            .map_err(|e| LabkeyError::InvalidInput(format!("failed to parse netrc file: {e:?}")))?;

        for (entry_host, machine) in &netrc.hosts {
            if entry_host == host {
                let password = machine.password.as_ref().ok_or_else(|| {
                    LabkeyError::InvalidInput(format!(
                        "netrc entry for {host} has no password field"
                    ))
                })?;
                return Ok(Self::Basic {
                    email: machine.login.clone(),
                    password: password.clone(),
                });
            }
        }

        Err(LabkeyError::InvalidInput(format!(
            "no netrc entry found for host \"{host}\""
        )))
    }
}

/// Resolve the user's home directory.
///
/// Uses `$HOME` on Unix and `%USERPROFILE%` on Windows, matching the
/// conventional locations for `.netrc` / `_netrc` files.
fn home_dir() -> Option<std::path::PathBuf> {
    #[cfg(unix)]
    {
        std::env::var_os("HOME").map(std::path::PathBuf::from)
    }
    #[cfg(windows)]
    {
        std::env::var_os("USERPROFILE").map(std::path::PathBuf::from)
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

/// Configuration for constructing a [`LabkeyClient`].
///
/// # Example
///
/// ```no_run
/// use labkey_rs::{ClientConfig, Credential, LabkeyClient};
///
/// let api_key = std::env::var("LABKEY_API_KEY").expect("LABKEY_API_KEY must be set");
/// let config = ClientConfig::new(
///     "https://labkey.example.com/labkey",
///     Credential::ApiKey(api_key),
///     "/MyProject/MyFolder",
/// );
/// let client = LabkeyClient::new(config).expect("valid configuration");
/// ```
#[non_exhaustive]
pub struct ClientConfig {
    /// The base URL of the LabKey server (e.g., `"https://labkey.example.com/labkey"`).
    pub base_url: String,
    /// Authentication credentials.
    pub credential: Credential,
    /// Default container path (e.g., `"/MyProject/MyFolder"`).
    /// Individual requests can override this.
    pub container_path: String,
    /// Optional custom `User-Agent` header value.
    ///
    /// If not set, the client uses `labkey-rs/{version}`.
    pub user_agent: Option<String>,
    /// Whether to allow invalid/self-signed TLS certificates.
    pub accept_self_signed_certs: bool,
    /// Optional proxy URL used for all HTTP and HTTPS requests.
    pub proxy_url: Option<String>,
    /// Optional CSRF token sent as `X-LABKEY-CSRF` on every request.
    ///
    /// Java's `Connection` maintains a full CSRF lifecycle — it reads the
    /// `X-LABKEY-CSRF` cookie from every response and re-sends it as a header
    /// on subsequent requests. For API-key authentication the server typically
    /// does not require CSRF validation, so this client takes a simpler
    /// approach: supply a token here and it will be included on every request.
    /// Leave it `None` (the default) for API-key or guest auth.
    pub csrf_token: Option<String>,
}

impl ClientConfig {
    /// Create a new client configuration.
    #[must_use]
    pub fn new(
        base_url: impl Into<String>,
        credential: Credential,
        container_path: impl Into<String>,
    ) -> Self {
        Self {
            base_url: base_url.into(),
            credential,
            container_path: container_path.into(),
            user_agent: None,
            accept_self_signed_certs: false,
            proxy_url: None,
            csrf_token: None,
        }
    }

    /// Set a custom `User-Agent` header value.
    #[must_use]
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Set a proxy URL used for all requests.
    #[must_use]
    pub fn with_proxy_url(mut self, proxy_url: impl Into<String>) -> Self {
        self.proxy_url = Some(proxy_url.into());
        self
    }

    /// Enable or disable acceptance of invalid/self-signed TLS certificates.
    #[must_use]
    pub fn with_accept_self_signed_certs(mut self, accept: bool) -> Self {
        self.accept_self_signed_certs = accept;
        self
    }

    /// Set a CSRF token to send as `X-LABKEY-CSRF` on every request.
    #[must_use]
    pub fn with_csrf_token(mut self, token: impl Into<String>) -> Self {
        self.csrf_token = Some(token.into());
        self
    }
}

#[derive(Debug, Clone)]
struct HttpClientConfig {
    user_agent: String,
    accept_self_signed_certs: bool,
    proxy_url: Option<String>,
}

impl HttpClientConfig {
    fn from_client_config(config: &ClientConfig) -> Self {
        let default_user_agent = format!("labkey-rs/{}", env!("CARGO_PKG_VERSION"));
        Self {
            user_agent: config.user_agent.clone().unwrap_or(default_user_agent),
            accept_self_signed_certs: config.accept_self_signed_certs,
            proxy_url: config.proxy_url.clone(),
        }
    }
}

/// Percent-encode each segment of a container path individually.
///
/// Container names in LabKey can contain spaces and special characters.
/// We split on `/`, encode each segment, and rejoin — matching the JS
/// client's `encodePath` function in `ActionURL.ts`.
fn encode_container_path(path: &str) -> String {
    path.trim_matches('/')
        .split('/')
        .map(|segment| urlencoding::encode(segment))
        .collect::<Vec<_>>()
        .join("/")
}

/// Async client for the LabKey Server REST API.
///
/// Construct one via [`LabkeyClient::new`], then call endpoint methods like
/// [`select_rows`](Self::select_rows) or [`execute_sql`](Self::execute_sql).
pub struct LabkeyClient {
    http: reqwest::Client,
    http_config: HttpClientConfig,
    base_url: Url,
    container_path: String,
    credential: Credential,
    csrf_token: Option<String>,
}

/// Internal request options for fine-grained HTTP behavior.
#[derive(Debug, Default)]
pub(crate) struct RequestOptions {
    /// Optional per-request timeout override.
    pub timeout: Option<Duration>,
    /// Disable redirect following for this request.
    pub no_follow_redirects: bool,
    /// Additional non-success HTTP statuses that should be treated as success.
    pub accepted_statuses: Vec<StatusCode>,
}

impl LabkeyClient {
    /// Create a new client from the given configuration.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError::Url`] if `config.base_url` is not a valid URL.
    /// Returns [`LabkeyError::Http`] if HTTP client construction fails (for
    /// example, invalid proxy URL or invalid user-agent value).
    pub fn new(config: ClientConfig) -> Result<Self, LabkeyError> {
        let base_url = Url::parse(&config.base_url)?;
        let http_config = HttpClientConfig::from_client_config(&config);
        let http = Self::build_http_client(&http_config, false)?;
        Ok(Self {
            http,
            http_config,
            base_url,
            container_path: config.container_path,
            credential: config.credential,
            csrf_token: config.csrf_token,
        })
    }

    fn build_http_client(
        config: &HttpClientConfig,
        no_follow_redirects: bool,
    ) -> Result<reqwest::Client, LabkeyError> {
        let mut builder = reqwest::Client::builder()
            .user_agent(config.user_agent.clone())
            .danger_accept_invalid_certs(config.accept_self_signed_certs);

        if let Some(proxy_url) = config.proxy_url.as_deref() {
            builder = builder.proxy(reqwest::Proxy::all(proxy_url)?);
        }

        if no_follow_redirects {
            builder = builder.redirect(reqwest::redirect::Policy::none());
        }

        Ok(builder.build()?)
    }

    /// Build a LabKey action URL.
    ///
    /// LabKey URLs follow the pattern
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
    /// Sets `X-Requested-With: XMLHttpRequest` (which LabKey servers expect
    /// on API requests), the appropriate authentication credentials, and the
    /// `X-LABKEY-CSRF` header when a CSRF token is configured.
    fn prepare_request(&self, builder: reqwest::RequestBuilder) -> reqwest::RequestBuilder {
        let mut builder = builder.header("X-Requested-With", "XMLHttpRequest");
        if let Some(token) = &self.csrf_token {
            builder = builder.header("X-LABKEY-CSRF", token.as_str());
        }
        match &self.credential {
            Credential::Basic { email, password } => builder.basic_auth(email, Some(password)),
            Credential::ApiKey(key) => builder.basic_auth("apikey", Some(key)),
            Credential::Guest => builder,
        }
    }

    fn apply_request_options(
        builder: reqwest::RequestBuilder,
        options: &RequestOptions,
    ) -> reqwest::RequestBuilder {
        if let Some(timeout) = options.timeout {
            builder.timeout(timeout)
        } else {
            builder
        }
    }

    fn client_for_options(&self, options: &RequestOptions) -> Result<reqwest::Client, LabkeyError> {
        if options.no_follow_redirects {
            Self::build_http_client(&self.http_config, true)
        } else {
            Ok(self.http.clone())
        }
    }

    fn build_get_request(
        &self,
        client: &reqwest::Client,
        url: Url,
        params: &[(String, String)],
        options: &RequestOptions,
    ) -> Result<reqwest::Request, LabkeyError> {
        let builder = client.get(url).query(params);
        let builder = self.prepare_request(builder);
        let builder = Self::apply_request_options(builder, options);
        Ok(builder.build()?)
    }

    fn build_json_post_request<B: serde::Serialize>(
        &self,
        client: &reqwest::Client,
        url: Url,
        body: &B,
        options: &RequestOptions,
    ) -> Result<reqwest::Request, LabkeyError> {
        let builder = client.post(url).json(body);
        let builder = self.prepare_request(builder);
        let builder = Self::apply_request_options(builder, options);
        Ok(builder.build()?)
    }

    fn build_form_post_request(
        &self,
        client: &reqwest::Client,
        url: Url,
        params: &[(String, String)],
        options: &RequestOptions,
    ) -> Result<reqwest::Request, LabkeyError> {
        let builder = client.post(url).form(params);
        let builder = self.prepare_request(builder);
        let builder = Self::apply_request_options(builder, options);
        Ok(builder.build()?)
    }

    fn build_post_request_without_body(
        &self,
        client: &reqwest::Client,
        url: Url,
        options: &RequestOptions,
    ) -> Result<reqwest::Request, LabkeyError> {
        let builder = client.post(url).header("Content-Type", "application/json");
        let builder = self.prepare_request(builder);
        let builder = Self::apply_request_options(builder, options);
        Ok(builder.build()?)
    }

    fn build_post_request_with_params(
        &self,
        client: &reqwest::Client,
        url: Url,
        params: &[(String, String)],
        options: &RequestOptions,
    ) -> Result<reqwest::Request, LabkeyError> {
        let builder = client
            .post(url)
            .query(params)
            .header("Content-Type", "application/json");
        let builder = self.prepare_request(builder);
        let builder = Self::apply_request_options(builder, options);
        Ok(builder.build()?)
    }

    /// Send a GET request and deserialize the JSON response.
    pub(crate) async fn get<T: serde::de::DeserializeOwned>(
        &self,
        url: Url,
        params: &[(String, String)],
    ) -> Result<T, LabkeyError> {
        self.get_with_options(url, params, &RequestOptions::default())
            .await
    }

    /// Send a GET request with request options and deserialize the JSON response.
    pub(crate) async fn get_with_options<T: serde::de::DeserializeOwned>(
        &self,
        url: Url,
        params: &[(String, String)],
        options: &RequestOptions,
    ) -> Result<T, LabkeyError> {
        let client = self.client_for_options(options)?;
        let request = self.build_get_request(&client, url, params, options)?;
        let response = client.execute(request).await?;
        self.handle_response(response, &options.accepted_statuses)
            .await
    }

    /// Send a POST request with a JSON body and deserialize the JSON response.
    pub(crate) async fn post<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        url: Url,
        body: &B,
    ) -> Result<T, LabkeyError> {
        self.post_with_options(url, body, &RequestOptions::default())
            .await
    }

    /// Send a POST request with a JSON body and request options.
    pub(crate) async fn post_with_options<B: serde::Serialize, T: serde::de::DeserializeOwned>(
        &self,
        url: Url,
        body: &B,
        options: &RequestOptions,
    ) -> Result<T, LabkeyError> {
        let client = self.client_for_options(options)?;
        let request = self.build_json_post_request(&client, url, body, options)?;
        let response = client.execute(request).await?;
        self.handle_response(response, &options.accepted_statuses)
            .await
    }

    /// Send a POST request with a JSON body and request options,
    /// returning the raw text response body.
    #[cfg(feature = "experimental")]
    pub(crate) async fn post_text_with_options<B: serde::Serialize>(
        &self,
        url: Url,
        body: &B,
        options: &RequestOptions,
    ) -> Result<String, LabkeyError> {
        let client = self.client_for_options(options)?;
        let request = self.build_json_post_request(&client, url, body, options)?;
        let response = client.execute(request).await?;
        self.handle_text_response(response, &options.accepted_statuses)
            .await
    }

    /// Send a POST request with form-encoded key-value pairs and deserialize
    /// the JSON response.
    ///
    /// This mirrors the JS client's behavior when `method: 'POST'` is set on
    /// query read endpoints: the same parameters that would normally be URL
    /// query string values are sent as an `application/x-www-form-urlencoded`
    /// body instead.
    pub(crate) async fn post_form<T: serde::de::DeserializeOwned>(
        &self,
        url: Url,
        params: &[(String, String)],
    ) -> Result<T, LabkeyError> {
        let options = RequestOptions::default();
        let client = self.client_for_options(&options)?;
        let request = self.build_form_post_request(&client, url, params, &options)?;
        let response = client.execute(request).await?;
        self.handle_response(response, &options.accepted_statuses)
            .await
    }

    /// Send a multipart/form-data POST request and deserialize the JSON response.
    pub(crate) async fn post_multipart<T: serde::de::DeserializeOwned>(
        &self,
        url: Url,
        body: reqwest::multipart::Form,
        options: &RequestOptions,
    ) -> Result<T, LabkeyError> {
        let client = self.client_for_options(options)?;
        let builder = client.post(url).multipart(body);
        let builder = self.prepare_request(builder);
        let builder = Self::apply_request_options(builder, options);
        let request = builder.build()?;
        let response = client.execute(request).await?;
        self.handle_response(response, &options.accepted_statuses)
            .await
    }

    /// Send a POST request with no body.
    pub(crate) async fn post_without_body(&self, url: Url) -> Result<(), LabkeyError> {
        self.post_without_body_with_options(url, &RequestOptions::default())
            .await
    }

    /// Send a POST request with no body and request options.
    pub(crate) async fn post_without_body_with_options(
        &self,
        url: Url,
        options: &RequestOptions,
    ) -> Result<(), LabkeyError> {
        let client = self.client_for_options(options)?;
        let request = self.build_post_request_without_body(&client, url, options)?;
        let response = client.execute(request).await?;
        self.handle_empty_response(response, &options.accepted_statuses)
            .await
    }

    /// Send a POST request with query parameters and deserialize the JSON response.
    pub(crate) async fn post_with_params_with_options<T: serde::de::DeserializeOwned>(
        &self,
        url: Url,
        params: &[(String, String)],
        options: &RequestOptions,
    ) -> Result<T, LabkeyError> {
        let client = self.client_for_options(options)?;
        let request = self.build_post_request_with_params(&client, url, params, options)?;
        let response = client.execute(request).await?;
        self.handle_response(response, &options.accepted_statuses)
            .await
    }

    /// Check the response status and either deserialize the success body or
    /// construct an appropriate error.
    ///
    /// LabKey sometimes returns HTTP 200 with a JSON body containing an
    /// `"exception"` key instead of the expected response shape. Java's
    /// `Command.java` detects this and throws `CommandException`. We match
    /// that behavior: on success status codes, we read the body as text,
    /// check for an embedded exception, and return [`LabkeyError::Api`] if
    /// found. Otherwise we deserialize into `T`.
    ///
    /// On non-success status codes, the body is read as text and we attempt
    /// to parse it as [`ApiErrorBody`]. If that fails, we return
    /// [`LabkeyError::UnexpectedResponse`] with the raw text.
    async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: reqwest::Response,
        accepted_statuses: &[StatusCode],
    ) -> Result<T, LabkeyError> {
        let status = response.status();
        let text = response.text().await?;

        if status.is_success() || accepted_statuses.contains(&status) {
            if let Some(api_error) = Self::check_for_embedded_exception(&text) {
                return Err(LabkeyError::Api {
                    status,
                    body: api_error,
                });
            }
            Ok(serde_json::from_str::<T>(&text)?)
        } else {
            match serde_json::from_str::<ApiErrorBody>(&text) {
                Ok(api_error) => Err(LabkeyError::Api {
                    status,
                    body: api_error,
                }),
                Err(_) => Err(LabkeyError::UnexpectedResponse { status, text }),
            }
        }
    }

    /// Check whether a response body contains an embedded LabKey exception.
    ///
    /// LabKey sometimes returns HTTP 200 with a JSON body like
    /// `{"exception": "Something went wrong", "exceptionClass": "..."}`.
    /// This method uses a cheap string heuristic to avoid parsing the body
    /// as [`ApiErrorBody`] on every request — only when the text contains
    /// `"exception"` do we attempt the parse.
    fn check_for_embedded_exception(text: &str) -> Option<ApiErrorBody> {
        if !text.contains("\"exception\"") {
            return None;
        }
        let body: ApiErrorBody = serde_json::from_str(text).ok()?;
        if body.exception.is_some() {
            Some(body)
        } else {
            None
        }
    }

    async fn handle_empty_response(
        &self,
        response: reqwest::Response,
        accepted_statuses: &[StatusCode],
    ) -> Result<(), LabkeyError> {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();

        if status.is_success() || accepted_statuses.contains(&status) {
            if let Some(api_error) = Self::check_for_embedded_exception(&text) {
                return Err(LabkeyError::Api {
                    status,
                    body: api_error,
                });
            }
            Ok(())
        } else {
            match serde_json::from_str::<ApiErrorBody>(&text) {
                Ok(api_error) => Err(LabkeyError::Api {
                    status,
                    body: api_error,
                }),
                Err(_) => Err(LabkeyError::UnexpectedResponse { status, text }),
            }
        }
    }

    #[cfg(feature = "experimental")]
    async fn handle_text_response(
        &self,
        response: reqwest::Response,
        accepted_statuses: &[StatusCode],
    ) -> Result<String, LabkeyError> {
        let status = response.status();
        let text = response.text().await?;

        if status.is_success() || accepted_statuses.contains(&status) {
            if let Some(api_error) = Self::check_for_embedded_exception(&text) {
                return Err(LabkeyError::Api {
                    status,
                    body: api_error,
                });
            }
            Ok(text)
        } else {
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

/// Internal-only helpers used by integration tests to exercise private request
/// plumbing without expanding the default crate API surface.
#[cfg(feature = "internal-test-support")]
pub mod __internal_test_support {
    use std::time::Duration;

    use url::Url;

    use crate::error::LabkeyError;

    use super::{LabkeyClient, RequestOptions};

    /// Execute a GET request with an explicit timeout through the internal
    /// request-options path.
    ///
    /// # Errors
    ///
    /// Returns whatever error the underlying request returns, including
    /// transport, timeout, and response-decoding failures.
    pub async fn get_with_timeout<T: serde::de::DeserializeOwned>(
        client: &LabkeyClient,
        url: Url,
        params: &[(String, String)],
        timeout: Duration,
    ) -> Result<T, LabkeyError> {
        let options = RequestOptions {
            timeout: Some(timeout),
            ..RequestOptions::default()
        };
        client.get_with_options(url, params, &options).await
    }

    /// Execute a multipart POST request through the internal multipart
    /// transport path.
    ///
    /// # Errors
    ///
    /// Returns whatever error the underlying request returns, including
    /// transport and response-decoding failures.
    pub async fn post_multipart<T: serde::de::DeserializeOwned>(
        client: &LabkeyClient,
        url: Url,
        body: reqwest::multipart::Form,
    ) -> Result<T, LabkeyError> {
        client
            .post_multipart(url, body, &RequestOptions::default())
            .await
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn test_client(base_url: &str, container_path: &str) -> LabkeyClient {
        LabkeyClient::new(ClientConfig::new(
            base_url,
            Credential::ApiKey("test-key".into()),
            container_path,
        ))
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
        let result = LabkeyClient::new(ClientConfig::new(
            "not a url",
            Credential::ApiKey("key".into()),
            "/",
        ));
        assert!(matches!(result, Err(crate::error::LabkeyError::Url(_))));
    }

    #[test]
    fn client_config_struct_literal_still_constructs_in_crate() {
        let _ = ClientConfig {
            base_url: "https://labkey.example.com/labkey".into(),
            credential: Credential::ApiKey("test-key".into()),
            container_path: "/Project".into(),
            user_agent: None,
            accept_self_signed_certs: false,
            proxy_url: None,
            csrf_token: None,
        };
    }

    #[test]
    fn client_config_new_defaults_are_stable() {
        let config = ClientConfig::new(
            "https://labkey.example.com/labkey",
            Credential::ApiKey("test-key".into()),
            "/Project",
        );

        assert_eq!(config.base_url, "https://labkey.example.com/labkey");
        assert_eq!(config.container_path, "/Project");
        assert!(config.user_agent.is_none());
        assert!(!config.accept_self_signed_certs);
        assert!(config.proxy_url.is_none());
        assert!(config.csrf_token.is_none());
    }

    #[test]
    fn custom_user_agent_is_applied_to_client_configuration() {
        let config = ClientConfig::new(
            "https://labkey.example.com/labkey",
            Credential::ApiKey("test-key".into()),
            "/MyProject/MyFolder",
        )
        .with_user_agent("my-client/1.2.3");
        let client = LabkeyClient::new(config).expect("valid client config");

        assert_eq!(client.http_config.user_agent, "my-client/1.2.3");
    }

    #[test]
    fn default_user_agent_includes_crate_version() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject");

        let expected_user_agent = format!("labkey-rs/{}", env!("CARGO_PKG_VERSION"));
        assert_eq!(client.http_config.user_agent, expected_user_agent);
    }

    #[test]
    fn new_accepts_self_signed_certs_option() {
        let config = ClientConfig::new(
            "https://labkey.example.com/labkey",
            Credential::ApiKey("test-key".into()),
            "/Project",
        )
        .with_accept_self_signed_certs(true);

        let client = LabkeyClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn new_accepts_proxy_url_option() {
        let config = ClientConfig::new(
            "https://labkey.example.com/labkey",
            Credential::ApiKey("test-key".into()),
            "/Project",
        )
        .with_proxy_url("http://127.0.0.1:8080");

        let client = LabkeyClient::new(config);
        assert!(client.is_ok());
    }

    #[test]
    fn request_options_default_values() {
        let options = RequestOptions::default();
        assert!(options.timeout.is_none());
        assert!(!options.no_follow_redirects);
        assert!(options.accepted_statuses.is_empty());
    }

    #[test]
    fn build_get_request_matches_expected_url_and_headers() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");
        let url = client.build_url("query", "getQuery.api", None);
        let params = vec![("schemaName".to_string(), "lists".to_string())];
        let request = client
            .build_get_request(&client.http, url, &params, &RequestOptions::default())
            .expect("should build request");

        assert_eq!(request.method(), reqwest::Method::GET);
        assert_eq!(
            request.url().as_str(),
            "https://labkey.example.com/labkey/MyProject/MyFolder/query-getQuery.api?schemaName=lists"
        );
        assert_eq!(
            request
                .headers()
                .get("x-requested-with")
                .and_then(|v| v.to_str().ok()),
            Some("XMLHttpRequest")
        );
        assert_eq!(
            request
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok()),
            Some("Basic YXBpa2V5OnRlc3Qta2V5")
        );
    }

    #[test]
    fn build_post_request_matches_expected_url_and_headers() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");
        let url = client.build_url("query", "executeSql.api", None);
        let body = serde_json::json!({"schemaName": "core"});
        let request = client
            .build_json_post_request(&client.http, url, &body, &RequestOptions::default())
            .expect("should build request");

        assert_eq!(request.method(), reqwest::Method::POST);
        assert_eq!(
            request.url().as_str(),
            "https://labkey.example.com/labkey/MyProject/MyFolder/query-executeSql.api"
        );
        assert_eq!(
            request
                .headers()
                .get("x-requested-with")
                .and_then(|v| v.to_str().ok()),
            Some("XMLHttpRequest")
        );
        assert_eq!(
            request
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok()),
            Some("Basic YXBpa2V5OnRlc3Qta2V5")
        );
        assert!(
            request
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok())
                .is_some_and(|value| value.starts_with("application/json"))
        );
    }

    #[test]
    fn build_get_request_applies_timeout_option() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject");
        let url = client.build_url("query", "getQuery.api", None);
        let options = RequestOptions {
            timeout: Some(Duration::from_secs(5)),
            ..RequestOptions::default()
        };

        let request = client
            .build_get_request(&client.http, url, &[], &options)
            .expect("should build request");

        assert_eq!(request.timeout(), Some(&Duration::from_secs(5)));
    }

    #[test]
    fn build_post_request_with_params_matches_expected_query_and_headers() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");
        let url = client.build_url("pipeline-analysis", "startAnalysis.api", None);
        let params = vec![
            ("protocolName".to_string(), "RNAseq".to_string()),
            ("taskId".to_string(), "pipeline-123".to_string()),
        ];

        let request = client
            .build_post_request_with_params(&client.http, url, &params, &RequestOptions::default())
            .expect("should build request");

        assert_eq!(request.method(), reqwest::Method::POST);
        assert_eq!(
            request.url().as_str(),
            "https://labkey.example.com/labkey/MyProject/MyFolder/pipeline-analysis-startAnalysis.api?protocolName=RNAseq&taskId=pipeline-123"
        );
        assert_eq!(
            request
                .headers()
                .get("x-requested-with")
                .and_then(|v| v.to_str().ok()),
            Some("XMLHttpRequest")
        );
        assert_eq!(
            request
                .headers()
                .get("authorization")
                .and_then(|v| v.to_str().ok()),
            Some("Basic YXBpa2V5OnRlc3Qta2V5")
        );
        assert_eq!(
            request
                .headers()
                .get("content-type")
                .and_then(|v| v.to_str().ok()),
            Some("application/json")
        );
    }

    #[test]
    fn guest_credential_sends_no_authorization_header() {
        let client = LabkeyClient::new(ClientConfig::new(
            "https://labkey.example.com/labkey",
            Credential::Guest,
            "/MyProject",
        ))
        .expect("guest client config should be valid");

        let url = client.build_url("query", "getQuery.api", None);
        let request = client
            .build_get_request(&client.http, url, &[], &RequestOptions::default())
            .expect("should build request");

        assert!(
            request.headers().get("authorization").is_none(),
            "Guest credential should not add an Authorization header"
        );
        assert_eq!(
            request
                .headers()
                .get("x-requested-with")
                .and_then(|v| v.to_str().ok()),
            Some("XMLHttpRequest"),
            "Guest requests should still include the X-Requested-With header"
        );
    }

    #[test]
    fn from_netrc_reader_finds_matching_host() {
        let netrc_content = b"machine labkey.example.com login user@example.com password s3cret\n";
        let reader = std::io::Cursor::new(netrc_content);

        let credential = Credential::from_netrc_reader(reader, "labkey.example.com")
            .expect("should find matching host in netrc content");

        match credential {
            Credential::Basic { email, password } => {
                assert_eq!(email, "user@example.com");
                assert_eq!(password, "s3cret");
            }
            other => panic!("expected Credential::Basic, got {other:?}"),
        }
    }

    #[test]
    fn from_netrc_reader_returns_error_for_unknown_host() {
        let netrc_content = b"machine other.example.com login user password pass\n";
        let reader = std::io::Cursor::new(netrc_content);

        let result = Credential::from_netrc_reader(reader, "labkey.example.com");
        assert!(
            result.is_err(),
            "should fail when host is not in netrc content"
        );

        let err_msg = result
            .expect_err("should fail when host is not in netrc content")
            .to_string();
        assert!(
            err_msg.contains("labkey.example.com"),
            "error should mention the host that was not found: {err_msg}"
        );
    }

    #[test]
    fn from_netrc_reader_returns_error_for_missing_password() {
        let netrc_content = b"machine labkey.example.com login user@example.com\n";
        let reader = std::io::Cursor::new(netrc_content);

        let result = Credential::from_netrc_reader(reader, "labkey.example.com");

        let err_msg = result
            .expect_err("should fail when netrc entry has no password")
            .to_string();
        assert!(
            err_msg.contains("no password"),
            "error should explain the password is missing: {err_msg}"
        );
    }

    #[test]
    fn from_netrc_reader_selects_first_matching_host_from_multiple_entries() {
        let netrc_content = b"\
machine first.example.com login alice password pw1\n\
machine labkey.example.com login bob@lab.org password labpass\n\
machine third.example.com login carol password pw3\n";
        let reader = std::io::Cursor::new(netrc_content);

        let credential = Credential::from_netrc_reader(reader, "labkey.example.com")
            .expect("should find labkey.example.com among multiple entries");

        match credential {
            Credential::Basic { email, password } => {
                assert_eq!(email, "bob@lab.org");
                assert_eq!(password, "labpass");
            }
            other => panic!("expected Credential::Basic, got {other:?}"),
        }
    }

    #[test]
    fn from_netrc_reader_returns_error_for_empty_content() {
        let reader = std::io::Cursor::new(b"");

        let result = Credential::from_netrc_reader(reader, "labkey.example.com");
        assert!(result.is_err(), "should fail when netrc content is empty");
    }

    #[test]
    fn csrf_token_header_is_sent_when_configured() {
        let config = ClientConfig::new(
            "https://labkey.example.com/labkey",
            Credential::ApiKey("test-key".into()),
            "/MyProject",
        )
        .with_csrf_token("abc123");

        let client = LabkeyClient::new(config).expect("valid config with CSRF token");
        let url = client.build_url("query", "getQuery.api", None);
        let request = client
            .build_get_request(&client.http, url, &[], &RequestOptions::default())
            .expect("should build request");

        assert_eq!(
            request
                .headers()
                .get("X-LABKEY-CSRF")
                .and_then(|v| v.to_str().ok()),
            Some("abc123"),
            "CSRF token should be sent as X-LABKEY-CSRF header"
        );
    }

    #[test]
    fn csrf_token_header_is_absent_when_not_configured() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject");
        let url = client.build_url("query", "getQuery.api", None);
        let request = client
            .build_get_request(&client.http, url, &[], &RequestOptions::default())
            .expect("should build request");

        assert!(
            request.headers().get("X-LABKEY-CSRF").is_none(),
            "CSRF header should not be present when no token is configured"
        );
    }

    #[test]
    fn check_for_embedded_exception_detects_exception_in_200_body() {
        let text = r#"{"exception": "Query failed", "exceptionClass": "org.labkey.api.query.QueryParseException"}"#;
        let result = LabkeyClient::check_for_embedded_exception(text);
        assert!(result.is_some(), "should detect embedded exception");
        let body = result.expect("already checked is_some");
        assert_eq!(body.exception.as_deref(), Some("Query failed"));
        assert_eq!(
            body.exception_class.as_deref(),
            Some("org.labkey.api.query.QueryParseException")
        );
    }

    #[test]
    fn check_for_embedded_exception_returns_none_for_normal_response() {
        let text = r#"{"rows": [], "rowCount": 0}"#;
        assert!(
            LabkeyClient::check_for_embedded_exception(text).is_none(),
            "normal response without 'exception' key should not trigger detection"
        );
    }

    #[test]
    fn check_for_embedded_exception_returns_none_for_empty_body() {
        assert!(
            LabkeyClient::check_for_embedded_exception("").is_none(),
            "empty body should not trigger detection"
        );
    }

    #[test]
    fn check_for_embedded_exception_ignores_exception_as_value_not_key() {
        let text = r#"{"message": "This is not an exception", "status": "ok"}"#;
        assert!(
            LabkeyClient::check_for_embedded_exception(text).is_none(),
            "the word 'exception' appearing as a value (not a key) should not trigger detection"
        );
    }

    #[test]
    fn check_for_embedded_exception_returns_none_when_exception_is_null() {
        let text = r#"{"exception": null, "exceptionClass": null}"#;
        assert!(
            LabkeyClient::check_for_embedded_exception(text).is_none(),
            "null exception value should not be treated as an embedded error"
        );
    }

    #[test]
    fn check_for_embedded_exception_handles_exception_with_errors_array() {
        let text = r#"{
            "exception": "Validation failed",
            "exceptionClass": "org.labkey.api.action.ValidationException",
            "errors": [{"id": "Name", "msg": "Name is required"}]
        }"#;
        let result = LabkeyClient::check_for_embedded_exception(text);
        assert!(
            result.is_some(),
            "should detect exception with errors array"
        );
        let body = result.expect("already checked is_some");
        assert_eq!(body.errors.len(), 1);
        assert_eq!(body.errors[0].msg.as_deref(), Some("Name is required"));
    }
}
