//! Shared security response models used by `LabKey` security endpoints.

use serde::Deserialize;

/// Container metadata returned by security and project endpoints.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Container {
    /// Container identifier.
    #[serde(default)]
    pub id: Option<String>,
    /// Container path (for example, `/Home/Project/Folder`).
    #[serde(default)]
    pub path: Option<String>,
    /// Display title.
    #[serde(default)]
    pub title: Option<String>,
    /// Internal folder name.
    #[serde(default)]
    pub name: Option<String>,
    /// Folder type value from the server `type` key.
    #[serde(rename = "type", default)]
    pub type_: Option<String>,
    /// Whether this container is the root project.
    #[serde(default)]
    pub is_project: bool,
    /// Optional server-provided format URLs.
    #[serde(default)]
    pub formats: Option<ContainerFormats>,
}

/// Container URL format fields returned by list endpoints.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ContainerFormats {
    /// URL for this container.
    #[serde(default)]
    pub container_path: Option<String>,
    /// URL for this container's children listing.
    #[serde(default)]
    pub children: Option<String>,
}

/// Recursive container hierarchy entry.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ContainerHierarchy {
    /// Container identifier.
    #[serde(default)]
    pub id: Option<String>,
    /// Container path.
    #[serde(default)]
    pub path: Option<String>,
    /// Display title.
    #[serde(default)]
    pub title: Option<String>,
    /// Internal folder name.
    #[serde(default)]
    pub name: Option<String>,
    /// Folder type value from the server `type` key.
    #[serde(rename = "type", default)]
    pub type_: Option<String>,
    /// Child containers in the hierarchy.
    #[serde(default)]
    pub children: Vec<ContainerHierarchy>,
}

/// Key/value module property metadata.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ModuleProperty {
    /// Property key.
    pub name: String,
    /// Property value.
    #[serde(default)]
    pub value: Option<String>,
}

/// Folder type metadata returned by folder-type endpoints.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FolderType {
    /// Folder type name.
    pub name: String,
    /// Folder type label.
    #[serde(default)]
    pub label: Option<String>,
    /// Optional folder type description.
    #[serde(default)]
    pub description: Option<String>,
    /// Web parts configured for this folder type.
    #[serde(default)]
    pub web_parts: Vec<FolderTypeWebPart>,
}

/// Web-part metadata nested under a folder type.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FolderTypeWebPart {
    /// Web-part name.
    pub name: String,
    /// Optional user-visible title.
    #[serde(default)]
    pub title: Option<String>,
    /// Optional web-part location.
    #[serde(default)]
    pub location: Option<String>,
}

/// Module metadata returned by module-listing endpoints.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ModuleInfo {
    /// Module name.
    pub name: String,
    /// Module display label.
    #[serde(default)]
    pub label: Option<String>,
    /// Module version.
    #[serde(default)]
    pub version: Option<String>,
    /// Module properties keyed by property name.
    #[serde(default)]
    pub properties: Vec<ModuleProperty>,
}

/// User principal metadata.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct User {
    /// User id.
    pub user_id: i64,
    /// Email address.
    #[serde(default)]
    pub email: Option<String>,
    /// Display name.
    #[serde(default)]
    pub display_name: Option<String>,
    /// Whether the user account is active.
    #[serde(default)]
    pub active: Option<bool>,
}

/// Group principal metadata.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Group {
    /// Group id.
    pub group_id: i64,
    /// Group name.
    pub name: String,
    /// Display name if provided.
    #[serde(default)]
    pub display_name: Option<String>,
    /// Whether the group can be edited by the current user.
    #[serde(default)]
    pub editable: Option<bool>,
}

/// Role metadata.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Role {
    /// Role unique id.
    #[serde(default)]
    pub unique_name: Option<String>,
    /// Role display name.
    pub name: String,
    /// Permissions assigned to this role.
    #[serde(default)]
    pub permissions: Vec<RolePermission>,
}

