//! Fixed-point arithmetic library for WAD-scaled calculations.
//!
//! This module implements the math library used by Morpho Blue for
//! fixed-point arithmetic with 18 decimal places (WAD = 10^18).
//!
//! # Overview
//!
//! Morpho Blue uses fixed-point arithmetic to avoid floating-point precision issues
//! on the blockchain. All values are scaled by WAD (10^18):
//!
//! ```rust
//! use morpho_rs_sim::WAD;
//! use alloy_primitives::U256;
//!
//! // Representing values
//! let one = WAD;                          // 1.0
//! let half = WAD / U256::from(2);         // 0.5
//! let ten_percent = WAD / U256::from(10); // 0.1
//! let two_point_five = U256::from(25) * WAD / U256::from(10); // 2.5
//! ```
//!
//! # Key Functions
//!
//! ## Multiplication and Division
//!
//! - [`w_mul_down`] / [`w_mul_up`]: Multiply two WAD values
//! - [`w_div_down`] / [`w_div_up`]: Divide two WAD values
//!
//! ## Share Conversion
//!
//! - [`shares_to_assets`]: Convert shares to underlying assets
//! - [`assets_to_shares`]: Convert assets to shares
//!
//! ## Interest Calculations
//!
//! - [`w_taylor_compounded`]: Taylor series for e^(rate * time) - 1
//! - [`rate_to_apy`]: Convert per-second rate to APY
//!
//! # Rounding
//!
//! Most operations have `_down` and `_up` variants for explicit rounding control:
//! - Use `_down` when calculating what users receive (conservative)
//! - Use `_up` when calculating what users owe (protective for protocol)

use alloy_primitives::U256;

/// WAD constant: 10^18, used for fixed-point arithmetic with 18 decimals
pub const WAD: U256 = U256::from_limbs([1_000_000_000_000_000_000, 0, 0, 0]);

/// Number of seconds in a year (365 days)
pub const SECONDS_PER_YEAR: u64 = 365 * 24 * 60 * 60;

/// Maximum U256 value
pub const MAX_UINT_256: U256 = U256::MAX;

/// Virtual shares constant for Morpho Blue markets
pub const VIRTUAL_SHARES: U256 = U256::from_limbs([1_000_000, 0, 0, 0]);

/// Virtual assets constant for Morpho Blue markets
pub const VIRTUAL_ASSETS: U256 = U256::from_limbs([1, 0, 0, 0]);

/// Rounding direction for arithmetic operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundingDirection {
    /// Round toward zero (floor for positive numbers)
    Down,
    /// Round away from zero (ceiling for positive numbers)
    Up,
}

/// Returns the minimum of two U256 values
#[inline]
pub fn min(a: U256, b: U256) -> U256 {
    if a <= b {
        a
    } else {
        b
    }
}

/// Returns the maximum of two U256 values
#[inline]
pub fn max(a: U256, b: U256) -> U256 {
    if a >= b {
        a
    } else {
        b
    }
}

/// Returns `a - b` floored to zero if `b > a`
#[inline]
pub fn zero_floor_sub(a: U256, b: U256) -> U256 {
    if a <= b {
        U256::ZERO
    } else {
        a - b
    }
}

/// Multiply two numbers and divide by a denominator, rounding down
///
/// Returns `(x * y) / denominator`
#[inline]
pub fn mul_div_down(x: U256, y: U256, denominator: U256) -> U256 {
    if denominator.is_zero() {
        return U256::ZERO; // Avoid panic, caller should handle
    }
    (x * y) / denominator
}

/// Multiply two numbers and divide by a denominator, rounding up
///
/// Returns `ceil((x * y) / denominator)`
#[inline]
pub fn mul_div_up(x: U256, y: U256, denominator: U256) -> U256 {
    if denominator.is_zero() {
        return U256::ZERO;
    }
    let product = x * y;
    let remainder = product % denominator;
    let result = product / denominator;
    if remainder > U256::ZERO {
        result + U256::from(1)
    } else {
        result
    }
}

/// Multiply two numbers and divide by a denominator with specified rounding
#[inline]
pub fn mul_div(x: U256, y: U256, denominator: U256, rounding: RoundingDirection) -> U256 {
    match rounding {
        RoundingDirection::Down => mul_div_down(x, y, denominator),
        RoundingDirection::Up => mul_div_up(x, y, denominator),
    }
}

/// WAD-based multiplication, rounded down
///
/// Returns `(x * y) / WAD`
#[inline]
pub fn w_mul_down(x: U256, y: U256) -> U256 {
    mul_div_down(x, y, WAD)
}

