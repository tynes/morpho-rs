//! End-to-end fork tests for verifying ERC4626 withdraw behavior.
//!
//! These tests verify that the `withdraw` function operates on assets (not shares)
//! by forking mainnet and interacting with a real vault.
//!
//! Run with: `cargo test --test e2e_fork -- --ignored`
//! Requires `ETH_RPC_URL` environment variable to be set.

use alloy::{
    network::Ethereum,
    node_bindings::{Anvil, AnvilInstance},
    primitives::{address, keccak256, Address, U256},
    providers::{ext::AnvilApi, ProviderBuilder},
    sol_types::SolValue,
};
use morpho_rs_contracts::{Erc4626Client, VaultV1TransactionClient};

// Steakhouse USDC vault on mainnet - has share price > 1
const STEAKHOUSE_USDC_VAULT: Address = address!("BEEF01735c132Ada46AA9aA4c54623cAA92A64CB");
// USDC on mainnet
const USDC_ADDRESS: Address = address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
// USDC balanceOf mapping is at slot 9
const USDC_BALANCE_SLOT: U256 = U256::from_limbs([9, 0, 0, 0]);
// Anvil's default account 0 private key
const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

/// Reads an env var, returning the default if not set or invalid.
fn env_var_or_default<T: std::str::FromStr>(name: &str, default: T) -> T {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

/// Spawns a forked Anvil instance with rate limiting protection.
///
/// Configuration is read from environment variables with sensible defaults:
/// - `ETH_RPC_URL` (required): The RPC URL to fork from
/// - `ANVIL_COMPUTE_UNITS_PER_SECOND` (default: 100): Compute units per second
/// - `ANVIL_RETRIES` (default: 5): Number of retries for failed requests
/// - `ANVIL_FORK_RETRY_BACKOFF` (default: 1000): Backoff in ms between retries
/// - `ANVIL_TIMEOUT` (default: 45000): Timeout in ms for RPC requests
///
/// Returns `None` if `ETH_RPC_URL` is not set.
fn spawn_forked_anvil() -> Option<AnvilInstance> {
    let rpc_url = match std::env::var("ETH_RPC_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("Skipping test: ETH_RPC_URL not set");
            return None;
        }
    };

    let compute_units = env_var_or_default("ANVIL_COMPUTE_UNITS_PER_SECOND", 100u64);
    let retries = env_var_or_default("ANVIL_RETRIES", 5u32);
    let backoff = env_var_or_default("ANVIL_FORK_RETRY_BACKOFF", 1000u64);
    let timeout = env_var_or_default("ANVIL_TIMEOUT", 45000u64);

    let anvil = Anvil::new()
        .fork(&rpc_url)
        .arg("--compute-units-per-second")
        .arg(compute_units.to_string())
        .arg("--retries")
        .arg(retries.to_string())
        .arg("--fork-retry-backoff")
        .arg(backoff.to_string())
        .timeout(timeout)
        .try_spawn()
        .expect("Failed to spawn Anvil");

    Some(anvil)
}

/// Fund an account with USDC by manipulating storage directly.
async fn fund_account_with_usdc<P: AnvilApi<Ethereum>>(provider: &P, account: Address, amount: U256) {
    // Calculate the storage slot for balanceOf[account]
    // For mapping(address => uint256), slot = keccak256(abi.encode(key, slot))
    let slot_hash = keccak256((account, USDC_BALANCE_SLOT).abi_encode());

    provider
        .anvil_set_storage_at(USDC_ADDRESS, slot_hash.into(), amount.into())
        .await
        .expect("Failed to set USDC balance");
}

