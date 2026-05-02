macro_rules! fn_checked_div_rem_euclid_signed {
    ($fn_name:ident, $t:ty) => {
        #[inline]
        #[must_use]
        pub(crate) fn $fn_name(a: $t, b: $t) -> Option<($t, $t, $t)> {
            if b == 0 {
                return None;
            }

            // Positive divisor magnitude. This fails for MIN because abs(MIN)
            // is not representable in signed integer.
            let div = b.checked_abs()?;

            let q0 = a.checked_div(b)?;
            let r0 = a.checked_rem(b)?;

            // Rust remainder has the same sign as `a`.
            // If it is already non-negative, it is Euclidean.
            if r0 >= 0 {
                return Some((q0, r0, div));
            }

            // Adjust one step so:
            // a = q * b + r
            // 0 <= r < abs(b)
            if b > 0 {
                let q = q0.checked_sub(1)?;
                let r = r0.checked_add(b)?;
                Some((q, r, div))
            } else {
                let q = q0.checked_add(1)?;
                let r = r0.checked_sub(b)?;
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
mod tests;
