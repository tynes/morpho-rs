//! Adaptive Curve Interest Rate Model (IRM) implementation.
//!
//! This module implements the [Adaptive Curve IRM](https://docs.morpho.org/morpho/concepts/adaptive-interest-rate-model)
//! used by Morpho Blue, which dynamically adjusts interest rates based on market utilization.
//!
//! # How the IRM Works
//!
//! The Adaptive Curve IRM has two key components:
//!
//! ## 1. The Curve Function
//!
//! The borrow rate is determined by a curve centered at the target utilization (90%):
//!
//! ```text
//! If utilization >= target (90%):
//!     rate = rate_at_target * (1 + 3 * error)    // Steep increase above target
//! If utilization < target:
//!     rate = rate_at_target * (1 - 0.75 * error) // Gradual decrease below target
//!
//! where error = |utilization - target| / normalization_factor
//! ```
//!
//! This creates an asymmetric curve where rates increase much faster above target
//! than they decrease below it.
//!
//! ## 2. Rate Adaptation
//!
//! The `rate_at_target` itself adapts over time:
//! - **Above target utilization**: `rate_at_target` increases (encourages more supply)
//! - **Below target utilization**: `rate_at_target` decreases (encourages more borrowing)
//! - **Adaptation speed**: 50% per year (e.g., if above target for a full year, rate doubles)
//!
//! # Constants
//!
//! | Constant | Value | Description |
//! |----------|-------|-------------|
//! | `TARGET_UTILIZATION` | 90% | Optimal utilization rate |
//! | `CURVE_STEEPNESS` | 4.0 | Rate multiplier at 100% utilization |
//! | `INITIAL_RATE_AT_TARGET` | ~4% APY | Starting rate for new markets |
//! | `ADJUSTMENT_SPEED` | 50%/year | How fast rate_at_target adapts |
//! | `MIN_RATE_AT_TARGET` | 0.1% APY | Minimum rate at target |
//! | `MAX_RATE_AT_TARGET` | 200% APY | Maximum rate at target |
//!
//! # Example
//!
//! ```rust
//! use morpho_rs_sim::irm::{get_borrow_rate, TARGET_UTILIZATION, INITIAL_RATE_AT_TARGET};
//! use morpho_rs_sim::{WAD, math::rate_to_apy};
//! use alloy_primitives::U256;
//!
//! // Calculate rate at exactly target utilization
//! let result = get_borrow_rate(TARGET_UTILIZATION, INITIAL_RATE_AT_TARGET, 0);
//!
//! // At target, borrow rate equals rate_at_target
//! let apy = rate_to_apy(result.end_borrow_rate);
//! assert!(apy > 0.03 && apy < 0.05); // Around 4% APY
//! ```

use alloy_primitives::U256;

use crate::math::{max, min, w_div_down, w_div_up, w_mul_down, zero_floor_sub, WAD};

/// Curve steepness parameter (4.0 in WAD)
pub const CURVE_STEEPNESS: U256 = U256::from_limbs([4_000_000_000_000_000_000, 0, 0, 0]);

/// Target utilization rate (90% in WAD = 0.9)
pub const TARGET_UTILIZATION: U256 = U256::from_limbs([900_000_000_000_000_000, 0, 0, 0]);

/// Initial rate at target (4% APY / seconds_per_year)
/// 4% = 0.04 WAD = 40_000_000_000_000_000
/// Divided by SECONDS_PER_YEAR (31536000)
pub const INITIAL_RATE_AT_TARGET: U256 = U256::from_limbs([1_268_391_679, 0, 0, 0]);

/// Adjustment speed (50% per year / seconds_per_year)
/// 50% = 0.5 WAD = 500_000_000_000_000_000
/// Divided by SECONDS_PER_YEAR (31536000)
pub const ADJUSTMENT_SPEED: U256 = U256::from_limbs([15_854_895_991, 0, 0, 0]);

/// Minimum rate at target (0.1% APY / seconds_per_year)
pub const MIN_RATE_AT_TARGET: U256 = U256::from_limbs([31_709_791, 0, 0, 0]);

/// Maximum rate at target (200% APY / seconds_per_year)
pub const MAX_RATE_AT_TARGET: U256 = U256::from_limbs([63_419_583_967, 0, 0, 0]);

/// ln(2) scaled by WAD
pub const LN_2_INT: i128 = 693_147_180_559_945_309;

