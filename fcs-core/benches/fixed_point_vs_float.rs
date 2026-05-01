use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fcs_core::{FixedPoint, RoundingMode};
use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::{Decimal, RoundingStrategy};
use std::hint::black_box;

fn build_i64_inputs(n: usize) -> Vec<i64> {
    // Deterministic pseudo-random-ish sequence without pulling in RNG crates.
    let mut x = 0x9E3779B97F4A7C15_u64;
    let mut out = Vec::with_capacity(n);
    for _ in 0..n {
        // splitmix64
        x = x.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = x;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^= z >> 31;
        // keep values in a safe-ish range to avoid overflow dominating
        let v = (z as i64) % 1_000_000_000;
        out.push(v);
    }
    out
}

fn round_f64(v: f64, mode: RoundingMode) -> f64 {
    match mode {
        RoundingMode::HalfEven => v.round_ties_even(),
        RoundingMode::HalfCeil => {
            // Round to nearest; ties go to the greater numeric result (toward +∞).
            let floor = v.floor();
            let ceil = v.ceil();
            let df = (v - floor).abs();
            let dc = (ceil - v).abs();
            if df < dc {
                floor
            } else if dc < df {
                ceil
            } else {
                ceil
            }
        }
        RoundingMode::HalfFloor => {
            // Round to nearest; ties go to the smaller numeric result (toward -∞).
            let floor = v.floor();
            let ceil = v.ceil();
            let df = (v - floor).abs();
            let dc = (ceil - v).abs();
            if df < dc {
                floor
            } else if dc < df {
                ceil
            } else {
                floor
            }
        }
        RoundingMode::Floor => v.floor(),
        RoundingMode::Ceil => v.ceil(),
        RoundingMode::TowardZero => v.trunc(),
        RoundingMode::AwayFromZero => {
            if v.is_sign_negative() {
                v.floor()
            } else {
                v.ceil()
            }
        }
        _ => unreachable!(),
    }
}

fn rounding_strategy(mode: RoundingMode) -> RoundingStrategy {
    match mode {
        RoundingMode::HalfEven => RoundingStrategy::MidpointNearestEven,
        RoundingMode::HalfCeil => RoundingStrategy::MidpointAwayFromZero,
        RoundingMode::HalfFloor => RoundingStrategy::MidpointTowardZero,
        RoundingMode::Floor => RoundingStrategy::ToNegativeInfinity,
        RoundingMode::Ceil => RoundingStrategy::ToPositiveInfinity,
        RoundingMode::TowardZero => RoundingStrategy::ToZero,
        RoundingMode::AwayFromZero => RoundingStrategy::AwayFromZero,
        _ => unreachable!(),
    }
}

fn dec_from_atoms(atoms: i64, scale: i64) -> Decimal {
    // scale is power-of-10; map to decimal places.
    let dp = if scale == 1 { 0 } else { scale.ilog10() as u32 };
    let mut d = Decimal::from_i64(atoms).expect("i64 -> Decimal");
    if dp != 0 {
        d.set_scale(dp).expect("set_scale");
    }
    d
}

fn dec_quantize(d: Decimal, target_scale: i64, mode: RoundingMode) -> Decimal {
    let dp = if target_scale == 1 {
        0
    } else {
        target_scale.ilog10() as u32
    };
    d.round_dp_with_strategy(dp, rounding_strategy(mode))
}

fn bench_add(c: &mut Criterion) {
    let scale = 100;
    let inputs = build_i64_inputs(10_000);

    let fp: Vec<FixedPoint> = inputs.iter().map(|&a| FixedPoint::new(a, scale)).collect();
    let fl: Vec<f64> = inputs
        .iter()
        .map(|&a| (a as f64) / (scale as f64))
        .collect();
    let dec: Vec<Decimal> = inputs.iter().map(|&a| dec_from_atoms(a, scale)).collect();

    let mut group = c.benchmark_group("add");

    group.bench_function("fixed_point", |b| {
        b.iter(|| {
            let mut acc = FixedPoint::new(0, scale);
            for v in &fp {
                acc = black_box(acc + *v);
            }
            black_box(acc)
        })
    });

    group.bench_function("f64", |b| {
        b.iter(|| {
            let mut acc = 0.0_f64;
            for v in &fl {
                acc = black_box(acc + *v);
            }
            black_box(acc)
        })
    });

    group.bench_function("decimal", |b| {
        b.iter(|| {
            let mut acc = Decimal::ZERO;
            for v in &dec {
                acc = black_box(acc + *v);
            }
            black_box(acc)
        })
    });

    group.finish();
}

