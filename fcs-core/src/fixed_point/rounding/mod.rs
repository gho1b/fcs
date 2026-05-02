//! Rounding utilities for fixed-point arithmetic.
//!
//! This module provides:
//! - [`RoundingMode`]: rounding policy (nearest, ties handling, directed rounding).
//! - `checked_round_i64` / `checked_round_i28`: integer rounding helpers used by fixed-point ops.
//!
//! ## Semantics
//! Given `number / denom`, we first compute the Euclidean quotient/remainder:
//! `number = q * denom + r` with `0 <= r < abs(denom)`.
//! Rounding chooses between `q` (lower) and `q + 1` (upper).
//!
//! ## Failure (`None`)
//! Returned when the division itself is invalid or would overflow, e.g.:
//! - `denom == 0`
//! - `abs(denom)` is not representable (`denom == MIN` for signed ints)
//! - `number == MIN && denom == -1` (division overflow)
//! - `q + 1` overflows in the chosen rounding direction
//!
//! ## Examples
//! ```rust,ignore
//! // `checked_round_*` helpers are `pub(crate)` and are intended to be used via the fixed-point API.
//! use fcs_core::fixed_point::rounding::{RoundingMode, checked_round_i64};
//! assert_eq!(checked_round_i64(25, 10, RoundingMode::HalfEven), Some(2)); // 2.5 -> 2
//! assert_eq!(checked_round_i64(25, 10, RoundingMode::HalfUp), Some(3));   // 2.5 -> 3
//! assert_eq!(checked_round_i64(-25, 10, RoundingMode::HalfUp), Some(-3)); // -2.5 -> -3
//! ```

use crate::fixed_point::helper::{
    checked_div_rem_euclid_signed_i128, checked_div_rem_euclid_signed_i64,
};
use std::cmp::Ordering;

/// Rounding policy for converting `number / denom` into an integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum RoundingMode {
    /// Round to nearest; ties go to the even result.
    ///
    /// Often called banker's rounding (unbiased rounding).
    #[default]
    HalfEven,
    /// Round to nearest; ties go away from zero.
    ///
    /// Examples:
    /// - `2.5 -> 3`
    /// - `-2.5 -> -3`
    HalfUp,
    /// Round to nearest; ties go toward zero.
    ///
    /// ToNearestTiesTowardZero
    ///
    /// Examples:
    /// - `2.5 -> 2`
    /// - `-2.5 -> -2`
    HalfDown,
    /// Round toward negative infinity.
    ///
    /// TowardNegativeInfinity
    Floor,
    /// Round toward positive infinity.
    ///
    /// TowardPositiveInfinity
    Ceil,
    /// Round toward zero.
    TowardZero,
    /// Round away from zero.
    AwayFromZero,
}

macro_rules! fn_checked_round {
    ($fn_name: ident, $t: ty, $div_rem: ident) => {
        pub(crate) fn $fn_name(number: $t, denom: $t, rounding_mode: RoundingMode) -> Option<$t> {
            // `denom == 0`, `abs(denom)` overflow, and division overflow are handled by `$div_rem`.
            let (q, r, p_denom) = $div_rem(number, denom)?;

            // Rounding unneeded
            if r == 0 {
                return Some(q);
            }

            let is_negative = q.is_negative();

            let choose_upper = match rounding_mode {
                RoundingMode::Floor => false,
                RoundingMode::Ceil => true,
                RoundingMode::TowardZero => is_negative,
                RoundingMode::AwayFromZero => !is_negative,
                RoundingMode::HalfEven | RoundingMode::HalfUp | RoundingMode::HalfDown => {
                    match r.cmp(&(p_denom - r)) {
                        Ordering::Greater => true,
                        Ordering::Less => false,
                        Ordering::Equal => match rounding_mode {
                            // Tie to even between q and q + 1.
                            // If q is odd, q + 1 is even.
                            RoundingMode::HalfEven => q & 1 != 0,

                            // Java BigDecimal-style HALF_UP:
                            // tie away from zero.
                            //
                            // positive: choose upper
                            // negative: choose lower
                            RoundingMode::HalfUp => !is_negative,

                            // Java BigDecimal-style HALF_DOWN:
                            // tie toward zero.
                            //
                            // positive: choose lower
                            // negative: choose upper
                            RoundingMode::HalfDown => is_negative,
                            _ => unreachable!(),
                        },
                    }
                }
            };

            if choose_upper {
                Some(q.checked_add(1)?)
            } else {
                Some(q)
            }
        }
    };
}

fn_checked_round!(checked_round_i64, i64, checked_div_rem_euclid_signed_i64);
// NOTE: Despite its name, `checked_round_i28` rounds `i128` values.
// The name is kept to avoid churn in call sites.
fn_checked_round!(checked_round_i28, i128, checked_div_rem_euclid_signed_i128);

#[cfg(test)]
mod tests;
