//! Shared types reused across multiple API modules.

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

#[cfg(test)]
mod tests {
    use super::AuditBehavior;

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
}
