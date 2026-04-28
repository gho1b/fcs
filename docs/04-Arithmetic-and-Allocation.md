# Arithmetic and Allocation

## Objective

Dokumen ini mendefinisikan semantic operasi inti untuk monetary computation yang generik dan deterministik.

## Normative Scope

Dokumen ini adalah owner untuk:

- penjumlahan dan pengurangan monetary amount
- apply-rate dan multiply semantics
- division semantics
- allocation dan apportionment
- residual handling
- valid vs invalid operation boundary

## Operation Preconditions

Operasi `add`, `subtract`, `compare`, dan `allocate` hanya valid bila:

- `currency` identik
- `scale` identik
- policy tambahan yang diperlukan tersedia

Jika precondition gagal, implementasi harus:

- menolak operasi, atau
- melakukan normalisasi eksplisit sesuai policy yang terdokumentasi

## Addition and Subtraction

Aturan dasar:

```text
result.amount = left.amount + right.amount
```

dan

```text
result.amount = left.amount - right.amount
```

Invariants:

- `currency` hasil harus sama dengan operand
- `scale` hasil harus sama dengan operand
- overflow harus diperlakukan sebagai error, bukan wraparound diam-diam

## Multiply by Rate

Perkalian dengan rate digunakan untuk:

- derived price amount
- tax
- fee
- discount
- proportional allocation
- field lain yang membutuhkan perhitungan berbasis rate

Standar ini merekomendasikan rate direpresentasikan sebagai rasio integer atau fixed-point policy-defined, bukan floating point.

Reference model:

```text
raw_amount = amount * numerator / denominator
```

Keluaran dari operasi ini belum final sampai aturan rounding stage diterapkan.

## Division Semantics

Division finansial hampir selalu menghasilkan sisa. Karena itu:

- hasil quotient harus deterministik
- residual harus punya aturan distribusi
- division tidak boleh menyembunyikan loss secara implicit

Reference model:

```text
quotient = dividend / divisor
remainder = dividend % divisor
```

`remainder` harus:

- ditolak bila context tidak mengizinkan residual
- atau didistribusikan secara eksplisit oleh allocation policy

## Allocation

Allocation adalah pembagian satu amount ke banyak target dengan total konservatif.

Invariant allocation:

- jumlah seluruh hasil allocation harus sama dengan original amount
- tidak boleh ada unit value hilang atau tercipta
- urutan distribusi residual harus deterministik

Reference algorithm family yang diizinkan:

- largest remainder
- stable index order
- weighted allocation berdasarkan integer weight

Project boleh memilih algoritme, tetapi harus:

- menamai policy-nya
- menjaga hasil replayable
- mendokumentasikan aturan tie-break residual

## Residual Handling

Residual handling minimum harus menyatakan:

- residual unit minimum dalam skala saat ini
- siapa yang menerima residual pertama
- urutan tie-break
- apakah residual boleh tetap tersisa sebagai carry

Contoh policy:

```text
allocation_policy = WEIGHTED_LARGEST_REMAINDER_V1
residual_tiebreak = INPUT_ORDER
```

## Invalid Operations

Operasi berikut tidak valid tanpa policy tambahan:

- add/subtract lintas currency
- add/subtract lintas scale
- divide tanpa definisi residual handling
- multiply by floating point binary number sebagai source of truth

## Compliance Outline

Implementasi yang patuh harus dapat membuktikan:

- addition/subtraction tidak menerima mismatch representation
- multiply/divide tidak kehilangan residual secara diam-diam
- allocation menjaga conservation of amount
- algoritme residual dapat direplay dengan input yang sama