/// WAD-based multiplication, rounded up
///
/// Returns `ceil((x * y) / WAD)`
#[inline]
pub fn w_mul_up(x: U256, y: U256) -> U256 {
    mul_div_up(x, y, WAD)
}

/// WAD-based multiplication with specified rounding
#[inline]
pub fn w_mul(x: U256, y: U256, rounding: RoundingDirection) -> U256 {
    mul_div(x, y, WAD, rounding)
}

/// WAD-based division, rounded down
///
/// Returns `(x * WAD) / y`
#[inline]
pub fn w_div_down(x: U256, y: U256) -> U256 {
    mul_div_down(x, WAD, y)
}

/// WAD-based division, rounded up
///
/// Returns `ceil((x * WAD) / y)`
#[inline]
pub fn w_div_up(x: U256, y: U256) -> U256 {
    mul_div_up(x, WAD, y)
}

/// WAD-based division with specified rounding
#[inline]
pub fn w_div(x: U256, y: U256, rounding: RoundingDirection) -> U256 {
    mul_div(x, WAD, y, rounding)
}

/// Taylor series approximation of e^(x*n) - 1 for interest accrual.
///
/// Uses the first three non-zero terms of the Taylor expansion:
/// e^x - 1 ≈ x + x²/2 + x³/6
///
/// This is used by Morpho Blue to calculate continuously compounded interest.
///
/// # Arguments
/// * `x` - The per-second rate (WAD-scaled)
/// * `n` - The number of seconds elapsed
pub fn w_taylor_compounded(x: U256, n: U256) -> U256 {
    let first_term = x * n;
    let second_term = mul_div_down(first_term, first_term, U256::from(2) * WAD);
    let third_term = mul_div_down(second_term, first_term, U256::from(3) * WAD);

    first_term + second_term + third_term
}

/// Converts shares to assets using Morpho Blue's share math.
///
/// This function computes the underlying asset value of a given number of shares,
/// using virtual shares and assets to prevent share price manipulation attacks
/// (similar to ERC4626 virtual shares).
///
/// # Formula
///
/// ```text
/// assets = shares * (total_assets + VIRTUAL_ASSETS) / (total_shares + VIRTUAL_SHARES)
/// ```
///
/// The virtual amounts ensure the first depositor cannot manipulate the share price
/// by donating assets before the deposit.
///
/// # Arguments
///
/// * `shares` - Number of shares to convert
/// * `total_assets` - Total assets in the pool
/// * `total_shares` - Total shares outstanding
/// * `rounding` - [`RoundingDirection::Down`] for user receives, [`RoundingDirection::Up`] for user owes
///
/// # Returns
///
/// The asset value of the shares
///
/// # Example
///
/// ```rust
/// use morpho_rs_sim::math::{shares_to_assets, RoundingDirection};
/// use morpho_rs_sim::WAD;
/// use alloy_primitives::U256;
///
/// // With 2:1 asset:share ratio
/// let shares = U256::from(1000) * WAD;
/// let total_assets = U256::from(2000) * WAD;
/// let total_shares = U256::from(1000) * WAD;
///
/// let assets = shares_to_assets(shares, total_assets, total_shares, RoundingDirection::Down);
///
/// // 1000 shares ≈ 2000 assets (slight difference due to virtual amounts)
/// assert!(assets > U256::from(1990) * WAD);
/// ```
pub fn shares_to_assets(
    shares: U256,
    total_assets: U256,
    total_shares: U256,
    rounding: RoundingDirection,
) -> U256 {
    mul_div(
        shares,
        total_assets + VIRTUAL_ASSETS,
        total_shares + VIRTUAL_SHARES,
        rounding,
    )
}

/// Convert assets to shares using Morpho Blue's share math
///
/// Includes virtual shares/assets to prevent share manipulation attacks
pub fn assets_to_shares(
    assets: U256,
    total_assets: U256,
    total_shares: U256,
    rounding: RoundingDirection,
) -> U256 {
    mul_div(
        assets,
        total_shares + VIRTUAL_SHARES,
        total_assets + VIRTUAL_ASSETS,
        rounding,
    )
}