/// E2E test verifying that withdraw() operates on assets, not shares.
///
/// This test:
/// 1. Forks mainnet and deposits into a vault with share price > 1
/// 2. Calls withdraw() with a specific asset amount
/// 3. Verifies that the exact asset amount is received (proving withdraw uses assets)
#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_withdraw_uses_assets_not_shares() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    // Create a provider for Anvil-specific operations (storage manipulation)
    let anvil_provider = ProviderBuilder::new().connect_http(anvil.endpoint_url());

    // Create the vault client for ERC20/ERC4626 operations
    let client = VaultV1TransactionClient::new(&anvil.endpoint(), TEST_PRIVATE_KEY)
        .expect("Failed to create client");
    let test_account = client.signer_address();

    // Fund test account with 10,000 USDC (USDC has 6 decimals)
    let deposit_amount = U256::from(10_000_000_000u64); // 10,000 USDC
    fund_account_with_usdc(&anvil_provider, test_account, deposit_amount).await;

    // Verify USDC balance
    let usdc_balance = client
        .get_balance(USDC_ADDRESS, test_account)
        .await
        .expect("Failed to get USDC balance");
    assert_eq!(usdc_balance, deposit_amount, "USDC funding failed");

    // Approve vault to spend USDC
    let _approve_receipt = client
        .approve(USDC_ADDRESS, STEAKHOUSE_USDC_VAULT, deposit_amount)
        .send()
        .await
        .expect("Failed to approve");

    // Deposit USDC into vault
    let deposit_receipt = client
        .deposit(STEAKHOUSE_USDC_VAULT, deposit_amount, test_account)
        .send()
        .await
        .expect("Failed to deposit");

    assert!(deposit_receipt.status(), "Deposit transaction failed");

    // Check shares received (vault shares are ERC20 tokens on the vault address)
    let shares_balance = client
        .get_balance(STEAKHOUSE_USDC_VAULT, test_account)
        .await
        .expect("Failed to get shares balance");

    // Key assertion: shares received should be LESS than assets deposited
    // because the vault's share price is > 1
    assert!(
        shares_balance < deposit_amount,
        "Expected shares ({}) < deposit amount ({}) because share price > 1",
        shares_balance,
        deposit_amount
    );

    println!(
        "Deposited {} USDC, received {} shares (share price > 1 confirmed)",
        deposit_amount, shares_balance
    );

    // Now test withdraw - request a specific ASSET amount
    let withdraw_asset_amount = U256::from(1_000_000_000u64); // 1,000 USDC

    // Get USDC balance before withdraw
    let usdc_before = client
        .get_balance(USDC_ADDRESS, test_account)
        .await
        .expect("Failed to get USDC balance before");

    // Get shares balance before withdraw
    let shares_before = client
        .get_balance(STEAKHOUSE_USDC_VAULT, test_account)
        .await
        .expect("Failed to get shares before");

    // Call withdraw with asset amount
    let withdraw_receipt = client
        .withdraw(STEAKHOUSE_USDC_VAULT, withdraw_asset_amount, test_account, test_account)
        .send()
        .await
        .expect("Failed to withdraw");

    assert!(withdraw_receipt.status(), "Withdraw transaction failed");

    // Get USDC balance after withdraw
    let usdc_after = client
        .get_balance(USDC_ADDRESS, test_account)
        .await
        .expect("Failed to get USDC balance after");

    // Get shares balance after withdraw
    let shares_after = client
        .get_balance(STEAKHOUSE_USDC_VAULT, test_account)
        .await
        .expect("Failed to get shares after");

    // Calculate actual amounts
    let usdc_received = usdc_after - usdc_before;
    let shares_burned = shares_before - shares_after;

    println!(
        "Withdraw requested: {} USDC (assets)",
        withdraw_asset_amount
    );
    println!("USDC received: {}", usdc_received);
    println!("Shares burned: {}", shares_burned);

    // KEY ASSERTION: withdraw() should return exactly the requested ASSET amount
    // This proves that the withdraw function parameter is assets, not shares
    assert_eq!(
        usdc_received, withdraw_asset_amount,
        "withdraw() should return exactly the requested asset amount"
    );

    // Additional assertion: fewer shares should be burned than assets received
    // because share price > 1
    assert!(
        shares_burned < withdraw_asset_amount,
        "Expected shares burned ({}) < assets received ({}) because share price > 1",
        shares_burned,
        withdraw_asset_amount
    );

    println!("\n✓ Test passed: withdraw() operates on ASSETS, not shares");
    println!(
        "  - Requested {} assets, received exactly {} USDC",
        withdraw_asset_amount, usdc_received
    );
    println!(
        "  - Only {} shares were burned (less than assets due to share price > 1)",
        shares_burned
    );
}

