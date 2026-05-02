use super::*;
use std::cmp::Ordering;

const DENOM: i64 = 10;
const DENOM_128: i128 = 10;

#[derive(Clone, Copy, Debug)]
struct Sample {
    // fixed-point: value = num / denom
    num: i64,
    // label hanya untuk memperjelas error message
    label: &'static str,
}

#[test]
fn rounding_matrix_java_style() {
    // Kolom-kolom yang kamu definisikan:
    //  1.1, 1.5, 1.6, -1.1, -1.5, -1.6, 2.5, -2.5
    let samples = [
        Sample {
            num: 11,
            label: " 1.1",
        },
        Sample {
            num: 15,
            label: " 1.5",
        },
        Sample {
            num: 16,
            label: " 1.6",
        },
        Sample {
            num: -11,
            label: "-1.1",
        },
        Sample {
            num: -15,
            label: "-1.5",
        },
        Sample {
            num: -16,
            label: "-1.6",
        },
        Sample {
            num: 25,
            label: " 2.5",
        },
        Sample {
            num: -25,
            label: "-2.5",
        },
    ];

    // Expected output persis seperti matriks Java
    // Urutan harus sama dengan `samples` di atas.
    let cases: &[(RoundingMode, [i64; 8])] = &[
        (RoundingMode::HalfEven, [1, 2, 2, -1, -2, -2, 2, -2]),
        (RoundingMode::HalfUp, [1, 2, 2, -1, -2, -2, 3, -3]),
        (RoundingMode::HalfDown, [1, 1, 2, -1, -1, -2, 2, -2]),
        (RoundingMode::Floor, [1, 1, 1, -2, -2, -2, 2, -3]),
        (RoundingMode::Ceil, [2, 2, 2, -1, -1, -1, 3, -2]),
        (RoundingMode::TowardZero, [1, 1, 1, -1, -1, -1, 2, -2]),
        (RoundingMode::AwayFromZero, [2, 2, 2, -2, -2, -2, 3, -3]),
    ];

    for (mode, expected) in cases {
        for (idx, s) in samples.iter().enumerate() {
            let got = checked_round_i64(s.num, DENOM, *mode)
                .unwrap_or_else(|| panic!("got None: mode={mode:?}, sample={}", s.label));
            assert_eq!(
                got, expected[idx],
                "mismatch: mode={mode:?}, sample={} (num={}, denom={})",
                s.label, s.num, DENOM
            );
        }
    }
}

#[test]
fn exact_division_no_rounding_all_modes() {
    // rem == 0 harus return atoms langsung.
    let nums = [20, -20, 0, 100, -100];
    let modes = [
        RoundingMode::HalfEven,
        RoundingMode::HalfUp,
        RoundingMode::HalfDown,
        RoundingMode::Floor,
        RoundingMode::Ceil,
        RoundingMode::TowardZero,
        RoundingMode::AwayFromZero,
    ];

    for &n in &nums {
        for &m in &modes {
            let got = checked_round_i64(n, DENOM, m).unwrap();
            // Karena n kelipatan 10, hasil harus n/10 (dan Euclid sama dengan division biasa di sini)
            assert_eq!(got, n / DENOM, "mode={m:?}, n={n}");
        }
    }
}

fn ref_round_i64(number: i64, denom: i64, rounding_mode: RoundingMode) -> Option<i64> {
    // Mirror the preconditions enforced by checked_div_rem_euclid_signed_i64.
    if denom == 0 || denom == i64::MIN {
        return None;
    }
    if number == i64::MIN && denom == -1 {
        return None;
    }

    // For these tests we only use denom > 0. Keep the reference small and explicit.
    debug_assert!(denom > 0);

    let n = number as i128;
    let d = denom as i128;

    let q = n.div_euclid(d); // floor for d > 0
    let r = n.rem_euclid(d); // 0 <= r < d
    if r == 0 {
        return Some(q as i64);
    }

    let q_i64: i64 = q as i64;
    let choose_upper = match rounding_mode {
        RoundingMode::Floor => false,
        RoundingMode::Ceil => true,
        RoundingMode::TowardZero => q_i64.is_negative(),
        RoundingMode::AwayFromZero => !q_i64.is_negative(),
        RoundingMode::HalfEven | RoundingMode::HalfUp | RoundingMode::HalfDown => {
            let d_i128 = d;
            match r.cmp(&(d_i128 - r)) {
                Ordering::Greater => true,
                Ordering::Less => false,
                Ordering::Equal => match rounding_mode {
                    RoundingMode::HalfEven => (q_i64 & 1) != 0,
                    RoundingMode::HalfUp => !q_i64.is_negative(),
                    RoundingMode::HalfDown => q_i64.is_negative(),
                    _ => unreachable!(),
                },
            }
        }
    };

    if choose_upper {
        q_i64.checked_add(1)
    } else {
        Some(q_i64)
    }
}

