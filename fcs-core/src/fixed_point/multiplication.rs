use super::FixedPoint;
use crate::{FixedPointError, RoundingMode};

pub struct MulResult {
    atoms: i128,
    scale: i128,
}

impl MulResult {
    #[inline]
    pub fn reduce_until_scale_at_least(self, min_scale: i128) -> Self {
        let (atoms, scale) = reduce_decimal(self.atoms, min_scale);
        Self { atoms, scale }
    }

    #[inline]
    pub fn reduce_to_minimal(self) -> Self {
        self.reduce_until_scale_at_least(1)
    }

    pub fn try_to_fixed_point_exact(self) -> Result<FixedPoint, FixedPointError> {
        let atoms: i64 = self.atoms.try_into()?;
        let scale: i64 = self.scale.try_into()?;

        Ok(FixedPoint { atoms, scale })
    }

    pub fn try_to_fixed_point_exact_scale(
        self,
        target_scale: i64,
        rounding_mode: RoundingMode, // rounding mode dibutuhkan untuk menangani problem target_scale < self.scale
    ) -> Result<FixedPoint, FixedPointError> {
        let t_scale = target_scale as i128;
        let scale: i64 = 0;
        if self.scale > t_scale {
            // karena t_scale lebih besar, logisnya scale up
            let multiplier = t_scale / self.scale;
            let atoms = self.atoms * multiplier;

            Ok(FixedPoint {
                atoms: atoms.try_into()?,
                scale: t_scale.try_into()?,
            })
        } else {
            // buat fractional baru untuk menggantikan fungsi atoms * scale / t_scale karena atoms * scale bisa jadi overflow
            // operasi ini di else karena lebih mahal
            let frac = self.scale / t_scale;
            let mut atoms = self.atoms.div_euclid(frac);
            let rem: i128 = self.atoms.rem_euclid(frac);

            // Jalankan solusi rounding
            // if rounding_mode.should_up(atoms, rem, t_scale, 0, 1, 2) {
            //     atoms = atoms
            //         .checked_add(1)
            //         .ok_or(FixedPointError::ArithmeticOverflow)?;
            // }

            Ok(FixedPoint {
                atoms: atoms.try_into()?,
                scale,
            })
        }
    }

    pub fn try_to_fixed_point_quantized(
        self,
        target_scale: i64,
        rounding_mode: RoundingMode,
    ) -> Result<FixedPoint, FixedPointError> {
        Ok(FixedPoint { atoms: 0, scale: 0 })
    }
}

#[inline]
fn reduce_decimal(mut atoms: i128, mut scale: i128) -> (i128, i128) {
    while scale > 0 && atoms % 10 == 0 {
        atoms /= 10;
        scale /= 10;
    }

    (atoms, scale)
}

#[inline]
fn checked_gcd_i128(mut a: i128, mut b: i128) -> Option<i128> {
    a = a.checked_abs()?;
    b = b.checked_abs()?;

    while b != 0 {
        let r = a % b;
        a = b;
        b = r;
    }

    Some(a)
}

impl FixedPoint {
    pub fn checked_mul_fixed_point(&self, rhs: FixedPoint) -> Option<MulResult> {
        let mut lhs_atoms = self.atoms as i128;
        let mut rhs_atoms = rhs.atoms as i128;
        let mut lhs_scale = self.scale as i128;
        let mut rhs_scale = rhs.scale as i128;

        // pre-reduce each operand
        (lhs_atoms, lhs_scale) = reduce_decimal(lhs_atoms, lhs_scale);
        (rhs_atoms, rhs_scale) = reduce_decimal(rhs_atoms, rhs_scale);

        // cross-cancel to reduce overflow risk
        let g1 = checked_gcd_i128(lhs_atoms.abs(), rhs_scale)?;
        if g1 > 1 {
            lhs_atoms /= g1;
            rhs_scale /= g1;
        }

        let g2 = checked_gcd_i128(rhs_atoms.abs(), lhs_scale)?;
        if g2 > 1 {
            rhs_atoms /= g2;
            lhs_scale /= g2;
        }

        let atoms = lhs_atoms.checked_mul(rhs_atoms)?;
        let scale = lhs_scale.checked_mul(rhs_scale)?;

        let (atoms, scale) = reduce_decimal(atoms, scale);

        Some(MulResult { atoms, scale })
    }
}