/// Converts a per-second rate to an Annual Percentage Yield (APY).
///
/// This function computes the effective annual return with continuous compounding,
/// converting from Morpho Blue's per-second rate format to a human-readable percentage.
///
/// # Formula
///
/// ```text
/// APY = e^(rate_per_second * seconds_per_year) - 1
/// ```
///
/// # Arguments
///
/// * `rate` - Per-second interest rate (WAD-scaled)
///
/// # Returns
///
/// APY as a decimal (e.g., 0.05 = 5% APY)
///
/// # Example
///
/// ```rust
/// use morpho_rs_sim::math::rate_to_apy;
/// use morpho_rs_sim::{WAD, SECONDS_PER_YEAR};
/// use alloy_primitives::U256;
///
/// // ~4% per-second rate (typical Morpho rate at target)
/// let rate = U256::from(1_268_391_679u64);
/// let apy = rate_to_apy(rate);
///
/// println!("APY: {:.2}%", apy * 100.0); // ~4.08%
/// assert!(apy > 0.04 && apy < 0.045);
///
/// // 0% rate gives 0% APY
/// assert_eq!(rate_to_apy(U256::ZERO), 0.0);
/// ```
pub fn rate_to_apy(rate: U256) -> f64 {
    // Convert to f64 for exponential calculation
    let rate_f64 = rate_to_f64(rate);
    let annual_rate = rate_f64 * (SECONDS_PER_YEAR as f64);
    annual_rate.exp_m1()
}

/// Convert a WAD-scaled U256 to f64
pub fn rate_to_f64(rate: U256) -> f64 {
    // Convert to string and parse as f64, then divide by WAD
    let rate_u128 = rate.saturating_to::<u128>();
    (rate_u128 as f64) / 1e18
}

