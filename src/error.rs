//! Error types for the `LabKey` client.
//!
//! The central type is [`LabkeyError`], which covers everything from network
//! failures to structured API errors returned by the server. When the server
//! returns a non-success status code, the client reads the response body and
//! attempts to parse it as an [`ApiErrorBody`] before falling back to
//! [`LabkeyError::UnexpectedResponse`].

use thiserror::Error;

/// Individual field-level error returned by the `LabKey` server.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct FieldError {
    /// The field this error relates to, if any.
    #[serde(default)]
    pub id: Option<String>,
    /// The error message.
    #[serde(default)]
    pub msg: Option<String>,
}

/// Structured error body returned by `LabKey` API endpoints.
///
/// A typical server error response looks like:
///
/// ```json
/// {
///   "exception": "Query 'nonexistent' in schema 'core' doesn't exist.",
///   "exceptionClass": "org.labkey.api.query.QueryParseException",
///   "errors": [{ "id": "some_field", "msg": "Detailed error message" }]
/// }
/// ```
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApiErrorBody {
    /// Human-readable error message.
    #[serde(default)]
    pub exception: Option<String>,
    /// Java exception class name from the server.
    #[serde(rename = "exceptionClass", default)]
    pub exception_class: Option<String>,
    /// Per-field errors, if any.
    #[serde(default)]
    pub errors: Vec<FieldError>,
}

impl std::fmt::Display for ApiErrorBody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match (&self.exception, &self.exception_class) {
            (Some(msg), Some(class)) => write!(f, "{msg} [{class}]"),
            (Some(msg), None) => write!(f, "{msg}"),
            (None, Some(class)) => write!(f, "(no message) [{class}]"),
            (None, None) => write!(f, "(no message)"),
        }
    }
}

/// Errors that can occur when using the `LabKey` client.
#[derive(Debug, Error)]
pub enum LabkeyError {
    /// HTTP-level error from reqwest (connection failures, timeouts, etc.).
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// Invalid client-side input detected before making a request.
    #[error("invalid input: {0}")]
    InvalidInput(String),

    /// The server returned a non-success status code with a structured error body.
    #[error("LabKey API error (HTTP {status}): {body}")]
    Api {
        /// The HTTP status code.
        status: reqwest::StatusCode,
        /// The parsed error body.
        body: ApiErrorBody,
    },

    /// The server returned a non-success status code but the body wasn't
    /// parseable as a `LabKey` error.
    #[error("HTTP {status}: {text}")]
    UnexpectedResponse {
        /// The HTTP status code.
        status: reqwest::StatusCode,
        /// The raw response body text.
        text: String,
    },

    /// JSON deserialization failed on an otherwise successful response.
    #[error("failed to deserialize response: {0}")]
    Deserialization(#[from] serde_json::Error),

    /// URL construction failed.
    #[error("invalid URL: {0}")]
    Url(#[from] url::ParseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_error_body_display_with_message_and_class() {
        let body = ApiErrorBody {
            exception: Some("Something went wrong".into()),
            exception_class: Some("java.lang.RuntimeException".into()),
            errors: vec![],
        };
        assert_eq!(
            body.to_string(),
            "Something went wrong [java.lang.RuntimeException]"
        );
    }

    #[test]
    fn api_error_body_display_with_message_only() {
        let body = ApiErrorBody {
            exception: Some("Something went wrong".into()),
            exception_class: None,
            errors: vec![],
        };
        assert_eq!(body.to_string(), "Something went wrong");
    }

    #[test]
    fn api_error_body_display_without_message() {
        let body = ApiErrorBody {
            exception: None,
            exception_class: None,
            errors: vec![],
        };
        assert_eq!(body.to_string(), "(no message)");
    }

    #[test]
    fn api_error_body_deserializes_from_json() {
        let json = r#"{
            "exception": "Query 'nonexistent' in schema 'core' doesn't exist.",
            "exceptionClass": "org.labkey.api.query.QueryParseException",
            "errors": [{ "id": "some_field", "msg": "Detailed error message" }]
        }"#;
        let body: ApiErrorBody = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(
            body.exception.as_deref(),
            Some("Query 'nonexistent' in schema 'core' doesn't exist.")
        );
        assert_eq!(
            body.exception_class.as_deref(),
            Some("org.labkey.api.query.QueryParseException")
        );
        assert_eq!(body.errors.len(), 1);
        assert_eq!(body.errors[0].id.as_deref(), Some("some_field"));
        assert_eq!(
            body.errors[0].msg.as_deref(),
            Some("Detailed error message")
        );
    }

    #[test]
    fn api_error_body_deserializes_with_missing_fields() {
        let json = r#"{ "exception": "Oops" }"#;
        let body: ApiErrorBody = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(body.exception.as_deref(), Some("Oops"));
        assert!(body.exception_class.is_none());
        assert!(body.errors.is_empty());
    }

    #[test]
    fn labkey_error_display_api_variant() {
        let err = LabkeyError::Api {
            status: reqwest::StatusCode::NOT_FOUND,
            body: ApiErrorBody {
                exception: Some("Not found".into()),
                exception_class: Some("org.labkey.api.view.NotFoundException".into()),
                errors: vec![],
            },
        };
        assert_eq!(
            err.to_string(),
            "LabKey API error (HTTP 404 Not Found): Not found [org.labkey.api.view.NotFoundException]"
        );
    }

    #[test]
    fn labkey_error_from_url_parse_error() {
        let parse_err = url::Url::parse("not a url").expect_err("should fail to parse");
        let err = LabkeyError::from(parse_err);
        assert!(matches!(err, LabkeyError::Url(_)));
    }

    #[test]
    fn labkey_error_from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json")
            .expect_err("should fail to parse");
        let err = LabkeyError::from(json_err);
        assert!(matches!(err, LabkeyError::Deserialization(_)));
    }

    #[test]
    fn labkey_error_display_unexpected_response() {
        let err = LabkeyError::UnexpectedResponse {
            status: reqwest::StatusCode::INTERNAL_SERVER_ERROR,
            text: "<html>Server Error</html>".into(),
        };
        assert_eq!(
            err.to_string(),
            "HTTP 500 Internal Server Error: <html>Server Error</html>"
        );
    }
}
