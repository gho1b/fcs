# Principles

## Prinsip Inti

1. Monetary source of truth harus berupa fixed-point integer.
2. Financial computation harus deterministik dan replayable.
3. Regulatory policy harus dipisahkan dari business policy.
4. Correction terhadap fakta finansial harus append-only.
5. Semua policy penting harus versioned.

## Representation Principle

- Tidak boleh menggunakan floating point sebagai source of truth.
- Semua amount harus diinterpretasikan menggunakan `currency + scale`.
- Presentation value adalah turunan, bukan authority.

## Computation Principle

- Komputasi harus menggunakan integer arithmetic.
- Rounding harus eksplisit, tidak boleh implicit.
- `scale` dan `rounding quantum` adalah konsep berbeda.

## Policy Principle

- `rounding_mode` customer-facing dapat berbeda dari tax rounding.
- Tax policy dapat berbeda antar jurisdiksi.
- Perubahan regulatory rule harus dilakukan melalui policy atau adapter version baru.

## Correction Principle

- Historical fact tidak boleh diubah diam-diam.
- Correction harus traceable.
- Operational loss harus dipisahkan dari customer-facing correction.

## Integration Principle

- Input dengan representasi yang tidak cocok harus ditolak atau dinormalisasi secara eksplisit oleh policy.
- Gateway atau external system bukan source of truth untuk hasil komputasi internal, kecuali memang itu yang
  didefinisikan oleh contract project-specific.
