//! MorphoClient fork tests for verifying transaction operations.
//!
//! These tests verify that the MorphoClient vault operations work correctly
//! by forking mainnet and interacting with real vaults.
//!
//! Run with: `cargo test --test morpho_client_fork_tests -- --ignored`
//! Requires `ETH_RPC_URL` environment variable to be set.

mod fork_helpers;

use alloy::{primitives::U256, providers::ProviderBuilder};
use fork_helpers::{
    fund_account_with_usdc, spawn_forked_anvil, EXPECTED_SIGNER_ADDRESS, STEAKHOUSE_USDC_VAULT,
    TEST_PRIVATE_KEY, USDC_ADDRESS,
};
use morpho_rs_api::{ApiError, MorphoClient, MorphoClientConfig};

// ============================================================================
// Configuration Tests (No RPC needed)
// ============================================================================

#[test]
fn test_signer_address_returns_correct_address() {
    let config = MorphoClientConfig::new()
        .with_rpc_url("http://localhost:8545")
        .with_private_key(TEST_PRIVATE_KEY);
    let client = MorphoClient::with_config(config).expect("Failed to create client");

    assert_eq!(
        client.signer_address(),
        Some(EXPECTED_SIGNER_ADDRESS),
        "Signer address should match expected address"
    );
}

#[test]
fn test_has_transaction_support_true_when_configured() {
    let config = MorphoClientConfig::new()
        .with_rpc_url("http://localhost:8545")
        .with_private_key(TEST_PRIVATE_KEY);
    let client = MorphoClient::with_config(config).expect("Failed to create client");

    assert!(
        client.has_transaction_support(),
        "Should have transaction support when RPC and key are configured"
    );
}

#[test]
fn test_has_transaction_support_false_when_api_only() {
    let client = MorphoClient::new();

    assert!(
        !client.has_transaction_support(),
        "Should not have transaction support for API-only client"
    );
    assert!(
        client.signer_address().is_none(),
        "Signer address should be None for API-only client"
    );
}

#[test]
fn test_vault_v1_returns_error_without_config() {
    let client = MorphoClient::new();

    let result = client.vault_v1();
    assert!(
        matches!(result, Err(ApiError::TransactionNotConfigured)),
        "vault_v1() should return TransactionNotConfigured error"
    );
}

#[test]
fn test_vault_v2_returns_error_without_config() {
    let client = MorphoClient::new();

    let result = client.vault_v2();
    assert!(
        matches!(result, Err(ApiError::TransactionNotConfigured)),
        "vault_v2() should return TransactionNotConfigured error"
    );
}

#[test]
fn test_auto_approve_default_true() {
    let client = MorphoClient::new();
    assert!(client.auto_approve(), "auto_approve should default to true");
}

#[test]
fn test_auto_approve_can_be_disabled() {
    let config = MorphoClientConfig::new()
        .with_rpc_url("http://localhost:8545")
        .with_private_key(TEST_PRIVATE_KEY)
        .with_auto_approve(false);
    let client = MorphoClient::with_config(config).expect("Failed to create client");

    assert!(
        !client.auto_approve(),
        "auto_approve should be false when disabled"
    );
}

#[test]
fn test_vault_operations_inherit_auto_approve() {
    let config = MorphoClientConfig::new()
        .with_rpc_url("http://localhost:8545")
        .with_private_key(TEST_PRIVATE_KEY)
        .with_auto_approve(false);
    let client = MorphoClient::with_config(config).expect("Failed to create client");

    let v1_ops = client.vault_v1().expect("Failed to get v1 operations");
    let v2_ops = client.vault_v2().expect("Failed to get v2 operations");

    assert!(!v1_ops.auto_approve(), "V1 operations should inherit auto_approve=false");
    assert!(!v2_ops.auto_approve(), "V2 operations should inherit auto_approve=false");
}

#[test]
fn test_vault_operations_signer_address() {
    let config = MorphoClientConfig::new()
        .with_rpc_url("http://localhost:8545")
        .with_private_key(TEST_PRIVATE_KEY);
    let client = MorphoClient::with_config(config).expect("Failed to create client");

    let v1_ops = client.vault_v1().expect("Failed to get v1 operations");
    let v2_ops = client.vault_v2().expect("Failed to get v2 operations");

    assert_eq!(
        v1_ops.signer_address(),
        EXPECTED_SIGNER_ADDRESS,
        "V1 operations signer should match"
    );
    assert_eq!(
        v2_ops.signer_address(),
        EXPECTED_SIGNER_ADDRESS,
        "V2 operations signer should match"
    );
}