#[test]
fn none_cases_propagate() {
    let modes = [
        RoundingMode::HalfEven,
        RoundingMode::HalfUp,
        RoundingMode::HalfDown,
        RoundingMode::Floor,
        RoundingMode::Ceil,
        RoundingMode::TowardZero,
        RoundingMode::AwayFromZero,
    ];

    for &m in &modes {
        assert_eq!(checked_round_i64(123, 0, m), None, "mode={m:?}");
        assert_eq!(checked_round_i64(123, i64::MIN, m), None, "mode={m:?}");
        assert_eq!(checked_round_i64(i64::MIN, -1, m), None, "mode={m:?}");
    }
}

#[test]
fn half_even_ties_go_to_even_across_signs() {
    // denom=2 gives us exact x.5 ties as odd numerators.
    let d = 2;
    for k in -9_i64..=9 {
        let n = 2 * k + 1; // k + 0.5
        let got = checked_round_i64(n, d, RoundingMode::HalfEven).unwrap();
        let lo = (n as i128).div_euclid(d as i128) as i64;
        let expected = if (lo & 1) != 0 { lo + 1 } else { lo };
        assert_eq!(got, expected, "n={n}, d={d}, lo={lo}");
    }
}

#[test]
fn symmetry_on_negation_matches_expected_modes() {
    // For denom > 0:
    // - symmetric modes: round(-x) == -round(x)
    // - floor/ceil swap: floor(-x) == -ceil(x), ceil(-x) == -floor(x)
    let d = 10;
    let nums = [-99, -51, -25, -11, -1, 1, 11, 25, 51, 99];

    for &n in &nums {
        for &m in &[
            RoundingMode::HalfEven,
            RoundingMode::HalfUp,
            RoundingMode::HalfDown,
            RoundingMode::TowardZero,
            RoundingMode::AwayFromZero,
        ] {
            let a = checked_round_i64(n, d, m).unwrap();
            let b = checked_round_i64(-n, d, m).unwrap();
            assert_eq!(b, -a, "mode={m:?}, n={n}");
        }

        let f = checked_round_i64(n, d, RoundingMode::Floor).unwrap();
        let c = checked_round_i64(n, d, RoundingMode::Ceil).unwrap();
        let f_neg = checked_round_i64(-n, d, RoundingMode::Floor).unwrap();
        let c_neg = checked_round_i64(-n, d, RoundingMode::Ceil).unwrap();
        assert_eq!(f_neg, -c, "n={n} floor(-x) != -ceil(x)");
        assert_eq!(c_neg, -f, "n={n} ceil(-x) != -floor(x)");
    }
}

#[test]
fn matches_reference_implementation_for_many_samples() {
    let denoms = [1_i64, 2, 3, 10, 16, 99, 1000];
    let modes = [
        RoundingMode::HalfEven,
        RoundingMode::HalfUp,
        RoundingMode::HalfDown,
        RoundingMode::Floor,
        RoundingMode::Ceil,
        RoundingMode::TowardZero,
        RoundingMode::AwayFromZero,
    ];

    for &d in &denoms {
        for n in -500_i64..=500 {
            for &m in &modes {
                let got = checked_round_i64(n, d, m);
                let expected = ref_round_i64(n, d, m);
                assert_eq!(got, expected, "mismatch n={n} d={d} mode={m:?}");
            }
        }
    }
}

fn ref_round_i128(number: i128, denom: i128, rounding_mode: RoundingMode) -> Option<i128> {
    // Mirror the preconditions enforced by checked_div_rem_euclid_signed_i128.
    if denom == 0 || denom == i128::MIN {
        return None;
    }
    if number == i128::MIN && denom == -1 {
        return None;
    }

    // Keep the reference implementation explicit and constrained.
    debug_assert!(denom > 0);

    let q = number.div_euclid(denom); // floor for denom > 0
    let r = number.rem_euclid(denom); // 0 <= r < denom
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
            match r.cmp(&(denom - r)) {
                Ordering::Greater => true,
                Ordering::Less => false,
                Ordering::Equal => match rounding_mode {
                    RoundingMode::HalfEven => (q & 1) != 0,
                    RoundingMode::HalfUp => !is_negative,
                    RoundingMode::HalfDown => is_negative,
                    _ => unreachable!(),
                },
            }
        }
    };

    if choose_upper {
        q.checked_add(1)
    } else {
        Some(q)
    }
}

