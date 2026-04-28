# Compliance Test

## Objective

Dokumen ini mendefinisikan uji kepatuhan minimum untuk implementasi yang mengklaim mengikuti financial computation
standard.

Dokumen ini adalah alat verifikasi operasional. Untuk governance normatif dan authority implementasi, lihat
[25-Implementation-Governance](25-Implementation-Governance.md).

## Money

- Sistem menyimpan monetary value dalam integer (bilangan bulat).
- Sistem selalu menyimpan `currency` dan `scale`.
- Sistem menolak operasi aritmetika lintas `currency` atau `scale` tanpa normalisasi eksplisit.
- Sistem tidak menggunakan floating point sebagai source of truth.
- Sistem mendefinisikan dengan jelas apakah contract tertentu mengizinkan `amount` negatif.

## Arithmetic and Allocation

- Sistem hanya mengizinkan `add/subtract/compare` pada representation yang kompatibel.
- Sistem mendefinisikan perilaku overflow sebagai error atau guardrail ekuivalen.
- Sistem menjalankan apply-rate tanpa menjadikan floating point sebagai authority.
- Sistem mendefinisikan residual handling untuk division atau allocation.
- Total hasil allocation selalu sama dengan source amount.

## Rounding

- Sistem membedakan `scale` dan `rounding_quantum`.
- Sistem dapat menjalankan `round_to_quantum(raw_amount, mode, quantum)` secara deterministik.
- Perubahan aturan rounding membutuhkan policy atau adapter version yang jelas.
- Jika multi-stage rounding dipakai, setiap stage terdokumentasi dan replayable.

## Policy and Versioning

- Setiap hasil final dapat ditelusuri ke `policy_version`.
- Breaking semantic change menghasilkan version policy baru.
- Historical result tidak direinterpretasi diam-diam dengan policy terbaru.

## Temporal Semantics

- Sistem mendefinisikan timestamp acuan untuk memilih policy.
- Sistem mendefinisikan timezone bila timezone memengaruhi hasil.
- Replay menggunakan temporal context yang sama dengan perhitungan asal.

## Tax

- Tax dihitung melalui tax policy yang eksplisit.
- Sistem mendukung `tax_rounding_mode` dan `tax_rounding_quantum`.
- Tax adapter dapat diganti tanpa mengubah kontrak inti `TaxResolverPort`.
- _Jurisdiction-specific_ rule **tidak** diperlakukan sebagai aturan universal.
- Jika tax basis dapat bersifat inclusive atau extracted, mode basis tersebut terdokumentasi secara eksplisit.
- Jika satu transaksi memiliki lebih dari satu komponen tax, result menyatakan komponen dan totalnya secara konsisten.
- Jika adapter mendukung compound tax atau tax-on-tax, urutan evaluasinya terdokumentasi dan replayable.
- Jika jurisdiction-based tax membutuhkan sourcing context, contract input menyatakan context tersebut secara eksplisit.
- Jika workflow refund atau credit note memakai negative tax basis, alurnya dijelaskan melalui tax contract atau correction flow.

## Correction

- Historical record tidak dimutasi diam-diam.
- Correction selalu tercatat sebagai fakta baru.
- Financial adjustment dapat ditelusuri ke referensi asal.
- _Operational loss_ terpisah dari _customer-facing correction_.

## Integration

- Input dengan `scale` yang tidak cocok ditolak atau dinormalisasi secara eksplisit sesuai policy.
- Contract serialization stabil di JSON/event/persistence.
- Presentation value tidak menggantikan stored fixed-point amount sebagai authority.

## Recommended Golden Vectors

Implementasi sebaiknya memiliki test vector minimum untuk:

- add/subtract mismatch rejection
- multiply then round
- divide with residual
- weighted allocation
- tax before dan sesudah regulatory cut-off
- reversal dan adjustment traceability

## Review Form

Gunakan form ini untuk governance review, architecture review, atau compliance sign-off.

Status yang direkomendasikan:

- `PASS`
- `PASS WITH DECLARED DEVIATION`
- `FAIL`
- `NOT APPLICABLE`

### Review Metadata

| Field | Value |
|---|---|
| Implementation Name | |
| Repository / Service | |
| Version / Commit | |
| Standard Version | |
| Policy Version(s) | |
| Reviewer | |
| Review Date | |
| Outcome | |

### A. Semantic Authority

| Check | Status | Evidence | Notes |
|---|---|---|---|
| Satu authority semantik untuk `Money` dan invariant inti telah ditetapkan | | | |
| Authority semantik mencakup representation, equality, normalization, rounding, dan determinism | | | |
| Bentuk authority semantik terdokumentasi | | | |
| Konflik interpretasi dapat dirujukkan ke authority yang jelas | | | |

### B. No Shadow Semantics

