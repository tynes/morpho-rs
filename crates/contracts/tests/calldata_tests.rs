//! Unit tests for calldata encoding.
//!
//! These tests verify correct ABI encoding of ERC-4626 and ERC-20 function
//! calldata without requiring RPC connections.

use alloy::primitives::{address, keccak256, Address, U256};
use alloy::signers::local::PrivateKeySigner;
use alloy::sol_types::SolCall;
use morpho_rs_contracts::{Erc4626Client, VaultV1TransactionClient, VaultV2TransactionClient};

// Anvil's default account 0 private key
const TEST_PRIVATE_KEY: &str = "0xac0974bec39a17e36ba4a6b4d238ff944bacb478cbed5efcae784d7bf4f2ff80";
// Expected address for the test private key
const EXPECTED_SIGNER_ADDRESS: Address = address!("f39Fd6e51aad88F6F4ce6aB8827279cffFb92266");

// Test vault and token addresses
const TEST_VAULT: Address = address!("BEEF01735c132Ada46AA9aA4c54623cAA92A64CB");
const TEST_TOKEN: Address = address!("A0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48");
const TEST_RECEIVER: Address = address!("1234567890123456789012345678901234567890");
const TEST_OWNER: Address = address!("abcdabcdabcdabcdabcdabcdabcdabcdabcdabcd");

// ERC-4626 function selectors (first 4 bytes of keccak256 hash)
// deposit(uint256,address)
const DEPOSIT_SELECTOR: [u8; 4] = [0x6e, 0x55, 0x3f, 0x65];
// withdraw(uint256,address,address)
const WITHDRAW_SELECTOR: [u8; 4] = [0xb4, 0x60, 0xaf, 0x94];
// mint(uint256,address)
const MINT_SELECTOR: [u8; 4] = [0x94, 0xbf, 0x80, 0x4d];
// redeem(uint256,address,address)
const REDEEM_SELECTOR: [u8; 4] = [0xba, 0x08, 0x76, 0x52];

// ERC-20 function selectors
// approve(address,uint256)
const APPROVE_SELECTOR: [u8; 4] = [0x09, 0x5e, 0xa7, 0xb3];

/// Helper to create a V1 client for testing
fn create_v1_client() -> VaultV1TransactionClient {
    VaultV1TransactionClient::new("http://localhost:8545", TEST_PRIVATE_KEY)
        .expect("Failed to create V1 client")
}

/// Helper to create a V2 client for testing
fn create_v2_client() -> VaultV2TransactionClient {
    VaultV2TransactionClient::new("http://localhost:8545", TEST_PRIVATE_KEY)
        .expect("Failed to create V2 client")
}

// ============================================================================
// ERC-4626 Deposit Calldata Tests
// ============================================================================

#[test]
fn test_deposit_calldata_selector() {
    let client = create_v1_client();
    let amount = U256::from(1_000_000u64);

    let prepared = client.deposit(TEST_VAULT, amount, TEST_RECEIVER);
    let (addr, call) = prepared.prepare();

    assert_eq!(addr, TEST_VAULT);

    // Verify the selector
    let calldata = call.abi_encode();
    assert_eq!(&calldata[0..4], &DEPOSIT_SELECTOR);
}

#[test]
fn test_deposit_calldata_encoding() {
    let client = create_v1_client();
    let amount = U256::from(1_000_000u64);

    let prepared = client.deposit(TEST_VAULT, amount, TEST_RECEIVER);
    let (_, call) = prepared.prepare();
    let calldata = call.abi_encode();

    // Calldata should be 4 (selector) + 32 (amount) + 32 (receiver) = 68 bytes
    assert_eq!(calldata.len(), 68);

    // Decode amount from bytes 4-36 (32 bytes, big-endian)
    let decoded_amount = U256::from_be_slice(&calldata[4..36]);
    assert_eq!(decoded_amount, amount);

    // Decode receiver from bytes 36-68 (32 bytes, right-padded address)
    let decoded_receiver = Address::from_slice(&calldata[48..68]);
    assert_eq!(decoded_receiver, TEST_RECEIVER);
}