fn bench_quantize(c: &mut Criterion) {
    let from_scale = 1_000;
    let to_scale = 100;
    let inputs = build_i64_inputs(10_000);
    let fp: Vec<FixedPoint> = inputs
        .iter()
        .map(|&a| FixedPoint::new(a, from_scale))
        .collect();
    let fl: Vec<f64> = inputs
        .iter()
        .map(|&a| (a as f64) / (from_scale as f64))
        .collect();
    let dec: Vec<Decimal> = inputs
        .iter()
        .map(|&a| dec_from_atoms(a, from_scale))
        .collect();

    let modes = [
        RoundingMode::HalfEven,
        RoundingMode::HalfCeil,
        RoundingMode::Floor,
        RoundingMode::Ceil,
        RoundingMode::TowardZero,
        RoundingMode::AwayFromZero,
    ];

    let mut group = c.benchmark_group("quantize");

    for mode in modes {
        group.bench_with_input(
            BenchmarkId::new("fixed_point", format!("{mode:?}")),
            &mode,
            |b, &m| {
                b.iter(|| {
                    let mut acc = 0_i64;
                    for v in &fp {
                        let q = black_box(v.try_quantize(to_scale, m).unwrap());
                        acc = acc.wrapping_add(q.atoms());
                    }
                    black_box(acc)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("f64", format!("{mode:?}")),
            &mode,
            |b, &m| {
                b.iter(|| {
                    let mut acc = 0.0_f64;
                    let q = to_scale as f64;
                    for &v in &fl {
                        // quantize to 2 decimals: round(v * 100) / 100
                        let scaled = v * q;
                        let rounded = round_f64(scaled, m);
                        acc = black_box(acc + rounded / q);
                    }
                    black_box(acc)
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("decimal", format!("{mode:?}")),
            &mode,
            |b, &m| {
                b.iter(|| {
                    let mut acc = Decimal::ZERO;
                    for &v in &dec {
                        let q = black_box(dec_quantize(v, to_scale, m));
                        acc = black_box(acc + q);
                    }
                    black_box(acc)
                })
            },
        );
    }

    group.finish();
}

fn bench_div_round(c: &mut Criterion) {
    let scale = 100;
    let inputs = build_i64_inputs(10_000);
    let divisors = [3_i64, 7, -3, -7];
    let mode = RoundingMode::HalfEven;

    let fp: Vec<FixedPoint> = inputs.iter().map(|&a| FixedPoint::new(a, scale)).collect();
    let fl: Vec<f64> = inputs
        .iter()
        .map(|&a| (a as f64) / (scale as f64))
        .collect();
    let dec: Vec<Decimal> = inputs.iter().map(|&a| dec_from_atoms(a, scale)).collect();

    let mut group = c.benchmark_group("div_round");

    for &d in &divisors {
        group.bench_with_input(BenchmarkId::new("fixed_point", d), &d, |b, &div| {
            b.iter(|| {
                let mut acc = 0_i64;
                for v in &fp {
                    let r = black_box(v.try_div_i64(div).unwrap());
                    let q = black_box(r.to_fixed_point(mode));
                    acc = acc.wrapping_add(q.atoms());
                }
                black_box(acc)
            })
        });

        group.bench_with_input(BenchmarkId::new("f64", d), &d, |b, &div| {
            b.iter(|| {
                let mut acc = 0.0_f64;
                let divf = div as f64;
                for &v in &fl {
                    let q = v / divf;
                    acc = black_box(acc + round_f64(q, mode));
                }
                black_box(acc)
            })
        });

        group.bench_with_input(BenchmarkId::new("decimal", d), &d, |b, &div| {
            b.iter(|| {
                let mut acc = Decimal::ZERO;
                let divd = Decimal::from_i64(div).unwrap();
                for &v in &dec {
                    let q = v / divd;
                    let rq = q.round_dp_with_strategy(0, rounding_strategy(mode));
                    acc = black_box(acc + rq);
                }
                black_box(acc)
            })
        });
    }

    group.finish();
}

fn bench_regulatory_round_reembed(c: &mut Criterion) {
    // Regulatory scenario:
    // - value is stored at a fine scale (e.g. 3 decimals, scale=1000)
    // - regulation mandates rounding at whole units (scale=1)
    // - result must be re-embedded back into the original scale
    //
    // Example: atoms=12345, scale=1000 -> round to scale=1 => 12 -> back to scale=1000 => 12000
    let working_scale_i = 1_000;
    let regulatory_scale_i = 1;
    let working_scale_f = working_scale_i as f64;
    let regulatory_scale_f = regulatory_scale_i as f64;

    let mode = RoundingMode::HalfEven;

    let inputs = build_i64_inputs(10_000);
    let fp: Vec<FixedPoint> = inputs
        .iter()
        .map(|&a| FixedPoint::new(a, working_scale_i))
        .collect();
    let fl: Vec<f64> = inputs
        .iter()
        .map(|&a| (a as f64) / working_scale_f)
        .collect();
    let dec: Vec<Decimal> = inputs
        .iter()
        .map(|&a| dec_from_atoms(a, working_scale_i))
        .collect();

    let mut group = c.benchmark_group("regulatory_round_reembed");

    group.bench_function("fixed_point/quantize_then_rescale", |b| {
        b.iter(|| {
            let mut acc = 0_i64;
            for v in &fp {
                let reg = black_box(v.try_quantize(regulatory_scale_i, mode).unwrap());
                let re = black_box(reg.try_rescale_exact(working_scale_i).unwrap());
                acc = acc.wrapping_add(re.atoms());
            }
            black_box(acc)
        })
    });

    // normalize_to is currently an alias of quantize; keep this bench as a guard if semantics change.
    group.bench_function("fixed_point/normalize_to_then_rescale", |b| {
        b.iter(|| {
            let mut acc = 0_i64;
            for v in &fp {
                let reg = black_box(v.try_normalize_to(regulatory_scale_i, mode).unwrap());
                let re = black_box(reg.try_rescale_exact(working_scale_i).unwrap());
                acc = acc.wrapping_add(re.atoms());
            }
            black_box(acc)
        })
    });

    group.bench_function("f64/round_then_reembed", |b| {
        b.iter(|| {
            let mut acc = 0.0_f64;
            for &v in &fl {
                let reg = round_f64(v * regulatory_scale_f, mode) / regulatory_scale_f;
                let re = round_f64(reg * working_scale_f, mode) / working_scale_f;
                acc = black_box(acc + re);
            }
            black_box(acc)
        })
    });

    group.bench_function("decimal/round_then_reembed", |b| {
        b.iter(|| {
            let mut acc = Decimal::ZERO;
            for &v in &dec {
                let reg = black_box(dec_quantize(v, regulatory_scale_i, mode));
                let re = black_box(dec_quantize(reg, working_scale_i, mode));
                acc = black_box(acc + re);
            }
            black_box(acc)
        })
    });

    group.finish();
}

fn compute_tax_fixed_point(
    base: FixedPoint,
    rate_bps: i64,
    compute_scale: i64,
    regulatory_scale: i64,
    working_scale: i64,
    mode: RoundingMode,
) -> FixedPoint {
    // base: fixed-point at `working_scale`
    // rate_bps: basis points, i.e. 10000 = 100%
    // compute_scale: intermediate scale for tax computation (e.g. 1000)
    //
    // tax_raw scale = compute_scale:
    // atoms = base.atoms * rate_bps * (compute_scale / working_scale) / 10000
    let scale_factor = (compute_scale / working_scale) as i128;
    let numer = (base.atoms() as i128) * (rate_bps as i128) * scale_factor;
    let numer_i64: i64 = numer.try_into().unwrap();

    // Represent "numer / 10000" as a higher-scale FixedPoint, then quantize down.
    // scale_hi = compute_scale * 10000 (both are powers of 10).
    let scale_hi = compute_scale * 10_000;
    let raw_hi = FixedPoint::new(numer_i64, scale_hi);
    let raw = raw_hi.try_quantize(compute_scale, mode).unwrap();

    // regulatory rounding, then store back to working scale
    let reg = raw.try_quantize(regulatory_scale, mode).unwrap();
    reg.try_rescale_exact(working_scale).unwrap()
}

fn compute_tax_raw_fixed_point(
    base: FixedPoint,
    rate_bps: i64,
    compute_scale: i64,
    working_scale: i64,
    mode: RoundingMode,
) -> FixedPoint {
    // Returns tax amount at `compute_scale`, rounded from the exact rational result.
    let scale_factor = (compute_scale / working_scale) as i128;
    let numer = (base.atoms() as i128) * (rate_bps as i128) * scale_factor;

    let numer_i64: i64 = numer.try_into().unwrap();
    let scale_hi = compute_scale * 10_000;
    let raw_hi = FixedPoint::new(numer_i64, scale_hi);
    raw_hi.try_quantize(compute_scale, mode).unwrap()
}

fn compute_tax_f64(
    base: f64,
    rate_bps: i64,
    compute_scale: f64,
    regulatory_scale: f64,
    working_scale: f64,
    mode: RoundingMode,
) -> f64 {
    // base is already in major units (e.g. 12.34)
    let rate = (rate_bps as f64) / 10_000.0;
    let raw = base * rate;

    // quantize to compute_scale (e.g. 3 decimals), then to regulatory, then back to working
    let raw_q = round_f64(raw * compute_scale, mode) / compute_scale;
    let reg = round_f64(raw_q * regulatory_scale, mode) / regulatory_scale;
    round_f64(reg * working_scale, mode) / working_scale
}

fn compute_tax_raw_f64(base: f64, rate_bps: i64, compute_scale: f64, mode: RoundingMode) -> f64 {
    let rate = (rate_bps as f64) / 10_000.0;
    let raw = base * rate;
    round_f64(raw * compute_scale, mode) / compute_scale
}

fn bench_accounting_tax_pipeline(c: &mut Criterion) {
    // Scenario:
    // - base amounts stored at 2 decimals (scale=100)
    // - compute tax at 3 decimals (compute_scale=1000)
    // - regulatory rounding requires whole units (regulatory_scale=1)
    // - store back at 2 decimals
    //
    // Also compares per-line rounding vs total rounding.
    let working_scale_i = 100;
    let compute_scale_i = 1_000;
    let regulatory_scale_i = 1;

    let working_scale_f = working_scale_i as f64;
    let compute_scale_f = compute_scale_i as f64;
    let regulatory_scale_f = regulatory_scale_i as f64;

    let rate_bps = 1_100_i64; // 11%
    let mode = RoundingMode::HalfEven;

    let inputs = build_i64_inputs(10_000);
    let bases_fp: Vec<FixedPoint> = inputs
        .iter()
        .map(|&a| FixedPoint::new(a, working_scale_i))
        .collect();
    let bases_f64: Vec<f64> = inputs
        .iter()
        .map(|&a| (a as f64) / working_scale_f)
        .collect();
    let bases_dec: Vec<Decimal> = inputs
        .iter()
        .map(|&a| dec_from_atoms(a, working_scale_i))
        .collect();

    let mut group = c.benchmark_group("accounting_tax_pipeline");

    group.bench_function("fixed_point/per_line", |b| {
        b.iter(|| {
            let mut acc = FixedPoint::new(0, working_scale_i);
            for &base in &bases_fp {
                let tax = compute_tax_fixed_point(
                    base,
                    rate_bps,
                    compute_scale_i,
                    regulatory_scale_i,
                    working_scale_i,
                    mode,
                );
                acc = black_box(acc + tax);
            }
            black_box(acc)
        })
    });

    group.bench_function("fixed_point/total_then_round", |b| {
        b.iter(|| {
            let mut base_sum = FixedPoint::new(0, working_scale_i);
            for &base in &bases_fp {
                base_sum = black_box(base_sum + base);
            }
            let tax = compute_tax_fixed_point(
                base_sum,
                rate_bps,
                compute_scale_i,
                regulatory_scale_i,
                working_scale_i,
                mode,
            );
            black_box(tax)
        })
    });

    group.bench_function("f64/per_line", |b| {
        b.iter(|| {
            let mut acc = 0.0_f64;
            for &base in &bases_f64 {
                let tax = compute_tax_f64(
                    base,
                    rate_bps,
                    compute_scale_f,
                    regulatory_scale_f,
                    working_scale_f,
                    mode,
                );
                acc = black_box(acc + tax);
            }
            black_box(acc)
        })
    });

    group.bench_function("f64/total_then_round", |b| {
        b.iter(|| {
            let mut base_sum = 0.0_f64;
            for &base in &bases_f64 {
                base_sum = black_box(base_sum + base);
            }
            let tax = compute_tax_f64(
                base_sum,
                rate_bps,
                compute_scale_f,
                regulatory_scale_f,
                working_scale_f,
                mode,
            );
            black_box(tax)
        })
    });

    group.bench_function("decimal/per_line", |b| {
        b.iter(|| {
            let mut acc = Decimal::ZERO;
            let rate = Decimal::from_i64(rate_bps).unwrap() / Decimal::from_i64(10_000).unwrap();
            for &base in &bases_dec {
                let raw = base * rate;
                let raw_q = dec_quantize(raw, compute_scale_i, mode);
                let reg = dec_quantize(raw_q, regulatory_scale_i, mode);
                let stored = dec_quantize(reg, working_scale_i, mode);
                acc = black_box(acc + stored);
            }
            black_box(acc)
        })
    });

    group.bench_function("decimal/total_then_round", |b| {
        b.iter(|| {
            let mut base_sum = Decimal::ZERO;
            for &base in &bases_dec {
                base_sum = black_box(base_sum + base);
            }
            let rate = Decimal::from_i64(rate_bps).unwrap() / Decimal::from_i64(10_000).unwrap();
            let raw = base_sum * rate;
            let raw_q = dec_quantize(raw, compute_scale_i, mode);
            let reg = dec_quantize(raw_q, regulatory_scale_i, mode);
            let stored = dec_quantize(reg, working_scale_i, mode);
            black_box(stored)
        })
    });

    // More realistic tax engine variant:
    // - compute per-line raw tax at compute_scale
    // - bucket aggregate (e.g., per jurisdiction/product group)
    // - apply regulatory rounding once per bucket
    let bucket_count = 64usize;

    group.bench_function("fixed_point/bucketed_round", |b| {
        b.iter(|| {
            let mut buckets = vec![FixedPoint::new(0, compute_scale_i); bucket_count];

            for (i, &base) in bases_fp.iter().enumerate() {
                let raw = compute_tax_raw_fixed_point(
                    base,
                    rate_bps,
                    compute_scale_i,
                    working_scale_i,
                    mode,
                );
                let idx = i & (bucket_count - 1);
                buckets[idx] = black_box(buckets[idx] + raw);
            }

            let mut acc = FixedPoint::new(0, working_scale_i);
            for bucket in buckets {
                let reg = bucket.try_quantize(regulatory_scale_i, mode).unwrap();
                let stored = reg.try_rescale_exact(working_scale_i).unwrap();
                acc = black_box(acc + stored);
            }

            black_box(acc)
        })
    });

    group.bench_function("f64/bucketed_round", |b| {
        b.iter(|| {
            let mut buckets = vec![0.0_f64; bucket_count];

            for (i, &base) in bases_f64.iter().enumerate() {
                let raw = compute_tax_raw_f64(base, rate_bps, compute_scale_f, mode);
                let idx = i & (bucket_count - 1);
                buckets[idx] = black_box(buckets[idx] + raw);
            }

            let mut acc = 0.0_f64;
            for bucket in buckets {
                let reg = round_f64(bucket * regulatory_scale_f, mode) / regulatory_scale_f;
                let stored = round_f64(reg * working_scale_f, mode) / working_scale_f;
                acc = black_box(acc + stored);
            }

            black_box(acc)
        })
    });

    group.bench_function("decimal/bucketed_round", |b| {
        b.iter(|| {
            let mut buckets = vec![Decimal::ZERO; bucket_count];
            let rate = Decimal::from_i64(rate_bps).unwrap() / Decimal::from_i64(10_000).unwrap();

            for (i, &base) in bases_dec.iter().enumerate() {
                let raw = dec_quantize(base * rate, compute_scale_i, mode);
                let idx = i & (bucket_count - 1);
                buckets[idx] = black_box(buckets[idx] + raw);
            }

            let mut acc = Decimal::ZERO;
            for bucket in buckets {
                let reg = dec_quantize(bucket, regulatory_scale_i, mode);
                let stored = dec_quantize(reg, working_scale_i, mode);
                acc = black_box(acc + stored);
            }

            black_box(acc)
        })
    });

    // Even more "tax-engine-like":
    // - group-by key (e.g., jurisdiction + product category) using a hash map
    // - raw tax computed per-line, summed per key at compute_scale
    // - regulatory rounding applied once per key
    //
    // This includes hashing/entry overhead which is common in real tax engines.
    let key_count = 1024u32;
    let keys: Vec<u32> = inputs
        .iter()
        .map(|&a| {
            // Deterministic key derivation from atoms, spread bits.
            let x = (a as u64).wrapping_mul(0x9E3779B97F4A7C15);
            ((x >> 32) as u32) & (key_count - 1)
        })
        .collect();

    group.bench_function("fixed_point/hashmap_round", |b| {
        use std::collections::HashMap;

        b.iter(|| {
            let mut map: HashMap<u32, FixedPoint> = HashMap::with_capacity(key_count as usize);

            for (i, &base) in bases_fp.iter().enumerate() {
                let raw = compute_tax_raw_fixed_point(
                    base,
                    rate_bps,
                    compute_scale_i,
                    working_scale_i,
                    mode,
                );
                let key = keys[i];
                let entry = map
                    .entry(key)
                    .or_insert_with(|| FixedPoint::new(0, compute_scale_i));
                *entry = black_box(*entry + raw);
            }

            let mut acc = FixedPoint::new(0, working_scale_i);
            for (_k, bucket) in map {
                let reg = bucket.try_quantize(regulatory_scale_i, mode).unwrap();
                let stored = reg.try_rescale_exact(working_scale_i).unwrap();
                acc = black_box(acc + stored);
            }

            black_box(acc)
        })
    });

    group.bench_function("f64/hashmap_round", |b| {
        use std::collections::HashMap;

        b.iter(|| {
            let mut map: HashMap<u32, f64> = HashMap::with_capacity(key_count as usize);

            for (i, &base) in bases_f64.iter().enumerate() {
                let raw = compute_tax_raw_f64(base, rate_bps, compute_scale_f, mode);
                let key = keys[i];
                let entry = map.entry(key).or_insert(0.0);
                *entry = black_box(*entry + raw);
            }

            let mut acc = 0.0_f64;
            for (_k, bucket) in map {
                let reg = round_f64(bucket * regulatory_scale_f, mode) / regulatory_scale_f;
                let stored = round_f64(reg * working_scale_f, mode) / working_scale_f;
                acc = black_box(acc + stored);
            }

            black_box(acc)
        })
    });

    group.bench_function("decimal/hashmap_round", |b| {
        use std::collections::HashMap;

        b.iter(|| {
            let mut map: HashMap<u32, Decimal> = HashMap::with_capacity(key_count as usize);
            let rate = Decimal::from_i64(rate_bps).unwrap() / Decimal::from_i64(10_000).unwrap();

            for (i, &base) in bases_dec.iter().enumerate() {
                let raw = dec_quantize(base * rate, compute_scale_i, mode);
                let key = keys[i];
                let entry = map.entry(key).or_insert(Decimal::ZERO);
                *entry = black_box(*entry + raw);
            }

            let mut acc = Decimal::ZERO;
            for (_k, bucket) in map {
                let reg = dec_quantize(bucket, regulatory_scale_i, mode);
                let stored = dec_quantize(reg, working_scale_i, mode);
                acc = black_box(acc + stored);
            }

            black_box(acc)
        })
    });

    group.finish();
}

fn bench_bank_reconcile(c: &mut Criterion) {
    // Scenario:
    // - Many transactions across many accounts
    // - Ledger and bank statement each provide per-account ending balances
    // - Reconcile computes delta per account and counts mismatches
    //
    // This is a typical banking reconcile shape: group-by + accumulate.
    let account_count: u32 = 1024;
    let tx_count: usize = 50_000;
    let scale_i = 100; // cents
    let scale_f = scale_i as f64;
    let mode = RoundingMode::HalfEven;

    // Deterministic transaction amounts (atoms) and account ids.
    let amounts = build_i64_inputs(tx_count);
    let account_ids: Vec<u32> = amounts
        .iter()
        .map(|&a| {
            let x = (a as u64).wrapping_mul(0x9E3779B97F4A7C15);
            ((x >> 32) as u32) & (account_count - 1)
        })
        .collect();

    // Build ledger transactions.
    let tx_fp: Vec<(u32, FixedPoint)> = account_ids
        .iter()
        .copied()
        .zip(amounts.iter().copied())
        .map(|(acct, atoms)| (acct, FixedPoint::new(atoms, scale_i)))
        .collect();
    let tx_f64: Vec<(u32, f64)> = account_ids
        .iter()
        .copied()
        .zip(amounts.iter().copied())
        .map(|(acct, atoms)| (acct, (atoms as f64) / scale_f))
        .collect();
    let tx_dec: Vec<(u32, Decimal)> = account_ids
        .iter()
        .copied()
        .zip(amounts.iter().copied())
        .map(|(acct, atoms)| (acct, dec_from_atoms(atoms, scale_i)))
        .collect();

    // Simulate bank ending balances as ledger ending balances plus a deterministic adjustment
    // (e.g., bank fees/interest/timing differences). Keep adjustment small but non-zero.
    let bank_adj_fp: Vec<(u32, FixedPoint)> = (0..account_count)
        .map(|acct| {
            let adj_atoms = ((acct as i64 * 17) % 7) - 3; // [-3..3] cents
            (acct, FixedPoint::new(adj_atoms, scale_i))
        })
        .collect();
    let bank_adj_f64: Vec<(u32, f64)> = (0..account_count)
        .map(|acct| {
            let adj_atoms = ((acct as i64 * 17) % 7) - 3;
            (acct, (adj_atoms as f64) / scale_f)
        })
        .collect();
    let bank_adj_dec: Vec<(u32, Decimal)> = (0..account_count)
        .map(|acct| {
            let adj_atoms = ((acct as i64 * 17) % 7) - 3;
            (acct, dec_from_atoms(adj_atoms, scale_i))
        })
        .collect();

    let mut group = c.benchmark_group("bank_reconcile");

    group.bench_function("fixed_point/hashmap", |b| {
        use std::collections::HashMap;

        b.iter(|| {
            let mut ledger: HashMap<u32, FixedPoint> =
                HashMap::with_capacity(account_count as usize);
            for &(acct, amt) in &tx_fp {
                let entry = ledger
                    .entry(acct)
                    .or_insert_with(|| FixedPoint::new(0, scale_i));
                *entry = black_box(*entry + amt);
            }

            // Build bank balances by applying deterministic adjustment.
            let mut mismatch_count = 0u32;
            let mut checksum = 0i64;
            for &(acct, adj) in &bank_adj_fp {
                let ledger_bal = *ledger.get(&acct).unwrap_or(&FixedPoint::new(0, scale_i));
                let bank_bal = black_box(ledger_bal + adj);
                let delta = black_box(bank_bal - ledger_bal);
                if delta.atoms() != 0 {
                    mismatch_count += 1;
                }
                checksum = checksum.wrapping_add(delta.atoms());
            }

            black_box((mismatch_count, checksum))
        })
    });

    group.bench_function("f64/hashmap", |b| {
        use std::collections::HashMap;

        b.iter(|| {
            let mut ledger: HashMap<u32, f64> = HashMap::with_capacity(account_count as usize);
            for &(acct, amt) in &tx_f64 {
                let entry = ledger.entry(acct).or_insert(0.0);
                *entry = black_box(*entry + amt);
            }

            let mut mismatch_count = 0u32;
            let mut checksum = 0.0_f64;
            for &(acct, adj) in &bank_adj_f64 {
                let ledger_bal = *ledger.get(&acct).unwrap_or(&0.0);
                let bank_bal = black_box(ledger_bal + adj);
                let delta = black_box(bank_bal - ledger_bal);

                // Quantize delta to cents for a fair comparison to FixedPoint.
                let delta_cents = round_f64(delta * scale_f, mode);
                if delta_cents != 0.0 {
                    mismatch_count += 1;
                }
                checksum = black_box(checksum + delta_cents);
            }

            black_box((mismatch_count, checksum))
        })
    });

    group.bench_function("decimal/hashmap", |b| {
        use std::collections::HashMap;

        b.iter(|| {
            let mut ledger: HashMap<u32, Decimal> = HashMap::with_capacity(account_count as usize);
            for &(acct, amt) in &tx_dec {
                let entry = ledger.entry(acct).or_insert(Decimal::ZERO);
                *entry = black_box(*entry + amt);
            }

            let mut mismatch_count = 0u32;
            let mut checksum = Decimal::ZERO;
            for &(acct, adj) in &bank_adj_dec {
                let ledger_bal = *ledger.get(&acct).unwrap_or(&Decimal::ZERO);
                let bank_bal = black_box(ledger_bal + adj);
                let delta = black_box(bank_bal - ledger_bal);

                let delta_cents = dec_quantize(delta, scale_i, mode);
                if !delta_cents.is_zero() {
                    mismatch_count += 1;
                }
                checksum = black_box(checksum + delta_cents);
            }

            black_box((mismatch_count, checksum.to_i64().unwrap_or(0)))
        })
    });

    group.finish();
}

fn bench_cross_scale_add(c: &mut Criterion) {
    let inputs = build_i64_inputs(10_000);

    let lhs: Vec<FixedPoint> = inputs.iter().map(|&a| FixedPoint::new(a, 100)).collect();
    let rhs: Vec<FixedPoint> = inputs.iter().map(|&a| FixedPoint::new(a, 1_000)).collect();

    let mut group = c.benchmark_group("cross_scale_add");

    group.bench_function("fixed_point/100_plus_1000", |b| {
        b.iter(|| {
            let mut acc = FixedPoint::new(0, 1_000);
            for (&l, &r) in lhs.iter().zip(rhs.iter()) {
                acc = black_box(acc + black_box(l + r));
            }
            black_box(acc)
        })
    });

    group.finish();
}

fn benches(c: &mut Criterion) {
    bench_add(c);
    bench_quantize(c);
    bench_div_round(c);
    bench_regulatory_round_reembed(c);
    bench_accounting_tax_pipeline(c);
    bench_bank_reconcile(c);
    bench_cross_scale_add(c);
}

criterion_group!(fixed_point_benches, benches);
criterion_main!(fixed_point_benches);
