//! Experiment models and lineage endpoints.

use std::collections::HashMap;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{client::LabkeyClient, common::opt, error::LabkeyError};

/// Experiment entity type filter values used by lineage APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ExpType {
    /// Data entities.
    #[serde(rename = "Data")]
    Data,
    /// Material entities.
    #[serde(rename = "Material")]
    Material,
    /// Experiment run entities.
    #[serde(rename = "ExperimentRun")]
    ExperimentRun,
}

impl ExpType {
    const fn as_wire(self) -> &'static str {
        match self {
            Self::Data => "Data",
            Self::Material => "Material",
            Self::ExperimentRun => "ExperimentRun",
        }
    }
}

/// Sequence type values used by experiment entity sequence endpoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SeqType {
    /// Generated id sequence.
    #[serde(rename = "genId")]
    GenId,
    /// Root sample count sequence.
    #[serde(rename = "rootSampleCount")]
    RootSampleCount,
    /// Sample count sequence.
    #[serde(rename = "sampleCount")]
    SampleCount,
}

/// Primary-key filter element for experiment entities.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PkFilter {
    /// Field key used for filtering.
    pub field_key: String,
    /// Filter value.
    pub value: serde_json::Value,
}

/// Common experiment object fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ExpObject {
    /// User-entered comment.
    #[serde(default)]
    pub comment: Option<String>,
    /// Container id or path-like identifier.
    #[serde(default)]
    pub container: Option<String>,
    /// Full container path.
    #[serde(default)]
    pub container_path: Option<String>,
    /// CPAS type identifier.
    #[serde(default)]
    pub cpas_type: Option<String>,
    /// Creation timestamp.
    #[serde(default)]
    pub created: Option<String>,
    /// Creating user display name.
    #[serde(default)]
    pub created_by: Option<String>,
    /// Entity id.
    #[serde(default)]
    pub id: Option<i64>,
    /// Entity LSID.
    #[serde(default)]
    pub lsid: Option<String>,
    /// Modification timestamp.
    #[serde(default)]
    pub modified: Option<String>,
    /// Last modifying user display name.
    #[serde(default)]
    pub modified_by: Option<String>,
    /// Entity name.
    #[serde(default)]
    pub name: Option<String>,
    /// Primary-key filters for this entity.
    #[serde(default)]
    pub pk_filters: Vec<PkFilter>,
    /// Query name used for this entity type.
    #[serde(default)]
    pub query_name: Option<String>,
    /// Whether this entity is access-restricted.
    #[serde(default)]
    pub restricted: Option<bool>,
    /// Schema name used for this entity type.
    #[serde(default)]
    pub schema_name: Option<String>,
    /// Server-provided type label.
    #[serde(rename = "type")]
    #[serde(default)]
    pub type_: Option<String>,
    /// URL for opening this entity.
    #[serde(default)]
    pub url: Option<String>,
    /// Additional properties attached to this object.
    #[serde(default)]
    pub properties: Option<serde_json::Value>,
}

/// Reference to an associated data class.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DataClassRef {
    /// Data class id.
    pub id: i64,
    /// Data class name.
    pub name: String,
}

/// Reference to an associated sample set.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SampleSetRef {
    /// Sample set id.
    pub id: i64,
    /// Sample set name.
    pub name: String,
}

/// Experiment data object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ExpData {
    /// Common experiment object fields.
    #[serde(flatten)]
    pub exp_object: ExpObject,
    /// Associated data class.
    #[serde(default)]
    pub data_class: Option<DataClassRef>,
    /// Data file URL.
    #[serde(default)]
    pub data_file_url: Option<String>,
    /// Data type.
    #[serde(default)]
    pub data_type: Option<String>,
    /// Pipeline path.
    #[serde(default)]
    pub pipeline_path: Option<String>,
    /// Data role label.
    #[serde(default)]
    pub role: Option<String>,
}

/// Experiment material object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Material {
    /// Common experiment object fields.
    #[serde(flatten)]
    pub exp_object: ExpObject,
    /// Associated sample set.
    #[serde(default)]
    pub sample_set: Option<SampleSetRef>,
}

/// Experiment run object.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Run {
    /// Common experiment object fields.
    #[serde(flatten)]
    pub exp_object: ExpObject,
    /// Input data entities.
    #[serde(default)]
    pub data_inputs: Vec<ExpData>,
    /// Output data entities.
    #[serde(default)]
    pub data_outputs: Vec<ExpData>,
    /// Input material entities.
    #[serde(default)]
    pub material_inputs: Vec<Material>,
    /// Output material entities.
    #[serde(default)]
    pub material_outputs: Vec<Material>,
}

