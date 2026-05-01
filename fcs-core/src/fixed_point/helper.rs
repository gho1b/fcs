macro_rules! fn_checked_div_rem_euclid_signed {
    ($fn_name:ident, $t:ty) => {
        #[inline]
        #[must_use]
        pub(crate) fn $fn_name(a: $t, b: $t) -> Option<($t, $t, $t)> {
            if b == 0 {
                return None;
            }

            // Kamu ingin mengembalikan abs(b) sebagai $t.
            // Ini overflow saat b == $t::MIN, jadi kita kembalikan None (konsisten dengan fungsi kamu).
            let div = b.checked_abs()?;

            // Truncating division & remainder (seperti / dan % di Rust)
            let q0 = a.checked_div(b)?;
            let r0 = a.checked_rem(b)?;

            // Jika remainder sudah >= 0, itu sudah Euclidean remainder (range 0..|b|)
            if r0 >= 0 {
                return Some((q0, r0, div));
            }

            // Jika r0 negatif, lakukan penyesuaian 1 langkah agar r menjadi non-negatif
            if b > 0 {
                let q = q0.checked_sub(1)?;
                let r = r0.checked_add(b)?; // b positif
                Some((q, r, div))
            } else {
                let q = q0.checked_add(1)?;
                let r = r0.checked_sub(b)?; // b negatif, jadi r0 - b = r0 + |b|
                Some((q, r, div))
            }
        }
    };
}

macro_rules! fn_checked_div_rem_euclid_unsigned {
    ($fn_name:ident, $t:ty, $t2:ty) => {
        #[inline]
        #[must_use]
        pub(crate) fn $fn_name(a: $t, b: $t) -> Option<($t, $t2, $t2)> {
            if b == 0 {
                return None;
            }

            let div = b as $t2;

            // Truncating division & remainder (seperti / dan % di Rust)
            let q0 = a.checked_div(b)?;
            let r0 = a.checked_rem(b)?;

            // Jika remainder sudah >= 0, itu sudah Euclidean remainder (range 0..|b|)
            if r0 >= 0 {
                return Some((q0, r0 as $t2, div));
            }

            // Jika r0 negatif, lakukan penyesuaian 1 langkah agar r menjadi non-negatif
            if b > 0 {
                let q = q0.checked_sub(1)?;
                let r = r0.checked_add(b)?; // b positif
                Some((q, r as $t2, div))
            } else {
                let q = q0.checked_add(1)?;
                let r = r0.checked_sub(b)?; // b negatif, jadi r0 - b = r0 + |b|
                Some((q, r as $t2, div))
            }
        }
    };
}
macro_rules! fn_div_rem_single_pass {
    ($fn_name:ident, $t:ty) => {
        #[inline]
        #[must_use]
        pub(crate) fn $fn_name(a: $t, b: $t) -> ($t, $t, $t) {
            // panics if b == 0 or (a == MIN && b == -1) as per Rust's / and % rules
            let q0 = a / b;
            let r0 = a % b;

            // abs(b) as i128 will overflow for b == MIN; if you need div for that case,
            // you must return u128 instead. Keeping i128 here matches your earlier "checked_abs" limitation.
            let div = b.abs(); // will panic if b == MIN when using abs() on signed

            if r0 >= 0 {
                return (q0, r0, div);
            }

            if b > 0 {
                (q0 - 1, r0 + b, div)
            } else {
                (q0 + 1, r0 - b, div)
            }
        }
    };
}

