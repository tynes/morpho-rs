//! Market state and operations for Morpho Blue markets.
//!
//! This module implements the core [`Market`] struct and functions for simulating
//! market state changes, calculating APYs, and analyzing operation impacts.
//!
//! # Overview
//!
//! A Morpho Blue market is a lending pool with:
//! - **Supply side**: Lenders deposit assets and earn interest
//! - **Borrow side**: Borrowers take loans and pay interest
//! - **Share-based accounting**: Positions are tracked via shares, not raw assets
//! - **Adaptive interest rates**: Rates adjust based on utilization
//!
//! # Key Operations
//!
//! - [`Market::supply`] / [`Market::withdraw`] - Lender operations
//! - [`Market::borrow`] / [`Market::repay`] - Borrower operations
//! - [`Market::accrue_interest`] - Update market state with accrued interest
//! - [`supply_apy_impact`] / [`borrow_apy_impact`] - Analyze APY changes
//!
//! # Example
//!
//! ```rust
//! use morpho_rs_sim::{Market, supply_apy_impact, WAD};
//! use alloy_primitives::{FixedBytes, U256};
//!
//! // Create a market
//! let market = Market::new(
//!     FixedBytes::ZERO,
//!     U256::from(1_000_000) * WAD,  // 1M supply
//!     U256::from(800_000) * WAD,    // 800K borrow (80% utilization)
//!     U256::from(1_000_000) * WAD,
//!     U256::from(800_000) * WAD,
//!     1000,
//!     U256::from(100_000_000_000_000_000u64), // 10% fee
//!     Some(U256::from(1_268_391_679u64)),
//! );
//!
//! // Simulate supply and check APY impact
//! let impact = supply_apy_impact(&market, U256::from(100_000) * WAD, 1000).unwrap();
//! assert!(impact.apy_delta < 0.0); // APY decreases with more supply
//! ```

use alloy_primitives::U256;

use crate::error::{MarketId, SimError};
use crate::irm::get_borrow_rate;
use crate::math::{
    self, assets_to_shares, mul_div_down, mul_div_up, rate_to_apy, shares_to_assets,
    w_div_down, w_div_up, w_mul_down, w_mul_up, w_taylor_compounded, zero_floor_sub,
    RoundingDirection, WAD,
};

/// Liquidation cursor used to calculate the liquidation incentive (30%)
pub const LIQUIDATION_CURSOR: U256 = U256::from_limbs([300_000_000_000_000_000, 0, 0, 0]);

/// Maximum liquidation incentive factor (115%)
pub const MAX_LIQUIDATION_INCENTIVE_FACTOR: U256 =
    U256::from_limbs([1_150_000_000_000_000_000, 0, 0, 0]);

/// Oracle price scale (1e36)
pub const ORACLE_PRICE_SCALE: U256 = U256::from_limbs([
    0xC097CE7BC90715B3,
    0x4B9F,
    0,
    0,
]); // 10^36

/// Represents a lending market on Morpho Blue.
#[derive(Debug, Clone)]
pub struct Market {
    /// The market's unique identifier (keccak256 hash of market params)
    pub id: MarketId,

    /// The amount of loan assets supplied in total on the market
    pub total_supply_assets: U256,

    /// The amount of loan assets borrowed in total from the market
    pub total_borrow_assets: U256,

    /// The total supply shares representing lender positions
    pub total_supply_shares: U256,

    /// The total borrow shares representing borrower debt
    pub total_borrow_shares: U256,

    /// The block timestamp (in seconds) when interest was last accrued
    pub last_update: u64,

    /// The protocol fee percentage (WAD-scaled, e.g., 0.1 WAD = 10%)
    pub fee: U256,

    /// If the market uses the Adaptive Curve IRM, the rate at target utilization.
    /// None for markets using other IRMs.
    pub rate_at_target: Option<U256>,

    /// Oracle price (collateral/loan, scaled by ORACLE_PRICE_SCALE)
    /// None if oracle is not set or reverts
    pub price: Option<U256>,

    /// Liquidation LTV (WAD-scaled)
    pub lltv: U256,
}

