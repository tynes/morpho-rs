//! Transaction module for interacting with Morpho vaults on-chain.
//!
//! This module provides clients for executing transactions against ERC4626 vaults.

pub mod erc20;
pub mod erc4626;
pub mod vault_v1;
pub mod vault_v2;

pub use vault_v1::VaultV1TransactionClient;
pub use vault_v2::VaultV2TransactionClient;
