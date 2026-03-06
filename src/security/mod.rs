//! Security module scaffolding and shared response vocabulary.

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
