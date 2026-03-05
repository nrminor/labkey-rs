//! Container-focused security endpoints.

use reqwest::StatusCode;
use serde::{Deserialize, Serialize};

use crate::{
    client::LabkeyClient,
    common::opt,
    error::LabkeyError,
    security::{Container, ContainerHierarchy},
};

/// Options for [`LabkeyClient::create_container`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct CreateContainerOptions {
    /// Required name of the new project/folder/workbook.
    pub name: String,
    /// Optional container override for where the container is created.
    pub container_path: Option<String>,
    /// Optional description, used primarily for workbook creation.
    pub description: Option<String>,
    /// Optional folder type name to apply.
    pub folder_type: Option<String>,
    /// Optional workbook flag.
    pub is_workbook: Option<bool>,
    /// Optional title, used primarily for workbook creation.
    pub title: Option<String>,
}

/// Options for [`LabkeyClient::delete_container`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct DeleteContainerOptions {
    /// Optional container override for which container is deleted.
    pub container_path: Option<String>,
    /// Optional audit comment for why the container was deleted.
    pub comment: Option<String>,
}

/// Options for [`LabkeyClient::rename_container`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct RenameContainerOptions {
    /// Optional container override for which container is renamed.
    pub container_path: Option<String>,
    /// New container name.
    pub name: Option<String>,
    /// New container title.
    pub title: Option<String>,
    /// Add an alias for the old name when renaming.
    pub add_alias: Option<bool>,
}

/// Options for [`LabkeyClient::get_containers`].
#[derive(Debug, Clone, bon::Builder)]
#[non_exhaustive]
pub struct GetContainersOptions {
    /// One or more container ids and/or full paths.
    pub containers: Option<Vec<String>>,
    /// Optional container override to run the request under.
    pub container_path: Option<String>,
    /// Optional recursion depth when `include_subfolders` is true.
    pub depth: Option<i32>,
    /// Include descendant containers.
    pub include_subfolders: Option<bool>,
    /// Include effective permissions in each returned container.
    pub include_effective_permissions: Option<bool>,
    /// Include workbook children.
    pub include_workbook_children: Option<bool>,
    /// Include standard container properties.
    pub include_standard_properties: Option<bool>,
    /// Include inheritable format metadata.
    pub include_inheritable_formats: Option<bool>,
    /// Include module properties for the specified modules.
    pub module_properties: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateContainerBody {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    folder_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_workbook: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

#[derive(Debug, Serialize)]
struct DeleteContainerBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    comment: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct RenameContainerBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    add_alias: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetContainersEnvelope {
    containers: Vec<ContainerHierarchy>,
}

fn build_get_containers_params(options: &GetContainersOptions) -> Vec<(String, String)> {
    let mut params: Vec<(String, String)> = [
        opt("includeSubfolders", options.include_subfolders),
        opt("depth", options.depth),
        opt(
            "includeEffectivePermissions",
            options.include_effective_permissions,
        ),
        opt("includeWorkbookChildren", options.include_workbook_children),
        opt(
            "includeStandardProperties",
            options.include_standard_properties,
        ),
        opt(
            "includeInheritableFormats",
            options.include_inheritable_formats,
        ),
    ]
    .into_iter()
    .flatten()
    .collect();

    if let Some(containers) = options
        .containers
        .as_ref()
        .filter(|containers| !containers.is_empty())
    {
        if containers.len() > 1 {
            params.push(("multipleContainers".to_string(), "true".to_string()));
        }
        params.extend(
            containers
                .iter()
                .cloned()
                .map(|container| ("container".to_string(), container)),
        );
    }

    if let Some(module_properties) = options
        .module_properties
        .as_ref()
        .filter(|values| !values.is_empty())
    {
        params.extend(
            module_properties
                .iter()
                .cloned()
                .map(|module| ("moduleProperties".to_string(), module)),
        );
    }

    params
}

fn extract_containers(
    response: &serde_json::Value,
) -> Result<Vec<ContainerHierarchy>, LabkeyError> {
    if response.get("containers").is_some() {
        return serde_json::from_value::<GetContainersEnvelope>(response.clone())
            .map(|envelope| envelope.containers)
            .map_err(|_| LabkeyError::UnexpectedResponse {
                status: StatusCode::OK,
                text: format!("invalid getContainers envelope response: {response}"),
            });
    }

    serde_json::from_value::<ContainerHierarchy>(response.clone())
        .map(|container| vec![container])
        .map_err(|_| LabkeyError::UnexpectedResponse {
            status: StatusCode::OK,
            text: format!("invalid getContainers single-container response: {response}"),
        })
}

fn validate_rename_input(name: Option<&str>, title: Option<&str>) -> Result<(), LabkeyError> {
    if name.is_none() && title.is_none() {
        return Err(LabkeyError::InvalidInput(
            "rename_container requires at least one of `name` or `title`".to_string(),
        ));
    }

    Ok(())
}

impl LabkeyClient {
    /// Create a project, folder, or workbook container.
    ///
    /// Sends a POST request to `core-createContainer.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the HTTP request fails, the server returns
    /// an error response, or the response body cannot be deserialized.
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
    /// use labkey_rs::security::CreateContainerOptions;
    ///
    /// let container = client
    ///     .create_container(
    ///         CreateContainerOptions::builder()
    ///             .name("AssayFolder".to_string())
    ///             .folder_type("Study".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Created container: {:?}", container.path);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_container(
        &self,
        options: CreateContainerOptions,
    ) -> Result<Container, LabkeyError> {
        let url = self.build_url(
            "core",
            "createContainer.api",
            options.container_path.as_deref(),
        );
        let body = CreateContainerBody {
            name: options.name,
            description: options.description,
            folder_type: options.folder_type,
            is_workbook: options.is_workbook,
            title: options.title,
        };
        self.post(url, &body).await
    }

    /// Delete a container.
    ///
    /// Sends a POST request to `core-deleteContainer.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the HTTP request fails, the server returns
    /// an error response, or the response body cannot be deserialized.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), labkey_rs::LabkeyError> {
    /// # let config = labkey_rs::ClientConfig::new(
    /// #     "https://labkey.example.com/labkey",
    /// #     labkey_rs::Credential::ApiKey("key".into()),
    /// #     "/MyProject",
    /// # );
    /// # let client = labkey_rs::LabkeyClient::new(config)?;
    /// use labkey_rs::security::DeleteContainerOptions;
    ///
    /// let _ = client
    ///     .delete_container(
    ///         DeleteContainerOptions::builder()
    ///             .container_path("/MyProject/OldFolder".to_string())
    ///             .comment("Cleanup".to_string())
    ///             .build(),
    ///     )
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_container(
        &self,
        options: DeleteContainerOptions,
    ) -> Result<serde_json::Value, LabkeyError> {
        let url = self.build_url(
            "core",
            "deleteContainer.api",
            options.container_path.as_deref(),
        );
        let body = DeleteContainerBody {
            comment: options.comment,
        };
        self.post(url, &body).await
    }

    /// Rename a container's name, title, or both.
    ///
    /// Sends a POST request to `admin-renameContainer.api`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError::InvalidInput`] when both `name` and `title` are
    /// omitted. Returns [`LabkeyError`] for request, response, or deserialization
    /// failures.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), labkey_rs::LabkeyError> {
    /// # let config = labkey_rs::ClientConfig::new(
    /// #     "https://labkey.example.com/labkey",
    /// #     labkey_rs::Credential::ApiKey("key".into()),
    /// #     "/MyProject",
    /// # );
    /// # let client = labkey_rs::LabkeyClient::new(config)?;
    /// use labkey_rs::security::RenameContainerOptions;
    ///
    /// let updated = client
    ///     .rename_container(
    ///         RenameContainerOptions::builder()
    ///             .container_path("/MyProject/FolderA".to_string())
    ///             .name("FolderB".to_string())
    ///             .add_alias(true)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Renamed container id: {:?}", updated.id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rename_container(
        &self,
        options: RenameContainerOptions,
    ) -> Result<Container, LabkeyError> {
        validate_rename_input(options.name.as_deref(), options.title.as_deref())?;

        let url = self.build_url(
            "admin",
            "renameContainer.api",
            options.container_path.as_deref(),
        );
        let body = RenameContainerBody {
            name: options.name,
            title: options.title,
            add_alias: options.add_alias,
        };
        self.post(url, &body).await
    }

