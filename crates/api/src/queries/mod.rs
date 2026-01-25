//! GraphQL query definitions.

pub mod simulation;
pub mod user;
pub mod v1;
pub mod v2;
pub mod v2_simulation;

pub use simulation::{GetVaultForSimulation, GetVaultsForSimulation};
pub use user::{GetUserAccountOverview, GetUserVaultPositions};
pub use v1::{GetVaultV1ByAddress, GetVaultsV1};
pub use v2::{GetVaultV2ByAddress, GetVaultsV2};
pub use v2_simulation::{GetVaultV2ForSimulation, GetVaultsV2ForSimulation};