/// Test that demonstrates the share/asset conversion rates.
#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_share_price_is_greater_than_one() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    // Create the vault client (private key doesn't matter for view-only operations)
    let client = VaultV1TransactionClient::new(&anvil.endpoint(), TEST_PRIVATE_KEY)
        .expect("Failed to create client");

    // Check conversion rates
    let one_usdc = U256::from(1_000_000u64); // 1 USDC (6 decimals)

    let shares_for_one_usdc = client
        .convert_to_shares(STEAKHOUSE_USDC_VAULT, one_usdc)
        .await
        .expect("Failed to convert to shares");

    let assets_for_one_share = client
        .convert_to_assets(STEAKHOUSE_USDC_VAULT, one_usdc) // Using same scale for comparison
        .await
        .expect("Failed to convert to assets");

    println!("Steakhouse USDC Vault conversion rates:");
    println!("  1 USDC ({}) -> {} shares", one_usdc, shares_for_one_usdc);
    println!(
        "  1 share unit ({}) -> {} assets (USDC)",
        one_usdc, assets_for_one_share
    );

    // Share price > 1 means you get fewer shares for your assets
    assert!(
        shares_for_one_usdc < one_usdc,
        "Expected shares ({}) < assets ({}) for share price > 1",
        shares_for_one_usdc,
        one_usdc
    );

    // And more assets for your shares
    assert!(
        assets_for_one_share > one_usdc,
        "Expected assets ({}) > shares ({}) for share price > 1",
        assets_for_one_share,
        one_usdc
    );

    println!("\n✓ Confirmed: Share price > 1 for Steakhouse USDC vault");
}