/// Group of experiment runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RunGroup {
    /// Common experiment object fields.
    #[serde(flatten)]
    pub exp_object: ExpObject,
    /// Batch protocol id.
    #[serde(default)]
    pub batch_protocol_id: Option<i64>,
    /// Whether this run group is hidden.
    #[serde(default)]
    pub hidden: Option<bool>,
    /// Runs in this group.
    #[serde(default)]
    pub runs: Vec<Run>,
}

/// Parent-child edge in a lineage graph.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LineageEdge {
    /// Target node LSID.
    pub lsid: String,
    /// Relationship role.
    pub role: String,
}

/// Node in a lineage graph.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LineageNode {
    /// Common experiment object fields.
    #[serde(flatten)]
    pub exp_object: ExpObject,
    /// Absolute path of this node.
    #[serde(default)]
    pub absolute_path: Option<String>,
    /// Child edges from this node.
    #[serde(default)]
    pub children: Vec<LineageEdge>,
    /// Data file URL when present.
    #[serde(default)]
    pub data_file_url: Option<String>,
    /// Graph distance from seed.
    #[serde(default)]
    pub distance: Option<i64>,
    /// URL for a list/details page.
    #[serde(default)]
    pub list_url: Option<String>,
    /// Parent edges for this node.
    #[serde(default)]
    pub parents: Vec<LineageEdge>,
    /// Pipeline path when present.
    #[serde(default)]
    pub pipeline_path: Option<String>,
    /// Protocol attached to this node when present.
    #[serde(default)]
    pub protocol: Option<ExpObject>,
    /// Optional run steps included by converter options.
    #[serde(default)]
    pub steps: Option<Vec<Run>>,
    /// Input data entities.
    #[serde(default)]
    pub data_inputs: Vec<ExpData>,
    /// Output data entities.
    #[serde(default)]
    pub data_outputs: Vec<ExpData>,
    /// Input material entities.
    #[serde(default)]
    pub material_inputs: Vec<Material>,
    /// Output material entities.
    #[serde(default)]
    pub material_outputs: Vec<Material>,
    /// Additional server-provided keys.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Response from [`LabkeyClient::lineage`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LineageResponse {
    /// Singular seed LSID for deprecated single-`lsid` requests.
    #[serde(default)]
    pub seed: Option<String>,
    /// Seed LSIDs for the lineage response.
    #[serde(default)]
    pub seeds: Vec<String>,
    /// Lineage nodes keyed by LSID.
    pub nodes: HashMap<String, LineageNode>,
}

/// Response from [`LabkeyClient::resolve`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResolveResponse {
    /// Resolved lineage nodes.
    pub data: Vec<LineageNode>,
}

/// Options for [`LabkeyClient::lineage`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct LineageOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Include parent nodes.
    pub parents: Option<bool>,
    /// Include child nodes.
    pub children: Option<bool>,
    /// Optional graph depth.
    pub depth: Option<i64>,
    /// Optional experiment entity type filter.
    pub exp_type: Option<ExpType>,
    /// Optional CPAS type filter.
    pub cpas_type: Option<String>,
    /// Optional run protocol LSID filter.
    pub run_protocol_lsid: Option<String>,
    /// Deprecated single seed LSID.
    pub lsid: Option<String>,
    /// Seed LSIDs. When provided, these are emitted as repeated `lsid` params.
    pub lsids: Option<Vec<String>>,
    /// Include run and step inputs and outputs.
    pub include_inputs_and_outputs: Option<bool>,
    /// Include experiment object properties.
    pub include_properties: Option<bool>,
    /// Include run steps.
    pub include_run_steps: Option<bool>,
}

/// Options for [`LabkeyClient::resolve`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct ResolveOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// LSIDs to resolve, emitted as repeated `lsid` query parameters.
    pub lsids: Option<Vec<String>>,
    /// Include run and step inputs and outputs.
    pub include_inputs_and_outputs: Option<bool>,
    /// Include experiment object properties.
    pub include_properties: Option<bool>,
    /// Include run steps.
    pub include_run_steps: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResolveEnvelope {
    data: Vec<LineageNode>,
}

