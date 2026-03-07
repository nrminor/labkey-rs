//! Message board API for sending messages within a LabKey container.
//!
//! LabKey containers can have message boards for threaded discussions. This
//! module provides [`SendMessageOptions`] for posting messages with HTML or
//! plain text content to specific recipients (users or participant groups).

use serde::{Deserialize, Serialize};

use crate::{client::LabkeyClient, error::LabkeyError};

/// Content type for message bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ContentType {
    /// Plain text content.
    #[serde(rename = "text/plain")]
    TextPlain,
    /// HTML content.
    #[serde(rename = "text/html")]
    TextHtml,
}

/// Recipient delivery type for email messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RecipientType {
    /// Blind carbon copy recipient.
    #[serde(rename = "BCC")]
    Bcc,
    /// Carbon copy recipient.
    #[serde(rename = "CC")]
    Cc,
    /// Primary recipient.
    #[serde(rename = "TO")]
    To,
}

/// One message body part in [`SendMessageOptions::msg_content`].
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct MsgContent {
    /// Message body for this content part.
    pub content: String,
    /// Content type for this body part.
    #[serde(rename = "type")]
    pub type_: ContentType,
}

impl MsgContent {
    /// Create a message content part.
    #[must_use]
    pub fn new(content: impl Into<String>, type_: ContentType) -> Self {
        Self {
            content: content.into(),
            type_,
        }
    }
}

/// A message recipient represented by address or principal id.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum Recipient {
    /// Recipient represented by an email address.
    #[non_exhaustive]
    Address {
        /// Recipient delivery type.
        #[serde(rename = "type")]
        type_: RecipientType,
        /// Email address for this recipient.
        address: String,
    },
    /// Recipient represented by a principal id.
    #[non_exhaustive]
    PrincipalId {
        /// Recipient delivery type.
        #[serde(rename = "type")]
        type_: RecipientType,
        /// User or group principal id.
        #[serde(rename = "principalId")]
        principal_id: i64,
    },
}

impl Recipient {
    /// Construct an address-based recipient.
    #[must_use]
    pub fn address(type_: RecipientType, address: impl Into<String>) -> Self {
        Self::Address {
            type_,
            address: address.into(),
        }
    }

    /// Construct a principal-id-based recipient.
    #[must_use]
    pub fn principal_id(type_: RecipientType, principal_id: i64) -> Self {
        Self::PrincipalId {
            type_,
            principal_id,
        }
    }
}