#[test]
fn test_deposit_large_amount() {
    let client = create_v1_client();
    // Test with max uint256 / 2 to ensure large values encode correctly
    let amount = U256::MAX / U256::from(2);

    let prepared = client.deposit(TEST_VAULT, amount, TEST_RECEIVER);
    let (_, call) = prepared.prepare();
    let calldata = call.abi_encode();

    let decoded_amount = U256::from_be_slice(&calldata[4..36]);
    assert_eq!(decoded_amount, amount);
}

// ============================================================================
// ERC-4626 Withdraw Calldata Tests
// ============================================================================

#[test]
fn test_withdraw_calldata_selector() {
    let client = create_v1_client();
    let amount = U256::from(500_000u64);

    let prepared = client.withdraw(TEST_VAULT, amount, TEST_RECEIVER, TEST_OWNER);
    let (addr, call) = prepared.prepare();

    assert_eq!(addr, TEST_VAULT);

    let calldata = call.abi_encode();
    assert_eq!(&calldata[0..4], &WITHDRAW_SELECTOR);
}

#[test]
fn test_withdraw_calldata_encoding() {
    let client = create_v1_client();
    let amount = U256::from(500_000u64);

    let prepared = client.withdraw(TEST_VAULT, amount, TEST_RECEIVER, TEST_OWNER);
    let (_, call) = prepared.prepare();
    let calldata = call.abi_encode();

    // Calldata should be 4 (selector) + 32 (amount) + 32 (receiver) + 32 (owner) = 100 bytes
    assert_eq!(calldata.len(), 100);

    // Decode amount
    let decoded_amount = U256::from_be_slice(&calldata[4..36]);
    assert_eq!(decoded_amount, amount);

    // Decode receiver
    let decoded_receiver = Address::from_slice(&calldata[48..68]);
    assert_eq!(decoded_receiver, TEST_RECEIVER);

    // Decode owner
    let decoded_owner = Address::from_slice(&calldata[80..100]);
    assert_eq!(decoded_owner, TEST_OWNER);
}

// ============================================================================
// ERC-4626 Mint Calldata Tests
// ============================================================================

#[test]
fn test_mint_calldata_selector() {
    let client = create_v1_client();
    let shares = U256::from(750_000u64);

    let prepared = client.mint(TEST_VAULT, shares, TEST_RECEIVER);
    let (addr, call) = prepared.prepare();

    assert_eq!(addr, TEST_VAULT);

    let calldata = call.abi_encode();
    assert_eq!(&calldata[0..4], &MINT_SELECTOR);
}

#[test]
fn test_mint_calldata_encoding() {
    let client = create_v1_client();
    let shares = U256::from(750_000u64);

    let prepared = client.mint(TEST_VAULT, shares, TEST_RECEIVER);
    let (_, call) = prepared.prepare();
    let calldata = call.abi_encode();

    // Calldata should be 4 (selector) + 32 (shares) + 32 (receiver) = 68 bytes
    assert_eq!(calldata.len(), 68);

    // Decode shares
    let decoded_shares = U256::from_be_slice(&calldata[4..36]);
    assert_eq!(decoded_shares, shares);

    // Decode receiver
    let decoded_receiver = Address::from_slice(&calldata[48..68]);
    assert_eq!(decoded_receiver, TEST_RECEIVER);
}

// ============================================================================
// ERC-4626 Redeem Calldata Tests
// ============================================================================

#[test]
fn test_redeem_calldata_selector() {
    let client = create_v1_client();
    let shares = U256::from(250_000u64);

    let prepared = client.redeem(TEST_VAULT, shares, TEST_RECEIVER, TEST_OWNER);
    let (addr, call) = prepared.prepare();

    assert_eq!(addr, TEST_VAULT);

    let calldata = call.abi_encode();
    assert_eq!(&calldata[0..4], &REDEEM_SELECTOR);
}

