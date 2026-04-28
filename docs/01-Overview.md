# Overview

## Tujuan

Standar ini memisahkan financial computation dari business domain tertentu.

Target utamanya adalah menyediakan fondasi generik untuk project yang membutuhkan:

- komputasi uang tanpa floating point
- tax policy yang bisa berbeda per negara atau jurisdiksi
- correction yang append-only
- kontrak data yang stabil untuk event, persistence, dan integrasi

## Positioning

Financial computation standard adalah kernel generik yang berada:

- **di bawah domain bisnis** — domain bisnis yang menggunakannya
- **di atas storage / infra** — tidak terikat pada database atau framework tertentu
- **netral terhadap framework** — tidak memaksa pola arsitektur apapun
- **netral terhadap organisasi data internal project**

Kernel ini dapat digunakan oleh berbagai jenis domain di atasnya. Contoh penggunaan
(bukan limitasi atau prescription):

- pricing retail
- billing kontrak
- fee calculation
- refund dan correction
- tax reporting preparation

Domain yang menggunakannya bertanggung jawab mendefinisikan workflow dan business rule
di atas kernel ini.

## Document Layers

Repositori ini dibaca dalam tiga lapisan:

1. **Foundation**
   - `01-Overview`
   - `02-Principles`
   - `03-Money`
2. **Computation Semantics**
   - `04-Arithmetic-and-Allocation`
   - `05-Rounding`
   - `06-Policy-and-Versioning`
   - `07-Temporal-Semantics`
   - `08-Tax`
   - `09-Correction`
3. **Integration and Verification**
   - `10-Reference-Contracts`
   - `11-Compliance-Test`
   - `12-FFI-Plan`

## Boundary

Kernel ini hanya mendefinisikan:

- money representation
- computation invariants
- arithmetic semantics
- allocation semantics
- rounding semantics
- policy snapshot semantics
- temporal semantics
- tax abstraction
- correction semantics

Kernel ini tidak mendefinisikan:

- produk atau katalog
- strategi pricing domain
- agreement validation spesifik UI/backend
- aturan fulfillment domain
- lifecycle order, invoice, atau entitas bisnis apapun

## Relationship to Project-Specific Docs

Project-specific docs boleh:

- memilih `policy_version`
- memilih atau mengimplementasikan `tax_adapter`
- menetapkan `tax_rounding_quantum`
- menambahkan domain event dan workflow spesifik

Namun project-specific docs tidak boleh melanggar invariant inti dari standar ini.

## Reading Order

Urutan baca yang direkomendasikan:

1. `01-Overview` untuk boundary
2. `02-Principles` untuk invariant global
3. `03-07` untuk semantic core
4. `08-09` untuk regulatory and correction modules
5. `10-12` untuk contract, compliance, dan integration plan
