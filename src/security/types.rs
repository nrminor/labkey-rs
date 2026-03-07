//! Shared security response models used by LabKey security endpoints.

use serde::{Deserialize, Serialize};

/// Container metadata returned by security and project endpoints.
///
/// Fields match the JS `Container` interface (`constants.ts`). All fields
/// beyond `id` are optional because different endpoints return different
/// subsets of the full container object.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Container {
    /// Container identifier (GUID).
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
    /// Active module names for this container.
    #[serde(default)]
    pub active_modules: Vec<String>,
    /// Effective permission unique names for the current user (present when
    /// requested via `includeEffectivePermissions`).
    #[serde(default)]
    pub effective_permissions: Vec<String>,
    /// Folder type name string.
    #[serde(default)]
    pub folder_type: Option<String>,
    /// Server-provided date/time/number format strings.
    #[serde(default)]
    pub formats: Option<ContainerFormats>,
    /// Whether this container has a restricted active module.
    #[serde(default)]
    pub has_restricted_active_module: Option<bool>,
    /// URL to the container icon.
    #[serde(default)]
    pub icon_href: Option<String>,
    /// Whether this container is archived.
    #[serde(default)]
    pub is_archived: Option<bool>,
    /// Whether this container is a container tab.
    #[serde(default)]
    pub is_container_tab: Option<bool>,
    /// Whether this container is a workbook.
    #[serde(default)]
    pub is_workbook: Option<bool>,
    /// Parent container identifier.
    #[serde(default)]
    pub parent_id: Option<String>,
    /// Parent container path.
    #[serde(default)]
    pub parent_path: Option<String>,
    /// Sort order within the parent container.
    #[serde(default)]
    pub sort_order: Option<i64>,
    /// Start URL for this container.
    #[serde(default)]
    pub start_url: Option<String>,
}

impl Container {
    /// Check whether the current user has a specific effective permission on
    /// this container.
    ///
    /// The `effective_permissions` field must have been populated by the server
    /// (request the container with `includeEffectivePermissions=true`). If the
    /// field is empty, this always returns `false`.
    ///
    /// Matches the JS `hasEffectivePermission` helper from `Permission.ts`.
    #[must_use]
    pub fn has_effective_permission(&self, permission: &str) -> bool {
        self.effective_permissions.iter().any(|p| p == permission)
    }
}

/// Date, time, and number format strings associated with a container.
///
/// The JS `Container` interface nests these under a `formats` object with
/// four string fields. The server may also include other keys, so unknown
/// fields are captured in `extra`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ContainerFormats {
    /// Date format string (e.g. `"yyyy-MM-dd"`).
    #[serde(default)]
    pub date_format: Option<String>,
    /// Date-time format string.
    #[serde(default)]
    pub date_time_format: Option<String>,
    /// Number format string.
    #[serde(default)]
    pub number_format: Option<String>,
    /// Time format string.
    #[serde(default)]
    pub time_format: Option<String>,
    /// Additional server-provided format keys.
    #[serde(flatten)]
    pub extra: std::collections::HashMap<String, serde_json::Value>,
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
    /// Effective permission unique names for the current user (present when
    /// requested via `includeEffectivePermissions`).
    #[serde(default)]
    pub effective_permissions: Vec<String>,
    /// Module properties associated with this container.
    #[serde(default)]
    pub module_properties: Vec<ModuleProperty>,
}

/// Key/value module property metadata.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ModuleProperty {
    /// Property key.
    pub name: String,
    /// Property value. Typed as `serde_json::Value` to match the JS `any`
    /// typing — the server may send strings, numbers, booleans, or objects.
    #[serde(default)]
    pub value: Option<serde_json::Value>,
    /// Effective (inherited) value after property inheritance resolution.
    #[serde(default)]
    pub effective_value: Option<serde_json::Value>,
    /// Module that defines this property.
    #[serde(default)]
    pub module: Option<String>,
}

