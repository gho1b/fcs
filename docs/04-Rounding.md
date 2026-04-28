# Rounding

## Objective

Dokumen ini mendefinisikan konsep rounding generik untuk financial computation.

## Core Concepts

- `rounding_mode` menentukan cara pembulatan
- `rounding_quantum` menentukan kelipatan hasil akhir
- `scale` menentukan representasi penyimpanan amount

Ketiganya tidak boleh dicampur.

## General Rule

Rounding harus selalu eksplisit pada titik komputasi yang jelas.

Contoh generic:

```text
rounded_amount = round_to_quantum(raw_amount, rounding_mode, rounding_quantum)
```

## Distinction

`scale`:

- menjelaskan cara membaca dan menyimpan amount

`rounding_quantum`:

- menjelaskan kelipatan pembulatan hasil

Contoh:

```text
scale = 1000
rounding_quantum = 1000
```

Berarti:

- amount disimpan dalam integer dengan 3 digit precision
- hasil tertentu dibulatkan ke kelipatan 1000 pada integer tersebut

## Policy Guidance

- _Customer-facing rounding_ harus mengikuti business expectation.
- Tax rounding harus mengikuti regulatory policy aktif.
- Jika rounding rule berubah, maka policy version baru harus dibuat.

## Example

```text
raw_amount = 12500500
rounding_mode = HALF_UP
rounding_quantum = 1000
result = 12501000
```

## Guardrails

- `scale` adalah skala yang dipakai saat data disimpan.
- `rounding_quantum` adalah skala yang dipakai untuk pembulatan nilai.
- Jangan melakukan implicit rounding di banyak tahap tanpa policy yang jelas.
- Jika multi-stage rounding dibutuhkan, setiap tahap harus dinyatakan eksplisit oleh policy.
