//! Experiment models and APIs for lineage, batch, run, and sequence operations.

use std::collections::HashMap;
use std::time::Duration;

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    client::{LabkeyClient, RequestOptions},
    common::opt,
    error::LabkeyError,
    query::{InsertRowsOptions, ModifyRowsResults},
};

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

impl SeqType {
    const fn as_wire(self) -> &'static str {
        match self {
            Self::GenId => "genId",
            Self::RootSampleCount => "rootSampleCount",
            Self::SampleCount => "sampleCount",
        }
    }
}

/// Entity kind values used by sequence APIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EntityKindName {
    /// Sequence attached to a data class.
    #[serde(rename = "DataClass")]
    DataClass,
    /// Sequence attached to a sample set.
    #[serde(rename = "SampleSet")]
    SampleSet,
}

impl EntityKindName {
    const fn as_wire(self) -> &'static str {
        match self {
            Self::DataClass => "DataClass",
            Self::SampleSet => "SampleSet",
        }
    }
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
    /// Seed LSIDs. When provided, these are emitted as repeated `lsids` params.
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
    /// LSIDs to resolve, emitted as repeated `lsids` query parameters.
    pub lsids: Option<Vec<String>>,
    /// Include run and step inputs and outputs.
    pub include_inputs_and_outputs: Option<bool>,
    /// Include experiment object properties.
    pub include_properties: Option<bool>,
    /// Include run steps.
    pub include_run_steps: Option<bool>,
}

/// One-of input for [`LabkeyClient::create_hidden_run_group`].
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum HiddenRunGroupMembers {
    /// Use explicit run row IDs.
    RunIds(Vec<i64>),
    /// Use a `DataRegion` selection key.
    SelectionKey(String),
}

/// Options for [`LabkeyClient::save_batch`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SaveBatchOptions {
    /// The assay protocol id.
    pub assay_id: i64,
    /// Modified run group to save.
    pub batch: RunGroup,
    /// Optional assay name.
    pub assay_name: Option<String>,
    /// Optional protocol name for non-assay-backed runs.
    pub protocol_name: Option<String>,
    /// Optional assay provider name.
    pub provider_name: Option<String>,
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Optional request timeout override.
    pub timeout: Option<Duration>,
}

/// Options for [`LabkeyClient::save_batches`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SaveBatchesOptions {
    /// The assay protocol id.
    pub assay_id: i64,
    /// Modified run groups to save.
    pub batches: Vec<RunGroup>,
    /// Optional assay name.
    pub assay_name: Option<String>,
    /// Optional protocol name for non-assay-backed runs.
    pub protocol_name: Option<String>,
    /// Optional assay provider name.
    pub provider_name: Option<String>,
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Optional request timeout override.
    pub timeout: Option<Duration>,
}

/// Options for [`LabkeyClient::load_batch`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct LoadBatchOptions {
    /// The assay protocol id.
    pub assay_id: Option<i64>,
    /// Optional assay name.
    pub assay_name: Option<String>,
    /// Batch id to load.
    pub batch_id: i64,
    /// Optional protocol name for non-assay-backed runs.
    pub protocol_name: Option<String>,
    /// Optional assay provider name.
    pub provider_name: Option<String>,
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Optional request timeout override.
    pub timeout: Option<Duration>,
}

/// Options for [`LabkeyClient::load_batches`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct LoadBatchesOptions {
    /// The assay protocol id.
    pub assay_id: Option<i64>,
    /// Optional assay name.
    pub assay_name: Option<String>,
    /// Batch ids to load.
    pub batch_ids: Vec<i64>,
    /// Optional protocol name for non-assay-backed runs.
    pub protocol_name: Option<String>,
    /// Optional assay provider name.
    pub provider_name: Option<String>,
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Optional request timeout override.
    pub timeout: Option<Duration>,
}

/// Options for [`LabkeyClient::load_runs`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct LoadRunsOptions {
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Optional request timeout override.
    pub timeout: Option<Duration>,
    /// Run LSIDs to load.
    pub lsids: Option<Vec<String>>,
    /// Run row IDs to load.
    pub run_ids: Option<Vec<i64>>,
    /// Include run and step inputs and outputs.
    pub include_inputs_and_outputs: Option<bool>,
    /// Include experiment object properties.
    pub include_properties: Option<bool>,
    /// Include run steps.
    pub include_run_steps: Option<bool>,
}

/// Options for [`LabkeyClient::save_runs`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SaveRunsOptions {
    /// Runs to save.
    pub runs: Vec<Run>,
    /// Optional assay protocol id.
    pub assay_id: Option<i64>,
    /// Optional assay name.
    pub assay_name: Option<String>,
    /// Optional protocol name for non-assay-backed runs.
    pub protocol_name: Option<String>,
    /// Optional assay provider name.
    pub provider_name: Option<String>,
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Optional request timeout override.
    pub timeout: Option<Duration>,
}

