//! CLI argument validation tests.
//!
//! These tests verify that the CLI properly validates arguments and provides
//! helpful error messages without requiring network access.

use predicates::prelude::*;

use super::helpers::morpho_cmd;

#[test]
fn test_help_output() {
    morpho_cmd()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("morpho"))
        .stdout(predicate::str::contains("vaultv1"))
        .stdout(predicate::str::contains("vaultv2"))
        .stdout(predicate::str::contains("positions"));
}

#[test]
fn test_vaultv1_help_output() {
    morpho_cmd()
        .args(["vaultv1", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("info"));
}

#[test]
fn test_vaultv2_help_output() {
    morpho_cmd()
        .args(["vaultv2", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("info"));
}

#[test]
fn test_invalid_command() {
    morpho_cmd()
        .arg("invalid_command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_vaultv1_info_missing_address() {
    morpho_cmd()
        .args(["vaultv1", "info"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_vaultv2_info_missing_address() {
    morpho_cmd()
        .args(["vaultv2", "info"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_positions_missing_address() {
    morpho_cmd()
        .args(["positions"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_invalid_chain_value() {
    morpho_cmd()
        .args(["vaultv1", "list", "--chain", "invalid_chain"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_invalid_output_format() {
    morpho_cmd()
        .args(["vaultv1", "list", "--format", "invalid_format"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("error"));
}

#[test]
fn test_vaultv1_list_help() {
    morpho_cmd()
        .args(["vaultv1", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--chain"))
        .stdout(predicate::str::contains("--limit"));
}

#[test]
fn test_vaultv2_list_help() {
    morpho_cmd()
        .args(["vaultv2", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--chain"))
        .stdout(predicate::str::contains("--limit"));
}

#[test]
fn test_positions_help() {
    morpho_cmd()
        .args(["positions", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--chain"))
        .stdout(predicate::str::contains("ADDRESS"));
}
