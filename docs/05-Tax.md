# Tax

## Objective

Dokumen ini mendefinisikan abstraction dan policy tax yang reusable lintas negara dan lintas project.

## Tax as Regulatory Policy

Tax bukan business rule umum. Tax adalah regulatory policy yang:

- dapat berbeda per jurisdiksi
- dapat berubah karena regulasi
- perlu versioning yang jelas

## Core Policy Fields

Minimal tax policy yang direkomendasikan:

```text
tax_adapter
tax_scope
tax_rounding_mode
tax_rounding_quantum
tax_presentation
tax_rate
```

Nilai field dapat:

- statically configured
- diturunkan dari adapter aktif
- ditentukan oleh policy version

## Tax Computation Model

Formula generic:

```text
tax_raw_amount = compute_tax_raw(taxable_base, tax_rate)
tax_amount = round_to_quantum(tax_raw_amount, tax_rounding_mode, tax_rounding_quantum)
```

## Tax Port

Port tax harus cukup generik untuk menangani beberapa model regulasi:

```rust
pub trait TaxResolverPort {
    fn calculate_tax(
        &self,
        ctx: &TaxContext,
        lines: &[TaxableLine],
    ) -> Result<TaxResult, TaxError>;
}
```

## Required Capability

Port tax sebaiknya mampu mengakomodasi:

- `TRANSACTION_TOTAL`
- `LINE_ITEM`
- `JURISDICTION_BASED`
- `CATEGORY_BASED`

## Taxable Line

Reference model:

```rust
pub struct TaxableLine {
    pub line_id: Uuid,
    pub item_ref_id: Uuid,
    pub base_amount: i64,
    pub discount_amount: i64,
    pub taxable_amount: i64,
}
```

Invariant:

```text
taxable_amount = base_amount - discount_amount
taxable_amount >= 0
```

## Tax Result

Reference model:

```rust
pub struct TaxResult {
    pub total_tax_amount: i64,
    pub line_taxes: Vec<LineTax>,
    pub presentation_mode: TaxPresentationMode,
}
```

## Example: Indonesia Adapter

Contoh adapter konkret:

```text
tax_adapter = IndonesiaTaxResolverV1
tax_rounding_mode = HALF_UP
tax_rounding_quantum = 1000
```

Contoh hasil:

```text
12_500_320 -> 12_500_000
12_500_500 -> 12_501_000
```

Contoh ini menunjukkan bahwa `tax_rounding_quantum` adalah detail policy adapter, bukan aturan universal untuk semua
negara.
