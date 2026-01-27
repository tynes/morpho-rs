//! End-to-end fork tests for verifying ERC4626 withdraw behavior.
//!
//! These tests verify that the `withdraw` function operates on assets (not shares)
//! by forking mainnet and interacting with a real vault.
//!
//! Run with: `cargo test --test e2e_fork -- --ignored`
//! Requires `ETH_RPC_URL` environment variable to be set.

use alloy::{
    network::{Ethereum, EthereumWallet},
    node_bindings::Anvil,
    primitives::{address, keccak256, Address, U256},
    providers::{ext::AnvilApi, ProviderBuilder},
    signers::local::PrivateKeySigner,
    sol_types::SolValue,
};

// Steakhouse USDC vault on mainnet - has share price > 1
const STEAKHOUSE_USDC_VAULT: Address = address!("BEEF01735c132Ada46AA9aA4c54623cAA92A64CB");
// USDC on mainnet
const USDC_ADDRESS: Address = address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
// USDC balanceOf mapping is at slot 9
const USDC_BALANCE_SLOT: U256 = U256::from_limbs([9, 0, 0, 0]);
// Anvil's default account 0 private key
const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";

use alloy::sol;

sol! {
    #[sol(rpc)]
    interface IERC20 {
        function approve(address spender, uint256 amount) external returns (bool);
        function balanceOf(address account) external view returns (uint256);
    }

    #[sol(rpc)]
    interface IERC4626 {
        function deposit(uint256 assets, address receiver) external returns (uint256 shares);
        function withdraw(uint256 assets, address receiver, address owner) external returns (uint256 shares);
        function balanceOf(address account) external view returns (uint256);
        function convertToShares(uint256 assets) external view returns (uint256 shares);
        function convertToAssets(uint256 shares) external view returns (uint256 assets);
    }
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
    // Get RPC URL from environment
    let rpc_url = match std::env::var("ETH_RPC_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("Skipping test: ETH_RPC_URL not set");
            return;
        }
    };

    // Spawn Anvil forking mainnet
    let anvil = Anvil::new()
        .fork(rpc_url)
        .try_spawn()
        .expect("Failed to spawn Anvil");

    // Set up provider with test wallet
    let signer: PrivateKeySigner = TEST_PRIVATE_KEY
        .parse()
        .expect("Failed to parse private key");
    let test_account = signer.address();
    let wallet = EthereumWallet::from(signer);

    let provider = ProviderBuilder::new()
        .wallet(wallet)
        .connect_http(anvil.endpoint_url());

    // Fund test account with 10,000 USDC (USDC has 6 decimals)
    let deposit_amount = U256::from(10_000_000_000u64); // 10,000 USDC
    fund_account_with_usdc(&provider, test_account, deposit_amount).await;

    // Verify USDC balance
    let usdc = IERC20::new(USDC_ADDRESS, &provider);
    let usdc_balance = usdc
        .balanceOf(test_account)
        .call()
        .await
        .expect("Failed to get USDC balance");
    assert_eq!(usdc_balance, deposit_amount, "USDC funding failed");

    // Approve vault to spend USDC
    let _approve_receipt = usdc
        .approve(STEAKHOUSE_USDC_VAULT, deposit_amount)
        .send()
        .await
        .expect("Failed to send approve")
        .get_receipt()
        .await
        .expect("Failed to get approve receipt");

    // Deposit USDC into vault
    let vault = IERC4626::new(STEAKHOUSE_USDC_VAULT, &provider);
    let deposit_receipt = vault
        .deposit(deposit_amount, test_account)
        .send()
        .await
        .expect("Failed to send deposit")
        .get_receipt()
        .await
        .expect("Failed to get deposit receipt");

    assert!(deposit_receipt.status(), "Deposit transaction failed");

    // Check shares received
    let shares_balance = vault
        .balanceOf(test_account)
        .call()
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
    let usdc_before = usdc
        .balanceOf(test_account)
        .call()
        .await
        .expect("Failed to get USDC balance before");

    // Get shares balance before withdraw
    let shares_before = vault
        .balanceOf(test_account)
        .call()
        .await
        .expect("Failed to get shares before");

    // Call withdraw with asset amount
    let withdraw_receipt = vault
        .withdraw(withdraw_asset_amount, test_account, test_account)
        .send()
        .await
        .expect("Failed to send withdraw")
        .get_receipt()
        .await
        .expect("Failed to get withdraw receipt");

    assert!(withdraw_receipt.status(), "Withdraw transaction failed");

    // Get USDC balance after withdraw
    let usdc_after = usdc
        .balanceOf(test_account)
        .call()
        .await
        .expect("Failed to get USDC balance after");

    // Get shares balance after withdraw
    let shares_after = vault
        .balanceOf(test_account)
        .call()
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
    // Get RPC URL from environment
    let rpc_url = match std::env::var("ETH_RPC_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("Skipping test: ETH_RPC_URL not set");
            return;
        }
    };

    // Spawn Anvil forking mainnet
    let anvil = Anvil::new()
        .fork(rpc_url)
        .try_spawn()
        .expect("Failed to spawn Anvil");

    let provider = ProviderBuilder::new().connect_http(anvil.endpoint_url());

    let vault = IERC4626::new(STEAKHOUSE_USDC_VAULT, &provider);

    // Check conversion rates
    let one_usdc = U256::from(1_000_000u64); // 1 USDC (6 decimals)

    let shares_for_one_usdc = vault
        .convertToShares(one_usdc)
        .call()
        .await
        .expect("Failed to convert to shares");

    let assets_for_one_share = vault
        .convertToAssets(one_usdc) // Using same scale for comparison
        .call()
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
