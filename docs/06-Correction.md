# Correction

## Objective

Dokumen ini mendefinisikan strategi correction finansial yang append-only dan reusable lintas project.

## Primary Rule

```text
Historical event MUST NOT be mutated. Correction MUST be append-only.
```

## Correction Types

### Replacement

Digunakan bila struktur transaksi salah dan transaksi perlu diganti.

Contoh:

```text
OriginalRecordCancelled
ReplacementRecordCreated(new_id)
```

### Reversal

Digunakan bila efek finansial harus dibalik penuh atau sebagian melalui fakta penyeimbang.

Contoh:

```text
PaymentReceived +50000
PaymentReversed -50000
```

### Financial Adjustment

Digunakan bila transaksi tetap valid tetapi nilai akhirnya perlu dikoreksi.

Contoh:

```text
CompensationIssued -10000
PartialRefundIssued -5000
FinancialAdjustmentRecorded -5000
```

## Decision Rule

```text
Jika struktur salah -> Replacement
Jika efek finansial harus dibalik -> Reversal
Jika transaksi tetap valid tapi nilai berubah -> Financial Adjustment
```

## Separation Rule

Customer-facing correction tidak boleh dicampur dengan operational cost.

```text
Customer-facing correction != Operational loss
```

## Guardrails

- Correction harus traceable ke fakta asal.
- Refund tidak boleh direpresentasikan sebagai operational loss.
- Operational loss tidak boleh disamarkan sebagai refund customer.
- Correction note atau business memo tidak boleh menjadi pengganti correction event internal.