impl Market {
    /// Creates a new market with the given parameters.
    ///
    /// This constructor creates a market without oracle price information,
    /// suitable for markets that don't support collateralized borrowing
    /// or when oracle data is not available.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique 32-byte market identifier (keccak256 hash of market params)
    /// * `total_supply_assets` - Total assets supplied to the market (WAD-scaled)
    /// * `total_borrow_assets` - Total assets borrowed from the market (WAD-scaled)
    /// * `total_supply_shares` - Total supply shares representing lender positions
    /// * `total_borrow_shares` - Total borrow shares representing borrower debt
    /// * `last_update` - Unix timestamp when interest was last accrued
    /// * `fee` - Protocol fee percentage (WAD-scaled, e.g., 0.1 WAD = 10%)
    /// * `rate_at_target` - For Adaptive Curve IRM, the rate at 90% utilization.
    ///   Pass `None` for markets using other IRMs (will have 0% APY).
    ///
    /// # Example
    ///
    /// ```rust
    /// use morpho_rs_sim::{Market, WAD};
    /// use alloy_primitives::{FixedBytes, U256};
    ///
    /// let market = Market::new(
    ///     FixedBytes::ZERO,
    ///     U256::from(1_000_000) * WAD,  // 1M total supply
    ///     U256::from(800_000) * WAD,    // 800K total borrow
    ///     U256::from(1_000_000) * WAD,  // 1M supply shares
    ///     U256::from(800_000) * WAD,    // 800K borrow shares
    ///     1704067200,                    // last update timestamp
    ///     U256::from(100_000_000_000_000_000u64), // 10% protocol fee
    ///     Some(U256::from(1_268_391_679u64)),     // ~4% rate at target
    /// );
    ///
    /// assert_eq!(market.liquidity(), U256::from(200_000) * WAD);
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: MarketId,
        total_supply_assets: U256,
        total_borrow_assets: U256,
        total_supply_shares: U256,
        total_borrow_shares: U256,
        last_update: u64,
        fee: U256,
        rate_at_target: Option<U256>,
    ) -> Self {
        Self {
            id,
            total_supply_assets,
            total_borrow_assets,
            total_supply_shares,
            total_borrow_shares,
            last_update,
            fee,
            rate_at_target,
            price: None,
            lltv: U256::ZERO,
        }
    }

    /// Create a new market with price and LLTV
    #[allow(clippy::too_many_arguments)]
    pub fn new_with_oracle(
        id: MarketId,
        total_supply_assets: U256,
        total_borrow_assets: U256,
        total_supply_shares: U256,
        total_borrow_shares: U256,
        last_update: u64,
        fee: U256,
        rate_at_target: Option<U256>,
        price: Option<U256>,
        lltv: U256,
    ) -> Self {
        Self {
            id,
            total_supply_assets,
            total_borrow_assets,
            total_supply_shares,
            total_borrow_shares,
            last_update,
            fee,
            rate_at_target,
            price,
            lltv,
        }
    }

    /// Returns the market's current liquidity (supply - borrow)
    pub fn liquidity(&self) -> U256 {
        self.total_supply_assets
            .saturating_sub(self.total_borrow_assets)
    }

    /// Returns the market's utilization rate (WAD-scaled)
    ///
    /// Utilization = totalBorrowAssets / totalSupplyAssets
    pub fn utilization(&self) -> U256 {
        get_utilization(self.total_supply_assets, self.total_borrow_assets)
    }

    /// Get the borrow rates for this market
    fn get_accrual_borrow_rates(&self, timestamp: u64) -> Result<AccrualRates, SimError> {
        if timestamp < self.last_update {
            return Err(SimError::InvalidInterestAccrual {
                timestamp,
                last_update: self.last_update,
            });
        }

        let elapsed = timestamp - self.last_update;

        match self.rate_at_target {
            None => Ok(AccrualRates {
                elapsed,
                avg_borrow_rate: U256::ZERO,
                end_borrow_rate: U256::ZERO,
                end_rate_at_target: None,
            }),
            Some(rate_at_target) => {
                let result = get_borrow_rate(self.utilization(), rate_at_target, elapsed);
                Ok(AccrualRates {
                    elapsed,
                    avg_borrow_rate: result.avg_borrow_rate,
                    end_borrow_rate: result.end_borrow_rate,
                    end_rate_at_target: Some(result.end_rate_at_target),
                })
            }
        }
    }

    /// Returns the instantaneous borrow rate at the given timestamp
    pub fn get_end_borrow_rate(&self, timestamp: u64) -> Result<U256, SimError> {
        Ok(self.get_accrual_borrow_rates(timestamp)?.end_borrow_rate)
    }

    /// Returns the average borrow rate over the period from last_update to timestamp
    pub fn get_avg_borrow_rate(&self, timestamp: u64) -> Result<U256, SimError> {
        Ok(self.get_accrual_borrow_rates(timestamp)?.avg_borrow_rate)
    }

    /// Returns the supply rate at the given timestamp
    ///
    /// Supply rate = borrow_rate * utilization * (1 - fee)
    pub fn get_supply_rate(&self, timestamp: u64) -> Result<U256, SimError> {
        let borrow_rate = self.get_end_borrow_rate(timestamp)?;
        Ok(w_mul_up(
            w_mul_down(borrow_rate, self.utilization()),
            WAD - self.fee,
        ))
    }

    /// Returns the average supply rate over the period
    pub fn get_avg_supply_rate(&self, timestamp: u64) -> Result<U256, SimError> {
        let borrow_rate = self.get_avg_borrow_rate(timestamp)?;
        Ok(w_mul_up(
            w_mul_down(borrow_rate, self.utilization()),
            WAD - self.fee,
        ))
    }

    /// Returns the instantaneous borrow APY at the given timestamp
    pub fn get_borrow_apy(&self, timestamp: u64) -> Result<f64, SimError> {
        let rate = self.get_end_borrow_rate(timestamp)?;
        Ok(rate_to_apy(rate))
    }

    /// Returns the instantaneous supply APY at the given timestamp
    pub fn get_supply_apy(&self, timestamp: u64) -> Result<f64, SimError> {
        let rate = self.get_supply_rate(timestamp)?;
        Ok(rate_to_apy(rate))
    }

    /// Returns the average supply APY over the accrual period
    pub fn get_avg_supply_apy(&self, timestamp: u64) -> Result<f64, SimError> {
        let rate = self.get_avg_supply_rate(timestamp)?;
        Ok(rate_to_apy(rate))
    }

    /// Accrues interest on the market up to the given timestamp.
    ///
    /// This is the core function for advancing market state. Interest is calculated
    /// using the Adaptive Curve IRM (if configured) and applied to both supply and
    /// borrow totals. Protocol fees are minted as additional supply shares.
    ///
    /// # How Interest Accrual Works
    ///
    /// 1. Calculate elapsed time since `last_update`
    /// 2. Compute average borrow rate over the period using IRM
    /// 3. Calculate interest: `total_borrow * (e^(rate * time) - 1)`
    /// 4. Add interest to both `total_supply_assets` and `total_borrow_assets`
    /// 5. Mint fee shares to protocol: `fee_amount * shares / (total_assets - fee)`
    /// 6. Update `rate_at_target` based on utilization deviation
    ///
    /// # Arguments
    ///
    /// * `timestamp` - Current Unix timestamp in seconds. Must be >= `last_update`.
    ///
    /// # Returns
    ///
    /// A new `Market` instance with updated state. The original market is unchanged.
    ///
    /// # Errors
    ///
    /// - [`SimError::InvalidInterestAccrual`] if `timestamp < last_update`
    ///
    /// # Example
    ///
    /// ```rust
    /// use morpho_rs_sim::{Market, WAD};
    /// use alloy_primitives::{FixedBytes, U256};
    ///
    /// let market = Market::new(
    ///     FixedBytes::ZERO,
    ///     U256::from(1_000_000) * WAD,
    ///     U256::from(800_000) * WAD,
    ///     U256::from(1_000_000) * WAD,
    ///     U256::from(800_000) * WAD,
    ///     1000,  // last update at timestamp 1000
    ///     U256::from(100_000_000_000_000_000u64),
    ///     Some(U256::from(1_268_391_679u64)),
    /// );
    ///
    /// // Accrue 1 day of interest
    /// let accrued = market.accrue_interest(1000 + 86400).unwrap();
    ///
    /// // Both supply and borrow increased by the same interest amount
    /// assert!(accrued.total_supply_assets > market.total_supply_assets);
    /// assert!(accrued.total_borrow_assets > market.total_borrow_assets);
    /// ```
    pub fn accrue_interest(&self, timestamp: u64) -> Result<Market, SimError> {
        let rates = self.get_accrual_borrow_rates(timestamp)?;

        let AccruedInterest { interest, fee_shares } = get_accrued_interest(
            rates.avg_borrow_rate,
            self.total_supply_assets,
            self.total_borrow_assets,
            self.total_supply_shares,
            self.fee,
            rates.elapsed,
        );

        Ok(Market {
            id: self.id,
            total_supply_assets: self.total_supply_assets + interest,
            total_borrow_assets: self.total_borrow_assets + interest,
            total_supply_shares: self.total_supply_shares + fee_shares,
            total_borrow_shares: self.total_borrow_shares,
            last_update: timestamp,
            fee: self.fee,
            rate_at_target: rates.end_rate_at_target.or(self.rate_at_target),
            price: self.price,
            lltv: self.lltv,
        })
    }

    /// Supplies assets to the market as a lender.
    ///
    /// This simulates depositing assets into the market to earn interest.
    /// The operation first accrues pending interest, then mints supply shares
    /// proportional to the deposit amount.
    ///
    /// # Share Calculation
    ///
    /// Shares are calculated using virtual shares/assets to prevent manipulation:
    /// ```text
    /// shares = assets * (total_shares + VIRTUAL_SHARES) / (total_assets + VIRTUAL_ASSETS)
    /// ```
    ///
    /// # Arguments
    ///
    /// * `assets` - Amount of loan assets to supply (WAD-scaled)
    /// * `timestamp` - Current Unix timestamp for interest accrual
    ///
    /// # Returns
    ///
    /// A tuple of (new_market, shares_minted). The original market is unchanged.
    ///
    /// # Example
    ///
    /// ```rust
    /// use morpho_rs_sim::{Market, WAD};
    /// use alloy_primitives::{FixedBytes, U256};
    ///
    /// let market = Market::new(
    ///     FixedBytes::ZERO,
    ///     U256::from(1_000_000) * WAD,
    ///     U256::from(800_000) * WAD,
    ///     U256::from(1_000_000) * WAD,
    ///     U256::from(800_000) * WAD,
    ///     1000,
    ///     U256::from(100_000_000_000_000_000u64),
    ///     Some(U256::from(1_268_391_679u64)),
    /// );
    ///
    /// let supply_amount = U256::from(100_000) * WAD;
    /// let (new_market, shares) = market.supply(supply_amount, 1000).unwrap();
    ///
    /// assert!(shares > U256::ZERO);
    /// assert_eq!(new_market.total_supply_assets, market.total_supply_assets + supply_amount);
    /// ```
    pub fn supply(&self, assets: U256, timestamp: u64) -> Result<(Market, U256), SimError> {
        let mut market = self.accrue_interest(timestamp)?;

        let shares = market.to_supply_shares(assets, RoundingDirection::Down);

        market.total_supply_assets += assets;
        market.total_supply_shares += shares;

        Ok((market, shares))
    }

    /// Withdraw assets from the market
    pub fn withdraw(&self, assets: U256, timestamp: u64) -> Result<(Market, U256), SimError> {
        let mut market = self.accrue_interest(timestamp)?;

        let shares = market.to_supply_shares(assets, RoundingDirection::Up);

        market.total_supply_assets -= assets;
        market.total_supply_shares -= shares;

        if market.total_borrow_assets > market.total_supply_assets {
            return Err(SimError::InsufficientMarketLiquidity { market_id: self.id });
        }

        Ok((market, shares))
    }

    /// Borrows assets from the market.
    ///
    /// This simulates taking a loan from the market. The operation first accrues
    /// pending interest, then mints borrow shares representing the debt.
    ///
    /// # Important Notes
    ///
    /// - This function only updates market state. For collateralized borrowing
    ///   with health checks, use [`crate::position::Position::borrow`] instead.
    /// - The market must have sufficient liquidity (`total_supply - total_borrow >= assets`)
    ///
    /// # Arguments
    ///
    /// * `assets` - Amount of loan assets to borrow (WAD-scaled)
    /// * `timestamp` - Current Unix timestamp for interest accrual
    ///
    /// # Returns
    ///
    /// A tuple of (new_market, shares_minted). The original market is unchanged.
    ///
    /// # Errors
    ///
    /// - [`SimError::InsufficientMarketLiquidity`] if `assets > liquidity()`
    ///
    /// # Example
    ///
    /// ```rust
    /// use morpho_rs_sim::{Market, WAD};
    /// use alloy_primitives::{FixedBytes, U256};
    ///
    /// let market = Market::new(
    ///     FixedBytes::ZERO,
    ///     U256::from(1_000_000) * WAD,
    ///     U256::from(800_000) * WAD,
    ///     U256::from(1_000_000) * WAD,
    ///     U256::from(800_000) * WAD,
    ///     1000,
    ///     U256::from(100_000_000_000_000_000u64),
    ///     Some(U256::from(1_268_391_679u64)),
    /// );
    ///
    /// // Borrow 100K (market has 200K liquidity)
    /// let borrow_amount = U256::from(100_000) * WAD;
    /// let (new_market, shares) = market.borrow(borrow_amount, 1000).unwrap();
    ///
    /// assert!(shares > U256::ZERO);
    /// assert_eq!(new_market.total_borrow_assets, market.total_borrow_assets + borrow_amount);
    /// ```
    pub fn borrow(&self, assets: U256, timestamp: u64) -> Result<(Market, U256), SimError> {
        let mut market = self.accrue_interest(timestamp)?;

        let shares = market.to_borrow_shares(assets, RoundingDirection::Up);

        market.total_borrow_assets += assets;
        market.total_borrow_shares += shares;

        if market.total_borrow_assets > market.total_supply_assets {
            return Err(SimError::InsufficientMarketLiquidity { market_id: self.id });
        }

        Ok((market, shares))
    }

    /// Repay borrowed assets
    ///
    /// Returns the updated market and the amount of shares burned
    pub fn repay(&self, assets: U256, timestamp: u64) -> Result<(Market, U256), SimError> {
        let mut market = self.accrue_interest(timestamp)?;

        let shares = market.to_borrow_shares(assets, RoundingDirection::Down);

        market.total_borrow_assets -= assets;
        market.total_borrow_shares -= shares;

        Ok((market, shares))
    }

    /// Convert supply shares to assets
    pub fn to_supply_assets(&self, shares: U256, rounding: RoundingDirection) -> U256 {
        shares_to_assets(
            shares,
            self.total_supply_assets,
            self.total_supply_shares,
            rounding,
        )
    }

    /// Convert assets to supply shares
    pub fn to_supply_shares(&self, assets: U256, rounding: RoundingDirection) -> U256 {
        assets_to_shares(
            assets,
            self.total_supply_assets,
            self.total_supply_shares,
            rounding,
        )
    }

    /// Convert borrow shares to assets
    pub fn to_borrow_assets(&self, shares: U256, rounding: RoundingDirection) -> U256 {
        shares_to_assets(
            shares,
            self.total_borrow_assets,
            self.total_borrow_shares,
            rounding,
        )
    }

    /// Convert assets to borrow shares
    pub fn to_borrow_shares(&self, assets: U256, rounding: RoundingDirection) -> U256 {
        assets_to_shares(
            assets,
            self.total_borrow_assets,
            self.total_borrow_shares,
            rounding,
        )
    }

    // ==================== Utilization Targeting ====================

    /// Returns the smallest volume to supply until the market reaches the target utilization
    pub fn get_supply_to_utilization(&self, target_utilization: U256) -> U256 {
        get_supply_to_utilization(
            self.total_supply_assets,
            self.total_borrow_assets,
            target_utilization,
        )
    }

    /// Returns the amount to withdraw until the market reaches the target utilization
    pub fn get_withdraw_to_utilization(&self, target_utilization: U256) -> U256 {
        get_withdraw_to_utilization(
            self.total_supply_assets,
            self.total_borrow_assets,
            target_utilization,
        )
    }

    /// Returns the amount to borrow until the market reaches the target utilization
    pub fn get_borrow_to_utilization(&self, target_utilization: U256) -> U256 {
        get_borrow_to_utilization(
            self.total_supply_assets,
            self.total_borrow_assets,
            target_utilization,
        )
    }

    /// Returns the smallest volume to repay until the market reaches the target utilization
    pub fn get_repay_to_utilization(&self, target_utilization: U256) -> U256 {
        get_repay_to_utilization(
            self.total_supply_assets,
            self.total_borrow_assets,
            target_utilization,
        )
    }

    // ==================== Liquidation Calculations ====================

    /// Returns the value of collateral in loan assets
    pub fn get_collateral_value(&self, collateral: U256) -> Option<U256> {
        self.price
            .map(|price| mul_div_down(collateral, price, ORACLE_PRICE_SCALE))
    }

    /// Returns the maximum debt allowed given a certain amount of collateral
    pub fn get_max_borrow_assets(&self, collateral: U256) -> Option<U256> {
        self.get_collateral_value(collateral)
            .map(|value| w_mul_down(value, self.lltv))
    }

    /// Returns the liquidation incentive factor for this market
    pub fn get_liquidation_incentive_factor(&self) -> U256 {
        get_liquidation_incentive_factor(self.lltv)
    }

    /// Returns the amount of collateral that would be seized in a liquidation
    pub fn get_liquidation_seized_assets(&self, repaid_shares: U256) -> Option<U256> {
        let price = self.price?;
        if price.is_zero() {
            return Some(U256::ZERO);
        }

        let repaid_assets = self.to_borrow_assets(repaid_shares, RoundingDirection::Down);
        let incentive = self.get_liquidation_incentive_factor();
        let value_with_incentive = w_mul_down(repaid_assets, incentive);

        Some(mul_div_down(value_with_incentive, ORACLE_PRICE_SCALE, price))
    }

    /// Check if a position is healthy
    pub fn is_healthy(&self, collateral: U256, borrow_shares: U256) -> Option<bool> {
        let max_borrow = self.get_max_borrow_assets(collateral)?;
        let current_borrow = self.to_borrow_assets(borrow_shares, RoundingDirection::Up);
        Some(max_borrow >= current_borrow)
    }

    /// Returns the health factor of a position (WAD-scaled)
    pub fn get_health_factor(&self, collateral: U256, borrow_shares: U256) -> Option<U256> {
        let borrow_assets = self.to_borrow_assets(borrow_shares, RoundingDirection::Up);
        if borrow_assets.is_zero() {
            return Some(U256::MAX);
        }

        let max_borrow = self.get_max_borrow_assets(collateral)?;
        Some(w_div_down(max_borrow, borrow_assets))
    }

    /// Returns the LTV of a position (WAD-scaled)
    pub fn get_ltv(&self, collateral: U256, borrow_shares: U256) -> Option<U256> {
        if borrow_shares.is_zero() {
            return Some(U256::ZERO);
        }

        let collateral_value = self.get_collateral_value(collateral)?;
        if collateral_value.is_zero() {
            return Some(U256::MAX);
        }

        let borrow_assets = self.to_borrow_assets(borrow_shares, RoundingDirection::Up);
        Some(w_div_up(borrow_assets, collateral_value))
    }

    /// Returns the liquidation price of a position
    pub fn get_liquidation_price(&self, collateral: U256, borrow_shares: U256) -> Option<U256> {
        if borrow_shares.is_zero() || self.total_borrow_shares.is_zero() {
            return None;
        }

        let collateral_power = w_mul_down(collateral, self.lltv);
        if collateral_power.is_zero() {
            return Some(U256::MAX);
        }

        let borrow_assets = self.to_borrow_assets(borrow_shares, RoundingDirection::Up);
        Some(mul_div_up(borrow_assets, ORACLE_PRICE_SCALE, collateral_power))
    }

    /// Returns the amount of collateral that can be withdrawn while staying healthy
    pub fn get_withdrawable_collateral(&self, collateral: U256, borrow_shares: U256) -> Option<U256> {
        let price = self.price?;
        if price.is_zero() {
            return Some(U256::ZERO);
        }

        let borrow_assets = self.to_borrow_assets(borrow_shares, RoundingDirection::Up);
        let required_collateral = w_div_up(
            mul_div_up(borrow_assets, ORACLE_PRICE_SCALE, price),
            self.lltv,
        );

        Some(zero_floor_sub(collateral, required_collateral))
    }

    /// Returns the seizable collateral amount for a liquidation
    pub fn get_seizable_collateral(&self, collateral: U256, borrow_shares: U256) -> Option<U256> {
        let price = self.price?;
        if price.is_zero() {
            return Some(U256::ZERO);
        }

        // Can't seize from healthy position
        if self.is_healthy(collateral, borrow_shares)? {
            return Some(U256::ZERO);
        }

        let max_seizable = self.get_liquidation_seized_assets(borrow_shares)?;
        Some(math::min(collateral, max_seizable))
    }
}

