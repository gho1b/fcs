use crate::error::FixedPointError;
use core::fmt;
use std::iter::Sum;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

mod division;
pub(crate) mod helper;
mod multiplication;
pub(crate) mod rounding;
#[cfg(test)]
mod tests;

pub use division::DivResult;
use rounding::RoundingMode;

const VALID_SCALES: [i64; 19] = [
    1,
    10,
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
    1_000_000_000,
    10_000_000_000,
    100_000_000_000,
    1_000_000_000_000,
    10_000_000_000_000,
    100_000_000_000_000,
    1_000_000_000_000_000,
    10_000_000_000_000_000,
    100_000_000_000_000_000,
    1_000_000_000_000_000_000,
];

pub(crate) fn valid_scale(scale: i64) -> bool {
    VALID_SCALES.binary_search(&scale).is_ok()
}

#[must_use]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct FixedPoint {
    atoms: i64,
    scale: i64,
}

impl fmt::Display for FixedPoint {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let scale = self.scale;
        let atoms = self.atoms;

        let sign = if atoms < 0 { "-" } else { "" };
        let abs = atoms.unsigned_abs();
        let scale_u = self.scale as u64;
        let whole = abs / scale_u;
        let frac = abs % scale_u;
        let frac_width = scale.ilog10() as usize;

        if frac_width > 0 {
            write!(f, "{}{}.{:0width$}", sign, whole, frac, width = frac_width)
        } else {
            write!(f, "{}{}", sign, whole)
        }
    }
}

impl FixedPoint {
    pub fn new(atoms: i64, scale: i64) -> Self {
        assert!(scale > 0, "scale must be greater than zero");
        assert!(valid_scale(scale), "scale must power of 10");
        Self { atoms, scale }
    }

    pub fn new_precision(atoms: i64, precision: u8) -> Self {
        assert!(
            precision <= 18,
            "precision too large for i64 scale (max 18)"
        );

        Self::new(atoms, 10_i64.pow(precision as u32))
    }

    #[inline]
    pub fn same_scale(&self, other: &Self) -> bool {
        self.scale == other.scale
    }

    pub fn atoms(&self) -> i64 {
        self.atoms
    }

    pub fn scale(&self) -> i64 {
        self.scale
    }

    #[inline]
    pub fn try_rescale_exact(&self, target_scale: i64) -> Result<Self, FixedPointError> {
        if !valid_scale(target_scale) {
            return Err(FixedPointError::InvalidScale {
                scale: target_scale,
            });
        }

        if target_scale == self.scale {
            return Ok(*self);
        }

        if target_scale > self.scale {
            let factor = target_scale / self.scale;
            debug_assert_eq!(target_scale % self.scale, 0);

            let atoms = self
                .atoms
                .checked_mul(factor)
                .ok_or(FixedPointError::ArithmeticOverflow)?;

            return Ok(Self::new(atoms, target_scale));
        }

        let factor = self.scale / target_scale;
        debug_assert_eq!(self.scale % target_scale, 0);

        if self.atoms % factor != 0 {
            return Err(FixedPointError::NonExactRescale {
                from: self.scale,
                to: target_scale,
            });
        }

        Ok(Self::new(self.atoms / factor, target_scale))
    }

