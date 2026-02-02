//! End-to-end transaction tests with Anvil fork.
//!
//! These tests are ignored by default because they require:
//! - ETH_RPC_URL environment variable set to a valid Ethereum RPC endpoint
//! - Network access to fork from the RPC endpoint
//!
//! Run with: `ETH_RPC_URL="https://eth.llamarpc.com" cargo test -p morpho-rs-cli --test integration -- --ignored`

use predicates::prelude::*;

use super::helpers::morpho_cmd;

/// Test that vaultv1 deposit command validates required arguments.
/// This doesn't perform an actual deposit but verifies CLI argument handling.
#[test]
#[ignore]
fn test_vaultv1_deposit_missing_private_key() {
    morpho_cmd()
        .args([
            "vaultv1",
            "deposit",
            "0x8eB67A509616cd6A7c1B3c8C21D48FF57df3d458",
            "1000000",
            "--rpc-url",
            "https://eth.llamarpc.com",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--private-key").or(predicate::str::contains("required")));
}

/// Test that vaultv1 deposit command validates required arguments.
#[test]
#[ignore]
fn test_vaultv1_deposit_missing_rpc_url() {
    morpho_cmd()
        .args([
            "vaultv1",
            "deposit",
            "0x8eB67A509616cd6A7c1B3c8C21D48FF57df3d458",
            "1000000",
            "--private-key",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--rpc-url").or(predicate::str::contains("required")));
}

/// Test that vaultv1 withdraw command validates required arguments.
#[test]
#[ignore]
fn test_vaultv1_withdraw_missing_private_key() {
    morpho_cmd()
        .args([
            "vaultv1",
            "withdraw",
            "0x8eB67A509616cd6A7c1B3c8C21D48FF57df3d458",
            "1000000",
            "--rpc-url",
            "https://eth.llamarpc.com",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--private-key").or(predicate::str::contains("required")));
}

/// Test that vaultv1 withdraw command validates required arguments.
#[test]
#[ignore]
fn test_vaultv1_withdraw_missing_rpc_url() {
    morpho_cmd()
        .args([
            "vaultv1",
            "withdraw",
            "0x8eB67A509616cd6A7c1B3c8C21D48FF57df3d458",
            "1000000",
            "--private-key",
            "0x0000000000000000000000000000000000000000000000000000000000000001",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--rpc-url").or(predicate::str::contains("required")));
}
