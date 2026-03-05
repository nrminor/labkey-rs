//! Report models and API endpoints.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use crate::{
    client::{LabkeyClient, RequestOptions},
    common::opt,
    error::LabkeyError,
};

/// Options for [`LabkeyClient::create_session`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct CreateSessionOptions {
    /// Client context value echoed by [`LabkeyClient::get_sessions`].
    pub client_context: serde_json::Value,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Response payload from [`LabkeyClient::create_session`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct CreateSessionResponse {
    /// Identifier for the created report session.
    pub report_session_id: String,
}

/// Options for [`LabkeyClient::delete_session`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct DeleteSessionOptions {
    /// Identifier for the report session to delete.
    pub report_session_id: String,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Output parameter returned from report execution.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OutputParam {
    /// Optional name of the output parameter.
    #[serde(default)]
    pub name: Option<String>,
    /// Output parameter type.
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
    /// Output value. For `type == "json"`, this is decoded to structured JSON.
    #[serde(default)]
    pub value: Option<serde_json::Value>,
    /// Unknown fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response payload from [`LabkeyClient::execute`] and [`LabkeyClient::execute_function`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ExecuteResponse {
    /// Information written by the script to the console.
    #[serde(default)]
    pub console: Vec<String>,
    /// Any errors reported during script execution.
    #[serde(default)]
    pub errors: Vec<String>,
    /// Output parameters produced by the script.
    #[serde(default)]
    pub output_params: Vec<OutputParam>,
}

/// Options for [`LabkeyClient::execute`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct ExecuteOptions {
    /// Identifier for the report to execute.
    pub report_id: Option<String>,
    /// Name of the report to execute.
    pub report_name: Option<String>,
    /// Schema for `report_name`-based lookup.
    pub schema_name: Option<String>,
    /// Query for `report_name`-based lookup.
    pub query_name: Option<String>,
    /// Existing report session to execute within.
    pub report_session_id: Option<String>,
    /// Input parameters to flatten as `inputParams[key]` entries.
    pub input_params: Option<BTreeMap<String, serde_json::Value>>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::execute_function`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct ExecuteFunctionOptions {
    /// Name of the function to execute.
    pub function_name: String,
    /// Existing report session to execute within.
    pub report_session_id: Option<String>,
    /// Input parameters to flatten as `inputParams[key]` entries.
    pub input_params: Option<BTreeMap<String, serde_json::Value>>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::get_sessions`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetSessionsOptions {
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Response payload from [`LabkeyClient::get_sessions`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GetSessionsResponse {
    /// Sessions previously created by this client.
    #[serde(default)]
    pub report_sessions: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateSessionBody {
    client_context: serde_json::Value,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ExecuteBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    function_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    report_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    report_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    report_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_name: Option<String>,
    #[serde(flatten)]
    input_params: BTreeMap<String, serde_json::Value>,
}

impl LabkeyClient {
    /// Create a report session that can be reused across report executions.
    ///
    /// Sends a POST request to `reports-createSession.api` with JSON body
    /// `{ clientContext }`.
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
    /// use labkey_rs::report::CreateSessionOptions;
    ///
    /// let response = client
    ///     .create_session(
    ///         CreateSessionOptions::builder()
    ///             .client_context(serde_json::json!({ "env": "rstudio" }))
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", response.report_session_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_session(
        &self,
        options: CreateSessionOptions,
    ) -> Result<CreateSessionResponse, LabkeyError> {
        let url = self.build_url(
            "reports",
            "createSession.api",
            options.container_path.as_deref(),
        );
        let body = CreateSessionBody {
            client_context: options.client_context,
        };
        self.post(url, &body).await
    }

    /// Delete a report session.
    ///
    /// Sends a POST request to `reports-deleteSession.api` with `reportSessionId`
    /// encoded as a query parameter.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the input is invalid, the request fails, the
    /// server returns an error response, or the response body cannot be deserialized.
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
    /// use labkey_rs::report::DeleteSessionOptions;
    ///
    /// client
    ///     .delete_session(
    ///         DeleteSessionOptions::builder()
    ///             .report_session_id("session-123".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_session(
        &self,
        options: DeleteSessionOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        validate_non_blank(
            "delete_session",
            "report_session_id",
            &options.report_session_id,
        )?;

        let url = self.build_url(
            "reports",
            "deleteSession.api",
            options.container_path.as_deref(),
        );
        let params = [opt(
            "reportSessionId",
            Some(options.report_session_id.as_str()),
        )]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        self.post_with_params_with_options(url, &params, &RequestOptions::default())
            .await
    }

    /// Execute a report script.
    ///
    /// Sends a POST request to `reports-execute.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the input is invalid, the request fails, the
    /// server returns an error response, or the response body cannot be deserialized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), labkey_rs::LabkeyError> {
    /// # use std::collections::BTreeMap;
    /// # let config = labkey_rs::ClientConfig::new(
    /// #     "https://labkey.example.com/labkey",
    /// #     labkey_rs::Credential::ApiKey("key".into()),
    /// #     "/",
    /// # );
    /// # let client = labkey_rs::LabkeyClient::new(config)?;
    /// use labkey_rs::report::ExecuteOptions;
    ///
    /// let mut input_params = BTreeMap::new();
    /// input_params.insert("x".to_string(), serde_json::json!(1));
    /// input_params.insert("y".to_string(), serde_json::json!(2));
    ///
    /// let response = client
    ///     .execute(
    ///         ExecuteOptions::builder()
    ///             .report_id("db:123".to_string())
    ///             .input_params(input_params)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", response.errors.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute(&self, options: ExecuteOptions) -> Result<ExecuteResponse, LabkeyError> {
        validate_execute_options(&options)?;
        let url = self.build_url("reports", "execute.api", options.container_path.as_deref());
        let body = build_execute_body_from_execute_options(options);
        let mut response: ExecuteResponse = self.post(url, &body).await?;
        decode_json_output_params(&mut response)?;
        Ok(response)
    }

    /// Execute a function within an existing report session.
    ///
    /// Sends a POST request to `reports-execute.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the input is invalid, the request fails, the
    /// server returns an error response, or the response body cannot be deserialized.
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
    /// use labkey_rs::report::ExecuteFunctionOptions;
    ///
    /// let response = client
    ///     .execute_function(
    ///         ExecuteFunctionOptions::builder()
    ///             .function_name("getSummary".to_string())
    ///             .report_session_id("session-123".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", response.console.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn execute_function(
        &self,
        options: ExecuteFunctionOptions,
    ) -> Result<ExecuteResponse, LabkeyError> {
        validate_execute_function_options(&options)?;
        let url = self.build_url("reports", "execute.api", options.container_path.as_deref());
        let body = build_execute_body_from_execute_function_options(options);
        let mut response: ExecuteResponse = self.post(url, &body).await?;
        decode_json_output_params(&mut response)?;
        Ok(response)
    }

    /// List report sessions created by this client.
    ///
    /// Sends a POST request to `reports-getSessions.api` with no JSON body.
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
    /// use labkey_rs::report::GetSessionsOptions;
    ///
    /// let sessions = client
    ///     .get_sessions(GetSessionsOptions::builder().build())
    ///     .await?;
    ///
    /// println!("{}", sessions.report_sessions.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_sessions(
        &self,
        options: GetSessionsOptions,
    ) -> Result<GetSessionsResponse, LabkeyError> {
        let url = self.build_url(
            "reports",
            "getSessions.api",
            options.container_path.as_deref(),
        );
        self.post_with_params_with_options(url, &[], &RequestOptions::default())
            .await
    }
}

