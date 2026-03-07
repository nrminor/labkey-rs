//! Permission type, role, and system group constants matching the upstream JS
//! `security/constants.ts` enumerations.

/// Commonly used permission type strings matching the JS `PermissionTypes` enum.
///
/// Each constant is the fully-qualified Java class name of the corresponding
/// LabKey permission. Use these with [`Container::has_effective_permission`]
/// to test whether a user or group has a particular permission.
///
/// This type cannot be instantiated; it exists only to namespace the constants.
///
/// [`Container::has_effective_permission`]: crate::security::Container::has_effective_permission
pub enum PermissionTypes {}

impl PermissionTypes {
    /// `org.labkey.api.security.permissions.AddUserPermission`
    pub const ADD_USER: &str = "org.labkey.api.security.permissions.AddUserPermission";
    /// `org.labkey.api.security.permissions.AdminPermission`
    pub const ADMIN: &str = "org.labkey.api.security.permissions.AdminPermission";
    /// `org.labkey.api.security.permissions.AdminOperationsPermission`
    pub const ADMIN_OPERATIONS: &str =
        "org.labkey.api.security.permissions.AdminOperationsPermission";
    /// `org.labkey.api.security.permissions.ApplicationAdminPermission`
    pub const APPLICATION_ADMIN: &str =
        "org.labkey.api.security.permissions.ApplicationAdminPermission";
    /// `org.labkey.api.audit.permissions.CanSeeAuditLogPermission`
    pub const CAN_SEE_AUDIT_LOG: &str = "org.labkey.api.audit.permissions.CanSeeAuditLogPermission";
    /// `org.labkey.api.security.permissions.SeeGroupDetailsPermission`
    pub const CAN_SEE_GROUP_DETAILS: &str =
        "org.labkey.api.security.permissions.SeeGroupDetailsPermission";
    /// `org.labkey.api.security.permissions.SeeUserDetailsPermission`
    pub const CAN_SEE_USER_DETAILS: &str =
        "org.labkey.api.security.permissions.SeeUserDetailsPermission";
    /// `org.labkey.api.security.permissions.DeletePermission`
    pub const DELETE: &str = "org.labkey.api.security.permissions.DeletePermission";
    /// `org.labkey.api.assay.security.DesignAssayPermission`
    pub const DESIGN_ASSAY: &str = "org.labkey.api.assay.security.DesignAssayPermission";
    /// `org.labkey.api.security.permissions.DesignDataClassPermission`
    pub const DESIGN_DATA_CLASS: &str =
        "org.labkey.api.security.permissions.DesignDataClassPermission";
    /// `org.labkey.api.lists.permissions.DesignListPermission`
    pub const DESIGN_LIST: &str = "org.labkey.api.lists.permissions.DesignListPermission";
    /// `org.labkey.api.security.permissions.DesignSampleTypePermission`
    pub const DESIGN_SAMPLE_SET: &str =
        "org.labkey.api.security.permissions.DesignSampleTypePermission";
    /// `org.labkey.api.inventory.security.StorageDesignPermission`
    pub const DESIGN_STORAGE: &str = "org.labkey.api.inventory.security.StorageDesignPermission";
    /// `org.labkey.api.security.permissions.EditSharedViewPermission`
    pub const EDIT_SHARED_VIEW: &str =
        "org.labkey.api.security.permissions.EditSharedViewPermission";
    /// `org.labkey.api.inventory.security.StorageDataUpdatePermission`
    pub const EDIT_STORAGE_DATA: &str =
        "org.labkey.api.inventory.security.StorageDataUpdatePermission";
    /// `org.labkey.api.security.permissions.InsertPermission`
    pub const INSERT: &str = "org.labkey.api.security.permissions.InsertPermission";
    /// `org.labkey.api.lists.permissions.ManagePicklistsPermission`
    pub const MANAGE_PICKLISTS: &str = "org.labkey.api.lists.permissions.ManagePicklistsPermission";
    /// `org.labkey.api.security.permissions.SampleWorkflowJobPermission`
    pub const MANAGE_SAMPLE_WORKFLOWS: &str =
        "org.labkey.api.security.permissions.SampleWorkflowJobPermission";
    /// `org.labkey.api.security.permissions.MoveEntitiesPermission`
    pub const MOVE_ENTITIES: &str = "org.labkey.api.security.permissions.MoveEntitiesPermission";
    /// `org.labkey.api.security.permissions.QCAnalystPermission`
    pub const QC_ANALYST: &str = "org.labkey.api.security.permissions.QCAnalystPermission";
    /// `org.labkey.api.security.permissions.ReadPermission`
    pub const READ: &str = "org.labkey.api.security.permissions.ReadPermission";
    /// `org.labkey.api.security.permissions.AssayReadPermission`
    pub const READ_ASSAY: &str = "org.labkey.api.security.permissions.AssayReadPermission";
    /// `org.labkey.api.security.permissions.DataClassReadPermission`
    pub const READ_DATA_CLASS: &str = "org.labkey.api.security.permissions.DataClassReadPermission";
    /// `org.labkey.api.security.permissions.MediaReadPermission`
    pub const READ_MEDIA: &str = "org.labkey.api.security.permissions.MediaReadPermission";
    /// `org.labkey.api.security.permissions.NotebookReadPermission`
    pub const READ_NOTEBOOKS: &str = "org.labkey.api.security.permissions.NotebookReadPermission";
    /// `org.labkey.api.security.permissions.ReadSomePermission`
    pub const READ_SOME: &str = "org.labkey.api.security.permissions.ReadSomePermission";
    /// `org.labkey.api.security.permissions.SampleWorkflowDeletePermission`
    pub const SAMPLE_WORKFLOW_DELETE: &str =
        "org.labkey.api.security.permissions.SampleWorkflowDeletePermission";
    /// `org.labkey.api.reports.permissions.ShareReportPermission`
    pub const SHARE_REPORT: &str = "org.labkey.api.reports.permissions.ShareReportPermission";
    /// `org.labkey.api.security.permissions.UpdatePermission`
    pub const UPDATE: &str = "org.labkey.api.security.permissions.UpdatePermission";
    /// `org.labkey.api.security.permissions.UserManagementPermission`
    pub const USER_MANAGEMENT: &str =
        "org.labkey.api.security.permissions.UserManagementPermission";
}

