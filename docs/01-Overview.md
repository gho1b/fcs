# Overview

## Tujuan

Standar ini memisahkan financial computation dari business domain tertentu.

Target utamanya adalah menyediakan fondasi generik untuk project yang membutuhkan:

- komputasi uang tanpa floating point
- tax policy yang bisa berbeda per negara
- correction yang append-only
- kontrak data yang stabil untuk event, persistence, dan integrasi

## Positioning

Financial computation standard adalah kernel generik.

Business domain di atasnya dapat menggunakannya untuk:

- pricing retail
- billing kontrak
- fee calculation
- refund dan correction
- tax reporting preparation

## Boundary

Kernel ini hanya mendefinisikan:

- money representation
- computation invariants
- rounding semantics
- tax abstraction
- correction semantics

Kernel ini tidak mendefinisikan:

- produk atau katalog
- strategi pricing domain
- agreement validation spesifik UI/backend
- aturan fulfillment domain

## Relationship to Project-Specific Docs

Project-specific docs boleh:

- memilih `policy_version`
- memilih `tax_adapter`
- menetapkan `tax_rounding_quantum`
- menambahkan domain event dan workflow spesifik

Namun project-specific docs tidak boleh melanggar invariant inti dari standar ini.