// ============================================================================
// VaultV1Operations Fork Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_v1_operations_get_asset() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    let config = MorphoClientConfig::new()
        .with_rpc_url(&anvil.endpoint())
        .with_private_key(TEST_PRIVATE_KEY);
    let client = MorphoClient::with_config(config).expect("Failed to create client");

    let v1_ops = client.vault_v1().expect("Failed to get v1 operations");
    let asset = v1_ops.get_asset(STEAKHOUSE_USDC_VAULT).await.expect("Failed to get asset");

    assert_eq!(asset, USDC_ADDRESS, "Vault asset should be USDC");
}

#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_v1_operations_get_balance() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    let config = MorphoClientConfig::new()
        .with_rpc_url(&anvil.endpoint())
        .with_private_key(TEST_PRIVATE_KEY);
    let client = MorphoClient::with_config(config).expect("Failed to create client");

    let v1_ops = client.vault_v1().expect("Failed to get v1 operations");

    // Fresh account should have zero balance
    let balance = v1_ops.balance(STEAKHOUSE_USDC_VAULT).await.expect("Failed to get balance");
    assert_eq!(balance, U256::ZERO, "Fresh account should have zero vault balance");
}

#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_v1_operations_deposit_with_auto_approve() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    let anvil_provider = ProviderBuilder::new().connect_http(anvil.endpoint_url());

    let config = MorphoClientConfig::new()
        .with_rpc_url(&anvil.endpoint())
        .with_private_key(TEST_PRIVATE_KEY)
        .with_auto_approve(true);
    let client = MorphoClient::with_config(config).expect("Failed to create client");

    let v1_ops = client.vault_v1().expect("Failed to get v1 operations");
    let signer = v1_ops.signer_address();

    // Fund account with USDC
    let deposit_amount = U256::from(1_000_000_000u64); // 1,000 USDC
    fund_account_with_usdc(&anvil_provider, signer, deposit_amount).await;

    // Get balance before deposit
    let balance_before = v1_ops.balance(STEAKHOUSE_USDC_VAULT).await.expect("Failed to get balance before");

    // Deposit with auto_approve=true (should handle approval automatically)
    let receipt = v1_ops
        .deposit(STEAKHOUSE_USDC_VAULT, deposit_amount)
        .await
        .expect("Failed to deposit");

    assert!(receipt.status(), "Deposit transaction should succeed");

    // Get balance after deposit
    let balance_after = v1_ops.balance(STEAKHOUSE_USDC_VAULT).await.expect("Failed to get balance after");

    // Should have received shares
    assert!(
        balance_after > balance_before,
        "Should have received shares after deposit"
    );

    println!(
        "✓ V1 deposit with auto_approve: deposited {} USDC, received {} shares",
        deposit_amount,
        balance_after - balance_before
    );
}

#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_v1_operations_withdraw() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    let anvil_provider = ProviderBuilder::new().connect_http(anvil.endpoint_url());

    let config = MorphoClientConfig::new()
        .with_rpc_url(&anvil.endpoint())
        .with_private_key(TEST_PRIVATE_KEY)
        .with_auto_approve(true);
    let client = MorphoClient::with_config(config).expect("Failed to create client");

    let v1_ops = client.vault_v1().expect("Failed to get v1 operations");
    let signer = v1_ops.signer_address();

    // Fund account and deposit first
    let deposit_amount = U256::from(2_000_000_000u64); // 2,000 USDC
    fund_account_with_usdc(&anvil_provider, signer, deposit_amount).await;

    let _deposit_receipt = v1_ops
        .deposit(STEAKHOUSE_USDC_VAULT, deposit_amount)
        .await
        .expect("Failed to deposit");

    // Get max withdrawable amount
    let max_withdraw = v1_ops
        .max_withdraw(STEAKHOUSE_USDC_VAULT)
        .await
        .expect("Failed to get max withdraw");

    // Withdraw half
    let withdraw_amount = max_withdraw / U256::from(2);

    let withdraw_receipt = v1_ops
        .withdraw(STEAKHOUSE_USDC_VAULT, withdraw_amount)
        .await
        .expect("Failed to withdraw");

    assert!(withdraw_receipt.status(), "Withdraw transaction should succeed");

    println!("✓ V1 withdraw: withdrew {} USDC", withdraw_amount);
}

#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_v1_operations_get_decimals() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    let config = MorphoClientConfig::new()
        .with_rpc_url(&anvil.endpoint())
        .with_private_key(TEST_PRIVATE_KEY);
    let client = MorphoClient::with_config(config).expect("Failed to create client");

    let v1_ops = client.vault_v1().expect("Failed to get v1 operations");

    // USDC has 6 decimals
    let decimals = v1_ops.get_decimals(USDC_ADDRESS).await.expect("Failed to get decimals");
    assert_eq!(decimals, 6, "USDC should have 6 decimals");
}

