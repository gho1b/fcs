# Compliance Test

## Objective

Dokumen ini mendefinisikan uji kepatuhan minimum untuk implementasi yang mengklaim mengikuti financial computation
standard.

## Money

- Sistem menyimpan monetary value dalam integer (bilangan bulat).
- Sistem selalu menyimpan `currency` dan `scale`.
- Sistem menolak operasi aritmetika lintas `currency` atau `scale` tanpa normalisasi eksplisit.
- Sistem tidak menggunakan floating point sebagai source of truth.

## Rounding

- Sistem membedakan `scale` dan `rounding_quantum`.
- Sistem dapat menjalankan `round_to_quantum(raw_amount, mode, quantum)` secara deterministik.
- Perubahan aturan rounding membutuhkan policy atau adapter version yang jelas.

## Tax

- Tax dihitung melalui tax policy yang eksplisit.
- Sistem mendukung `tax_rounding_mode` dan `tax_rounding_quantum`.
- Tax adapter dapat diganti tanpa mengubah kontrak inti `TaxResolverPort`.
- _Jurisdiction-specific_ rule **tidak** diperlakukan sebagai aturan universal.

## Correction

- Historical record tidak dimutasi diam-diam.
- Correction selalu tercatat sebagai fakta baru.
- Financial adjustment dapat ditelusuri ke referensi asal.
- _Operational loss_ terpisah dari _customer-facing correction_.

## Integration

- Input dengan `scale` yang tidak cocok ditolak atau dinormalisasi secara eksplisit sesuai policy.
- Contract serialization stabil di JSON/event/persistence.
- Presentation value tidak menggantikan stored fixed-point amount sebagai authority.
