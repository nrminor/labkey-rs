//! Specimen repository and request management endpoints.
//!
//! LabKey's specimen module tracks physical specimens (blood draws, tissue
//! samples, etc.) across repositories and manages specimen requests between
//! providing locations. This module provides endpoints for browsing repositories,
//! looking up vials by ID or row ID, managing specimen requests (create, cancel,
//! add/remove vials), and querying providing locations and vial type summaries.

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    client::{LabkeyClient, RequestOptions},
    error::LabkeyError,
};

/// Identifier format used by vial request mutation endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum VialIdType {
    /// Global unique vial identifier.
    #[serde(rename = "GlobalUniqueId")]
    GlobalUniqueId,
}

/// Vial identifier accepted by request mutation APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum VialId {
    /// Vial id represented as a string.
    Text(String),
    /// Vial id represented as a row id.
    RowId(i64),
}

impl VialId {
    /// Construct a string-backed vial identifier.
    #[must_use]
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text(value.into())
    }

    /// Construct a row-id-backed vial identifier.
    #[must_use]
    pub fn row_id(value: i64) -> Self {
        Self::RowId(value)
    }
}

/// Options for [`LabkeyClient::add_specimens_to_request`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct AddSpecimensToRequestOptions {
    /// Preferred location id for vial selection.
    pub preferred_location: i64,
    /// Target specimen request id.
    pub request_id: i64,
    /// Hash identifiers for primary specimens.
    pub specimen_hashes: Vec<String>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::add_vials_to_request`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct AddVialsToRequestOptions {
    /// Target specimen request id.
    pub request_id: i64,
    /// Vial identifiers.
    pub vial_ids: Vec<VialId>,
    /// Optional id type. Defaults to `GlobalUniqueId` when omitted.
    pub id_type: Option<VialIdType>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::cancel_request`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct CancelRequestOptions {
    /// Target specimen request id.
    pub request_id: i64,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::get_open_requests`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetOpenRequestsOptions {
    /// Include requests for all users.
    pub all_users: Option<bool>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::get_providing_locations`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetProvidingLocationsOptions {
    /// Hash identifiers for primary specimens.
    pub specimen_hashes: Vec<String>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::get_repositories`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetRepositoriesOptions {
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::get_request`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetRequestOptions {
    /// Target specimen request id.
    pub request_id: i64,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::get_specimen_web_part_groups`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetSpecimenWebPartGroupsOptions {
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::get_vials_by_row_id`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetVialsByRowIdOptions {
    /// Vial row ids.
    pub row_ids: Vec<i64>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::get_vial_type_summary`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetVialTypeSummaryOptions {
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::remove_vials_from_request`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct RemoveVialsFromRequestOptions {
    /// Target specimen request id.
    pub request_id: i64,
    /// Vial identifiers.
    pub vial_ids: Vec<VialId>,
    /// Optional id type. Defaults to `GlobalUniqueId` when omitted.
    pub id_type: Option<VialIdType>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AddSpecimensToRequestBody {
    preferred_location: i64,
    request_id: i64,
    #[serde(rename = "specimenHashes")]
    specimen_hashes: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct AddVialsToRequestBody {
    id_type: VialIdType,
    request_id: i64,
    #[serde(rename = "vialIds")]
    vial_ids: Vec<VialId>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CancelRequestBody {
    request_id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetOpenRequestsBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    all_users: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetProvidingLocationsBody {
    #[serde(rename = "specimenHashes")]
    specimen_hashes: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetRequestBody {
    request_id: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetVialsByRowIdBody {
    #[serde(rename = "rowIds")]
    row_ids: Vec<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RemoveVialsFromRequestBody {
    id_type: VialIdType,
    request_id: i64,
    #[serde(rename = "vialIds")]
    vial_ids: Vec<VialId>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenRequestsEnvelope {
    #[serde(default)]
    requests: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProvidingLocationsEnvelope {
    #[serde(default)]
    locations: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RepositoriesEnvelope {
    #[serde(default)]
    repositories: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestEnvelope {
    #[serde(default)]
    request: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct VialsEnvelope {
    #[serde(default)]
    vials: Option<serde_json::Value>,
}

impl LabkeyClient {
    /// Add specimens to an existing request.
    ///
    /// Sends a POST request to `specimen-api-addSpecimensToRequest.api`.
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
    /// use labkey_rs::specimen::AddSpecimensToRequestOptions;
    ///
    /// let response = client
    ///     .add_specimens_to_request(
    ///         AddSpecimensToRequestOptions::builder()
    ///             .request_id(1001)
    ///             .preferred_location(17)
    ///             .specimen_hashes(vec!["hash-1".to_string(), "hash-2".to_string()])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", response["success"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_specimens_to_request(
        &self,
        options: AddSpecimensToRequestOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "specimen-api",
            "addSpecimensToRequest.api",
            options.container_path.as_deref(),
        );
        let body = AddSpecimensToRequestBody {
            preferred_location: options.preferred_location,
            request_id: options.request_id,
            specimen_hashes: options.specimen_hashes,
        };
        self.post(url, &body).await
    }

    /// Add vials to an existing request.
    ///
    /// Sends a POST request to `specimen-api-addVialsToRequest.api`.
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
    /// use labkey_rs::specimen::{AddVialsToRequestOptions, VialId};
    ///
    /// let response = client
    ///     .add_vials_to_request(
    ///         AddVialsToRequestOptions::builder()
    ///             .request_id(1001)
    ///             .vial_ids(vec![VialId::text("VIAL-1"), VialId::text("VIAL-2")])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", response["success"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn add_vials_to_request(
        &self,
        options: AddVialsToRequestOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "specimen-api",
            "addVialsToRequest.api",
            options.container_path.as_deref(),
        );
        let body = AddVialsToRequestBody {
            id_type: id_type_or_default(options.id_type),
            request_id: options.request_id,
            vial_ids: options.vial_ids,
        };
        self.post(url, &body).await
    }

    /// Cancel an existing request.
    ///
    /// Sends a POST request to `specimen-api-cancelRequest.api`.
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
    /// use labkey_rs::specimen::CancelRequestOptions;
    ///
    /// let response = client
    ///     .cancel_request(CancelRequestOptions::builder().request_id(1001).build())
    ///     .await?;
    ///
    /// println!("{}", response["success"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn cancel_request(
        &self,
        options: CancelRequestOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "specimen-api",
            "cancelRequest.api",
            options.container_path.as_deref(),
        );
        let body = CancelRequestBody {
            request_id: options.request_id,
        };
        self.post(url, &body).await
    }

    /// Retrieve open specimen requests.
    ///
    /// Sends a POST request to `specimen-api-getOpenRequests.api` and returns
    /// the unwrapped `response.requests` payload.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, the response envelope is missing `requests`, or the body
    /// cannot be deserialized.
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
    /// use labkey_rs::specimen::GetOpenRequestsOptions;
    ///
    /// let requests = client
    ///     .get_open_requests(GetOpenRequestsOptions::builder().all_users(true).build())
    ///     .await?;
    ///
    /// println!("{}", requests);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_open_requests(
        &self,
        options: GetOpenRequestsOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "specimen-api",
            "getOpenRequests.api",
            options.container_path.as_deref(),
        );
        let body = GetOpenRequestsBody {
            all_users: options.all_users,
        };
        let response: OpenRequestsEnvelope = self.post(url, &body).await?;
        extract_open_requests(response)
    }

    /// Retrieve providing locations for specimen hashes.
    ///
    /// Sends a POST request to `specimen-api-getProvidingLocations.api` and
    /// returns the unwrapped `response.locations` payload.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, the response envelope is missing `locations`, or the
    /// body cannot be deserialized.
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
    /// use labkey_rs::specimen::GetProvidingLocationsOptions;
    ///
    /// let locations = client
    ///     .get_providing_locations(
    ///         GetProvidingLocationsOptions::builder()
    ///             .specimen_hashes(vec!["hash-1".to_string()])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", locations);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_providing_locations(
        &self,
        options: GetProvidingLocationsOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "specimen-api",
            "getProvidingLocations.api",
            options.container_path.as_deref(),
        );
        let body = GetProvidingLocationsBody {
            specimen_hashes: options.specimen_hashes,
        };
        let response: ProvidingLocationsEnvelope = self.post(url, &body).await?;
        extract_providing_locations(response)
    }

    /// Retrieve repository locations.
    ///
    /// Sends a POST request to `specimen-api-getRepositories.api` with no JSON
    /// body and returns the unwrapped `response.repositories` payload.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, the response envelope is missing `repositories`, or the
    /// body cannot be deserialized.
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
    /// use labkey_rs::specimen::GetRepositoriesOptions;
    ///
    /// let repositories = client
    ///     .get_repositories(GetRepositoriesOptions::builder().build())
    ///     .await?;
    ///
    /// println!("{}", repositories);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_repositories(
        &self,
        options: GetRepositoriesOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "specimen-api",
            "getRepositories.api",
            options.container_path.as_deref(),
        );
        let response: RepositoriesEnvelope = post_without_json_body(self, url).await?;
        extract_repositories(response)
    }

    /// Retrieve a specimen request by id.
    ///
    /// Sends a POST request to `specimen-api-getRequest.api` and returns the
    /// unwrapped `response.request` payload.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, the response envelope is missing `request`, or the body
    /// cannot be deserialized.
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
    /// use labkey_rs::specimen::GetRequestOptions;
    ///
    /// let request = client
    ///     .get_request(GetRequestOptions::builder().request_id(1001).build())
    ///     .await?;
    ///
    /// println!("{}", request["rowId"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_request(
        &self,
        options: GetRequestOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "specimen-api",
            "getRequest.api",
            options.container_path.as_deref(),
        );
        let body = GetRequestBody {
            request_id: options.request_id,
        };
        let response: RequestEnvelope = self.post(url, &body).await?;
        extract_request(response)
    }

    /// Retrieve specimen web part group metadata.
    ///
    /// Sends a POST request to `specimen-api-getSpecimenWebPartGroups.api`
    /// with no JSON body.
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
    /// use labkey_rs::specimen::GetSpecimenWebPartGroupsOptions;
    ///
    /// let response = client
    ///     .get_specimen_web_part_groups(GetSpecimenWebPartGroupsOptions::builder().build())
    ///     .await?;
    ///
    /// println!("{}", response["success"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_specimen_web_part_groups(
        &self,
        options: GetSpecimenWebPartGroupsOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "specimen-api",
            "getSpecimenWebPartGroups.api",
            options.container_path.as_deref(),
        );
        post_without_json_body(self, url).await
    }

    /// Retrieve vials by row id.
    ///
    /// Sends a POST request to `specimen-api-getVialsByRowId.api` and returns
    /// the unwrapped `response.vials` payload.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, the response envelope is missing `vials`, or the body
    /// cannot be deserialized.
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
    /// use labkey_rs::specimen::GetVialsByRowIdOptions;
    ///
    /// let vials = client
    ///     .get_vials_by_row_id(
    ///         GetVialsByRowIdOptions::builder()
    ///             .row_ids(vec![1, 2])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", vials);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_vials_by_row_id(
        &self,
        options: GetVialsByRowIdOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "specimen-api",
            "getVialsByRowId.api",
            options.container_path.as_deref(),
        );
        let body = GetVialsByRowIdBody {
            row_ids: options.row_ids,
        };
        let response: VialsEnvelope = self.post(url, &body).await?;
        extract_vials(response)
    }

    /// Retrieve vial type summary metadata.
    ///
    /// Sends a POST request to `specimen-api-getVialTypeSummary.api` with no
    /// JSON body.
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
    /// use labkey_rs::specimen::GetVialTypeSummaryOptions;
    ///
    /// let response = client
    ///     .get_vial_type_summary(GetVialTypeSummaryOptions::builder().build())
    ///     .await?;
    ///
    /// println!("{}", response["success"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_vial_type_summary(
        &self,
        options: GetVialTypeSummaryOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "specimen-api",
            "getVialTypeSummary.api",
            options.container_path.as_deref(),
        );
        post_without_json_body(self, url).await
    }

    /// Remove vials from an existing request.
    ///
    /// Sends a POST request to `specimen-api-removeVialsFromRequest` (no
    /// `.api` suffix).
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
    /// use labkey_rs::specimen::{RemoveVialsFromRequestOptions, VialId};
    ///
    /// let response = client
    ///     .remove_vials_from_request(
    ///         RemoveVialsFromRequestOptions::builder()
    ///             .request_id(1001)
    ///             .vial_ids(vec![VialId::text("VIAL-1")])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", response["success"]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn remove_vials_from_request(
        &self,
        options: RemoveVialsFromRequestOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "specimen-api",
            "removeVialsFromRequest",
            options.container_path.as_deref(),
        );
        let body = RemoveVialsFromRequestBody {
            id_type: id_type_or_default(options.id_type),
            request_id: options.request_id,
            vial_ids: options.vial_ids,
        };
        self.post(url, &body).await
    }
}