/// Internal struct for accrual rate calculation results
struct AccrualRates {
    elapsed: u64,
    avg_borrow_rate: U256,
    end_borrow_rate: U256,
    end_rate_at_target: Option<U256>,
}

/// Result of interest accrual calculation
struct AccruedInterest {
    /// Total interest accrued
    interest: U256,
    /// Fee shares minted to the protocol
    fee_shares: U256,
}

// ==================== Utility Functions ====================

/// Calculate the utilization rate (WAD-scaled)
pub fn get_utilization(total_supply_assets: U256, total_borrow_assets: U256) -> U256 {
    if total_supply_assets.is_zero() {
        if total_borrow_assets > U256::ZERO {
            return U256::MAX;
        }
        return U256::ZERO;
    }
    w_div_down(total_borrow_assets, total_supply_assets)
}

/// Calculate the interest accrued on a market
fn get_accrued_interest(
    borrow_rate: U256,
    total_supply_assets: U256,
    total_borrow_assets: U256,
    total_supply_shares: U256,
    fee: U256,
    elapsed: u64,
) -> AccruedInterest {
    let interest = w_mul_down(
        total_borrow_assets,
        w_taylor_compounded(borrow_rate, U256::from(elapsed)),
    );

    let fee_amount = w_mul_down(interest, fee);

    // Calculate fee shares using adjusted total supply (after interest, before fee)
    let fee_shares = assets_to_shares(
        fee_amount,
        total_supply_assets + interest - fee_amount,
        total_supply_shares,
        RoundingDirection::Down,
    );

    AccruedInterest {
        interest,
        fee_shares,
    }
}