/// Options for [`LabkeyClient::save_materials`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SaveMaterialsOptions {
    /// Name of the sample set query.
    pub name: String,
    /// Material rows to save.
    pub materials: Vec<serde_json::Value>,
    /// Optional container override for request routing.
    pub container_path: Option<String>,
}

/// Options for [`LabkeyClient::create_hidden_run_group`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct CreateHiddenRunGroupOptions {
    /// Exactly one member-selection mode.
    pub members: HiddenRunGroupMembers,
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Optional request timeout override.
    pub timeout: Option<Duration>,
}

/// Options for [`LabkeyClient::set_entity_sequence`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct SetEntitySequenceOptions {
    /// Sequence type to update.
    pub seq_type: SeqType,
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Optional entity kind.
    pub kind_name: Option<EntityKindName>,
    /// Optional new value for the sequence.
    pub new_value: Option<i64>,
    /// Optional entity row ID.
    pub row_id: Option<i64>,
    /// Optional request timeout override.
    pub timeout: Option<Duration>,
}

/// Options for [`LabkeyClient::get_entity_sequence`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetEntitySequenceOptions {
    /// Sequence type to read.
    pub seq_type: SeqType,
    /// Optional container override for request routing.
    pub container_path: Option<String>,
    /// Optional entity kind.
    pub kind_name: Option<EntityKindName>,
    /// Optional entity row ID.
    pub row_id: Option<i64>,
}

