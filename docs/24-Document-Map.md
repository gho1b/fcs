# Document Map

## Objective

Dokumen ini memetakan ownership dan fungsi dokumen utama dalam Financial Computation Standard agar pembaca tahu di mana
sebuah keputusan normatif hidup dan di mana verifikasinya dilakukan.

## Core Semantic Documents

| Dokumen                                                         | Owner                                                                     |
|-----------------------------------------------------------------|---------------------------------------------------------------------------|
| [03-Money](03-Money.md)                                         | Primitive `Money`, representasi, equality, comparison, transport baseline |
| [04-Arithmetic-and-Allocation](04-Arithmetic-and-Allocation.md) | Arithmetic semantics, allocation, residual handling                       |
| [05-Rounding](05-Rounding.md)                                   | Rounding semantics dan stage discipline                                   |
| [06-Temporal-Semantics](06-Temporal-Semantics.md)               | Effective time, replay time, timezone dan cut-off                         |
| [07-Policy-and-Versioning](07-Policy-and-Versioning.md)         | Policy snapshot, versioning, compatibility rule                           |
| [08-Tax](08-Tax.md)                                             | Tax abstraction, tax result, sourcing dan tax policy boundary             |
| [09-Correction](09-Correction.md)                               | Append-only correction semantics                                          |

## Governance and Verification Documents

| Dokumen                                                         | Fungsi                                                                                  |
|-----------------------------------------------------------------|-----------------------------------------------------------------------------------------|
| [25-Implementation-Governance](25-Implementation-Governance.md) | Governance normatif untuk semantic authority, no shadow semantics, dan equivalence rule |
| [13-Compliance-Test](13-Compliance-Test.md)                     | Checklist kepatuhan minimum dan review form operasional                                 |
| [12-Reference-Contracts](12-Reference-Contracts.md)             | Shape kontrak referensi minimum                                                         |
| [21-Audit-Model](21-Audit-Model.md)                             | Model audit dan traceability                                                            |

## Reading Guidance

Urutan baca yang direkomendasikan:

1. `01-Overview`
2. `02-Principles`
3. `03-09` untuk semantic core
4. `25-Implementation-Governance` untuk authority implementasi
5. `13-Compliance-Test` untuk review dan sign-off
6. `12-Reference-Contracts` untuk contract shape

## Conflict Resolution Rule

Jika terdapat konflik pembacaan:

1. dokumen semantic core menjadi authority untuk semantic primitive dan invariant
2. [25-Implementation-Governance](25-Implementation-Governance.md) menjadi authority untuk governance implementasi
3. [13-Compliance-Test](13-Compliance-Test.md) menjadi alat verifikasi operasional, bukan sumber semantic baru
