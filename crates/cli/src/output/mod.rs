//! Output formatting for CLI results.

pub mod detail;
pub mod positions;
pub mod table;

pub use detail::{format_v1_vault_detail, format_v2_vault_detail};
pub use positions::format_user_positions;
pub use table::{format_v1_vaults_table, format_v2_vaults_table};