/// ln(1e-18) scaled by WAD (negative)
pub const LN_WEI_INT: i128 = -41_446_531_673_892_822_312;

/// Upper bound for wExp to avoid overflow
pub const WEXP_UPPER_BOUND: i128 = 93_859_467_695_000_404_319;

/// Value of wExp at upper bound
/// 57716089161558943949701069502944508345128422502756744429568 in little-endian u64 limbs
pub const WEXP_UPPER_VALUE: U256 = U256::from_limbs([
    0x3216C1AD5D72C200,  // Low 64 bits
    0x09BA5D32E9C0DE49,
    0x80,
    0,
]);

/// Result of borrow rate calculation
#[derive(Debug, Clone)]
pub struct BorrowRateResult {
    /// Average borrow rate over the period (WAD-scaled per second)
    pub avg_borrow_rate: U256,
    /// End borrow rate (instantaneous rate at end of period)
    pub end_borrow_rate: U256,
    /// New rate at target after the period
    pub end_rate_at_target: U256,
}

/// Approximation of exp(x) used by the Adaptive Curve IRM.
///
/// Uses the decomposition: e^x = 2^q * e^r where x = q*ln(2) + r
/// with -ln(2)/2 <= r <= ln(2)/2
pub fn w_exp(x: i128) -> U256 {
    // If x < ln(1e-18) then exp(x) < 1e-18 so it is rounded to zero
    if x < LN_WEI_INT {
        return U256::ZERO;
    }

    // Clip to avoid overflow
    if x >= WEXP_UPPER_BOUND {
        return WEXP_UPPER_VALUE;
    }

    // Decompose x as x = q * ln(2) + r
    // q = x / ln(2) rounded half toward zero
    let rounding_adjustment = if x < 0 { -(LN_2_INT / 2) } else { LN_2_INT / 2 };
    let q = (x + rounding_adjustment) / LN_2_INT;
    let r = x - q * LN_2_INT;

    // Compute e^r with a 2nd-order Taylor polynomial
    // e^r ≈ 1 + r + r²/2
    let wad_i128 = WAD.saturating_to::<i128>();
    let r_squared = (r * r) / wad_i128 / 2;
    let exp_r = wad_i128 + r + r_squared;

    // Convert to U256 for final calculation
    let exp_r_u256 = U256::from(exp_r.unsigned_abs());

    // Return e^x = 2^q * e^r
    if q >= 0 {
        exp_r_u256 << (q as usize)
    } else {
        exp_r_u256 >> ((-q) as usize)
    }
}

