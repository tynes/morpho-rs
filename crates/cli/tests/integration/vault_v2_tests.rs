//! Integration tests for V2 vault commands.

use predicates::prelude::*;

use super::helpers::{mock_graphql_error, mock_graphql_response, morpho_cmd_with_mock, start_mock_server};

#[tokio::test]
async fn test_v2_list_json_output() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    morpho_cmd_with_mock(&server)
        .args(["vaultv2", "list", "--format", "json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Test V2 USDC Vault"))
        .stdout(predicate::str::contains("tv2USDC"))
        // Address is lowercased in output
        .stdout(predicate::str::contains("0xabcdef1234567890abcdef1234567890abcdef12"));
}

#[tokio::test]
async fn test_v2_list_table_output() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    morpho_cmd_with_mock(&server)
        .args(["vaultv2", "list"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Test V2 USDC Vault"));
}

#[tokio::test]
async fn test_v2_list_with_limit() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    morpho_cmd_with_mock(&server)
        .args(["vaultv2", "list", "--limit", "5"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Test V2 USDC Vault"));
}

#[tokio::test]
async fn test_v2_list_with_chain_filter() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    morpho_cmd_with_mock(&server)
        .args(["vaultv2", "list", "--chain", "ethereum"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Test V2 USDC Vault"));
}

#[tokio::test]
async fn test_v2_list_whitelisted_only() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_list").await;

    morpho_cmd_with_mock(&server)
        .args(["vaultv2", "list", "--whitelisted"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Test V2 USDC Vault"));
}

#[tokio::test]
async fn test_v2_info_success() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_info").await;

    morpho_cmd_with_mock(&server)
        .args([
            "vaultv2",
            "info",
            "0xABCdef1234567890ABCdef1234567890ABCdef12",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Test V2 USDC Vault"))
        .stdout(predicate::str::contains("tv2USDC"));
}

#[tokio::test]
async fn test_v2_info_json_output() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "v2_info").await;

    morpho_cmd_with_mock(&server)
        .args([
            "vaultv2",
            "info",
            "0xABCdef1234567890ABCdef1234567890ABCdef12",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"name\": \"Test V2 USDC Vault\""))
        .stdout(predicate::str::contains("\"symbol\": \"tv2USDC\""));
}

#[tokio::test]
async fn test_v2_info_vault_not_found() {
    let server = start_mock_server().await;
    mock_graphql_error(&server, "Vault not found").await;

    morpho_cmd_with_mock(&server)
        .args([
            "vaultv2",
            "info",
            "0x0000000000000000000000000000000000000000",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Vault not found"));
}