#[test]
fn rounding_matrix_java_style_i128_small_samples() {
    // Same matrix as i64 test, but exercise the i128 implementation.
    let samples: [i128; 8] = [11, 15, 16, -11, -15, -16, 25, -25];
    let cases: &[(RoundingMode, [i128; 8])] = &[
        (RoundingMode::HalfEven, [1, 2, 2, -1, -2, -2, 2, -2]),
        (RoundingMode::HalfUp, [1, 2, 2, -1, -2, -2, 3, -3]),
        (RoundingMode::HalfDown, [1, 1, 2, -1, -1, -2, 2, -2]),
        (RoundingMode::Floor, [1, 1, 1, -2, -2, -2, 2, -3]),
        (RoundingMode::Ceil, [2, 2, 2, -1, -1, -1, 3, -2]),
        (RoundingMode::TowardZero, [1, 1, 1, -1, -1, -1, 2, -2]),
        (RoundingMode::AwayFromZero, [2, 2, 2, -2, -2, -2, 3, -3]),
    ];

    for (mode, expected) in cases {
        for (idx, &n) in samples.iter().enumerate() {
            let got = checked_round_i128(n, DENOM_128, *mode)
                .unwrap_or_else(|| panic!("got None: mode={mode:?}, sample_idx={idx}"));
            assert_eq!(
                got, expected[idx],
                "mismatch: mode={mode:?}, idx={idx} (n={n}, denom={DENOM_128})"
            );
        }
    }
}

#[test]
fn exact_division_no_rounding_all_modes_i128() {
    let nums: [i128; 5] = [20, -20, 0, 100, -100];
    let modes = [
        RoundingMode::HalfEven,
        RoundingMode::HalfUp,
        RoundingMode::HalfDown,
        RoundingMode::Floor,
        RoundingMode::Ceil,
        RoundingMode::TowardZero,
        RoundingMode::AwayFromZero,
    ];

    for &n in &nums {
        for &m in &modes {
            let got = checked_round_i128(n, DENOM_128, m).unwrap();
            assert_eq!(got, n / DENOM_128, "mode={m:?}, n={n}");
        }
    }
}

#[test]
fn none_cases_propagate_i128() {
    let modes = [
        RoundingMode::HalfEven,
        RoundingMode::HalfUp,
        RoundingMode::HalfDown,
        RoundingMode::Floor,
        RoundingMode::Ceil,
        RoundingMode::TowardZero,
        RoundingMode::AwayFromZero,
    ];

    for &m in &modes {
        assert_eq!(checked_round_i128(123, 0, m), None, "mode={m:?}");
        assert_eq!(checked_round_i128(123, i128::MIN, m), None, "mode={m:?}");
        assert_eq!(checked_round_i128(i128::MIN, -1, m), None, "mode={m:?}");
    }
}

#[test]
fn half_even_ties_go_to_even_across_signs_i128() {
    let d: i128 = 2;
    for k in -50_i128..=50 {
        let n = 2 * k + 1; // k + 0.5
        let got = checked_round_i128(n, d, RoundingMode::HalfEven).unwrap();
        let lo = n.div_euclid(d);
        let expected = if (lo & 1) != 0 { lo + 1 } else { lo };
        assert_eq!(got, expected, "n={n}, d={d}, lo={lo}");
    }
}

#[test]
fn symmetry_on_negation_matches_expected_modes_i128() {
    let d: i128 = 10;
    let nums: [i128; 10] = [-99, -51, -25, -11, -1, 1, 11, 25, 51, 99];

    for &n in &nums {
        for &m in &[
            RoundingMode::HalfEven,
            RoundingMode::HalfUp,
            RoundingMode::HalfDown,
            RoundingMode::TowardZero,
            RoundingMode::AwayFromZero,
        ] {
            let a = checked_round_i128(n, d, m).unwrap();
            let b = checked_round_i128(-n, d, m).unwrap();
            assert_eq!(b, -a, "mode={m:?}, n={n}");
        }

        let f = checked_round_i128(n, d, RoundingMode::Floor).unwrap();
        let c = checked_round_i128(n, d, RoundingMode::Ceil).unwrap();
        let f_neg = checked_round_i128(-n, d, RoundingMode::Floor).unwrap();
        let c_neg = checked_round_i128(-n, d, RoundingMode::Ceil).unwrap();
        assert_eq!(f_neg, -c, "n={n} floor(-x) != -ceil(x)");
        assert_eq!(c_neg, -f, "n={n} ceil(-x) != -floor(x)");
    }
}

#[test]
fn matches_reference_implementation_for_many_samples_i128() {
    let denoms: [i128; 7] = [1, 2, 3, 10, 16, 99, 1000];
    let modes = [
        RoundingMode::HalfEven,
        RoundingMode::HalfUp,
        RoundingMode::HalfDown,
        RoundingMode::Floor,
        RoundingMode::Ceil,
        RoundingMode::TowardZero,
        RoundingMode::AwayFromZero,
    ];

    for &d in &denoms {
        for n in -500_i128..=500 {
            for &m in &modes {
                let got = checked_round_i128(n, d, m);
                let expected = ref_round_i128(n, d, m);
                assert_eq!(got, expected, "mismatch n={n} d={d} mode={m:?}");
            }
        }
    }
}