#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_v1_operations_convert_to_assets() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    let config = MorphoClientConfig::new()
        .with_rpc_url(&anvil.endpoint())
        .with_private_key(TEST_PRIVATE_KEY);
    let client = MorphoClient::with_config(config).expect("Failed to create client");

    let v1_ops = client.vault_v1().expect("Failed to get v1 operations");

    // Convert some shares to assets
    let shares = U256::from(1_000_000u64); // 1 share unit
    let assets = v1_ops
        .convert_to_assets(STEAKHOUSE_USDC_VAULT, shares)
        .await
        .expect("Failed to convert to assets");

    // For a vault with share price > 1, assets should be > shares
    assert!(
        assets > shares,
        "Assets ({}) should be > shares ({}) for vault with share price > 1",
        assets,
        shares
    );

    println!("✓ V1 convert_to_assets: {} shares = {} assets", shares, assets);
}

// ============================================================================
// VaultV2Operations Fork Tests
// ============================================================================

#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_v2_operations_get_asset() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    let config = MorphoClientConfig::new()
        .with_rpc_url(&anvil.endpoint())
        .with_private_key(TEST_PRIVATE_KEY);
    let client = MorphoClient::with_config(config).expect("Failed to create client");

    let v2_ops = client.vault_v2().expect("Failed to get v2 operations");
    let asset = v2_ops.get_asset(STEAKHOUSE_USDC_VAULT).await.expect("Failed to get asset");

    assert_eq!(asset, USDC_ADDRESS, "Vault asset should be USDC");
}

#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_v2_operations_deposit() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    let anvil_provider = ProviderBuilder::new().connect_http(anvil.endpoint_url());

    let config = MorphoClientConfig::new()
        .with_rpc_url(&anvil.endpoint())
        .with_private_key(TEST_PRIVATE_KEY)
        .with_auto_approve(true);
    let client = MorphoClient::with_config(config).expect("Failed to create client");

    let v2_ops = client.vault_v2().expect("Failed to get v2 operations");
    let signer = v2_ops.signer_address();

    // Fund account with USDC
    let deposit_amount = U256::from(1_000_000_000u64); // 1,000 USDC
    fund_account_with_usdc(&anvil_provider, signer, deposit_amount).await;

    // Get balance before deposit
    let balance_before = v2_ops.balance(STEAKHOUSE_USDC_VAULT).await.expect("Failed to get balance before");

    // Deposit with auto_approve=true
    let receipt = v2_ops
        .deposit(STEAKHOUSE_USDC_VAULT, deposit_amount)
        .await
        .expect("Failed to deposit");

    assert!(receipt.status(), "Deposit transaction should succeed");

    // Get balance after deposit
    let balance_after = v2_ops.balance(STEAKHOUSE_USDC_VAULT).await.expect("Failed to get balance after");

    // Should have received shares
    assert!(
        balance_after > balance_before,
        "Should have received shares after deposit"
    );

    println!(
        "✓ V2 deposit: deposited {} USDC, received {} shares",
        deposit_amount,
        balance_after - balance_before
    );
}

#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_v2_operations_get_allowance() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    let config = MorphoClientConfig::new()
        .with_rpc_url(&anvil.endpoint())
        .with_private_key(TEST_PRIVATE_KEY);
    let client = MorphoClient::with_config(config).expect("Failed to create client");

    let v2_ops = client.vault_v2().expect("Failed to get v2 operations");

    // Fresh account should have zero allowance
    let allowance = v2_ops
        .get_allowance(STEAKHOUSE_USDC_VAULT)
        .await
        .expect("Failed to get allowance");

    assert_eq!(allowance, U256::ZERO, "Fresh account should have zero allowance");
}

#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_v2_operations_approve_returns_none_when_sufficient() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    let config = MorphoClientConfig::new()
        .with_rpc_url(&anvil.endpoint())
        .with_private_key(TEST_PRIVATE_KEY);
    let client = MorphoClient::with_config(config).expect("Failed to create client");

    let v2_ops = client.vault_v2().expect("Failed to get v2 operations");

    // First approve a large amount
    let large_amount = U256::from(10_000_000_000u64);
    let first_approval = v2_ops
        .approve(STEAKHOUSE_USDC_VAULT, large_amount)
        .await
        .expect("Failed to approve");

    // First approval should return Some (because we had zero allowance)
    assert!(
        first_approval.is_some(),
        "First approval should return Some"
    );

    // Now try to approve a smaller amount
    let small_amount = U256::from(1_000_000_000u64);
    let second_approval = v2_ops
        .approve(STEAKHOUSE_USDC_VAULT, small_amount)
        .await
        .expect("Failed to check approval");

    // Second approval should return None (sufficient allowance exists)
    assert!(
        second_approval.is_none(),
        "Second approval should return None when sufficient allowance exists"
    );
}