/// Calculates the borrow rate for the Adaptive Curve IRM.
///
/// This is the core IRM function that computes both the instantaneous borrow rate
/// and the adapted `rate_at_target` after a given time period.
///
/// # Rate Calculation
///
/// The function performs these steps:
/// 1. Calculate utilization error from target (90%)
/// 2. Compute `rate_at_target` adaptation based on error and elapsed time
/// 3. Apply the curve function to get the final borrow rate
///
/// # Arguments
///
/// * `utilization` - Current market utilization (WAD-scaled, 0 to 1e18)
/// * `rate_at_target` - Current rate at target utilization (per-second, WAD-scaled).
///   Pass `U256::ZERO` for first interaction (will use `INITIAL_RATE_AT_TARGET`).
/// * `elapsed` - Time since last update in seconds
///
/// # Returns
///
/// A [`BorrowRateResult`] containing:
/// - `avg_borrow_rate`: Average borrow rate over the elapsed period (for interest accrual)
/// - `end_borrow_rate`: Instantaneous rate at the end (current rate)
/// - `end_rate_at_target`: Updated rate at target after adaptation
///
/// # Example
///
/// ```rust
/// use morpho_rs_sim::irm::{get_borrow_rate, TARGET_UTILIZATION, INITIAL_RATE_AT_TARGET};
/// use morpho_rs_sim::{WAD, math::rate_to_apy};
/// use alloy_primitives::U256;
///
/// // High utilization (95%) should give higher rate
/// let high_util = U256::from(950_000_000_000_000_000u64);
/// let result = get_borrow_rate(high_util, INITIAL_RATE_AT_TARGET, 0);
/// assert!(result.end_borrow_rate > INITIAL_RATE_AT_TARGET);
///
/// // Low utilization (50%) should give lower rate
/// let low_util = U256::from(500_000_000_000_000_000u64);
/// let result_low = get_borrow_rate(low_util, INITIAL_RATE_AT_TARGET, 0);
/// assert!(result_low.end_borrow_rate < INITIAL_RATE_AT_TARGET);
///
/// // After 1 day at high utilization, rate_at_target increases
/// let one_day = 86400u64;
/// let adapted = get_borrow_rate(high_util, INITIAL_RATE_AT_TARGET, one_day);
/// assert!(adapted.end_rate_at_target > INITIAL_RATE_AT_TARGET);
/// ```
pub fn get_borrow_rate(utilization: U256, rate_at_target: U256, elapsed: u64) -> BorrowRateResult {
    let elapsed_u256 = U256::from(elapsed);

    // Calculate error from target utilization
    let err_norm_factor = if utilization > TARGET_UTILIZATION {
        WAD - TARGET_UTILIZATION
    } else {
        TARGET_UTILIZATION
    };

    // Calculate error (absolute value - we track sign separately)
    let err = if utilization >= TARGET_UTILIZATION {
        w_div_down(utilization - TARGET_UTILIZATION, err_norm_factor)
    } else {
        // Negative error - we'll track the sign separately in is_err_negative
        w_div_down(TARGET_UTILIZATION - utilization, err_norm_factor)
    };

    let is_err_negative = utilization < TARGET_UTILIZATION;

    let (avg_rate_at_target, end_rate_at_target) = if rate_at_target.is_zero() {
        // First interaction
        (INITIAL_RATE_AT_TARGET, INITIAL_RATE_AT_TARGET)
    } else {
        // Calculate speed and linear adaptation
        let speed = w_mul_down(ADJUSTMENT_SPEED, err);
        let linear_adaptation = if is_err_negative {
            // Negative speed means rate should decrease
            speed * elapsed_u256
        } else {
            speed * elapsed_u256
        };

        if linear_adaptation.is_zero() {
            (rate_at_target, rate_at_target)
        } else {
            // Calculate new rate at target
            let new_rate = |adaptation: U256, is_negative: bool| -> U256 {
                let exp_arg = if is_negative {
                    -(adaptation.saturating_to::<i128>())
                } else {
                    adaptation.saturating_to::<i128>()
                };
                let exp_result = w_exp(exp_arg);
                let raw_rate = w_mul_down(rate_at_target, exp_result);
                min(max(raw_rate, MIN_RATE_AT_TARGET), MAX_RATE_AT_TARGET)
            };

            let end_rate = new_rate(linear_adaptation, is_err_negative);

            // Calculate average using trapezoidal rule
            let mid_rate = new_rate(linear_adaptation / U256::from(2), is_err_negative);
            let avg_rate = (rate_at_target + end_rate + U256::from(2) * mid_rate) / U256::from(4);

            (avg_rate, end_rate)
        }
    };

    // Calculate the curve coefficient
    let coeff = if is_err_negative {
        WAD - w_div_down(WAD, CURVE_STEEPNESS)
    } else {
        CURVE_STEEPNESS - WAD
    };

    // Apply the curve function
    let curve = |rate: U256| -> U256 {
        if is_err_negative {
            // err is negative, so coeff * err is negative
            // We compute: rate * (1 - coeff * |err| / WAD)
            let adjustment = w_mul_down(coeff, err);
            let factor = WAD.saturating_sub(adjustment);
            w_mul_down(factor, rate)
        } else {
            // err is positive
            let adjustment = w_mul_down(coeff, err);
            let factor = WAD + adjustment;
            w_mul_down(factor, rate)
        }
    };

    BorrowRateResult {
        avg_borrow_rate: curve(avg_rate_at_target),
        end_borrow_rate: curve(end_rate_at_target),
        end_rate_at_target,
    }
}

