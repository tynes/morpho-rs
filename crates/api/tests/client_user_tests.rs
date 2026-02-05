//! User position tests using wiremock.

mod helpers;

use alloy_primitives::U256;
use helpers::{client_config_with_mock, mock_graphql_response, start_mock_server};
use morpho_rs_api::{MorphoApiClient, MorphoClient, NamedChain};

#[tokio::test]
async fn test_get_user_vault_positions_single_chain() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "user_positions").await;

    let config = client_config_with_mock(&server);
    let client = MorphoApiClient::with_config(config);

    let positions = client
        .get_user_vault_positions(
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            Some(NamedChain::Mainnet),
        )
        .await
        .unwrap();

    // Check address
    assert_eq!(
        positions.address.to_string().to_lowercase(),
        "0xd8da6bf26964af9d7eed9e03e53415d37aa96045"
    );

    // Check V1 positions
    assert_eq!(positions.vault_positions.len(), 2);

    let v1_pos1 = &positions.vault_positions[0];
    assert_eq!(v1_pos1.id, "pos-v1-1");
    assert_eq!(v1_pos1.shares, U256::from(1_000_000_000_000u64));
    assert_eq!(v1_pos1.assets, U256::from(1_000_000_000_000u64));
    assert_eq!(v1_pos1.assets_usd, Some(1_000_000.0));
    assert_eq!(v1_pos1.vault.name, "Steakhouse USDC");

    // Check position state
    let state = v1_pos1.state.as_ref().unwrap();
    assert_eq!(state.pnl, Some(U256::from(50_000_000_000u64)));
    assert_eq!(state.pnl_usd, Some(50_000.0));
    assert_eq!(state.roe, Some(0.05));

    let v1_pos2 = &positions.vault_positions[1];
    assert_eq!(v1_pos2.vault.name, "Gauntlet WETH Prime");

    // Check V2 positions
    assert_eq!(positions.vault_v2_positions.len(), 1);

    let v2_pos = &positions.vault_v2_positions[0];
    assert_eq!(v2_pos.id, "pos-v2-1");
    assert_eq!(v2_pos.shares, U256::from(500_000_000_000u64));
    assert_eq!(v2_pos.assets_usd, Some(500_000.0));
    assert_eq!(v2_pos.pnl_usd, Some(25_000.0));
    assert_eq!(v2_pos.vault.name, "Test V2 USDC Vault");
}

#[tokio::test]
async fn test_get_user_positions_v1_and_v2() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "user_positions").await;

    let config = client_config_with_mock(&server);
    let client = MorphoApiClient::with_config(config);

    let positions = client
        .get_user_vault_positions(
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            Some(NamedChain::Mainnet),
        )
        .await
        .unwrap();

    // Should have both V1 and V2 positions
    assert!(!positions.vault_positions.is_empty());
    assert!(!positions.vault_v2_positions.is_empty());

    // Calculate total assets across all positions
    let total_v1_usd: f64 = positions
        .vault_positions
        .iter()
        .filter_map(|p| p.assets_usd)
        .sum();

    let total_v2_usd: f64 = positions
        .vault_v2_positions
        .iter()
        .filter_map(|p| p.assets_usd)
        .sum();

    assert_eq!(total_v1_usd, 1_750_000.0); // 1M + 750K
    assert_eq!(total_v2_usd, 500_000.0);
}

#[tokio::test]
async fn test_morpho_client_user_positions() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "user_positions").await;

    let config = client_config_with_mock(&server);
    let api_client = MorphoApiClient::with_config(config.clone());

    // Test via MorphoClient wrapper
    use morpho_rs_api::MorphoClientConfig;

    let morpho_config = MorphoClientConfig::new().with_api_config(config);
    let client = MorphoClient::with_config(morpho_config).unwrap();

    let positions = client
        .get_user_vault_positions(
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            Some(NamedChain::Mainnet),
        )
        .await
        .unwrap();

    assert_eq!(positions.vault_positions.len(), 2);
    assert_eq!(positions.vault_v2_positions.len(), 1);

    // Compare with direct API client
    mock_graphql_response(&server, "user_positions").await;
    let direct_positions = api_client
        .get_user_vault_positions(
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            Some(NamedChain::Mainnet),
        )
        .await
        .unwrap();

    assert_eq!(
        positions.vault_positions.len(),
        direct_positions.vault_positions.len()
    );
}

#[tokio::test]
async fn test_morpho_client_default() {
    let client = MorphoClient::new();
    // Should be able to use API methods without transaction support
    assert!(!client.has_transaction_support());
    assert!(client.signer_address().is_none());
}

#[tokio::test]
async fn test_morpho_api_client_default() {
    let client = MorphoApiClient::new();
    // Just ensure it can be created
    let _ = &client.v1;
    let _ = &client.v2;
}

#[tokio::test]
async fn test_user_position_vault_info() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "user_positions").await;

    let config = client_config_with_mock(&server);
    let client = MorphoApiClient::with_config(config);

    let positions = client
        .get_user_vault_positions(
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            Some(NamedChain::Mainnet),
        )
        .await
        .unwrap();

    // Check vault info on V1 position
    let v1_vault = &positions.vault_positions[0].vault;
    assert_eq!(
        v1_vault.address.to_string().to_lowercase(),
        "0x8eb67a509616cd6a7c1b3c8c21d48ff57df3d458"
    );
    assert_eq!(v1_vault.name, "Steakhouse USDC");
    assert_eq!(v1_vault.symbol, "steakUSDC");
    assert_eq!(v1_vault.chain, NamedChain::Mainnet);

    // Check vault info on V2 position
    let v2_vault = &positions.vault_v2_positions[0].vault;
    assert_eq!(
        v2_vault.address.to_string().to_lowercase(),
        "0xabcdef1234567890abcdef1234567890abcdef12"
    );
    assert_eq!(v2_vault.name, "Test V2 USDC Vault");
    assert_eq!(v2_vault.symbol, "tv2USDC");
}
