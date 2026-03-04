use std::{fs, path::PathBuf};

use serde::de::DeserializeOwned;

pub use labkey_rs::{ClientConfig, Credential, LabkeyClient, LabkeyError};
pub use wiremock::{Mock, MockServer, ResponseTemplate};

/// Build a test client targeting a mock server URL.
pub fn test_client(mock_server_url: &str) -> LabkeyClient {
    let config = ClientConfig::new(
        mock_server_url,
        Credential::ApiKey("test-api-key".to_string()),
        "/MyProject/MyFolder",
    );
    LabkeyClient::new(config).expect("test client should construct")
}

/// Load and deserialize a JSON fixture from `tests/fixtures/`.
pub fn fixture<T: DeserializeOwned>(name: &str) -> T {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("fixtures");
    path.push(name);

    let text = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture at {}: {err}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("failed to parse fixture at {}: {err}", path.display()))
}
