# Tax

## Objective

Dokumen ini mendefinisikan abstraksi tax dan boundary policy tax yang reusable lintas negara dan lintas project.

## Normative Scope

Dokumen ini adalah owner untuk:

- tax abstraction
- tax adapter boundary
- tax policy field minimum
- taxable line semantics
- tax result semantics

## Tax as Regulatory Policy

Tax bukan business rule umum. Tax adalah regulatory policy yang:

- dapat berbeda per jurisdiksi
- dapat berubah karena regulasi
- perlu versioning yang jelas

Standar ini mendefinisikan **abstraksi dan port** tax, bukan implementasi atau aturan jurisdiksi spesifik.

Abstraksi tax di dokumen ini bergantung pada:

- arithmetic semantics dari `04-Arithmetic-and-Allocation`
- rounding semantics dari `05-Rounding`
- policy/versioning semantics dari `06-Policy-and-Versioning`
- temporal semantics dari `07-Temporal-Semantics`

## Core Policy Fields

Minimal tax policy yang direkomendasikan:

```text
tax_adapter
tax_scope
tax_basis_mode
tax_rounding_mode
tax_rounding_quantum
tax_presentation
tax_rate
tax_computation_scope
```

Keterangan minimum:

- `tax_adapter`: identifier adapter atau resolver yang aktif
- `tax_scope`: scope regulasi atau business exposure dari tax
- `tax_basis_mode`: cara basis tax diinterpretasikan terhadap amount sumber
- `tax_rounding_mode`: mode rounding untuk hasil tax
- `tax_rounding_quantum`: quantum rounding untuk hasil tax
- `tax_presentation`: cara tax diekspos ke result atau contract luar
- `tax_rate`: rate atau parameter rate yang dipakai adapter
- `tax_computation_scope`: level agregasi internal saat adapter menghitung tax

Nilai field dapat:

- statically configured
- diturunkan dari adapter aktif
- ditentukan oleh policy version

## Tax Computation Model

Formula generic untuk ad valorem tax:

```text
tax_raw_amount = compute_tax_raw(taxable_base, tax_rate)
tax_amount = round_to_quantum(tax_raw_amount, tax_rounding_mode, tax_rounding_quantum)
```

Pemilihan `tax_rate`, `tax_scope`, dan adapter aktif harus tunduk pada policy serta temporal context yang berlaku.

Formula di atas bukan satu-satunya model tax yang valid. Adapter generik juga dapat menangani:

- fixed-amount tax
- threshold-based tax
- tiered atau bracket tax
- minimum atau floor tax
- maximum atau cap tax

Dalam kasus tersebut, `tax_rate` harus dipahami sebagai parameter policy tax secara umum, bukan selalu satu angka persentase tunggal.

## Tax Basis Mode

`tax_basis_mode` menjelaskan hubungan antara amount sumber dan basis tax.

Mode minimum yang direkomendasikan:

```text
EXCLUSIVE
INCLUSIVE
EXTRACTED
```

- `EXCLUSIVE`: amount sumber belum mengandung tax
- `INCLUSIVE`: amount sumber sudah mengandung tax sebagai bagian dari gross amount
- `EXTRACTED`: tax harus diekstrak dari gross amount menurut rule adapter

Dokumen ini tidak menetapkan satu mode universal. Project atau adapter harus memilih mode yang relevan untuk policy aktif.

## Tax Computation Scope

`tax_scope` dan `tax_computation_scope` tidak identik.

- `tax_scope` menjelaskan cakupan tax dari sudut contract atau policy
- `tax_computation_scope` menjelaskan di level mana adapter menghitung tax secara internal

Contoh:

- satu jurisdiksi dapat mewajibkan tax ditampilkan per line item
- tetapi implementasi adapter dapat menghitung dulu di `TransactionTotal`
- atau sebaliknya, regulasi mengharuskan line-level calculation lalu dijumlahkan

`TaxComputationScope` adalah flag untuk menyatakan bagaimana tax dihitung di dalam implementasi adapter.

