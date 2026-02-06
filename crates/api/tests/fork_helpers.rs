//! Fork test helper utilities for API crate integration tests.
//!
//! Shared utilities for tests that require forking mainnet and interacting
//! with real contracts.

use alloy::{
    network::Ethereum,
    node_bindings::{Anvil, AnvilInstance},
    primitives::{address, keccak256, Address, U256},
    providers::ext::AnvilApi,
    sol_types::SolValue,
};

// Steakhouse USDC vault on mainnet - has share price > 1
pub const STEAKHOUSE_USDC_VAULT: Address = address!("BEEF01735c132Ada46AA9aA4c54623cAA92A64CB");
// USDC on mainnet
pub const USDC_ADDRESS: Address = address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
// USDC balanceOf mapping is at slot 9
const USDC_BALANCE_SLOT: U256 = U256::from_limbs([9, 0, 0, 0]);
// Anvil's default account 0 private key
pub const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
// Expected address for the test private key
pub const EXPECTED_SIGNER_ADDRESS: Address = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");

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
pub fn spawn_forked_anvil() -> Option<AnvilInstance> {
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
pub async fn fund_account_with_usdc<P: AnvilApi<Ethereum>>(provider: &P, account: Address, amount: U256) {
    // Calculate the storage slot for balanceOf[account]
    // For mapping(address => uint256), slot = keccak256(abi.encode(key, slot))
    let slot_hash = keccak256((account, USDC_BALANCE_SLOT).abi_encode());

    provider
        .anvil_set_storage_at(USDC_ADDRESS, slot_hash.into(), amount.into())
        .await
        .expect("Failed to set USDC balance");
}
