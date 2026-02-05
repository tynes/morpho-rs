//! V1 client integration tests using wiremock.

mod helpers;

use alloy_primitives::U256;
use helpers::{client_config_with_mock, mock_graphql_response, start_mock_server};
use morpho_rs_api::{
    NamedChain, OrderDirection, VaultFiltersV1, VaultOrderByV1, VaultQueryOptionsV1, VaultV1Client,
};

#[tokio::test]
async fn test_get_vaults_empty() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "empty_vaults").await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let vaults = client.get_vaults(None).await.unwrap();
    assert!(vaults.is_empty());
}

#[tokio::test]
async fn test_get_vaults_multiple() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let vaults = client.get_vaults(None).await.unwrap();

    assert_eq!(vaults.len(), 2);

    // Check first vault
    let vault1 = &vaults[0];
    assert_eq!(vault1.name, "Steakhouse USDC");
    assert_eq!(vault1.symbol, "steakUSDC");
    assert_eq!(vault1.chain, NamedChain::Mainnet);
    assert!(vault1.listed);
    assert!(!vault1.featured);
    assert!(vault1.whitelisted);
    assert_eq!(vault1.asset.symbol, "USDC");
    assert_eq!(vault1.asset.decimals, 6);

    // Check vault state
    let state1 = vault1.state.as_ref().unwrap();
    assert_eq!(state1.total_assets, U256::from(1_000_000_000_000u64));
    assert_eq!(state1.total_assets_usd, Some(1_000_000.0));
    assert_eq!(state1.fee, 0.15);
    assert_eq!(state1.apy, 0.08);
    assert_eq!(state1.net_apy, 0.068);

    // Check second vault
    let vault2 = &vaults[1];
    assert_eq!(vault2.name, "Gauntlet WETH Prime");
    assert_eq!(vault2.symbol, "gtWETH");
    assert!(vault2.featured);
    assert_eq!(vault2.asset.symbol, "WETH");
}

#[tokio::test]
async fn test_get_vault_single() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_info").await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let vault = client
        .get_vault(
            "0x8eB67A509616cd6A7c1B3c8C21D48FF57df3d458",
            NamedChain::Mainnet,
        )
        .await
        .unwrap();

    assert_eq!(vault.name, "Steakhouse USDC");
    assert_eq!(vault.symbol, "steakUSDC");
    assert_eq!(vault.chain, NamedChain::Mainnet);
    assert_eq!(
        vault.address.to_string().to_lowercase(),
        "0x8eb67a509616cd6a7c1b3c8c21d48ff57df3d458"
    );
}

#[tokio::test]
async fn test_get_vault_with_allocations() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_info_with_markets").await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let vault = client
        .get_vault(
            "0x8eB67A509616cd6A7c1B3c8C21D48FF57df3d458",
            NamedChain::Mainnet,
        )
        .await
        .unwrap();

    let state = vault.state.as_ref().unwrap();
    assert_eq!(state.allocation.len(), 2);

    // Check first allocation
    let alloc1 = &state.allocation[0];
    assert_eq!(alloc1.loan_asset_symbol, Some("USDC".to_string()));
    assert_eq!(alloc1.collateral_asset_symbol, Some("WETH".to_string()));
    assert_eq!(alloc1.supply_assets, U256::from(500_000_000_000u64));
    assert!(alloc1.enabled);
    assert_eq!(alloc1.supply_queue_index, Some(0));
    assert_eq!(alloc1.withdraw_queue_index, Some(0));

    // Check market state
    let market_state = alloc1.market_state.as_ref().unwrap();
    assert_eq!(
        market_state.total_supply_assets,
        U256::from(1_000_000_000_000u64)
    );
    assert_eq!(
        market_state.total_borrow_assets,
        U256::from(500_000_000_000u64)
    );
    assert_eq!(
        market_state.liquidity,
        U256::from(500_000_000_000u64) // total_supply - total_borrow
    );

    // Check second allocation
    let alloc2 = &state.allocation[1];
    assert_eq!(alloc2.collateral_asset_symbol, Some("wstETH".to_string()));
    assert_eq!(alloc2.supply_queue_index, Some(1));
}

