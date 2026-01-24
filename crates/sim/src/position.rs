//! Position tracking for Morpho Blue markets.
//!
//! This module implements position tracking with computed properties
//! like health factor, LTV, liquidation price, and capacity limits.

use alloy_primitives::{Address, U256};

use crate::error::{MarketId, SimError};
use crate::market::Market;
use crate::math::{w_div_down, w_div_up, zero_floor_sub, RoundingDirection};

/// Represents a user's position in a Morpho Blue market.
#[derive(Debug, Clone)]
pub struct Position {
    /// The user holding this position
    pub user: Address,
    /// The market ID
    pub market_id: MarketId,
    /// Amount of supply shares held
    pub supply_shares: U256,
    /// Amount of borrow shares held
    pub borrow_shares: U256,
    /// Amount of collateral assets held
    pub collateral: U256,
}

impl Position {
    /// Create a new position
    pub fn new(
        user: Address,
        market_id: MarketId,
        supply_shares: U256,
        borrow_shares: U256,
        collateral: U256,
    ) -> Self {
        Self {
            user,
            market_id,
            supply_shares,
            borrow_shares,
            collateral,
        }
    }

    /// Create an empty position
    pub fn empty(user: Address, market_id: MarketId) -> Self {
        Self {
            user,
            market_id,
            supply_shares: U256::ZERO,
            borrow_shares: U256::ZERO,
            collateral: U256::ZERO,
        }
    }

    /// Returns the supply assets for this position
    pub fn supply_assets(&self, market: &Market) -> U256 {
        market.to_supply_assets(self.supply_shares, RoundingDirection::Down)
    }

    /// Returns the borrow assets for this position
    pub fn borrow_assets(&self, market: &Market) -> U256 {
        market.to_borrow_assets(self.borrow_shares, RoundingDirection::Up)
    }

    /// Returns the collateral value in loan assets
    pub fn collateral_value(&self, market: &Market) -> Option<U256> {
        market.get_collateral_value(self.collateral)
    }

    /// Returns the maximum debt allowed for this position
    pub fn max_borrow_assets(&self, market: &Market) -> Option<U256> {
        market.get_max_borrow_assets(self.collateral)
    }

    /// Returns the additional borrowable amount
    pub fn max_borrowable_assets(&self, market: &Market) -> Option<U256> {
        let max_borrow = self.max_borrow_assets(market)?;
        let current_borrow = self.borrow_assets(market);
        Some(zero_floor_sub(max_borrow, current_borrow))
    }

    /// Check if this position is healthy
    pub fn is_healthy(&self, market: &Market) -> Option<bool> {
        market.is_healthy(self.collateral, self.borrow_shares)
    }

    /// Check if this position can be liquidated
    pub fn is_liquidatable(&self, market: &Market) -> Option<bool> {
        self.is_healthy(market).map(|healthy| !healthy)
    }

    /// Returns the health factor (WAD-scaled)
    pub fn health_factor(&self, market: &Market) -> Option<U256> {
        market.get_health_factor(self.collateral, self.borrow_shares)
    }

    /// Returns the LTV (WAD-scaled)
    pub fn ltv(&self, market: &Market) -> Option<U256> {
        market.get_ltv(self.collateral, self.borrow_shares)
    }

    /// Returns the liquidation price
    pub fn liquidation_price(&self, market: &Market) -> Option<U256> {
        market.get_liquidation_price(self.collateral, self.borrow_shares)
    }

    /// Returns the price variation to liquidation (WAD-scaled, negative = safe)
    pub fn price_variation_to_liquidation(&self, market: &Market) -> Option<i128> {
        let price = market.price?;
        if price.is_zero() {
            return None;
        }

        let liq_price = self.liquidation_price(market)?;

        // Variation = (liq_price - price) / price
        // Negative when healthy (price needs to drop)
        // Positive when unhealthy (price needs to rise)
        if liq_price >= price {
            // Unhealthy or at liquidation
            let variation = w_div_up(liq_price - price, price);
            Some(variation.saturating_to::<i128>())
        } else {
            // Healthy - negative variation
            let variation = w_div_down(price - liq_price, price);
            Some(-(variation.saturating_to::<i128>()))
        }
    }

