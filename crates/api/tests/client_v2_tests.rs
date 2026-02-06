//! V2 client integration tests using wiremock.

mod helpers;

use alloy_primitives::U256;
use helpers::{client_config_with_mock, mock_graphql_response, start_mock_server};
use morpho_rs_api::{
    NamedChain, OrderDirection, VaultFiltersV2, VaultOrderByV2, VaultQueryOptionsV2, VaultV2Client,
};

#[tokio::test]
async fn test_get_vaults_v2_empty() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "empty_v2_vaults").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let vaults = client.get_vaults(None).await.unwrap();
    assert!(vaults.is_empty());
}

#[tokio::test]
async fn test_get_vaults_v2_multiple() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let vaults = client.get_vaults(None).await.unwrap();

    assert_eq!(vaults.len(), 2);

    // Check first vault
    let vault1 = &vaults[0];
    assert_eq!(vault1.name, "Test V2 USDC Vault");
    assert_eq!(vault1.symbol, "tv2USDC");
    assert_eq!(vault1.chain, NamedChain::Mainnet);
    assert!(vault1.listed);
    assert!(vault1.whitelisted);
    assert_eq!(vault1.asset.symbol, "USDC");
    assert_eq!(vault1.total_assets, U256::from(2_000_000_000_000u64));
    assert_eq!(vault1.total_assets_usd, Some(2_000_000.0));
    assert_eq!(vault1.performance_fee, Some(0.1));
    assert_eq!(vault1.management_fee, Some(0.02));
    assert_eq!(vault1.avg_apy, Some(0.06));
    assert_eq!(vault1.avg_net_apy, Some(0.052));
    assert_eq!(vault1.liquidity, U256::from(500_000_000_000u64));

    // Check second vault
    let vault2 = &vaults[1];
    assert_eq!(vault2.name, "Test V2 WETH Vault");
    assert_eq!(vault2.asset.symbol, "WETH");
}

#[tokio::test]
async fn test_get_vault_v2_single() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_info").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let vault = client
        .get_vault(
            "0xABCdef1234567890ABCdef1234567890ABCdef12",
            NamedChain::Mainnet,
        )
        .await
        .unwrap();

    assert_eq!(vault.name, "Test V2 USDC Vault");
    assert_eq!(vault.symbol, "tv2USDC");
    assert_eq!(vault.chain, NamedChain::Mainnet);
    assert_eq!(
        vault.address.to_string().to_lowercase(),
        "0xabcdef1234567890abcdef1234567890abcdef12"
    );
}

#[tokio::test]
async fn test_get_vaults_v2_by_chain() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let vaults = client.get_vaults_by_chain(NamedChain::Mainnet).await.unwrap();

    assert_eq!(vaults.len(), 2);
    for vault in &vaults {
        assert_eq!(vault.chain, NamedChain::Mainnet);
    }
}

#[tokio::test]
async fn test_get_vaults_v2_with_filters() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let filters = VaultFiltersV2::new()
        .chain(NamedChain::Mainnet)
        .listed(true)
        .min_total_assets_usd(100_000.0);

    let vaults = client.get_vaults(Some(filters)).await.unwrap();
    assert_eq!(vaults.len(), 2);
}

#[tokio::test]
async fn test_get_vaults_v2_with_options() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let options = VaultQueryOptionsV2::new()
        .filters(VaultFiltersV2::new().chain(NamedChain::Mainnet))
        .order_by(VaultOrderByV2::NetApy)
        .order_direction(OrderDirection::Desc)
        .limit(10);

    let vaults = client.get_vaults_with_options(options).await.unwrap();
    assert_eq!(vaults.len(), 2);
}

