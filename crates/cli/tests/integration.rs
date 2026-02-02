//! Integration tests for the Morpho CLI.
//!
//! These tests verify the full command execution path with mocked API responses.
//!
//! # Test Categories
//!
//! - **Query command tests**: V1/V2 list, info, positions with wiremock
//! - **CLI validation tests**: Argument parsing, help text, error handling
//! - **E2E transaction tests**: Deposit/withdraw with Anvil fork (ignored by default)
//!
//! # Running Tests
//!
//! ```bash
//! # All integration tests (except ignored)
//! cargo test -p morpho-rs-cli --test integration
//!
//! # E2E transaction tests (requires ETH_RPC_URL)
//! ETH_RPC_URL="https://eth.llamarpc.com" cargo test -p morpho-rs-cli --test integration -- --ignored
//! ```

mod integration {
    pub mod helpers;
    pub mod vault_v1_tests;
    pub mod vault_v2_tests;
    pub mod positions_tests;
    pub mod cli_validation_tests;
    pub mod e2e_transaction;
}
