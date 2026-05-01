use crate::error::FixedPointError;

use super::{FixedPoint, RoundingMode};

#[must_use]
#[inline]
pub(super) fn checked_div_rem_euclid_signed(a: i64, b: i64) -> Option<(i64, i64, i64)> {
    if b == 0 {
        return None;
    }
    if b == i64::MIN {
        return None;
    }

    let div = b.abs();
    let q = a.checked_div(div)?;
    let r = a - q.checked_mul(div)?;
    let (q, r) = if r >= 0 {
        (q, r)
    } else {
        (q.checked_sub(1)?, r.checked_add(div)?)
    };

    if b > 0 {
        Some((q, r, div))
    } else if r == 0 {
        Some((q.checked_neg()?, 0, div))
    } else {
        let q2 = q.checked_neg()?.checked_sub(1)?;
        let r2 = div.checked_sub(r)?;
        Some((q2, r2, div))
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub struct DivResult {
    pub(crate) quotient: FixedPoint,
    pub(crate) rem: i64,
    pub(crate) div: i64,
}

impl DivResult {
    pub fn try_to_fixed_point(&self, mode: RoundingMode) -> Result<FixedPoint, FixedPointError> {
        let result = *self;
        let mut value = result.quotient;
        let atoms = value.atoms;
        let r = self.rem;
        let d = self.div;

        debug_assert!(d > 0);
        debug_assert!(r >= 0 && r < d);

        if r == 0 {
            return Ok(value);
        }

        let mut inc = || -> Result<(), FixedPointError> {
            value.atoms = value
                .atoms
                .checked_add(1)
                .ok_or(FixedPointError::ArithmeticOverflow)?;
            Ok(())
        };

        match mode {
            RoundingMode::Floor => {}
            RoundingMode::Ceil => inc()?,
            RoundingMode::TowardZero => {
                if atoms.is_negative() {
                    inc()?;
                }
            }
            RoundingMode::AwayFromZero => {
                if atoms >= 0 {
                    inc()?;
                }
            }
            RoundingMode::HalfEven | RoundingMode::HalfUp | RoundingMode::HalfDown => {
                let two_r = (r as i128) * 2;
                let d128 = d as i128;

                let should_up = if two_r > d128 {
                    true
                } else if two_r < d128 {
                    false
                } else {
                    match mode {
                        RoundingMode::HalfUp => true,
                        RoundingMode::HalfDown => false,
                        RoundingMode::HalfEven => atoms & 1 != 0,
                        _ => unreachable!(),
                    }
                };

                if should_up {
                    inc()?;
                }
            }
        }

        Ok(value)
    }

    pub fn to_fixed_point(&self, mode: RoundingMode) -> FixedPoint {
        self.try_to_fixed_point(mode)
            .unwrap_or_else(|e| panic!("{e}"))
    }
}

impl FixedPoint {
    pub fn try_div_i64(&self, rhs: i64) -> Result<DivResult, FixedPointError> {
        if rhs == 0 {
            return Err(FixedPointError::InvalidDivisor {
                operation: "div",
                divisor: rhs,
            });
        }

        if rhs == i64::MIN {
            return Err(FixedPointError::InvalidDivisor {
                operation: "div",
                divisor: rhs,
            });
        }

        let mut quotient = *self;
        let (atoms, rem, div) = checked_div_rem_euclid_signed(quotient.atoms, rhs)
            .ok_or(FixedPointError::ArithmeticOverflow)?;

        quotient.atoms = atoms;

        Ok(DivResult { quotient, rem, div })
    }

    pub fn div_i64(&self, rhs: i64) -> DivResult {
        self.try_div_i64(rhs).unwrap_or_else(|e| panic!("{e}"))
    }

    pub fn try_div_i32(&self, rhs: i32) -> Result<DivResult, FixedPointError> {
        self.try_div_i64(rhs as i64)
    }

    pub fn div_i32(&self, rhs: i32) -> DivResult {
        self.try_div_i64(rhs as i64)
            .unwrap_or_else(|e| panic!("{e}"))
    }
}
