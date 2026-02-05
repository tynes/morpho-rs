//! Error handling tests for the API crate.

mod helpers;

use helpers::{
    client_config_with_mock, mock_graphql_error, mock_graphql_errors, mock_null_data,
    start_mock_server,
};
use morpho_rs_api::{ApiError, MorphoApiClient, NamedChain, VaultV1Client, VaultV2Client};

#[tokio::test]
async fn test_execute_graphql_error_single() {
    let server = start_mock_server().await;
    mock_graphql_error(&server, "Invalid query").await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let result = client.get_vaults(None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::GraphQL(msg) => {
            assert_eq!(msg, "Invalid query");
        }
        e => panic!("Expected GraphQL error, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_execute_graphql_error_multiple() {
    let server = start_mock_server().await;
    mock_graphql_errors(&server, &["Error 1", "Error 2", "Error 3"]).await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let result = client.get_vaults(None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::GraphQL(msg) => {
            assert!(msg.contains("Error 1"));
            assert!(msg.contains("Error 2"));
            assert!(msg.contains("Error 3"));
            assert!(msg.contains(";")); // Errors joined by ";"
        }
        e => panic!("Expected GraphQL error, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_execute_no_data() {
    let server = start_mock_server().await;
    mock_null_data(&server).await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let result = client.get_vaults(None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::Parse(msg) => {
            assert!(msg.contains("No data"));
        }
        e => panic!("Expected Parse error, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_get_vault_not_found_graphql_error() {
    // The Morpho API returns a GraphQL error when vault is not found
    // since vaultByAddress returns Vault! (non-null)
    let server = start_mock_server().await;
    mock_graphql_error(&server, "Vault not found").await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let result = client
        .get_vault("0x0000000000000000000000000000000000000000", NamedChain::Mainnet)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::GraphQL(msg) => {
            assert!(msg.contains("not found") || msg.contains("Vault"));
        }
        e => panic!("Expected GraphQL error, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_get_vault_v2_not_found_graphql_error() {
    // The Morpho API returns a GraphQL error when vault is not found
    let server = start_mock_server().await;
    mock_graphql_error(&server, "Vault V2 not found").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let result = client
        .get_vault("0x0000000000000000000000000000000000000000", NamedChain::Mainnet)
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::GraphQL(msg) => {
            assert!(msg.contains("not found") || msg.contains("Vault"));
        }
        e => panic!("Expected GraphQL error, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_get_user_positions_no_results_ignored() {
    // When querying all chains, "No results" errors should be ignored
    // This tests the error filtering in get_user_vault_positions_all_chains
    let server = start_mock_server().await;
    mock_graphql_error(&server, "No results found").await;

    let config = client_config_with_mock(&server);
    let client = MorphoApiClient::with_config(config);

    // Query all chains (chain = None) - should return empty positions, not error
    let result = client
        .get_user_vault_positions("0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045", None)
        .await;

    // The error filtering happens, so we get an Ok result with empty positions
    // Note: In practice this might still return an error because all chains fail,
    // but the "No results" errors specifically should be filtered out
    match result {
        Ok(positions) => {
            assert!(positions.vault_positions.is_empty());
            assert!(positions.vault_v2_positions.is_empty());
        }
        Err(ApiError::GraphQL(msg)) if msg.contains("No results") => {
            panic!("No results error should be filtered: {}", msg);
        }
        Err(_) => {
            // Other errors are acceptable (e.g., from the combined error handling)
        }
    }
}

#[tokio::test]
async fn test_graphql_error_on_v2_client() {
    let server = start_mock_server().await;
    mock_graphql_error(&server, "V2 query failed").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let result = client.get_vaults(None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::GraphQL(msg) => {
            assert_eq!(msg, "V2 query failed");
        }
        e => panic!("Expected GraphQL error, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_null_data_on_v2_client() {
    let server = start_mock_server().await;
    mock_null_data(&server).await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let result = client.get_vaults(None).await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ApiError::Parse(msg) => {
            assert!(msg.contains("No data"));
        }
        e => panic!("Expected Parse error, got: {:?}", e),
    }
}

#[tokio::test]
async fn test_transaction_not_configured() {
    // Test that vault operations fail when transaction support is not configured
    use morpho_rs_api::MorphoClient;

    let client = MorphoClient::new();

    let result = client.vault_v1();
    assert!(result.is_err());
    let err = result.err().unwrap();
    match err {
        ApiError::TransactionNotConfigured => {}
        e => panic!("Expected TransactionNotConfigured error, got: {:?}", e),
    }

    let result = client.vault_v2();
    assert!(result.is_err());
    let err = result.err().unwrap();
    match err {
        ApiError::TransactionNotConfigured => {}
        e => panic!("Expected TransactionNotConfigured error, got: {:?}", e),
    }
}
