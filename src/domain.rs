//! Domain models and property-controller endpoints.
//!
//! A "domain" in LabKey is the schema definition behind a list, dataset, sample
//! type, or other structured data source. It consists of a [`DomainDesign`]
//! containing [`DomainField`] definitions, optional indices, and conditional
//! formatting rules. This module provides endpoints for creating, reading,
//! updating, and deleting domains, as well as querying property usages and
//! validating name expressions.

use std::{collections::HashMap, time::Duration};

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    client::{LabkeyClient, RequestOptions},
    common::opt,
    error::LabkeyError,
};

/// Domain kind values used by LabKey property APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum DomainKind {
    /// Data class domain.
    #[serde(rename = "DataClass")]
    DataClass,
    /// Integer-list domain.
    #[serde(rename = "IntList")]
    IntList,
    /// Sample-set domain.
    #[serde(rename = "SampleSet")]
    SampleSet,
    /// Study dataset domain keyed by date.
    #[serde(rename = "StudyDatasetDate")]
    StudyDatasetDate,
    /// Study dataset domain keyed by visit.
    #[serde(rename = "StudyDatasetVisit")]
    StudyDatasetVisit,
    /// Unknown or server-provided fallback domain kind.
    #[serde(rename = "Unknown")]
    Unknown,
    /// Variant-list domain.
    #[serde(rename = "VarList")]
    VarList,
}

impl DomainKind {
    const fn as_wire(self) -> &'static str {
        match self {
            Self::DataClass => "DataClass",
            Self::IntList => "IntList",
            Self::SampleSet => "SampleSet",
            Self::StudyDatasetDate => "StudyDatasetDate",
            Self::StudyDatasetVisit => "StudyDatasetVisit",
            Self::Unknown => "Unknown",
            Self::VarList => "VarList",
        }
    }
}

/// Conditional format rule applied to a domain field.
///
/// Matches the Java `ConditionalFormat` class
/// (`domain/ConditionalFormat.java`). The `filter` field is a URL-encoded
/// filter string on the wire (e.g. `"~eq=val&~gt=5"`); Java parses this
/// into `ConditionalFormatFilter` objects, but we keep it as a raw string
/// to avoid coupling to the filter URL encoding format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ConditionalFormat {
    /// URL-encoded filter expression.
    #[serde(default)]
    pub filter: Option<String>,
    /// Text color (CSS hex string, e.g. `"#FF0000"`).
    #[serde(default)]
    pub text_color: Option<String>,
    /// Background color (CSS hex string).
    #[serde(default)]
    pub background_color: Option<String>,
    /// Whether to render text in bold.
    #[serde(default)]
    pub bold: bool,
    /// Whether to render text in italic.
    #[serde(default)]
    pub italic: bool,
    /// Whether to render text with strikethrough.
    #[serde(default)]
    pub strikethrough: bool,
}

/// A field definition inside a domain design.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DomainField {
    /// Concept URI for this field.
    #[serde(rename = "conceptURI")]
    #[serde(default)]
    pub concept_uri: Option<String>,
    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,
    /// Display format string.
    #[serde(default)]
    pub format: Option<String>,
    /// User-visible label.
    #[serde(default)]
    pub label: Option<String>,
    /// Lookup container path.
    #[serde(default)]
    pub lookup_container: Option<String>,
    /// Lookup schema name.
    #[serde(default)]
    pub lookup_schema: Option<String>,
    /// Lookup query name.
    #[serde(default)]
    pub lookup_query: Option<String>,
    /// Property name.
    #[serde(default)]
    pub name: Option<String>,
    /// Property id.
    #[serde(default)]
    pub property_id: Option<i64>,
    /// Property URI.
    #[serde(rename = "propertyURI")]
    #[serde(default)]
    pub property_uri: Option<String>,
    /// Ontology URI.
    #[serde(rename = "ontologyURI")]
    #[serde(default)]
    pub ontology_uri: Option<String>,
    /// Range URI.
    #[serde(rename = "rangeURI")]
    #[serde(default)]
    pub range_uri: Option<String>,
    /// Whether a value is required.
    #[serde(default)]
    pub required: Option<bool>,
    /// Search terms metadata.
    #[serde(default)]
    pub search_terms: Option<String>,
    /// Semantic type URI.
    #[serde(default)]
    pub semantic_type: Option<String>,
    /// Whether this field is hidden from the default view.
    #[serde(default)]
    pub hidden: Option<bool>,
    /// Protected Health Information level (e.g. `"NotPHI"`, `"Limited"`,
    /// `"PHI"`, `"Restricted"`). Java-only; the JS client has no typed
    /// equivalent.
    #[serde(rename = "PHI", default)]
    pub phi: Option<String>,
    /// Whether this field is a measure (for charting/aggregation).
    #[serde(default)]
    pub measure: Option<bool>,
    /// Whether this field is a dimension (for charting/grouping).
    #[serde(default)]
    pub dimension: Option<bool>,
    /// Whether missing-value indicators are enabled for this field.
    #[serde(default)]
    pub mv_enabled: Option<bool>,
    /// Derivation data scope for derived fields.
    #[serde(default)]
    pub derivation_data_scope: Option<String>,
    /// Conditional formatting rules applied to this field.
    #[serde(default)]
    pub conditional_formats: Vec<ConditionalFormat>,
    /// Additional server-provided keys.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Index definition for a domain.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DomainIndex {
    /// Names of indexed columns.
    #[serde(default)]
    pub column_names: Vec<String>,
    /// Whether this index enforces uniqueness.
    #[serde(default)]
    pub unique: Option<bool>,
}

