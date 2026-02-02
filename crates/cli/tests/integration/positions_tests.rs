//! Integration tests for positions command.

use predicates::prelude::*;

use super::helpers::{mock_graphql_response, morpho_cmd_with_mock, start_mock_server};

#[tokio::test]
async fn test_positions_success() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "positions").await;

    morpho_cmd_with_mock(&server)
        .args([
            "positions",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            "--chain",
            "ethereum",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Steakhouse USDC"));
}

#[tokio::test]
async fn test_positions_json_output() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "positions").await;

    morpho_cmd_with_mock(&server)
        .args([
            "positions",
            "0xd8dA6BF26964aF9D7eEd9e03E53415D37aA96045",
            "--chain",
            "ethereum",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("vault_positions"))
        .stdout(predicate::str::contains("vault_v2_positions"));
}

#[tokio::test]
async fn test_positions_empty() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "positions_empty").await;

    morpho_cmd_with_mock(&server)
        .args([
            "positions",
            "0x0000000000000000000000000000000000000001",
            "--chain",
            "ethereum",
        ])
        .assert()
        .success()
        // Empty positions should still succeed, just show empty results
        .stdout(predicate::str::contains("No positions"));
}

#[tokio::test]
async fn test_positions_json_empty() {
    let server = start_mock_server().await;
    mock_graphql_response(&server, "positions_empty").await;

    morpho_cmd_with_mock(&server)
        .args([
            "positions",
            "0x0000000000000000000000000000000000000001",
            "--chain",
            "ethereum",
            "--format",
            "json",
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"vault_positions\": []"))
        .stdout(predicate::str::contains("\"vault_v2_positions\": []"));
}