#[tokio::test]
async fn test_get_vaults_v2_client_side_asset_filter() {
    // V2 API doesn't support server-side asset filtering, so it's done client-side
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    // Filter to only USDC vaults (client-side)
    let options = VaultQueryOptionsV2::new().asset_symbols(["USDC"]);

    let vaults = client.get_vaults_with_options(options).await.unwrap();

    // Should filter to only USDC vault
    assert_eq!(vaults.len(), 1);
    assert_eq!(vaults[0].asset.symbol, "USDC");
}

#[tokio::test]
async fn test_get_vaults_v2_client_side_asset_filter_case_insensitive() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    // Filter with different case
    let options = VaultQueryOptionsV2::new().asset_symbols(["usdc", "weth"]);

    let vaults = client.get_vaults_with_options(options).await.unwrap();

    // Should match both vaults (case-insensitive)
    assert_eq!(vaults.len(), 2);
}

#[tokio::test]
async fn test_get_vaults_v2_client_side_address_filter() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    // Filter by asset address (client-side)
    let options =
        VaultQueryOptionsV2::new().asset_addresses(["0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"]);

    let vaults = client.get_vaults_with_options(options).await.unwrap();

    assert_eq!(vaults.len(), 1);
    assert_eq!(vaults[0].asset.symbol, "USDC");
}

#[tokio::test]
async fn test_get_vaults_v2_client_side_curator_filter() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    // Filter to only vaults with a specific curator (from fixture)
    let options = VaultQueryOptionsV2::new()
        .curator_addresses(["0xCA11ab1eCA11ab1eCA11ab1eCA11ab1eCA11ab1e"]);

    let vaults = client.get_vaults_with_options(options).await.unwrap();

    // Should filter to only the vault with matching curator
    assert_eq!(vaults.len(), 1);
    assert_eq!(vaults[0].name, "Test V2 USDC Vault");
}

#[tokio::test]
async fn test_get_vaults_v2_client_side_curator_filter_case_insensitive() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    // Filter with different case
    let options = VaultQueryOptionsV2::new()
        .curator_addresses(["0xca11ab1eca11ab1eca11ab1eca11ab1eca11ab1e"]);

    let vaults = client.get_vaults_with_options(options).await.unwrap();

    assert_eq!(vaults.len(), 1);
    assert_eq!(vaults[0].name, "Test V2 USDC Vault");
}

#[tokio::test]
async fn test_get_vaults_v2_by_curator() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let vaults = client
        .get_vaults_by_curator("0xCA11ab1eCA11ab1eCA11ab1eCA11ab1eCA11ab1e", None)
        .await
        .unwrap();

    assert_eq!(vaults.len(), 1);
    assert_eq!(vaults[0].name, "Test V2 USDC Vault");
}

#[tokio::test]
async fn test_get_vaults_v2_by_curator_no_match() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let vaults = client
        .get_vaults_by_curator("0x0000000000000000000000000000000000000000", None)
        .await
        .unwrap();

    assert!(vaults.is_empty());
}

#[tokio::test]
async fn test_get_top_vaults_v2_by_apy() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let vaults = client.get_top_vaults_by_apy(5, None).await.unwrap();
    assert_eq!(vaults.len(), 2);
}

#[tokio::test]
async fn test_get_vaults_v2_by_asset() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    // This uses client-side filtering
    let vaults = client.get_vaults_by_asset("USDC", None).await.unwrap();

    assert_eq!(vaults.len(), 1);
    assert_eq!(vaults[0].asset.symbol, "USDC");
}

#[tokio::test]
async fn test_get_whitelisted_vaults_v2() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let vaults = client.get_whitelisted_vaults(None).await.unwrap();

    assert_eq!(vaults.len(), 2);
    for vault in &vaults {
        assert!(vault.whitelisted);
    }
}

#[tokio::test]
async fn test_v2_client_default() {
    let client = VaultV2Client::default();
    assert_eq!(client.config().page_size, 100);
}

