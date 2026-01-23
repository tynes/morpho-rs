//! Integration tests for the contracts crate.

use contracts::{ContractError, VaultV1TransactionClient, VaultV2TransactionClient};

#[test]
fn test_v1_client_construction_with_valid_inputs() {
    let private_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let result = VaultV1TransactionClient::new("http://localhost:8545", private_key);
    assert!(result.is_ok());

    let client = result.unwrap();
    // Verify signer address is derived correctly
    assert!(!client.signer_address().is_zero());
}

#[test]
fn test_v2_client_construction_with_valid_inputs() {
    let private_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let result = VaultV2TransactionClient::new("http://localhost:8545", private_key);
    assert!(result.is_ok());

    let client = result.unwrap();
    // Verify signer address is derived correctly
    assert!(!client.signer_address().is_zero());
}

#[test]
fn test_v1_client_invalid_private_key() {
    let result = VaultV1TransactionClient::new("http://localhost:8545", "not-a-valid-key");
    assert!(matches!(result, Err(ContractError::InvalidPrivateKey)));
}

#[test]
fn test_v2_client_invalid_private_key() {
    let result = VaultV2TransactionClient::new("http://localhost:8545", "not-a-valid-key");
    assert!(matches!(result, Err(ContractError::InvalidPrivateKey)));
}

#[test]
fn test_v1_client_invalid_rpc_url() {
    let private_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let result = VaultV1TransactionClient::new("not a url", private_key);
    assert!(matches!(result, Err(ContractError::RpcConnection(_))));
}

#[test]
fn test_v2_client_invalid_rpc_url() {
    let private_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let result = VaultV2TransactionClient::new("not a url", private_key);
    assert!(matches!(result, Err(ContractError::RpcConnection(_))));
}

#[test]
fn test_error_conversion() {
    // Test that errors can be used with ? operator
    fn fallible() -> contracts::Result<()> {
        let _client = VaultV1TransactionClient::new("http://localhost:8545", "invalid")?;
        Ok(())
    }

    let result = fallible();
    assert!(result.is_err());
}

#[test]
fn test_v1_and_v2_clients_are_separate() {
    // Ensure both clients can be constructed independently
    let private_key = "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    let v1 = VaultV1TransactionClient::new("http://localhost:8545", private_key).unwrap();
    let v2 = VaultV2TransactionClient::new("http://localhost:8545", private_key).unwrap();

    // Both should derive the same signer address from the same key
    assert_eq!(v1.signer_address(), v2.signer_address());
}
