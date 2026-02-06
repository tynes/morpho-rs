//! Error types for the simulation library.
//!
//! This module defines the [`SimError`] enum which covers all error conditions
//! that can occur during market, vault, and position simulations.
//!
//! # Error Categories
//!
//! ## Interest Accrual Errors
//! - [`SimError::InvalidInterestAccrual`]: Timestamp is before last update
//!
//! ## Capacity Errors
//! - [`SimError::AllCapsReached`]: Vault deposit exceeds market caps
//! - [`SimError::NotEnoughLiquidity`]: Insufficient liquidity for withdrawal
//! - [`SimError::InsufficientMarketLiquidity`]: Market lacks liquidity for borrow/withdraw
//! - [`SimError::SupplyCapExceeded`]: Reallocation exceeds market cap
//!
//! ## Position Errors
//! - [`SimError::InsufficientPosition`]: Trying to withdraw more than owned
//! - [`SimError::InsufficientCollateral`]: Borrow would make position unhealthy
//! - [`SimError::UnknownOraclePrice`]: No oracle price for health calculations
//!
//! ## Vault Errors
//! - [`SimError::MarketNotFound`]: Market not in vault allocations
//! - [`SimError::MarketNotEnabled`]: Market disabled for operations
//! - [`SimError::InconsistentReallocation`]: Supply/withdraw mismatch in reallocation
//!
//! ## Public Allocator Errors
//! - [`SimError::PublicAllocatorNotConfigured`]: Vault has no public allocator
//! - [`SimError::MaxInflowExceeded`]: Exceeds market's max_in limit
//! - [`SimError::MaxOutflowExceeded`]: Exceeds market's max_out limit
//!
//! # Example
//!
//! ```rust
//! use morpho_rs_sim::{Market, SimError, WAD};
//! use alloy_primitives::{FixedBytes, U256};
//!
//! let market = Market::new(
//!     FixedBytes::ZERO,
//!     U256::from(1_000_000) * WAD,
//!     U256::from(800_000) * WAD,
//!     U256::from(1_000_000) * WAD,
//!     U256::from(800_000) * WAD,
//!     1000,
//!     U256::ZERO,
//!     None,
//! );
//!
//! // Try to borrow more than liquidity
//! let result = market.borrow(U256::from(300_000) * WAD, 1000);
//!
//! match result {
//!     Err(SimError::InsufficientMarketLiquidity { market_id }) => {
//!         println!("Market {:?} has insufficient liquidity", market_id);
//!     }
//!     _ => unreachable!(),
//! }
//! ```

use alloy_primitives::{Address, FixedBytes};
use thiserror::Error;

/// Type alias for a 32-byte market ID
pub type MarketId = FixedBytes<32>;

/// Errors that can occur during simulation
#[derive(Debug, Error)]
pub enum SimError {
    /// Interest accrual was attempted with a timestamp before the last update
    #[error("Invalid interest accrual: timestamp {timestamp} is before last update {last_update}")]
    InvalidInterestAccrual { timestamp: u64, last_update: u64 },

    /// All market supply caps have been reached during a vault deposit
    #[error("All caps reached for vault {vault}: {remaining} assets could not be deposited")]
    AllCapsReached { vault: Address, remaining: u128 },

    /// Not enough liquidity for withdrawal
    #[error("Not enough liquidity for vault {vault}: {remaining} assets could not be withdrawn")]
    NotEnoughLiquidity { vault: Address, remaining: u128 },

    /// Division by zero
    #[error("Division by zero")]
    DivisionByZero,

    /// Market not found in vault allocations
    #[error("Market {market_id} not found in vault allocations")]
    MarketNotFound { market_id: MarketId },

    /// Vault has no supply queue
    #[error("Vault has empty supply queue")]
    EmptySupplyQueue,

    /// Target APY cannot be achieved
    #[error("Target APY delta {target} cannot be achieved (deposit can only decrease APY)")]
    InvalidApyTarget { target: f64 },

    /// Binary search failed to converge
    #[error("Binary search failed to converge within {max_iterations} iterations")]
    ConvergenceFailure { max_iterations: u32 },

    /// Insufficient position for operation
    #[error("Insufficient position for user {user} in market {market_id}")]
    InsufficientPosition { user: Address, market_id: MarketId },

    /// Insufficient collateral for borrow
    #[error("Insufficient collateral for user {user} in market {market_id}")]
    InsufficientCollateral { user: Address, market_id: MarketId },

    /// Insufficient liquidity in market
    #[error("Insufficient liquidity in market {market_id}")]
    InsufficientMarketLiquidity { market_id: MarketId },

    /// Unknown oracle price
    #[error("Oracle price unknown for market {market_id}")]
    UnknownOraclePrice { market_id: MarketId },

    /// Market not enabled in vault
    #[error("Market {market_id} not enabled in vault {vault}")]
    MarketNotEnabled { vault: Address, market_id: MarketId },

    /// Unauthorized market (cap is zero)
    #[error("Unauthorized market {market_id} in vault {vault}")]
    UnauthorizedMarket { vault: Address, market_id: MarketId },

