//! Retry/backoff integration tests.

mod helpers;

use helpers::{client_config_with_mock, load_fixture, start_mock_server};
use morpho_rs_api::{ClientConfig, VaultV1Client};
use wiremock::matchers::method;
use wiremock::{Mock, MockServer, ResponseTemplate};

/// Create a client config with fast retries for testing.
fn fast_retry_config(server: &MockServer, max_retries: u32) -> ClientConfig {
    client_config_with_mock(server)
        .with_max_retries(max_retries)
        .with_retry_base_delay_ms(10) // Fast retries for tests
}

#[tokio::test]
async fn test_retry_success_after_transient_failures() {
    let server = start_mock_server().await;

    // First 2 requests return 500, third returns success
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
        .up_to_n_times(2)
        .mount(&server)
        .await;

    let body = load_fixture("v1_list");
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let config = fast_retry_config(&server, 3);
    let client = VaultV1Client::with_config(config);

    let vaults = client.get_vaults(None).await.unwrap();
    assert_eq!(vaults.len(), 2);
}

#[tokio::test]
async fn test_retry_exhausted_returns_error() {
    let server = start_mock_server().await;

    // All requests return 500
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
        .mount(&server)
        .await;

    let config = fast_retry_config(&server, 2);
    let client = VaultV1Client::with_config(config);

    let result = client.get_vaults(None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_no_retry_with_zero_max_retries() {
    let server = start_mock_server().await;

    // First request returns 500, second would return success
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Server Error"))
        .expect(1)
        .mount(&server)
        .await;

    let config = fast_retry_config(&server, 0);
    let client = VaultV1Client::with_config(config);

    let result = client.get_vaults(None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_graphql_errors_not_retried() {
    let server = start_mock_server().await;

    // GraphQL errors should not be retried
    Mock::given(method("POST"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(r#"{"errors":[{"message":"Bad query"}],"data":null}"#),
        )
        .expect(1) // Should only be called once
        .mount(&server)
        .await;

    let config = fast_retry_config(&server, 3);
    let client = VaultV1Client::with_config(config);

    let result = client.get_vaults(None).await;
    assert!(result.is_err());
    match result.unwrap_err() {
        morpho_rs_api::ApiError::GraphQL(msg) => assert_eq!(msg, "Bad query"),
        e => panic!("Expected GraphQL error, got: {e:?}"),
    }
}

#[tokio::test]
async fn test_success_on_first_attempt_no_retry() {
    let server = start_mock_server().await;

    let body = load_fixture("v1_list");
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .expect(1) // Should only be called once
        .mount(&server)
        .await;

    let config = fast_retry_config(&server, 3);
    let client = VaultV1Client::with_config(config);

    let vaults = client.get_vaults(None).await.unwrap();
    assert_eq!(vaults.len(), 2);
}

#[tokio::test]
async fn test_retry_config_builder() {
    let config = ClientConfig::new()
        .with_max_retries(5)
        .with_retry_base_delay_ms(500)
        .with_request_timeout_secs(60);

    assert_eq!(config.max_retries, 5);
    assert_eq!(config.retry_base_delay_ms, 500);
    assert_eq!(config.request_timeout_secs, 60);
}