fn build_execute_body_from_execute_options(options: ExecuteOptions) -> ExecuteBody {
    ExecuteBody {
        function_name: None,
        query_name: options.query_name,
        report_id: options.report_id,
        report_name: options.report_name,
        report_session_id: options.report_session_id,
        schema_name: options.schema_name,
        input_params: flatten_input_params(options.input_params.as_ref()),
    }
}

fn build_execute_body_from_execute_function_options(
    options: ExecuteFunctionOptions,
) -> ExecuteBody {
    ExecuteBody {
        function_name: Some(options.function_name),
        query_name: None,
        report_id: None,
        report_name: None,
        report_session_id: options.report_session_id,
        schema_name: None,
        input_params: flatten_input_params(options.input_params.as_ref()),
    }
}

fn flatten_input_params(
    input_params: Option<&BTreeMap<String, serde_json::Value>>,
) -> BTreeMap<String, serde_json::Value> {
    input_params
        .map(|params| {
            params
                .iter()
                .map(|(key, value)| (format!("inputParams[{key}]"), value.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn decode_json_output_params(response: &mut ExecuteResponse) -> Result<(), LabkeyError> {
    for output_param in &mut response.output_params {
        if output_param.type_.as_deref() == Some("json")
            && matches!(output_param.value, Some(serde_json::Value::String(_)))
        {
            let decoded = output_param
                .value
                .as_ref()
                .and_then(serde_json::Value::as_str)
                .map(serde_json::from_str)
                .transpose()?;
            output_param.value = decoded;
        }
    }

    Ok(())
}

fn validate_non_blank(endpoint: &str, field: &str, value: &str) -> Result<(), LabkeyError> {
    if value.trim().is_empty() {
        return Err(LabkeyError::InvalidInput(format!(
            "{endpoint} requires a non-empty {field}"
        )));
    }

    Ok(())
}

fn validate_execute_options(options: &ExecuteOptions) -> Result<(), LabkeyError> {
    let has_report_id = options
        .report_id
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    let has_report_name = options
        .report_name
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());

    if !has_report_id && !has_report_name {
        return Err(LabkeyError::InvalidInput(
            "execute requires at least one of report_id or report_name".to_string(),
        ));
    }

    Ok(())
}

fn validate_execute_function_options(options: &ExecuteFunctionOptions) -> Result<(), LabkeyError> {
    validate_non_blank("execute_function", "function_name", &options.function_name)
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
    fn report_endpoint_urls_match_expected_actions() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url("reports", "createSession.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/reports-createSession.api"
        );
        assert_eq!(
            client
                .build_url("reports", "deleteSession.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/reports-deleteSession.api"
        );
        assert_eq!(
            client
                .build_url("reports", "execute.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/reports-execute.api"
        );
        assert_eq!(
            client
                .build_url("reports", "getSessions.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/reports-getSessions.api"
        );
    }

    #[test]
    fn execute_body_flattens_input_params_using_bracket_notation() {
        let mut input_params = BTreeMap::new();
        input_params.insert("x".to_string(), serde_json::json!(1));
        input_params.insert("y".to_string(), serde_json::json!("foo"));

        let body = build_execute_body_from_execute_options(
            ExecuteOptions::builder()
                .report_id("db:123".to_string())
                .input_params(input_params)
                .build(),
        );

        let body_json = serde_json::to_value(body).expect("execute body should serialize");
        assert_eq!(body_json["reportId"], serde_json::json!("db:123"));
        assert_eq!(body_json["inputParams[x]"], serde_json::json!(1));
        assert_eq!(body_json["inputParams[y]"], serde_json::json!("foo"));
    }

    #[test]
    fn execute_function_body_flattens_input_params_using_bracket_notation() {
        let mut input_params = BTreeMap::new();
        input_params.insert("alpha".to_string(), serde_json::json!(true));

        let body = build_execute_body_from_execute_function_options(
            ExecuteFunctionOptions::builder()
                .function_name("getSummary".to_string())
                .input_params(input_params)
                .build(),
        );

        let body_json = serde_json::to_value(body).expect("execute body should serialize");
        assert_eq!(body_json["functionName"], serde_json::json!("getSummary"));
        assert_eq!(body_json["inputParams[alpha]"], serde_json::json!(true));
        assert!(body_json.get("reportId").is_none());
    }

    #[test]
    fn decode_json_output_params_decodes_json_typed_values() {
        let mut response: ExecuteResponse = serde_json::from_value(serde_json::json!({
            "console": ["ok"],
            "errors": [],
            "outputParams": [
                {"name": "jsonout", "type": "json", "value": "{\"a\":1}"},
                {"name": "textout", "type": "text", "value": "plain"}
            ]
        }))
        .expect("response should deserialize");

        decode_json_output_params(&mut response).expect("output params should decode");

        assert_eq!(response.output_params[0].name.as_deref(), Some("jsonout"));
        assert_eq!(
            response.output_params[0].value,
            Some(serde_json::json!({"a": 1}))
        );
        assert_eq!(response.output_params[1].name.as_deref(), Some("textout"));
        assert_eq!(
            response.output_params[1].value,
            Some(serde_json::json!("plain"))
        );
    }

    #[test]
    fn decode_json_output_params_errors_on_invalid_json_string() {
        let mut response: ExecuteResponse = serde_json::from_value(serde_json::json!({
            "outputParams": [
                {"type": "json", "value": "{not-json"}
            ]
        }))
        .expect("response should deserialize");

        let error = decode_json_output_params(&mut response).expect_err("decode should fail");
        assert!(matches!(error, LabkeyError::Deserialization(_)));
    }

    #[test]
    fn delete_session_uses_query_param_shape() {
        let params = [opt("reportSessionId", Some("session-123"))]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        assert_eq!(
            params,
            vec![("reportSessionId".to_string(), "session-123".to_string())]
        );
    }

    #[test]
    fn response_models_deserialize_happy_and_minimal_shapes() {
        let create_session: CreateSessionResponse = serde_json::from_value(serde_json::json!({
            "reportSessionId": "session-1"
        }))
        .expect("create_session response should deserialize");
        assert_eq!(create_session.report_session_id, "session-1");

        let execute_response: ExecuteResponse = serde_json::from_value(serde_json::json!({}))
            .expect("execute response should deserialize");
        assert!(execute_response.console.is_empty());
        assert!(execute_response.errors.is_empty());
        assert!(execute_response.output_params.is_empty());

        let get_sessions_response: GetSessionsResponse =
            serde_json::from_value(serde_json::json!({}))
                .expect("get_sessions response should deserialize");
        assert!(get_sessions_response.report_sessions.is_empty());
    }

    #[test]
    fn execute_validation_requires_report_id_or_report_name() {
        let options = ExecuteOptions::builder().build();

        let error = validate_execute_options(&options).expect_err("validation should fail");
        assert!(matches!(error, LabkeyError::InvalidInput(_)));
    }

    #[test]
    fn execute_function_validation_requires_non_blank_function_name() {
        let options = ExecuteFunctionOptions::builder()
            .function_name("\n\t".to_string())
            .build();

        let error =
            validate_execute_function_options(&options).expect_err("validation should fail");
        assert!(matches!(error, LabkeyError::InvalidInput(_)));
    }

    #[test]
    fn delete_session_validation_requires_non_blank_report_session_id() {
        let error = validate_non_blank("delete_session", "report_session_id", "\n\t")
            .expect_err("validation should fail");
        assert!(matches!(error, LabkeyError::InvalidInput(_)));
    }
}
