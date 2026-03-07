//! Visualization models and API endpoints.

use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize, de::Error as DeError};
use url::Url;

use crate::{client::LabkeyClient, common::opt, error::LabkeyError, filter::Filter};

/// Icon persistence behavior for [`LabkeyClient::save_visualization`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum IconType {
    /// Auto-generate an icon/thumbnail.
    #[serde(rename = "AUTO")]
    Auto,
    /// Keep an existing custom icon/thumbnail.
    #[serde(rename = "CUSTOM")]
    Custom,
    /// Do not persist an icon/thumbnail.
    #[serde(rename = "NONE")]
    None,
}

/// A plottable measure returned by [`LabkeyClient::get_measures`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Measure {
    /// Optional server-provided description.
    #[serde(default)]
    pub description: Option<String>,
    /// Optional user-facing label.
    #[serde(default)]
    pub label: Option<String>,
    /// Underlying column name.
    #[serde(default)]
    pub name: Option<String>,
    /// Query name that owns the measure.
    #[serde(default)]
    pub query_name: Option<String>,
    /// Schema name that owns the query.
    #[serde(default)]
    pub schema_name: Option<String>,
    /// Data type reported by the server.
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
    /// Whether this measure comes from a user-defined query.
    #[serde(default, rename = "isUserDefined")]
    pub is_user_defined: Option<bool>,
    /// Unknown fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// A dimension returned by [`LabkeyClient::get_dimensions`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Dimension {
    /// Optional server-provided description.
    #[serde(default)]
    pub description: Option<String>,
    /// Optional user-facing label.
    #[serde(default)]
    pub label: Option<String>,
    /// Underlying column name.
    #[serde(default)]
    pub name: Option<String>,
    /// Query name that owns the dimension.
    #[serde(default)]
    pub query_name: Option<String>,
    /// Schema name that owns the query.
    #[serde(default)]
    pub schema_name: Option<String>,
    /// Data type reported by the server.
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
    /// Whether this dimension comes from a user-defined query.
    #[serde(default, rename = "isUserDefined")]
    pub is_user_defined: Option<bool>,
    /// Unknown fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response payload from [`LabkeyClient::get_visualization`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct VisualizationResponse {
    /// Saved visualization identifier.
    #[serde(default)]
    pub id: Option<String>,
    /// Saved visualization name.
    #[serde(default)]
    pub name: Option<String>,
    /// Visualization type key.
    #[serde(default, rename = "type")]
    pub type_: Option<String>,
    /// Decoded visualization configuration.
    #[serde(
        default,
        rename = "visualizationConfig",
        deserialize_with = "deserialize_visualization_config"
    )]
    pub visualization_config: serde_json::Value,
    /// Whether the current user can delete this visualization.
    #[serde(default, rename = "canDelete")]
    pub can_delete: Option<bool>,
    /// Whether the current user can edit this visualization.
    #[serde(default, rename = "canEdit")]
    pub can_edit: Option<bool>,
    /// Whether the current user can share this visualization.
    #[serde(default, rename = "canShare")]
    pub can_share: Option<bool>,
    /// User ID of the visualization creator.
    #[serde(default, rename = "createdBy")]
    pub created_by: Option<i64>,
    /// Optional description of the visualization.
    #[serde(default)]
    pub description: Option<String>,
    /// Whether this visualization is inheritable by child containers.
    #[serde(default)]
    pub inheritable: Option<bool>,
    /// Owner ID of the visualization. Typed as [`serde_json::Value`] because
    /// the server may return a number, string, or null.
    #[serde(default, rename = "ownerId")]
    pub owner_id: Option<serde_json::Value>,
    /// Query name scoping this visualization.
    #[serde(default, rename = "queryName")]
    pub query_name: Option<String>,
    /// Schema name scoping this visualization.
    #[serde(default, rename = "schemaName")]
    pub schema_name: Option<String>,
    /// Server-assigned report identifier.
    #[serde(default, rename = "reportId")]
    pub report_id: Option<String>,
    /// Arbitrary report properties. Typed as [`serde_json::Value`] because
    /// the structure is server-defined and not fixed.
    #[serde(default, rename = "reportProps")]
    pub report_props: Option<serde_json::Value>,
    /// Whether this visualization is shared with other users.
    #[serde(default)]
    pub shared: Option<bool>,
    /// URL of the visualization thumbnail image. The wire key is
    /// `thumbnailURL` (all-caps URL), not the camelCase `thumbnailUrl`.
    #[serde(default, rename = "thumbnailURL")]
    pub thumbnail_url: Option<String>,
    /// Unknown fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response payload from [`LabkeyClient::save_visualization`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SaveVisualizationResponse {
    /// Saved visualization name.
    #[serde(default)]
    pub name: Option<String>,
    /// Numeric visualization identifier.
    #[serde(default)]
    pub visualization_id: Option<i64>,
    /// Unknown fields preserved for forward compatibility.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Options for [`LabkeyClient::get_visualization`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetVisualizationOptions {
    /// Optional saved visualization name.
    pub name: Option<String>,
    /// Optional report identifier.
    pub report_id: Option<serde_json::Value>,
    /// Optional query scope.
    pub query_name: Option<String>,
    /// Optional schema scope.
    pub schema_name: Option<String>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// A measure entry used by [`GetVisualizationDataOptions`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct VisualizationDataMeasure {
    /// Measure descriptor payload.
    pub measure: serde_json::Value,
    /// Optional time-axis mode.
    pub time: Option<serde_json::Value>,
    /// Optional pivot dimension descriptor.
    pub dimension: Option<serde_json::Value>,
    /// Optional date configuration payload.
    pub date_options: Option<serde_json::Value>,
    /// Optional query filters converted to URL-encoded key/value strings.
    pub filter_array: Option<Vec<Filter>>,
}

