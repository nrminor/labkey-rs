//! Users, groups, containers, permissions, policies, and session management.
//!
//! LabKey's security model is container-based: every project and folder is a
//! container with its own permission policy. Policies assign roles (Reader,
//! Editor, Admin, etc.) to principals (users and groups). This module is split
//! into submodules by concern — containers, groups, users, permissions, policies,
//! and sessions — with shared response types (like [`Container`], [`User`], and
//! [`Policy`]) and well-known constants (like [`PermissionTypes`] and
//! [`PermissionRoles`]).
//!
//! All types are re-exported at this level, so callers can use
//! `labkey_rs::security::GetContainersOptions` without reaching into submodules.

mod constants;
mod container;
mod group;
mod permission;
mod policy;
mod session;
mod types;
mod user;

pub use constants::{PermissionRoles, PermissionTypes, SystemGroups};
pub use container::{
    CreateContainerOptions, DeleteContainerOptions, GetContainersOptions, GetFolderTypesOptions,
    GetFolderTypesResponse, GetModulesOptions, GetModulesResponse, GetReadableContainersOptions,
    MoveContainerOptions, MoveContainerResponse, RenameContainerOptions,
};
pub use group::{
    AddGroupMembersOptions, AddGroupMembersResponse, CreateGroupOptions, CreateGroupResponse,
    DeleteGroupOptions, DeleteGroupResponse, GetGroupsForCurrentUserOptions,
    GetGroupsForCurrentUserResponse, GroupForCurrentUser, RemoveGroupMembersOptions,
    RemoveGroupMembersResponse, RenameGroupOptions, RenameGroupResponse,
};
pub use permission::{
    GetGroupPermissionsOptions, GetRolesOptions, GetSecurableResourcesOptions,
    GetUserPermissionsOptions, GetUserPermissionsResponse, GroupPermissionsResponse,
    PermissionUser, PermissionsContainer, UserPermissionsContainer,
};
pub use policy::{
    DeletePolicyOptions, DeletePolicyResponse, GetPolicyOptions, GetPolicyResponse,
    SavePolicyOptions, SavePolicyResponse,
};
pub use session::{
    DeleteUserOptions, DeleteUserResponse, ImpersonateTarget, ImpersonateUserOptions,
    LogoutOptions, StopImpersonatingOptions, WhoAmIOptions, WhoAmIResponse,
};
pub use types::{
    Container, ContainerFormats, ContainerHierarchy, FolderType, FolderTypeWebPart, Group,
    ModuleInfo, ModuleProperty, Policy, PolicyAssignment, Role, RolePermission, SecurableResource,
    User,
};
pub use user::{
    CreateNewUserOptions, CreateNewUserResponse, CreatedUser, EnsureLoginOptions,
    EnsureLoginResponse, GetUsersOptions, GetUsersResponse, GetUsersWithPermissionsOptions,
};