    /// Returns the borrow capacity usage (WAD-scaled)
    pub fn borrow_capacity_usage(&self, market: &Market) -> Option<U256> {
        let max_borrow = self.max_borrow_assets(market)?;
        if max_borrow.is_zero() {
            if self.borrow_shares.is_zero() {
                return Some(U256::ZERO);
            }
            return Some(U256::MAX);
        }

        let current_borrow = self.borrow_assets(market);
        Some(w_div_up(current_borrow, max_borrow))
    }

    /// Returns the amount of collateral that can be withdrawn
    pub fn withdrawable_collateral(&self, market: &Market) -> Option<U256> {
        market.get_withdrawable_collateral(self.collateral, self.borrow_shares)
    }

    /// Returns the seizable collateral in a liquidation
    pub fn seizable_collateral(&self, market: &Market) -> Option<U256> {
        market.get_seizable_collateral(self.collateral, self.borrow_shares)
    }

    /// Returns the maximum withdrawable supply assets
    pub fn withdrawable_supply(&self, market: &Market) -> U256 {
        let supply_assets = self.supply_assets(market);
        let liquidity = market.liquidity();
        crate::math::min(supply_assets, liquidity)
    }

    // ==================== Position Mutations ====================

    /// Supply assets to the position
    pub fn supply(
        &self,
        market: &Market,
        assets: U256,
        timestamp: u64,
    ) -> Result<(Position, Market, U256), SimError> {
        let (new_market, shares) = market.supply(assets, timestamp)?;

        let mut new_position = self.clone();
        new_position.supply_shares += shares;

        Ok((new_position, new_market, shares))
    }

    /// Withdraw assets from the position
    pub fn withdraw(
        &self,
        market: &Market,
        assets: U256,
        timestamp: u64,
    ) -> Result<(Position, Market, U256), SimError> {
        let (new_market, shares) = market.withdraw(assets, timestamp)?;

        let mut new_position = self.clone();
        if shares > new_position.supply_shares {
            return Err(SimError::InsufficientPosition {
                user: self.user,
                market_id: self.market_id,
            });
        }
        new_position.supply_shares -= shares;

        Ok((new_position, new_market, shares))
    }

    /// Supply collateral to the position
    pub fn supply_collateral(&self, assets: U256) -> Position {
        let mut new_position = self.clone();
        new_position.collateral += assets;
        new_position
    }

    /// Withdraw collateral from the position
    pub fn withdraw_collateral(
        &self,
        market: &Market,
        assets: U256,
        timestamp: u64,
    ) -> Result<Position, SimError> {
        if market.price.is_none() {
            return Err(SimError::UnknownOraclePrice {
                market_id: self.market_id,
            });
        }

        let accrued_market = market.accrue_interest(timestamp)?;

        let mut new_position = self.clone();
        if assets > new_position.collateral {
            return Err(SimError::InsufficientPosition {
                user: self.user,
                market_id: self.market_id,
            });
        }
        new_position.collateral -= assets;

        // Check health after withdrawal
        if let Some(false) = new_position.is_healthy(&accrued_market) {
            return Err(SimError::InsufficientCollateral {
                user: self.user,
                market_id: self.market_id,
            });
        }

        Ok(new_position)
    }

    /// Borrow assets from the position
    pub fn borrow(
        &self,
        market: &Market,
        assets: U256,
        timestamp: u64,
    ) -> Result<(Position, Market, U256), SimError> {
        if market.price.is_none() {
            return Err(SimError::UnknownOraclePrice {
                market_id: self.market_id,
            });
        }

        let (new_market, shares) = market.borrow(assets, timestamp)?;

        let mut new_position = self.clone();
        new_position.borrow_shares += shares;

        // Check health after borrowing
        if let Some(false) = new_position.is_healthy(&new_market) {
            return Err(SimError::InsufficientCollateral {
                user: self.user,
                market_id: self.market_id,
            });
        }

        Ok((new_position, new_market, shares))
    }