/// Options for [`LabkeyClient::get_visualization_data`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetVisualizationDataOptions {
    /// Measures to retrieve.
    pub measures: Vec<VisualizationDataMeasure>,
    /// Optional sort descriptors.
    pub sorts: Option<serde_json::Value>,
    /// Legacy filter-query source.
    pub filter_query: Option<String>,
    /// Legacy URL-encoded filter source.
    pub filter_url: Option<String>,
    /// Optional row limit.
    pub limit: Option<serde_json::Value>,
    /// Optional group-by descriptors.
    pub group_bys: Option<serde_json::Value>,
    /// If true, return metadata without rows.
    pub meta_data_only: Option<bool>,
    /// Optional `visualization.param.*` URL parameters.
    pub parameters: Option<BTreeMap<String, String>>,
    /// Optional custom endpoint URL used verbatim.
    pub endpoint: Option<String>,
    /// If true, join all measures to the first measure.
    #[builder(default)]
    pub join_to_first: bool,
    /// Override the client's default container path for this request when
    /// `endpoint` is not provided.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::get_measures`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetMeasuresOptions {
    /// Optional visualization filter strings (for example `schema|query|type`).
    pub filters: Option<Vec<String>>,
    /// Include date measures instead of numeric measures.
    pub date_measures: Option<bool>,
    /// Include all columns.
    pub all_columns: Option<bool>,
    /// Include hidden measures.
    pub show_hidden: Option<bool>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::get_types`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetTypesOptions {
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::save_visualization`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SaveVisualizationOptions {
    /// Saved report name.
    pub name: String,
    /// Visualization type key.
    pub type_: String,
    /// Arbitrary visualization configuration object.
    pub visualization_config: serde_json::Value,
    /// Optional report description.
    pub description: Option<String>,
    /// Optional replace behavior.
    pub replace: Option<bool>,
    /// Optional shared visibility flag.
    pub shared: Option<bool>,
    /// Optional thumbnail behavior.
    pub thumbnail_type: Option<IconType>,
    /// Optional icon behavior.
    pub icon_type: Option<IconType>,
    /// Optional custom SVG payload.
    pub svg: Option<String>,
    /// Optional schema scope.
    pub schema_name: Option<String>,
    /// Optional query scope.
    pub query_name: Option<String>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::get_dimensions`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetDimensionsOptions {
    /// Query name for the owning measure.
    pub query_name: String,
    /// Schema name for the owning measure.
    pub schema_name: String,
    /// Include dimensions from demographic datasets.
    pub include_demographics: Option<bool>,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::get_dimension_values`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetDimensionValuesOptions {
    /// Dimension column name.
    pub name: String,
    /// Query name for the dimension.
    pub query_name: String,
    /// Schema name for the query.
    pub schema_name: String,
    /// Override the client's default container path for this request.
    pub container_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetVisualizationBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    report_id: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VisualizationDataBody {
    measures: Vec<VisualizationDataMeasureBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sorts: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter_query: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    group_bys: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    meta_data_only: Option<bool>,
    join_to_first: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct VisualizationDataMeasureBody {
    measure: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    time: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimension: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date_options: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter_array: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveVisualizationBody {
    json: String,
    name: String,
    #[serde(rename = "type")]
    type_: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    replace: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    shared: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thumbnail_type: Option<IconType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icon_type: Option<IconType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    svg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_name: Option<String>,
}

#[derive(Debug, Deserialize)]
struct MeasuresEnvelope {
    measures: Option<Vec<Measure>>,
}

#[derive(Debug, Deserialize)]
struct TypesEnvelope {
    types: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct DimensionsEnvelope {
    dimensions: Option<Vec<Dimension>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DimensionValuesEnvelope {
    success: Option<bool>,
    values: Option<Vec<serde_json::Value>>,
}

impl LabkeyClient {
    /// Retrieve a previously saved visualization definition.
    ///
    /// Sends a POST request to `visualization-getVisualization.api`.
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
    /// use labkey_rs::visualization::GetVisualizationOptions;
    ///
    /// let visualization = client
    ///     .get_visualization(
    ///         GetVisualizationOptions::builder()
    ///             .name("safety-dashboard".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{:?}", visualization.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_visualization(
        &self,
        options: GetVisualizationOptions,
    ) -> Result<VisualizationResponse, LabkeyError> {
        let url = self.build_url(
            "visualization",
            "getVisualization.api",
            options.container_path.as_deref(),
        );
        let body = GetVisualizationBody {
            name: options.name,
            report_id: options.report_id,
            query_name: options.query_name,
            schema_name: options.schema_name,
        };
        self.post(url, &body).await
    }

    /// Retrieve visualization data rows and metadata for provided measures.
    ///
    /// Sends a POST request to `visualization-getData` (no `.api` suffix) when
    /// `endpoint` is not provided. When `endpoint` is set, it is used verbatim
    /// and query parameters are appended as `visualization.param.*` pairs.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if endpoint URL input is invalid, the request
    /// fails, the server returns an error response, or the response body cannot
    /// be deserialized.
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
    /// use labkey_rs::visualization::{GetVisualizationDataOptions, VisualizationDataMeasure};
    ///
    /// let data = client
    ///     .get_visualization_data(
    ///         GetVisualizationDataOptions::builder()
    ///             .measures(vec![
    ///                 VisualizationDataMeasure::builder()
    ///                     .measure(serde_json::json!({"name": "Result"}))
    ///                     .build(),
    ///             ])
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", data.is_object());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_visualization_data(
        &self,
        options: GetVisualizationDataOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let params = build_visualization_params(options.parameters.as_ref());
        let url = build_visualization_data_url(self, &options, &params)?;
        let body = build_visualization_data_body(&options);
        self.post(url, &body).await
    }

    /// Retrieve plottable measures in the current container.
    ///
    /// Sends a GET request to `visualization-getMeasures.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, the response body cannot be deserialized, or the
    /// expected `measures` envelope field is missing.
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
    /// use labkey_rs::visualization::GetMeasuresOptions;
    ///
    /// let measures = client
    ///     .get_measures(GetMeasuresOptions::builder().build())
    ///     .await?;
    ///
    /// println!("{}", measures.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_measures(
        &self,
        options: GetMeasuresOptions,
    ) -> Result<Vec<Measure>, LabkeyError> {
        let url = self.build_url(
            "visualization",
            "getMeasures.api",
            options.container_path.as_deref(),
        );
        let params = [
            opt("dateMeasures", options.date_measures),
            opt("allColumns", options.all_columns),
            opt("showHidden", options.show_hidden),
        ]
        .into_iter()
        .flatten()
        .chain(
            options
                .filters
                .unwrap_or_default()
                .into_iter()
                .map(|value| ("filters".to_string(), value)),
        )
        .collect::<Vec<_>>();

        let envelope: MeasuresEnvelope = self.get(url, &params).await?;
        extract_measures(envelope)
    }

    /// Retrieve available visualization type descriptors.
    ///
    /// Sends a GET request to `visualization-getVisualizationTypes.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, the response body cannot be deserialized, or the
    /// expected `types` envelope field is missing.
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
    /// use labkey_rs::visualization::GetTypesOptions;
    ///
    /// let types = client.get_types(GetTypesOptions::builder().build()).await?;
    /// println!("{}", types.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_types(
        &self,
        options: GetTypesOptions,
    ) -> Result<Vec<serde_json::Value>, LabkeyError> {
        let url = self.build_url(
            "visualization",
            "getVisualizationTypes.api",
            options.container_path.as_deref(),
        );
        let envelope: TypesEnvelope = self.get(url, &[]).await?;
        extract_types(envelope)
    }

    /// Save a visualization definition for reuse.
    ///
    /// Sends a POST request to `visualization-saveVisualization.api` and
    /// serializes `visualization_config` into the wire `json` field.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if JSON encoding fails, the request fails, the
    /// server returns an error response, or the response body cannot be
    /// deserialized.
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
    /// use labkey_rs::visualization::SaveVisualizationOptions;
    ///
    /// let saved = client
    ///     .save_visualization(
    ///         SaveVisualizationOptions::builder()
    ///             .name("demo-chart".to_string())
    ///             .type_("line".to_string())
    ///             .visualization_config(serde_json::json!({"x": "Visit"}))
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{:?}", saved.visualization_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_visualization(
        &self,
        options: SaveVisualizationOptions,
    ) -> Result<SaveVisualizationResponse, LabkeyError> {
        let url = self.build_url(
            "visualization",
            "saveVisualization.api",
            options.container_path.as_deref(),
        );
        let body = build_save_visualization_body(options)?;
        self.post(url, &body).await
    }

    /// Retrieve dimensions available for a schema/query pair.
    ///
    /// Sends a GET request to `visualization-getDimensions.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, the response body cannot be deserialized, or the
    /// expected `dimensions` envelope field is missing.
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
    /// use labkey_rs::visualization::GetDimensionsOptions;
    ///
    /// let dimensions = client
    ///     .get_dimensions(
    ///         GetDimensionsOptions::builder()
    ///             .schema_name("study".to_string())
    ///             .query_name("LabResults".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", dimensions.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_dimensions(
        &self,
        options: GetDimensionsOptions,
    ) -> Result<Vec<Dimension>, LabkeyError> {
        let url = self.build_url(
            "visualization",
            "getDimensions.api",
            options.container_path.as_deref(),
        );
        let params = [
            opt("queryName", Some(options.query_name)),
            opt("schemaName", Some(options.schema_name)),
            opt("includeDemographics", options.include_demographics),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let envelope: DimensionsEnvelope = self.get(url, &params).await?;
        extract_dimensions(envelope)
    }

    /// Retrieve unique values for a specific visualization dimension.
    ///
    /// Sends a GET request to `visualization-getDimensionValues.api` and
    /// returns `response.values` only when `response.success == true`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server returns an
    /// error response, the response body cannot be deserialized, or success
    /// is false / values are missing.
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
    /// use labkey_rs::visualization::GetDimensionValuesOptions;
    ///
    /// let values = client
    ///     .get_dimension_values(
    ///         GetDimensionValuesOptions::builder()
    ///             .name("Visit".to_string())
    ///             .schema_name("study".to_string())
    ///             .query_name("LabResults".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("{}", values.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_dimension_values(
        &self,
        options: GetDimensionValuesOptions,
    ) -> Result<Vec<serde_json::Value>, LabkeyError> {
        let url = self.build_url(
            "visualization",
            "getDimensionValues.api",
            options.container_path.as_deref(),
        );
        let params = [
            opt("name", Some(options.name)),
            opt("queryName", Some(options.query_name)),
            opt("schemaName", Some(options.schema_name)),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

        let envelope: DimensionValuesEnvelope = self.get(url, &params).await?;
        extract_dimension_values(envelope)
    }
}

fn deserialize_visualization_config<'de, D>(deserializer: D) -> Result<serde_json::Value, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::String(payload) => {
            serde_json::from_str(&payload).map_err(DeError::custom)
        }
        other => Ok(other),
    }
}

