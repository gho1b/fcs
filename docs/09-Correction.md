# Correction

## Objective

Dokumen ini mendefinisikan strategi correction finansial yang append-only dan reusable lintas project.

Semantik correction ini berlaku untuk domain finansial manapun — termasuk namun tidak terbatas pada: payment,
subscription, service charge, fee, ataupun billing.

## Normative Scope

Dokumen ini adalah owner untuk:

- correction taxonomy
- traceability rule
- replacement/reversal/adjustment boundary
- separation antara customer-facing correction dan operational loss

## Primary Rule

```text
Historical event MUST NOT be mutated. Correction MUST be append-only.
```

## Correction Types

### Replacement

Digunakan bila struktur transaksi salah dan transaksi perlu diganti.

Pattern generik:

```text
OriginalRecordCancelled
ReplacementRecordCreated(new_id)
```

### Reversal

Digunakan bila efek finansial harus dibalik penuh atau sebagian melalui fakta penyeimbang.

Pattern generik:

```text
TransactionRecorded   +50000
TransactionReversed   -50000
```

> **Contoh ini bersifat ilustratif.** Nama event konkret ditentukan oleh domain project masing-masing
> (mis. `PaymentReceived` / `PaymentReversed`, atau `FeeCharged` / `FeeReversed`).

### Financial Adjustment

Digunakan bila transaksi tetap valid tetapi nilai akhirnya perlu dikoreksi.

Pattern generik:

```text
AdjustmentIssued            -10000
PartialCorrectionIssued     -5000
FinancialAdjustmentRecorded -5000
```

> **Contoh ini bersifat ilustratif.** Nama event konkret (mis. `CompensationIssued`, `PartialRefundIssued`) ditentukan
> oleh domain project masing-masing.

## Decision Rule

```text
Jika struktur salah                             -> Replacement
Jika efek finansial harus dibalik               -> Reversal
Jika transaksi tetap valid tapi nilai berubah   -> Financial Adjustment
```

## Relationship to Money and Policy

- Sign dari adjustment atau reversal harus konsisten dengan contract yang memakainya.
- Correction event harus tetap membawa atau dapat merujuk ke representation context yang benar (`currency`, `scale`).
- Jika correction dipengaruhi policy baru, relasi antara policy lama dan baru harus dapat dijelaskan secara
  audit-friendly.

## Separation Rule

Customer-facing correction tidak boleh dicampur dengan operational cost.

```text
Customer-facing correction != Operational loss
```

## Guardrails

- Correction **harus** traceable ke fakta asal.
- Customer-facing correction **tidak boleh** direpresentasikan sebagai operational loss.
- Operational loss **tidak boleh** disamarkan sebagai customer-facing correction.
- Correction note atau business memo **tidak boleh** menjadi pengganti correction event internal.
- Historical replay **harus** dapat menunjukkan urutan original fact dan correction fact.