#[tokio::test]
async fn test_v2_options_top_by_apy() {
    let options = VaultQueryOptionsV2::top_by_apy(10);
    assert_eq!(options.limit, Some(10));
    assert_eq!(options.order_by, Some(VaultOrderByV2::NetApy));
    assert_eq!(options.order_direction, Some(OrderDirection::Desc));
}

#[tokio::test]
async fn test_v2_options_top_by_tvl() {
    let options = VaultQueryOptionsV2::top_by_tvl(5);
    assert_eq!(options.limit, Some(5));
    assert_eq!(options.order_by, Some(VaultOrderByV2::TotalAssetsUsd));
    assert_eq!(options.order_direction, Some(OrderDirection::Desc));
}

#[tokio::test]
async fn test_v2_options_top_by_liquidity() {
    let options = VaultQueryOptionsV2::top_by_liquidity(15);
    assert_eq!(options.limit, Some(15));
    assert_eq!(options.order_by, Some(VaultOrderByV2::LiquidityUsd));
    assert_eq!(options.order_direction, Some(OrderDirection::Desc));
}

#[tokio::test]
async fn test_v2_options_has_client_filter() {
    let options_no_filter = VaultQueryOptionsV2::new();
    assert!(!options_no_filter.has_client_filter());

    let options_with_symbol = VaultQueryOptionsV2::new().asset_symbols(["USDC"]);
    assert!(options_with_symbol.has_client_filter());

    let options_with_address =
        VaultQueryOptionsV2::new().asset_addresses(["0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48"]);
    assert!(options_with_address.has_client_filter());

    let options_with_curator =
        VaultQueryOptionsV2::new().curator_addresses(["0xCA11ab1eCA11ab1eCA11ab1eCA11ab1eCA11ab1e"]);
    assert!(options_with_curator.has_client_filter());
}

#[tokio::test]
async fn test_vault_v2_trait_implementation() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_info").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let vault = client
        .get_vault(
            "0xABCdef1234567890ABCdef1234567890ABCdef12",
            NamedChain::Mainnet,
        )
        .await
        .unwrap();

    // Test Vault trait methods
    use morpho_rs_api::Vault;
    assert_eq!(vault.name(), "Test V2 USDC Vault");
    assert_eq!(vault.symbol(), "tv2USDC");
    assert_eq!(vault.chain(), NamedChain::Mainnet);
    assert!(vault.listed());
    assert!(vault.whitelisted());
    assert_eq!(vault.asset().symbol, "USDC");
    assert_eq!(vault.total_assets(), U256::from(2_000_000_000_000u64));
    assert_eq!(vault.total_assets_usd(), Some(2_000_000.0));
    assert_eq!(vault.net_apy(), 0.052); // Uses avg_net_apy for V2
    assert_eq!(vault.liquidity(), U256::from(500_000_000_000u64));
    assert!(!vault.has_critical_warnings());
}

#[tokio::test]
async fn test_vault_v2_rewards() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_info_metamorpho").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let vault = client
        .get_vault(
            "0xABCdef1234567890ABCdef1234567890ABCdef12",
            NamedChain::Mainnet,
        )
        .await
        .unwrap();

    assert_eq!(vault.rewards.len(), 1);
    let reward = &vault.rewards[0];
    assert_eq!(reward.asset_symbol, "MORPHO");
    assert_eq!(reward.supply_apr, Some(0.02));
}

#[tokio::test]
async fn test_vault_v2_warnings() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_info_morpho_market").await;

    let config = client_config_with_mock(&server);
    let client = VaultV2Client::with_config(config);

    let vault = client
        .get_vault(
            "0xdef1234567890ABCdef1234567890ABCdef12345",
            NamedChain::Mainnet,
        )
        .await
        .unwrap();

    assert_eq!(vault.warnings.len(), 1);
    assert_eq!(vault.warnings[0].warning_type, "LOW_LIQUIDITY");
    // Level is formatted as Debug from the enum
    assert!(vault.warnings[0].level.contains("Yellow"));
}