/// Folder type metadata returned by folder-type endpoints.
///
/// Matches the JS `FolderType` interface (`security/Container.ts`).
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
    /// Active module names for this folder type.
    #[serde(default)]
    pub active_modules: Vec<String>,
    /// Default module name.
    #[serde(default)]
    pub default_module: Option<String>,
    /// Whether this folder type is a workbook type.
    #[serde(default)]
    pub workbook_type: Option<bool>,
    /// Preferred (removable) web parts for this folder type.
    #[serde(default)]
    pub preferred_web_parts: Vec<FolderTypeWebPart>,
    /// Required (non-removable) web parts for this folder type.
    #[serde(default)]
    pub required_web_parts: Vec<FolderTypeWebPart>,
}

/// Web-part metadata nested under a folder type.
///
/// Matches the JS `FolderTypeWebParts` interface (`security/Container.ts`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct FolderTypeWebPart {
    /// Web-part name.
    pub name: String,
    /// Auto-set properties for this web part.
    #[serde(default)]
    pub properties: std::collections::HashMap<String, serde_json::Value>,
}

/// Module metadata returned by module-listing endpoints.
///
/// Includes fields from both the JS `GetModulesModules` interface and
/// additional server-provided fields (`label`, `version`, `properties`)
/// that appear in real responses but are absent from the JS type definition.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ModuleInfo {
    /// Module name.
    pub name: String,
    /// Whether this module is active in the current container.
    #[serde(default)]
    pub active: Option<bool>,
    /// Whether this module is enabled on the server.
    #[serde(default)]
    pub enabled: Option<bool>,
    /// Module display label (server-provided, not in JS type).
    #[serde(default)]
    pub label: Option<String>,
    /// Whether this module is required by the server.
    #[serde(default)]
    pub required: Option<bool>,
    /// Whether this module requires site-level permissions.
    #[serde(default)]
    pub require_site_permission: Option<bool>,
    /// Tab name for this module in the UI.
    #[serde(default)]
    pub tab_name: Option<String>,
    /// Module version (server-provided, not in JS type).
    #[serde(default)]
    pub version: Option<String>,
    /// Module properties keyed by property name (server-provided, not in JS type).
    #[serde(default)]
    pub properties: Vec<ModuleProperty>,
}

/// User principal metadata.
///
/// The JS `User` interface uses `id` as the primary identifier, while some
/// server endpoints send `userId`. The `alias` accepts both.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct User {
    /// User id. Accepts both `"userId"` and `"id"` from the wire.
    #[serde(alias = "id")]
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
    /// Avatar URL.
    #[serde(default)]
    pub avatar: Option<String>,
    /// Phone number.
    #[serde(default)]
    pub phone: Option<String>,
}

/// Group principal metadata.
///
/// Matches the JS `Group` interface (`security/types.ts`). The server sends
/// `id` in `getGroupPerms.api` responses, but some endpoints use `groupId`.
/// The `alias` accepts both wire keys. Three deprecated JS fields (`role`,
/// `roleLabel`, `permissions`) are intentionally omitted.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Group {
    /// Group id.
    ///
    /// Accepts both `"id"` (JS wire format from `getGroupPerms.api`) and
    /// `"groupId"` (used by some group-management endpoints).
    #[serde(alias = "id")]
    pub group_id: i64,
    /// Group name.
    pub name: String,
    /// Display name if provided.
    #[serde(default)]
    pub display_name: Option<String>,
    /// Whether the group can be edited by the current user.
    #[serde(default)]
    pub editable: Option<bool>,
    /// Effective permission unique names for the current user on this group.
    #[serde(default)]
    pub effective_permissions: Vec<String>,
    /// Nested subgroups (recursive).
    #[serde(default)]
    pub groups: Vec<Group>,
    /// Whether this is a project-level group.
    #[serde(default)]
    pub is_project_group: Option<bool>,
    /// Whether this is a system group.
    #[serde(default)]
    pub is_system_group: Option<bool>,
    /// Role unique names assigned to this group.
    #[serde(default)]
    pub roles: Vec<String>,
    /// Group type code (e.g. `"g"` for group, `"r"` for role, `"m"` for module).
    #[serde(rename = "type", default)]
    pub type_: Option<String>,
}