/// Full domain design payload used for both requests and responses.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DomainDesign {
    /// Domain id.
    #[serde(default)]
    pub domain_id: Option<i64>,
    /// Domain URI.
    #[serde(rename = "domainURI")]
    #[serde(default)]
    pub domain_uri: Option<String>,
    /// Domain name.
    #[serde(default)]
    pub name: Option<String>,
    /// Domain description.
    #[serde(default)]
    pub description: Option<String>,
    /// Domain schema name when applicable.
    #[serde(default)]
    pub schema_name: Option<String>,
    /// Domain query name when applicable.
    #[serde(default)]
    pub query_name: Option<String>,
    /// Domain field definitions.
    #[serde(default)]
    pub fields: Option<Vec<DomainField>>,
    /// Domain index definitions.
    #[serde(default)]
    pub indices: Option<Vec<DomainIndex>>,
    /// Additional server-provided keys.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Usage information for a property descriptor.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PropertyUsage {
    /// Objects using this property.
    #[serde(default)]
    pub objects: Vec<serde_json::Value>,
    /// Property id.
    pub property_id: i64,
    /// Property URI.
    #[serde(rename = "propertyURI")]
    pub property_uri: String,
    /// Count of usages.
    pub usage_count: i64,
}

/// Options for [`LabkeyClient::create_domain`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct CreateDomainOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Domain design payload.
    pub domain_design: Option<DomainDesign>,
    /// Domain template group name.
    pub domain_group: Option<String>,
    /// Domain template name.
    pub domain_template: Option<String>,
    /// Module containing the template.
    pub module: Option<String>,
    /// Domain kind.
    pub kind: Option<DomainKind>,
    /// Domain-kind-specific options.
    pub options: Option<serde_json::Value>,
    /// Whether to create the domain when template mode is used.
    pub create_domain: Option<bool>,
    /// Whether to import template data.
    pub import_data: Option<bool>,
    /// Alternate server domain-kind string, when needed.
    pub domain_kind: Option<String>,
    /// Optional request timeout override.
    pub timeout: Option<Duration>,
}

/// Options for [`LabkeyClient::get_domain`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetDomainOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Domain id.
    pub domain_id: Option<i64>,
    /// Domain query name.
    pub query_name: Option<String>,
    /// Domain schema name.
    pub schema_name: Option<String>,
}

/// Options for [`LabkeyClient::get_domain_details`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetDomainDetailsOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Domain id.
    pub domain_id: Option<i64>,
    /// Domain query name.
    pub query_name: Option<String>,
    /// Domain schema name.
    pub schema_name: Option<String>,
    /// Domain kind for domain creation metadata lookups.
    pub domain_kind: Option<DomainKind>,
}

/// Response from [`LabkeyClient::get_domain_details`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct GetDomainDetailsResponse {
    /// Domain definition.
    pub domain_design: DomainDesign,
    /// Domain kind name.
    pub domain_kind_name: String,
    /// Domain-kind-specific options.
    #[serde(default)]
    pub options: Option<serde_json::Value>,
}

/// Options for [`LabkeyClient::save_domain`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SaveDomainOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Domain design payload.
    pub domain_design: Option<DomainDesign>,
    /// Domain schema name.
    pub schema_name: Option<String>,
    /// Domain query name.
    pub query_name: Option<String>,
    /// Domain id.
    pub domain_id: Option<i64>,
    /// Include warnings in server-side validation.
    pub include_warnings: Option<bool>,
    /// Domain-kind-specific options.
    pub options: Option<serde_json::Value>,
    /// Optional audit comment.
    pub audit_user_comment: Option<String>,
}