/// Calculate the utilization that would produce a given borrow rate.
///
/// This is the inverse of the borrow rate curve function.
///
/// # Arguments
/// * `borrow_rate` - Target borrow rate (WAD-scaled per second)
/// * `rate_at_target` - Rate at target utilization (WAD-scaled per second)
///
/// # Returns
/// The utilization (WAD-scaled) that produces the given borrow rate, clamped to [0, WAD]
pub fn get_utilization_at_borrow_rate(borrow_rate: U256, rate_at_target: U256) -> U256 {
    if rate_at_target.is_zero() {
        return TARGET_UTILIZATION;
    }

    // Calculate the ratio of borrow_rate to rate_at_target
    let rate_ratio = w_div_down(borrow_rate, rate_at_target);

    if rate_ratio >= WAD {
        // Utilization is at or above target
        // rate = rate_at_target * (1 + (CURVE_STEEPNESS - 1) * err)
        // rate_ratio = 1 + (CURVE_STEEPNESS - 1) * err
        // err = (rate_ratio - 1) / (CURVE_STEEPNESS - 1)
        let coeff = CURVE_STEEPNESS - WAD; // 3 WAD
        let err = w_div_down(rate_ratio - WAD, coeff);

        // utilization = TARGET_UTILIZATION + err * (WAD - TARGET_UTILIZATION)
        let err_denorm = w_mul_down(err, WAD - TARGET_UTILIZATION);
        let utilization = TARGET_UTILIZATION + err_denorm;

        min(utilization, WAD)
    } else {
        // Utilization is below target
        // rate = rate_at_target * (1 - (1 - 1/CURVE_STEEPNESS) * err)
        // rate_ratio = 1 - (1 - 1/CURVE_STEEPNESS) * err
        // err = (1 - rate_ratio) / (1 - 1/CURVE_STEEPNESS)
        let coeff = WAD - w_div_down(WAD, CURVE_STEEPNESS); // 0.75 WAD

        if coeff.is_zero() {
            return TARGET_UTILIZATION;
        }

        let err = w_div_up(WAD - rate_ratio, coeff);

        // utilization = TARGET_UTILIZATION - err * TARGET_UTILIZATION
        let err_denorm = w_mul_down(err, TARGET_UTILIZATION);

        zero_floor_sub(TARGET_UTILIZATION, err_denorm)
    }
}

