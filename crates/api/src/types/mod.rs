//! Type definitions for the Morpho API.

pub mod asset;
pub mod chain;
pub mod scalars;
pub mod user;
pub mod vault;
pub mod vault_v1;
pub mod vault_v2;

pub use asset::Asset;
pub use chain::Chain;
pub use user::{
    MarketInfo, UserAccountOverview, UserMarketPosition, UserState, UserVaultPositions,
    UserVaultV1Position, UserVaultV2Position, VaultInfo, VaultPositionState,
};
pub use vault::{Vault, VaultVersion};
pub use vault_v1::{VaultAllocation, VaultAllocator, VaultStateV1, VaultV1, VaultWarning};
pub use vault_v2::{VaultAdapter, VaultReward, VaultV2, VaultV2Warning};