/// Test the Erc4626Client trait methods against a real vault.
#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_erc4626_client_view_functions() {
    use morpho_rs_contracts::{Erc4626Client, VaultV1TransactionClient, VaultV2TransactionClient};

    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    // Create both V1 and V2 clients to verify trait works on both
    let v1_client = VaultV1TransactionClient::new(&anvil.endpoint(), TEST_PRIVATE_KEY)
        .expect("Failed to create V1 client");
    let v2_client = VaultV2TransactionClient::new(&anvil.endpoint(), TEST_PRIVATE_KEY)
        .expect("Failed to create V2 client");

    let test_amount = U256::from(1_000_000u64); // 1 USDC

    // Test get_asset (existing)
    let asset_v1 = v1_client.get_asset(STEAKHOUSE_USDC_VAULT).await
        .expect("V1: Failed to get asset");
    let asset_v2 = v2_client.get_asset(STEAKHOUSE_USDC_VAULT).await
        .expect("V2: Failed to get asset");
    assert_eq!(asset_v1, USDC_ADDRESS, "V1: Asset should be USDC");
    assert_eq!(asset_v2, USDC_ADDRESS, "V2: Asset should be USDC");

    // Test total_assets (new)
    let total_assets_v1 = v1_client.total_assets(STEAKHOUSE_USDC_VAULT).await
        .expect("V1: Failed to get total assets");
    let total_assets_v2 = v2_client.total_assets(STEAKHOUSE_USDC_VAULT).await
        .expect("V2: Failed to get total assets");
    assert_eq!(total_assets_v1, total_assets_v2, "V1 and V2 should return same total assets");
    assert!(total_assets_v1 > U256::ZERO, "Total assets should be > 0");
    println!("Total assets: {}", total_assets_v1);

    // Test convert_to_shares (new)
    let shares_v1 = v1_client.convert_to_shares(STEAKHOUSE_USDC_VAULT, test_amount).await
        .expect("V1: Failed to convert to shares");
    let shares_v2 = v2_client.convert_to_shares(STEAKHOUSE_USDC_VAULT, test_amount).await
        .expect("V2: Failed to convert to shares");
    assert_eq!(shares_v1, shares_v2, "V1 and V2 should return same shares");
    assert!(shares_v1 < test_amount, "Shares should be less than assets (share price > 1)");
    println!("Convert {} assets to {} shares", test_amount, shares_v1);

    // Test convert_to_assets (new)
    let assets_v1 = v1_client.convert_to_assets(STEAKHOUSE_USDC_VAULT, test_amount).await
        .expect("V1: Failed to convert to assets");
    let assets_v2 = v2_client.convert_to_assets(STEAKHOUSE_USDC_VAULT, test_amount).await
        .expect("V2: Failed to convert to assets");
    assert_eq!(assets_v1, assets_v2, "V1 and V2 should return same assets");
    assert!(assets_v1 > test_amount, "Assets should be greater than shares (share price > 1)");
    println!("Convert {} shares to {} assets", test_amount, assets_v1);

    // Test max_deposit (new)
    let test_receiver = v1_client.signer_address();
    let max_deposit_v1 = v1_client.max_deposit(STEAKHOUSE_USDC_VAULT, test_receiver).await
        .expect("V1: Failed to get max deposit");
    let max_deposit_v2 = v2_client.max_deposit(STEAKHOUSE_USDC_VAULT, test_receiver).await
        .expect("V2: Failed to get max deposit");
    assert_eq!(max_deposit_v1, max_deposit_v2, "V1 and V2 should return same max deposit");
    println!("Max deposit: {}", max_deposit_v1);

    // Test max_withdraw (new)
    let max_withdraw_v1 = v1_client.max_withdraw(STEAKHOUSE_USDC_VAULT, test_receiver).await
        .expect("V1: Failed to get max withdraw");
    let max_withdraw_v2 = v2_client.max_withdraw(STEAKHOUSE_USDC_VAULT, test_receiver).await
        .expect("V2: Failed to get max withdraw");
    assert_eq!(max_withdraw_v1, max_withdraw_v2, "V1 and V2 should return same max withdraw");
    println!("Max withdraw for test account: {}", max_withdraw_v1);

    // Test max_mint (new)
    let max_mint_v1 = v1_client.max_mint(STEAKHOUSE_USDC_VAULT, test_receiver).await
        .expect("V1: Failed to get max mint");
    let max_mint_v2 = v2_client.max_mint(STEAKHOUSE_USDC_VAULT, test_receiver).await
        .expect("V2: Failed to get max mint");
    assert_eq!(max_mint_v1, max_mint_v2, "V1 and V2 should return same max mint");
    println!("Max mint: {}", max_mint_v1);

    // Test max_redeem (new)
    let max_redeem_v1 = v1_client.max_redeem(STEAKHOUSE_USDC_VAULT, test_receiver).await
        .expect("V1: Failed to get max redeem");
    let max_redeem_v2 = v2_client.max_redeem(STEAKHOUSE_USDC_VAULT, test_receiver).await
        .expect("V2: Failed to get max redeem");
    assert_eq!(max_redeem_v1, max_redeem_v2, "V1 and V2 should return same max redeem");
    println!("Max redeem for test account: {}", max_redeem_v1);

    // Test preview_deposit (new)
    let preview_deposit_v1 = v1_client.preview_deposit(STEAKHOUSE_USDC_VAULT, test_amount).await
        .expect("V1: Failed to preview deposit");
    let preview_deposit_v2 = v2_client.preview_deposit(STEAKHOUSE_USDC_VAULT, test_amount).await
        .expect("V2: Failed to preview deposit");
    assert_eq!(preview_deposit_v1, preview_deposit_v2, "V1 and V2 should return same preview deposit");
    assert_eq!(preview_deposit_v1, shares_v1, "Preview deposit should equal convert_to_shares");
    println!("Preview deposit {} assets -> {} shares", test_amount, preview_deposit_v1);

    // Test preview_mint (new)
    let preview_mint_v1 = v1_client.preview_mint(STEAKHOUSE_USDC_VAULT, test_amount).await
        .expect("V1: Failed to preview mint");
    let preview_mint_v2 = v2_client.preview_mint(STEAKHOUSE_USDC_VAULT, test_amount).await
        .expect("V2: Failed to preview mint");
    assert_eq!(preview_mint_v1, preview_mint_v2, "V1 and V2 should return same preview mint");
    println!("Preview mint {} shares -> {} assets needed", test_amount, preview_mint_v1);

    // Test preview_withdraw (new)
    let preview_withdraw_v1 = v1_client.preview_withdraw(STEAKHOUSE_USDC_VAULT, test_amount).await
        .expect("V1: Failed to preview withdraw");
    let preview_withdraw_v2 = v2_client.preview_withdraw(STEAKHOUSE_USDC_VAULT, test_amount).await
        .expect("V2: Failed to preview withdraw");
    assert_eq!(preview_withdraw_v1, preview_withdraw_v2, "V1 and V2 should return same preview withdraw");
    println!("Preview withdraw {} assets -> {} shares burned", test_amount, preview_withdraw_v1);

    // Test preview_redeem (new)
    let preview_redeem_v1 = v1_client.preview_redeem(STEAKHOUSE_USDC_VAULT, test_amount).await
        .expect("V1: Failed to preview redeem");
    let preview_redeem_v2 = v2_client.preview_redeem(STEAKHOUSE_USDC_VAULT, test_amount).await
        .expect("V2: Failed to preview redeem");
    assert_eq!(preview_redeem_v1, preview_redeem_v2, "V1 and V2 should return same preview redeem");
    assert_eq!(preview_redeem_v1, assets_v1, "Preview redeem should equal convert_to_assets");
    println!("Preview redeem {} shares -> {} assets", test_amount, preview_redeem_v1);

    println!("\n✓ All Erc4626Client view functions work correctly on both V1 and V2 clients");
}

