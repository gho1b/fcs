# Reference Contracts

## Objective

Dokumen ini menyediakan **kontrak referensi minimum** untuk implementasi lintas project.

> **Penting:** Semua nilai pada contoh JSON berikut bersifat **ilustratif** — termasuk pilihan currency, scale, policy
> version, dan nama adapter. Nilai konkret ditentukan oleh policy dan konteks project masing-masing. Yang bersifat
> **normatif** adalah **shape (struktur field) dan invariant semantic-nya**, bukan nilai spesifiknya.

---

## Monetary Amount

Shape minimum:

```json
{
  "amount": <integer>,
  "currency": "<ISO-4217 atau kode project>",
  "scale": <integer positif>
}
```

> Shape ini berlaku bila satu record memang membawa nilai uang lengkap di tempat yang sama.
> Standar ini tidak mewajibkan semua envelope metadata mengandung `amount`.

Contoh ilustratif (IDR, 3 digit precision):

```json
{
  "amount": 150000000,
  "currency": "IDR",
  "scale": 1000
}
```

Contoh ilustratif (USD, 2 digit precision):

```json
{
  "amount": 10099,
  "currency": "USD",
  "scale": 100
}
```

---

## Monetary Context

Shape minimum:

```json
{
  "currency": "<currency-code>",
  "scale": <integer positif>,
  "policyVersion": "<policy-name>/<version>"
}
```

> `policyVersion` adalah identifier yang ditentukan oleh project. Prefix dan format bebas, asalkan konsisten dan
> versioned. Contoh: `"fiscal/v1"`, `"billing/v2"`, `"payment/v1"`.

Jika project memisahkan context dari value, maka `currency` dan `scale` dapat ditempatkan di context/header, sedangkan
`amount` berada di detail record yang merujuk ke context tersebut.

---

## Computation Context

Shape minimum:

```json
{
  "policyVersion": "<policy-name>/<version>",
  "effectiveAt": "<RFC-3339 timestamp or local-date by contract>",
  "computedAt": "<RFC-3339 timestamp>",
  "timezone": "<IANA timezone or project-defined stable identifier>"
}
```

> Bentuk tepat `effectiveAt` dapat berupa tanggal lokal jika policy memang bekerja pada granularity tanggal.
> Yang normatif adalah semantic-nya, bukan format literal tunggal.

---

## Rounding Policy Snapshot

Shape minimum:

```json
{
  "roundingMode": "<mode>",
  "roundingQuantum": <integer>,
  "roundingStage": "<stage-name>"
}
```

---

## Tax Policy Snapshot

Shape minimum:

```json
{
  "taxAdapter": "<adapter-identifier>",
  "taxScope": "<scope>",
  "taxBasisMode": "<EXCLUSIVE|INCLUSIVE|EXTRACTED>",
  "taxRoundingMode": "<mode>",
  "taxRoundingQuantum": <integer>,
  "taxPresentation": "<presentation-mode>",
  "taxComputationScope": "<LineLevel|TransactionTotal|TaxGroupLevel>"
}
```

> `taxAdapter` adalah identifier adapter per-jurisdiksi yang dipilih oleh project. Standar ini tidak menetapkan adapter
> default. Implementasi adapter adalah tanggung jawab project atau library jurisdiksi yang bersangkutan.

Contoh ilustratif (satu jurisdiksi tertentu):

```json
{
  "taxAdapter": "ExampleJurisdictionTaxResolverV1",
  "taxScope": "TRANSACTION_TOTAL",
  "taxRoundingMode": "HALF_UP",
  "taxRoundingQuantum": 1000,
  "taxPresentation": "BULK_TRANSACTION_LEVEL"
}
```

---

## Taxable Line

```json
{
  "lineId": "uuidv7",
  "itemRefId": "uuidv7",
  "baseAmount": <integer>,
  "discountAmount": <integer>,
  "taxableAmount": <integer>,
  "taxCategory": "<category-code, optional>",
  "exemptionReason": "<reason-code, optional>",
  "jurisdictionRef": "<jurisdiction-ref, optional>",
  "taxGroupRef": "<group-ref, optional>"
}
```

Invariant minimum: `taxableAmount = baseAmount - discountAmount`.

> Jika project hanya mengizinkan komputasi tax normal pada positive basis, aturan `taxableAmount >= 0`
> boleh ditegakkan di contract project. Negative basis untuk refund atau correction harus dijelaskan secara eksplisit.

---

## Tax Result

```json
{
  "totalTaxAmount": <integer>,
  "taxComputationScope": "<LineLevel|TransactionTotal|TaxGroupLevel>",
  "lineTaxes": [],
  "presentationMode": "<presentation-mode>",
  "components": [],
  "provenance": {
    "adapterId": "<adapter-identifier>",
    "jurisdictionRef": "<jurisdiction-ref, optional>",
    "ruleRef": "<rule-ref, optional>",
    "rateRef": "<rate-ref, optional>",
    "exemptionReason": "<reason-code, optional>"
  }
}
```

---

## Financial Adjustment Record

Shape minimum:

```json
{
  "adjustmentId": "uuidv7",
  "referenceId": "uuidv7",
  "adjustmentType": "<type>",
  "amount": <integer, negatif untuk pengurangan>,
  "currency": "<currency-code>",
  "scale": <integer positif>,
  "reason": "<reason-code>"
}
```

> `adjustmentType` dan `reason` adalah kode yang didefinisikan oleh project (mis.
> `"PARTIAL_REFUND"`, `"SERVICE_COMPENSATION"`, `"FEE_REVERSAL"`). Standar ini tidak
> menetapkan daftar nilai yang valid — hanya mewajibkan field ini ada dan bermakna bagi
> audit trail.

---

## Allocation Result

Shape minimum:

```json
{
  "sourceAmount": <integer>,
  "allocatedAmounts": [
    <integer>
  ],
  "residualHandling": "<policy-name>",
  "conservedTotal": <integer>
}
```

---

## Notes

- Field naming boleh diadaptasi per project (camelCase, snake_case, dsb).
- Invariant representation dan policy meaning tidak boleh diubah diam-diam.
- Penambahan field opsional oleh project diperbolehkan selama tidak mengubah semantic field inti.
- Dokumen ini hanya menetapkan shape minimum. Lifecycle dan workflow tetap dimiliki domain project.
