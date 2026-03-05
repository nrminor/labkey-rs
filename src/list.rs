//! List convenience API built on domain creation.

use std::{collections::HashMap, time::Duration};

use crate::{
    client::LabkeyClient,
    domain::{CreateDomainOptions, DomainDesign, DomainKind},
    error::LabkeyError,
};

/// List key type used when creating list domains.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ListKeyType {
    /// Integer-keyed list (`IntList`).
    IntList,
    /// String-keyed list (`VarList`).
    VarList,
}

impl ListKeyType {
    const fn as_domain_kind(self) -> DomainKind {
        match self {
            Self::IntList => DomainKind::IntList,
            Self::VarList => DomainKind::VarList,
        }
    }
}

/// Options for [`LabkeyClient::create_list`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct CreateListOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Optional base domain design to extend.
    pub domain_design: Option<DomainDesign>,
    /// Name for the new list domain.
    pub name: String,
    /// Key field name (`options.keyName` in the delegated domain payload).
    pub key_name: String,
    /// List key type mapped to domain kind.
    pub key_type: ListKeyType,
    /// Optional request timeout override.
    pub timeout: Option<Duration>,
}

fn default_domain_design(name: String) -> DomainDesign {
    DomainDesign {
        domain_id: None,
        domain_uri: None,
        name: Some(name),
        description: None,
        schema_name: None,
        query_name: None,
        fields: None,
        indices: None,
        extra: HashMap::new(),
    }
}

fn create_list_options_json(key_name: String) -> serde_json::Value {
    let mut options = serde_json::Map::new();
    options.insert("keyName".to_string(), serde_json::Value::String(key_name));
    serde_json::Value::Object(options)
}

fn map_create_list_to_create_domain_options(options: CreateListOptions) -> CreateDomainOptions {
    let mut domain_design = options
        .domain_design
        .unwrap_or_else(|| default_domain_design(options.name.clone()));
    domain_design.name = Some(options.name);

    CreateDomainOptions::builder()
        .maybe_container_path(options.container_path)
        .domain_design(domain_design)
        .kind(options.key_type.as_domain_kind())
        .options(create_list_options_json(options.key_name))
        .maybe_timeout(options.timeout)
        .build()
}

fn validate_create_list_options(options: &CreateListOptions) -> Result<(), LabkeyError> {
    if options.name.trim().is_empty() {
        return Err(LabkeyError::InvalidInput(
            "create_list requires non-empty name".to_string(),
        ));
    }
    if options.key_name.trim().is_empty() {
        return Err(LabkeyError::InvalidInput(
            "create_list requires non-empty key_name".to_string(),
        ));
    }
    Ok(())
}

impl LabkeyClient {
    /// Create a list by delegating to [`LabkeyClient::create_domain`].
    ///
    /// This convenience method maps [`ListKeyType`] to the corresponding
    /// [`crate::domain::DomainKind`], sets the delegated domain-design name,
    /// and passes `options.keyName` through domain-creation options.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if list inputs are invalid or the delegated
    /// domain-creation request fails.
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
    /// use labkey_rs::list::{CreateListOptions, ListKeyType};
    ///
    /// let _ = client
    ///     .create_list(
    ///         CreateListOptions::builder()
    ///             .name("StudyList".to_string())
    ///             .key_name("RowId".to_string())
    ///             .key_type(ListKeyType::IntList)
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_list(
        &self,
        options: CreateListOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        validate_create_list_options(&options)?;
        self.create_domain(map_create_list_to_create_domain_options(options))
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DomainKind;

    fn list_key_type_variant_count(value: ListKeyType) -> usize {
        match value {
            ListKeyType::IntList | ListKeyType::VarList => 2,
        }
    }

    #[test]
    fn list_key_type_variant_count_regression() {
        assert_eq!(list_key_type_variant_count(ListKeyType::IntList), 2);
    }

    #[test]
    fn create_list_delegation_maps_int_list_expected_fields() {
        let mapped = map_create_list_to_create_domain_options(
            CreateListOptions::builder()
                .name("MyList".to_string())
                .key_name("RowId".to_string())
                .key_type(ListKeyType::IntList)
                .build(),
        );

        assert_eq!(mapped.kind, Some(DomainKind::IntList));
        assert_eq!(
            mapped.domain_design.and_then(|value| value.name),
            Some("MyList".to_string())
        );
        assert_eq!(
            mapped.options,
            Some(serde_json::json!({
                "keyName": "RowId"
            }))
        );
    }

    #[test]
    fn create_list_delegation_maps_var_list_and_overrides_domain_name() {
        let mapped = map_create_list_to_create_domain_options(
            CreateListOptions::builder()
                .name("OverrideName".to_string())
                .key_name("Name".to_string())
                .key_type(ListKeyType::VarList)
                .domain_design(DomainDesign {
                    domain_id: Some(1),
                    domain_uri: None,
                    name: Some("OriginalName".to_string()),
                    description: None,
                    schema_name: None,
                    query_name: None,
                    fields: None,
                    indices: None,
                    extra: HashMap::new(),
                })
                .build(),
        );

        assert_eq!(mapped.kind, Some(DomainKind::VarList));
        assert_eq!(
            mapped.domain_design.and_then(|value| value.name),
            Some("OverrideName".to_string())
        );
    }

    #[test]
    fn create_list_rejects_blank_name_or_key_name() {
        let blank_name = CreateListOptions::builder()
            .name("   ".to_string())
            .key_name("RowId".to_string())
            .key_type(ListKeyType::IntList)
            .build();
        let blank_key_name = CreateListOptions::builder()
            .name("MyList".to_string())
            .key_name("\t".to_string())
            .key_type(ListKeyType::IntList)
            .build();

        assert!(matches!(
            validate_create_list_options(&blank_name),
            Err(LabkeyError::InvalidInput(message)) if message.contains("name")
        ));
        assert!(matches!(
            validate_create_list_options(&blank_key_name),
            Err(LabkeyError::InvalidInput(message)) if message.contains("key_name")
        ));
    }
}
