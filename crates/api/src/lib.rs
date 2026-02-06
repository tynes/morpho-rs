//! Morpho Vaults Rust API Library
//!
//! This crate provides a Rust client for querying Morpho V1 (MetaMorpho) and V2 vaults
//! via their GraphQL API, and executing on-chain transactions.
//!
//! # Example
//!
//! ```no_run
//! use morpho_rs_api::{MorphoClient, MorphoClientConfig, NamedChain};
//! use alloy::primitives::{Address, U256};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), morpho_rs_api::ApiError> {
//!     // API-only client (no transactions)
//!     let client = MorphoClient::new();
//!     let vaults = client.get_vaults_by_chain(NamedChain::Mainnet).await?;
//!
//!     // Full client with transaction support
//!     let config = MorphoClientConfig::new()
//!         .with_rpc_url("https://eth.llamarpc.com")
//!         .with_private_key("0x...");
//!     let client = MorphoClient::with_config(config)?;
//!
//!     // V1 vault operations using bound signer address
//!     let vault: Address = "0x...".parse().unwrap();
//!     let balance = client.vault_v1()?.balance(vault).await?;
//!
//!     // Approve and deposit
//!     let amount = U256::from(1000000);
//!     client.vault_v1()?.approve(vault, amount).await?;
//!     client.vault_v1()?.deposit(vault, amount).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Feature Flags
//!
//! - **`sim`** — Enables simulation support via the `morpho-rs-sim` crate. When enabled,
//!   [`VaultV1`] and [`VaultV2`] gain a [`to_vault_simulation()`](VaultV1::to_vault_simulation)
//!   method that converts API response data into a [`morpho_rs_sim::VaultSimulation`] for
//!   computing projected APY and simulating deposits/withdrawals. Also adds the
//!   [`ApiError::Simulation`] variant for propagating simulation errors.

pub mod client;
pub mod error;
pub mod filters;
pub mod queries;
pub mod types;

// Re-export main types at crate root
pub use client::{
    ClientConfig, MorphoApiClient, MorphoClient, MorphoClientConfig, VaultV1Client,
    VaultV1Operations, VaultV2Client, VaultV2Operations, DEFAULT_API_URL,
};
pub use error::{ApiError, Result};
pub use filters::{VaultFiltersV1, VaultFiltersV2, VaultQueryOptionsV1, VaultQueryOptionsV2};
pub use morpho_rs_contracts::{Erc4626Client, VaultV1TransactionClient, VaultV2TransactionClient};
pub use types::{
    chain_from_id, chain_serde, Asset, MarketInfo, MarketStateV1, MarketStateV2,
    MetaMorphoAllocation, MorphoMarketPosition, NamedChain, OrderDirection, UserAccountOverview,
    UserMarketPosition, UserState, UserVaultPositions, UserVaultV1Position, UserVaultV2Position,
    Vault, VaultAdapter, VaultAdapterData, VaultAllocation, VaultAllocator, VaultInfo,
    VaultOrderByV1, VaultOrderByV2, VaultPositionState, VaultReward, VaultStateV1, VaultV1,
    VaultV2, VaultV2Warning, VaultVersion, VaultWarning, SUPPORTED_CHAINS,
};
