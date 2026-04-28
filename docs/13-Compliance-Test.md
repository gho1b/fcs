# Compliance Test

## Objective

Dokumen ini mendefinisikan uji kepatuhan minimum untuk implementasi yang mengklaim mengikuti financial computation
standard.

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