/// Permission metadata nested in role responses.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RolePermission {
    /// Permission class name.
    #[serde(default)]
    pub class_name: Option<String>,
    /// Permission display name.
    pub name: String,
}

/// Securable resource metadata.
///
/// Field names match the JS client's `SecurableResource` interface
/// (`security/types.ts`), where the identifier is `id` (not `resourceId`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SecurableResource {
    /// Unique resource identifier (typically a GUID).
    pub id: String,
    /// Resource display name.
    #[serde(default)]
    pub name: Option<String>,
    /// Resource description.
    #[serde(default)]
    pub description: Option<String>,
    /// Fully-qualified Java class name of the resource
    /// (e.g. `"org.labkey.study.model.StudyImpl"`).
    #[serde(default)]
    pub resource_class: Option<String>,
    /// Parent resource identifier.
    #[serde(default)]
    pub parent_id: Option<String>,
    /// Parent container path.
    #[serde(default)]
    pub parent_container_path: Option<String>,
    /// Effective permission unique names for the current user (present when
    /// `includeEffectivePermissions` was requested).
    #[serde(default)]
    pub effective_permissions: Vec<String>,
    /// Child resources.
    #[serde(default)]
    pub children: Vec<SecurableResource>,
}

/// Policy assignment metadata.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PolicyAssignment {
    /// User id for user-scoped assignments.
    #[serde(default)]
    pub user_id: Option<i64>,
    /// Group id for group-scoped assignments.
    #[serde(default)]
    pub group_id: Option<i64>,
    /// Roles assigned to the principal.
    #[serde(default)]
    pub role_names: Vec<String>,
}

/// Security policy metadata.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Policy {
    /// Policy resource id.
    #[serde(default)]
    pub resource_id: Option<String>,
    /// Resource id requested by the caller when policy inheritance is resolved.
    #[serde(default)]
    pub requested_resource_id: Option<String>,
    /// Policy assignments.
    #[serde(default)]
    pub assignments: Vec<PolicyAssignment>,
}

#[cfg(test)]
mod tests {
    use super::{
        Container, ContainerFormats, ContainerHierarchy, FolderType, Group, ModuleInfo, Policy,
        PolicyAssignment, Role, SecurableResource, User,
    };

    #[test]
    fn container_maps_type_field() {
        let value = serde_json::json!({
            "id": "f123",
            "path": "/Home/Project",
            "title": "Project",
            "name": "Project",
            "type": "Folder",
            "isProject": true
        });
        let container: Container = serde_json::from_value(value).expect("valid container");
        assert_eq!(container.type_.as_deref(), Some("Folder"));
        assert!(container.is_project);
    }

    #[test]
    fn container_hierarchy_deserializes_recursive_children() {
        let value = serde_json::json!({
            "id": "root",
            "path": "/Home",
            "type": "Project",
            "children": [
                {
                    "id": "child",
                    "path": "/Home/Child",
                    "type": "Folder",
                    "children": []
                }
            ]
        });
        let hierarchy: ContainerHierarchy = serde_json::from_value(value).expect("valid hierarchy");
        assert_eq!(hierarchy.children.len(), 1);
        assert_eq!(hierarchy.children[0].type_.as_deref(), Some("Folder"));
    }

    #[test]
    fn container_formats_deserializes() {
        let value = serde_json::json!({
            "containerPath": "/project.url",
            "children": "/project/children.url"
        });
        let formats: ContainerFormats = serde_json::from_value(value).expect("valid formats");
        assert_eq!(formats.container_path.as_deref(), Some("/project.url"));
        assert_eq!(formats.children.as_deref(), Some("/project/children.url"));
    }

    #[test]
    fn user_deserializes_minimal_fixture() {
        let value = serde_json::json!({"userId": 101});
        let user: User = serde_json::from_value(value).expect("valid user");
        assert_eq!(user.user_id, 101);
    }