| Check | Status | Evidence | Notes |
|---|---|---|---|
| Derived system tidak mengubah makna `amount`, `currency`, atau `scale` | | | |
| Derived system tidak mengubah equality/comparison behavior inti | | | |
| Derived system tidak menambahkan normalization semantics lokal | | | |
| Derived system tidak memperlakukan zero sebagai pengecualian implisit yang tidak sah | | | |
| Derived system tidak melakukan reinterpretation diam-diam di parsing, transport, atau persistence | | | |

### C. Reuse or Proven Equivalence

| Check | Status | Evidence | Notes |
|---|---|---|---|
| Komponen inti resmi digunakan bila tersedia, atau ada alasan eksplisit mengapa tidak | | | |
| Implementasi non-core lolos compliance test yang sama | | | |
| Golden vectors yang dipakai identik dengan authority semantik | | | |
| Hasil implementasi non-core identik untuk input yang identik | | | |

### D. Money Representation

| Check | Status | Evidence | Notes |
|---|---|---|---|
| `Money.amount` diperlakukan sebagai integer semantics | | | |
| `currency` dan `scale` selalu tersedia pada `Money` valid | | | |
| Value yang belum memiliki `currency` atau `scale` final tidak diperlakukan sebagai `Money` valid | | | |
| Equality dan comparison mengikuti aturan normatif standard inti | | | |
| Header inheritance/override memiliki precedence rule yang terdokumentasi | | | |

### E. Arithmetic and Normalization

| Check | Status | Evidence | Notes |
|---|---|---|---|
| Arithmetic hanya dilakukan pada context yang kompatibel atau setelah normalisasi eksplisit | | | |
| Normalisasi tidak dilakukan secara ad hoc di logic turunan | | | |
| Transformasi lintas `scale` dapat dijelaskan dan direplay | | | |
| Overflow behavior terdokumentasi | | | |
| Allocation/residual handling mematuhi standard yang relevan | | | |

### F. Rounding

| Check | Status | Evidence | Notes |
|---|---|---|---|
| Rounding selalu eksplisit | | | |
| `rounding_mode`, `rounding_quantum`, dan stage terdokumentasi | | | |
| Multi-stage rounding replayable | | | |
| Derived system tidak menambahkan rounding lokal yang tidak sah | | | |

### G. Transport and Serialization

| Check | Status | Evidence | Notes |
|---|---|---|---|
| Semantic integer value tetap lossless setelah parsing/serialization | | | |
| Transport tidak menyebabkan truncation atau reinterpretation | | | |
| Alternate encoding lossless digunakan bila transport native tidak aman | | | |
| Batas compatibility transport terdokumentasi | | | |

### H. Policy and Temporal Compliance

| Check | Status | Evidence | Notes |
|---|---|---|---|
| Hasil final dapat ditelusuri ke `policy_version` | | | |
| Timestamp acuan untuk policy selection terdokumentasi | | | |
| Replay menggunakan policy dan temporal context yang benar | | | |
| Historical policy tidak diganti diam-diam saat recomputation | | | |

### I. Tax and Correction Compliance

| Check | Status | Evidence | Notes |
|---|---|---|---|
| Tax adapter tidak mengubah semantic inti `Money` atau arithmetic | | | |
| Tax result dapat ditelusuri ke policy dan computation scope yang benar | | | |
| Correction tetap append-only dan traceable | | | |
| Reversal/adjustment/conversion tidak disamarkan sebagai arithmetic biasa | | | |

### J. Invariant Protection

| Check | Status | Evidence | Notes |
|---|---|---|---|
| Strict equality dilindungi | | | |
| Currency isolation dilindungi | | | |
| Normalization requirement dilindungi | | | |
| Zero discipline dilindungi | | | |
| Deterministic replay dilindungi | | | |
| Semantic field inti tidak dimutasi diam-diam di tengah komputasi | | | |

### K. Compliance Evidence

| Check | Status | Evidence | Notes |
|---|---|---|---|
| Versi standard yang diikuti dapat ditunjukkan | | | |
| Versi policy atau adapter aktif dapat ditunjukkan | | | |
| Hasil compliance test tersedia | | | |
| Known deviations terdokumentasi | | | |
| Transport boundary dan compatibility assumptions terdokumentasi | | | |

## Minimal Reviewer Questions

1. Authority semantik implementasi ini apa?
2. Apakah implementasi ini reuse authority inti atau membuat implementasi sendiri?
3. Jika implementasi sendiri, evidence equivalence-nya apa?
4. Apakah ada semantic rule yang diubah di service ini?
5. Bagaimana integer lossless dijaga di transport yang dipakai?
6. Bagaimana normalization, rounding, dan replay dibuktikan konsisten?
7. Apakah ada deviation yang belum diungkap?

Untuk melihat bagaimana dokumen ini diposisikan terhadap dokumen lain, lihat [24-Document-Map](24-Document-Map.md).