fn build_lineage_params(options: &LineageOptions) -> Vec<(String, String)> {
    let mut params: Vec<(String, String)> = [
        opt(
            "includeInputsAndOutputs",
            options.include_inputs_and_outputs,
        ),
        opt("includeProperties", options.include_properties),
        opt("includeRunSteps", options.include_run_steps),
        opt("parents", options.parents),
        opt("children", options.children),
        opt("depth", options.depth),
        options
            .exp_type
            .map(|value| ("expType".to_string(), value.as_wire().to_string())),
        opt("cpasType", options.cpas_type.clone()),
        opt("runProtocolLsid", options.run_protocol_lsid.clone()),
    ]
    .into_iter()
    .flatten()
    .collect();

    if let Some(lsids) = options.lsids.as_ref() {
        params.extend(
            lsids
                .iter()
                .cloned()
                .map(|value| ("lsid".to_string(), value)),
        );
    } else if let Some(lsid) = options.lsid.as_ref() {
        params.push(("lsid".to_string(), lsid.clone()));
    }

    params
}

fn build_resolve_params(options: &ResolveOptions) -> Vec<(String, String)> {
    let mut params: Vec<(String, String)> = [
        opt(
            "includeInputsAndOutputs",
            options.include_inputs_and_outputs,
        ),
        opt("includeProperties", options.include_properties),
        opt("includeRunSteps", options.include_run_steps),
    ]
    .into_iter()
    .flatten()
    .collect();

    if let Some(lsids) = options.lsids.as_ref() {
        params.extend(
            lsids
                .iter()
                .cloned()
                .map(|value| ("lsid".to_string(), value)),
        );
    }

    params
}

fn extract_resolve_response(response: &serde_json::Value) -> Result<ResolveResponse, LabkeyError> {
    serde_json::from_value::<ResolveEnvelope>(response.clone())
        .map(|envelope| ResolveResponse {
            data: envelope.data,
        })
        .map_err(|_| LabkeyError::UnexpectedResponse {
            status: StatusCode::OK,
            text: format!("invalid resolve response: {response}"),
        })
}

fn validate_lineage_options(options: &LineageOptions) -> Result<(), LabkeyError> {
    match (&options.lsid, &options.lsids) {
        (Some(_), Some(_)) => {
            return Err(LabkeyError::InvalidInput(
                "lineage requires exactly one of `lsid` or `lsids`".to_string(),
            ));
        }
        (None, None) => {
            return Err(LabkeyError::InvalidInput(
                "lineage requires one of `lsid` or `lsids`".to_string(),
            ));
        }
        (Some(lsid), None) if lsid.trim().is_empty() => {
            return Err(LabkeyError::InvalidInput(
                "lineage `lsid` must not be blank".to_string(),
            ));
        }
        (None, Some(lsids)) if lsids.is_empty() => {
            return Err(LabkeyError::InvalidInput(
                "lineage `lsids` must not be empty".to_string(),
            ));
        }
        (None, Some(lsids)) if lsids.iter().any(|value| value.trim().is_empty()) => {
            return Err(LabkeyError::InvalidInput(
                "lineage `lsids` entries must not be blank".to_string(),
            ));
        }
        _ => {}
    }

    Ok(())
}

fn validate_resolve_options(options: &ResolveOptions) -> Result<(), LabkeyError> {
    let Some(lsids) = options.lsids.as_ref() else {
        return Err(LabkeyError::InvalidInput(
            "resolve requires `lsids`".to_string(),
        ));
    };

    if lsids.is_empty() {
        return Err(LabkeyError::InvalidInput(
            "resolve `lsids` must not be empty".to_string(),
        ));
    }
    if lsids.iter().any(|value| value.trim().is_empty()) {
        return Err(LabkeyError::InvalidInput(
            "resolve `lsids` entries must not be blank".to_string(),
        ));
    }

    Ok(())
}

impl LabkeyClient {
    /// Get parent and child lineage for experiment entities through `experiment-lineage.api`.
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
    /// use labkey_rs::experiment::LineageOptions;
    ///
    /// let _ = client
    ///     .lineage(
    ///         LineageOptions::builder()
    ///             .lsids(vec!["urn:lsid:test:run-1".to_string()])
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn lineage(&self, options: LineageOptions) -> Result<LineageResponse, LabkeyError> {
        validate_lineage_options(&options)?;
        let url = self.build_url(
            "experiment",
            "lineage.api",
            options.container_path.as_deref(),
        );
        let params = build_lineage_params(&options);
        self.get(url, &params).await
    }