    /// Retrieve container hierarchy information.
    ///
    /// Sends a GET request to `project-getContainers.api`.
    ///
    /// The server may return either a single container object or an envelope
    /// object with a `containers` array. This method always returns
    /// `Vec<ContainerHierarchy>`.
    ///
    /// # Errors
    ///
    /// Returns [`LabkeyError`] if the HTTP request fails, the server returns
    /// an error response, the response body cannot be deserialized, or the
    /// returned response shape is not recognized.
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
    /// use labkey_rs::security::GetContainersOptions;
    ///
    /// let containers = client
    ///     .get_containers(
    ///         GetContainersOptions::builder()
    ///             .containers(vec!["/Home".to_string(), "/Home/Project".to_string()])
    ///             .include_subfolders(true)
    ///             .build(),
    ///     )
    ///     .await?;
    ///
    /// println!("Containers returned: {}", containers.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_containers(
        &self,
        options: GetContainersOptions,
    ) -> Result<Vec<ContainerHierarchy>, LabkeyError> {
        let url = self.build_url(
            "project",
            "getContainers.api",
            options.container_path.as_deref(),
        );
        let params = build_get_containers_params(&options);
        let response: serde_json::Value = self.get(url, &params).await?;
        extract_containers(&response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ClientConfig, Credential};

    fn test_client(base_url: &str, container_path: &str) -> LabkeyClient {
        LabkeyClient::new(ClientConfig::new(
            base_url,
            Credential::ApiKey("test-key".to_string()),
            container_path,
        ))
        .expect("valid client config")
    }

    #[test]
    fn security_container_endpoint_urls_match_expected_actions() {
        let client = test_client("https://labkey.example.com/labkey", "/MyProject/MyFolder");

        assert_eq!(
            client
                .build_url("core", "createContainer.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/core-createContainer.api"
        );
        assert_eq!(
            client
                .build_url("core", "deleteContainer.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/core-deleteContainer.api"
        );
        assert_eq!(
            client
                .build_url("admin", "renameContainer.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/admin-renameContainer.api"
        );
        assert_eq!(
            client
                .build_url("project", "getContainers.api", Some("/Alt/Container"))
                .as_str(),
            "https://labkey.example.com/labkey/Alt/Container/project-getContainers.api"
        );
    }

    #[test]
    fn create_container_body_serializes_required_and_optional_fields() {
        let body = CreateContainerBody {
            name: "FolderA".to_string(),
            description: Some("desc".to_string()),
            folder_type: Some("Study".to_string()),
            is_workbook: Some(true),
            title: Some("Folder A".to_string()),
        };

        let value = serde_json::to_value(body).expect("should serialize");
        assert_eq!(value.get("name"), Some(&serde_json::json!("FolderA")));
        assert_eq!(value.get("description"), Some(&serde_json::json!("desc")));
        assert_eq!(value.get("folderType"), Some(&serde_json::json!("Study")));
        assert_eq!(value.get("isWorkbook"), Some(&serde_json::json!(true)));
        assert_eq!(value.get("title"), Some(&serde_json::json!("Folder A")));
    }

    #[test]
    fn rename_container_body_serializes_body_fields() {
        let body = RenameContainerBody {
            name: Some("Renamed".to_string()),
            title: Some("Renamed Title".to_string()),
            add_alias: Some(true),
        };

        let value = serde_json::to_value(body).expect("should serialize");
        assert_eq!(value.get("name"), Some(&serde_json::json!("Renamed")));
        assert_eq!(
            value.get("title"),
            Some(&serde_json::json!("Renamed Title"))
        );
        assert_eq!(value.get("addAlias"), Some(&serde_json::json!(true)));
    }

    #[test]
    fn rename_container_requires_name_or_title() {
        let error = validate_rename_input(None, None).expect_err("rename should fail");
        assert!(matches!(
            error,
            LabkeyError::InvalidInput(message)
                if message == "rename_container requires at least one of `name` or `title`"
        ));
    }

    #[test]
    fn get_containers_params_include_multiple_containers_flag_when_needed() {
        let options = GetContainersOptions::builder()
            .containers(vec!["/Home".to_string(), "/Home/Project".to_string()])
            .include_subfolders(true)
            .include_standard_properties(false)
            .module_properties(vec!["core".to_string(), "query".to_string()])
            .build();

        let params = build_get_containers_params(&options);

        assert!(params.contains(&("multipleContainers".to_string(), "true".to_string())));
        assert!(params.contains(&("container".to_string(), "/Home".to_string())));
        assert!(params.contains(&("container".to_string(), "/Home/Project".to_string())));
        assert!(params.contains(&("includeSubfolders".to_string(), "true".to_string())));
        assert!(params.contains(&("includeStandardProperties".to_string(), "false".to_string(),)));
        assert!(params.contains(&("moduleProperties".to_string(), "core".to_string())));
        assert!(params.contains(&("moduleProperties".to_string(), "query".to_string())));
    }

    #[test]
    fn get_containers_params_single_container_omits_multiple_containers_flag() {
        let options = GetContainersOptions::builder()
            .containers(vec!["/Home".to_string()])
            .build();

        let params = build_get_containers_params(&options);

        assert!(!params.iter().any(|(k, _)| k == "multipleContainers"));
        assert_eq!(
            params
                .iter()
                .filter(|(k, _)| k == "container")
                .collect::<Vec<_>>()
                .len(),
            1
        );
    }

    #[test]
    fn get_containers_extracts_single_object_response() {
        let response = serde_json::json!({
            "id": "c1",
            "path": "/Home",
            "type": "Folder",
            "children": []
        });

        let containers = extract_containers(&response).expect("single response should parse");
        assert_eq!(containers.len(), 1);
        assert_eq!(containers[0].id.as_deref(), Some("c1"));
    }

    #[test]
    fn get_containers_extracts_envelope_response() {
        let response = serde_json::json!({
            "containers": [
                {"id": "c1", "path": "/Home", "type": "Folder", "children": []},
                {"id": "c2", "path": "/Home/Project", "type": "Folder", "children": []}
            ]
        });

        let containers = extract_containers(&response).expect("envelope response should parse");
        assert_eq!(containers.len(), 2);
        assert_eq!(containers[1].id.as_deref(), Some("c2"));
    }

    #[test]
    fn get_containers_rejects_invalid_envelope_field() {
        let response = serde_json::json!({"containers": "invalid"});

        let error = extract_containers(&response).expect_err("invalid envelope should fail");
        assert!(matches!(error, LabkeyError::UnexpectedResponse { .. }));
    }

    #[test]
    fn get_containers_rejects_null_envelope_field() {
        let response = serde_json::json!({"containers": null});

        let error = extract_containers(&response).expect_err("null envelope should fail");
        assert!(matches!(error, LabkeyError::UnexpectedResponse { .. }));
    }
}
