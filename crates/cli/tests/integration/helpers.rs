//! Test helper utilities for CLI integration tests.

#![allow(deprecated)] // Command::cargo_bin deprecation

use assert_cmd::Command;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Start a mock GraphQL server.
pub async fn start_mock_server() -> MockServer {
    MockServer::start().await
}

/// Create a CLI command pointing to a mock server.
pub fn morpho_cmd_with_mock(mock: &MockServer) -> Command {
    let mut cmd = Command::cargo_bin("morpho").unwrap();
    cmd.env("MORPHO_API_URL", mock.uri());
    cmd
}

/// Create a CLI command without mock server (for validation tests).
pub fn morpho_cmd() -> Command {
    Command::cargo_bin("morpho").unwrap()
}

/// Load a fixture file as a string.
pub fn load_fixture(name: &str) -> String {
    let path = format!(
        "{}/tests/fixtures/{}.json",
        env!("CARGO_MANIFEST_DIR"),
        name
    );
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("Failed to load fixture: {}", path))
}

/// Mock a GraphQL POST request with a fixture response.
pub async fn mock_graphql_response(server: &MockServer, fixture_name: &str) {
    let body = load_fixture(fixture_name);
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(server)
        .await;
}

/// Mock a GraphQL error response.
pub async fn mock_graphql_error(server: &MockServer, error_message: &str) {
    let body = format!(
        r#"{{"errors":[{{"message":"{}"}}],"data":null}}"#,
        error_message
    );
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(server)
        .await;
}