    /// Supply cap exceeded
    #[error("Supply cap exceeded for market {market_id} in vault {vault}: cap is {cap}")]
    SupplyCapExceeded {
        vault: Address,
        market_id: MarketId,
        cap: u128,
    },

    /// Inconsistent reallocation (total supplied != total withdrawn)
    #[error("Inconsistent reallocation in vault {vault}: supplied {supplied}, withdrawn {withdrawn}")]
    InconsistentReallocation {
        vault: Address,
        supplied: u128,
        withdrawn: u128,
    },

    /// Public allocator not configured
    #[error("Public allocator not configured for vault {vault}")]
    PublicAllocatorNotConfigured { vault: Address },

    /// Max inflow exceeded for public allocator
    #[error("Max inflow exceeded for market {market_id} in vault {vault}")]
    MaxInflowExceeded { vault: Address, market_id: MarketId },

    /// Max outflow exceeded for public allocator
    #[error("Max outflow exceeded for market {market_id} in vault {vault}")]
    MaxOutflowExceeded { vault: Address, market_id: MarketId },

    /// Empty withdrawals list for public reallocate
    #[error("Empty withdrawals list for vault {vault}")]
    EmptyWithdrawals { vault: Address },

    /// Deposit market included in withdrawals
    #[error("Deposit market {market_id} included in withdrawals for vault {vault}")]
    DepositMarketInWithdrawals { vault: Address, market_id: MarketId },

    /// Withdrawals not sorted
    #[error("Withdrawals not sorted for vault {vault}")]
    WithdrawalsNotSorted { vault: Address },
}

impl SimError {
    /// Returns `true` if the error is retryable.
    ///
    /// Simulation errors are deterministic and never retryable — the same inputs
    /// will always produce the same error.
    pub fn is_retryable(&self) -> bool {
        false
    }

    /// Returns `true` if the error is caused by invalid user input or configuration.
    ///
    /// User errors indicate the caller provided invalid parameters (e.g., trying to
    /// withdraw more than they own, borrowing without sufficient collateral). These
    /// are distinct from system/state errors like `DivisionByZero` or
    /// `InvalidInterestAccrual` which indicate a data or logic issue.
    pub fn is_user_error(&self) -> bool {
        matches!(
            self,
            SimError::InsufficientPosition { .. }
                | SimError::InsufficientCollateral { .. }
                | SimError::InsufficientMarketLiquidity { .. }
                | SimError::NotEnoughLiquidity { .. }
                | SimError::AllCapsReached { .. }
                | SimError::EmptyWithdrawals { .. }
                | SimError::DepositMarketInWithdrawals { .. }
                | SimError::WithdrawalsNotSorted { .. }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::FixedBytes;

    #[test]
    fn test_is_never_retryable() {
        let errors = [
            SimError::DivisionByZero,
            SimError::EmptySupplyQueue,
            SimError::InsufficientMarketLiquidity {
                market_id: FixedBytes::ZERO,
            },
            SimError::InvalidInterestAccrual {
                timestamp: 100,
                last_update: 200,
            },
        ];
        for err in &errors {
            assert!(!err.is_retryable());
        }
    }

    #[test]
    fn test_is_user_error_position_errors() {
        let err = SimError::InsufficientPosition {
            user: Address::ZERO,
            market_id: FixedBytes::ZERO,
        };
        assert!(err.is_user_error());

        let err = SimError::InsufficientCollateral {
            user: Address::ZERO,
            market_id: FixedBytes::ZERO,
        };
        assert!(err.is_user_error());
    }

    #[test]
    fn test_is_user_error_liquidity_errors() {
        let err = SimError::InsufficientMarketLiquidity {
            market_id: FixedBytes::ZERO,
        };
        assert!(err.is_user_error());

        let err = SimError::NotEnoughLiquidity {
            vault: Address::ZERO,
            remaining: 100,
        };
        assert!(err.is_user_error());

        let err = SimError::AllCapsReached {
            vault: Address::ZERO,
            remaining: 100,
        };
        assert!(err.is_user_error());
    }

    #[test]
    fn test_is_user_error_withdrawal_errors() {
        let err = SimError::EmptyWithdrawals {
            vault: Address::ZERO,
        };
        assert!(err.is_user_error());

        let err = SimError::WithdrawalsNotSorted {
            vault: Address::ZERO,
        };
        assert!(err.is_user_error());

        let err = SimError::DepositMarketInWithdrawals {
            vault: Address::ZERO,
            market_id: FixedBytes::ZERO,
        };
        assert!(err.is_user_error());
    }

    #[test]
    fn test_is_not_user_error_system_errors() {
        assert!(!SimError::DivisionByZero.is_user_error());
        assert!(!SimError::EmptySupplyQueue.is_user_error());
        assert!(
            !(SimError::InvalidInterestAccrual {
                timestamp: 100,
                last_update: 200,
            })
            .is_user_error()
        );
        assert!(
            !(SimError::MarketNotFound {
                market_id: FixedBytes::ZERO,
            })
            .is_user_error()
        );
        assert!(
            !(SimError::InvalidApyTarget { target: 0.5 }).is_user_error()
        );
        assert!(
            !(SimError::ConvergenceFailure {
                max_iterations: 100,
            })
            .is_user_error()
        );
    }
}