/// E2E test verifying that mint() operates on shares, not assets.
///
/// This test:
/// 1. Forks mainnet and prepares to mint shares in a vault with share price > 1
/// 2. Calls mint() with a specific share amount
/// 3. Verifies that the exact share amount is received (proving mint uses shares)
/// 4. Verifies that MORE assets were consumed than shares received
#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_mint_uses_shares_not_assets() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    // Create a provider for Anvil-specific operations (storage manipulation)
    let anvil_provider = ProviderBuilder::new().connect_http(anvil.endpoint_url());

    // Create the vault client for ERC20/ERC4626 operations
    let client = VaultV1TransactionClient::new(&anvil.endpoint(), TEST_PRIVATE_KEY)
        .expect("Failed to create client");
    let test_account = client.signer_address();

    // Fund test account with 10,000 USDC (USDC has 6 decimals)
    let fund_amount = U256::from(10_000_000_000u64); // 10,000 USDC
    fund_account_with_usdc(&anvil_provider, test_account, fund_amount).await;

    // Calculate how many shares we want to mint
    // We'll mint a specific share amount and verify we get exactly that many shares
    let shares_to_mint = U256::from(500_000_000u64); // 500 shares (scaled)

    // Preview how many assets will be needed to mint these shares
    let assets_needed = client
        .preview_mint(STEAKHOUSE_USDC_VAULT, shares_to_mint)
        .await
        .expect("Failed to preview mint");

    println!("Shares to mint: {}", shares_to_mint);
    println!("Assets needed (preview): {}", assets_needed);

    // Approve vault to spend USDC (approve more than needed to be safe)
    let _approve_receipt = client
        .approve(USDC_ADDRESS, STEAKHOUSE_USDC_VAULT, assets_needed * U256::from(2))
        .send()
        .await
        .expect("Failed to approve");

    // Get balances before mint
    let usdc_before = client
        .get_balance(USDC_ADDRESS, test_account)
        .await
        .expect("Failed to get USDC balance before");
    let shares_before = client
        .get_balance(STEAKHOUSE_USDC_VAULT, test_account)
        .await
        .expect("Failed to get shares before");

    // Mint shares
    let mint_receipt = client
        .mint(STEAKHOUSE_USDC_VAULT, shares_to_mint, test_account)
        .send()
        .await
        .expect("Failed to mint");

    assert!(mint_receipt.status(), "Mint transaction failed");

    // Get balances after mint
    let usdc_after = client
        .get_balance(USDC_ADDRESS, test_account)
        .await
        .expect("Failed to get USDC balance after");
    let shares_after = client
        .get_balance(STEAKHOUSE_USDC_VAULT, test_account)
        .await
        .expect("Failed to get shares after");

    // Calculate actual amounts
    let shares_received = shares_after - shares_before;
    let assets_consumed = usdc_before - usdc_after;

    println!("Shares received: {}", shares_received);
    println!("Assets consumed: {}", assets_consumed);

    // KEY ASSERTION: mint() should return exactly the requested SHARE amount
    // This proves that the mint function parameter is shares, not assets
    assert_eq!(
        shares_received, shares_to_mint,
        "mint() should mint exactly the requested share amount"
    );

    // Additional assertion: more assets should be consumed than shares received
    // because share price > 1 (you pay more assets per share)
    assert!(
        assets_consumed > shares_received,
        "Expected assets consumed ({}) > shares received ({}) because share price > 1",
        assets_consumed,
        shares_received
    );

    println!("\n✓ Test passed: mint() operates on SHARES, not assets");
    println!(
        "  - Requested {} shares, received exactly {} shares",
        shares_to_mint, shares_received
    );
    println!(
        "  - Consumed {} assets (more than shares due to share price > 1)",
        assets_consumed
    );
}

