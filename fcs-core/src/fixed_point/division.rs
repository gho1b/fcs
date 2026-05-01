use super::{helper, FixedPoint, RoundingMode};
use crate::error::FixedPointError;

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

        let positive = atoms.is_positive();
        let step = if positive { 1 } else { -1 };

        let mut inc = || -> Result<(), FixedPointError> {
            value.atoms = value
                .atoms
                .checked_add(1)
                .ok_or(FixedPointError::ArithmeticOverflow)?;
            Ok(())
        };

        //         let increment_abs = match mode {
        //             RoundingMode::TowardZero => false,
        //             RoundingMode::AwayFromZero => true,
        //             RoundingMode::Ceil => positive,
        //             RoundingMode::Floor => !positive,
        //             RoundingMode::HalfEven
        //             | RoundingMode::HalfCeil
        //             | RoundingMode::HalfFloor
        //             | RoundingMode::HalfTowardsZero
        //             | RoundingMode::HalfAwayFromZero => {
        //                 let twice_r = r.abs() * 2;
        // let b = 2i32.try_into().ok();
        //             }
        //         };

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
        let (atoms, rem, div) = helper::checked_div_rem_euclid_signed_i64(quotient.atoms, rhs)
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