    /// Repay borrowed assets
    pub fn repay(
        &self,
        market: &Market,
        assets: U256,
        timestamp: u64,
    ) -> Result<(Position, Market, U256), SimError> {
        let (new_market, shares) = market.repay(assets, timestamp)?;

        let mut new_position = self.clone();
        if shares > new_position.borrow_shares {
            return Err(SimError::InsufficientPosition {
                user: self.user,
                market_id: self.market_id,
            });
        }
        new_position.borrow_shares -= shares;

        Ok((new_position, new_market, shares))
    }
}

/// Capacity limit information
#[derive(Debug, Clone)]
pub struct CapacityLimit {
    /// The maximum amount
    pub value: U256,
    /// The reason for the limit
    pub reason: CapacityLimitReason,
}

/// Reasons for capacity limits
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapacityLimitReason {
    /// Limited by user's balance
    Balance,
    /// Limited by market liquidity
    Liquidity,
    /// Limited by position size
    Position,
    /// Limited by collateral/health factor
    Collateral,
    /// No limit
    None,
}

/// All capacity limits for a position
#[derive(Debug, Clone)]
pub struct PositionCapacities {
    /// Max supply capacity
    pub supply: CapacityLimit,
    /// Max withdraw capacity
    pub withdraw: CapacityLimit,
    /// Max borrow capacity
    pub borrow: CapacityLimit,
    /// Max repay capacity
    pub repay: CapacityLimit,
    /// Max supply collateral capacity
    pub supply_collateral: CapacityLimit,
    /// Max withdraw collateral capacity
    pub withdraw_collateral: CapacityLimit,
}