#[test]
fn test_redeem_calldata_encoding() {
    let client = create_v1_client();
    let shares = U256::from(250_000u64);

    let prepared = client.redeem(TEST_VAULT, shares, TEST_RECEIVER, TEST_OWNER);
    let (_, call) = prepared.prepare();
    let calldata = call.abi_encode();

    // Calldata should be 4 (selector) + 32 (shares) + 32 (receiver) + 32 (owner) = 100 bytes
    assert_eq!(calldata.len(), 100);

    // Decode shares
    let decoded_shares = U256::from_be_slice(&calldata[4..36]);
    assert_eq!(decoded_shares, shares);

    // Decode receiver
    let decoded_receiver = Address::from_slice(&calldata[48..68]);
    assert_eq!(decoded_receiver, TEST_RECEIVER);

    // Decode owner
    let decoded_owner = Address::from_slice(&calldata[80..100]);
    assert_eq!(decoded_owner, TEST_OWNER);
}

// ============================================================================
// ERC-20 Approve Calldata Tests
// ============================================================================

#[test]
fn test_approve_calldata_selector() {
    let client = create_v1_client();
    let amount = U256::from(1_000_000u64);

    let prepared = client.approve(TEST_TOKEN, TEST_VAULT, amount);
    let (addr, call) = prepared.prepare();

    assert_eq!(addr, TEST_TOKEN);

    let calldata = call.abi_encode();
    assert_eq!(&calldata[0..4], &APPROVE_SELECTOR);
}

#[test]
fn test_approve_calldata_encoding() {
    let client = create_v1_client();
    let amount = U256::from(1_000_000u64);

    let prepared = client.approve(TEST_TOKEN, TEST_VAULT, amount);
    let (_, call) = prepared.prepare();
    let calldata = call.abi_encode();

    // Calldata should be 4 (selector) + 32 (spender) + 32 (amount) = 68 bytes
    assert_eq!(calldata.len(), 68);

    // Decode spender (vault address)
    let decoded_spender = Address::from_slice(&calldata[16..36]);
    assert_eq!(decoded_spender, TEST_VAULT);

    // Decode amount
    let decoded_amount = U256::from_be_slice(&calldata[36..68]);
    assert_eq!(decoded_amount, amount);
}

#[test]
fn test_approve_max_amount() {
    let client = create_v1_client();

    let prepared = client.approve(TEST_TOKEN, TEST_VAULT, U256::MAX);
    let (_, call) = prepared.prepare();
    let calldata = call.abi_encode();

    let decoded_amount = U256::from_be_slice(&calldata[36..68]);
    assert_eq!(decoded_amount, U256::MAX);
}

// ============================================================================
// Signer Address Derivation Tests
// ============================================================================

#[test]
fn test_signer_address_derivation_v1() {
    let client = create_v1_client();
    assert_eq!(client.signer_address(), EXPECTED_SIGNER_ADDRESS);
}

#[test]
fn test_signer_address_derivation_v2() {
    let client = create_v2_client();
    assert_eq!(client.signer_address(), EXPECTED_SIGNER_ADDRESS);
}

#[test]
fn test_signer_address_derivation_direct() {
    // Verify using direct alloy signer derivation
    let signer: PrivateKeySigner = TEST_PRIVATE_KEY
        .parse()
        .expect("Failed to parse private key");
    assert_eq!(signer.address(), EXPECTED_SIGNER_ADDRESS);
}

#[test]
fn test_different_private_key_produces_different_address() {
    // Different private key should produce different address
    let different_key = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
    let client = VaultV1TransactionClient::new("http://localhost:8545", different_key)
        .expect("Failed to create client");

    // This should NOT be the same as our expected address
    assert_ne!(client.signer_address(), EXPECTED_SIGNER_ADDRESS);
}

// ============================================================================
// Function Selector Verification Tests
// ============================================================================

#[test]
fn test_deposit_selector_matches_keccak() {
    // Verify our hardcoded selector matches keccak256("deposit(uint256,address)")
    let hash = keccak256("deposit(uint256,address)");
    assert_eq!(&hash[0..4], &DEPOSIT_SELECTOR);
}

