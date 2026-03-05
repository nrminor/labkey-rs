//! Security module scaffolding and shared response vocabulary.

mod container;
mod group;
mod types;
mod user;

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
pub use types::{
    Container, ContainerFormats, ContainerHierarchy, FolderType, FolderTypeWebPart, Group,
    ModuleInfo, ModuleProperty, Policy, PolicyAssignment, Role, RolePermission, SecurableResource,
    User,
};
pub use user::{
    CreateNewUserOptions, CreateNewUserResponse, CreatedUser, EnsureLoginOptions,
    EnsureLoginResponse, GetUsersOptions, GetUsersResponse, GetUsersWithPermissionsOptions,
};
