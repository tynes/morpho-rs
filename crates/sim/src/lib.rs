//! Morpho Vault Simulation SDK
//!
//! This crate provides simulation capabilities for Morpho Blue markets and
//! MetaMorpho vaults, including APY calculations and deposit impact analysis.
//!
//! # Overview
//!
//! The simulation SDK allows you to:
//! - Calculate market-level supply and borrow APYs
//! - Simulate vault deposits and withdrawals
//! - Calculate the APY impact of operations
//! - Find the deposit amount needed for a target APY change
//! - Track and manage positions with health factor calculations
//! - Simulate reallocations across markets
//! - Find optimal allocations for yield maximization
//!
//! # Example
//!
//! ```rust,ignore
//! use morpho_rs_sim::{
//!     vault::{Vault, VaultMarketConfig, VaultSimulation, vault_deposit_apy_impact},
//!     market::Market,
//! };
//! use alloy_primitives::U256;
//! use std::collections::HashMap;
//!
//! // Create markets and vault configuration
//! // ...
//!
//! // Calculate APY impact of a deposit
//! let deposit_amount = U256::from(100_000) * morpho_rs_sim::math::WAD;
//! let impact = vault_deposit_apy_impact(&simulation, deposit_amount, timestamp)?;
//!
//! println!("APY before: {:.2}%", impact.apy_before * 100.0);
//! println!("APY after: {:.2}%", impact.apy_after * 100.0);
//! println!("APY change: {:.4}%", impact.apy_delta * 100.0);
//! ```

pub mod error;
pub mod irm;
pub mod market;
pub mod math;
pub mod position;
pub mod vault;

// Re-export commonly used types
pub use error::{MarketId, SimError};

// Market exports
pub use market::{
    borrow_apy_impact, find_best_market_for_supply, get_liquidation_incentive_factor,
    get_utilization, rank_markets_by_borrow_apy, rank_markets_by_supply_apy, supply_apy_impact,
    BorrowApyImpact, Market, MarketRanking, SupplyApyImpact,
    LIQUIDATION_CURSOR, MAX_LIQUIDATION_INCENTIVE_FACTOR, ORACLE_PRICE_SCALE,
};

// Math exports
pub use math::{RoundingDirection, SECONDS_PER_YEAR, WAD};

// Position exports
pub use position::{CapacityLimit, CapacityLimitReason, Position, PositionCapacities};

// Vault exports
pub use vault::{
    amount_for_vault_apy_impact, find_best_vault_for_deposit, find_optimal_market_allocation,
    rank_vaults_by_apy, vault_deposit_apy_impact, vault_withdraw_apy_impact,
    OptimalAllocation, PublicAllocatorConfig, PublicAllocatorMarketConfig,
    ReallocationStep, Vault, VaultApyImpact, VaultMarketConfig, VaultRanking, VaultSimulation,
};

// IRM exports
pub use irm::{
    get_borrow_rate, get_supply_for_borrow_rate, get_utilization_at_borrow_rate, w_exp,
    BorrowRateResult, ADJUSTMENT_SPEED, CURVE_STEEPNESS, INITIAL_RATE_AT_TARGET,
    MAX_RATE_AT_TARGET, MIN_RATE_AT_TARGET, TARGET_UTILIZATION,
};