impl Position {
    /// Get all capacity limits for this position
    pub fn get_capacities(
        &self,
        market: &Market,
        loan_balance: U256,
        collateral_balance: U256,
    ) -> PositionCapacities {
        let supply_assets = self.supply_assets(market);
        let borrow_assets = self.borrow_assets(market);
        let liquidity = market.liquidity();

        // Supply: limited by balance
        let supply = CapacityLimit {
            value: loan_balance,
            reason: CapacityLimitReason::Balance,
        };

        // Withdraw: limited by position or liquidity
        let withdraw = if supply_assets <= liquidity {
            CapacityLimit {
                value: supply_assets,
                reason: CapacityLimitReason::Position,
            }
        } else {
            CapacityLimit {
                value: liquidity,
                reason: CapacityLimitReason::Liquidity,
            }
        };

        // Borrow: limited by collateral or liquidity
        let borrow = match self.max_borrowable_assets(market) {
            Some(max_borrowable) => {
                if max_borrowable <= liquidity {
                    CapacityLimit {
                        value: max_borrowable,
                        reason: CapacityLimitReason::Collateral,
                    }
                } else {
                    CapacityLimit {
                        value: liquidity,
                        reason: CapacityLimitReason::Liquidity,
                    }
                }
            }
            None => CapacityLimit {
                value: U256::ZERO,
                reason: CapacityLimitReason::Collateral,
            },
        };

        // Repay: limited by balance or position
        let repay = if loan_balance <= borrow_assets {
            CapacityLimit {
                value: loan_balance,
                reason: CapacityLimitReason::Balance,
            }
        } else {
            CapacityLimit {
                value: borrow_assets,
                reason: CapacityLimitReason::Position,
            }
        };

        // Supply collateral: limited by balance
        let supply_collateral = CapacityLimit {
            value: collateral_balance,
            reason: CapacityLimitReason::Balance,
        };

        // Withdraw collateral: limited by health factor
        let withdraw_collateral = match self.withdrawable_collateral(market) {
            Some(withdrawable) => {
                if withdrawable <= self.collateral {
                    CapacityLimit {
                        value: withdrawable,
                        reason: CapacityLimitReason::Collateral,
                    }
                } else {
                    CapacityLimit {
                        value: self.collateral,
                        reason: CapacityLimitReason::Position,
                    }
                }
            }
            None => CapacityLimit {
                value: U256::ZERO,
                reason: CapacityLimitReason::Collateral,
            },
        };

        PositionCapacities {
            supply,
            withdraw,
            borrow,
            repay,
            supply_collateral,
            withdraw_collateral,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market::ORACLE_PRICE_SCALE;
    use crate::math::WAD;
    use alloy_primitives::FixedBytes;

    fn create_test_market() -> Market {
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

    fn create_test_position() -> Position {
        Position::new(
            Address::ZERO,
            FixedBytes::ZERO,
            U256::from(1000) * WAD, // 1000 supply shares
            U256::from(500) * WAD,  // 500 borrow shares
            U256::from(1000) * WAD, // 1000 collateral
        )
    }

    #[test]
    fn test_position_assets() {
        let market = create_test_market();
        let position = create_test_position();

        let supply = position.supply_assets(&market);
        let borrow = position.borrow_assets(&market);

        // With 1:1 ratios, should be approximately equal to shares
        assert!(supply > U256::from(990) * WAD);
        assert!(borrow > U256::from(490) * WAD);
    }

    #[test]
    fn test_position_health() {
        let market = create_test_market();
        let position = create_test_position();

        let is_healthy = position.is_healthy(&market);
        assert_eq!(is_healthy, Some(true));

        let hf = position.health_factor(&market);
        assert!(hf.is_some());
        // With 80% LLTV, 1000 collateral can borrow 800
        // Borrowing 500, HF = 800/500 = 1.6
        let hf_f64 = crate::math::rate_to_f64(hf.unwrap());
        assert!(hf_f64 > 1.5);
    }

    #[test]
    fn test_position_ltv() {
        let market = create_test_market();
        let position = create_test_position();

        let ltv = position.ltv(&market);
        assert!(ltv.is_some());
        // LTV = 500/1000 = 0.5
        let ltv_f64 = crate::math::rate_to_f64(ltv.unwrap());
        assert!((ltv_f64 - 0.5).abs() < 0.1);
    }

    #[test]
    fn test_position_supply() {
        let market = create_test_market();
        let position = create_test_position();

        let (new_position, new_market, shares) =
            position.supply(&market, U256::from(100) * WAD, 1000).unwrap();

        assert!(shares > U256::ZERO);
        assert!(new_position.supply_shares > position.supply_shares);
        assert!(new_market.total_supply_assets > market.total_supply_assets);
    }

    #[test]
    fn test_position_borrow() {
        let market = create_test_market();
        let position = create_test_position();

        // Should be able to borrow more (have room)
        let (new_position, _, shares) =
            position.borrow(&market, U256::from(100) * WAD, 1000).unwrap();

        assert!(shares > U256::ZERO);
        assert!(new_position.borrow_shares > position.borrow_shares);
    }

    #[test]
    fn test_position_borrow_insufficient_collateral() {
        let market = create_test_market();
        let position = create_test_position();

        // Try to borrow too much
        let result = position.borrow(&market, U256::from(500) * WAD, 1000);
        assert!(matches!(result, Err(SimError::InsufficientCollateral { .. })));
    }

    #[test]
    fn test_get_capacities() {
        let market = create_test_market();
        let position = create_test_position();

        let capacities = position.get_capacities(
            &market,
            U256::from(10000) * WAD, // loan balance
            U256::from(5000) * WAD,  // collateral balance
        );

        // Should have supply capacity equal to balance
        assert_eq!(capacities.supply.value, U256::from(10000) * WAD);
        assert_eq!(capacities.supply.reason, CapacityLimitReason::Balance);

        // Should have some borrow capacity
        assert!(capacities.borrow.value > U256::ZERO);
    }

    // ==================== New Tests ====================

    #[test]
    fn test_position_no_borrow() {
        // Position with no borrow should have max health factor
        let market = create_test_market();
        let position = Position::new(
            Address::ZERO,
            market.id,
            U256::from(1000) * WAD, // supply shares
            U256::ZERO,              // no borrow
            U256::from(1000) * WAD,  // collateral
        );

        let hf = position.health_factor(&market);
        assert!(hf.is_some());
        assert_eq!(hf.unwrap(), U256::MAX);

        assert!(position.is_healthy(&market).unwrap());
        assert!(!position.is_liquidatable(&market).unwrap());
    }

    #[test]
    fn test_position_no_collateral() {
        // Position with borrow but no collateral should be liquidatable
        let market = create_test_market();
        let position = Position::new(
            Address::ZERO,
            market.id,
            U256::from(1000) * WAD, // supply shares
            U256::from(500) * WAD,  // borrow shares
            U256::ZERO,              // no collateral
        );

        let hf = position.health_factor(&market);
        // With zero collateral and non-zero borrow, health factor is 0
        assert!(hf.is_some());
        assert_eq!(hf.unwrap(), U256::ZERO);

        assert!(!position.is_healthy(&market).unwrap());
        assert!(position.is_liquidatable(&market).unwrap());
    }

    #[test]
    fn test_position_liquidation_price() {
        let market = create_test_market();
        let position = create_test_position();

        let liq_price = position.liquidation_price(&market);
        assert!(liq_price.is_some());

        // With 80% LLTV, 1000 collateral, ~500 borrow
        // Liquidation when: borrow * price_scale / (collateral * price * lltv) >= 1
        // price <= borrow * price_scale / (collateral * lltv)
        let price = market.price.unwrap();
        let liq = liq_price.unwrap();

        // Liquidation price should be below current price (healthy position)
        assert!(liq < price);
    }

    #[test]
    fn test_position_price_variation_to_liquidation() {
        let market = create_test_market();
        let position = create_test_position();

        let variation = position.price_variation_to_liquidation(&market);
        assert!(variation.is_some());

        // Healthy position: negative variation (price needs to drop)
        let var = variation.unwrap();
        assert!(var < 0);
    }

    #[test]
    fn test_position_withdrawable_collateral() {
        let market = create_test_market();
        let position = create_test_position();

        let withdrawable = position.withdrawable_collateral(&market);
        assert!(withdrawable.is_some());

        // With borrow, can't withdraw all collateral
        let w = withdrawable.unwrap();
        assert!(w > U256::ZERO);
        assert!(w < position.collateral);
    }

    #[test]
    fn test_position_withdrawable_collateral_no_borrow() {
        let market = create_test_market();
        let position = Position::new(
            Address::ZERO,
            market.id,
            U256::from(1000) * WAD,
            U256::ZERO, // no borrow
            U256::from(1000) * WAD,
        );

        let withdrawable = position.withdrawable_collateral(&market);
        assert!(withdrawable.is_some());

        // No borrow, can withdraw all collateral
        assert_eq!(withdrawable.unwrap(), position.collateral);
    }

    #[test]
    fn test_position_max_borrow() {
        let market = create_test_market();
        let position = Position::new(
            Address::ZERO,
            market.id,
            U256::from(1000) * WAD,
            U256::ZERO, // no current borrow
            U256::from(1000) * WAD,
        );

        let max_borrow = position.max_borrow_assets(&market);
        assert!(max_borrow.is_some());

        // With 80% LLTV and 1000 collateral at 1:1 price, max borrow = 800
        let max = max_borrow.unwrap();
        let max_f64 = crate::math::rate_to_f64(max);
        assert!(max_f64 > 790.0 && max_f64 < 810.0);
    }

    #[test]
    fn test_position_borrow_capacity_usage() {
        let market = create_test_market();
        let position = create_test_position();

        let usage = position.borrow_capacity_usage(&market);
        assert!(usage.is_some());

        // With ~500 borrow and ~800 max borrow, usage should be ~62.5%
        let usage_f64 = crate::math::rate_to_f64(usage.unwrap());
        assert!(usage_f64 > 0.5 && usage_f64 < 0.75);
    }

    #[test]
    fn test_position_supply_and_withdraw() {
        let market = create_test_market();
        let position = create_test_position();

        // Supply some assets
        let (new_pos, new_market, shares) = position
            .supply(&market, U256::from(100) * WAD, 1000)
            .unwrap();

        assert!(shares > U256::ZERO);
        assert!(new_pos.supply_shares > position.supply_shares);

        // Withdraw some
        let (final_pos, _, assets) = new_pos
            .withdraw(&new_market, U256::from(50) * WAD, 1000)
            .unwrap();

        assert!(assets > U256::ZERO);
        assert!(final_pos.supply_shares < new_pos.supply_shares);
    }

    #[test]
    fn test_position_supply_and_withdraw_collateral() {
        let market = create_test_market();
        let position = Position::new(
            Address::ZERO,
            market.id,
            U256::ZERO,
            U256::ZERO,
            U256::ZERO,
        );

        // Supply collateral
        let new_pos = position.supply_collateral(U256::from(100) * WAD);
        assert_eq!(new_pos.collateral, U256::from(100) * WAD);

        // Withdraw some collateral
        let final_pos = new_pos
            .withdraw_collateral(&market, U256::from(50) * WAD, 1000)
            .unwrap();

        assert_eq!(final_pos.collateral, U256::from(50) * WAD);
    }

    #[test]
    fn test_position_repay() {
        let market = create_test_market();
        let position = create_test_position();

        // Repay some debt
        let (new_pos, new_market, shares_repaid) = position
            .repay(&market, U256::from(100) * WAD, 1000)
            .unwrap();

        assert!(shares_repaid > U256::ZERO);
        assert!(new_pos.borrow_shares < position.borrow_shares);
        assert!(new_market.total_borrow_assets < market.total_borrow_assets);
    }

    #[test]
    fn test_position_with_undefined_price() {
        // Market with no price
        let market = Market::new(
            FixedBytes::ZERO,
            U256::from(1_000_000) * WAD,
            U256::from(800_000) * WAD,
            U256::from(1_000_000) * WAD,
            U256::from(800_000) * WAD,
            1000,
            U256::from(100_000_000_000_000_000u64),
            Some(U256::from(1_268_391_679u64)),
        );
        // Note: Market::new doesn't set price or lltv (lltv=0), so these should return None
        // or special values (liquidation_price returns MAX when lltv=0)

        let position = create_test_position();

        // Price-dependent methods should return None
        assert!(position.ltv(&market).is_none());
        assert!(position.health_factor(&market).is_none());
        assert!(position.is_healthy(&market).is_none());
        assert!(position.max_borrow_assets(&market).is_none());
        assert!(position.withdrawable_collateral(&market).is_none());

        // liquidation_price with lltv=0 returns MAX (collateral has zero power)
        assert_eq!(position.liquidation_price(&market), Some(U256::MAX));
    }

    #[test]
    fn test_capacity_limit_reasons() {
        let market = create_test_market();
        let position = create_test_position();

        // Test with limited balance
        let capacities = position.get_capacities(
            &market,
            U256::from(100) * WAD,  // small loan balance
            U256::from(5000) * WAD, // large collateral balance
        );

        // Supply should be limited by balance
        assert_eq!(capacities.supply.reason, CapacityLimitReason::Balance);
        assert_eq!(capacities.supply.value, U256::from(100) * WAD);

        // Test with limited liquidity - create a position with MORE supply than liquidity
        let low_liquidity_market = Market::new_with_oracle(
            FixedBytes::ZERO,
            U256::from(1_000_000) * WAD,
            U256::from(999_500) * WAD, // Very high utilization, only 500 WAD liquidity
            U256::from(1_000_000) * WAD,
            U256::from(999_500) * WAD,
            1000,
            U256::from(100_000_000_000_000_000u64),
            Some(U256::from(1_268_391_679u64)),
            Some(ORACLE_PRICE_SCALE),
            U256::from(800_000_000_000_000_000u64),
        );

        // Position has 1000 supply shares (~1000 assets), market has only 500 liquidity
        // So withdraw should be limited by liquidity
        let capacities2 = position.get_capacities(
            &low_liquidity_market,
            U256::from(10000) * WAD,
            U256::from(5000) * WAD,
        );

        // Withdraw should be limited by liquidity (only 500 available, less than position's 1000)
        assert_eq!(capacities2.withdraw.reason, CapacityLimitReason::Liquidity);
    }
}