fn_checked_div_rem_euclid_signed!(checked_div_rem_euclid_signed_i64, i64);
fn_checked_div_rem_euclid_signed!(checked_div_rem_euclid_signed_i128, i128);
fn_checked_div_rem_euclid_unsigned!(checked_div_rem_euclid_unsigned_i64, i64, u64);
fn_checked_div_rem_euclid_unsigned!(checked_div_rem_euclid_unsigned_i128, i128, u128);
fn_div_rem_single_pass!(div_rem_euclid_single_pass_i64, i64);
fn_div_rem_single_pass!(div_rem_euclid_single_pass_i128, i128);

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_euclid_i64(a: i64, b: i64) {
        let (q, r, d) = checked_div_rem_euclid_signed_i64(a, b).expect("expected Some");

        // Our contract: `d == abs(b)` and `r` is Euclidean (0 <= r < d).
        assert!(d > 0);
        assert_eq!(d, b.abs());
        assert!(r >= 0 && r < d, "r={r} d={d} (a={a}, b={b})");

        // For i64 we can always recompose safely in i128.
        let lhs = (q as i128) * (b as i128) + (r as i128);
        assert_eq!(lhs, a as i128);

        // Cross-check against stdlib checked Euclidean division.
        assert_eq!(q, a.checked_div_euclid(b).expect("expected Some"));
        assert_eq!(r, a.checked_rem_euclid(b).expect("expected Some"));

        // Unsigned remainder helper is only well-defined for positive divisors.
        if b > 0 {
            let (qu, ru, du) = checked_div_rem_euclid_unsigned_i64(a, b).expect("expected Some");
            assert_eq!(qu, q);
            assert_eq!(ru, r as u64);
            assert_eq!(du, d as u64);
        }

        // Single-pass variant should match core's div_euclid/rem_euclid for safe inputs.
        let (q2, r2, d2) = div_rem_euclid_single_pass_i64(a, b);
        assert_eq!(q2, a.div_euclid(b));
        assert_eq!(r2, a.rem_euclid(b));
        assert_eq!(d2, b.abs());
    }

    fn assert_euclid_i128(a: i128, b: i128) {
        let (q, r, d) = checked_div_rem_euclid_signed_i128(a, b).expect("expected Some");

        assert!(d > 0);
        assert_eq!(d, b.abs());
        assert!(r >= 0 && r < d, "r={r} d={d} (a={a}, b={b})");

        // Recomposition can overflow `i128` even when `a` is representable
        // (e.g., q*b underflows below i128::MIN and then +r brings it back).
        // Only assert recomposition when it is safe.
        if let Some(lhs) = q.checked_mul(b).and_then(|x| x.checked_add(r)) {
            assert_eq!(lhs, a);
        }

        assert_eq!(q, a.checked_div_euclid(b).expect("expected Some"));
        assert_eq!(r, a.checked_rem_euclid(b).expect("expected Some"));

        if b > 0 {
            let (qu, ru, du) = checked_div_rem_euclid_unsigned_i128(a, b).expect("expected Some");
            assert_eq!(qu, q);
            assert_eq!(ru, r as u128);
            assert_eq!(du, d as u128);
        }

        let (q2, r2, d2) = div_rem_euclid_single_pass_i128(a, b);
        assert_eq!(q2, a.div_euclid(b));
        assert_eq!(r2, a.rem_euclid(b));
        assert_eq!(d2, b.abs());
    }

    #[test]
    fn checked_div_rem_euclid_signed_i64_none_on_zero_divisor() {
        assert_eq!(checked_div_rem_euclid_signed_i64(123, 0), None);
    }

    #[test]
    fn checked_div_rem_euclid_signed_i64_none_on_min_divisor() {
        assert_eq!(checked_div_rem_euclid_signed_i64(123, i64::MIN), None);
    }

    #[test]
    fn checked_div_rem_euclid_signed_i64_none_on_min_over_minus_one() {
        assert_eq!(checked_div_rem_euclid_signed_i64(i64::MIN, -1), None);
    }

    #[test]
    fn checked_div_rem_euclid_signed_i64_produces_euclidean_remainder() {
        // hand-picked edge-ish cases across sign combinations
        for (a, b) in [
            (7, 3),
            (-7, 3),
            (7, -3),
            (-7, -3),
            (0, 3),
            (1, 3),
            (-1, 3),
            (i64::MAX, 7),
            (i64::MIN + 1, 7),
            (i64::MIN + 1, -7),
        ] {
            assert_euclid_i64(a, b);
        }
    }

    #[test]
    fn checked_div_rem_euclid_signed_i128_produces_euclidean_remainder() {
        for (a, b) in [
            (7_i128, 3_i128),
            (-7_i128, 3_i128),
            (7_i128, -3_i128),
            (-7_i128, -3_i128),
            (i128::MAX, 9_i128),
            (-(1_i128 << 100) + 123, 9_i128),
            (-(1_i128 << 100) + 123, -9_i128),
        ] {
            assert_euclid_i128(a, b);
        }
    }

    #[test]
    fn checked_div_rem_euclid_unsigned_i64_matches_signed_for_positive_divisor() {
        for (a, b) in [(7_i64, 3_i64), (-7_i64, 3_i64), (i64::MAX, 7_i64)] {
            let (qs, rs, ds) = checked_div_rem_euclid_signed_i64(a, b).unwrap();
            let (qu, ru, du) = checked_div_rem_euclid_unsigned_i64(a, b).unwrap();
            assert_eq!(qs, qu);
            assert_eq!(rs as u64, ru);
            assert_eq!(ds as u64, du);
        }
    }

    #[test]
    fn checked_div_rem_euclid_unsigned_i128_matches_signed_for_positive_divisor() {
        for (a, b) in [(7_i128, 3_i128), (-7_i128, 3_i128), (i128::MAX, 9_i128)] {
            let (qs, rs, ds) = checked_div_rem_euclid_signed_i128(a, b).unwrap();
            let (qu, ru, du) = checked_div_rem_euclid_unsigned_i128(a, b).unwrap();
            assert_eq!(qs, qu);
            assert_eq!(rs as u128, ru);
            assert_eq!(ds as u128, du);
        }
    }

    #[test]
    fn div_rem_euclid_single_pass_i64_matches_checked_for_safe_inputs() {
        for (a, b) in [
            (7_i64, 3_i64),
            (-7_i64, 3_i64),
            (7_i64, -3_i64),
            (-7_i64, -3_i64),
            (i64::MAX, 7_i64),
            (i64::MIN + 1, 7_i64),
        ] {
            let (q1, r1, d1) = checked_div_rem_euclid_signed_i64(a, b).unwrap();
            let (q2, r2, d2) = div_rem_euclid_single_pass_i64(a, b);
            assert_eq!((q1, r1, d1), (q2, r2, d2));
        }
    }

    #[test]
    #[should_panic]
    fn div_rem_euclid_single_pass_i64_panics_on_zero_divisor() {
        let _ = div_rem_euclid_single_pass_i64(1, 0);
    }

    #[test]
    #[should_panic]
    fn div_rem_euclid_single_pass_i64_panics_on_min_divisor_abs_overflow() {
        // b.abs() panics for MIN.
        let _ = div_rem_euclid_single_pass_i64(1, i64::MIN);
    }

    #[test]
    fn div_rem_euclid_single_pass_i128_matches_checked_for_safe_inputs() {
        for (a, b) in [
            (7_i128, 3_i128),
            (-7_i128, 3_i128),
            (7_i128, -3_i128),
            (-7_i128, -3_i128),
            (i128::MAX, 9_i128),
            (i128::MIN + 1, 9_i128),
        ] {
            let (q1, r1, d1) = checked_div_rem_euclid_signed_i128(a, b).unwrap();
            let (q2, r2, d2) = div_rem_euclid_single_pass_i128(a, b);
            assert_eq!((q1, r1, d1), (q2, r2, d2));
        }
    }

    #[test]
    #[should_panic]
    fn div_rem_euclid_single_pass_i128_panics_on_min_divisor_abs_overflow() {
        let _ = div_rem_euclid_single_pass_i128(1, i128::MIN);
    }
}
