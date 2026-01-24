//! Contract bindings and transaction clients for Morpho vaults.
//!
//! This crate provides Solidity contract bindings and transaction clients
//! for interacting with Morpho V1 (MetaMorpho) and V2 vaults on-chain.
//!
//! # Example
//!
//! ```no_run
//! use morpho_rs_contracts::{VaultV1TransactionClient, VaultV2TransactionClient};
//! use alloy::primitives::{Address, U256};
//!
//! #[tokio::main]
//! async fn main() -> morpho_rs_contracts::Result<()> {
//!     let client = VaultV1TransactionClient::new(
//!         "https://eth.llamarpc.com",
//!         "0x...", // private key
//!     )?;
//!
//!     // Get vault asset
//!     let vault: Address = "0x...".parse().unwrap();
//!     let asset = client.get_asset(vault).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod erc20;
pub mod erc4626;
pub mod error;
pub mod prepared_call;
pub mod provider;
pub mod vault_v1;
pub mod vault_v2;

pub use error::{ContractError, Result};
pub use prepared_call::PreparedCall;
pub use provider::HttpProvider;
pub use vault_v1::VaultV1TransactionClient;
pub use vault_v2::VaultV2TransactionClient;