/// Role metadata.
///
/// Matches the JS `Role` interface (`security/Permission.ts`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Role {
    /// Role unique id (fully-qualified Java class name).
    #[serde(default)]
    pub unique_name: Option<String>,
    /// Role display name.
    pub name: String,
    /// Human-readable role description.
    #[serde(default)]
    pub description: Option<String>,
    /// Principal ids excluded from this role.
    #[serde(default)]
    pub excluded_principals: Vec<i64>,
    /// Permissions assigned to this role.
    #[serde(default)]
    pub permissions: Vec<RolePermission>,
    /// Module that defines this role.
    #[serde(default)]
    pub source_module: Option<String>,
}

/// Permission metadata nested in role responses.
///
/// Matches the JS `RolePermission` interface (`security/Permission.ts`).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct RolePermission {
    /// Unique permission name (typically a fully-qualified Java class name).
    ///
    /// Matches the JS `uniqueName` field. The server may also send this as
    /// `className` in some response shapes, so both keys are accepted.
    #[serde(default, alias = "className")]
    pub unique_name: Option<String>,
    /// Permission display name.
    pub name: String,
    /// Human-readable permission description.
    #[serde(default)]
    pub description: Option<String>,
    /// Module that defines this permission.
    #[serde(default)]
    pub source_module: Option<String>,
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

/// A single role assignment for a principal (user or group).
///
/// Each assignment pairs one principal (`user_id`) with one role. The server
/// sends one assignment entry per role-principal pair, matching the JS
/// `Policy` interface (`security/Policy.ts:51-60`).
///
/// Despite the field name, `user_id` is a principal id that can refer to
/// either a user or a group — the LabKey security model treats both as
/// principals.
#[derive(Debug, Clone, Serialize, Deserialize, bon::Builder)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PolicyAssignment {
    /// Principal id (user or group).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<i64>,
    /// Fully-qualified role class name assigned to this principal
    /// (e.g. `"org.labkey.api.security.roles.EditorRole"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

/// Security policy metadata.
///
/// Matches the JS `Policy` interface (`security/Policy.ts`). Derives both
/// `Deserialize` (for `get_policy` responses) and `Serialize` (for
/// `save_policy` requests). The `requested_resource_id` field is a
/// client-side annotation injected by `get_policy` and is skipped during
/// serialization.
#[derive(Debug, Clone, Serialize, Deserialize, bon::Builder)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Policy {
    /// Policy resource id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_id: Option<String>,
    /// Resource id requested by the caller when policy inheritance is resolved.
    /// This is a client-side annotation, not sent to the server.
    #[serde(default, skip_serializing)]
    pub requested_resource_id: Option<String>,
    /// Last modification timestamp (ISO date string).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    /// Last modification timestamp in epoch milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_millis: Option<i64>,
    /// Policy assignments.
    #[serde(default)]
    pub assignments: Vec<PolicyAssignment>,
}

#[cfg(test)]
mod tests {
    use super::{
        Container, ContainerFormats, ContainerHierarchy, FolderType, Group, ModuleInfo,
        ModuleProperty, Policy, PolicyAssignment, Role, SecurableResource, User,
    };