/// Options for [`LabkeyClient::drop_domain`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct DropDomainOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Domain schema name.
    pub schema_name: String,
    /// Domain query name.
    pub query_name: String,
    /// Optional domain design payload.
    pub domain_design: Option<DomainDesign>,
    /// Optional audit comment.
    pub audit_user_comment: Option<String>,
}

/// Options for [`LabkeyClient::update_domain`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct UpdateDomainOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Domain id.
    pub domain_id: i64,
    /// Fields to create.
    pub create_fields: Option<Vec<DomainField>>,
    /// Fields to update.
    pub update_fields: Option<Vec<DomainField>>,
    /// Field ids to delete.
    pub delete_fields: Option<Vec<i64>>,
    /// Include warnings in server-side validation.
    pub include_warnings: Option<bool>,
}

/// Options for [`LabkeyClient::list_domains`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct ListDomainsOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Domain kinds to include.
    pub domain_kinds: Option<Vec<DomainKind>>,
    /// Include field definitions in each domain result.
    pub include_fields: Option<bool>,
    /// Include project-level and shared domains.
    pub include_project_and_shared: Option<bool>,
}

/// Response from [`LabkeyClient::list_domains`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ListDomainsResponse {
    /// Returned domain designs.
    #[serde(default)]
    pub data: Vec<DomainDesign>,
    /// Optional success flag when returned.
    #[serde(default)]
    pub success: Option<bool>,
}

/// Options for [`LabkeyClient::validate_name_expressions`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct ValidateNameExpressionsOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Domain design payload.
    pub domain_design: Option<DomainDesign>,
    /// Domain-kind-specific options.
    pub options: Option<serde_json::Value>,
    /// Domain kind.
    pub kind: Option<DomainKind>,
    /// When `Some(true)`, include generated name preview in the response.
    /// Omitted from the request when `None`; the server treats omission as
    /// `false`.
    pub include_name_preview: Option<bool>,
}

/// Options for [`LabkeyClient::get_domain_name_previews`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetDomainNamePreviewsOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Domain id.
    pub domain_id: Option<i64>,
    /// Domain query name.
    pub query_name: Option<String>,
    /// Domain schema name.
    pub schema_name: Option<String>,
}

/// Options for [`LabkeyClient::get_properties`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetPropertiesOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Limit properties to these domain ids.
    pub domain_ids: Option<Vec<i64>>,
    /// Limit properties to these domain kind strings.
    pub domain_kinds: Option<Vec<String>>,
    /// Filter expressions to apply.
    pub filters: Option<Vec<String>>,
    /// Max rows to return.
    pub max_rows: Option<i64>,
    /// Start offset.
    pub offset: Option<i64>,
    /// Limit properties to these property ids.
    pub property_ids: Option<Vec<i64>>,
    /// Limit properties to these property URIs.
    pub property_uris: Option<Vec<String>>,
    /// Search term.
    pub search: Option<String>,
    /// Sort expression.
    pub sort: Option<String>,
}

