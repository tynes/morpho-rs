//! Integration tests for V1 vault commands.

use predicates::prelude::*;

use super::helpers::{mock_graphql_error, mock_graphql_response, morpho_cmd_with_mock, start_mock_server};

#[tokio::test]
async fn test_v1_list_json_output() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_list").await;

    morpho_cmd_with_mock(&server)
        .args(["vaultv1", "list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Steakhouse USDC"))
        .stdout(predicate::str::contains("steakUSDC"))
        // Address is lowercased in output
        .stdout(predicate::str::contains("0x8eb67a509616cd6a7c1b3c8c21d48ff57df3d458"));
}

#[tokio::test]
async fn test_v1_list_table_output() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_list").await;

    morpho_cmd_with_mock(&server)
        .args(["vaultv1", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Steakhouse USDC"))
        .stdout(predicate::str::contains("Gauntlet WETH Prime"));
}

#[tokio::test]
async fn test_v1_list_with_limit() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_list").await;

    morpho_cmd_with_mock(&server)
        .args(["vaultv1", "list", "--limit", "10"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Steakhouse USDC"));
}

#[tokio::test]
async fn test_v1_list_with_chain_filter() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_list").await;

    morpho_cmd_with_mock(&server)
        .args(["vaultv1", "list", "--chain", "ethereum"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Steakhouse USDC"));
}

#[tokio::test]
async fn test_v1_list_whitelisted_only() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_list").await;

    morpho_cmd_with_mock(&server)
        .args(["vaultv1", "list", "--whitelisted"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Steakhouse USDC"));
}

#[tokio::test]
async fn test_v1_info_success() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_info").await;

    morpho_cmd_with_mock(&server)
        .args([
            "vaultv1",
            "info",
            "0x8eB67A509616cd6A7c1B3c8C21D48FF57df3d458",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Steakhouse USDC"))
        .stdout(predicate::str::contains("steakUSDC"));
}

#[tokio::test]
async fn test_v1_info_json_output() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v1_info").await;

    morpho_cmd_with_mock(&server)
        .args([
            "vaultv1",
            "info",
            "0x8eB67A509616cd6A7c1B3c8C21D48FF57df3d458",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"Steakhouse USDC\""))
        .stdout(predicate::str::contains("\"symbol\": \"steakUSDC\""));
}

#[tokio::test]
async fn test_v1_info_vault_not_found() {
    let server = start_mock_server().await;
    mock_graphql_error(&server, "Vault not found").await;

    morpho_cmd_with_mock(&server)
        .args([
            "vaultv1",
            "info",
            "0x0000000000000000000000000000000000000000",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Vault not found"));
}
