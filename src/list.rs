//! List convenience API built on domain creation.

use std::{collections::HashMap, time::Duration};

use crate::{
    client::LabkeyClient,
    domain::{CreateDomainOptions, DomainDesign, DomainField, DomainIndex, DomainKind},
    error::LabkeyError,
};

/// List key type used when creating list domains.
///
/// Maps to the JS client's `keyType` parameter in `List.create`. `IntList`
/// and `VarList` correspond to the two domain kinds, while
/// `AutoIncrementInteger` produces an `IntList` domain with an explicit
/// `keyType: "AutoIncrementInteger"` in the domain options.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ListKeyType {
    /// Integer-keyed list (`IntList`).
    IntList,
    /// String-keyed list (`VarList`). Injects `keyType: "Varchar"` into
    /// domain options.
    VarList,
    /// Auto-increment integer-keyed list. Uses the `IntList` domain kind
    /// with `keyType: "AutoIncrementInteger"` in domain options.
    AutoIncrementInteger,
}

impl ListKeyType {
    const fn as_domain_kind(self) -> DomainKind {
        match self {
            Self::IntList | Self::AutoIncrementInteger => DomainKind::IntList,
            Self::VarList => DomainKind::VarList,
        }
    }

    /// Returns the `keyType` value to inject into domain options, if any.
    /// `IntList` uses the server default and needs no explicit key type.
    const fn key_type_value(self) -> Option<&'static str> {
        match self {
            Self::IntList => None,
            Self::VarList => Some("Varchar"),
            Self::AutoIncrementInteger => Some("AutoIncrementInteger"),
        }
    }
}

/// Options for [`LabkeyClient::create_list`].
///
/// For simple lists, use the shorthand `description`, `fields`, and `indices`
/// fields directly — they are folded into a [`DomainDesign`] automatically.
/// For full control, provide an explicit `domain_design` instead. The two
/// approaches are mutually exclusive: providing both `domain_design` and any
/// shorthand field is rejected at validation time.
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct CreateListOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Optional base domain design for full control over the domain
    /// configuration. Mutually exclusive with the `description`, `fields`,
    /// and `indices` shorthand fields.
    pub domain_design: Option<DomainDesign>,
    /// Shorthand: list description, folded into the domain design when no
    /// explicit `domain_design` is provided.
    pub description: Option<String>,
    /// Shorthand: field definitions, folded into the domain design when no
    /// explicit `domain_design` is provided.
    pub fields: Option<Vec<DomainField>>,
    /// Shorthand: index definitions, folded into the domain design when no
    /// explicit `domain_design` is provided.
    pub indices: Option<Vec<DomainIndex>>,
    /// Name for the new list domain.
    pub name: String,
    /// Key field name (`options.keyName` in the delegated domain payload).
    pub key_name: String,
    /// List key type mapped to domain kind.
    pub key_type: ListKeyType,
    /// Optional request timeout override.
    pub timeout: Option<Duration>,
}

fn create_list_options_json(key_name: String, key_type: ListKeyType) -> serde_json::Value {
    let mut options = serde_json::Map::new();
    options.insert("keyName".to_string(), serde_json::Value::String(key_name));
    if let Some(kt) = key_type.key_type_value() {
        options.insert(
            "keyType".to_string(),
            serde_json::Value::String(kt.to_string()),
        );
    }
    serde_json::Value::Object(options)
}

fn map_create_list_to_create_domain_options(options: CreateListOptions) -> CreateDomainOptions {
    let domain_design = if let Some(design) = options.domain_design {
        // Explicit domain design takes full control — shorthands were already
        // rejected by validation if present alongside it.
        design
    } else {
        // Build a domain design from the name and any shorthand fields.
        DomainDesign {
            domain_id: None,
            domain_uri: None,
            name: Some(options.name),
            description: options.description,
            schema_name: None,
            query_name: None,
            fields: options.fields,
            indices: options.indices,
            extra: HashMap::new(),
        }
    };

    CreateDomainOptions::builder()
        .maybe_container_path(options.container_path)
        .domain_design(domain_design)
        .kind(options.key_type.as_domain_kind())
        .options(create_list_options_json(options.key_name, options.key_type))
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
    let has_shorthand =
        options.description.is_some() || options.fields.is_some() || options.indices.is_some();
    if options.domain_design.is_some() && has_shorthand {
        return Err(LabkeyError::InvalidInput(
            "create_list does not allow shorthand fields (description, fields, indices) \
             when an explicit domain_design is provided — use one approach or the other"
                .to_string(),
        ));
    }
    Ok(())
}