/// Commonly used permission role strings matching the JS `PermissionRoles` enum.
///
/// Each constant is the fully-qualified Java class name of the corresponding
/// LabKey role. Use these when assigning or checking roles on containers,
/// resources, or policies.
///
/// This type cannot be instantiated; it exists only to namespace the constants.
pub enum PermissionRoles {}

impl PermissionRoles {
    /// `org.labkey.api.security.roles.ApplicationAdminRole`
    pub const APPLICATION_ADMIN: &str = "org.labkey.api.security.roles.ApplicationAdminRole";
    /// `org.labkey.api.security.roles.AuthorRole`
    pub const AUTHOR: &str = "org.labkey.api.security.roles.AuthorRole";
    /// `org.labkey.api.security.roles.EditorRole`
    pub const EDITOR: &str = "org.labkey.api.security.roles.EditorRole";
    /// `org.labkey.api.security.roles.EditorWithoutDeleteRole`
    pub const EDITOR_WITHOUT_DELETE: &str = "org.labkey.api.security.roles.EditorWithoutDeleteRole";
    /// `org.labkey.api.security.roles.FolderAdminRole`
    pub const FOLDER_ADMIN: &str = "org.labkey.api.security.roles.FolderAdminRole";
    /// `org.labkey.api.security.roles.ProjectAdminRole`
    pub const PROJECT_ADMIN: &str = "org.labkey.api.security.roles.ProjectAdminRole";
    /// `org.labkey.api.security.roles.ReaderRole`
    pub const READER: &str = "org.labkey.api.security.roles.ReaderRole";
}

/// System group IDs that are constant across all LabKey Server installations.
///
/// These negative IDs are assigned at initial server startup and never change.
/// Use them to reference the built-in system groups by ID rather than by name.
///
/// This type cannot be instantiated; it exists only to namespace the constants.
pub enum SystemGroups {}

