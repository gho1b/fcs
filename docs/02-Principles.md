# Principles

## Prinsip Inti

1. Monetary source of truth harus berupa fixed-point integer.
2. Financial computation harus deterministik dan replayable.
3. Regulatory policy harus dipisahkan dari business policy.
4. Correction terhadap fakta finansial harus append-only.
5. Semua policy penting harus versioned.
6. Semua operasi yang dapat mengubah hasil angka harus punya semantic owner yang eksplisit.
7. Effective time untuk policy dan computation harus dapat ditelusuri.

## Representation Principle

- Tidak boleh menggunakan floating point sebagai source of truth.
- Semua amount harus diinterpretasikan menggunakan `currency + scale`.
- Presentation value adalah turunan, bukan authority.

## Computation Principle

- Komputasi harus menggunakan integer arithmetic.
- Rounding harus eksplisit, tidak boleh implicit.
- `scale` dan `rounding quantum` adalah konsep berbeda.
- Allocation dan residual handling tidak boleh dibiarkan implicit.
- Operasi divide atau apply-rate harus menyatakan bagaimana sisa dibagikan.

## Policy Principle

- `rounding_mode` customer-facing dapat berbeda dari tax rounding.
- Tax policy dapat berbeda antar jurisdiksi.
- Perubahan regulatory rule harus dilakukan melalui policy atau adapter version baru.
- Policy yang memengaruhi hasil final harus dapat disnapshot pada saat komputasi dilakukan.
- Historical replay harus bisa menentukan policy versi mana yang berlaku pada waktu tertentu.

## Temporal Principle

- Financial fact harus dapat dibedakan dari waktu komputasinya.
- Effective date dan computed date tidak boleh dianggap identik secara default.
- Jika timezone memengaruhi hasil regulasi, timezone tersebut harus menjadi bagian dari contract policy atau context.

## Correction Principle

- Historical fact tidak boleh diubah diam-diam.
- Correction harus traceable.
- Operational loss harus dipisahkan dari customer-facing correction.

## Integration Principle

- Input dengan representasi yang tidak cocok harus ditolak atau dinormalisasi secara eksplisit oleh policy.
- Gateway atau external system bukan source of truth untuk hasil komputasi internal, kecuali memang itu yang
  didefinisikan oleh contract project-specific.
- DTO, event, dan persistence contract harus stabil untuk field inti yang bersifat normatif.