/// Options for [`LabkeyClient::send_message`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SendMessageOptions {
    /// Message content parts.
    pub msg_content: Option<Vec<MsgContent>>,
    /// Sender email address.
    pub msg_from: Option<String>,
    /// Message recipients.
    pub msg_recipients: Option<Vec<Recipient>>,
    /// Message subject.
    pub msg_subject: Option<String>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Response payload from [`LabkeyClient::send_message`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SendMessageResponse {
    /// Indicates whether the operation succeeded.
    #[serde(default)]
    pub success: Option<bool>,
    /// Optional status message from the server.
    #[serde(default)]
    pub message: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SendMessageBody {
    #[serde(rename = "msgContent")]
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<Vec<MsgContent>>,
    #[serde(rename = "msgFrom")]
    #[serde(skip_serializing_if = "Option::is_none")]
    from: Option<String>,
    #[serde(rename = "msgRecipients")]
    #[serde(skip_serializing_if = "Option::is_none")]
    recipients: Option<Vec<Recipient>>,
    #[serde(rename = "msgSubject")]
    #[serde(skip_serializing_if = "Option::is_none")]
    subject: Option<String>,
}

impl LabkeyClient {
    /// Send a message through the server's announcement endpoint.
    ///
    /// Sends a POST request to `announcements-sendMessage.api`.
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
    /// use labkey_rs::message::{ContentType, MsgContent, Recipient, RecipientType, SendMessageOptions};
    ///
    /// let response = client
    ///     .send_message(
    ///         SendMessageOptions::builder()
    ///             .msg_subject("Status update".to_string())
    ///             .msg_content(vec![MsgContent::new("Pipeline completed", ContentType::TextPlain)])
    ///             .msg_recipients(vec![Recipient::address(RecipientType::To, "team@example.com")])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{:?}", response.success);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn send_message(
        &self,
        options: SendMessageOptions,
    ) -> Result<SendMessageResponse, LabkeyError> {
        let url = self.build_url(
            "announcements",
            "sendMessage.api",
            options.container_path.as_deref(),
        );
        let body = SendMessageBody {
            content: options.msg_content,
            from: options.msg_from,
            recipients: options.msg_recipients,
            subject: options.msg_subject,
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

    fn recipient_type_variant_count(value: RecipientType) -> usize {
        match value {
            RecipientType::Bcc | RecipientType::Cc | RecipientType::To => 3,
        }
    }

    fn content_type_variant_count(value: ContentType) -> usize {
        match value {
            ContentType::TextHtml | ContentType::TextPlain => 2,
        }
    }

    #[test]
    fn message_endpoint_url_matches_expected_action() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url("announcements", "sendMessage.api", Some("/Alt/Container"),)
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/announcements-sendMessage.api"
        );
    }

    #[test]
    fn send_message_body_serializes_expected_keys() {
        let body = SendMessageBody {
            from: Some("sender@example.com".to_string()),
            subject: Some("Subject".to_string()),
            content: Some(vec![MsgContent {
                content: "Hello".to_string(),
                type_: ContentType::TextPlain,
            }]),
            recipients: Some(vec![Recipient::Address {
                type_: RecipientType::To,
                address: "recipient@example.com".to_string(),
            }]),
        };

        let json = serde_json::to_value(body).expect("body should serialize");
        assert_eq!(json["msgFrom"], serde_json::json!("sender@example.com"));
        assert_eq!(json["msgSubject"], serde_json::json!("Subject"));
        assert_eq!(
            json["msgContent"][0]["type"],
            serde_json::json!("text/plain")
        );
        assert_eq!(json["msgRecipients"][0]["type"], serde_json::json!("TO"));
    }

    #[test]
    fn message_enums_round_trip_and_counts_remain_stable() {
        assert_eq!(
            serde_json::to_string(&RecipientType::Bcc).expect("serialize recipient type"),
            "\"BCC\""
        );
        assert_eq!(
            serde_json::to_string(&RecipientType::Cc).expect("serialize recipient type"),
            "\"CC\""
        );
        assert_eq!(
            serde_json::to_string(&RecipientType::To).expect("serialize recipient type"),
            "\"TO\""
        );
        assert_eq!(
            serde_json::to_string(&ContentType::TextPlain).expect("serialize content type"),
            "\"text/plain\""
        );
        assert_eq!(
            serde_json::to_string(&ContentType::TextHtml).expect("serialize content type"),
            "\"text/html\""
        );
        assert_eq!(recipient_type_variant_count(RecipientType::Bcc), 3);
        assert_eq!(content_type_variant_count(ContentType::TextHtml), 2);
    }

    #[test]
    fn recipient_principal_id_serializes_with_type_and_principal_id_keys() {
        let recipient = Recipient::principal_id(RecipientType::Cc, 42);
        let json = serde_json::to_value(&recipient).expect("should serialize");

        assert_eq!(json["type"], serde_json::json!("CC"));
        assert_eq!(json["principalId"], serde_json::json!(42));
        assert!(
            json.get("address").is_none(),
            "PrincipalId variant should not include an address key"
        );

        let round_tripped: Recipient =
            serde_json::from_value(json).expect("should deserialize back");
        match round_tripped {
            Recipient::PrincipalId {
                type_,
                principal_id,
            } => {
                assert_eq!(type_, RecipientType::Cc);
                assert_eq!(principal_id, 42);
            }
            other => panic!("expected PrincipalId variant, got {other:?}"),
        }
    }

    #[test]
    fn send_message_response_deserializes_happy_and_minimal_shapes() {
        let happy: SendMessageResponse = serde_json::from_value(serde_json::json!({
            "success": true,
            "message": "sent"
        }))
        .expect("happy response should deserialize");
        assert_eq!(happy.success, Some(true));
        assert_eq!(happy.message.as_deref(), Some("sent"));

        let minimal: SendMessageResponse = serde_json::from_value(serde_json::json!({}))
            .expect("minimal response should deserialize");
        assert_eq!(minimal.success, None);
        assert_eq!(minimal.message, None);
    }
}