/// Calculate the liquidation incentive factor
pub fn get_liquidation_incentive_factor(lltv: U256) -> U256 {
    math::min(
        MAX_LIQUIDATION_INCENTIVE_FACTOR,
        w_div_down(
            WAD,
            WAD - w_mul_down(LIQUIDATION_CURSOR, WAD - lltv),
        ),
    )
}

/// Returns the smallest volume to supply until the market reaches the target utilization
pub fn get_supply_to_utilization(
    total_supply_assets: U256,
    total_borrow_assets: U256,
    target_utilization: U256,
) -> U256 {
    if target_utilization.is_zero() {
        if get_utilization(total_supply_assets, total_borrow_assets).is_zero() {
            return U256::ZERO;
        }
        return U256::MAX;
    }

    zero_floor_sub(
        w_div_up(total_borrow_assets, target_utilization),
        total_supply_assets,
    )
}

/// Returns the amount to withdraw until the market reaches the target utilization
pub fn get_withdraw_to_utilization(
    total_supply_assets: U256,
    total_borrow_assets: U256,
    target_utilization: U256,
) -> U256 {
    if target_utilization.is_zero() {
        if total_borrow_assets.is_zero() {
            return total_supply_assets;
        }
        return U256::ZERO;
    }

    zero_floor_sub(
        total_supply_assets,
        w_div_up(total_borrow_assets, target_utilization),
    )
}

