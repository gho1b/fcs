use super::*;

#[test]
fn basic_add_sub() {
    let a = FixedPoint::new(5, 100);
    let b = FixedPoint::new(1200, 100);
    let c = a + b;
    assert_eq!(c.atoms(), 1205);
    assert_eq!(c.units(), 12);
    assert_eq!(c.subunits(), 5);
    let d = c - a;
    assert_eq!(d.atoms(), 1200);
}

#[test]
fn add_supports_cross_scale_and_returns_max_scale() {
    // 1.00 + 0.001 = 1.001 (scale=1000)
    let a = FixedPoint::new(100, 100);
    let b = FixedPoint::new(1, 1000);
    let c = a + b;
    assert_eq!(c.scale(), 1000);
    assert_eq!(c.atoms(), 1001);

    // commutative
    let d = b + a;
    assert_eq!(d, c);
}

#[test]
fn try_add_overflow() {
    let err = FixedPoint::new(i64::MAX, 100)
        .try_add(&FixedPoint::new(1, 100))
        .unwrap_err();
    assert!(matches!(err, FixedPointError::ArithmeticOverflow));
}

#[test]
fn try_sub_overflow() {
    let err = FixedPoint::new(i64::MIN, 100)
        .try_sub(&FixedPoint::new(1, 100))
        .unwrap_err();
    assert!(matches!(err, FixedPointError::ArithmeticOverflow));
}

#[test]
fn negative_units_subunits_euclid() {
    let x = FixedPoint::new(-1, 100);
    assert_eq!(x.units(), -1);
    assert_eq!(x.subunits(), 99);
    let y = FixedPoint::new(-101, 100);
    assert_eq!(y.units(), -2);
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
fn sub_supports_cross_scale_and_returns_max_scale() {
    // 1.00 - 0.001 = 0.999 (scale=1000)
    let a = FixedPoint::new(100, 100);
    let b = FixedPoint::new(1, 1000);
    let c = a - b;
    assert_eq!(c.scale(), 1000);
    assert_eq!(c.atoms(), 999);
}

#[test]
#[should_panic]
fn add_assign_panics_on_overflow() {
    let mut a = FixedPoint::new(i64::MAX, 100);
    a += FixedPoint::new(1, 100);
}

#[test]
fn sum_supports_cross_scale_and_returns_max_scale() {
    let values = [
        FixedPoint::new(100, 100), // 1.00
        FixedPoint::new(1, 1000),  // 0.001
        FixedPoint::new(-50, 100), // -0.50
    ];
    let s: FixedPoint = values.into_iter().sum();
    assert_eq!(s.scale(), 1000);
    assert_eq!(s.atoms(), 501); // 0.501
}

#[test]
fn sum_empty_returns_zero_scale_one() {
    let empty: [FixedPoint; 0] = [];
    let s: FixedPoint = empty.into_iter().sum();
    assert_eq!(s, FixedPoint::new(0, 1));
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
    assert!(matches!(
        value.try_rescale_exact(0).unwrap_err(),
        FixedPointError::InvalidScale { scale: 0 }
    ));
    assert!(matches!(
        value.try_rescale_exact(12).unwrap_err(),
        FixedPointError::InvalidScale { scale: 12 }
    ));
}

#[test]
fn try_rescale_exact_rejects_lossy_downscale() {
    assert!(matches!(
        FixedPoint::new(123, 100).try_rescale_exact(10).unwrap_err(),
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
        FixedPoint::new(-1_230, 1_000)
            .try_rescale_exact(100)
            .unwrap()
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
    assert!(matches!(
        value.try_quantize(0, RoundingMode::HalfEven).unwrap_err(),
        FixedPointError::InvalidScale { scale: 0 }
    ));
    assert!(matches!(
        value.try_quantize(12, RoundingMode::HalfEven).unwrap_err(),
        FixedPointError::InvalidScale { scale: 12 }
    ));
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
    assert!(matches!(
        value
            .try_normalize_to(12, RoundingMode::HalfEven)
            .unwrap_err(),
        FixedPointError::InvalidScale { scale: 12 }
    ));
    let err = FixedPoint::new(i64::MAX, 1)
        .try_normalize_to(10, RoundingMode::HalfEven)
        .unwrap_err();
    assert!(matches!(err, FixedPointError::ArithmeticOverflow));
}

#[test]
fn normalize_to_matches_quantize_on_success() {
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

    // Cross-scale add upgrades to the higher scale.
    value.try_add_mut(&FixedPoint::new(1, 1000)).unwrap();
    assert_eq!(value, FixedPoint::new(1231, 1000));

    // Overflow during upscaling keeps the original value unchanged.
    let mut overflow = FixedPoint::new(i64::MAX, 1);
    let err = overflow
        .try_add_mut(&FixedPoint::new(1, 1_000_000_000_000_000_000))
        .unwrap_err();
    assert!(matches!(err, FixedPointError::ArithmeticOverflow));
    assert_eq!(overflow, FixedPoint::new(i64::MAX, 1));

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

    // Cross-scale sub upgrades to the higher scale.
    value.try_sub_mut(&FixedPoint::new(1, 1000)).unwrap();
    assert_eq!(value, FixedPoint::new(1769, 1000));

    // Overflow during upscaling keeps the original value unchanged.
    let mut overflow = FixedPoint::new(i64::MIN, 1);
    let err = overflow
        .try_sub_mut(&FixedPoint::new(1, 1_000_000_000_000_000_000))
        .unwrap_err();
    assert!(matches!(err, FixedPointError::ArithmeticOverflow));
    assert_eq!(overflow, FixedPoint::new(i64::MIN, 1));

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
    let r = FixedPoint::new(7, 1).try_div_i64(-3).unwrap();
    assert_eq!(r.div, 3);
    assert_eq!(r.rem, 2);
    assert_eq!(r.quotient.atoms(), -3);
}

#[test]
fn div_negative_tie_half_even() {
    let r = FixedPoint::new(1, 1).try_div_i64(-2).unwrap();
    assert_eq!(r.quotient.atoms(), -1);
    assert_eq!(r.rem, 1);
    assert_eq!(r.div, 2);
    assert_eq!(r.to_fixed_point(RoundingMode::HalfEven).atoms(), 0);
}

#[test]
fn try_div_i64_rejects_invalid_divisors() {
    let value = FixedPoint::new(7, 1);
    assert!(matches!(
        value.try_div_i64(0).unwrap_err(),
        FixedPointError::InvalidDivisor {
            operation: "div",
            divisor: 0
        }
    ));
    assert!(matches!(
        value.try_div_i64(i64::MIN).unwrap_err(),
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
    assert!(matches!(
        result.try_to_fixed_point(RoundingMode::Ceil).unwrap_err(),
        FixedPointError::ArithmeticOverflow
    ));
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