fn id_type_or_default(id_type: Option<VialIdType>) -> VialIdType {
    id_type.unwrap_or(VialIdType::GlobalUniqueId)
}

async fn post_without_json_body<T: serde::de::DeserializeOwned>(
    client: &LabkeyClient,
    url: url::Url,
) -> Result<T, LabkeyError> {
    let params: Vec<(String, String)> = Vec::new();
    client
        .post_with_params_with_options(url, &params, &RequestOptions::default())
        .await
}

fn extract_required_envelope_value(
    value: Option<serde_json::Value>,
    endpoint_name: &str,
    field_name: &str,
) -> Result<serde_json::Value, LabkeyError> {
    match value {
        Some(inner) => Ok(inner),
        None => Err(LabkeyError::UnexpectedResponse {
            status: StatusCode::OK,
            text: format!("{endpoint_name} response missing {field_name} envelope"),
        }),
    }
}

fn extract_open_requests(response: OpenRequestsEnvelope) -> Result<serde_json::Value, LabkeyError> {
    extract_required_envelope_value(response.requests, "get_open_requests", "requests")
}

fn extract_providing_locations(
    response: ProvidingLocationsEnvelope,
) -> Result<serde_json::Value, LabkeyError> {
    extract_required_envelope_value(response.locations, "get_providing_locations", "locations")
}

