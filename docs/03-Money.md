# Money

## Objective

Dokumen ini mendefinisikan cara merepresentasikan nilai uang secara stabil di JSON, database, event, dan reporting.

## Canonical Representation

Semua monetary value direpresentasikan sebagai fixed-point integer:

```text
decimal_value = amount / scale
```

Contoh:

```text
amount = 150000000
currency = IDR
scale = 1000
decimal_value = 150000.000
```

## Required Header

Setiap monetary record minimum harus membawa:

```json
{
  "amount": 150000000,
  "currency": "IDR",
  "scale": 1000
}
```

Project-specific policy boleh menambahkan:

```json
{
  "policy_version": "pricing/v1"
}
```

## Invariants

- `amount` adalah integer.
- `scale` adalah integer positif.
- Operasi aritmetika hanya valid bila `currency` dan `scale` sama.
- Stored fixed-point amount adalah source of truth.
- Normalized presentation value tidak boleh disimpan sebagai authority pengganti.

## Reporting Rule

- Reporting tidak boleh mengagregasi raw amount lintas transaksi kecuali `currency` dan `scale` identik.
- Jika `scale` berbeda, reporting harus menormalisasi setiap transaksi sebelum agregasi.

## Consequences

Positif:

- deterministik
- tidak ada floating point drift
- aman untuk audit

Negatif:

- consumer harus memahami `scale`
- perubahan `scale` memerlukan versioning atau _**migration strategy**_
