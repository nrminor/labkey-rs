//! Shared types reused across multiple API modules.

use crate::filter::ContainerFilter;

/// Controls how `LabKey` records audit details for write operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[non_exhaustive]
pub enum AuditBehavior {
    /// Do not create audit details.
    #[serde(rename = "NONE")]
    None,
    /// Create summary-level audit details.
    #[serde(rename = "SUMMARY")]
    Summary,
    /// Create detailed audit details.
    #[serde(rename = "DETAILED")]
    Detailed,
}

/// Serialize a [`ContainerFilter`] to its string representation for use
/// as a query parameter value.
#[must_use]
pub(crate) fn container_filter_to_string(cf: ContainerFilter) -> String {
    match cf {
        ContainerFilter::AllFolders => "AllFolders",
        ContainerFilter::AllInProject => "AllInProject",
        ContainerFilter::AllInProjectPlusShared => "AllInProjectPlusShared",
        ContainerFilter::Current => "Current",
        ContainerFilter::CurrentAndFirstChildren => "CurrentAndFirstChildren",
        ContainerFilter::CurrentAndParents => "CurrentAndParents",
        ContainerFilter::CurrentAndSubfolders => "CurrentAndSubfolders",
        ContainerFilter::CurrentAndSubfoldersPlusShared => "CurrentAndSubfoldersPlusShared",
        ContainerFilter::CurrentPlusProject => "CurrentPlusProject",
        ContainerFilter::CurrentPlusProjectAndShared => "CurrentPlusProjectAndShared",
    }
    .to_string()
}

/// Shorthand for building an optional query parameter pair.
pub(crate) fn opt<V: ToString>(
    key: impl Into<String>,
    value: Option<V>,
) -> Option<(String, String)> {
    value.map(|v| (key.into(), v.to_string()))
}

#[cfg(test)]
mod tests {
    use super::{AuditBehavior, container_filter_to_string, opt};
    use crate::filter::ContainerFilter;

    #[test]
    fn audit_behavior_round_trip_none() {
        let json = serde_json::to_string(&AuditBehavior::None).expect("should serialize");
        assert_eq!(json, "\"NONE\"");
        let recovered: AuditBehavior = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(recovered, AuditBehavior::None);
    }

    #[test]
    fn audit_behavior_round_trip_summary() {
        let json = serde_json::to_string(&AuditBehavior::Summary).expect("should serialize");
        assert_eq!(json, "\"SUMMARY\"");
        let recovered: AuditBehavior = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(recovered, AuditBehavior::Summary);
    }

    #[test]
    fn audit_behavior_round_trip_detailed() {
        let json = serde_json::to_string(&AuditBehavior::Detailed).expect("should serialize");
        assert_eq!(json, "\"DETAILED\"");
        let recovered: AuditBehavior = serde_json::from_str(&json).expect("should deserialize");
        assert_eq!(recovered, AuditBehavior::Detailed);
    }

    fn variant_count(value: AuditBehavior) -> usize {
        match value {
            AuditBehavior::None | AuditBehavior::Summary | AuditBehavior::Detailed => 3,
        }
    }

    #[test]
    fn audit_behavior_variant_count_regression() {
        assert_eq!(variant_count(AuditBehavior::None), 3);
    }

    #[test]
    fn container_filter_to_string_returns_expected_wire_value() {
        assert_eq!(
            container_filter_to_string(ContainerFilter::CurrentAndSubfolders),
            "CurrentAndSubfolders"
        );
    }

    #[test]
    fn opt_returns_none_for_none_value() {
        let pair: Option<(String, String)> = opt("includeMetadata", None::<bool>);
        assert!(pair.is_none());
    }

    #[test]
    fn opt_returns_key_value_pair_for_some_value() {
        let pair = opt("includeMetadata", Some(true));
        assert_eq!(
            pair,
            Some(("includeMetadata".to_string(), "true".to_string()))
        );
    }
}