#[test]
fn test_withdraw_selector_matches_keccak() {
    let hash = keccak256("withdraw(uint256,address,address)");
    assert_eq!(&hash[0..4], &WITHDRAW_SELECTOR);
}

#[test]
fn test_mint_selector_matches_keccak() {
    let hash = keccak256("mint(uint256,address)");
    assert_eq!(&hash[0..4], &MINT_SELECTOR);
}

#[test]
fn test_redeem_selector_matches_keccak() {
    let hash = keccak256("redeem(uint256,address,address)");
    assert_eq!(&hash[0..4], &REDEEM_SELECTOR);
}

#[test]
fn test_approve_selector_matches_keccak() {
    let hash = keccak256("approve(address,uint256)");
    assert_eq!(&hash[0..4], &APPROVE_SELECTOR);
}

// ============================================================================
// V2 Client Calldata Tests (verify same encoding as V1)
// ============================================================================

#[test]
fn test_v2_deposit_same_encoding_as_v1() {
    let v1_client = create_v1_client();
    let v2_client = create_v2_client();
    let amount = U256::from(1_000_000u64);

    let v1_prepared = v1_client.deposit(TEST_VAULT, amount, TEST_RECEIVER);
    let v2_prepared = v2_client.deposit(TEST_VAULT, amount, TEST_RECEIVER);

    let (_, v1_call) = v1_prepared.prepare();
    let (_, v2_call) = v2_prepared.prepare();

    assert_eq!(v1_call.abi_encode(), v2_call.abi_encode());
}

#[test]
fn test_v2_withdraw_same_encoding_as_v1() {
    let v1_client = create_v1_client();
    let v2_client = create_v2_client();
    let amount = U256::from(500_000u64);

    let v1_prepared = v1_client.withdraw(TEST_VAULT, amount, TEST_RECEIVER, TEST_OWNER);
    let v2_prepared = v2_client.withdraw(TEST_VAULT, amount, TEST_RECEIVER, TEST_OWNER);

    let (_, v1_call) = v1_prepared.prepare();
    let (_, v2_call) = v2_prepared.prepare();

    assert_eq!(v1_call.abi_encode(), v2_call.abi_encode());
}

#[test]
fn test_v2_mint_same_encoding_as_v1() {
    let v1_client = create_v1_client();
    let v2_client = create_v2_client();
    let shares = U256::from(750_000u64);

    let v1_prepared = v1_client.mint(TEST_VAULT, shares, TEST_RECEIVER);
    let v2_prepared = v2_client.mint(TEST_VAULT, shares, TEST_RECEIVER);

    let (_, v1_call) = v1_prepared.prepare();
    let (_, v2_call) = v2_prepared.prepare();

    assert_eq!(v1_call.abi_encode(), v2_call.abi_encode());
}

#[test]
fn test_v2_redeem_same_encoding_as_v1() {
    let v1_client = create_v1_client();
    let v2_client = create_v2_client();
    let shares = U256::from(250_000u64);

    let v1_prepared = v1_client.redeem(TEST_VAULT, shares, TEST_RECEIVER, TEST_OWNER);
    let v2_prepared = v2_client.redeem(TEST_VAULT, shares, TEST_RECEIVER, TEST_OWNER);

    let (_, v1_call) = v1_prepared.prepare();
    let (_, v2_call) = v2_prepared.prepare();

    assert_eq!(v1_call.abi_encode(), v2_call.abi_encode());
}

#[test]
fn test_v2_approve_same_encoding_as_v1() {
    let v1_client = create_v1_client();
    let v2_client = create_v2_client();
    let amount = U256::from(1_000_000u64);

    let v1_prepared = v1_client.approve(TEST_TOKEN, TEST_VAULT, amount);
    let v2_prepared = v2_client.approve(TEST_TOKEN, TEST_VAULT, amount);

    let (_, v1_call) = v1_prepared.prepare();
    let (_, v2_call) = v2_prepared.prepare();

    assert_eq!(v1_call.abi_encode(), v2_call.abi_encode());
}
