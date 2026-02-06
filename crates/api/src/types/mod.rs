//! Type definitions for the Morpho API.
//!
//! # GraphQL Conversion (`from_gql`)
//!
//! Each type that comes from the Morpho GraphQL API provides a `from_gql()` constructor
//! that converts raw GraphQL scalar values (strings, floats) into strongly-typed Rust types:
//!
//! - **Hex address strings** (`"0x..."`) are parsed into [`Address`](alloy_primitives::Address)
//! - **Bigint strings** (`"1000000000000000000"`) are parsed into [`U256`](alloy_primitives::U256)
//! - **Chain IDs** (`i64`) are converted to [`NamedChain`]
//!
//! Most `from_gql()` methods return `Option<Self>`, returning `None` when a required field
//! cannot be parsed (e.g., invalid hex address). The exceptions are [`MarketInfo::from_gql`]
//! and [`UserState::from_gql`], which always succeed because all their fields are either
//! optional or plain numeric types.
//!
//! # Simulation Conversion (`to_vault_simulation`)
//!
//! When the `sim` feature is enabled, [`VaultV1`] and [`VaultV2`] gain a
//! `to_vault_simulation()` method that converts API data into a `VaultSimulation` for
//! offline APY calculations and deposit/withdrawal simulations. This method builds
//! supply/withdraw queues from the allocation data and converts the fee from the API's
//! fractional format to WAD-scaled.

pub mod asset;
pub mod chain;
pub mod ordering;
pub mod scalars;
pub mod user;
pub mod vault;
pub mod vault_v1;
pub mod vault_v2;

pub use alloy_chains::NamedChain;
pub use asset::Asset;
pub use chain::{chain_from_id, chain_serde, SUPPORTED_CHAINS};
pub use ordering::{OrderDirection, VaultOrderByV1, VaultOrderByV2};
pub use user::{
    MarketInfo, UserAccountOverview, UserMarketPosition, UserState, UserVaultPositions,
    UserVaultV1Position, UserVaultV2Position, VaultInfo, VaultPositionState,
};
pub use vault::{Vault, VaultVersion};
pub use vault_v1::{MarketStateV1, VaultAllocation, VaultAllocator, VaultStateV1, VaultV1, VaultWarning};
pub use vault_v2::{
    MarketStateV2, MetaMorphoAllocation, MorphoMarketPosition, VaultAdapter, VaultAdapterData,
    VaultReward, VaultV2, VaultV2Warning,
};