fn build_visualization_data_body(options: &GetVisualizationDataOptions) -> VisualizationDataBody {
    VisualizationDataBody {
        measures: options
            .measures
            .iter()
            .map(|measure| VisualizationDataMeasureBody {
                measure: measure.measure.clone(),
                time: measure.time.clone(),
                dimension: measure.dimension.clone(),
                date_options: measure.date_options.clone(),
                filter_array: measure
                    .filter_array
                    .as_ref()
                    .map(|filters| convert_measure_filter_array(filters)),
            })
            .collect(),
        sorts: options.sorts.clone(),
        filter_query: options.filter_query.clone(),
        filter_url: options.filter_url.clone(),
        limit: options.limit.clone(),
        group_bys: options.group_bys.clone(),
        meta_data_only: options.meta_data_only,
        join_to_first: options.join_to_first,
    }
}

fn build_visualization_params(
    parameters: Option<&BTreeMap<String, String>>,
) -> Vec<(String, String)> {
    parameters
        .map(|pairs| {
            pairs
                .iter()
                .map(|(key, value)| (format!("visualization.param.{key}"), value.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn build_visualization_data_url(
    client: &LabkeyClient,
    options: &GetVisualizationDataOptions,
    params: &[(String, String)],
) -> Result<Url, LabkeyError> {
    let mut url = if let Some(endpoint) = options.endpoint.as_deref() {
        Url::parse(endpoint).map_err(|error| {
            LabkeyError::InvalidInput(format!(
                "get_visualization_data requires an absolute endpoint URL: {error}"
            ))
        })?
    } else {
        client.build_url(
            "visualization",
            "getData",
            options.container_path.as_deref(),
        )
    };

    if !params.is_empty() {
        let mut query_pairs = url.query_pairs_mut();
        for (key, value) in params {
            query_pairs.append_pair(key, value);
        }
    }

    Ok(url)
}

fn convert_measure_filter_array(filters: &[Filter]) -> Vec<String> {
    filters
        .iter()
        .map(|filter| {
            let param_name = filter.url_param_name("query");
            let param_value = filter.url_param_value();
            let key = urlencoding::encode(&param_name);
            let value = urlencoding::encode(&param_value);
            format!("{key}={value}")
        })
        .collect()
}

fn build_save_visualization_body(
    options: SaveVisualizationOptions,
) -> Result<SaveVisualizationBody, LabkeyError> {
    Ok(SaveVisualizationBody {
        json: serde_json::to_string(&options.visualization_config)?,
        name: options.name,
        type_: options.type_,
        description: options.description,
        replace: options.replace,
        shared: options.shared,
        thumbnail_type: options.thumbnail_type,
        icon_type: options.icon_type,
        svg: options.svg,
        schema_name: options.schema_name,
        query_name: options.query_name,
    })
}

fn extract_dimension_values(
    envelope: DimensionValuesEnvelope,
) -> Result<Vec<serde_json::Value>, LabkeyError> {
    if envelope.success == Some(true) {
        envelope
            .values
            .ok_or_else(|| LabkeyError::UnexpectedResponse {
                status: reqwest::StatusCode::OK,
                text:
                    "get_dimension_values response.success was true but response.values was missing"
                        .to_string(),
            })
    } else {
        Err(LabkeyError::UnexpectedResponse {
            status: reqwest::StatusCode::OK,
            text: "get_dimension_values response.success was not true".to_string(),
        })
    }
}

fn extract_measures(envelope: MeasuresEnvelope) -> Result<Vec<Measure>, LabkeyError> {
    envelope
        .measures
        .ok_or_else(|| LabkeyError::UnexpectedResponse {
            status: reqwest::StatusCode::OK,
            text: "get_measures response did not contain response.measures".to_string(),
        })
}

fn extract_types(envelope: TypesEnvelope) -> Result<Vec<serde_json::Value>, LabkeyError> {
    envelope
        .types
        .ok_or_else(|| LabkeyError::UnexpectedResponse {
            status: reqwest::StatusCode::OK,
            text: "get_types response did not contain response.types".to_string(),
        })
}

fn extract_dimensions(envelope: DimensionsEnvelope) -> Result<Vec<Dimension>, LabkeyError> {
    envelope
        .dimensions
        .ok_or_else(|| LabkeyError::UnexpectedResponse {
            status: reqwest::StatusCode::OK,
            text: "get_dimensions response did not contain response.dimensions".to_string(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClientConfig, Credential, filter::FilterType, filter::FilterValue};

    fn test_client(base_url: &str, container_path: &str) -> LabkeyClient {
        LabkeyClient::new(ClientConfig::new(
            base_url,
            Credential::ApiKey("test-key".to_string()),
            container_path,
        ))
        .expect("valid client config")
    }

    fn variant_count(value: IconType) -> usize {
        match value {
            IconType::Auto | IconType::Custom | IconType::None => 3,
        }
    }

    #[test]
    fn icon_type_round_trip_and_variant_count() {
        let auto = serde_json::to_string(&IconType::Auto).expect("serialize icon type");
        let custom = serde_json::to_string(&IconType::Custom).expect("serialize icon type");
        let none = serde_json::to_string(&IconType::None).expect("serialize icon type");

        assert_eq!(auto, "\"AUTO\"");
        assert_eq!(custom, "\"CUSTOM\"");
        assert_eq!(none, "\"NONE\"");

        let decoded_auto: IconType = serde_json::from_str(&auto).expect("deserialize icon type");
        let decoded_custom: IconType =
            serde_json::from_str(&custom).expect("deserialize icon type");
        let decoded_none: IconType = serde_json::from_str(&none).expect("deserialize icon type");

        assert_eq!(decoded_auto, IconType::Auto);
        assert_eq!(decoded_custom, IconType::Custom);
        assert_eq!(decoded_none, IconType::None);
        assert_eq!(variant_count(IconType::Auto), 3);
    }

    #[test]
    fn visualization_response_deserializes_with_decoded_config() {
        let response: VisualizationResponse = serde_json::from_value(serde_json::json!({
            "id": "vis-1",
            "name": "Safety",
            "type": "line",
            "visualizationConfig": "{\"x\":\"Visit\",\"y\":\"Result\"}"
        }))
        .expect("visualization response should deserialize");

        assert_eq!(response.id.as_deref(), Some("vis-1"));
        assert_eq!(response.name.as_deref(), Some("Safety"));
        assert_eq!(response.type_.as_deref(), Some("line"));
        assert_eq!(
            response.visualization_config["x"],
            serde_json::json!("Visit")
        );
    }

    #[test]
    fn save_visualization_body_encodes_visualization_config_as_json_string() {
        let body = build_save_visualization_body(
            SaveVisualizationOptions::builder()
                .name("demo".to_string())
                .type_("line".to_string())
                .visualization_config(serde_json::json!({"a": 1, "b": "x"}))
                .build(),
        )
        .expect("save body should build");

        let body_json = serde_json::to_value(body).expect("save body should serialize");
        assert_eq!(body_json["name"], serde_json::json!("demo"));
        assert_eq!(body_json["type"], serde_json::json!("line"));
        assert_eq!(
            body_json["json"],
            serde_json::json!("{\"a\":1,\"b\":\"x\"}")
        );
    }

    #[test]
    fn get_visualization_data_prefixes_parameter_keys() {
        let mut params = BTreeMap::new();
        params.insert("participantId".to_string(), "P-100".to_string());
        params.insert("cohort".to_string(), "A".to_string());

        let prefixed = build_visualization_params(Some(&params));

        assert!(prefixed.contains(&(
            "visualization.param.participantId".to_string(),
            "P-100".to_string()
        )));
        assert!(prefixed.contains(&("visualization.param.cohort".to_string(), "A".to_string())));
    }

    #[test]
    fn get_visualization_data_url_uses_default_no_api_and_custom_endpoint_verbatim() {
        let client = test_client("https://labkey.example.com/labkey", "/Project/Folder");
        let options = GetVisualizationDataOptions::builder()
            .measures(vec![
                VisualizationDataMeasure::builder()
                    .measure(serde_json::json!({"name": "Result"}))
                    .build(),
            ])
            .build();

        let default_url = build_visualization_data_url(&client, &options, &[])
            .expect("default get_data URL should build");
        assert_eq!(
            default_url.as_str(),
            "https://labkey.example.com/labkey/Project/Folder/visualization-getData"
        );

        let custom = GetVisualizationDataOptions::builder()
            .measures(vec![
                VisualizationDataMeasure::builder()
                    .measure(serde_json::json!({"name": "Result"}))
                    .build(),
            ])
            .endpoint("https://cdn.example.com/custom/data?x=1".to_string())
            .build();
        let custom_url = build_visualization_data_url(
            &client,
            &custom,
            &[("visualization.param.a".to_string(), "b".to_string())],
        )
        .expect("custom endpoint URL should build");
        assert_eq!(
            custom_url.as_str(),
            "https://cdn.example.com/custom/data?x=1&visualization.param.a=b"
        );
    }

    #[test]
    fn visualization_endpoint_urls_match_expected_actions() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url("visualization", "getVisualization.api", Some("/Alt/Vis"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Vis/visualization-getVisualization.api"
        );
        assert_eq!(
            client
                .build_url("visualization", "getData", Some("/Alt/Vis"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Vis/visualization-getData"
        );
        assert_eq!(
            client
                .build_url("visualization", "getMeasures.api", Some("/Alt/Vis"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Vis/visualization-getMeasures.api"
        );
        assert_eq!(
            client
                .build_url(
                    "visualization",
                    "getVisualizationTypes.api",
                    Some("/Alt/Vis")
                )
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Vis/visualization-getVisualizationTypes.api"
        );
        assert_eq!(
            client
                .build_url("visualization", "saveVisualization.api", Some("/Alt/Vis"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Vis/visualization-saveVisualization.api"
        );
        assert_eq!(
            client
                .build_url("visualization", "getDimensions.api", Some("/Alt/Vis"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Vis/visualization-getDimensions.api"
        );
        assert_eq!(
            client
                .build_url("visualization", "getDimensionValues.api", Some("/Alt/Vis"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Vis/visualization-getDimensionValues.api"
        );
    }

    #[test]
    fn get_visualization_data_converts_measure_filters_to_encoded_strings() {
        let filter = Filter::new(
            "ParticipantId",
            FilterType::Equal,
            FilterValue::Single("P-100".to_string()),
        );
        let body = build_visualization_data_body(
            &GetVisualizationDataOptions::builder()
                .measures(vec![
                    VisualizationDataMeasure::builder()
                        .measure(serde_json::json!({"name": "Result"}))
                        .filter_array(vec![filter])
                        .build(),
                ])
                .build(),
        );

        let body_json =
            serde_json::to_value(body).expect("visualization data body should serialize");
        assert_eq!(
            body_json["measures"][0]["filterArray"][0],
            serde_json::json!("query.ParticipantId~eq=P-100")
        );
    }

    #[test]
    fn get_visualization_data_filter_conversion_url_encodes_reserved_characters() {
        let filter = Filter::new(
            "ParticipantId",
            FilterType::Equal,
            FilterValue::Single("A&B Value".to_string()),
        );

        let body = build_visualization_data_body(
            &GetVisualizationDataOptions::builder()
                .measures(vec![
                    VisualizationDataMeasure::builder()
                        .measure(serde_json::json!({"name": "Result"}))
                        .filter_array(vec![filter])
                        .build(),
                ])
                .build(),
        );

        let encoded = serde_json::to_value(body).expect("visualization data body should serialize")
            ["measures"][0]["filterArray"][0]
            .as_str()
            .expect("encoded filter should be a string")
            .to_string();

        assert!(encoded.contains("query.ParticipantId~eq="));
        assert!(encoded.contains("%26"));
        assert!(encoded.contains("%20"));
    }

    #[test]
    fn measure_and_dimension_deserialize_with_minimal_fields() {
        let measure: Measure = serde_json::from_value(serde_json::json!({
            "name": "Result",
            "schemaName": "study",
            "queryName": "LabResults",
            "type": "float"
        }))
        .expect("measure should deserialize");
        assert_eq!(measure.name.as_deref(), Some("Result"));
        assert_eq!(measure.type_.as_deref(), Some("float"));

        let dimension: Dimension = serde_json::from_value(serde_json::json!({
            "name": "Visit",
            "schemaName": "study",
            "queryName": "LabResults",
            "type": "string"
        }))
        .expect("dimension should deserialize");
        assert_eq!(dimension.name.as_deref(), Some("Visit"));
        assert_eq!(dimension.type_.as_deref(), Some("string"));
    }

    #[test]
    fn dimension_values_extraction_requires_success_true_and_values() {
        let values = extract_dimension_values(DimensionValuesEnvelope {
            success: Some(true),
            values: Some(vec![
                serde_json::json!("Baseline"),
                serde_json::json!("Week 1"),
            ]),
        })
        .expect("successful envelope should extract values");
        assert_eq!(values.len(), 2);

        let missing_values_error = extract_dimension_values(DimensionValuesEnvelope {
            success: Some(true),
            values: None,
        })
        .expect_err("missing values should error");
        assert!(matches!(
            missing_values_error,
            LabkeyError::UnexpectedResponse { .. }
        ));

        let not_success_error = extract_dimension_values(DimensionValuesEnvelope {
            success: Some(false),
            values: Some(vec![]),
        })
        .expect_err("success false should error");
        assert!(matches!(
            not_success_error,
            LabkeyError::UnexpectedResponse { .. }
        ));
    }

    #[test]
    fn visualization_response_deserializes_typed_fields_and_leaves_no_extra() {
        let response: VisualizationResponse = serde_json::from_value(serde_json::json!({
            "id": "vis-42",
            "name": "Lab Dashboard",
            "type": "scatter",
            "visualizationConfig": "{\"axis\":\"x\"}",
            "canDelete": true,
            "canEdit": false,
            "canShare": true,
            "createdBy": 1001,
            "description": "A test visualization",
            "inheritable": true,
            "ownerId": 2002,
            "queryName": "LabResults",
            "schemaName": "study",
            "reportId": "db:42",
            "reportProps": {"color": "red"},
            "shared": false,
            "thumbnailURL": "https://example.com/thumb.png"
        }))
        .expect("full visualization response should deserialize");

        assert_eq!(response.id.as_deref(), Some("vis-42"));
        assert_eq!(response.name.as_deref(), Some("Lab Dashboard"));
        assert_eq!(response.type_.as_deref(), Some("scatter"));
        assert_eq!(
            response.visualization_config["axis"],
            serde_json::json!("x")
        );
        assert_eq!(response.can_delete, Some(true));
        assert_eq!(response.can_edit, Some(false));
        assert_eq!(response.can_share, Some(true));
        assert_eq!(response.created_by, Some(1001));
        assert_eq!(
            response.description.as_deref(),
            Some("A test visualization")
        );
        assert_eq!(response.inheritable, Some(true));
        assert_eq!(response.owner_id, Some(serde_json::json!(2002)));
        assert_eq!(response.query_name.as_deref(), Some("LabResults"));
        assert_eq!(response.schema_name.as_deref(), Some("study"));
        assert_eq!(response.report_id.as_deref(), Some("db:42"));
        assert_eq!(
            response.report_props,
            Some(serde_json::json!({"color": "red"}))
        );
        assert_eq!(response.shared, Some(false));
        assert_eq!(
            response.thumbnail_url.as_deref(),
            Some("https://example.com/thumb.png")
        );
        assert!(
            response.extra.is_empty(),
            "extra should be empty when all known fields are typed, but found: {:?}",
            response.extra
        );
    }

    #[test]
    fn visualization_response_minimal_defaults_optional_fields() {
        let response: VisualizationResponse = serde_json::from_value(serde_json::json!({
            "id": "vis-min",
            "name": "Minimal"
        }))
        .expect("minimal visualization response should deserialize");

        assert_eq!(response.id.as_deref(), Some("vis-min"));
        assert_eq!(response.name.as_deref(), Some("Minimal"));
        assert_eq!(response.type_, None);
        assert_eq!(response.visualization_config, serde_json::Value::Null);
        assert_eq!(response.can_delete, None);
        assert_eq!(response.can_edit, None);
        assert_eq!(response.can_share, None);
        assert_eq!(response.created_by, None);
        assert_eq!(response.description, None);
        assert_eq!(response.inheritable, None);
        assert_eq!(response.owner_id, None);
        assert_eq!(response.query_name, None);
        assert_eq!(response.schema_name, None);
        assert_eq!(response.report_id, None);
        assert_eq!(response.report_props, None);
        assert_eq!(response.shared, None);
        assert_eq!(response.thumbnail_url, None);
        assert!(response.extra.is_empty());
    }

    #[test]
    fn visualization_response_unknown_field_lands_in_extra() {
        let response: VisualizationResponse = serde_json::from_value(serde_json::json!({
            "id": "vis-ext",
            "name": "WithExtra",
            "futureField": 42,
            "anotherUnknown": "hello"
        }))
        .expect("visualization response with unknown fields should deserialize");

        assert_eq!(response.id.as_deref(), Some("vis-ext"));
        assert_eq!(response.name.as_deref(), Some("WithExtra"));
        assert_eq!(
            response.extra.get("futureField"),
            Some(&serde_json::json!(42))
        );
        assert_eq!(
            response.extra.get("anotherUnknown"),
            Some(&serde_json::json!("hello"))
        );
        assert_eq!(
            response.extra.len(),
            2,
            "only unknown fields should be in extra"
        );
    }

    #[test]
    fn visualization_response_accepts_non_numeric_owner_id_and_complex_report_props() {
        let response: VisualizationResponse = serde_json::from_value(serde_json::json!({
            "id": "vis-edge",
            "name": "EdgeCases",
            "ownerId": "user-string-id",
            "reportProps": {
                "nested": {"deep": true},
                "list": [1, 2, 3],
                "flag": false,
                "count": 42
            }
        }))
        .expect("non-numeric ownerId and complex reportProps should deserialize");

        assert_eq!(response.owner_id, Some(serde_json::json!("user-string-id")));
        let props = response.report_props.expect("reportProps should be Some");
        assert_eq!(props["nested"]["deep"], serde_json::json!(true));
        assert_eq!(props["list"], serde_json::json!([1, 2, 3]));
        assert_eq!(props["flag"], serde_json::json!(false));
        assert_eq!(props["count"], serde_json::json!(42));
        assert!(response.extra.is_empty());
    }

    #[test]
    fn envelope_extractors_require_expected_fields() {
        let measures_error =
            extract_measures(MeasuresEnvelope { measures: None }).expect_err("should error");
        assert!(matches!(
            measures_error,
            LabkeyError::UnexpectedResponse { .. }
        ));

        let types_error = extract_types(TypesEnvelope { types: None }).expect_err("should error");
        assert!(matches!(
            types_error,
            LabkeyError::UnexpectedResponse { .. }
        ));

        let dimensions_error =
            extract_dimensions(DimensionsEnvelope { dimensions: None }).expect_err("should error");
        assert!(matches!(
            dimensions_error,
            LabkeyError::UnexpectedResponse { .. }
        ));
    }
}