    #[test]
    fn group_deserializes_minimal_fixture() {
        let value = serde_json::json!({"groupId": 10, "name": "Developers"});
        let group: Group = serde_json::from_value(value).expect("valid group");
        assert_eq!(group.group_id, 10);
        assert_eq!(group.name, "Developers");
    }

    #[test]
    fn role_deserializes_minimal_fixture() {
        let value = serde_json::json!({"name": "Editor"});
        let role: Role = serde_json::from_value(value).expect("valid role");
        assert_eq!(role.name, "Editor");
        assert!(role.permissions.is_empty());
    }

    #[test]
    fn policy_deserializes_minimal_fixture() {
        let value = serde_json::json!({"assignments": []});
        let policy: Policy = serde_json::from_value(value).expect("valid policy");
        assert!(policy.assignments.is_empty());
    }

    #[test]
    fn securable_resource_deserializes_minimal_fixture() {
        let value = serde_json::json!({"id": "res-1", "children": []});
        let resource: SecurableResource = serde_json::from_value(value).expect("valid resource");
        assert_eq!(resource.id, "res-1");
        assert!(resource.children.is_empty());
        assert!(resource.name.is_none());
        assert!(resource.description.is_none());
        assert!(resource.resource_class.is_none());
        assert!(resource.parent_id.is_none());
        assert!(resource.parent_container_path.is_none());
        assert!(resource.effective_permissions.is_empty());
    }

    #[test]
    fn securable_resource_deserializes_all_js_fields() {
        let value = serde_json::json!({
            "id": "abc-123",
            "name": "My Study",
            "description": "A study resource",
            "resourceClass": "org.labkey.study.model.StudyImpl",
            "parentId": "parent-456",
            "parentContainerPath": "/Home/Project",
            "effectivePermissions": ["org.labkey.api.security.permissions.ReadPermission"],
            "children": [{
                "id": "child-789",
                "name": "Child",
                "description": "Child resource",
                "resourceClass": "org.labkey.api.data.DatasetDefinition",
                "children": []
            }]
        });
        let resource: SecurableResource = serde_json::from_value(value).expect("valid resource");
        assert_eq!(resource.id, "abc-123");
        assert_eq!(resource.name.as_deref(), Some("My Study"));
        assert_eq!(resource.description.as_deref(), Some("A study resource"));
        assert_eq!(
            resource.resource_class.as_deref(),
            Some("org.labkey.study.model.StudyImpl")
        );
        assert_eq!(resource.parent_id.as_deref(), Some("parent-456"));
        assert_eq!(
            resource.parent_container_path.as_deref(),
            Some("/Home/Project")
        );
        assert_eq!(resource.effective_permissions.len(), 1);
        assert_eq!(resource.children.len(), 1);
        assert_eq!(resource.children[0].id, "child-789");
    }

    #[test]
    fn module_info_deserializes_minimal_fixture() {
        let value = serde_json::json!({"name": "core", "properties": []});
        let module_info: ModuleInfo = serde_json::from_value(value).expect("valid module info");
        assert_eq!(module_info.name, "core");
        assert!(module_info.properties.is_empty());
    }

    #[test]
    fn folder_type_deserializes_minimal_fixture() {
        let value = serde_json::json!({"name": "Collaboration", "webParts": []});
        let folder_type: FolderType = serde_json::from_value(value).expect("valid folder type");
        assert_eq!(folder_type.name, "Collaboration");
        assert!(folder_type.web_parts.is_empty());
    }

    #[test]
    fn policy_assignment_deserializes_role_names() {
        let value = serde_json::json!({"groupId": 5, "roleNames": ["Editor"]});
        let assignment: PolicyAssignment =
            serde_json::from_value(value).expect("valid policy assignment");
        assert_eq!(assignment.group_id, Some(5));
        assert_eq!(assignment.role_names, vec!["Editor"]);
    }
}
