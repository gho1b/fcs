# Temporal Semantics

## Objective

Dokumen ini mendefinisikan semantic waktu yang diperlukan agar financial computation tetap benar saat policy, tax, dan event berubah sepanjang waktu.

## Normative Scope

Dokumen ini adalah owner untuk:

- `effective_at`
- `computed_at`
- replay time semantics
- timezone dan date cut-off yang memengaruhi hasil

## Core Distinctions

`effective_at`:

- waktu saat suatu fact dianggap berlaku secara bisnis atau regulasi

`computed_at`:

- waktu saat sistem menjalankan perhitungan

`recorded_at`:

- waktu saat sistem menyimpan fact atau result

Ketiganya boleh sama, tetapi tidak boleh diasumsikan sama secara default.

## Temporal Rules

- Policy selection harus menyatakan timestamp acuan yang dipakai.
- Jika jurisdiksi tax memakai cut-off tanggal lokal, timezone tersebut harus eksplisit.
- Replay harus dapat dilakukan dengan timestamp acuan yang sama seperti saat hasil awal dihitung.

## Timezone Rule

Jika timezone memengaruhi hasil, implementasi harus menentukan source of truth:

- timezone jurisdiksi
- timezone kontrak
- timezone sistem yang dibakukan

Mengandalkan timezone host runtime tanpa contract eksplisit tidak diperbolehkan.

## Regulatory Cut-off

Perubahan regulasi sering berlaku mulai tanggal tertentu. Karena itu sistem harus dapat:

- menentukan policy efektif per tanggal
- membedakan transaksi sebelum dan sesudah cut-off
- mereplay hasil historis tanpa ambigu

## Compliance Outline

Implementasi yang patuh harus dapat membuktikan:

- timestamp acuan untuk policy selection terdokumentasi
- timezone yang memengaruhi tax/rounding tidak implicit
- replay terhadap input historis menggunakan temporal context yang sama