/// Returns the amount to borrow until the market reaches the target utilization
pub fn get_borrow_to_utilization(
    total_supply_assets: U256,
    total_borrow_assets: U256,
    target_utilization: U256,
) -> U256 {
    zero_floor_sub(
        w_mul_down(total_supply_assets, target_utilization),
        total_borrow_assets,
    )
}

/// Returns the smallest volume to repay until the market reaches the target utilization
pub fn get_repay_to_utilization(
    total_supply_assets: U256,
    total_borrow_assets: U256,
    target_utilization: U256,
) -> U256 {
    zero_floor_sub(
        total_borrow_assets,
        w_mul_down(total_supply_assets, target_utilization),
    )
}

/// Result of supply APY impact calculation
#[derive(Debug, Clone)]
pub struct SupplyApyImpact {
    /// APY before the supply
    pub apy_before: f64,
    /// APY after the supply
    pub apy_after: f64,
    /// Change in APY
    pub apy_delta: f64,
    /// Shares received
    pub shares_received: U256,
}

/// Result of borrow APY impact calculation
#[derive(Debug, Clone)]
pub struct BorrowApyImpact {
    /// APY before the borrow
    pub apy_before: f64,
    /// APY after the borrow
    pub apy_after: f64,
    /// Change in APY
    pub apy_delta: f64,
    /// Shares minted
    pub shares_minted: U256,
}

/// Calculates the APY impact of supplying assets to a market.
///
/// This function simulates a supply operation and compares the APY before and after,
/// allowing you to understand how a deposit will affect market returns.
///
/// # Why APY Changes
///
/// When you supply to a market:
/// - **Utilization decreases**: More supply relative to borrow
/// - **Borrow rate decreases**: Lower utilization = lower rates on the IRM curve
/// - **Supply APY decreases**: Interest is spread across more suppliers
///
/// The magnitude of the change depends on the deposit size relative to market size.
///
/// # Arguments
///
/// * `market` - The market to analyze
/// * `amount` - Amount of assets to supply (WAD-scaled)
/// * `timestamp` - Current Unix timestamp
///
/// # Returns
///
/// A [`SupplyApyImpact`] containing:
/// - `apy_before`: APY before the supply (as decimal, e.g., 0.05 = 5%)
/// - `apy_after`: APY after the supply
/// - `apy_delta`: Change in APY (typically negative for supplies)
/// - `shares_received`: Supply shares that would be minted
///
/// # Example
///
/// ```rust
/// use morpho_rs_sim::{Market, supply_apy_impact, WAD};
/// use alloy_primitives::{FixedBytes, U256};
///
/// let market = Market::new(
///     FixedBytes::ZERO,
///     U256::from(1_000_000) * WAD,
///     U256::from(800_000) * WAD,
///     U256::from(1_000_000) * WAD,
///     U256::from(800_000) * WAD,
///     1000,
///     U256::from(100_000_000_000_000_000u64),
///     Some(U256::from(1_268_391_679u64)),
/// );
///
/// let impact = supply_apy_impact(&market, U256::from(100_000) * WAD, 1000).unwrap();
///
/// println!("APY: {:.2}% -> {:.2}%", impact.apy_before * 100.0, impact.apy_after * 100.0);
/// println!("Change: {:.4}%", impact.apy_delta * 100.0);
///
/// // Supply dilutes returns
/// assert!(impact.apy_delta < 0.0);
/// ```
pub fn supply_apy_impact(
    market: &Market,
    amount: U256,
    timestamp: u64,
) -> Result<SupplyApyImpact, SimError> {
    let apy_before = market.get_supply_apy(timestamp)?;
    let (new_market, shares) = market.supply(amount, timestamp)?;
    let apy_after = new_market.get_supply_apy(timestamp)?;

    Ok(SupplyApyImpact {
        apy_before,
        apy_after,
        apy_delta: apy_after - apy_before,
        shares_received: shares,
    })
}

/// Calculates the APY impact of borrowing from a market.
///
/// This function simulates a borrow operation and compares the borrow APY
/// before and after, helping borrowers understand their cost of capital.
///
/// # Why Borrow APY Changes
///
/// When you borrow from a market:
/// - **Utilization increases**: More borrow relative to supply
/// - **Borrow rate increases**: Higher utilization = higher rates on the IRM curve
///
/// The magnitude depends on the borrow size relative to market liquidity.
///
/// # Arguments
///
/// * `market` - The market to analyze
/// * `amount` - Amount of assets to borrow (WAD-scaled)
/// * `timestamp` - Current Unix timestamp
///
/// # Returns
///
/// A [`BorrowApyImpact`] containing:
/// - `apy_before`: Borrow APY before (cost of borrowing, as decimal)
/// - `apy_after`: Borrow APY after
/// - `apy_delta`: Change in borrow APY (typically positive, meaning higher cost)
/// - `shares_minted`: Borrow shares representing the debt
///
/// # Example
///
/// ```rust
/// use morpho_rs_sim::{Market, borrow_apy_impact, WAD};
/// use alloy_primitives::{FixedBytes, U256};
///
/// let market = Market::new(
///     FixedBytes::ZERO,
///     U256::from(1_000_000) * WAD,
///     U256::from(800_000) * WAD,
///     U256::from(1_000_000) * WAD,
///     U256::from(800_000) * WAD,
///     1000,
///     U256::from(100_000_000_000_000_000u64),
///     Some(U256::from(1_268_391_679u64)),
/// );
///
/// let impact = borrow_apy_impact(&market, U256::from(50_000) * WAD, 1000).unwrap();
///
/// // Borrowing increases utilization, raising rates for all borrowers
/// assert!(impact.apy_delta > 0.0);
/// ```
pub fn borrow_apy_impact(
    market: &Market,
    amount: U256,
    timestamp: u64,
) -> Result<BorrowApyImpact, SimError> {
    let apy_before = market.get_borrow_apy(timestamp)?;
    let (new_market, shares) = market.borrow(amount, timestamp)?;
    let apy_after = new_market.get_borrow_apy(timestamp)?;

    Ok(BorrowApyImpact {
        apy_before,
        apy_after,
        apy_delta: apy_after - apy_before,
        shares_minted: shares,
    })
}