#[tokio::test]
async fn test_get_vaults_by_chain() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let vaults = client.get_vaults_by_chain(NamedChain::Mainnet).await.unwrap();

    assert_eq!(vaults.len(), 2);
    // All vaults should be on mainnet
    for vault in &vaults {
        assert_eq!(vault.chain, NamedChain::Mainnet);
    }
}

#[tokio::test]
async fn test_get_vaults_with_filters() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let filters = VaultFiltersV1::new()
        .chain(NamedChain::Mainnet)
        .listed(true)
        .min_apy(0.01);

    let vaults = client.get_vaults(Some(filters)).await.unwrap();
    assert_eq!(vaults.len(), 2);
}

#[tokio::test]
async fn test_get_vaults_with_options() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let options = VaultQueryOptionsV1::new()
        .filters(VaultFiltersV1::new().chain(NamedChain::Mainnet))
        .order_by(VaultOrderByV1::NetApy)
        .order_direction(OrderDirection::Desc)
        .limit(10);

    let vaults = client.get_vaults_with_options(options).await.unwrap();
    assert_eq!(vaults.len(), 2);
}

#[tokio::test]
async fn test_get_top_vaults_by_apy() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let vaults = client.get_top_vaults_by_apy(5, None).await.unwrap();
    assert_eq!(vaults.len(), 2);
}

#[tokio::test]
async fn test_get_vaults_by_asset() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let vaults = client.get_vaults_by_asset("USDC", None).await.unwrap();
    // The mock returns 2 vaults, but server-side filtering would typically filter
    assert!(!vaults.is_empty());
}

#[tokio::test]
async fn test_get_whitelisted_vaults() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let vaults = client.get_whitelisted_vaults(None).await.unwrap();

    assert_eq!(vaults.len(), 2);
    // All returned vaults should be whitelisted
    for vault in &vaults {
        assert!(vault.whitelisted);
    }
}

#[tokio::test]
async fn test_client_default() {
    // Test that default client can be created
    let client = VaultV1Client::default();
    assert_eq!(client.config().page_size, 100);
}

#[tokio::test]
async fn test_v1_options_top_by_apy() {
    let options = VaultQueryOptionsV1::top_by_apy(10);
    assert_eq!(options.limit, Some(10));
    assert_eq!(options.order_by, Some(VaultOrderByV1::NetApy));
    assert_eq!(options.order_direction, Some(OrderDirection::Desc));
}

#[tokio::test]
async fn test_v1_options_top_by_tvl() {
    let options = VaultQueryOptionsV1::top_by_tvl(5);
    assert_eq!(options.limit, Some(5));
    assert_eq!(options.order_by, Some(VaultOrderByV1::TotalAssetsUsd));
    assert_eq!(options.order_direction, Some(OrderDirection::Desc));
}

#[tokio::test]
async fn test_vault_trait_implementation() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_info").await;

    let config = client_config_with_mock(&server);
    let client = VaultV1Client::with_config(config);

    let vault = client
        .get_vault(
            "0x8eB67A509616cd6A7c1B3c8C21D48FF57df3d458",
            NamedChain::Mainnet,
        )
        .await
        .unwrap();

    // Test Vault trait methods
    use morpho_rs_api::Vault;
    assert_eq!(vault.name(), "Steakhouse USDC");
    assert_eq!(vault.symbol(), "steakUSDC");
    assert_eq!(vault.chain(), NamedChain::Mainnet);
    assert!(vault.listed());
    assert!(vault.whitelisted());
    assert_eq!(vault.asset().symbol, "USDC");
    assert_eq!(vault.total_assets(), U256::from(1_000_000_000_000u64));
    assert_eq!(vault.total_assets_usd(), Some(1_000_000.0));
    assert_eq!(vault.net_apy(), 0.068);
    assert!(!vault.has_critical_warnings());
}