/// Response from [`LabkeyClient::set_entity_sequence`] and [`LabkeyClient::get_entity_sequence`].
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct EntitySequenceResponse {
    /// Sequence type.
    #[serde(default)]
    pub seq_type: Option<String>,
    /// Entity kind.
    #[serde(default)]
    pub kind_name: Option<String>,
    /// Entity row id.
    #[serde(default)]
    pub row_id: Option<i64>,
    /// Updated sequence value when present.
    #[serde(default)]
    pub new_value: Option<i64>,
    /// Returned sequence value when present.
    #[serde(default)]
    pub value: Option<i64>,
    /// Additional server-provided keys.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveBatchesBody {
    assay_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    assay_name: Option<String>,
    batches: Vec<RunGroup>,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocol_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoadBatchBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    assay_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assay_name: Option<String>,
    batch_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocol_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoadBatchesBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    assay_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assay_name: Option<String>,
    batch_ids: Vec<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocol_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_name: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct LoadRunsBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    run_ids: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lsids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_inputs_and_outputs: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_properties: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_run_steps: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SaveRunsBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    assay_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assay_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    protocol_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_name: Option<String>,
    runs: Vec<Run>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateHiddenRunGroupBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    run_ids: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    selection_key: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SetEntitySequenceBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    row_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind_name: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_value: Option<i64>,
    seq_type: SeqType,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResolveEnvelope {
    data: Vec<LineageNode>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoadBatchEnvelope {
    batch: RunGroup,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoadBatchesEnvelope {
    batches: Vec<RunGroup>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LoadRunsEnvelope {
    runs: Vec<Run>,
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
                .map(|value| ("lsids".to_string(), value)),
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
                .map(|value| ("lsids".to_string(), value)),
        );
    }

    params
}

fn build_save_batch_body(options: SaveBatchOptions) -> SaveBatchesBody {
    SaveBatchesBody {
        assay_id: options.assay_id,
        assay_name: options.assay_name,
        batches: vec![options.batch],
        protocol_name: options.protocol_name,
        provider_name: options.provider_name,
    }
}

fn build_save_batches_body(options: SaveBatchesOptions) -> SaveBatchesBody {
    SaveBatchesBody {
        assay_id: options.assay_id,
        assay_name: options.assay_name,
        batches: options.batches,
        protocol_name: options.protocol_name,
        provider_name: options.provider_name,
    }
}

fn build_load_batch_body(options: LoadBatchOptions) -> LoadBatchBody {
    LoadBatchBody {
        assay_id: options.assay_id,
        assay_name: options.assay_name,
        batch_id: options.batch_id,
        protocol_name: options.protocol_name,
        provider_name: options.provider_name,
    }
}

fn build_load_batches_body(options: LoadBatchesOptions) -> LoadBatchesBody {
    LoadBatchesBody {
        assay_id: options.assay_id,
        assay_name: options.assay_name,
        batch_ids: options.batch_ids,
        protocol_name: options.protocol_name,
        provider_name: options.provider_name,
    }
}

fn build_load_runs_body(options: LoadRunsOptions) -> LoadRunsBody {
    LoadRunsBody {
        run_ids: options.run_ids,
        lsids: options.lsids,
        include_inputs_and_outputs: options.include_inputs_and_outputs,
        include_properties: options.include_properties,
        include_run_steps: options.include_run_steps,
    }
}

fn build_save_runs_body(options: SaveRunsOptions) -> SaveRunsBody {
    SaveRunsBody {
        assay_id: options.assay_id,
        assay_name: options.assay_name,
        protocol_name: options.protocol_name,
        provider_name: options.provider_name,
        runs: options.runs,
    }
}

fn build_create_hidden_run_group_body(members: HiddenRunGroupMembers) -> CreateHiddenRunGroupBody {
    match members {
        HiddenRunGroupMembers::RunIds(run_ids) => CreateHiddenRunGroupBody {
            run_ids: Some(run_ids),
            selection_key: None,
        },
        HiddenRunGroupMembers::SelectionKey(selection_key) => CreateHiddenRunGroupBody {
            run_ids: None,
            selection_key: Some(selection_key),
        },
    }
}

fn build_set_entity_sequence_body(options: &SetEntitySequenceOptions) -> SetEntitySequenceBody {
    SetEntitySequenceBody {
        row_id: options.row_id,
        kind_name: options.kind_name.map(EntityKindName::as_wire),
        new_value: options.new_value,
        seq_type: options.seq_type,
    }
}

fn build_get_entity_sequence_params(options: &GetEntitySequenceOptions) -> Vec<(String, String)> {
    [
        opt("rowId", options.row_id),
        options
            .kind_name
            .map(|value| ("kindName".to_string(), value.as_wire().to_string())),
        Some((
            "seqType".to_string(),
            options.seq_type.as_wire().to_string(),
        )),
    ]
    .into_iter()
    .flatten()
    .collect()
}

fn save_materials_to_insert_rows_options(options: SaveMaterialsOptions) -> InsertRowsOptions {
    InsertRowsOptions::builder()
        .schema_name("Samples".to_string())
        .query_name(options.name)
        .rows(options.materials)
        .maybe_container_path(options.container_path)
        .build()
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

fn extract_batch_response(response: &serde_json::Value) -> Result<RunGroup, LabkeyError> {
    serde_json::from_value::<LoadBatchEnvelope>(response.clone())
        .map(|envelope| envelope.batch)
        .map_err(|_| LabkeyError::UnexpectedResponse {
            status: StatusCode::OK,
            text: format!("invalid load_batch response: {response}"),
        })
}

fn extract_batches_response(response: &serde_json::Value) -> Result<Vec<RunGroup>, LabkeyError> {
    serde_json::from_value::<LoadBatchesEnvelope>(response.clone())
        .map(|envelope| envelope.batches)
        .map_err(|_| LabkeyError::UnexpectedResponse {
            status: StatusCode::OK,
            text: format!("invalid batches response: {response}"),
        })
}

fn extract_runs_response(response: &serde_json::Value) -> Result<Vec<Run>, LabkeyError> {
    serde_json::from_value::<LoadRunsEnvelope>(response.clone())
        .map(|envelope| envelope.runs)
        .map_err(|_| LabkeyError::UnexpectedResponse {
            status: StatusCode::OK,
            text: format!("invalid runs response: {response}"),
        })
}

fn extract_single_batch_response(response: &serde_json::Value) -> Result<RunGroup, LabkeyError> {
    let batches = extract_batches_response(response)?;
    batches
        .into_iter()
        .next()
        .ok_or(LabkeyError::UnexpectedResponse {
            status: StatusCode::OK,
            text: format!("invalid save_batch response: {response}"),
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

fn validate_assay_identity(
    assay_id: Option<i64>,
    assay_name: Option<&str>,
    endpoint: &str,
) -> Result<(), LabkeyError> {
    if assay_id.is_none() && assay_name.is_none_or(str::is_empty) {
        return Err(LabkeyError::InvalidInput(format!(
            "{endpoint} requires one of `assay_id` or `assay_name`"
        )));
    }

    Ok(())
}

fn validate_save_batches_options(options: &SaveBatchesOptions) -> Result<(), LabkeyError> {
    if options.batches.is_empty() {
        return Err(LabkeyError::InvalidInput(
            "save_batches `batches` must not be empty".to_string(),
        ));
    }

    Ok(())
}

fn validate_load_batch_options(options: &LoadBatchOptions) -> Result<(), LabkeyError> {
    validate_assay_identity(
        options.assay_id,
        options.assay_name.as_deref(),
        "load_batch",
    )
}

fn validate_load_batches_options(options: &LoadBatchesOptions) -> Result<(), LabkeyError> {
    validate_assay_identity(
        options.assay_id,
        options.assay_name.as_deref(),
        "load_batches",
    )?;

    if options.batch_ids.is_empty() {
        return Err(LabkeyError::InvalidInput(
            "load_batches `batch_ids` must not be empty".to_string(),
        ));
    }

    Ok(())
}

fn validate_load_runs_options(options: &LoadRunsOptions) -> Result<(), LabkeyError> {
    if let Some(run_ids) = options.run_ids.as_ref()
        && run_ids.is_empty()
    {
        return Err(LabkeyError::InvalidInput(
            "load_runs `run_ids` must not be empty".to_string(),
        ));
    }

    if let Some(lsids) = options.lsids.as_ref() {
        if lsids.is_empty() {
            return Err(LabkeyError::InvalidInput(
                "load_runs `lsids` must not be empty".to_string(),
            ));
        }
        if lsids.iter().any(|value| value.trim().is_empty()) {
            return Err(LabkeyError::InvalidInput(
                "load_runs `lsids` entries must not be blank".to_string(),
            ));
        }
    }

    Ok(())
}

fn validate_save_runs_options(options: &SaveRunsOptions) -> Result<(), LabkeyError> {
    if options.runs.is_empty() {
        return Err(LabkeyError::InvalidInput(
            "save_runs `runs` must not be empty".to_string(),
        ));
    }

    Ok(())
}

fn validate_save_materials_options(options: &SaveMaterialsOptions) -> Result<(), LabkeyError> {
    if options.name.trim().is_empty() {
        return Err(LabkeyError::InvalidInput(
            "save_materials `name` must not be blank".to_string(),
        ));
    }

    Ok(())
}

fn validate_create_hidden_run_group_options(
    options: &CreateHiddenRunGroupOptions,
) -> Result<(), LabkeyError> {
    match options.members {
        HiddenRunGroupMembers::RunIds(ref run_ids) if run_ids.is_empty() => {
            Err(LabkeyError::InvalidInput(
                "create_hidden_run_group `run_ids` must not be empty".to_string(),
            ))
        }
        HiddenRunGroupMembers::SelectionKey(ref key) if key.trim().is_empty() => {
            Err(LabkeyError::InvalidInput(
                "create_hidden_run_group `selection_key` must not be blank".to_string(),
            ))
        }
        _ => Ok(()),
    }
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

    /// Save a single assay batch through `assay-saveAssayBatch.api`.
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
    /// use labkey_rs::experiment::{ExpObject, RunGroup, SaveBatchOptions};
    ///
    /// let _ = client
    ///     .save_batch(
    ///         SaveBatchOptions::builder()
    ///             .assay_id(101)
    ///             .batch(RunGroup {
    ///                 exp_object: ExpObject {
    ///                     name: Some("batch-1".to_string()),
    ///                     ..ExpObject {
    ///                         comment: None,
    ///                         container: None,
    ///                         container_path: None,
    ///                         cpas_type: None,
    ///                         created: None,
    ///                         created_by: None,
    ///                         id: None,
    ///                         lsid: None,
    ///                         modified: None,
    ///                         modified_by: None,
    ///                         name: None,
    ///                         pk_filters: vec![],
    ///                         query_name: None,
    ///                         restricted: None,
    ///                         schema_name: None,
    ///                         type_: None,
    ///                         url: None,
    ///                         properties: None,
    ///                     }
    ///                 },
    ///                 batch_protocol_id: None,
    ///                 hidden: None,
    ///                 runs: vec![],
    ///             })
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_batch(&self, options: SaveBatchOptions) -> Result<RunGroup, LabkeyError> {
        validate_assay_identity(
            Some(options.assay_id),
            options.assay_name.as_deref(),
            "save_batch",
        )?;
        let timeout = options.timeout;
        let container_path = options.container_path.clone();
        let body = build_save_batch_body(options);
        let url = self.build_url("assay", "saveAssayBatch.api", container_path.as_deref());
        let response: serde_json::Value = self
            .post_with_options(
                url,
                &body,
                &RequestOptions {
                    timeout,
                    ..RequestOptions::default()
                },
            )
            .await?;
        extract_single_batch_response(&response)
    }

    /// Save multiple assay batches through `assay-saveAssayBatch.api`.
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
    /// use labkey_rs::experiment::SaveBatchesOptions;
    ///
    /// let _ = client
    ///     .save_batches(
    ///         SaveBatchesOptions::builder()
    ///             .assay_id(101)
    ///             .batches(vec![])
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_batches(
        &self,
        options: SaveBatchesOptions,
    ) -> Result<Vec<RunGroup>, LabkeyError> {
        validate_save_batches_options(&options)?;
        validate_assay_identity(
            Some(options.assay_id),
            options.assay_name.as_deref(),
            "save_batches",
        )?;
        let timeout = options.timeout;
        let container_path = options.container_path.clone();
        let body = build_save_batches_body(options);
        let url = self.build_url("assay", "saveAssayBatch.api", container_path.as_deref());
        let response: serde_json::Value = self
            .post_with_options(
                url,
                &body,
                &RequestOptions {
                    timeout,
                    ..RequestOptions::default()
                },
            )
            .await?;
        extract_batches_response(&response)
    }

    /// Load a single assay batch through `assay-getAssayBatch.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server reports an API
    /// error, or the response envelope does not include `batch`.
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
    /// use labkey_rs::experiment::LoadBatchOptions;
    ///
    /// let _ = client
    ///     .load_batch(
    ///         LoadBatchOptions::builder()
    ///             .batch_id(10)
    ///             .assay_id(101)
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_batch(&self, options: LoadBatchOptions) -> Result<RunGroup, LabkeyError> {
        validate_load_batch_options(&options)?;
        let timeout = options.timeout;
        let container_path = options.container_path.clone();
        let body = build_load_batch_body(options);
        let url = self.build_url("assay", "getAssayBatch.api", container_path.as_deref());
        let response: serde_json::Value = self
            .post_with_options(
                url,
                &body,
                &RequestOptions {
                    timeout,
                    ..RequestOptions::default()
                },
            )
            .await?;
        extract_batch_response(&response)
    }

    /// Load multiple assay batches through `assay-getAssayBatches.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server reports an API
    /// error, or the response envelope does not include `batches`.
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
    /// use labkey_rs::experiment::LoadBatchesOptions;
    ///
    /// let _ = client
    ///     .load_batches(
    ///         LoadBatchesOptions::builder()
    ///             .batch_ids(vec![10, 11])
    ///             .assay_name("General".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_batches(
        &self,
        options: LoadBatchesOptions,
    ) -> Result<Vec<RunGroup>, LabkeyError> {
        validate_load_batches_options(&options)?;
        let timeout = options.timeout;
        let container_path = options.container_path.clone();
        let body = build_load_batches_body(options);
        let url = self.build_url("assay", "getAssayBatches.api", container_path.as_deref());
        let response: serde_json::Value = self
            .post_with_options(
                url,
                &body,
                &RequestOptions {
                    timeout,
                    ..RequestOptions::default()
                },
            )
            .await?;
        extract_batches_response(&response)
    }

    /// Load assay runs through `assay-getAssayRuns.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server reports an API
    /// error, or the response envelope does not include `runs`.
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
    /// use labkey_rs::experiment::LoadRunsOptions;
    ///
    /// let _ = client
    ///     .load_runs(
    ///         LoadRunsOptions::builder()
    ///             .lsids(vec!["urn:lsid:test:run-1".to_string()])
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn load_runs(&self, options: LoadRunsOptions) -> Result<Vec<Run>, LabkeyError> {
        validate_load_runs_options(&options)?;
        let timeout = options.timeout;
        let container_path = options.container_path.clone();
        let body = build_load_runs_body(options);
        let url = self.build_url("assay", "getAssayRuns.api", container_path.as_deref());
        let response: serde_json::Value = self
            .post_with_options(
                url,
                &body,
                &RequestOptions {
                    timeout,
                    ..RequestOptions::default()
                },
            )
            .await?;
        extract_runs_response(&response)
    }

    /// Save modified assay runs through `assay-saveAssayRuns.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails, the server reports an API
    /// error, or the response envelope does not include `runs`.
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
    /// use labkey_rs::experiment::SaveRunsOptions;
    ///
    /// let _ = client
    ///     .save_runs(SaveRunsOptions::builder().runs(vec![]).build())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_runs(&self, options: SaveRunsOptions) -> Result<Vec<Run>, LabkeyError> {
        validate_save_runs_options(&options)?;
        let timeout = options.timeout;
        let container_path = options.container_path.clone();
        let body = build_save_runs_body(options);
        let url = self.build_url("assay", "saveAssayRuns.api", container_path.as_deref());
        let response: serde_json::Value = self
            .post_with_options(
                url,
                &body,
                &RequestOptions {
                    timeout,
                    ..RequestOptions::default()
                },
            )
            .await?;
        extract_runs_response(&response)
    }

    /// Save material rows by delegating to [`LabkeyClient::insert_rows`].
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the delegated insert-rows request fails.
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
    /// use labkey_rs::experiment::SaveMaterialsOptions;
    ///
    /// let _ = client
    ///     .save_materials(
    ///         SaveMaterialsOptions::builder()
    ///             .name("MySamples".to_string())
    ///             .materials(vec![])
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn save_materials(
        &self,
        options: SaveMaterialsOptions,
    ) -> Result<ModifyRowsResults, LabkeyError> {
        validate_save_materials_options(&options)?;
        self.insert_rows(save_materials_to_insert_rows_options(options))
            .await
    }

    /// Create or recycle a hidden run group through `experiment-createHiddenRunGroup.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails or the response cannot be
    /// parsed as a [`RunGroup`].
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
    /// use labkey_rs::experiment::{CreateHiddenRunGroupOptions, HiddenRunGroupMembers};
    ///
    /// let _ = client
    ///     .create_hidden_run_group(
    ///         CreateHiddenRunGroupOptions::builder()
    ///             .members(HiddenRunGroupMembers::RunIds(vec![1, 2]))
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_hidden_run_group(
        &self,
        options: CreateHiddenRunGroupOptions,
    ) -> Result<RunGroup, LabkeyError> {
        validate_create_hidden_run_group_options(&options)?;
        let timeout = options.timeout;
        let container_path = options.container_path.clone();
        let body = build_create_hidden_run_group_body(options.members);
        let url = self.build_url(
            "experiment",
            "createHiddenRunGroup.api",
            container_path.as_deref(),
        );
        self.post_with_options(
            url,
            &body,
            &RequestOptions {
                timeout,
                ..RequestOptions::default()
            },
        )
        .await
    }

    /// Update entity sequence state through `experiment-setEntitySequence.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails or the response cannot be parsed.
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
    /// use labkey_rs::experiment::{EntityKindName, SeqType, SetEntitySequenceOptions};
    ///
    /// let _ = client
    ///     .set_entity_sequence(
    ///         SetEntitySequenceOptions::builder()
    ///             .seq_type(SeqType::GenId)
    ///             .kind_name(EntityKindName::SampleSet)
    ///             .new_value(100)
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn set_entity_sequence(
        &self,
        options: SetEntitySequenceOptions,
    ) -> Result<EntitySequenceResponse, LabkeyError> {
        let timeout = options.timeout;
        let container_path = options.container_path.clone();
        let body = build_set_entity_sequence_body(&options);
        let url = self.build_url(
            "experiment",
            "setEntitySequence.api",
            container_path.as_deref(),
        );
        self.post_with_options(
            url,
            &body,
            &RequestOptions {
                timeout,
                ..RequestOptions::default()
            },
        )
        .await
    }

    /// Get current entity sequence state through `experiment-getEntitySequence.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the request fails or the response cannot be parsed.
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
    /// use labkey_rs::experiment::{GetEntitySequenceOptions, SeqType};
    ///
    /// let _ = client
    ///     .get_entity_sequence(GetEntitySequenceOptions::builder().seq_type(SeqType::GenId).build())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_entity_sequence(
        &self,
        options: GetEntitySequenceOptions,
    ) -> Result<EntitySequenceResponse, LabkeyError> {
        let url = self.build_url(
            "experiment",
            "getEntitySequence.api",
            options.container_path.as_deref(),
        );
        let params = build_get_entity_sequence_params(&options);
        self.get(url, &params).await
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
    fn lineage_params_include_repeated_lsids_and_exp_type_wire_value() {
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

        let lsids_count = params.iter().filter(|(k, _)| k == "lsids").count();
        assert_eq!(lsids_count, 2);
        assert!(params.contains(&("lsids".to_string(), "urn:lsid:test:run-1".to_string())));
        assert!(params.contains(&("lsids".to_string(), "urn:lsid:test:run-2".to_string())));
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
        let cases = [
            ("lineage", "experiment", "lineage.api"),
            ("resolve", "experiment", "resolve.api"),
            ("save_batch", "assay", "saveAssayBatch.api"),
            ("save_batches", "assay", "saveAssayBatch.api"),
            ("load_batch", "assay", "getAssayBatch.api"),
            ("load_batches", "assay", "getAssayBatches.api"),
            ("load_runs", "assay", "getAssayRuns.api"),
            ("save_runs", "assay", "saveAssayRuns.api"),
            (
                "create_hidden_run_group",
                "experiment",
                "createHiddenRunGroup.api",
            ),
            ("set_entity_sequence", "experiment", "setEntitySequence.api"),
            ("get_entity_sequence", "experiment", "getEntitySequence.api"),
        ];

        for (_label, controller, action) in cases {
            let url = client.build_url(controller, action, None);
            assert_eq!(
                url.as_str(),
                format!("https://labkey.example.com/labkey/Project/Folder/{controller}-{action}")
            );
        }
    }

    #[test]
    fn resolve_params_include_repeated_lsids_and_converter_flags() {
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

        let lsids_count = params.iter().filter(|(k, _)| k == "lsids").count();
        assert_eq!(lsids_count, 2);
        assert!(params.contains(&("lsids".to_string(), "urn:lsid:test:data-1".to_string())));
        assert!(params.contains(&("lsids".to_string(), "urn:lsid:test:data-2".to_string())));
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

    #[test]
    fn entity_kind_name_round_trips_all_variants() {
        let variants = [EntityKindName::DataClass, EntityKindName::SampleSet];

        for variant in variants {
            let json = serde_json::to_string(&variant).expect("serialize entity kind");
            let restored: EntityKindName =
                serde_json::from_str(&json).expect("deserialize entity kind");
            assert_eq!(restored, variant);
        }
    }

    #[test]
    fn entity_kind_name_serializes_exact_wire_values() {
        assert_eq!(
            serde_json::to_string(&EntityKindName::DataClass).expect("serialize entity kind"),
            "\"DataClass\""
        );
        assert_eq!(
            serde_json::to_string(&EntityKindName::SampleSet).expect("serialize entity kind"),
            "\"SampleSet\""
        );
    }

    #[test]
    fn entity_kind_name_rejects_unknown_wire_value() {
        let err = serde_json::from_str::<EntityKindName>("\"UnknownKind\"")
            .expect_err("unknown entity kind should fail to deserialize");
        assert!(err.is_data());
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

    fn entity_kind_name_variant_count(value: EntityKindName) -> usize {
        match value {
            EntityKindName::DataClass | EntityKindName::SampleSet => 2,
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
    fn entity_kind_name_variant_count_regression() {
        assert_eq!(entity_kind_name_variant_count(EntityKindName::DataClass), 2);
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

    #[test]
    fn run_group_round_trips_serde() {
        let group_json = serde_json::json!({
            "name": "batch-1",
            "batchProtocolId": 7,
            "hidden": true,
            "runs": [
                {
                    "name": "run-1",
                    "dataInputs": [],
                    "dataOutputs": [],
                    "materialInputs": [],
                    "materialOutputs": []
                }
            ]
        });

        let group: RunGroup =
            serde_json::from_value(group_json.clone()).expect("deserialize run group");
        assert_eq!(group.exp_object.name.as_deref(), Some("batch-1"));
        assert_eq!(group.batch_protocol_id, Some(7));
        assert_eq!(group.hidden, Some(true));
        assert_eq!(group.runs.len(), 1);

        let serialized = serde_json::to_value(&group).expect("serialize run group");
        assert_eq!(serialized["name"], "batch-1");
        assert_eq!(serialized["batchProtocolId"], 7);
    }

    #[test]
    fn save_batch_body_wraps_single_batch_in_array() {
        let options = SaveBatchOptions::builder()
            .assay_id(12)
            .batch(RunGroup {
                exp_object: ExpObject {
                    comment: None,
                    container: None,
                    container_path: None,
                    cpas_type: None,
                    created: None,
                    created_by: None,
                    id: None,
                    lsid: None,
                    modified: None,
                    modified_by: None,
                    name: Some("batch-one".to_string()),
                    pk_filters: vec![],
                    query_name: None,
                    restricted: None,
                    schema_name: None,
                    type_: None,
                    url: None,
                    properties: None,
                },
                batch_protocol_id: None,
                hidden: None,
                runs: vec![],
            })
            .build();

        let body = build_save_batch_body(options);
        let body_json = serde_json::to_value(body).expect("serialize save batch body");
        assert_eq!(body_json["assayId"], 12);
        assert!(body_json["batches"].is_array());
        assert_eq!(
            body_json["batches"]
                .as_array()
                .expect("batches array")
                .len(),
            1
        );
        assert_eq!(body_json["batches"][0]["name"], "batch-one");
    }

    #[test]
    fn batch_and_runs_envelope_extraction_handles_success_and_missing_fields() {
        let batch_response = serde_json::json!({
            "batch": {
                "name": "loaded-batch",
                "runs": []
            }
        });
        let batches_response = serde_json::json!({
            "batches": [
                {
                    "name": "loaded-batch",
                    "runs": []
                }
            ]
        });
        let runs_response = serde_json::json!({
            "runs": [
                {
                    "name": "run-1",
                    "dataInputs": [],
                    "dataOutputs": [],
                    "materialInputs": [],
                    "materialOutputs": []
                }
            ]
        });

        let batch = extract_batch_response(&batch_response).expect("extract single batch");
        assert_eq!(batch.exp_object.name.as_deref(), Some("loaded-batch"));

        let batches = extract_batches_response(&batches_response).expect("extract batches");
        assert_eq!(batches.len(), 1);

        let runs = extract_runs_response(&runs_response).expect("extract runs");
        assert_eq!(runs.len(), 1);

        let missing_batch = extract_batch_response(&serde_json::json!({"success": true}))
            .expect_err("missing batch should fail");
        assert!(matches!(
            missing_batch,
            LabkeyError::UnexpectedResponse { .. }
        ));

        let missing_batches = extract_batches_response(&serde_json::json!({"success": true}))
            .expect_err("missing batches should fail");
        assert!(matches!(
            missing_batches,
            LabkeyError::UnexpectedResponse { .. }
        ));

        let missing_runs = extract_runs_response(&serde_json::json!({"success": true}))
            .expect_err("missing runs should fail");
        assert!(matches!(
            missing_runs,
            LabkeyError::UnexpectedResponse { .. }
        ));

        let empty_batches = extract_single_batch_response(&serde_json::json!({"batches": []}))
            .expect_err("empty save_batch envelope should fail");
        assert!(matches!(
            empty_batches,
            LabkeyError::UnexpectedResponse { .. }
        ));

        let invalid_batch_shape = extract_batch_response(&serde_json::json!({"batch": []}))
            .expect_err("invalid batch shape should fail");
        assert!(matches!(
            invalid_batch_shape,
            LabkeyError::UnexpectedResponse { .. }
        ));
    }

    #[test]
    fn save_materials_delegates_to_insert_rows_samples_schema() {
        let options = SaveMaterialsOptions::builder()
            .name("MySampleSet".to_string())
            .materials(vec![serde_json::json!({"Name": "M-1"})])
            .maybe_container_path(Some("/Project/Alt".to_string()))
            .build();

        let insert_options = save_materials_to_insert_rows_options(options);
        assert_eq!(insert_options.schema_name, "Samples");
        assert_eq!(insert_options.query_name, "MySampleSet");
        assert_eq!(insert_options.rows.len(), 1);
        assert_eq!(
            insert_options.container_path.as_deref(),
            Some("/Project/Alt")
        );
    }

    #[test]
    fn create_hidden_run_group_body_uses_exactly_one_input_mode() {
        let run_ids =
            build_create_hidden_run_group_body(HiddenRunGroupMembers::RunIds(vec![1, 2, 3]));
        let run_ids_json = serde_json::to_value(run_ids).expect("serialize run id body");
        assert_eq!(run_ids_json["runIds"], serde_json::json!([1, 2, 3]));
        assert!(run_ids_json.get("selectionKey").is_none());

        let selection = build_create_hidden_run_group_body(HiddenRunGroupMembers::SelectionKey(
            "selection-1".to_string(),
        ));
        let selection_json = serde_json::to_value(selection).expect("serialize selection key body");
        assert_eq!(selection_json["selectionKey"], "selection-1");
        assert!(selection_json.get("runIds").is_none());
    }

    #[test]
    fn get_entity_sequence_params_include_seq_type_and_optional_fields() {
        let options = GetEntitySequenceOptions::builder()
            .seq_type(SeqType::RootSampleCount)
            .kind_name(EntityKindName::SampleSet)
            .row_id(77)
            .build();

        let params = build_get_entity_sequence_params(&options);
        assert!(params.contains(&("seqType".to_string(), "rootSampleCount".to_string())));
        assert!(params.contains(&("kindName".to_string(), "SampleSet".to_string())));
        assert!(params.contains(&("rowId".to_string(), "77".to_string())));
    }

    #[test]
    fn set_entity_sequence_body_serializes_seq_type_and_omits_absent_optionals() {
        let body = build_set_entity_sequence_body(
            &SetEntitySequenceOptions::builder()
                .seq_type(SeqType::GenId)
                .build(),
        );
        let body_json = serde_json::to_value(body).expect("serialize set entity sequence body");
        assert_eq!(body_json["seqType"], "genId");
        assert!(body_json.get("rowId").is_none());
        assert!(body_json.get("kindName").is_none());
        assert!(body_json.get("newValue").is_none());
    }

    #[test]
    fn experiment_batch_and_run_validation_rejects_invalid_input() {
        let hidden_run_ids = validate_create_hidden_run_group_options(
            &CreateHiddenRunGroupOptions::builder()
                .members(HiddenRunGroupMembers::RunIds(vec![]))
                .build(),
        )
        .expect_err("empty run ids should fail");
        assert!(matches!(hidden_run_ids, LabkeyError::InvalidInput(_)));

        let hidden_selection = validate_create_hidden_run_group_options(
            &CreateHiddenRunGroupOptions::builder()
                .members(HiddenRunGroupMembers::SelectionKey("   ".to_string()))
                .build(),
        )
        .expect_err("blank selection key should fail");
        assert!(matches!(hidden_selection, LabkeyError::InvalidInput(_)));

        let save_batches = validate_save_batches_options(
            &SaveBatchesOptions::builder()
                .assay_id(1)
                .batches(vec![])
                .build(),
        )
        .expect_err("empty batches should fail");
        assert!(matches!(save_batches, LabkeyError::InvalidInput(_)));

        let load_batches = validate_load_batches_options(
            &LoadBatchesOptions::builder()
                .batch_ids(vec![])
                .assay_id(1)
                .build(),
        )
        .expect_err("empty batch ids should fail");
        assert!(matches!(load_batches, LabkeyError::InvalidInput(_)));

        let save_runs =
            validate_save_runs_options(&SaveRunsOptions::builder().runs(vec![]).build())
                .expect_err("empty runs should fail");
        assert!(matches!(save_runs, LabkeyError::InvalidInput(_)));

        let load_runs = validate_load_runs_options(
            &LoadRunsOptions::builder()
                .lsids(vec!["   ".to_string()])
                .build(),
        )
        .expect_err("blank run lsid should fail");
        assert!(matches!(load_runs, LabkeyError::InvalidInput(_)));

        let save_materials = validate_save_materials_options(
            &SaveMaterialsOptions::builder()
                .name("  ".to_string())
                .materials(vec![])
                .build(),
        )
        .expect_err("blank sample-set name should fail");
        assert!(matches!(save_materials, LabkeyError::InvalidInput(_)));
    }
}
