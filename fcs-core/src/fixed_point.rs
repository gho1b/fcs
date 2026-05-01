use crate::error::FixedPointError;
use core::fmt;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub enum RoundingMode {
    /// Round to nearest; ties go to the even result.
    ///
    /// Often called banker's rounding or unbiased rounding.
    #[default]
    HalfEven,
    /// Round to nearest; ties go to the greater numeric (toward +∞)
    ///
    /// Examples:
    /// - `2.5 -> 3`
    /// - `-2.5 -> -2`
    HalfUp,
    /// Round to nearest; ties go to the smaller numeric (toward -∞)
    ///
    /// Examples:
    /// - `2.5 -> 2`
    /// - `-2.5 -> -3`
    HalfDown,
    /// Round toward negative infinity.
    Floor,
    /// Round toward positive infinity.
    Ceil,
    /// Round toward zero.
    TowardZero,
    /// Round away from zero.
    AwayFromZero,
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

        let (atoms, rem, div) = checked_div_rem_euclid_signed(self.atoms, factor)
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

    /// Bagian unit (whole/major). Contoh: 1200/100 => 12
    #[inline]
    pub fn units(&self) -> i64 {
        self.atoms.div_euclid(self.scale)
    }

    /// Bagian subunit (0..scale-1). Aman untuk negatif karena euclid.
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
    #[cold]
    #[inline(never)]
    fn err_incompatible(op: &'static str, expected: i64, got: i64) -> FixedPointError {
        FixedPointError::IncompatibleScale {
            operation: op,
            expected,
            got,
        }
    }

    #[inline]
    pub fn try_add_mut(&mut self, other: &Self) -> Result<(), FixedPointError> {
        if !self.same_scale(other) {
            return Err(Self::err_incompatible("add", self.scale, other.scale));
        }

        self.atoms = self
            .atoms
            .checked_add(other.atoms)
            .ok_or(FixedPointError::ArithmeticOverflow)?;

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
        if !self.same_scale(other) {
            return Err(Self::err_incompatible("sub", self.scale, other.scale));
        }

        self.atoms = self
            .atoms
            .checked_sub(other.atoms)
            .ok_or(FixedPointError::ArithmeticOverflow)?;

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

#[must_use]
#[inline]
fn checked_div_rem_euclid_signed(a: i64, b: i64) -> Option<(i64, i64, i64)> {
    // returns (q_atoms, rem, div_pos) with div_pos > 0 and 0 <= rem < div_pos
    if b == 0 {
        return None;
    }
    if b == i64::MIN {
        // abs(i64::MIN) overflow in i64, reject (or switch to i128 variant)
        return None;
    }

    let div = b.abs(); // safe because b != i64::MIN
    // q,r for positive divisor div
    let q = a.checked_div(div)?; // safe (div > 0)
    let r = a - q.checked_mul(div)?; // trunc remainder, may be negative
    let (q, r) = if r >= 0 {
        (q, r)
    } else {
        // Euclidean adjust for div>0
        (q.checked_sub(1)?, r.checked_add(div)?)
    };
    // now 0 <= r < div

    if b > 0 {
        Some((q, r, div))
    } else {
        // negate (q + r/div) into canonical form
        if r == 0 {
            Some((q.checked_neg()?, 0, div))
        } else {
            let q2 = q.checked_neg()?.checked_sub(1)?;
            let r2 = div.checked_sub(r)?;
            Some((q2, r2, div))
        }
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
        let mut money = result.quotient;
        let atoms = money.atoms;
        let r = self.rem;
        let d = self.div;

        debug_assert!(d > 0);
        debug_assert!(r >= 0 && r < d);

        if r == 0 {
            return Ok(money);
        }

        let mut inc = || -> Result<(), FixedPointError> {
            money.atoms = money
                .atoms
                .checked_add(1)
                .ok_or(FixedPointError::ArithmeticOverflow)?;
            Ok(())
        };

        match mode {
            RoundingMode::Floor => {}
            RoundingMode::Ceil => {
                inc()?;
            }
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

        Ok(money)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_add_sub() {
        let a = FixedPoint::new(5, 100); // 0.05
        let b = FixedPoint::new(1200, 100); // 12.00

        let c = a + b; // 12.05
        assert_eq!(c.atoms(), 1205);
        assert_eq!(c.units(), 12);
        assert_eq!(c.subunits(), 5);

        let d = c - a; // 12.00
        assert_eq!(d.atoms(), 1200);
    }

    #[test]
    fn try_add_incompatible_scale() {
        let a = FixedPoint::new(100, 100);
        let b = FixedPoint::new(100, 1000);

        let err = a.try_add(&b).unwrap_err();
        assert!(matches!(
            err,
            FixedPointError::IncompatibleScale {
                operation: "add",
                expected: 100,
                got: 1000
            }
        ));
    }

    #[test]
    #[should_panic(expected = "scale incompatible for add: expected 100 but got 1000")]
    fn add_panics_on_incompatible_scale() {
        let a = FixedPoint::new(100, 100);
        let b = FixedPoint::new(100, 1000);
        let _ = a + b;
    }

    #[test]
    fn try_add_overflow() {
        let a = FixedPoint::new(i64::MAX, 100);
        let b = FixedPoint::new(1, 100);

        let err = a.try_add(&b).unwrap_err();
        assert!(matches!(err, FixedPointError::ArithmeticOverflow));
    }

    #[test]
    fn try_sub_overflow() {
        let a = FixedPoint::new(i64::MIN, 100);
        let b = FixedPoint::new(1, 100);

        let err = a.try_sub(&b).unwrap_err();
        assert!(matches!(err, FixedPointError::ArithmeticOverflow));
    }

    #[test]
    fn negative_units_subunits_euclid() {
        let x = FixedPoint::new(-1, 100); // -0.01

        // Euclidean: -1 = (-1)*100 + 99
        assert_eq!(x.units(), -1);
        assert_eq!(x.subunits(), 99);

        let y = FixedPoint::new(-101, 100); // -1.01
        assert_eq!(y.units(), -2); // -101 = (-2)*100 + 99
        assert_eq!(y.subunits(), 99);
    }

    #[test]
    fn add_identity_and_inverse() {
        let x = FixedPoint::new(123, 100);
        let zero = FixedPoint::new(0, 100);

        assert_eq!(x + zero, x);
        assert_eq!(x - zero, x);
        assert_eq!(x - x, zero);
    }

    #[test]
    fn add_commutative_same_scale() {
        let x = FixedPoint::new(10, 100);
        let y = FixedPoint::new(20, 100);

        assert_eq!(x + y, y + x);
    }

    #[test]
    fn try_sub_incompatible_scale() {
        let a = FixedPoint::new(100, 100);
        let b = FixedPoint::new(100, 1000);

        let err = a.try_sub(&b).unwrap_err();
        assert!(matches!(
            err,
            FixedPointError::IncompatibleScale {
                operation: "sub",
                expected: 100,
                got: 1000
            }
        ));
    }

    #[test]
    #[should_panic]
    fn sub_panics_on_incompatible_scale() {
        let a = FixedPoint::new(100, 100);
        let b = FixedPoint::new(100, 1000);
        let _ = a - b;
    }

    #[test]
    #[should_panic]
    fn add_assign_panics_on_overflow() {
        let mut a = FixedPoint::new(i64::MAX, 100);
        a += FixedPoint::new(1, 100);
    }

    #[test]
    fn display_formats_fixed_point_values() {
        assert_eq!(FixedPoint::new(0, 100).to_string(), "0.00");
        assert_eq!(FixedPoint::new(5, 100).to_string(), "0.05");
        assert_eq!(FixedPoint::new(1205, 100).to_string(), "12.05");
        assert_eq!(FixedPoint::new(-1205, 100).to_string(), "-12.05");
    }

    #[test]
    fn display_omits_fraction_for_scale_one() {
        assert_eq!(FixedPoint::new(42, 1).to_string(), "42");
        assert_eq!(FixedPoint::new(-42, 1).to_string(), "-42");
    }

    #[test]
    fn display_handles_i64_min() {
        assert_eq!(
            FixedPoint::new(i64::MIN, 1).to_string(),
            "-9223372036854775808"
        );
        assert_eq!(
            FixedPoint::new(i64::MIN, 100).to_string(),
            "-92233720368547758.08"
        );
    }

    #[test]
    fn new_precision_builds_power_of_ten_scale() {
        assert_eq!(FixedPoint::new_precision(42, 0), FixedPoint::new(42, 1));
        assert_eq!(FixedPoint::new_precision(42, 2), FixedPoint::new(42, 100));
        assert_eq!(
            FixedPoint::new_precision(42, 18),
            FixedPoint::new(42, 1_000_000_000_000_000_000)
        );
    }

    #[test]
    #[should_panic(expected = "scale must be greater than zero")]
    fn new_panics_on_non_positive_scale() {
        let _ = FixedPoint::new(42, 0);
    }

    #[test]
    #[should_panic(expected = "scale must power of 10")]
    fn new_panics_on_non_power_of_ten_scale() {
        let _ = FixedPoint::new(42, 12);
    }

    #[test]
    #[should_panic(expected = "precision too large for i64 scale (max 18)")]
    fn new_precision_panics_when_precision_too_large() {
        let _ = FixedPoint::new_precision(42, 19);
    }

    #[test]
    fn helper_methods_return_expected_values() {
        let zero = FixedPoint::new(0, 100);
        let value = FixedPoint::new(123, 100);
        let other_scale = FixedPoint::new(123, 1000);

        assert!(zero.is_zero());
        assert!(!value.is_zero());
        assert_eq!(value.scale(), 100);
        assert!(value.same_scale(&zero));
        assert!(!value.same_scale(&other_scale));
    }

    #[test]
    fn try_rescale_exact_preserves_value_when_exact() {
        assert_eq!(
            FixedPoint::new(123, 100).try_rescale_exact(1_000).unwrap(),
            FixedPoint::new(1_230, 1_000)
        );
        assert_eq!(
            FixedPoint::new(1_230, 1_000)
                .try_rescale_exact(100)
                .unwrap(),
            FixedPoint::new(123, 100)
        );
        assert_eq!(
            FixedPoint::new(-1_230, 1_000)
                .try_rescale_exact(100)
                .unwrap(),
            FixedPoint::new(-123, 100)
        );
        assert_eq!(
            FixedPoint::new(123, 100).try_rescale_exact(100).unwrap(),
            FixedPoint::new(123, 100)
        );
    }

    #[test]
    fn try_rescale_exact_rejects_invalid_target_scale() {
        let value = FixedPoint::new(123, 100);

        let err = value.try_rescale_exact(0).unwrap_err();
        assert!(matches!(err, FixedPointError::InvalidScale { scale: 0 }));

        let err = value.try_rescale_exact(12).unwrap_err();
        assert!(matches!(err, FixedPointError::InvalidScale { scale: 12 }));
    }

    #[test]
    fn try_rescale_exact_rejects_lossy_downscale() {
        let err = FixedPoint::new(123, 100).try_rescale_exact(10).unwrap_err();
        assert!(matches!(
            err,
            FixedPointError::NonExactRescale { from: 100, to: 10 }
        ));
    }

    #[test]
    fn try_rescale_exact_reports_overflow_when_upscaling() {
        let err = FixedPoint::new(i64::MAX, 1)
            .try_rescale_exact(10)
            .unwrap_err();
        assert!(matches!(err, FixedPointError::ArithmeticOverflow));
    }

    #[test]
    fn rescale_exact_matches_try_rescale_exact_on_success() {
        assert_eq!(
            FixedPoint::new(123, 100).rescale_exact(1_000),
            FixedPoint::new(123, 100).try_rescale_exact(1_000).unwrap()
        );
        assert_eq!(
            FixedPoint::new(-1_230, 1_000).rescale_exact(100),
            FixedPoint::new(-1_230, 1_000).try_rescale_exact(100).unwrap()
        );
    }

    #[test]
    #[should_panic(expected = "non-exact rescale from 100 to 10")]
    fn rescale_exact_panics_on_lossy_downscale() {
        let _ = FixedPoint::new(123, 100).rescale_exact(10);
    }

    #[test]
    fn try_quantize_preserves_or_increases_scale_exactly() {
        assert_eq!(
            FixedPoint::new(123, 100)
                .try_quantize(100, RoundingMode::HalfEven)
                .unwrap(),
            FixedPoint::new(123, 100)
        );
        assert_eq!(
            FixedPoint::new(123, 100)
                .try_quantize(1_000, RoundingMode::HalfEven)
                .unwrap(),
            FixedPoint::new(1_230, 1_000)
        );
    }

    #[test]
    fn try_quantize_rounds_positive_values() {
        assert_eq!(
            FixedPoint::new(125, 100)
                .try_quantize(10, RoundingMode::HalfEven)
                .unwrap(),
            FixedPoint::new(12, 10)
        );
        assert_eq!(
            FixedPoint::new(125, 100)
                .try_quantize(10, RoundingMode::HalfUp)
                .unwrap(),
            FixedPoint::new(13, 10)
        );
        assert_eq!(
            FixedPoint::new(129, 100)
                .try_quantize(10, RoundingMode::Ceil)
                .unwrap(),
            FixedPoint::new(13, 10)
        );
        assert_eq!(
            FixedPoint::new(129, 100)
                .try_quantize(10, RoundingMode::Floor)
                .unwrap(),
            FixedPoint::new(12, 10)
        );
    }

    #[test]
    fn try_quantize_rounds_negative_values() {
        assert_eq!(
            FixedPoint::new(-125, 100)
                .try_quantize(10, RoundingMode::HalfEven)
                .unwrap(),
            FixedPoint::new(-12, 10)
        );
        assert_eq!(
            FixedPoint::new(-125, 100)
                .try_quantize(10, RoundingMode::HalfDown)
                .unwrap(),
            FixedPoint::new(-13, 10)
        );
        assert_eq!(
            FixedPoint::new(-129, 100)
                .try_quantize(10, RoundingMode::Ceil)
                .unwrap(),
            FixedPoint::new(-12, 10)
        );
        assert_eq!(
            FixedPoint::new(-129, 100)
                .try_quantize(10, RoundingMode::AwayFromZero)
                .unwrap(),
            FixedPoint::new(-13, 10)
        );
    }

    #[test]
    fn try_quantize_rejects_invalid_target_scale() {
        let value = FixedPoint::new(123, 100);

        let err = value.try_quantize(0, RoundingMode::HalfEven).unwrap_err();
        assert!(matches!(err, FixedPointError::InvalidScale { scale: 0 }));

        let err = value.try_quantize(12, RoundingMode::HalfEven).unwrap_err();
        assert!(matches!(err, FixedPointError::InvalidScale { scale: 12 }));
    }

    #[test]
    fn try_quantize_reports_overflow_when_upscaling() {
        let err = FixedPoint::new(i64::MAX, 1)
            .try_quantize(10, RoundingMode::HalfEven)
            .unwrap_err();
        assert!(matches!(err, FixedPointError::ArithmeticOverflow));
    }

    #[test]
    fn try_normalize_to_matches_try_quantize_on_success() {
        let value = FixedPoint::new(125, 100);

        assert_eq!(
            value.try_normalize_to(10, RoundingMode::HalfEven).unwrap(),
            value.try_quantize(10, RoundingMode::HalfEven).unwrap()
        );
        assert_eq!(
            value.try_normalize_to(1_000, RoundingMode::HalfUp).unwrap(),
            value.try_quantize(1_000, RoundingMode::HalfUp).unwrap()
        );
        assert_eq!(
            FixedPoint::new(-129, 100)
                .try_normalize_to(10, RoundingMode::AwayFromZero)
                .unwrap(),
            FixedPoint::new(-129, 100)
                .try_quantize(10, RoundingMode::AwayFromZero)
                .unwrap()
        );
    }

    #[test]
    fn try_normalize_to_matches_try_quantize_on_error() {
        let value = FixedPoint::new(123, 100);

        let err = value.try_normalize_to(12, RoundingMode::HalfEven).unwrap_err();
        assert!(matches!(err, FixedPointError::InvalidScale { scale: 12 }));

        let err = FixedPoint::new(i64::MAX, 1)
            .try_normalize_to(10, RoundingMode::HalfEven)
            .unwrap_err();
        assert!(matches!(err, FixedPointError::ArithmeticOverflow));
    }

    #[test]
    fn normalize_to_matches_quantize_on_success() {
        let value = FixedPoint::new(125, 100);
        assert_eq!(
            value.normalize_to(10, RoundingMode::HalfUp),
            value.quantize(10, RoundingMode::HalfUp)
        );
        assert_eq!(
            FixedPoint::new(-129, 100).normalize_to(10, RoundingMode::Ceil),
            FixedPoint::new(-129, 100).quantize(10, RoundingMode::Ceil)
        );
    }

    #[test]
    #[should_panic(expected = "invalid scale: got 12")]
    fn normalize_to_panics_on_invalid_scale() {
        let _ = FixedPoint::new(123, 100).normalize_to(12, RoundingMode::HalfEven);
    }

    #[test]
    fn abs_and_checked_abs_preserve_scale() {
        let value = FixedPoint::new(-123, 100);

        assert_eq!(value.abs(), FixedPoint::new(123, 100));
        assert_eq!(value.checked_abs(), Some(FixedPoint::new(123, 100)));
        assert_eq!(FixedPoint::new(i64::MIN, 100).checked_abs(), None);
    }

    #[test]
    #[should_panic(expected = "arithmetic overflow")]
    fn abs_panics_on_i64_min() {
        let _ = FixedPoint::new(i64::MIN, 100).abs();
    }

    #[test]
    fn checked_neg_handles_i64_min() {
        assert_eq!(
            FixedPoint::new(123, 100).checked_neg(),
            Some(FixedPoint::new(-123, 100))
        );
        assert_eq!(FixedPoint::new(i64::MIN, 100).checked_neg(), None);
    }

    #[test]
    #[should_panic(expected = "arithmetic overflow")]
    fn neg_panics_on_i64_min() {
        let _ = -FixedPoint::new(i64::MIN, 100);
    }

    #[test]
    fn try_add_mut_success_and_error_leave_expected_state() {
        let mut value = FixedPoint::new(100, 100);
        value.try_add_mut(&FixedPoint::new(23, 100)).unwrap();
        assert_eq!(value, FixedPoint::new(123, 100));

        let err = value.try_add_mut(&FixedPoint::new(1, 1000)).unwrap_err();
        assert!(matches!(
            err,
            FixedPointError::IncompatibleScale {
                operation: "add",
                expected: 100,
                got: 1000
            }
        ));
        assert_eq!(value, FixedPoint::new(123, 100));

        let mut overflow = FixedPoint::new(i64::MAX, 100);
        let err = overflow.try_add_mut(&FixedPoint::new(1, 100)).unwrap_err();
        assert!(matches!(err, FixedPointError::ArithmeticOverflow));
        assert_eq!(overflow, FixedPoint::new(i64::MAX, 100));
    }

    #[test]
    fn try_sub_mut_success_and_error_leave_expected_state() {
        let mut value = FixedPoint::new(200, 100);
        value.try_sub_mut(&FixedPoint::new(23, 100)).unwrap();
        assert_eq!(value, FixedPoint::new(177, 100));

        let err = value.try_sub_mut(&FixedPoint::new(1, 1000)).unwrap_err();
        assert!(matches!(
            err,
            FixedPointError::IncompatibleScale {
                operation: "sub",
                expected: 100,
                got: 1000
            }
        ));
        assert_eq!(value, FixedPoint::new(177, 100));

        let mut overflow = FixedPoint::new(i64::MIN, 100);
        let err = overflow.try_sub_mut(&FixedPoint::new(1, 100)).unwrap_err();
        assert!(matches!(err, FixedPointError::ArithmeticOverflow));
        assert_eq!(overflow, FixedPoint::new(i64::MIN, 100));
    }

    #[test]
    fn multiplication_and_negation_work_for_fixed_point() {
        let value = FixedPoint::new(125, 100);
        assert_eq!(value.try_mul_i64(3).unwrap(), FixedPoint::new(375, 100));
        assert_eq!(value * -2, FixedPoint::new(-250, 100));
        assert_eq!(-value, FixedPoint::new(-125, 100));

        let mut mutable = value;
        mutable.try_mul_i64_mut(-4).unwrap();
        assert_eq!(mutable, FixedPoint::new(-500, 100));

        mutable *= -1;
        assert_eq!(mutable, FixedPoint::new(500, 100));
    }

    #[test]
    fn try_mul_i64_overflow_keeps_original_value() {
        let mut value = FixedPoint::new(i64::MAX, 100);
        let err = value.try_mul_i64_mut(2).unwrap_err();
        assert!(matches!(err, FixedPointError::ArithmeticOverflow));
        assert_eq!(value, FixedPoint::new(i64::MAX, 100));
    }

    #[test]
    #[should_panic(expected = "arithmetic overflow")]
    fn mul_panics_on_overflow() {
        let _ = FixedPoint::new(i64::MAX, 100) * 2;
    }

    #[test]
    fn div_negative_canonical() {
        let m = FixedPoint::new(7, 1);
        let r = m.try_div_i64(-3).unwrap();
        assert_eq!(r.div, 3);
        assert_eq!(r.rem, 2);
        assert_eq!(r.quotient.atoms(), -3); // -3 + 2/3 = -2.333..
    }

    #[test]
    fn div_negative_tie_half_even() {
        let m = FixedPoint::new(1, 1); // 1
        let r = m.try_div_i64(-2).unwrap(); // -0.5 => q=-1 rem=1 div=2
        assert_eq!(r.quotient.atoms(), -1);
        assert_eq!(r.rem, 1);
        assert_eq!(r.div, 2);

        let rounded = r.to_fixed_point(RoundingMode::HalfEven);
        // -0.5 tie between -1 and 0 => even is 0
        assert_eq!(rounded.atoms(), 0);
    }

    #[test]
    fn try_div_i64_rejects_invalid_divisors() {
        let value = FixedPoint::new(7, 1);

        let err = value.try_div_i64(0).unwrap_err();
        assert!(matches!(
            err,
            FixedPointError::InvalidDivisor {
                operation: "div",
                divisor: 0
            }
        ));

        let err = value.try_div_i64(i64::MIN).unwrap_err();
        assert!(matches!(
            err,
            FixedPointError::InvalidDivisor {
                operation: "div",
                divisor: i64::MIN
            }
        ));
    }

    #[test]
    fn div_result_preserves_canonical_invariants_and_reconstructs_original() {
        let cases = [
            (7_i64, 3_i64),
            (7, -3),
            (-7, 3),
            (-7, -3),
            (1, 2),
            (1, -2),
            (-1, 2),
            (-1, -2),
            (9_223_372_036_854_775_000, 10),
            (-9_223_372_036_854_775_000, -10),
        ];

        for (atoms, divisor) in cases {
            let original = FixedPoint::new(atoms, 100);
            let result = original.try_div_i64(divisor).unwrap();

            assert!(result.div > 0, "atoms={atoms}, divisor={divisor}");
            assert!(
                result.rem >= 0 && result.rem < result.div,
                "atoms={atoms}, divisor={divisor}, rem={}, div={}",
                result.rem,
                result.div
            );
            assert_eq!(result.quotient.scale(), original.scale());

            let lhs = result.quotient.atoms() as i128;
            let rhs = result.div as i128;
            let rem = result.rem as i128;
            let reconstructed = if divisor > 0 {
                lhs * rhs + rem
            } else {
                -(lhs * rhs + rem)
            };

            assert_eq!(
                reconstructed,
                original.atoms() as i128,
                "atoms={atoms}, divisor={divisor}"
            );
        }
    }

    #[test]
    fn div_i64_returns_expected_result() {
        let result = FixedPoint::new(7, 1).div_i64(3);
        assert_eq!(result.quotient.atoms(), 2);
        assert_eq!(result.rem, 1);
        assert_eq!(result.div, 3);
    }

    #[test]
    fn try_div_i32_returns_expected_result() {
        let result = FixedPoint::new(-7, 1).try_div_i32(3).unwrap();
        assert_eq!(result.quotient.atoms(), -3);
        assert_eq!(result.rem, 2);
        assert_eq!(result.div, 3);
    }

    #[test]
    fn div_i32_returns_expected_result() {
        let result = FixedPoint::new(7, 1).div_i32(-3);
        assert_eq!(result.quotient.atoms(), -3);
        assert_eq!(result.rem, 2);
        assert_eq!(result.div, 3);
    }

    #[test]
    fn try_to_fixed_point_reports_rounding_overflow() {
        let result = DivResult {
            quotient: FixedPoint::new(i64::MAX, 1),
            rem: 1,
            div: 2,
        };

        let err = result.try_to_fixed_point(RoundingMode::HalfUp).unwrap_err();
        assert!(matches!(err, FixedPointError::ArithmeticOverflow));

        let err = result.try_to_fixed_point(RoundingMode::Ceil).unwrap_err();
        assert!(matches!(err, FixedPointError::ArithmeticOverflow));

        assert_eq!(
            result.try_to_fixed_point(RoundingMode::Floor),
            Ok(FixedPoint::new(i64::MAX, 1))
        );
    }

    #[test]
    fn rounding_matrix_positive_values() {
        let cases = [
            (
                5_i64,
                2_i64,
                [
                    (RoundingMode::HalfEven, 2_i64),
                    (RoundingMode::HalfUp, 3),
                    (RoundingMode::HalfDown, 2),
                    (RoundingMode::Floor, 2),
                    (RoundingMode::Ceil, 3),
                    (RoundingMode::TowardZero, 2),
                    (RoundingMode::AwayFromZero, 3),
                ],
            ),
            (
                3_i64,
                2_i64,
                [
                    (RoundingMode::HalfEven, 2_i64),
                    (RoundingMode::HalfUp, 2),
                    (RoundingMode::HalfDown, 1),
                    (RoundingMode::Floor, 1),
                    (RoundingMode::Ceil, 2),
                    (RoundingMode::TowardZero, 1),
                    (RoundingMode::AwayFromZero, 2),
                ],
            ),
            (
                7_i64,
                3_i64,
                [
                    (RoundingMode::HalfEven, 2_i64),
                    (RoundingMode::HalfUp, 2),
                    (RoundingMode::HalfDown, 2),
                    (RoundingMode::Floor, 2),
                    (RoundingMode::Ceil, 3),
                    (RoundingMode::TowardZero, 2),
                    (RoundingMode::AwayFromZero, 3),
                ],
            ),
        ];

        for (atoms, div, expected_modes) in cases {
            let result = FixedPoint::new(atoms, 1).try_div_i64(div).unwrap();
            for (mode, expected) in expected_modes {
                assert_eq!(
                    result.to_fixed_point(mode),
                    FixedPoint::new(expected, 1),
                    "atoms={atoms}, div={div}, mode={mode:?}"
                );
            }
        }
    }

    #[test]
    fn rounding_matrix_negative_values() {
        let cases = [
            (
                -5_i64,
                2_i64,
                [
                    (RoundingMode::HalfEven, -2_i64),
                    (RoundingMode::HalfUp, -2),
                    (RoundingMode::HalfDown, -3),
                    (RoundingMode::Floor, -3),
                    (RoundingMode::Ceil, -2),
                    (RoundingMode::TowardZero, -2),
                    (RoundingMode::AwayFromZero, -3),
                ],
            ),
            (
                -3_i64,
                2_i64,
                [
                    (RoundingMode::HalfEven, -2_i64),
                    (RoundingMode::HalfUp, -1),
                    (RoundingMode::HalfDown, -2),
                    (RoundingMode::Floor, -2),
                    (RoundingMode::Ceil, -1),
                    (RoundingMode::TowardZero, -1),
                    (RoundingMode::AwayFromZero, -2),
                ],
            ),
            (
                -7_i64,
                3_i64,
                [
                    (RoundingMode::HalfEven, -2_i64),
                    (RoundingMode::HalfUp, -2),
                    (RoundingMode::HalfDown, -2),
                    (RoundingMode::Floor, -3),
                    (RoundingMode::Ceil, -2),
                    (RoundingMode::TowardZero, -2),
                    (RoundingMode::AwayFromZero, -3),
                ],
            ),
        ];

        for (atoms, div, expected_modes) in cases {
            let result = FixedPoint::new(atoms, 1).try_div_i64(div).unwrap();
            for (mode, expected) in expected_modes {
                assert_eq!(
                    result.to_fixed_point(mode),
                    FixedPoint::new(expected, 1),
                    "atoms={atoms}, div={div}, mode={mode:?}"
                );
            }
        }
    }
}
