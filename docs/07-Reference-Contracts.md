# Reference Contracts

## Objective

Dokumen ini menyediakan kontrak referensi minimum untuk implementasi lintas project.

## Monetary Amount

```json
{
  "amount": 150000000,
  "currency": "IDR",
  "scale": 1000
}
```

## Monetary Context

```json
{
  "currency": "IDR",
  "scale": 1000,
  "policyVersion": "pricing/v1"
}
```

## Tax Policy Snapshot

```json
{
  "taxAdapter": "IndonesiaTaxResolverV1",
  "taxScope": "TRANSACTION_TOTAL",
  "taxRoundingMode": "HALF_UP",
  "taxRoundingQuantum": 1000,
  "taxPresentation": "BULK_TRANSACTION_LEVEL"
}
```

## Taxable Line

```json
{
  "lineId": "uuidv7",
  "itemRefId": "uuidv7",
  "baseAmount": 50000000,
  "discountAmount": 5000000,
  "taxableAmount": 45000000
}
```

## Tax Result

```json
{
  "totalTaxAmount": 4950000,
  "lineTaxes": [],
  "presentationMode": "TransactionBulk"
}
```

## Financial Adjustment Record

```json
{
  "adjustmentId": "uuidv7",
  "referenceId": "uuidv7",
  "adjustmentType": "PARTIAL_REFUND",
  "amount": -5000,
  "currency": "IDR",
  "scale": 1000,
  "reason": "SERVICE_COMPENSATION"
}
```

## Notes

- Field naming boleh diadaptasi per project.
- Invariant representation dan policy meaning tidak boleh diubah diam-diam.
