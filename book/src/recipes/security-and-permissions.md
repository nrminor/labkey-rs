# Security and Permissions

The `security` module covers user management, group management, permissions, and container security policies. This recipe shows the most common operations. All of these require administrator-level permissions on the target container.

## Checking who you are

Before doing anything else, you can verify your authentication with `who_am_i`:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::security::WhoAmIOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let me = client
    .who_am_i(WhoAmIOptions::builder().build())
    .await?;

println!("Logged in as: {} (id: {})", me.display_name, me.id);
# Ok(())
# }
```

## Listing users

`get_users` retrieves users matching filter criteria:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::security::GetUsersOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let response = client
    .get_users(GetUsersOptions::builder().active(true).build())
    .await?;

for user in &response.users {
    println!("{}: {}", user.user_id, user.display_name);
}
# Ok(())
# }
```

You can filter by `name` (substring match on display name or email), `group_id` (members of a specific group), or `permissions` (users with specific permission strings).

## Creating users

`create_new_user` creates a user account and optionally sends a welcome email:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::security::CreateNewUserOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let response = client
    .create_new_user(
        CreateNewUserOptions::builder()
            .email("alice@example.com".to_string())
            .send_email(true)
            .build(),
    )
    .await?;

println!("Created: success={}", response.success);
# Ok(())
# }
```

## Managing groups

Groups are the primary way to organize users for permission assignment.

### Creating a group

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::security::CreateGroupOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
let group = client
    .create_group(
        CreateGroupOptions::builder()
            .group_name("Research Analysts".to_string())
            .build(),
    )
    .await?;

println!("Created group: {} (id: {})", group.name, group.id);
# Ok(())
# }
```

### Adding members to a group

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::security::AddGroupMembersOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/",
# ))?;
# let group_id = 101;
# let user_ids = vec![1001, 1002, 1003];
let response = client
    .add_group_members(
        AddGroupMembersOptions::builder()
            .group_id(group_id)
            .principal_ids(user_ids)
            .build(),
    )
    .await?;

println!("Added {} members", response.added.len());
# Ok(())
# }
```

Members are identified by their principal IDs (user IDs or group IDs — groups can be members of other groups).

## Checking permissions

### Container-level permissions

`get_group_permissions` returns the permissions assigned to groups in a container:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::security::GetGroupPermissionsOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/MyProject",
# ))?;
let permissions = client
    .get_group_permissions(GetGroupPermissionsOptions::builder().build())
    .await?;

println!("{:#?}", permissions);
# Ok(())
# }
```

### User-specific permissions

`get_user_permissions` checks what a specific user can do in a container:

```rust,no_run
# use labkey_rs::{ClientConfig, Credential, LabkeyClient};
# use labkey_rs::security::GetUserPermissionsOptions;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = LabkeyClient::new(ClientConfig::new(
#     "https://example.com", Credential::Guest, "/MyProject",
# ))?;
let permissions = client
    .get_user_permissions(
        GetUserPermissionsOptions::builder()
            .user_email("alice@example.com".to_string())
            .build(),
    )
    .await?;

println!("{:#?}", permissions);
# Ok(())
# }
```

You can identify the user by either `user_email` or `user_id`. If both are provided, `user_id` takes precedence.

## Permission constants

The `security::constants` module provides string constants for common permission types, roles, and system groups:

```rust,no_run
use labkey_rs::security::constants::{PermissionTypes, PermissionRoles, SystemGroups};

// Permission type strings
let _ = PermissionTypes::READ;
let _ = PermissionTypes::INSERT;
let _ = PermissionTypes::UPDATE;
let _ = PermissionTypes::DELETE;
let _ = PermissionTypes::ADMIN;

// Role strings
let _ = PermissionRoles::READER;
let _ = PermissionRoles::EDITOR;
let _ = PermissionRoles::FOLDER_ADMIN;

// System group IDs
let _ = SystemGroups::ADMINISTRATORS;
let _ = SystemGroups::USERS;
let _ = SystemGroups::GUESTS;
```

These are useful when constructing permission checks or policy assignments programmatically.

## Further reading

The `security` module also provides methods for managing security policies (`get_policy`, `save_policy`), working with containers (`create_container`, `get_containers`), session management (`impersonate_user`, `stop_impersonating`), and more. See the [API reference](https://docs.rs/labkey-rs/latest/labkey_rs/security/index.html) for the full set of security methods.