/// Ranking entry for market comparison
#[derive(Debug, Clone)]
pub struct MarketRanking {
    pub market_id: MarketId,
    pub apy: f64,
    pub liquidity: U256,
    pub utilization: U256,
}

/// Rank markets by supply APY (descending)
pub fn rank_markets_by_supply_apy(
    markets: &[(MarketId, &Market)],
    timestamp: u64,
) -> Result<Vec<MarketRanking>, SimError> {
    let mut rankings: Vec<MarketRanking> = markets
        .iter()
        .filter_map(|(id, market)| {
            market.get_supply_apy(timestamp).ok().map(|apy| MarketRanking {
                market_id: *id,
                apy,
                liquidity: market.liquidity(),
                utilization: market.utilization(),
            })
        })
        .collect();

    rankings.sort_by(|a, b| b.apy.partial_cmp(&a.apy).unwrap_or(std::cmp::Ordering::Equal));
    Ok(rankings)
}

/// Rank markets by borrow APY (ascending - lower is better for borrowers)
pub fn rank_markets_by_borrow_apy(
    markets: &[(MarketId, &Market)],
    timestamp: u64,
) -> Result<Vec<MarketRanking>, SimError> {
    let mut rankings: Vec<MarketRanking> = markets
        .iter()
        .filter_map(|(id, market)| {
            market.get_borrow_apy(timestamp).ok().map(|apy| MarketRanking {
                market_id: *id,
                apy,
                liquidity: market.liquidity(),
                utilization: market.utilization(),
            })
        })
        .collect();

    rankings.sort_by(|a, b| a.apy.partial_cmp(&b.apy).unwrap_or(std::cmp::Ordering::Equal));
    Ok(rankings)
}