/// E2E test verifying that redeem() operates on shares, not assets.
///
/// This test:
/// 1. Forks mainnet and deposits into a vault with share price > 1
/// 2. Calls redeem() with a specific share amount
/// 3. Verifies that the exact share amount is burned (proving redeem uses shares)
#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_redeem_uses_shares_not_assets() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    // Create a provider for Anvil-specific operations (storage manipulation)
    let anvil_provider = ProviderBuilder::new().connect_http(anvil.endpoint_url());

    // Create the vault client for ERC20/ERC4626 operations
    let client = VaultV1TransactionClient::new(&anvil.endpoint(), TEST_PRIVATE_KEY)
        .expect("Failed to create client");
    let test_account = client.signer_address();

    // Fund test account with 10,000 USDC (USDC has 6 decimals)
    let deposit_amount = U256::from(10_000_000_000u64); // 10,000 USDC
    fund_account_with_usdc(&anvil_provider, test_account, deposit_amount).await;

    // Approve and deposit to get shares
    let _approve_receipt = client
        .approve(USDC_ADDRESS, STEAKHOUSE_USDC_VAULT, deposit_amount)
        .send()
        .await
        .expect("Failed to approve");

    let deposit_receipt = client
        .deposit(STEAKHOUSE_USDC_VAULT, deposit_amount, test_account)
        .send()
        .await
        .expect("Failed to deposit");

    assert!(deposit_receipt.status(), "Deposit transaction failed");

    // Get shares balance after deposit
    let shares_after_deposit = client
        .get_balance(STEAKHOUSE_USDC_VAULT, test_account)
        .await
        .expect("Failed to get shares balance after deposit");

    println!("Deposited {} USDC, received {} shares", deposit_amount, shares_after_deposit);

    // Redeem a specific number of shares (half of what we have)
    let shares_to_redeem = shares_after_deposit / U256::from(2);

    // Preview how many assets we'll receive
    let assets_preview = client
        .preview_redeem(STEAKHOUSE_USDC_VAULT, shares_to_redeem)
        .await
        .expect("Failed to preview redeem");

    println!("Shares to redeem: {}", shares_to_redeem);
    println!("Assets expected (preview): {}", assets_preview);

    // Get balances before redeem
    let usdc_before = client
        .get_balance(USDC_ADDRESS, test_account)
        .await
        .expect("Failed to get USDC balance before");
    let shares_before = client
        .get_balance(STEAKHOUSE_USDC_VAULT, test_account)
        .await
        .expect("Failed to get shares before");

    // Redeem shares
    let redeem_receipt = client
        .redeem(STEAKHOUSE_USDC_VAULT, shares_to_redeem, test_account, test_account)
        .send()
        .await
        .expect("Failed to redeem");

    assert!(redeem_receipt.status(), "Redeem transaction failed");

    // Get balances after redeem
    let usdc_after = client
        .get_balance(USDC_ADDRESS, test_account)
        .await
        .expect("Failed to get USDC balance after");
    let shares_after = client
        .get_balance(STEAKHOUSE_USDC_VAULT, test_account)
        .await
        .expect("Failed to get shares after");

    // Calculate actual amounts
    let shares_burned = shares_before - shares_after;
    let assets_received = usdc_after - usdc_before;

    println!("Shares burned: {}", shares_burned);
    println!("Assets received: {}", assets_received);

    // KEY ASSERTION: redeem() should burn exactly the requested SHARE amount
    // This proves that the redeem function parameter is shares, not assets
    assert_eq!(
        shares_burned, shares_to_redeem,
        "redeem() should burn exactly the requested share amount"
    );

    // Additional assertion: more assets should be received than shares burned
    // because share price > 1 (each share is worth more than 1 asset)
    assert!(
        assets_received > shares_burned,
        "Expected assets received ({}) > shares burned ({}) because share price > 1",
        assets_received,
        shares_burned
    );

    println!("\n✓ Test passed: redeem() operates on SHARES, not assets");
    println!(
        "  - Requested {} shares to redeem, burned exactly {} shares",
        shares_to_redeem, shares_burned
    );
    println!(
        "  - Received {} assets (more than shares due to share price > 1)",
        assets_received
    );
}