impl LabkeyClient {
    /// Create a list by delegating to [`LabkeyClient::create_domain`].
    ///
    /// This convenience method maps [`ListKeyType`] to the corresponding
    /// [`crate::domain::DomainKind`] and passes `options.keyName` through
    /// domain-creation options. If a `domain_design` is provided, it is used
    /// as-is (including its name); otherwise a default design is created
    /// using `options.name`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if list inputs are invalid or the delegated
    /// domain-creation request fails.
    ///
    /// # Examples
    ///
    /// Minimal list with no fields (server creates the key column):
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
    ///
    /// Using shorthand fields to define columns inline:
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
    /// // Fields can be provided as raw JSON when you don't need the full
    /// // DomainField struct — the server fills in defaults for omitted
    /// // properties.
    /// let fields: Vec<labkey_rs::domain::DomainField> = serde_json::from_value(
    ///     serde_json::json!([
    ///         { "name": "name", "rangeURI": "string" },
    ///         { "name": "slogan", "rangeURI": "multiLine" },
    ///     ])
    /// ).unwrap();
    ///
    /// let _ = client
    ///     .create_list(
    ///         CreateListOptions::builder()
    ///             .name("Teams".to_string())
    ///             .key_name("rowId".to_string())
    ///             .key_type(ListKeyType::AutoIncrementInteger)
    ///             .description("Teams in the league".to_string())
    ///             .fields(fields)
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
            ListKeyType::IntList | ListKeyType::VarList | ListKeyType::AutoIncrementInteger => 3,
        }
    }

    #[test]
    fn list_key_type_variant_count_regression() {
        assert_eq!(list_key_type_variant_count(ListKeyType::IntList), 3);
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
    fn create_list_gives_domain_design_name_precedence_over_options_name() {
        // Matches JS client List.spec.ts "should give domainDesign precedence":
        // when a domainDesign with its own name is provided, that name wins
        // and options.name is ignored.
        let mapped = map_create_list_to_create_domain_options(
            CreateListOptions::builder()
                .name("OptionsName".to_string())
                .key_name("Name".to_string())
                .key_type(ListKeyType::VarList)
                .domain_design(DomainDesign {
                    domain_id: Some(1),
                    domain_uri: None,
                    name: Some("DomainDesignName".to_string()),
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
            Some("DomainDesignName".to_string())
        );
    }

    #[test]
    fn create_list_falls_back_to_options_name_when_no_domain_design_provided() {
        let mapped = map_create_list_to_create_domain_options(
            CreateListOptions::builder()
                .name("FallbackName".to_string())
                .key_name("RowId".to_string())
                .key_type(ListKeyType::IntList)
                .build(),
        );

        assert_eq!(
            mapped.domain_design.and_then(|value| value.name),
            Some("FallbackName".to_string())
        );
    }

    #[test]
    fn create_list_preserves_domain_design_with_none_name() {
        // When a domain_design is provided but its name is None, the
        // options.name is NOT injected — the design is used as-is.
        let mapped = map_create_list_to_create_domain_options(
            CreateListOptions::builder()
                .name("OptionsName".to_string())
                .key_name("RowId".to_string())
                .key_type(ListKeyType::IntList)
                .domain_design(DomainDesign {
                    domain_id: None,
                    domain_uri: None,
                    name: None,
                    description: Some("custom design".to_string()),
                    schema_name: None,
                    query_name: None,
                    fields: None,
                    indices: None,
                    extra: HashMap::new(),
                })
                .build(),
        );

        // The domain design's None name is preserved, not overwritten.
        assert!(
            mapped
                .domain_design
                .as_ref()
                .and_then(|value| value.name.as_ref())
                .is_none()
        );
        // But the description from the provided design survives.
        assert_eq!(
            mapped
                .domain_design
                .as_ref()
                .and_then(|value| value.description.as_ref())
                .map(String::as_str),
            Some("custom design")
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

    #[test]
    fn var_list_injects_key_type_varchar_into_options() {
        let mapped = map_create_list_to_create_domain_options(
            CreateListOptions::builder()
                .name("StringList".to_string())
                .key_name("Name".to_string())
                .key_type(ListKeyType::VarList)
                .build(),
        );

        assert_eq!(mapped.kind, Some(DomainKind::VarList));
        assert_eq!(
            mapped.options,
            Some(serde_json::json!({
                "keyName": "Name",
                "keyType": "Varchar"
            }))
        );
    }

    #[test]
    fn auto_increment_integer_maps_to_int_list_domain_with_key_type() {
        let mapped = map_create_list_to_create_domain_options(
            CreateListOptions::builder()
                .name("AutoList".to_string())
                .key_name("RowId".to_string())
                .key_type(ListKeyType::AutoIncrementInteger)
                .build(),
        );

        assert_eq!(mapped.kind, Some(DomainKind::IntList));
        assert_eq!(
            mapped.options,
            Some(serde_json::json!({
                "keyName": "RowId",
                "keyType": "AutoIncrementInteger"
            }))
        );
    }

    #[test]
    fn int_list_does_not_inject_key_type() {
        let mapped = map_create_list_to_create_domain_options(
            CreateListOptions::builder()
                .name("IntegerList".to_string())
                .key_name("RowId".to_string())
                .key_type(ListKeyType::IntList)
                .build(),
        );

        assert_eq!(mapped.kind, Some(DomainKind::IntList));
        // IntList relies on the server default — no keyType in options
        assert_eq!(
            mapped.options,
            Some(serde_json::json!({
                "keyName": "RowId"
            }))
        );
    }

    #[test]
    fn shorthand_description_folds_into_domain_design() {
        let mapped = map_create_list_to_create_domain_options(
            CreateListOptions::builder()
                .name("Teams".to_string())
                .key_name("RowId".to_string())
                .key_type(ListKeyType::IntList)
                .description("Teams in the league".to_string())
                .build(),
        );

        let design = mapped.domain_design.expect("should have a domain design");
        assert_eq!(design.name.as_deref(), Some("Teams"));
        assert_eq!(design.description.as_deref(), Some("Teams in the league"));
    }

    #[test]
    fn shorthand_fields_fold_into_domain_design() {
        let fields: Vec<crate::domain::DomainField> = serde_json::from_value(serde_json::json!([
            { "name": "name", "rangeURI": "string" },
            { "name": "slogan", "rangeURI": "multiLine" },
        ]))
        .expect("test field JSON should deserialize");

        let mapped = map_create_list_to_create_domain_options(
            CreateListOptions::builder()
                .name("Teams".to_string())
                .key_name("RowId".to_string())
                .key_type(ListKeyType::IntList)
                .fields(fields)
                .build(),
        );

        let design = mapped.domain_design.expect("should have a domain design");
        let fields = design.fields.expect("domain design should have fields");
        assert_eq!(fields.len(), 2);
        assert_eq!(fields[0].name.as_deref(), Some("name"));
        assert_eq!(fields[1].name.as_deref(), Some("slogan"));
    }

    #[test]
    fn shorthand_indices_fold_into_domain_design() {
        let indices: Vec<crate::domain::DomainIndex> = serde_json::from_value(serde_json::json!([
            { "columnNames": ["name"], "unique": true },
        ]))
        .expect("test index JSON should deserialize");

        let mapped = map_create_list_to_create_domain_options(
            CreateListOptions::builder()
                .name("Teams".to_string())
                .key_name("RowId".to_string())
                .key_type(ListKeyType::IntList)
                .indices(indices)
                .build(),
        );

        let design = mapped.domain_design.expect("should have a domain design");
        let indices = design.indices.expect("domain design should have indices");
        assert_eq!(indices.len(), 1);
    }

    #[test]
    fn validation_rejects_domain_design_with_shorthand_fields() {
        let options = CreateListOptions::builder()
            .name("Teams".to_string())
            .key_name("RowId".to_string())
            .key_type(ListKeyType::IntList)
            .domain_design(DomainDesign {
                domain_id: None,
                domain_uri: None,
                name: Some("Teams".to_string()),
                description: None,
                schema_name: None,
                query_name: None,
                fields: None,
                indices: None,
                extra: HashMap::new(),
            })
            .description("should conflict".to_string())
            .build();

        let result = validate_create_list_options(&options);
        assert!(
            matches!(&result, Err(LabkeyError::InvalidInput(msg)) if msg.contains("shorthand")),
            "should reject domain_design + shorthand combination: {result:?}"
        );
    }

    #[test]
    fn validation_rejects_domain_design_with_shorthand_fields_via_fields() {
        let fields: Vec<crate::domain::DomainField> = serde_json::from_value(serde_json::json!([
            { "name": "x" },
        ]))
        .expect("test field JSON should deserialize");

        let options = CreateListOptions::builder()
            .name("Teams".to_string())
            .key_name("RowId".to_string())
            .key_type(ListKeyType::IntList)
            .domain_design(DomainDesign {
                domain_id: None,
                domain_uri: None,
                name: Some("Teams".to_string()),
                description: None,
                schema_name: None,
                query_name: None,
                fields: None,
                indices: None,
                extra: HashMap::new(),
            })
            .fields(fields)
            .build();

        let result = validate_create_list_options(&options);
        assert!(
            matches!(&result, Err(LabkeyError::InvalidInput(msg)) if msg.contains("shorthand")),
            "should reject domain_design + fields combination: {result:?}"
        );
    }

    #[test]
    fn validation_accepts_domain_design_without_shorthands() {
        let options = CreateListOptions::builder()
            .name("Teams".to_string())
            .key_name("RowId".to_string())
            .key_type(ListKeyType::IntList)
            .domain_design(DomainDesign {
                domain_id: None,
                domain_uri: None,
                name: Some("Teams".to_string()),
                description: None,
                schema_name: None,
                query_name: None,
                fields: None,
                indices: None,
                extra: HashMap::new(),
            })
            .build();

        assert!(
            validate_create_list_options(&options).is_ok(),
            "domain_design alone should be valid"
        );
    }
}