impl SystemGroups {
    /// Administrators group (`-1`).
    pub const ADMINISTRATORS: i64 = -1;
    /// Users group (`-2`).
    pub const USERS: i64 = -2;
    /// Guests group (`-3`).
    pub const GUESTS: i64 = -3;
    /// Developers group (`-4`).
    pub const DEVELOPERS: i64 = -4;
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exhaustive verification of every `PermissionTypes` constant against the
    /// upstream JS `PermissionTypes` enum values.
    #[test]
    // Justification: table-driven exhaustive constant verification is inherently
    // long; splitting it would obscure the 1:1 mapping with the JS source.
    #[allow(clippy::too_many_lines)]
    fn permission_types_exhaustive_values() {
        let expected: &[(&str, &str)] = &[
            (
                PermissionTypes::ADD_USER,
                "org.labkey.api.security.permissions.AddUserPermission",
            ),
            (
                PermissionTypes::ADMIN,
                "org.labkey.api.security.permissions.AdminPermission",
            ),
            (
                PermissionTypes::ADMIN_OPERATIONS,
                "org.labkey.api.security.permissions.AdminOperationsPermission",
            ),
            (
                PermissionTypes::APPLICATION_ADMIN,
                "org.labkey.api.security.permissions.ApplicationAdminPermission",
            ),
            (
                PermissionTypes::CAN_SEE_AUDIT_LOG,
                "org.labkey.api.audit.permissions.CanSeeAuditLogPermission",
            ),
            (
                PermissionTypes::CAN_SEE_GROUP_DETAILS,
                "org.labkey.api.security.permissions.SeeGroupDetailsPermission",
            ),
            (
                PermissionTypes::CAN_SEE_USER_DETAILS,
                "org.labkey.api.security.permissions.SeeUserDetailsPermission",
            ),
            (
                PermissionTypes::DELETE,
                "org.labkey.api.security.permissions.DeletePermission",
            ),
            (
                PermissionTypes::DESIGN_ASSAY,
                "org.labkey.api.assay.security.DesignAssayPermission",
            ),
            (
                PermissionTypes::DESIGN_DATA_CLASS,
                "org.labkey.api.security.permissions.DesignDataClassPermission",
            ),
            (
                PermissionTypes::DESIGN_LIST,
                "org.labkey.api.lists.permissions.DesignListPermission",
            ),
            (
                PermissionTypes::DESIGN_SAMPLE_SET,
                "org.labkey.api.security.permissions.DesignSampleTypePermission",
            ),
            (
                PermissionTypes::DESIGN_STORAGE,
                "org.labkey.api.inventory.security.StorageDesignPermission",
            ),
            (
                PermissionTypes::EDIT_SHARED_VIEW,
                "org.labkey.api.security.permissions.EditSharedViewPermission",
            ),
            (
                PermissionTypes::EDIT_STORAGE_DATA,
                "org.labkey.api.inventory.security.StorageDataUpdatePermission",
            ),
            (
                PermissionTypes::INSERT,
                "org.labkey.api.security.permissions.InsertPermission",
            ),
            (
                PermissionTypes::MANAGE_PICKLISTS,
                "org.labkey.api.lists.permissions.ManagePicklistsPermission",
            ),
            (
                PermissionTypes::MANAGE_SAMPLE_WORKFLOWS,
                "org.labkey.api.security.permissions.SampleWorkflowJobPermission",
            ),
            (
                PermissionTypes::MOVE_ENTITIES,
                "org.labkey.api.security.permissions.MoveEntitiesPermission",
            ),
            (
                PermissionTypes::QC_ANALYST,
                "org.labkey.api.security.permissions.QCAnalystPermission",
            ),
            (
                PermissionTypes::READ,
                "org.labkey.api.security.permissions.ReadPermission",
            ),
            (
                PermissionTypes::READ_ASSAY,
                "org.labkey.api.security.permissions.AssayReadPermission",
            ),
            (
                PermissionTypes::READ_DATA_CLASS,
                "org.labkey.api.security.permissions.DataClassReadPermission",
            ),
            (
                PermissionTypes::READ_MEDIA,
                "org.labkey.api.security.permissions.MediaReadPermission",
            ),
            (
                PermissionTypes::READ_NOTEBOOKS,
                "org.labkey.api.security.permissions.NotebookReadPermission",
            ),
            (
                PermissionTypes::READ_SOME,
                "org.labkey.api.security.permissions.ReadSomePermission",
            ),
            (
                PermissionTypes::SAMPLE_WORKFLOW_DELETE,
                "org.labkey.api.security.permissions.SampleWorkflowDeletePermission",
            ),
            (
                PermissionTypes::SHARE_REPORT,
                "org.labkey.api.reports.permissions.ShareReportPermission",
            ),
            (
                PermissionTypes::UPDATE,
                "org.labkey.api.security.permissions.UpdatePermission",
            ),
            (
                PermissionTypes::USER_MANAGEMENT,
                "org.labkey.api.security.permissions.UserManagementPermission",
            ),
        ];

        assert_eq!(
            expected.len(),
            30,
            "PermissionTypes should have exactly 30 constants"
        );
        for (actual, want) in expected {
            assert_eq!(actual, want);
        }
    }

    /// Exhaustive verification of every `PermissionRoles` constant.
    #[test]
    fn permission_roles_exhaustive_values() {
        let expected: &[(&str, &str)] = &[
            (
                PermissionRoles::APPLICATION_ADMIN,
                "org.labkey.api.security.roles.ApplicationAdminRole",
            ),
            (
                PermissionRoles::AUTHOR,
                "org.labkey.api.security.roles.AuthorRole",
            ),
            (
                PermissionRoles::EDITOR,
                "org.labkey.api.security.roles.EditorRole",
            ),
            (
                PermissionRoles::EDITOR_WITHOUT_DELETE,
                "org.labkey.api.security.roles.EditorWithoutDeleteRole",
            ),
            (
                PermissionRoles::FOLDER_ADMIN,
                "org.labkey.api.security.roles.FolderAdminRole",
            ),
            (
                PermissionRoles::PROJECT_ADMIN,
                "org.labkey.api.security.roles.ProjectAdminRole",
            ),
            (
                PermissionRoles::READER,
                "org.labkey.api.security.roles.ReaderRole",
            ),
        ];

        assert_eq!(
            expected.len(),
            7,
            "PermissionRoles should have exactly 7 constants"
        );
        for (actual, want) in expected {
            assert_eq!(actual, want);
        }
    }

    /// Exhaustive verification of every `SystemGroups` constant.
    #[test]
    fn system_groups_exhaustive_values() {
        let expected: &[(i64, i64)] = &[
            (SystemGroups::ADMINISTRATORS, -1),
            (SystemGroups::USERS, -2),
            (SystemGroups::GUESTS, -3),
            (SystemGroups::DEVELOPERS, -4),
        ];

        assert_eq!(
            expected.len(),
            4,
            "SystemGroups should have exactly 4 constants"
        );
        for (actual, want) in expected {
            assert_eq!(actual, want);
        }
    }
}