/// Find the best market for a given supply amount
pub fn find_best_market_for_supply(
    markets: &[(MarketId, &Market)],
    amount: U256,
    timestamp: u64,
) -> Result<Option<(MarketId, f64)>, SimError> {
    let mut best: Option<(MarketId, f64)> = None;

    for (id, market) in markets {
        // Check if market has capacity
        if market.liquidity() < amount {
            continue;
        }

        if let Ok(impact) = supply_apy_impact(market, amount, timestamp) {
            match best {
                None => best = Some((*id, impact.apy_after)),
                Some((_, best_apy)) if impact.apy_after > best_apy => {
                    best = Some((*id, impact.apy_after));
                }
                _ => {}
            }
        }
    }

    Ok(best)
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::FixedBytes;

    fn create_test_market() -> Market {
        Market::new(
            FixedBytes::ZERO,
            U256::from(1_000_000) * WAD,  // 1M total supply
            U256::from(800_000) * WAD,    // 800K total borrow (80% utilization)
            U256::from(1_000_000) * WAD,  // 1M supply shares
            U256::from(800_000) * WAD,    // 800K borrow shares
            1000,                          // last update
            U256::from(100_000_000_000_000_000u64), // 10% fee
            Some(U256::from(1_268_391_679u64)),     // ~4% rate at target
        )
    }

    fn create_test_market_with_oracle() -> Market {
        Market::new_with_oracle(
            FixedBytes::ZERO,
            U256::from(1_000_000) * WAD,
            U256::from(800_000) * WAD,
            U256::from(1_000_000) * WAD,
            U256::from(800_000) * WAD,
            1000,
            U256::from(100_000_000_000_000_000u64),
            Some(U256::from(1_268_391_679u64)),
            Some(ORACLE_PRICE_SCALE), // 1:1 price
            U256::from(800_000_000_000_000_000u64), // 80% LLTV
        )
    }

    #[test]
    fn test_utilization() {
        let market = create_test_market();
        let utilization = market.utilization();
        let util_f64 = math::rate_to_f64(utilization);
        assert!((util_f64 - 0.8).abs() < 0.01); // ~80% utilization
    }

    #[test]
    fn test_liquidity() {
        let market = create_test_market();
        let liquidity = market.liquidity();
        assert_eq!(liquidity, U256::from(200_000) * WAD); // 200K liquidity
    }

    #[test]
    fn test_accrue_interest() {
        let market = create_test_market();
        let timestamp = 1000 + 86400; // 1 day later

        let accrued = market.accrue_interest(timestamp).unwrap();

        // Total supply and borrow should increase
        assert!(accrued.total_supply_assets > market.total_supply_assets);
        assert!(accrued.total_borrow_assets > market.total_borrow_assets);
        // Interest should be equal on both sides
        let supply_increase = accrued.total_supply_assets - market.total_supply_assets;
        let borrow_increase = accrued.total_borrow_assets - market.total_borrow_assets;
        assert_eq!(supply_increase, borrow_increase);
    }

    #[test]
    fn test_supply() {
        let market = create_test_market();
        let timestamp = 1000;
        let supply_amount = U256::from(100_000) * WAD;

        let (new_market, shares) = market.supply(supply_amount, timestamp).unwrap();

        assert!(shares > U256::ZERO);
        assert_eq!(
            new_market.total_supply_assets,
            market.total_supply_assets + supply_amount
        );
    }

    #[test]
    fn test_borrow() {
        let market = create_test_market();
        let timestamp = 1000;
        let borrow_amount = U256::from(100_000) * WAD;

        let (new_market, shares) = market.borrow(borrow_amount, timestamp).unwrap();

        assert!(shares > U256::ZERO);
        assert_eq!(
            new_market.total_borrow_assets,
            market.total_borrow_assets + borrow_amount
        );
    }

    #[test]
    fn test_borrow_exceeds_liquidity() {
        let market = create_test_market();
        let borrow_amount = U256::from(300_000) * WAD; // More than 200K liquidity

        let result = market.borrow(borrow_amount, 1000);
        assert!(matches!(result, Err(SimError::InsufficientMarketLiquidity { .. })));
    }

    #[test]
    fn test_get_supply_apy() {
        let market = create_test_market();
        let apy = market.get_supply_apy(1000).unwrap();

        // APY should be positive and reasonable
        assert!(apy > 0.0);
        assert!(apy < 1.0); // Less than 100%
    }

    #[test]
    fn test_invalid_timestamp() {
        let market = create_test_market();
        let result = market.accrue_interest(500); // Before last_update

        assert!(result.is_err());
    }

    #[test]
    fn test_supply_to_utilization() {
        let market = create_test_market();
        // Current utilization is 80%, target 50%
        let target = U256::from(500_000_000_000_000_000u64); // 50%
        let supply_needed = market.get_supply_to_utilization(target);

        // At 50% utilization with 800K borrow, need 1.6M supply
        // Currently have 1M, need 600K more
        assert!(supply_needed > U256::ZERO);
    }

    #[test]
    fn test_withdraw_to_utilization() {
        let market = create_test_market();
        // Current utilization is 80%, target 90%
        let target = U256::from(900_000_000_000_000_000u64); // 90%
        let withdraw_available = market.get_withdraw_to_utilization(target);

        // At 90% utilization with 800K borrow, need ~889K supply
        // Can withdraw ~111K
        assert!(withdraw_available > U256::ZERO);
    }

    #[test]
    fn test_health_factor() {
        let market = create_test_market_with_oracle();
        let collateral = U256::from(100) * WAD;
        let borrow_shares = U256::from(50) * WAD; // 50 assets at 1:1

        let hf = market.get_health_factor(collateral, borrow_shares);
        assert!(hf.is_some());
        // With 80% LLTV, 100 collateral can borrow 80
        // Borrowing 50, HF = 80/50 = 1.6 WAD
        let hf_f64 = math::rate_to_f64(hf.unwrap());
        assert!(hf_f64 > 1.5 && hf_f64 < 1.7);
    }

    #[test]
    fn test_ltv() {
        let market = create_test_market_with_oracle();
        let collateral = U256::from(100) * WAD;
        let borrow_shares = U256::from(50) * WAD;

        let ltv = market.get_ltv(collateral, borrow_shares);
        assert!(ltv.is_some());
        // LTV = 50/100 = 0.5 WAD
        let ltv_f64 = math::rate_to_f64(ltv.unwrap());
        assert!((ltv_f64 - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_is_healthy() {
        let market = create_test_market_with_oracle();

        // Healthy position
        let healthy = market.is_healthy(U256::from(100) * WAD, U256::from(50) * WAD);
        assert_eq!(healthy, Some(true));

        // Unhealthy position (borrowing more than 80% LLTV allows)
        let unhealthy = market.is_healthy(U256::from(100) * WAD, U256::from(90) * WAD);
        assert_eq!(unhealthy, Some(false));
    }

    #[test]
    fn test_supply_apy_impact() {
        let market = create_test_market();
        let amount = U256::from(100_000) * WAD;

        let impact = supply_apy_impact(&market, amount, 1000).unwrap();

        // Supplying should decrease APY (dilution)
        assert!(impact.apy_delta < 0.0);
        assert!(impact.shares_received > U256::ZERO);
    }

    // ==================== New Tests ====================

    #[test]
    fn test_liquidation_incentive_factor() {
        // Test matching TypeScript value from MarketUtils.test.ts
        // LLTV = 86% should give liquidation incentive factor of 1043841336116910229
        let lltv = U256::from(860_000_000_000_000_000u64); // 86%
        let factor = get_liquidation_incentive_factor(lltv);

        // Should match TypeScript: 1043841336116910229n
        assert_eq!(factor, U256::from(1043841336116910229u64));
    }

    #[test]
    fn test_liquidation_incentive_factor_max() {
        // Very low LLTV should hit MAX_LIQUIDATION_INCENTIVE_FACTOR
        let low_lltv = U256::from(100_000_000_000_000_000u64); // 10%
        let factor = get_liquidation_incentive_factor(low_lltv);

        // Should be capped at 115%
        assert_eq!(factor, MAX_LIQUIDATION_INCENTIVE_FACTOR);
    }

    #[test]
    fn test_supply_to_utilization_specific() {
        // Test cases from TypeScript MarketUtils.test.ts
        // Case 1: 100% utilization -> 90% target
        let supply1 = get_supply_to_utilization(WAD, WAD, U256::from(900_000_000_000_000_000u64));
        assert_eq!(supply1, U256::from(111_111_111_111_111_112u64));

        // Case 2: 0% utilization -> 90% target (no borrow)
        let supply2 = get_supply_to_utilization(WAD, U256::ZERO, U256::from(900_000_000_000_000_000u64));
        assert_eq!(supply2, U256::ZERO);

        // Case 3: 0% utilization, 0% target
        let supply3 = get_supply_to_utilization(WAD, U256::ZERO, U256::ZERO);
        assert_eq!(supply3, U256::ZERO);

        // Case 4: tiny borrow, 0% target (impossible without supply = MAX)
        let supply4 = get_supply_to_utilization(WAD, U256::from(1), U256::ZERO);
        assert_eq!(supply4, U256::MAX);
    }

    #[test]
    fn test_withdraw_to_utilization_specific() {
        // Test cases from TypeScript
        // Case 1: 100% utilization -> 90% target (can't withdraw)
        let withdraw1 = get_withdraw_to_utilization(WAD, WAD, U256::from(900_000_000_000_000_000u64));
        assert_eq!(withdraw1, U256::ZERO);

        // Case 2: 50% utilization -> 90% target
        let two_wad = U256::from(2) * WAD;
        let withdraw2 = get_withdraw_to_utilization(two_wad, WAD, U256::from(900_000_000_000_000_000u64));
        assert_eq!(withdraw2, U256::from(888_888_888_888_888_888u64));

        // Case 3: 0% utilization -> 90% target (can withdraw all)
        let withdraw3 = get_withdraw_to_utilization(WAD, U256::ZERO, U256::from(900_000_000_000_000_000u64));
        assert_eq!(withdraw3, WAD);
    }

    #[test]
    fn test_borrow_to_utilization_specific() {
        // Test cases from TypeScript
        // Case 1: 100% utilization -> 90% target (can't borrow more)
        let borrow1 = get_borrow_to_utilization(WAD, WAD, U256::from(900_000_000_000_000_000u64));
        assert_eq!(borrow1, U256::ZERO);

        // Case 2: 0% utilization -> 90% target
        let borrow2 = get_borrow_to_utilization(WAD, U256::ZERO, U256::from(900_000_000_000_000_000u64));
        assert_eq!(borrow2, U256::from(900_000_000_000_000_000u64));
    }

    #[test]
    fn test_repay_to_utilization_specific() {
        // Test cases from TypeScript
        // Case 1: 100% utilization -> 90% target
        let repay1 = get_repay_to_utilization(WAD, WAD, U256::from(900_000_000_000_000_000u64));
        assert_eq!(repay1, U256::from(100_000_000_000_000_000u64));

        // Case 2: 0% utilization -> 90% target (nothing to repay)
        let repay2 = get_repay_to_utilization(WAD, U256::ZERO, U256::from(900_000_000_000_000_000u64));
        assert_eq!(repay2, U256::ZERO);
    }

    #[test]
    fn test_accrue_interest_no_time_elapsed() {
        let market = create_test_market();
        let timestamp = 1000; // Same as last_update

        let accrued = market.accrue_interest(timestamp).unwrap();

        // No time elapsed, should be identical
        assert_eq!(accrued.total_supply_assets, market.total_supply_assets);
        assert_eq!(accrued.total_borrow_assets, market.total_borrow_assets);
    }

    #[test]
    fn test_liquidation_price() {
        let market = create_test_market_with_oracle();
        let collateral = U256::from(100) * WAD;
        let borrow_shares = U256::from(50) * WAD; // ~50 assets at 1:1

        let liq_price = market.get_liquidation_price(collateral, borrow_shares);
        assert!(liq_price.is_some());

        // With 80% LLTV, liquidation happens when:
        // borrow_value / (collateral_value * lltv) >= 1
        // 50 * price_scale / (100 * price * 0.8) >= 1
        // price <= 50 * price_scale / (100 * 0.8) = 0.625 * price_scale
        let liq_price_f64 = math::rate_to_f64(liq_price.unwrap()) / 1e18; // Normalize by ORACLE_PRICE_SCALE
        assert!(liq_price_f64 < 1.0); // Should be below current price
    }

    #[test]
    fn test_liquidation_price_no_borrow() {
        let market = create_test_market_with_oracle();
        let collateral = U256::from(100) * WAD;
        let borrow_shares = U256::ZERO;

        let liq_price = market.get_liquidation_price(collateral, borrow_shares);
        // No borrow means no liquidation price
        assert!(liq_price.is_none() || liq_price == Some(U256::ZERO));
    }

    #[test]
    fn test_seizable_collateral_healthy() {
        let market = create_test_market_with_oracle();
        let collateral = U256::from(100) * WAD;
        let borrow_shares = U256::from(50) * WAD; // Healthy position

        let seizable = market.get_seizable_collateral(collateral, borrow_shares);
        // Healthy position, nothing seizable
        assert_eq!(seizable, Some(U256::ZERO));
    }

    #[test]
    fn test_seizable_collateral_unhealthy() {
        let market = create_test_market_with_oracle();
        let collateral = U256::from(100) * WAD;
        let borrow_shares = U256::from(90) * WAD; // Unhealthy (90% > 80% LLTV)

        let seizable = market.get_seizable_collateral(collateral, borrow_shares);
        // Unhealthy position, should be able to seize some
        assert!(seizable.is_some());
        assert!(seizable.unwrap() > U256::ZERO);
    }

    #[test]
    fn test_withdrawable_collateral() {
        let market = create_test_market_with_oracle();
        let collateral = U256::from(100) * WAD;
        let borrow_shares = U256::from(50) * WAD;

        let withdrawable = market.get_withdrawable_collateral(collateral, borrow_shares);
        assert!(withdrawable.is_some());

        // With 80% LLTV and 50 borrow, need at least 62.5 collateral
        // Can withdraw up to 100 - 62.5 = 37.5
        let withdrawable_f64 = math::rate_to_f64(withdrawable.unwrap());
        assert!(withdrawable_f64 > 30.0 && withdrawable_f64 < 40.0);
    }

    #[test]
    fn test_withdrawable_collateral_no_borrow() {
        let market = create_test_market_with_oracle();
        let collateral = U256::from(100) * WAD;
        let borrow_shares = U256::ZERO;

        let withdrawable = market.get_withdrawable_collateral(collateral, borrow_shares);
        // No borrow, can withdraw all
        assert_eq!(withdrawable, Some(collateral));
    }

    #[test]
    fn test_repay() {
        let market = create_test_market();
        let timestamp = 1000;
        let repay_amount = U256::from(100_000) * WAD;

        let (new_market, shares_repaid) = market.repay(repay_amount, timestamp).unwrap();

        assert!(shares_repaid > U256::ZERO);
        assert_eq!(
            new_market.total_borrow_assets,
            market.total_borrow_assets - repay_amount
        );
    }

    #[test]
    fn test_withdraw() {
        let market = create_test_market();
        let timestamp = 1000;
        let withdraw_amount = U256::from(100_000) * WAD; // Less than liquidity

        let (new_market, shares_burned) = market.withdraw(withdraw_amount, timestamp).unwrap();

        assert!(shares_burned > U256::ZERO);
        assert_eq!(
            new_market.total_supply_assets,
            market.total_supply_assets - withdraw_amount
        );
    }

    #[test]
    fn test_withdraw_insufficient_liquidity() {
        let market = create_test_market();
        let withdraw_amount = U256::from(300_000) * WAD; // More than 200K liquidity

        let result = market.withdraw(withdraw_amount, 1000);
        assert!(matches!(result, Err(SimError::InsufficientMarketLiquidity { .. })));
    }

    #[test]
    fn test_borrow_apy() {
        let market = create_test_market();
        let apy = market.get_borrow_apy(1000).unwrap();

        // Borrow APY should be positive and higher than supply APY
        assert!(apy > 0.0);
        let supply_apy = market.get_supply_apy(1000).unwrap();
        assert!(apy > supply_apy);
    }

    #[test]
    fn test_utilization_zero_supply() {
        // Zero supply with zero borrow should be 0% utilization
        let util = get_utilization(U256::ZERO, U256::ZERO);
        assert_eq!(util, U256::ZERO);

        // Zero supply with non-zero borrow should be MAX
        let util2 = get_utilization(U256::ZERO, U256::from(1));
        assert_eq!(util2, U256::MAX);
    }

    #[test]
    fn test_market_ranking() {
        // Create markets with different utilizations
        let market1 = Market::new(
            FixedBytes::from_slice(&[1; 32]),
            U256::from(1_000_000) * WAD,
            U256::from(900_000) * WAD, // 90% utilization - higher APY
            U256::from(1_000_000) * WAD,
            U256::from(900_000) * WAD,
            1000,
            U256::from(100_000_000_000_000_000u64),
            Some(U256::from(1_268_391_679u64)),
        );
        let market2 = Market::new(
            FixedBytes::from_slice(&[2; 32]),
            U256::from(1_000_000) * WAD,
            U256::from(500_000) * WAD, // 50% utilization - lower APY
            U256::from(1_000_000) * WAD,
            U256::from(500_000) * WAD,
            1000,
            U256::from(100_000_000_000_000_000u64),
            Some(U256::from(1_268_391_679u64)),
        );

        let markets: Vec<(MarketId, &Market)> = vec![
            (market1.id, &market1),
            (market2.id, &market2),
        ];

        let rankings = rank_markets_by_supply_apy(&markets, 1000).unwrap();

        // Higher utilization market should have higher supply APY (more interest distributed)
        assert_eq!(rankings.len(), 2);
        assert_eq!(rankings[0].market_id, market1.id);
    }
}