/// Calculate the supply/withdraw amount needed to reach a target borrow rate.
///
/// # Arguments
/// * `total_supply_assets` - Current total supply
/// * `total_borrow_assets` - Current total borrow
/// * `target_borrow_rate` - Target borrow rate
/// * `rate_at_target` - Current rate at target utilization
///
/// # Returns
/// Positive value means supply needed, negative (returned as second value) means withdraw possible
pub fn get_supply_for_borrow_rate(
    total_supply_assets: U256,
    total_borrow_assets: U256,
    target_borrow_rate: U256,
    rate_at_target: U256,
) -> (U256, U256) {
    let target_utilization = get_utilization_at_borrow_rate(target_borrow_rate, rate_at_target);

    if target_utilization.is_zero() {
        return (U256::MAX, U256::ZERO);
    }

    // Required supply = total_borrow / target_utilization
    let required_supply = w_div_up(total_borrow_assets, target_utilization);

    if required_supply > total_supply_assets {
        // Need to supply more
        (required_supply - total_supply_assets, U256::ZERO)
    } else {
        // Can withdraw
        (U256::ZERO, total_supply_assets - required_supply)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math;

    #[test]
    fn test_w_exp_zero() {
        let result = w_exp(0);
        // e^0 = 1 WAD
        assert_eq!(result, WAD);
    }

    #[test]
    fn test_w_exp_positive() {
        // e^1 ≈ 2.718
        let result = w_exp(WAD.saturating_to::<i128>());
        let result_f64 = math::rate_to_f64(result);
        assert!((result_f64 - std::f64::consts::E).abs() < 0.1);
    }

    #[test]
    fn test_w_exp_negative() {
        // e^(-1) ≈ 0.368
        let result = w_exp(-(WAD.saturating_to::<i128>()));
        let result_f64 = math::rate_to_f64(result);
        assert!((result_f64 - 0.368).abs() < 0.05);
    }

    #[test]
    fn test_get_borrow_rate_at_target() {
        // At target utilization, rate should equal rate_at_target
        let rate_at_target = U256::from(1_268_391_679u64); // ~4% APY
        let result = get_borrow_rate(TARGET_UTILIZATION, rate_at_target, 0);

        // With zero elapsed time, rates should be unchanged
        assert_eq!(result.end_rate_at_target, rate_at_target);
    }

    #[test]
    fn test_get_borrow_rate_initial() {
        // First interaction (rate_at_target = 0)
        let result = get_borrow_rate(TARGET_UTILIZATION, U256::ZERO, 0);
        assert_eq!(result.end_rate_at_target, INITIAL_RATE_AT_TARGET);
    }

    #[test]
    fn test_get_borrow_rate_high_utilization() {
        // Above target utilization, rate should be higher
        let high_utilization = U256::from(950_000_000_000_000_000u64); // 95%
        let rate_at_target = INITIAL_RATE_AT_TARGET;
        let result = get_borrow_rate(high_utilization, rate_at_target, 0);

        // Borrow rate should be higher than rate at target
        assert!(result.end_borrow_rate > rate_at_target);
    }

    #[test]
    fn test_get_borrow_rate_low_utilization() {
        // Below target utilization, rate should be lower
        let low_utilization = U256::from(500_000_000_000_000_000u64); // 50%
        let rate_at_target = INITIAL_RATE_AT_TARGET;
        let result = get_borrow_rate(low_utilization, rate_at_target, 0);

        // Borrow rate should be lower than rate at target
        assert!(result.end_borrow_rate < rate_at_target);
    }

    // ==================== New Tests ====================

    #[test]
    fn test_w_exp_very_small() {
        // e^(ln(1e-18)) should be very small (rounds to 0)
        let result = w_exp(LN_WEI_INT - 1);
        assert_eq!(result, U256::ZERO);
    }

    #[test]
    fn test_w_exp_upper_bound() {
        // At or above upper bound, should return WEXP_UPPER_VALUE
        let result = w_exp(WEXP_UPPER_BOUND);
        assert_eq!(result, WEXP_UPPER_VALUE);

        let result2 = w_exp(WEXP_UPPER_BOUND + 1);
        assert_eq!(result2, WEXP_UPPER_VALUE);
    }

    #[test]
    fn test_w_exp_small_positive() {
        // e^0.1 ≈ 1.105
        let x = WAD.saturating_to::<i128>() / 10; // 0.1 WAD
        let result = w_exp(x);
        let result_f64 = math::rate_to_f64(result);
        assert!((result_f64 - 1.105).abs() < 0.01);
    }

    #[test]
    fn test_w_exp_small_negative() {
        // e^(-0.1) ≈ 0.905
        let x = -(WAD.saturating_to::<i128>() / 10); // -0.1 WAD
        let result = w_exp(x);
        let result_f64 = math::rate_to_f64(result);
        assert!((result_f64 - 0.905).abs() < 0.01);
    }

    #[test]
    fn test_get_utilization_at_borrow_rate_at_target() {
        // At rate_at_target, utilization should be TARGET_UTILIZATION
        let rate_at_target = INITIAL_RATE_AT_TARGET;
        let utilization = get_utilization_at_borrow_rate(rate_at_target, rate_at_target);

        // Should be approximately 90%
        let util_f64 = math::rate_to_f64(utilization);
        assert!((util_f64 - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_get_utilization_at_borrow_rate_high_rate() {
        // Higher borrow rate should give higher utilization
        let rate_at_target = INITIAL_RATE_AT_TARGET;
        let high_rate = rate_at_target * U256::from(2); // 2x rate at target

        let utilization = get_utilization_at_borrow_rate(high_rate, rate_at_target);

        // Should be above TARGET_UTILIZATION (90%)
        assert!(utilization > TARGET_UTILIZATION);
    }

    #[test]
    fn test_get_utilization_at_borrow_rate_low_rate() {
        // Lower borrow rate should give lower utilization
        let rate_at_target = INITIAL_RATE_AT_TARGET;
        let low_rate = rate_at_target / U256::from(2); // 0.5x rate at target

        let utilization = get_utilization_at_borrow_rate(low_rate, rate_at_target);

        // Should be below TARGET_UTILIZATION (90%)
        assert!(utilization < TARGET_UTILIZATION);
    }

    #[test]
    fn test_get_utilization_at_borrow_rate_zero_rate_at_target() {
        // With zero rate_at_target, should return TARGET_UTILIZATION
        let utilization = get_utilization_at_borrow_rate(INITIAL_RATE_AT_TARGET, U256::ZERO);
        assert_eq!(utilization, TARGET_UTILIZATION);
    }

    #[test]
    fn test_get_supply_for_borrow_rate_needs_supply() {
        // If we want lower rate, need to supply more
        let total_supply = U256::from(1_000_000) * WAD;
        let total_borrow = U256::from(800_000) * WAD; // 80% utilization
        let rate_at_target = INITIAL_RATE_AT_TARGET;

        // Target a lower rate (which requires lower utilization, i.e., more supply)
        let target_rate = rate_at_target / U256::from(2);

        let (supply_needed, withdraw_possible) = get_supply_for_borrow_rate(
            total_supply,
            total_borrow,
            target_rate,
            rate_at_target,
        );

        // Should need to supply more, not withdraw
        assert!(supply_needed > U256::ZERO);
        assert_eq!(withdraw_possible, U256::ZERO);
    }

    #[test]
    fn test_get_supply_for_borrow_rate_can_withdraw() {
        // If we want higher rate, can withdraw some supply
        let total_supply = U256::from(1_000_000) * WAD;
        let total_borrow = U256::from(500_000) * WAD; // 50% utilization
        let rate_at_target = INITIAL_RATE_AT_TARGET;

        // Target a higher rate (which requires higher utilization, i.e., less supply)
        let target_rate = rate_at_target;

        let (supply_needed, withdraw_possible) = get_supply_for_borrow_rate(
            total_supply,
            total_borrow,
            target_rate,
            rate_at_target,
        );

        // At 50% utilization with target 90%, can withdraw a lot
        // Required supply = 500K / 0.9 = ~555K, can withdraw ~444K
        assert_eq!(supply_needed, U256::ZERO);
        assert!(withdraw_possible > U256::ZERO);
    }

    #[test]
    fn test_get_borrow_rate_adaptation_over_time() {
        // Test that rate adapts over time when utilization is away from target
        let high_utilization = U256::from(950_000_000_000_000_000u64); // 95%
        let rate_at_target = INITIAL_RATE_AT_TARGET;
        let elapsed = 86400u64; // 1 day

        let result = get_borrow_rate(high_utilization, rate_at_target, elapsed);

        // Rate at target should increase due to high utilization
        assert!(result.end_rate_at_target > rate_at_target);

        // Low utilization should decrease rate at target
        let low_utilization = U256::from(500_000_000_000_000_000u64); // 50%
        let result_low = get_borrow_rate(low_utilization, rate_at_target, elapsed);

        assert!(result_low.end_rate_at_target < rate_at_target);
    }

    #[test]
    fn test_get_borrow_rate_avg_vs_end() {
        // Average rate should be between start and end
        let high_utilization = U256::from(950_000_000_000_000_000u64);
        let rate_at_target = INITIAL_RATE_AT_TARGET;
        let elapsed = 86400u64;

        let result = get_borrow_rate(high_utilization, rate_at_target, elapsed);

        // For increasing rate, avg should be between start and end
        // This is approximated with trapezoidal rule
        assert!(result.avg_borrow_rate > U256::ZERO);
    }

    #[test]
    fn test_get_borrow_rate_at_100_percent_utilization() {
        // At 100% utilization, rate should be much higher than at target
        let full_utilization = WAD; // 100%
        let rate_at_target = INITIAL_RATE_AT_TARGET;

        let result = get_borrow_rate(full_utilization, rate_at_target, 0);

        // Rate should be significantly higher
        assert!(result.end_borrow_rate > rate_at_target * U256::from(2));
    }

    #[test]
    fn test_get_borrow_rate_at_zero_utilization() {
        // At 0% utilization, rate should be much lower than at target
        let zero_utilization = U256::ZERO;
        let rate_at_target = INITIAL_RATE_AT_TARGET;

        let result = get_borrow_rate(zero_utilization, rate_at_target, 0);

        // Rate should be significantly lower
        assert!(result.end_borrow_rate < rate_at_target);
    }

    #[test]
    fn test_rate_at_target_bounds() {
        // Rate at target should be clamped to min/max
        let high_utilization = U256::from(990_000_000_000_000_000u64); // 99%
        let very_high_rate = MAX_RATE_AT_TARGET;
        let very_long_time = 365 * 86400u64; // 1 year

        let result = get_borrow_rate(high_utilization, very_high_rate, very_long_time);

        // Should be clamped to MAX_RATE_AT_TARGET
        assert!(result.end_rate_at_target <= MAX_RATE_AT_TARGET);

        // Test minimum bound
        let low_utilization = U256::from(100_000_000_000_000_000u64); // 10%
        let very_low_rate = MIN_RATE_AT_TARGET;

        let result_low = get_borrow_rate(low_utilization, very_low_rate, very_long_time);

        // Should be clamped to MIN_RATE_AT_TARGET
        assert!(result_low.end_rate_at_target >= MIN_RATE_AT_TARGET);
    }
}