    #[test]
    fn container_deserializes_full_js_shape() {
        let value = serde_json::json!({
            "id": "f123",
            "path": "/Home/Project",
            "title": "Project",
            "name": "Project",
            "type": "Folder",
            "isProject": true,
            "activeModules": ["Core", "Study"],
            "effectivePermissions": ["org.labkey.api.security.permissions.ReadPermission"],
            "folderType": "Study",
            "formats": {
                "dateFormat": "yyyy-MM-dd",
                "dateTimeFormat": "yyyy-MM-dd HH:mm",
                "numberFormat": "#,##0.##",
                "timeFormat": "HH:mm:ss"
            },
            "hasRestrictedActiveModule": false,
            "iconHref": "/icons/folder.png",
            "isArchived": false,
            "isContainerTab": false,
            "isWorkbook": true,
            "parentId": "parent-abc",
            "parentPath": "/Home",
            "sortOrder": 3,
            "startUrl": "/Home/Project/project-begin.view"
        });
        let container: Container = serde_json::from_value(value).expect("valid container");
        assert_eq!(container.type_.as_deref(), Some("Folder"));
        assert!(container.is_project);
        assert_eq!(container.active_modules, vec!["Core", "Study"]);
        assert_eq!(container.effective_permissions.len(), 1);
        assert_eq!(container.folder_type.as_deref(), Some("Study"));
        assert!(container.formats.is_some());
        let formats = container.formats.expect("formats should be present");
        assert_eq!(formats.date_format.as_deref(), Some("yyyy-MM-dd"));
        assert_eq!(formats.time_format.as_deref(), Some("HH:mm:ss"));
        assert_eq!(container.has_restricted_active_module, Some(false));
        assert_eq!(container.icon_href.as_deref(), Some("/icons/folder.png"));
        assert_eq!(container.is_archived, Some(false));
        assert_eq!(container.is_container_tab, Some(false));
        assert_eq!(container.is_workbook, Some(true));
        assert_eq!(container.parent_id.as_deref(), Some("parent-abc"));
        assert_eq!(container.parent_path.as_deref(), Some("/Home"));
        assert_eq!(container.sort_order, Some(3));
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
            "dateFormat": "yyyy-MM-dd",
            "dateTimeFormat": "yyyy-MM-dd HH:mm",
            "numberFormat": "#,##0.##",
            "timeFormat": "HH:mm:ss"
        });
        let formats: ContainerFormats = serde_json::from_value(value).expect("valid formats");
        assert_eq!(formats.date_format.as_deref(), Some("yyyy-MM-dd"));
        assert_eq!(
            formats.date_time_format.as_deref(),
            Some("yyyy-MM-dd HH:mm")
        );
        assert_eq!(formats.number_format.as_deref(), Some("#,##0.##"));
        assert_eq!(formats.time_format.as_deref(), Some("HH:mm:ss"));
    }

    #[test]
    fn user_deserializes_minimal_fixture() {
        let value = serde_json::json!({"userId": 101});
        let user: User = serde_json::from_value(value).expect("valid user");
        assert_eq!(user.user_id, 101);
    }

    #[test]
    fn group_deserializes_minimal_fixture() {
        // Server sends "id" in getGroupPerms responses (JS wire format).
        let value = serde_json::json!({"id": 10, "name": "Developers"});
        let group: Group = serde_json::from_value(value).expect("valid group");
        assert_eq!(group.group_id, 10);
        assert_eq!(group.name, "Developers");
    }

    #[test]
    fn group_deserializes_group_id_alias() {
        // Some endpoints use "groupId" instead of "id".
        let value = serde_json::json!({"groupId": 10, "name": "Developers"});
        let group: Group = serde_json::from_value(value).expect("valid group");
        assert_eq!(group.group_id, 10);
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
        let value = serde_json::json!({
            "name": "Collaboration",
            "preferredWebParts": [],
            "requiredWebParts": []
        });
        let folder_type: FolderType = serde_json::from_value(value).expect("valid folder type");
        assert_eq!(folder_type.name, "Collaboration");
        assert!(folder_type.preferred_web_parts.is_empty());
        assert!(folder_type.required_web_parts.is_empty());
    }

    #[test]
    fn policy_assignment_deserializes_js_wire_format() {
        // Server sends one role per assignment with userId (JS Policy interface).
        let value = serde_json::json!({
            "role": "org.labkey.api.security.roles.EditorRole",
            "userId": 1001
        });
        let assignment: PolicyAssignment =
            serde_json::from_value(value).expect("valid policy assignment");
        assert_eq!(assignment.user_id, Some(1001));
        assert_eq!(
            assignment.role,
            Some("org.labkey.api.security.roles.EditorRole".to_string())
        );
    }

    #[test]
    fn group_deserializes_full_js_shape() {
        let value = serde_json::json!({
            "id": 42,
            "name": "Developers",
            "effectivePermissions": ["org.labkey.api.security.permissions.ReadPermission"],
            "groups": [{"id": 99, "name": "SubGroup"}],
            "isProjectGroup": true,
            "isSystemGroup": false,
            "roles": ["org.labkey.security.roles.EditorRole"],
            "type": "g"
        });
        let group: Group = serde_json::from_value(value).expect("valid group");
        assert_eq!(group.group_id, 42);
        assert_eq!(group.effective_permissions.len(), 1);
        assert_eq!(group.groups.len(), 1);
        assert_eq!(group.groups[0].name, "SubGroup");
        assert_eq!(group.is_project_group, Some(true));
        assert_eq!(group.is_system_group, Some(false));
        assert_eq!(group.roles, vec!["org.labkey.security.roles.EditorRole"]);
        assert_eq!(group.type_.as_deref(), Some("g"));
    }

    #[test]
    fn role_deserializes_full_js_shape() {
        let value = serde_json::json!({
            "uniqueName": "org.labkey.security.roles.EditorRole",
            "name": "Editor",
            "description": "Can edit data",
            "excludedPrincipals": [1001, 1002],
            "permissions": [{
                "uniqueName": "org.labkey.api.security.permissions.ReadPermission",
                "name": "Read",
                "description": "Can read data",
                "sourceModule": "Core"
            }],
            "sourceModule": "Core"
        });
        let role: Role = serde_json::from_value(value).expect("valid role");
        assert_eq!(role.description.as_deref(), Some("Can edit data"));
        assert_eq!(role.excluded_principals, vec![1001, 1002]);
        assert_eq!(role.source_module.as_deref(), Some("Core"));
        assert_eq!(
            role.permissions[0].description.as_deref(),
            Some("Can read data")
        );
        assert_eq!(role.permissions[0].source_module.as_deref(), Some("Core"));
    }

    #[test]
    fn policy_deserializes_full_js_shape() {
        let value = serde_json::json!({
            "resourceId": "resource-1",
            "requestedResourceId": "requested-1",
            "modified": "2026-03-05T12:00:00Z",
            "modifiedMillis": 1_772_870_400_000_i64,
            "assignments": [{
                "role": "org.labkey.security.roles.EditorRole",
                "userId": 1001
            }]
        });
        let policy: Policy = serde_json::from_value(value).expect("valid policy");
        assert_eq!(policy.modified.as_deref(), Some("2026-03-05T12:00:00Z"));
        assert_eq!(policy.modified_millis, Some(1_772_870_400_000));
        assert_eq!(policy.assignments.len(), 1);
    }

    #[test]
    fn policy_serializes_without_requested_resource_id() {
        let policy = Policy {
            resource_id: Some("resource-1".to_string()),
            requested_resource_id: Some("should-be-skipped".to_string()),
            modified: None,
            modified_millis: None,
            assignments: vec![PolicyAssignment {
                user_id: Some(1001),
                role: Some("org.labkey.security.roles.EditorRole".to_string()),
            }],
        };
        let json = serde_json::to_value(&policy).expect("serialize policy");
        assert!(json.get("requestedResourceId").is_none());
        assert_eq!(json["resourceId"], "resource-1");
        assert_eq!(json["assignments"][0]["userId"], 1001);
    }

    #[test]
    fn user_deserializes_full_js_shape() {
        let value = serde_json::json!({
            "id": 42,
            "email": "user@example.com",
            "displayName": "Test User",
            "active": true,
            "avatar": "/avatars/42.png",
            "phone": "555-0100"
        });
        let user: User = serde_json::from_value(value).expect("valid user");
        assert_eq!(user.user_id, 42);
        assert_eq!(user.avatar.as_deref(), Some("/avatars/42.png"));
        assert_eq!(user.phone.as_deref(), Some("555-0100"));
    }

    #[test]
    fn folder_type_deserializes_full_js_shape() {
        let value = serde_json::json!({
            "name": "Study",
            "label": "Study Folder",
            "description": "For clinical studies",
            "activeModules": ["Study", "Pipeline"],
            "defaultModule": "Study",
            "workbookType": false,
            "preferredWebParts": [
                {"name": "Study Overview", "properties": {"showHeader": true}}
            ],
            "requiredWebParts": [
                {"name": "Data Pipeline", "properties": {}}
            ]
        });
        let ft: FolderType = serde_json::from_value(value).expect("valid folder type");
        assert_eq!(ft.active_modules, vec!["Study", "Pipeline"]);
        assert_eq!(ft.default_module.as_deref(), Some("Study"));
        assert_eq!(ft.workbook_type, Some(false));
        assert_eq!(ft.preferred_web_parts.len(), 1);
        assert_eq!(ft.preferred_web_parts[0].name, "Study Overview");
        assert!(
            ft.preferred_web_parts[0]
                .properties
                .contains_key("showHeader")
        );
        assert_eq!(ft.required_web_parts.len(), 1);
        assert_eq!(ft.required_web_parts[0].name, "Data Pipeline");
    }

    #[test]
    fn module_info_deserializes_full_shape() {
        let value = serde_json::json!({
            "name": "Study",
            "active": true,
            "enabled": true,
            "required": false,
            "requireSitePermission": false,
            "tabName": "Study"
        });
        let mi: ModuleInfo = serde_json::from_value(value).expect("valid module info");
        assert_eq!(mi.active, Some(true));
        assert_eq!(mi.enabled, Some(true));
        assert_eq!(mi.required, Some(false));
        assert_eq!(mi.require_site_permission, Some(false));
        assert_eq!(mi.tab_name.as_deref(), Some("Study"));
    }

    #[test]
    fn has_effective_permission_finds_matching_permission() {
        let container: Container = serde_json::from_value(serde_json::json!({
            "effectivePermissions": [
                "org.labkey.api.security.permissions.ReadPermission",
                "org.labkey.api.security.permissions.InsertPermission"
            ]
        }))
        .expect("valid container");

        assert!(
            container
                .has_effective_permission("org.labkey.api.security.permissions.ReadPermission")
        );
        assert!(
            !container
                .has_effective_permission("org.labkey.api.security.permissions.DeletePermission")
        );
    }

    #[test]
    fn has_effective_permission_returns_false_when_empty() {
        let container: Container =
            serde_json::from_value(serde_json::json!({})).expect("valid container");
        assert!(
            !container
                .has_effective_permission("org.labkey.api.security.permissions.ReadPermission")
        );
    }

    #[test]
    fn container_hierarchy_deserializes_effective_permissions_and_module_properties() {
        let value = serde_json::json!({
            "id": "root",
            "path": "/Home",
            "type": "Project",
            "children": [],
            "effectivePermissions": [
                "org.labkey.api.security.permissions.ReadPermission",
                "org.labkey.api.security.permissions.InsertPermission"
            ],
            "moduleProperties": [
                {
                    "name": "site.prefix",
                    "value": "LAB",
                    "effectiveValue": "LAB",
                    "module": "core"
                }
            ]
        });
        let h: ContainerHierarchy = serde_json::from_value(value).expect("valid hierarchy");
        assert_eq!(h.effective_permissions.len(), 2);
        assert_eq!(
            h.effective_permissions[0],
            "org.labkey.api.security.permissions.ReadPermission"
        );
        assert_eq!(h.module_properties.len(), 1);
        assert_eq!(h.module_properties[0].name, "site.prefix");
        assert_eq!(
            h.module_properties[0].value,
            Some(serde_json::Value::String("LAB".into()))
        );
        assert_eq!(h.module_properties[0].module.as_deref(), Some("core"));
    }

    #[test]
    fn container_hierarchy_defaults_new_fields_when_absent() {
        let value = serde_json::json!({
            "id": "root",
            "path": "/Home",
            "children": []
        });
        let h: ContainerHierarchy = serde_json::from_value(value).expect("valid hierarchy");
        assert!(h.effective_permissions.is_empty());
        assert!(h.module_properties.is_empty());
    }

    #[test]
    fn module_property_deserializes_all_fields() {
        let value = serde_json::json!({
            "name": "site.prefix",
            "value": "LAB",
            "effectiveValue": 42,
            "module": "core"
        });
        let mp: ModuleProperty = serde_json::from_value(value).expect("valid module property");
        assert_eq!(mp.name, "site.prefix");
        assert_eq!(mp.value, Some(serde_json::Value::String("LAB".into())));
        assert_eq!(
            mp.effective_value,
            Some(serde_json::Value::Number(42.into()))
        );
        assert_eq!(mp.module.as_deref(), Some("core"));
    }

    #[test]
    fn module_property_value_accepts_non_string_json() {
        let value = serde_json::json!({
            "name": "max.retries",
            "value": 5,
            "effectiveValue": true
        });
        let mp: ModuleProperty = serde_json::from_value(value).expect("valid module property");
        assert_eq!(mp.value, Some(serde_json::Value::Number(5.into())));
        assert_eq!(mp.effective_value, Some(serde_json::Value::Bool(true)));
        assert!(mp.module.is_none());
    }

    #[test]
    fn module_property_minimal_deserialization() {
        let value = serde_json::json!({"name": "some.prop"});
        let mp: ModuleProperty = serde_json::from_value(value).expect("valid module property");
        assert_eq!(mp.name, "some.prop");
        assert!(mp.value.is_none());
        assert!(mp.effective_value.is_none());
        assert!(mp.module.is_none());
    }

    #[test]
    fn module_property_value_accepts_nested_json_objects() {
        let value = serde_json::json!({
            "name": "complex.config",
            "value": {"host": "localhost", "port": 8080},
            "effectiveValue": [1, 2, 3]
        });
        let mp: ModuleProperty = serde_json::from_value(value).expect("valid module property");
        let obj = mp.value.as_ref().and_then(serde_json::Value::as_object);
        assert!(obj.is_some());
        assert_eq!(
            obj.and_then(|o| o.get("host")),
            Some(&serde_json::json!("localhost"))
        );
        assert!(
            mp.effective_value
                .as_ref()
                .and_then(serde_json::Value::as_array)
                .is_some()
        );
    }

    #[test]
    fn module_info_deserializes_with_widened_properties() {
        let value = serde_json::json!({
            "name": "core",
            "active": true,
            "properties": [
                {
                    "name": "site.prefix",
                    "value": "LAB",
                    "effectiveValue": "LAB-INHERITED",
                    "module": "core"
                },
                {
                    "name": "max.retries",
                    "value": 5
                }
            ]
        });
        let mi: ModuleInfo = serde_json::from_value(value).expect("valid module info");
        assert_eq!(mi.properties.len(), 2);
        assert_eq!(mi.properties[0].name, "site.prefix");
        assert_eq!(
            mi.properties[0].value,
            Some(serde_json::Value::String("LAB".into()))
        );
        assert_eq!(
            mi.properties[0].effective_value,
            Some(serde_json::Value::String("LAB-INHERITED".into()))
        );
        assert_eq!(mi.properties[0].module.as_deref(), Some("core"));
        assert_eq!(mi.properties[1].name, "max.retries");
        assert_eq!(
            mi.properties[1].value,
            Some(serde_json::Value::Number(5.into()))
        );
        assert!(mi.properties[1].effective_value.is_none());
        assert!(mi.properties[1].module.is_none());
    }
}
