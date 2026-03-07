//! Freezer storage management endpoints for physical sample locations.
//!
//! LabKey's storage module tracks where physical samples are stored — freezers,
//! shelves, boxes, and individual positions. This module provides endpoints for
//! creating, updating, and deleting storage items of various types (see
//! [`StorageType`]).

use serde::{Deserialize, Serialize};

use crate::{client::LabkeyClient, error::LabkeyError};

/// Storage item types accepted by storage commands.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum StorageType {
    /// Canister location.
    #[serde(rename = "Canister")]
    Canister,
    /// Freezer location.
    #[serde(rename = "Freezer")]
    Freezer,
    /// Physical location.
    #[serde(rename = "Physical Location")]
    PhysicalLocation,
    /// Primary storage location.
    #[serde(rename = "Primary Storage")]
    PrimaryStorage,
    /// Rack location.
    #[serde(rename = "Rack")]
    Rack,
    /// Shelf location.
    #[serde(rename = "Shelf")]
    Shelf,
    /// Storage unit type.
    #[serde(rename = "Storage Unit Type")]
    StorageUnitType,
    /// Terminal storage location.
    #[serde(rename = "Terminal Storage Location")]
    TerminalStorageLocation,
}

/// Response payload returned by storage commands.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct StorageCommandResponse {
    /// Returned storage item data.
    #[serde(default)]
    pub data: Option<serde_json::Value>,
    /// Status message from the server.
    #[serde(default)]
    pub message: Option<String>,
    /// Indicates whether the command succeeded.
    #[serde(default)]
    pub success: bool,
}