```rust
pub enum TaxComputationScope {
    LineLevel,
    TransactionTotal,
    TaxGroupLevel,
}
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

Implementasi konkret dari `TaxResolverPort` adalah tanggung jawab project atau adapter per-jurisdiksi — **bukan** bagian
dari standar ini.

## Required Capability

Port tax sebaiknya mampu mengakomodasi:

- `TRANSACTION_TOTAL`
- `LINE_ITEM`
- `JURISDICTION_BASED`
- `CATEGORY_BASED`
- `MULTI_COMPONENT`

Kemampuan ini tidak berarti semua mode harus aktif sekaligus pada satu adapter. Yang normatif adalah adapter
dapat mendeklarasikan mode mana yang didukung dan contract tidak mengasumsikan hanya ada satu model universal.

## Sourcing Context

Untuk `JURISDICTION_BASED` tax, adapter sering membutuhkan context tambahan yang tidak dapat disimpulkan hanya dari amount.

Context minimum yang layak dipertimbangkan oleh project:

- seller jurisdiction
- buyer jurisdiction
- place of supply
- customer tax status
- item tax category

Standar ini tidak mewajibkan shape tunggal untuk sourcing context, tetapi contract tax tidak boleh mengasumsikan semua jurisdiksi dapat diselesaikan tanpa input tersebut.

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
```

Jika jurisdiksi tertentu membutuhkan basis tambahan seperti exemption, surcharge, atau tax category, field tersebut
boleh ditambahkan oleh project atau adapter contract tanpa mengubah semantic minimum `TaxableLine`.

Catatan penting:

- Banyak workflow tax operasional hanya menerima `taxable_amount >= 0` pada fase komputasi normal.
- Namun standard ini tidak melarang kebutuhan historis seperti refund, credit note, atau reversal tax.
- Jika negative basis dibutuhkan, project harus menyatakannya secara eksplisit melalui contract tax atau melalui alur `09-Correction`.

Field tambahan yang umum dibutuhkan:

- `tax_category`
- `exemption_reason`
- `jurisdiction_ref`
- `tax_group_ref`

## Tax Result

Reference model:

```rust
pub struct TaxResult {
    pub total_tax_amount: i64,
    pub tax_computation_scope: TaxComputationScope,
    pub line_taxes: Vec<LineTax>,
    pub presentation_mode: TaxPresentationMode,
    pub components: Vec<TaxComponent>,
    pub provenance: TaxProvenance,
}
```

Invariant minimum:

- `total_tax_amount` harus konsisten dengan `line_taxes` sesuai `presentation_mode`
- result harus dapat ditelusuri ke policy tax yang aktif
- result tidak boleh ambigu mengenai level perhitungan yang dipakai adapter

`components` dibutuhkan bila satu transaction atau line dapat memiliki lebih dari satu tax, surcharge, levy, atau komponen tax lain.

`provenance` sebaiknya cukup untuk menunjukkan:

- rule atau adapter yang dipakai
- jurisdiction yang dipilih
- rate atau parameter policy yang diterapkan
- alasan exemption atau override bila ada

## Multi-Component Tax

Beberapa jurisdiksi membutuhkan lebih dari satu komponen tax pada basis yang sama atau berbeda.

Karena itu, spec ini menganggap:

- `total_tax_amount` adalah agregat final
- `components` adalah rincian normatif bila ada lebih dari satu komponen tax
- urutan evaluasi antar komponen harus eksplisit bila hasil satu komponen memengaruhi basis komponen lain

Tax-on-tax atau compound tax tidak wajib didukung oleh semua adapter, tetapi bila didukung, adapter harus mendokumentasikan evaluation order-nya.

## Non-Goals

Dokumen ini tidak menetapkan:

- daftar tax rate universal
- satu model rounding universal
- satu timezone universal
- adapter default untuk semua jurisdiksi

## Contoh Adapter (Ilustratif)

> **Bagian ini bersifat ilustratif.** Setiap jurisdiksi mengimplementasikan adapter sendiri dengan parameter yang sesuai
> regulasi setempat. Contoh berikut bukan bagian dari spec dan tidak boleh diperlakukan sebagai konfigurasi default.

Contoh adapter untuk jurisdiksi tertentu:

```text
tax_adapter         = ExampleJurisdictionTaxResolverV1
tax_rounding_mode   = HALF_UP
tax_rounding_quantum = 1000
```

Contoh hasil rounding dengan quantum 1000:

```text
12_500_320 -> 12_500_000
12_500_500 -> 12_501_000
```

Contoh ini menunjukkan bahwa `tax_rounding_quantum` adalah detail policy adapter, bukan aturan
universal untuk semua jurisdiksi. Adapter untuk jurisdiksi lain dapat menggunakan quantum, mode,
dan presentasi yang berbeda.
