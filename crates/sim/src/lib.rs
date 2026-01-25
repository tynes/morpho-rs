//! # Morpho Blue Simulation SDK
//!
//! A comprehensive simulation library for [Morpho Blue](https://morpho.org/) lending markets
//! and [MetaMorpho](https://docs.morpho.org/metamorpho/overview) vaults.
//!
//! This crate enables offline APY calculations, position health tracking, vault deposit/withdrawal
//! simulations, and yield optimization strategies without requiring on-chain transactions.
//!
//! ## Features
//!
//! - **Market Simulation**: Supply, borrow, withdraw, and repay operations with accurate interest accrual
//! - **APY Calculations**: Calculate supply/borrow APYs using the Adaptive Curve IRM
//! - **Vault Operations**: Simulate MetaMorpho vault deposits, withdrawals, and reallocations
//! - **Position Tracking**: Monitor health factors, LTV, liquidation prices, and capacity limits
//! - **Yield Optimization**: Find optimal market allocations and best vaults for deposits
//! - **Public Allocator**: Simulate public reallocation with flow limits
//!
//! ## Quick Start
//!
//! ### Market APY Calculation
//!
//! ```rust
//! use morpho_rs_sim::{Market, WAD};
//! use alloy_primitives::{FixedBytes, U256};
//!
//! // Create a market with 80% utilization
//! let market = Market::new(
//!     FixedBytes::ZERO,                           // market ID
//!     U256::from(1_000_000) * WAD,                // total supply: 1M
//!     U256::from(800_000) * WAD,                  // total borrow: 800K
//!     U256::from(1_000_000) * WAD,                // supply shares
//!     U256::from(800_000) * WAD,                  // borrow shares
//!     1704067200,                                 // last update timestamp
//!     U256::from(100_000_000_000_000_000u64),     // 10% protocol fee
//!     Some(U256::from(1_268_391_679u64)),         // rate at target (~4% APY)
//! );
//!
//! let timestamp = 1704153600;
//! let supply_apy = market.get_supply_apy(timestamp).unwrap();
//! let borrow_apy = market.get_borrow_apy(timestamp).unwrap();
//!
//! assert!(supply_apy > 0.0);
//! assert!(borrow_apy > supply_apy); // Borrow APY is always higher
//! ```
//!
//! ### Simulating Operations
//!
//! ```rust
//! use morpho_rs_sim::{supply_apy_impact, Market, WAD};
//! use alloy_primitives::{FixedBytes, U256};
//!
//! let market = Market::new(
//!     FixedBytes::ZERO,
//!     U256::from(1_000_000) * WAD,
//!     U256::from(800_000) * WAD,
//!     U256::from(1_000_000) * WAD,
//!     U256::from(800_000) * WAD,
//!     1704067200,
//!     U256::from(100_000_000_000_000_000u64),
//!     Some(U256::from(1_268_391_679u64)),
//! );
//!
//! let deposit = U256::from(100_000) * WAD;
//! let impact = supply_apy_impact(&market, deposit, 1704067200).unwrap();
//!
//! // Supplying dilutes returns, so APY decreases
//! assert!(impact.apy_delta < 0.0);
//! assert!(impact.shares_received > U256::ZERO);
//! ```
//!
//! ## Core Concepts
//!
//! ### WAD-Scaled Arithmetic
//!
//! All monetary values use fixed-point arithmetic with 18 decimal places:
//!
//! ```rust
//! use morpho_rs_sim::WAD;
//! use alloy_primitives::U256;
//!
//! let one_token = WAD;                      // 1.0
//! let half_token = WAD / U256::from(2);     // 0.5
//! let ten_percent = WAD / U256::from(10);   // 0.1 (10%)
//! ```
//!
//! ### Adaptive Curve IRM
//!
//! The Interest Rate Model adjusts rates based on utilization:
//! - **Target Utilization**: 90%
//! - **Below Target**: Lower rates to encourage borrowing
//! - **Above Target**: Higher rates to encourage supply and discourage borrowing
//! - **Rate Adaptation**: The "rate at target" adjusts over time based on utilization history
//!
//! ## Modules
//!
//! - [`market`]: Market state and operations (supply, borrow, APY calculations)
//! - [`vault`]: MetaMorpho vault simulation (deposits, withdrawals, reallocations)
//! - [`position`]: Position tracking with health factor and liquidation metrics
//! - [`irm`]: Adaptive Curve Interest Rate Model implementation
//! - [`math`]: Fixed-point arithmetic utilities
//! - [`error`]: Error types for simulation operations

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