/// Test that approve_if_needed skips approval when sufficient allowance exists.
#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_approve_if_needed_skips_when_sufficient() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    // Create the vault client
    let client = VaultV1TransactionClient::new(&anvil.endpoint(), TEST_PRIVATE_KEY)
        .expect("Failed to create client");

    let large_amount = U256::from(10_000_000_000u64); // 10,000 USDC
    let small_amount = U256::from(1_000_000_000u64); // 1,000 USDC

    // First, approve a large amount
    let initial_approval = client
        .approve(USDC_ADDRESS, STEAKHOUSE_USDC_VAULT, large_amount)
        .send()
        .await
        .expect("Failed to approve initial amount");

    assert!(initial_approval.status(), "Initial approval failed");

    // Verify allowance is set
    let allowance = client
        .get_allowance(USDC_ADDRESS, client.signer_address(), STEAKHOUSE_USDC_VAULT)
        .await
        .expect("Failed to get allowance");

    assert_eq!(allowance, large_amount, "Allowance should be set to large amount");

    // Now call approve_if_needed with a smaller amount
    let result = client
        .approve_if_needed(USDC_ADDRESS, STEAKHOUSE_USDC_VAULT, small_amount)
        .await
        .expect("approve_if_needed failed");

    // Should return None because sufficient allowance already exists
    assert!(
        result.is_none(),
        "approve_if_needed should return None when sufficient allowance exists"
    );

    // Verify allowance hasn't changed
    let allowance_after = client
        .get_allowance(USDC_ADDRESS, client.signer_address(), STEAKHOUSE_USDC_VAULT)
        .await
        .expect("Failed to get allowance after");

    assert_eq!(
        allowance_after, large_amount,
        "Allowance should remain unchanged"
    );

    println!("✓ Test passed: approve_if_needed correctly skips when sufficient allowance exists");
}

/// Test that approve_if_needed approves when insufficient allowance exists.
#[tokio::test]
#[ignore = "Requires ETH_RPC_URL environment variable"]
async fn test_approve_if_needed_approves_when_insufficient() {
    let Some(anvil) = spawn_forked_anvil() else {
        return;
    };

    // Create the vault client
    let client = VaultV1TransactionClient::new(&anvil.endpoint(), TEST_PRIVATE_KEY)
        .expect("Failed to create client");

    let requested_amount = U256::from(1_000_000_000u64); // 1,000 USDC

    // Verify initial allowance is zero (fresh anvil fork)
    let initial_allowance = client
        .get_allowance(USDC_ADDRESS, client.signer_address(), STEAKHOUSE_USDC_VAULT)
        .await
        .expect("Failed to get initial allowance");

    assert_eq!(initial_allowance, U256::ZERO, "Initial allowance should be zero");

    // Call approve_if_needed - should return Some since allowance is insufficient
    let result = client
        .approve_if_needed(USDC_ADDRESS, STEAKHOUSE_USDC_VAULT, requested_amount)
        .await
        .expect("approve_if_needed failed");

    // Should return Some(PreparedCall) because approval is needed
    assert!(
        result.is_some(),
        "approve_if_needed should return Some when allowance is insufficient"
    );

    // Get the prepared call and verify it targets the token
    let prepared = result.unwrap();
    assert_eq!(prepared.to(), USDC_ADDRESS, "Approval should target the token address");
    assert_eq!(prepared.value(), U256::ZERO, "Approval should not send ETH");

    // Send the approval
    let receipt = prepared.send().await.expect("Failed to send approval");
    assert!(receipt.status(), "Approval transaction failed");

    // Verify allowance is now set
    let allowance_after = client
        .get_allowance(USDC_ADDRESS, client.signer_address(), STEAKHOUSE_USDC_VAULT)
        .await
        .expect("Failed to get allowance after");

    assert_eq!(
        allowance_after, requested_amount,
        "Allowance should be set to requested amount"
    );

    println!("✓ Test passed: approve_if_needed correctly approves when insufficient allowance");
}