    /// Resolve experiment entities by LSID through `experiment-resolve.api`.
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
    /// use labkey_rs::experiment::ResolveOptions;
    ///
    /// let _ = client
    ///     .resolve(
    ///         ResolveOptions::builder()
    ///             .lsids(vec!["urn:lsid:test:data-1".to_string()])
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn resolve(&self, options: ResolveOptions) -> Result<ResolveResponse, LabkeyError> {
        validate_resolve_options(&options)?;
        let url = self.build_url(
            "experiment",
            "resolve.api",
            options.container_path.as_deref(),
        );
        let params = build_resolve_params(&options);
        let response: serde_json::Value = self.get(url, &params).await?;
        extract_resolve_response(&response)
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
        })
        .expect("valid test client")
    }

    #[test]
    fn lineage_node_deserializes_nested_parents_and_children() {
        let node_json = serde_json::json!({
            "lsid": "urn:lsid:test:node-1",
            "name": "node-1",
            "parents": [
                { "lsid": "urn:lsid:test:parent-1", "role": "input" }
            ],
            "children": [
                { "lsid": "urn:lsid:test:child-1", "role": "output" }
            ]
        });

        let node: LineageNode =
            serde_json::from_value(node_json).expect("deserialize lineage node");
        assert_eq!(node.parents.len(), 1);
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.parents[0].role, "input");
        assert_eq!(node.children[0].role, "output");
        assert_eq!(node.exp_object.name.as_deref(), Some("node-1"));
    }

    #[test]
    fn lineage_response_deserializes_nodes_hashmap() {
        let response_json = serde_json::json!({
            "seed": "urn:lsid:test:seed-1",
            "seeds": ["urn:lsid:test:seed-1"],
            "nodes": {
                "urn:lsid:test:seed-1": {
                    "lsid": "urn:lsid:test:seed-1",
                    "name": "seed",
                    "parents": [],
                    "children": []
                }
            }
        });

        let response: LineageResponse =
            serde_json::from_value(response_json).expect("deserialize lineage response");
        assert_eq!(response.seed.as_deref(), Some("urn:lsid:test:seed-1"));
        assert_eq!(response.seeds.len(), 1);
        assert_eq!(response.nodes.len(), 1);
        assert!(response.nodes.contains_key("urn:lsid:test:seed-1"));
    }

    #[test]
    fn lineage_params_include_repeated_lsid_and_exp_type_wire_value() {
        let options = LineageOptions::builder()
            .lsids(vec![
                "urn:lsid:test:run-1".to_string(),
                "urn:lsid:test:run-2".to_string(),
            ])
            .exp_type(ExpType::ExperimentRun)
            .include_inputs_and_outputs(true)
            .build();

        let params = build_lineage_params(&options);
        assert!(params.contains(&("includeInputsAndOutputs".to_string(), "true".to_string())));
        assert!(params.contains(&("expType".to_string(), "ExperimentRun".to_string())));

        let lsid_count = params.iter().filter(|(k, _)| k == "lsid").count();
        assert_eq!(lsid_count, 2);
        assert!(params.contains(&("lsid".to_string(), "urn:lsid:test:run-1".to_string())));
        assert!(params.contains(&("lsid".to_string(), "urn:lsid:test:run-2".to_string())));
    }

    #[test]
    fn lineage_single_lsid_edge_case_uses_deprecated_singular_param() {
        let options = LineageOptions::builder()
            .lsid("urn:lsid:test:seed-only".to_string())
            .build();

        let params = build_lineage_params(&options);
        let lsid_count = params.iter().filter(|(k, _)| k == "lsid").count();
        assert_eq!(lsid_count, 1);
        assert_eq!(
            params
                .iter()
                .find(|(k, _)| k == "lsid")
                .map(|(_, v)| v.as_str()),
            Some("urn:lsid:test:seed-only")
        );
    }

    #[test]
    fn all_experiment_endpoint_urls_match_expected_routes() {
        let client = test_client();
        let cases = [("lineage", "lineage.api"), ("resolve", "resolve.api")];

        for (_label, action) in cases {
            let url = client.build_url("experiment", action, None);
            assert_eq!(
                url.as_str(),
                format!("https://labkey.example.com/labkey/Project/Folder/experiment-{action}")
            );
        }
    }

    #[test]
    fn resolve_params_include_repeated_lsid_and_converter_flags() {
        let options = ResolveOptions::builder()
            .lsids(vec![
                "urn:lsid:test:data-1".to_string(),
                "urn:lsid:test:data-2".to_string(),
            ])
            .include_inputs_and_outputs(true)
            .include_properties(false)
            .include_run_steps(true)
            .build();

        let params = build_resolve_params(&options);
        assert!(params.contains(&("includeInputsAndOutputs".to_string(), "true".to_string())));
        assert!(params.contains(&("includeProperties".to_string(), "false".to_string())));
        assert!(params.contains(&("includeRunSteps".to_string(), "true".to_string())));

        let lsid_count = params.iter().filter(|(k, _)| k == "lsid").count();
        assert_eq!(lsid_count, 2);
        assert!(params.contains(&("lsid".to_string(), "urn:lsid:test:data-1".to_string())));
        assert!(params.contains(&("lsid".to_string(), "urn:lsid:test:data-2".to_string())));
    }

    #[test]
    fn lineage_validation_rejects_missing_and_conflicting_lsid_inputs() {
        let missing = validate_lineage_options(&LineageOptions::builder().build())
            .expect_err("missing lsid input should fail");
        assert!(matches!(missing, LabkeyError::InvalidInput(_)));

        let conflicting = validate_lineage_options(
            &LineageOptions::builder()
                .lsid("urn:lsid:test:one".to_string())
                .lsids(vec!["urn:lsid:test:two".to_string()])
                .build(),
        )
        .expect_err("conflicting lsid inputs should fail");
        assert!(matches!(conflicting, LabkeyError::InvalidInput(_)));
    }

    #[test]
    fn resolve_validation_rejects_missing_or_blank_lsids() {
        let missing = validate_resolve_options(&ResolveOptions::builder().build())
            .expect_err("missing lsids");
        assert!(matches!(missing, LabkeyError::InvalidInput(_)));

        let blank = validate_resolve_options(
            &ResolveOptions::builder()
                .lsids(vec!["   ".to_string()])
                .build(),
        )
        .expect_err("blank lsid should fail");
        assert!(matches!(blank, LabkeyError::InvalidInput(_)));
    }

    #[test]
    fn exp_type_round_trips_all_variants() {
        let variants = [ExpType::Data, ExpType::Material, ExpType::ExperimentRun];

        for variant in variants {
            let json = serde_json::to_string(&variant).expect("serialize exp type");
            let restored: ExpType = serde_json::from_str(&json).expect("deserialize exp type");
            assert_eq!(restored, variant);
        }
    }

    #[test]
    fn seq_type_round_trips_all_variants() {
        let variants = [
            SeqType::GenId,
            SeqType::RootSampleCount,
            SeqType::SampleCount,
        ];

        for variant in variants {
            let json = serde_json::to_string(&variant).expect("serialize seq type");
            let restored: SeqType = serde_json::from_str(&json).expect("deserialize seq type");
            assert_eq!(restored, variant);
        }
    }

    fn exp_type_variant_count(value: ExpType) -> usize {
        match value {
            ExpType::Data | ExpType::Material | ExpType::ExperimentRun => 3,
        }
    }

    fn seq_type_variant_count(value: SeqType) -> usize {
        match value {
            SeqType::GenId | SeqType::RootSampleCount | SeqType::SampleCount => 3,
        }
    }

    #[test]
    fn exp_type_variant_count_regression() {
        assert_eq!(exp_type_variant_count(ExpType::Data), 3);
    }

    #[test]
    fn seq_type_variant_count_regression() {
        assert_eq!(seq_type_variant_count(SeqType::GenId), 3);
    }

    #[test]
    fn resolve_envelope_extracts_data() {
        let response = serde_json::json!({
            "data": [
                {
                    "lsid": "urn:lsid:test:resolved-1",
                    "name": "resolved-1",
                    "parents": [],
                    "children": []
                }
            ]
        });

        let resolved = extract_resolve_response(&response).expect("extract resolve data");
        assert_eq!(resolved.data.len(), 1);
        assert_eq!(
            resolved.data[0].exp_object.lsid.as_deref(),
            Some("urn:lsid:test:resolved-1")
        );
    }

    #[test]
    fn resolve_missing_data_returns_unexpected_response() {
        let response = serde_json::json!({"success": true});
        let error = extract_resolve_response(&response).expect_err("missing data should fail");

        match error {
            LabkeyError::UnexpectedResponse { status, text } => {
                assert_eq!(status, StatusCode::OK);
                assert!(text.contains("invalid resolve response"));
            }
            other => panic!("expected unexpected response error, got {other:?}"),
        }
    }
}