/// Convert f64 to WAD-scaled U256
pub fn f64_to_wad(value: f64) -> U256 {
    let wad_value = value * 1e18;
    U256::from(wad_value as u128)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wad_constant() {
        assert_eq!(WAD, U256::from(1_000_000_000_000_000_000u64));
    }

    #[test]
    fn test_w_mul_down() {
        let a = U256::from(2) * WAD; // 2.0
        let b = U256::from(3) * WAD; // 3.0
        let result = w_mul_down(a, b);
        assert_eq!(result, U256::from(6) * WAD); // 6.0
    }

    #[test]
    fn test_w_div_down() {
        let a = U256::from(6) * WAD; // 6.0
        let b = U256::from(2) * WAD; // 2.0
        let result = w_div_down(a, b);
        assert_eq!(result, U256::from(3) * WAD); // 3.0
    }

    #[test]
    fn test_zero_floor_sub() {
        assert_eq!(zero_floor_sub(U256::from(5), U256::from(3)), U256::from(2));
        assert_eq!(zero_floor_sub(U256::from(3), U256::from(5)), U256::ZERO);
        assert_eq!(zero_floor_sub(U256::from(5), U256::from(5)), U256::ZERO);
    }

    #[test]
    fn test_taylor_compounded() {
        // Test Taylor series approximation for interest accrual
        // For a rate that gives ~5% when compounded over a year
        // rate_per_second * seconds_per_year should give ~0.05 WAD
        let rate_per_second = U256::from(1_585_489_599u64);
        let one_year = U256::from(SECONDS_PER_YEAR);
        let result = w_taylor_compounded(rate_per_second, one_year);

        // Taylor series: x + x²/2 + x³/6 ≈ 0.05 + 0.00125 + ... ≈ 0.0513
        // The result should be close to 5% (within a few percent due to higher-order terms)
        let result_f64 = rate_to_f64(result);
        assert!(result_f64 > 0.04 && result_f64 < 0.06);
    }

    #[test]
    fn test_rate_to_apy() {
        // rate_to_apy uses exp(rate * year) - 1
        // For rate_per_second that gives 5% annual rate:
        // APY = e^0.05 - 1 ≈ 0.0512 (5.12%)
        let rate_per_second = U256::from(1_585_489_599u64);
        let apy = rate_to_apy(rate_per_second);
        // Should be approximately 5% APY (with slight compounding effect)
        assert!(apy > 0.04 && apy < 0.06);
    }

    #[test]
    fn test_shares_to_assets() {
        let shares = U256::from(1000) * WAD;
        let total_assets = U256::from(2000) * WAD;
        let total_shares = U256::from(1000) * WAD;

        let assets = shares_to_assets(shares, total_assets, total_shares, RoundingDirection::Down);
        // With 2:1 ratio, 1000 shares should give ~2000 assets (minus virtual adjustment)
        assert!(assets > U256::from(1990) * WAD);
    }

    // ==================== New Tests ====================

    #[test]
    fn test_rate_to_apy_specific_values() {
        // Test cases from TypeScript MarketUtils.test.ts
        // 3% annual rate -> ~3.045% APY
        let rate_3_pct = U256::from(30_000_000_000_000_000u64) / U256::from(SECONDS_PER_YEAR);
        let apy_3 = rate_to_apy(rate_3_pct);
        assert!((apy_3 - 0.030454533936848223).abs() < 0.001);

        // 40% annual rate -> ~49.18% APY
        let rate_40_pct = U256::from(400_000_000_000_000_000u64) / U256::from(SECONDS_PER_YEAR);
        let apy_40 = rate_to_apy(rate_40_pct);
        assert!((apy_40 - 0.4918246976174727).abs() < 0.01);

        // 500% annual rate -> ~14741% APY (extreme case)
        let rate_500_pct = U256::from(5_000_000_000_000_000_000u64) / U256::from(SECONDS_PER_YEAR);
        let apy_500 = rate_to_apy(rate_500_pct);
        assert!(apy_500 > 100.0); // Should be very high
    }

    #[test]
    fn test_min_max() {
        let a = U256::from(5);
        let b = U256::from(10);

        assert_eq!(min(a, b), a);
        assert_eq!(min(b, a), a);
        assert_eq!(min(a, a), a);

        assert_eq!(max(a, b), b);
        assert_eq!(max(b, a), b);
        assert_eq!(max(a, a), a);
    }

    #[test]
    fn test_mul_div_rounding() {
        // Test that up rounds up and down rounds down
        let x = U256::from(10);
        let y = U256::from(3);
        let d = U256::from(7);

        // 10 * 3 / 7 = 30 / 7 = 4.28...
        let down = mul_div_down(x, y, d);
        let up = mul_div_up(x, y, d);

        assert_eq!(down, U256::from(4)); // Floor
        assert_eq!(up, U256::from(5)); // Ceiling
    }

    #[test]
    fn test_assets_to_shares() {
        let assets = U256::from(1000) * WAD;
        let total_assets = U256::from(2000) * WAD;
        let total_shares = U256::from(1000) * WAD;

        let shares = assets_to_shares(assets, total_assets, total_shares, RoundingDirection::Down);
        // With 1:2 ratio (more assets than shares), 1000 assets should give ~500 shares
        assert!(shares > U256::from(490) * WAD && shares < U256::from(510) * WAD);
    }

    #[test]
    fn test_f64_to_wad() {
        let value = 1.5;
        let wad_value = f64_to_wad(value);
        assert_eq!(wad_value, U256::from(1_500_000_000_000_000_000u64));

        let back = rate_to_f64(wad_value);
        assert!((back - value).abs() < 0.0001);
    }

    #[test]
    fn test_mul_div_zero_denominator() {
        // Should return zero instead of panic
        let result = mul_div_down(U256::from(10), U256::from(5), U256::ZERO);
        assert_eq!(result, U256::ZERO);

        let result_up = mul_div_up(U256::from(10), U256::from(5), U256::ZERO);
        assert_eq!(result_up, U256::ZERO);
    }

    #[test]
    fn test_w_mul_up() {
        // Test that w_mul_up rounds up
        let a = WAD + U256::from(1); // Slightly more than 1.0
        let b = WAD + U256::from(1); // Slightly more than 1.0

        let down = w_mul_down(a, b);
        let up = w_mul_up(a, b);

        // up should be >= down
        assert!(up >= down);
    }

    #[test]
    fn test_w_div_up() {
        // Test that w_div_up rounds up
        let a = WAD + U256::from(1); // Slightly more than 1.0
        let b = WAD + U256::from(1); // Slightly more than 1.0

        let down = w_div_down(a, b);
        let up = w_div_up(a, b);

        // up should be >= down
        assert!(up >= down);
    }

    #[test]
    fn test_taylor_compounded_zero() {
        // Zero rate should give zero interest
        let result = w_taylor_compounded(U256::ZERO, U256::from(SECONDS_PER_YEAR));
        assert_eq!(result, U256::ZERO);

        // Zero time should give zero interest
        let result2 = w_taylor_compounded(U256::from(1_000_000_000u64), U256::ZERO);
        assert_eq!(result2, U256::ZERO);
    }

    #[test]
    fn test_shares_to_assets_empty_pool() {
        // Empty pool (first deposit scenario)
        let shares = U256::from(1000) * WAD;
        let total_assets = U256::ZERO;
        let total_shares = U256::ZERO;

        let assets = shares_to_assets(shares, total_assets, total_shares, RoundingDirection::Down);
        // With virtual shares/assets, should still work
        // Result = shares * (0 + 1) / (0 + 1e6) ≈ shares / 1e6
        assert!(assets > U256::ZERO);
    }
}