    #[inline]
    pub fn rescale_exact(&self, target_scale: i64) -> Self {
        self.try_rescale_exact(target_scale)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    #[inline]
    pub fn try_quantize(
        &self,
        target_scale: i64,
        rounding_mode: RoundingMode,
    ) -> Result<Self, FixedPointError> {
        if !valid_scale(target_scale) {
            return Err(FixedPointError::InvalidScale {
                scale: target_scale,
            });
        }

        if target_scale >= self.scale {
            return self.try_rescale_exact(target_scale);
        }

        let factor = self.scale / target_scale;
        debug_assert_eq!(self.scale % target_scale, 0);

        let (atoms, rem, div) = helper::checked_div_rem_euclid_signed_i64(self.atoms, factor)
            .ok_or(FixedPointError::ArithmeticOverflow)?;

        let result = DivResult {
            quotient: Self::new(atoms, target_scale),
            rem,
            div,
        };

        result.try_to_fixed_point(rounding_mode)
    }

    #[inline]
    pub fn quantize(&self, target_scale: i64, rounding_mode: RoundingMode) -> Self {
        self.try_quantize(target_scale, rounding_mode)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    #[inline]
    pub fn try_normalize_to(
        &self,
        target_scale: i64,
        rounding_mode: RoundingMode,
    ) -> Result<Self, FixedPointError> {
        self.try_quantize(target_scale, rounding_mode)
    }

    #[inline]
    pub fn normalize_to(&self, target_scale: i64, rounding_mode: RoundingMode) -> Self {
        self.try_normalize_to(target_scale, rounding_mode)
            .unwrap_or_else(|e| panic!("{e}"))
    }

    #[inline]
    pub fn units(&self) -> i64 {
        self.atoms.div_euclid(self.scale)
    }

    #[inline]
    pub fn subunits(&self) -> i64 {
        self.atoms.rem_euclid(self.scale)
    }

    #[inline]
    pub fn is_zero(&self) -> bool {
        self.atoms == 0
    }

    #[inline]
    pub fn checked_abs(&self) -> Option<Self> {
        let atoms = self.atoms.checked_abs()?;
        Some(Self {
            atoms,
            scale: self.scale,
        })
    }

    #[inline]
    pub fn abs(&self) -> Self {
        self.checked_abs().expect("arithmetic overflow")
    }

    #[inline]
    pub fn checked_neg(&self) -> Option<Self> {
        Some(Self {
            atoms: self.atoms.checked_neg()?,
            scale: self.scale,
        })
    }
}

impl FixedPoint {
    #[inline]
    fn checked_upscale_atoms(
        atoms: i64,
        from_scale: i64,
        to_scale: i64,
    ) -> Result<i64, FixedPointError> {
        debug_assert!(to_scale >= from_scale);
        debug_assert_eq!(to_scale % from_scale, 0);
        if from_scale == to_scale {
            return Ok(atoms);
        }
        let factor = to_scale / from_scale;
        atoms
            .checked_mul(factor)
            .ok_or(FixedPointError::ArithmeticOverflow)
    }

    #[inline]
    pub fn try_add_mut(&mut self, other: &Self) -> Result<(), FixedPointError> {
        if self.same_scale(other) {
            self.atoms = self
                .atoms
                .checked_add(other.atoms)
                .ok_or(FixedPointError::ArithmeticOverflow)?;
            return Ok(());
        }

        let target_scale = self.scale.max(other.scale);

        let lhs_atoms = Self::checked_upscale_atoms(self.atoms, self.scale, target_scale)?;

        let rhs_atoms = Self::checked_upscale_atoms(other.atoms, other.scale, target_scale)?;

        self.atoms = lhs_atoms
            .checked_add(rhs_atoms)
            .ok_or(FixedPointError::ArithmeticOverflow)?;
        self.scale = target_scale;

        Ok(())
    }

    #[inline]
    pub fn try_add(&self, other: &Self) -> Result<Self, FixedPointError> {
        let mut value = *self;
        value.try_add_mut(other)?;
        Ok(value)
    }

    #[inline]
    pub fn try_sub_mut(&mut self, other: &Self) -> Result<(), FixedPointError> {
        if self.same_scale(other) {
            self.atoms = self
                .atoms
                .checked_sub(other.atoms)
                .ok_or(FixedPointError::ArithmeticOverflow)?;
            return Ok(());
        }

        let target_scale = self.scale.max(other.scale);

        let lhs_atoms = Self::checked_upscale_atoms(self.atoms, self.scale, target_scale)?;

        let rhs_atoms = Self::checked_upscale_atoms(other.atoms, other.scale, target_scale)?;

        self.atoms = lhs_atoms
            .checked_sub(rhs_atoms)
            .ok_or(FixedPointError::ArithmeticOverflow)?;
        self.scale = target_scale;

        Ok(())
    }

    #[inline]
    pub fn try_sub(&self, other: &Self) -> Result<Self, FixedPointError> {
        let mut value = *self;
        value.try_sub_mut(other)?;
        Ok(value)
    }

    #[inline]
    pub fn try_mul_i64_mut(&mut self, other: i64) -> Result<(), FixedPointError> {
        self.atoms = self
            .atoms
            .checked_mul(other)
            .ok_or(FixedPointError::ArithmeticOverflow)?;

        Ok(())
    }

    #[inline]
    pub fn try_mul_i64(&self, other: i64) -> Result<Self, FixedPointError> {
        let mut value = *self;
        value.try_mul_i64_mut(other)?;
        Ok(value)
    }
}

impl Add for FixedPoint {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.try_add(&rhs).unwrap_or_else(|e| panic!("{e}"))
    }
}

impl AddAssign for FixedPoint {
    fn add_assign(&mut self, rhs: Self) {
        self.try_add_mut(&rhs).unwrap_or_else(|e| panic!("{e}"))
    }
}

impl Sub for FixedPoint {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.try_sub(&rhs).unwrap_or_else(|e| panic!("{e}"))
    }
}

impl SubAssign for FixedPoint {
    fn sub_assign(&mut self, rhs: Self) {
        self.try_sub_mut(&rhs).unwrap_or_else(|e| panic!("{e}"))
    }
}

impl Mul<i64> for FixedPoint {
    type Output = Self;

    fn mul(self, rhs: i64) -> Self::Output {
        self.try_mul_i64(rhs).unwrap_or_else(|e| panic!("{e}"))
    }
}

impl MulAssign<i64> for FixedPoint {
    fn mul_assign(&mut self, rhs: i64) {
        self.try_mul_i64_mut(rhs).unwrap_or_else(|e| panic!("{e}"))
    }
}

impl Neg for FixedPoint {
    type Output = Self;

    fn neg(self) -> Self::Output {
        self.checked_neg()
            .unwrap_or_else(|| panic!("arithmetic overflow"))
    }
}

impl Sum for FixedPoint {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        let mut total_atoms: i128 = 0;
        let mut current_scale: i64 = 0;

        for f in iter {
            if current_scale == 0 {
                current_scale = f.scale;
                total_atoms = f.atoms as i128;
                continue;
            }

            if f.scale == current_scale {
                total_atoms += f.atoms as i128;
                continue;
            }

            if f.scale > current_scale {
                debug_assert_eq!(f.scale % current_scale, 0);
                let factor = (f.scale / current_scale) as i128;
                total_atoms = total_atoms
                    .checked_mul(factor)
                    .unwrap_or_else(|| panic!("{}", FixedPointError::ArithmeticOverflow));
                total_atoms += f.atoms as i128;
                current_scale = f.scale;
            } else {
                debug_assert_eq!(current_scale % f.scale, 0);
                let factor = (current_scale / f.scale) as i128;
                let up = (f.atoms as i128)
                    .checked_mul(factor)
                    .unwrap_or_else(|| panic!("{}", FixedPointError::ArithmeticOverflow));
                total_atoms += up;
            }
        }

        if current_scale == 0 {
            return FixedPoint::new(0, 1);
        }

        let atoms: i64 = total_atoms
            .try_into()
            .unwrap_or_else(|_| panic!("{}", FixedPointError::ArithmeticOverflow));

        FixedPoint::new(atoms, current_scale)
    }
}
