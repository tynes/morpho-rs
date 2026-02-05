//! Test helper utilities for API crate integration tests.

use morpho_rs_api::ClientConfig;
use url::Url;
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Start a mock GraphQL server.
pub async fn start_mock_server() -> MockServer {
    MockServer::start().await
}

/// Create a ClientConfig pointing to a mock server.
pub fn client_config_with_mock(mock: &MockServer) -> ClientConfig {
    ClientConfig::new().with_api_url(Url::parse(&mock.uri()).unwrap())
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

/// Mock a GraphQL error response with a single error message.
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

/// Mock a GraphQL response with multiple errors.
pub async fn mock_graphql_errors(server: &MockServer, error_messages: &[&str]) {
    let errors: Vec<String> = error_messages
        .iter()
        .map(|msg| format!(r#"{{"message":"{}"}}"#, msg))
        .collect();
    let body = format!(r#"{{"errors":[{}],"data":null}}"#, errors.join(","));
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(server)
        .await;
}

/// Mock an HTTP error response.
pub async fn mock_http_error(server: &MockServer, status_code: u16) {
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(status_code).set_body_string("Internal Server Error"))
        .mount(server)
        .await;
}

/// Mock a response with null data (no errors but no data).
pub async fn mock_null_data(server: &MockServer) {
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"data":null}"#))
        .mount(server)
        .await;
}