fn extract_repositories(response: RepositoriesEnvelope) -> Result<serde_json::Value, LabkeyError> {
    extract_required_envelope_value(response.repositories, "get_repositories", "repositories")
}

fn extract_request(response: RequestEnvelope) -> Result<serde_json::Value, LabkeyError> {
    extract_required_envelope_value(response.request, "get_request", "request")
}

fn extract_vials(response: VialsEnvelope) -> Result<serde_json::Value, LabkeyError> {
    extract_required_envelope_value(response.vials, "get_vials_by_row_id", "vials")
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

    fn vial_id_type_variant_count(value: VialIdType) -> usize {
        match value {
            VialIdType::GlobalUniqueId => 1,
        }
    }

    #[test]
    fn vial_id_types_and_identifiers_serialize_expected_wire_values() {
        assert_eq!(
            serde_json::to_string(&VialIdType::GlobalUniqueId)
                .expect("vial id type should serialize"),
            "\"GlobalUniqueId\""
        );
        assert_eq!(vial_id_type_variant_count(VialIdType::GlobalUniqueId), 1);

        assert_eq!(
            serde_json::to_value(VialId::text("VIAL-1")).expect("text vial id should serialize"),
            serde_json::json!("VIAL-1")
        );
        assert_eq!(
            serde_json::to_value(VialId::row_id(42)).expect("row-id vial id should serialize"),
            serde_json::json!(42)
        );
    }

    fn assert_specimen_url(client: &LabkeyClient, action: &str, expected_suffix: &str) {
        let actual = client
            .build_url("specimen-api", action, Some("/Alt/Container"))
            .to_string();
        assert_eq!(
            actual,
            format!("https://labkey.example.com/labkey/Alt/Container/{expected_suffix}")
        );
    }

    #[test]
    fn specimen_endpoint_urls_match_expected_actions_part_1() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_specimen_url(
            &client,
            "addSpecimensToRequest.api",
            "specimen-api-addSpecimensToRequest.api",
        );
        assert_specimen_url(
            &client,
            "addVialsToRequest.api",
            "specimen-api-addVialsToRequest.api",
        );
        assert_specimen_url(
            &client,
            "cancelRequest.api",
            "specimen-api-cancelRequest.api",
        );
        assert_specimen_url(
            &client,
            "getOpenRequests.api",
            "specimen-api-getOpenRequests.api",
        );
        assert_specimen_url(
            &client,
            "getProvidingLocations.api",
            "specimen-api-getProvidingLocations.api",
        );
        assert_specimen_url(
            &client,
            "getRepositories.api",
            "specimen-api-getRepositories.api",
        );
    }

    #[test]
    fn specimen_endpoint_urls_match_expected_actions_part_2() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_specimen_url(&client, "getRequest.api", "specimen-api-getRequest.api");
        assert_specimen_url(
            &client,
            "getSpecimenWebPartGroups.api",
            "specimen-api-getSpecimenWebPartGroups.api",
        );
        assert_specimen_url(
            &client,
            "getVialsByRowId.api",
            "specimen-api-getVialsByRowId.api",
        );
        assert_specimen_url(
            &client,
            "getVialTypeSummary.api",
            "specimen-api-getVialTypeSummary.api",
        );
        assert_specimen_url(
            &client,
            "removeVialsFromRequest",
            "specimen-api-removeVialsFromRequest",
        );
    }

    #[test]
    fn specimen_request_bodies_serialize_expected_wire_keys() {
        let add_specimens = AddSpecimensToRequestBody {
            preferred_location: 11,
            request_id: 101,
            specimen_hashes: vec!["hash-1".to_string(), "hash-2".to_string()],
        };
        let add_specimens_value =
            serde_json::to_value(add_specimens).expect("add specimens body should serialize");
        assert_eq!(
            add_specimens_value["preferredLocation"],
            serde_json::json!(11)
        );
        assert_eq!(add_specimens_value["requestId"], serde_json::json!(101));
        assert_eq!(
            add_specimens_value["specimenHashes"],
            serde_json::json!(["hash-1", "hash-2"])
        );

        let add_vials = AddVialsToRequestBody {
            id_type: id_type_or_default(None),
            request_id: 101,
            vial_ids: vec![VialId::text("vial-a"), VialId::text("vial-b")],
        };
        let add_vials_value =
            serde_json::to_value(add_vials).expect("add vials body should serialize");
        assert_eq!(
            add_vials_value["idType"],
            serde_json::json!("GlobalUniqueId")
        );
        assert_eq!(add_vials_value["requestId"], serde_json::json!(101));
        assert_eq!(
            add_vials_value["vialIds"],
            serde_json::json!(["vial-a", "vial-b"])
        );

        let by_row_id = GetVialsByRowIdBody {
            row_ids: vec![1, 2],
        };
        let by_row_id_value =
            serde_json::to_value(by_row_id).expect("get vials body should serialize");
        assert_eq!(by_row_id_value, serde_json::json!({"rowIds": [1, 2]}));
    }

    #[test]
    fn specimen_envelope_extraction_handles_happy_and_missing_paths() {
        assert_eq!(
            extract_open_requests(OpenRequestsEnvelope {
                requests: Some(serde_json::json!([{"requestId": 1}])),
            })
            .expect("requests envelope should extract"),
            serde_json::json!([{"requestId": 1}])
        );
        assert!(matches!(
            extract_open_requests(OpenRequestsEnvelope { requests: None }),
            Err(LabkeyError::UnexpectedResponse { .. })
        ));

        assert_eq!(
            extract_providing_locations(ProvidingLocationsEnvelope {
                locations: Some(serde_json::json!([{"name": "Repo A"}])),
            })
            .expect("locations envelope should extract"),
            serde_json::json!([{"name": "Repo A"}])
        );
        assert!(matches!(
            extract_providing_locations(ProvidingLocationsEnvelope { locations: None }),
            Err(LabkeyError::UnexpectedResponse { .. })
        ));

        assert_eq!(
            extract_repositories(RepositoriesEnvelope {
                repositories: Some(serde_json::json!([{"name": "Main"}])),
            })
            .expect("repositories envelope should extract"),
            serde_json::json!([{"name": "Main"}])
        );
        assert!(matches!(
            extract_repositories(RepositoriesEnvelope { repositories: None }),
            Err(LabkeyError::UnexpectedResponse { .. })
        ));

        assert_eq!(
            extract_request(RequestEnvelope {
                request: Some(serde_json::json!({"requestId": 1})),
            })
            .expect("request envelope should extract"),
            serde_json::json!({"requestId": 1})
        );
        assert!(matches!(
            extract_request(RequestEnvelope { request: None }),
            Err(LabkeyError::UnexpectedResponse { .. })
        ));

        assert_eq!(
            extract_vials(VialsEnvelope {
                vials: Some(serde_json::json!([{"rowId": 1}])),
            })
            .expect("vials envelope should extract"),
            serde_json::json!([{"rowId": 1}])
        );
        assert!(matches!(
            extract_vials(VialsEnvelope { vials: None }),
            Err(LabkeyError::UnexpectedResponse { .. })
        ));
    }

    #[tokio::test]
    async fn repositories_and_web_part_groups_post_without_json_body() {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/Alt/Specimen/specimen-api-getRepositories.api"))
            .and(header("x-requested-with", "XMLHttpRequest"))
            .and(basic_auth("apikey", "test-key"))
            .and(body_string(String::new()))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "repositories": [{"name": "Main"}]
            })))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("POST"))
            .and(path(
                "/Alt/Specimen/specimen-api-getSpecimenWebPartGroups.api",
            ))
            .and(header("x-requested-with", "XMLHttpRequest"))
            .and(basic_auth("apikey", "test-key"))
            .and(body_string(String::new()))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "success": true,
                "groups": []
            })))
            .expect(1)
            .mount(&server)
            .await;

        let client = test_client(&server.uri(), "/MyProject/MyFolder");

        let repositories = client
            .get_repositories(
                GetRepositoriesOptions::builder()
                    .container_path("/Alt/Specimen".to_string())
                    .build(),
            )
            .await
            .expect("repositories request should succeed");
        assert_eq!(repositories, serde_json::json!([{"name": "Main"}]));

        let web_part_groups = client
            .get_specimen_web_part_groups(
                GetSpecimenWebPartGroupsOptions::builder()
                    .container_path("/Alt/Specimen".to_string())
                    .build(),
            )
            .await
            .expect("specimen web part groups request should succeed");
        assert_eq!(web_part_groups["success"], serde_json::json!(true));
    }
}