/// Options for [`LabkeyClient::get_property_usages`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetPropertyUsagesOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Maximum usage objects per property.
    pub max_usage_count: Option<i64>,
    /// Property ids to query.
    pub property_ids: Option<Vec<i64>>,
    /// Property URIs to query.
    pub property_uris: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateDomainBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    create_domain: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain_design: Option<DomainDesign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain_group: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain_template: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    import_data: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<DomainKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    module: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveDomainBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    domain_design: Option<DomainDesign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    query_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_warnings: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audit_user_comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DropDomainBody {
    schema_name: String,
    query_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain_design: Option<DomainDesign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    audit_user_comment: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateDomainBody {
    domain_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_warnings: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    create_fields: Option<Vec<DomainField>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    update_fields: Option<Vec<DomainField>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    delete_fields: Option<Vec<i64>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidateNameExpressionsBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    domain_design: Option<DomainDesign>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<DomainKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_name_preview: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GetPropertiesBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    domain_ids: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain_kinds: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filters: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_rows: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    offset: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    property_ids: Option<Vec<i64>>,
    #[serde(rename = "propertyURIs", skip_serializing_if = "Option::is_none")]
    property_uris: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    search: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    sort: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PropertyUsagesEnvelope {
    data: Vec<PropertyUsage>,
}

fn extract_property_usages(
    response: &serde_json::Value,
) -> Result<Vec<PropertyUsage>, LabkeyError> {
    serde_json::from_value::<PropertyUsagesEnvelope>(response.clone())
        .map(|envelope| envelope.data)
        .map_err(|_| LabkeyError::UnexpectedResponse {
            status: StatusCode::OK,
            text: format!("invalid propertyUsages response: {response}"),
        })
}

impl LabkeyClient {
    /// Create a domain through `property-createDomain.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server reports an API
    /// error, or the response cannot be parsed.
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
    /// use labkey_rs::domain::{CreateDomainOptions, DomainKind};
    ///
    /// let _ = client
    ///     .create_domain(
    ///         CreateDomainOptions::builder()
    ///             .kind(DomainKind::IntList)
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_domain(
        &self,
        options: CreateDomainOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "property",
            "createDomain.api",
            options.container_path.as_deref(),
        );
        let request_options = RequestOptions {
            timeout: options.timeout,
            ..RequestOptions::default()
        };
        let body = CreateDomainBody {
            create_domain: options.create_domain,
            domain_design: options.domain_design,
            domain_group: options.domain_group,
            domain_kind: options.domain_kind,
            domain_template: options.domain_template,
            import_data: options.import_data,
            kind: options.kind,
            module: options.module,
            options: options.options,
        };

        self.post_with_options(url, &body, &request_options).await
    }

    /// Get a domain through the deprecated `property-getDomain.api` endpoint.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server reports an API
    /// error, or the response cannot be parsed.
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
    /// use labkey_rs::domain::GetDomainOptions;
    ///
    /// let _ = client
    ///     .get_domain(
    ///         GetDomainOptions::builder()
    ///             .schema_name("study".to_string())
    ///             .query_name("StudyProperties".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    #[deprecated(note = "use get_domain_details")]
    pub async fn get_domain(&self, options: GetDomainOptions) -> Result<DomainDesign, LabkeyError> {
        let url = self.build_url(
            "property",
            "getDomain.api",
            options.container_path.as_deref(),
        );
        let params: Vec<(String, String)> = [
            opt("schemaName", options.schema_name),
            opt("queryName", options.query_name),
            opt("domainId", options.domain_id),
        ]
        .into_iter()
        .flatten()
        .collect();

        self.get(url, &params).await
    }

    /// Get detailed domain information through `property-getDomainDetails.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server reports an API
    /// error, or the response cannot be parsed.
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
    /// use labkey_rs::domain::GetDomainDetailsOptions;
    ///
    /// let details = client
    ///     .get_domain_details(
    ///         GetDomainDetailsOptions::builder()
    ///             .schema_name("study".to_string())
    ///             .query_name("StudyProperties".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("domain kind: {}", details.domain_kind_name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_domain_details(
        &self,
        options: GetDomainDetailsOptions,
    ) -> Result<GetDomainDetailsResponse, LabkeyError> {
        let url = self.build_url(
            "property",
            "getDomainDetails.api",
            options.container_path.as_deref(),
        );
        let params: Vec<(String, String)> = [
            opt("schemaName", options.schema_name),
            opt("queryName", options.query_name),
            opt("domainId", options.domain_id),
            options
                .domain_kind
                .map(|value| ("domainKind".to_string(), value.as_wire().to_string())),
        ]
        .into_iter()
        .flatten()
        .collect();

        self.get(url, &params).await
    }

    /// Save a domain design through `property-saveDomain.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server reports an API
    /// error, or the response cannot be parsed.
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
    /// use labkey_rs::domain::SaveDomainOptions;
    ///
    /// let _ = client.save_domain(SaveDomainOptions::builder().build()).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_domain(
        &self,
        options: SaveDomainOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "property",
            "saveDomain.api",
            options.container_path.as_deref(),
        );
        let body = SaveDomainBody {
            domain_design: options.domain_design,
            schema_name: options.schema_name,
            query_name: options.query_name,
            domain_id: options.domain_id,
            include_warnings: options.include_warnings,
            audit_user_comment: options.audit_user_comment,
            options: options.options,
        };

        self.post(url, &body).await
    }

    /// Delete a domain through `property-deleteDomain.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server reports an API
    /// error, or the response cannot be parsed.
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
    /// use labkey_rs::domain::DropDomainOptions;
    ///
    /// let _ = client
    ///     .drop_domain(
    ///         DropDomainOptions::builder()
    ///             .schema_name("study".to_string())
    ///             .query_name("StudyProperties".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn drop_domain(
        &self,
        options: DropDomainOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "property",
            "deleteDomain.api",
            options.container_path.as_deref(),
        );
        let body = DropDomainBody {
            schema_name: options.schema_name,
            query_name: options.query_name,
            domain_design: options.domain_design,
            audit_user_comment: options.audit_user_comment,
        };

        self.post(url, &body).await
    }

    /// Update a domain through `property-updateDomain.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server reports an API
    /// error, or the response cannot be parsed.
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
    /// use labkey_rs::domain::UpdateDomainOptions;
    ///
    /// let _ = client
    ///     .update_domain(UpdateDomainOptions::builder().domain_id(101).build())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_domain(
        &self,
        options: UpdateDomainOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "property",
            "updateDomain.api",
            options.container_path.as_deref(),
        );
        let body = UpdateDomainBody {
            domain_id: options.domain_id,
            include_warnings: options.include_warnings,
            create_fields: options.create_fields,
            update_fields: options.update_fields,
            delete_fields: options.delete_fields,
        };

        self.post(url, &body).await
    }

    /// List domains through `property-listDomains.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server reports an API
    /// error, or the response cannot be parsed.
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
    /// use labkey_rs::domain::ListDomainsOptions;
    ///
    /// let response = client
    ///     .list_domains(ListDomainsOptions::builder().build())
    ///     .await?;
    ///
    /// println!("{} domains", response.data.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_domains(
        &self,
        options: ListDomainsOptions,
    ) -> Result<ListDomainsResponse, LabkeyError> {
        let url = self.build_url(
            "property",
            "listDomains.api",
            options.container_path.as_deref(),
        );
        let mut params: Vec<(String, String)> = [
            opt("includeFields", options.include_fields),
            opt(
                "includeProjectAndShared",
                options.include_project_and_shared,
            ),
        ]
        .into_iter()
        .flatten()
        .collect();

        if let Some(domain_kinds) = options.domain_kinds.as_ref() {
            params.extend(
                domain_kinds
                    .iter()
                    .map(|value| ("domainKinds".to_string(), value.as_wire().to_string())),
            );
        }

        self.get(url, &params).await
    }

    /// Validate domain naming expressions through `property-validateNameExpressions.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server reports an API
    /// error, or the response cannot be parsed.
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
    /// use labkey_rs::domain::ValidateNameExpressionsOptions;
    ///
    /// let _ = client
    ///     .validate_name_expressions(ValidateNameExpressionsOptions::builder().build())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate_name_expressions(
        &self,
        options: ValidateNameExpressionsOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "property",
            "validateNameExpressions.api",
            options.container_path.as_deref(),
        );
        let body = ValidateNameExpressionsBody {
            domain_design: options.domain_design,
            options: options.options,
            kind: options.kind,
            include_name_preview: options.include_name_preview,
        };

        self.post(url, &body).await
    }

    /// Get generated domain name previews through `property-getDomainNamePreviews.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server reports an API
    /// error, or the response cannot be parsed.
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
    /// use labkey_rs::domain::GetDomainNamePreviewsOptions;
    ///
    /// let _ = client
    ///     .get_domain_name_previews(GetDomainNamePreviewsOptions::builder().build())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_domain_name_previews(
        &self,
        options: GetDomainNamePreviewsOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "property",
            "getDomainNamePreviews.api",
            options.container_path.as_deref(),
        );
        let params: Vec<(String, String)> = [
            opt("schemaName", options.schema_name),
            opt("queryName", options.query_name),
            opt("domainId", options.domain_id),
        ]
        .into_iter()
        .flatten()
        .collect();

        self.get(url, &params).await
    }

    /// Query property descriptors through `property-getProperties.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server reports an API
    /// error, or the response cannot be parsed.
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
    /// use labkey_rs::domain::GetPropertiesOptions;
    ///
    /// let _ = client
    ///     .get_properties(GetPropertiesOptions::builder().build())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_properties(
        &self,
        options: GetPropertiesOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "property",
            "getProperties.api",
            options.container_path.as_deref(),
        );
        let body = GetPropertiesBody {
            domain_ids: options.domain_ids,
            domain_kinds: options.domain_kinds,
            filters: options.filters,
            max_rows: options.max_rows,
            offset: options.offset,
            property_ids: options.property_ids,
            property_uris: options.property_uris,
            search: options.search,
            sort: options.sort,
        };

        self.post(url, &body).await
    }

    /// Get property usages through `property-propertyUsages.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server reports an API
    /// error, or the response cannot be parsed, including malformed envelopes
    /// missing `response.data`.
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
    /// use labkey_rs::domain::GetPropertyUsagesOptions;
    ///
    /// let _ = client
    ///     .get_property_usages(GetPropertyUsagesOptions::builder().build())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_property_usages(
        &self,
        options: GetPropertyUsagesOptions,
    ) -> Result<Vec<PropertyUsage>, LabkeyError> {
        let url = self.build_url(
            "property",
            "propertyUsages.api",
            options.container_path.as_deref(),
        );
        let mut params: Vec<(String, String)> = [opt("maxUsageCount", options.max_usage_count)]
            .into_iter()
            .flatten()
            .collect();

        if let Some(property_ids) = options.property_ids.as_ref() {
            params.extend(
                property_ids
                    .iter()
                    .map(|value| ("propertyIds".to_string(), value.to_string())),
            );
        }
        if let Some(property_uris) = options.property_uris.as_ref() {
            params.extend(
                property_uris
                    .iter()
                    .map(|value| ("propertyURIs".to_string(), value.clone())),
            );
        }

        let response: serde_json::Value = self.get(url, &params).await?;
        extract_property_usages(&response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{ClientConfig, Credential};

    fn test_client() -> LabkeyClient {
        LabkeyClient::new(ClientConfig {
            base_url: "https://labkey.example.com/labkey".to_string(),
            credential: Credential::ApiKey("test-key".to_string()),
            container_path: "/Project/Folder".to_string(),
            user_agent: None,
            accept_self_signed_certs: false,
            proxy_url: None,
            csrf_token: None,
        })
        .expect("valid test client")
    }

    #[test]
    fn domain_design_round_trip() {
        let mut extra = HashMap::new();
        extra.insert("scale".to_string(), serde_json::json!(4));
        let field = DomainField {
            concept_uri: Some("urn:test:concept".to_string()),
            description: Some("Test field".to_string()),
            format: Some(String::new()),
            label: Some("Code".to_string()),
            lookup_container: None,
            lookup_schema: None,
            lookup_query: None,
            name: Some("code".to_string()),
            property_id: Some(101),
            property_uri: Some("urn:lsid:test:101".to_string()),
            ontology_uri: None,
            range_uri: Some("string".to_string()),
            required: Some(true),
            search_terms: None,
            semantic_type: None,
            hidden: None,
            phi: None,
            measure: None,
            dimension: None,
            mv_enabled: None,
            derivation_data_scope: None,
            conditional_formats: vec![],
            extra,
        };

        let domain = DomainDesign {
            domain_id: Some(55),
            domain_uri: Some("urn:lsid:test:domain".to_string()),
            name: Some("LookupCodes".to_string()),
            description: Some("test domain".to_string()),
            schema_name: Some("lists".to_string()),
            query_name: Some("LookupCodes".to_string()),
            fields: Some(vec![field]),
            indices: Some(vec![DomainIndex {
                column_names: vec!["code".to_string()],
                unique: Some(true),
            }]),
            extra: HashMap::new(),
        };

        let json = serde_json::to_string(&domain).expect("serialize domain design");
        let restored: DomainDesign =
            serde_json::from_str(&json).expect("deserialize domain design");
        assert_eq!(restored.name.as_deref(), Some("LookupCodes"));
        assert_eq!(restored.fields.as_ref().map(std::vec::Vec::len), Some(1));
        assert_eq!(restored.indices.as_ref().map(std::vec::Vec::len), Some(1));
    }

    #[test]
    fn domain_kind_round_trips_all_variants() {
        let variants = [
            DomainKind::DataClass,
            DomainKind::IntList,
            DomainKind::SampleSet,
            DomainKind::StudyDatasetDate,
            DomainKind::StudyDatasetVisit,
            DomainKind::Unknown,
            DomainKind::VarList,
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).expect("serialize domain kind");
            let restored: DomainKind =
                serde_json::from_str(&json).expect("deserialize domain kind");
            assert_eq!(restored, variant);
        }
    }

    #[test]
    fn domain_kind_serializes_to_expected_wire_values() {
        assert_eq!(
            serde_json::to_string(&DomainKind::DataClass).expect("serialize"),
            "\"DataClass\""
        );
        assert_eq!(
            serde_json::to_string(&DomainKind::IntList).expect("serialize"),
            "\"IntList\""
        );
        assert_eq!(
            serde_json::to_string(&DomainKind::SampleSet).expect("serialize"),
            "\"SampleSet\""
        );
        assert_eq!(
            serde_json::to_string(&DomainKind::StudyDatasetDate).expect("serialize"),
            "\"StudyDatasetDate\""
        );
        assert_eq!(
            serde_json::to_string(&DomainKind::StudyDatasetVisit).expect("serialize"),
            "\"StudyDatasetVisit\""
        );
        assert_eq!(
            serde_json::to_string(&DomainKind::Unknown).expect("serialize"),
            "\"Unknown\""
        );
        assert_eq!(
            serde_json::to_string(&DomainKind::VarList).expect("serialize"),
            "\"VarList\""
        );
    }

    fn domain_kind_variant_count(value: DomainKind) -> usize {
        match value {
            DomainKind::DataClass
            | DomainKind::IntList
            | DomainKind::SampleSet
            | DomainKind::StudyDatasetDate
            | DomainKind::StudyDatasetVisit
            | DomainKind::Unknown
            | DomainKind::VarList => 7,
        }
    }

    #[test]
    fn domain_kind_variant_count_regression() {
        assert_eq!(domain_kind_variant_count(DomainKind::DataClass), 7);
    }

    #[test]
    fn all_domain_endpoint_urls_match_expected_routes() {
        let client = test_client();
        let cases = [
            ("create", "createDomain.api"),
            ("get", "getDomain.api"),
            ("get_details", "getDomainDetails.api"),
            ("save", "saveDomain.api"),
            ("drop", "deleteDomain.api"),
            ("update", "updateDomain.api"),
            ("list", "listDomains.api"),
            ("validate", "validateNameExpressions.api"),
            ("name_previews", "getDomainNamePreviews.api"),
            ("get_properties", "getProperties.api"),
            ("property_usages", "propertyUsages.api"),
        ];

        for (_label, action) in cases {
            let url = client.build_url("property", action, None);
            assert_eq!(
                url.as_str(),
                format!("https://labkey.example.com/labkey/Project/Folder/property-{action}")
            );
        }
    }

    #[test]
    fn drop_domain_body_uses_expected_keys() {
        let body = DropDomainBody {
            schema_name: "study".to_string(),
            query_name: "StudyProperties".to_string(),
            domain_design: None,
            audit_user_comment: Some("cleanup".to_string()),
        };

        let json = serde_json::to_value(body).expect("serialize drop body");
        assert_eq!(json["schemaName"], "study");
        assert_eq!(json["queryName"], "StudyProperties");
        assert_eq!(json["auditUserComment"], "cleanup");
        assert!(json.get("domainDesign").is_none());
    }

    #[test]
    fn domain_design_and_field_use_uri_wire_keys() {
        let domain_json = serde_json::json!({
            "domainURI": "urn:lsid:test:domain-1",
            "fields": [
                {
                    "name": "Code",
                    "conceptURI": "urn:concept:1",
                    "propertyURI": "urn:property:1",
                    "ontologyURI": "urn:ontology:1",
                    "rangeURI": "string"
                }
            ]
        });

        let domain: DomainDesign =
            serde_json::from_value(domain_json).expect("deserialize with URI wire keys");
        assert_eq!(domain.domain_uri.as_deref(), Some("urn:lsid:test:domain-1"));
        let field = &domain.fields.as_ref().expect("fields")[0];
        assert_eq!(field.concept_uri.as_deref(), Some("urn:concept:1"));
        assert_eq!(field.property_uri.as_deref(), Some("urn:property:1"));
        assert_eq!(field.ontology_uri.as_deref(), Some("urn:ontology:1"));
        assert_eq!(field.range_uri.as_deref(), Some("string"));

        let serialized = serde_json::to_value(&domain).expect("serialize with URI wire keys");
        assert_eq!(serialized["domainURI"], "urn:lsid:test:domain-1");
        assert_eq!(serialized["fields"][0]["conceptURI"], "urn:concept:1");
        assert_eq!(serialized["fields"][0]["propertyURI"], "urn:property:1");
        assert_eq!(serialized["fields"][0]["ontologyURI"], "urn:ontology:1");
        assert_eq!(serialized["fields"][0]["rangeURI"], "string");
    }

    #[test]
    fn property_usages_envelope_extracts_data() {
        let response = serde_json::json!({
            "data": [
                {
                    "propertyId": 10,
                    "propertyURI": "urn:lsid:test:10",
                    "usageCount": 2,
                    "objects": [{"name": "obj-a"}]
                }
            ]
        });

        let usages = extract_property_usages(&response).expect("extract usage data");
        assert_eq!(usages.len(), 1);
        assert_eq!(usages[0].property_id, 10);
        assert_eq!(usages[0].usage_count, 2);
    }

    #[test]
    fn property_usages_missing_data_returns_unexpected_response() {
        let response = serde_json::json!({"success": true});
        let error = extract_property_usages(&response).expect_err("missing data should fail");
        match error {
            LabkeyError::UnexpectedResponse { status, text } => {
                assert_eq!(status, StatusCode::OK);
                assert!(text.contains("invalid propertyUsages response"));
            }
            other => panic!("expected unexpected response error, got {other:?}"),
        }
    }

    #[test]
    fn domain_field_deserializes_java_typed_fields() {
        let value = serde_json::json!({
            "name": "BloodPressure",
            "rangeURI": "int",
            "hidden": true,
            "PHI": "Limited",
            "measure": true,
            "dimension": false,
            "mvEnabled": true,
            "derivationDataScope": "ChildOnly",
            "conditionalFormats": [{
                "filter": "~gt=120",
                "textColor": "#FF0000",
                "backgroundColor": "#FFEEEE",
                "bold": true,
                "italic": false,
                "strikethrough": false
            }]
        });
        let field: DomainField = serde_json::from_value(value).expect("valid domain field");
        assert_eq!(field.hidden, Some(true));
        assert_eq!(field.phi.as_deref(), Some("Limited"));
        assert_eq!(field.measure, Some(true));
        assert_eq!(field.dimension, Some(false));
        assert_eq!(field.mv_enabled, Some(true));
        assert_eq!(field.derivation_data_scope.as_deref(), Some("ChildOnly"));
        assert_eq!(field.conditional_formats.len(), 1);
        let cf = &field.conditional_formats[0];
        assert_eq!(cf.filter.as_deref(), Some("~gt=120"));
        assert_eq!(cf.text_color.as_deref(), Some("#FF0000"));
        assert_eq!(cf.background_color.as_deref(), Some("#FFEEEE"));
        assert!(cf.bold);
        assert!(!cf.italic);
        assert!(!cf.strikethrough);
    }

    #[test]
    fn conditional_format_round_trips_serde() {
        let cf = ConditionalFormat {
            filter: Some("~eq=100".to_string()),
            text_color: Some("#000000".to_string()),
            background_color: None,
            bold: false,
            italic: true,
            strikethrough: false,
        };
        let json = serde_json::to_value(&cf).expect("serialize");
        let restored: ConditionalFormat = serde_json::from_value(json).expect("deserialize");
        assert_eq!(restored.filter.as_deref(), Some("~eq=100"));
        assert!(restored.italic);
        assert!(!restored.bold);
    }

    #[test]
    fn get_properties_body_serializes_property_uris_with_acronym_casing() {
        let body = GetPropertiesBody {
            domain_ids: None,
            domain_kinds: None,
            filters: None,
            max_rows: None,
            offset: None,
            property_ids: None,
            property_uris: Some(vec!["urn:test".to_string()]),
            search: None,
            sort: None,
        };
        let json = serde_json::to_value(body).expect("serialize get_properties body");
        assert!(
            json.get("propertyURIs").is_some(),
            "expected key propertyURIs (acronym casing), got keys: {:?}",
            json.as_object().map(|o| o.keys().collect::<Vec<_>>())
        );
        assert!(
            json.get("propertyUris").is_none(),
            "camelCase propertyUris must not be emitted"
        );
        assert_eq!(json["propertyURIs"], serde_json::json!(["urn:test"]));
    }

    #[test]
    fn validate_name_expressions_body_serializes_include_name_preview() {
        let body_with = ValidateNameExpressionsBody {
            domain_design: None,
            options: None,
            kind: None,
            include_name_preview: Some(true),
        };
        let json = serde_json::to_value(body_with).expect("serialize");
        assert_eq!(json["includeNamePreview"], serde_json::json!(true));

        let body_without = ValidateNameExpressionsBody {
            domain_design: None,
            options: None,
            kind: None,
            include_name_preview: None,
        };
        let json = serde_json::to_value(body_without).expect("serialize");
        assert!(
            json.get("includeNamePreview").is_none(),
            "includeNamePreview must be omitted when None"
        );

        let body_false = ValidateNameExpressionsBody {
            domain_design: None,
            options: None,
            kind: None,
            include_name_preview: Some(false),
        };
        let json = serde_json::to_value(body_false).expect("serialize");
        assert_eq!(
            json["includeNamePreview"],
            serde_json::json!(false),
            "explicit false must be emitted, not skipped"
        );
    }

    #[test]
    fn get_properties_body_omits_property_uris_when_none() {
        let body = GetPropertiesBody {
            domain_ids: None,
            domain_kinds: None,
            filters: None,
            max_rows: None,
            offset: None,
            property_ids: None,
            property_uris: None,
            search: None,
            sort: None,
        };
        let json = serde_json::to_value(body).expect("serialize empty get_properties body");
        assert!(
            json.get("propertyURIs").is_none(),
            "propertyURIs must be omitted when None"
        );
        assert!(
            json.get("propertyUris").is_none(),
            "buggy camelCase key must never appear"
        );
    }

    #[tokio::test]
    async fn validate_name_expressions_passes_include_name_preview_to_request_body() {
        let client = test_client();
        let options = ValidateNameExpressionsOptions::builder()
            .include_name_preview(true)
            .build();

        let url = client.build_url(
            "property",
            "validateNameExpressions.api",
            options.container_path.as_deref(),
        );
        let body = ValidateNameExpressionsBody {
            domain_design: options.domain_design,
            options: options.options,
            kind: options.kind,
            include_name_preview: options.include_name_preview,
        };
        let json = serde_json::to_value(&body).expect("serialize");

        assert_eq!(json["includeNamePreview"], serde_json::json!(true));
        assert!(
            url.as_str()
                .contains("property-validateNameExpressions.api"),
            "endpoint URL must target validateNameExpressions"
        );
    }
}