/// Options for [`LabkeyClient::create_storage_item`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct CreateStorageItemOptions {
    /// Properties for the item being created.
    pub props: serde_json::Value,
    /// Storage item type.
    pub storage_type: StorageType,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::update_storage_item`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct UpdateStorageItemOptions {
    /// Updated properties for the target item.
    pub props: serde_json::Value,
    /// Storage item type.
    pub storage_type: StorageType,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::delete_storage_item`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct DeleteStorageItemOptions {
    /// Row id of the item to delete.
    pub row_id: i64,
    /// Storage item type.
    pub storage_type: StorageType,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

#[derive(Debug, Serialize)]
struct StorageCommandBody {
    #[serde(rename = "type")]
    storage_type: StorageType,
    props: serde_json::Value,
}

#[derive(Debug, Serialize)]
struct DeleteStorageCommandBody {
    #[serde(rename = "type")]
    storage_type: StorageType,
    props: DeleteStorageProps,
}

#[derive(Debug, Serialize)]
struct DeleteStorageProps {
    #[serde(rename = "rowId")]
    row_id: i64,
}

impl LabkeyClient {
    /// Create a storage item.
    ///
    /// Sends a POST request to `storage-create.api`.
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
    /// use labkey_rs::storage::{CreateStorageItemOptions, StorageType};
    ///
    /// let response = client
    ///     .create_storage_item(
    ///         CreateStorageItemOptions::builder()
    ///             .storage_type(StorageType::Freezer)
    ///             .props(serde_json::json!({ "name": "Freezer A" }))
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", response.success);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_storage_item(
        &self,
        options: CreateStorageItemOptions,
    ) -> Result<StorageCommandResponse, LabkeyError> {
        let url = self.build_url("storage", "create.api", options.container_path.as_deref());
        let body = StorageCommandBody {
            storage_type: options.storage_type,
            props: options.props,
        };
        self.post(url, &body).await
    }

    /// Update a storage item.
    ///
    /// Sends a POST request to `storage-update.api`.
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
    /// use labkey_rs::storage::{StorageType, UpdateStorageItemOptions};
    ///
    /// let response = client
    ///     .update_storage_item(
    ///         UpdateStorageItemOptions::builder()
    ///             .storage_type(StorageType::Freezer)
    ///             .props(serde_json::json!({ "rowId": 100, "description": "Updated" }))
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", response.success);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_storage_item(
        &self,
        options: UpdateStorageItemOptions,
    ) -> Result<StorageCommandResponse, LabkeyError> {
        let url = self.build_url("storage", "update.api", options.container_path.as_deref());
        let body = StorageCommandBody {
            storage_type: options.storage_type,
            props: options.props,
        };
        self.post(url, &body).await
    }

    /// Delete a storage item.
    ///
    /// Sends a POST request to `storage-delete.api` with body
    /// `{ type, props: { rowId } }`.
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
    /// use labkey_rs::storage::{DeleteStorageItemOptions, StorageType};
    ///
    /// let response = client
    ///     .delete_storage_item(
    ///         DeleteStorageItemOptions::builder()
    ///             .storage_type(StorageType::Freezer)
    ///             .row_id(100)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", response.success);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_storage_item(
        &self,
        options: DeleteStorageItemOptions,
    ) -> Result<StorageCommandResponse, LabkeyError> {
        let url = self.build_url("storage", "delete.api", options.container_path.as_deref());
        let body = DeleteStorageCommandBody {
            storage_type: options.storage_type,
            props: DeleteStorageProps {
                row_id: options.row_id,
            },
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

    fn storage_type_variant_count(value: StorageType) -> usize {
        match value {
            StorageType::Canister
            | StorageType::Freezer
            | StorageType::PhysicalLocation
            | StorageType::PrimaryStorage
            | StorageType::Rack
            | StorageType::Shelf
            | StorageType::StorageUnitType
            | StorageType::TerminalStorageLocation => 8,
        }
    }

    #[test]
    fn storage_endpoint_urls_match_expected_actions() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url("storage", "create.api", Some("/Alt/Storage"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Storage/storage-create.api"
        );
        assert_eq!(
            client
                .build_url("storage", "update.api", Some("/Alt/Storage"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Storage/storage-update.api"
        );
        assert_eq!(
            client
                .build_url("storage", "delete.api", Some("/Alt/Storage"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Storage/storage-delete.api"
        );
    }

    #[test]
    fn delete_storage_body_has_required_row_id_shape() {
        let body = StorageCommandBody {
            storage_type: StorageType::Freezer,
            props: serde_json::to_value(DeleteStorageProps { row_id: 77 })
                .expect("props should serialize"),
        };
        let json = serde_json::to_value(body).expect("body should serialize");

        assert_eq!(json["type"], serde_json::json!("Freezer"));
        assert_eq!(json["props"]["rowId"], serde_json::json!(77));
    }

    #[test]
    fn create_storage_body_serializes_type_and_props() {
        let body = StorageCommandBody {
            storage_type: StorageType::PrimaryStorage,
            props: serde_json::json!({ "name": "Main Storage", "temperature": -20 }),
        };
        let json = serde_json::to_value(body).expect("body should serialize");

        let obj = json.as_object().expect("top level must be an object");
        assert_eq!(
            obj.len(),
            2,
            "body should have exactly two top-level keys: type and props"
        );
        assert_eq!(json["type"], serde_json::json!("Primary Storage"));
        assert_eq!(json["props"]["name"], serde_json::json!("Main Storage"));
        assert_eq!(json["props"]["temperature"], serde_json::json!(-20));
    }

    #[test]
    fn update_storage_body_serializes_type_and_props_with_row_id() {
        let body = StorageCommandBody {
            storage_type: StorageType::TerminalStorageLocation,
            props: serde_json::json!({ "rowId": 5, "label": "Box A" }),
        };
        let json = serde_json::to_value(body).expect("body should serialize");

        let obj = json.as_object().expect("top level must be an object");
        assert_eq!(
            obj.len(),
            2,
            "body should have exactly two top-level keys: type and props"
        );
        assert_eq!(json["type"], serde_json::json!("Terminal Storage Location"));
        assert_eq!(json["props"]["rowId"], serde_json::json!(5));
        assert_eq!(json["props"]["label"], serde_json::json!("Box A"));
    }

    #[test]
    fn storage_type_round_trip_and_variant_count_regression() {
        let pairs = [
            (StorageType::Canister, "\"Canister\""),
            (StorageType::Freezer, "\"Freezer\""),
            (StorageType::PhysicalLocation, "\"Physical Location\""),
            (StorageType::PrimaryStorage, "\"Primary Storage\""),
            (StorageType::Rack, "\"Rack\""),
            (StorageType::Shelf, "\"Shelf\""),
            (StorageType::StorageUnitType, "\"Storage Unit Type\""),
            (
                StorageType::TerminalStorageLocation,
                "\"Terminal Storage Location\"",
            ),
        ];

        for (variant, expected_wire) in pairs {
            assert_eq!(
                serde_json::to_string(&variant).expect("serialize storage type"),
                expected_wire
            );

            let decoded: StorageType =
                serde_json::from_str(expected_wire).expect("deserialize storage type");
            assert_eq!(variant, decoded);
        }

        assert_eq!(storage_type_variant_count(StorageType::Canister), 8);
    }

    #[test]
    fn storage_response_deserializes_happy_and_minimal_shapes() {
        let happy: StorageCommandResponse = serde_json::from_value(serde_json::json!({
            "success": true,
            "message": "saved",
            "data": { "rowId": 1 }
        }))
        .expect("happy response should deserialize");
        assert!(happy.success);
        assert_eq!(happy.message.as_deref(), Some("saved"));
        assert_eq!(happy.data.expect("data")["rowId"], serde_json::json!(1));

        let minimal: StorageCommandResponse = serde_json::from_value(serde_json::json!({}))
            .expect("minimal response should deserialize");
        assert!(!minimal.success);
        assert_eq!(minimal.message, None);
        assert_eq!(minimal.data, None);
    }
}
