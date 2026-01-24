//! Error types for the simulation library.

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
