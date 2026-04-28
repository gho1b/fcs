# Implementation Governance

## Objective

Dokumen ini mendefinisikan governance normatif untuk implementasi yang mengklaim mematuhi Financial Computation Standard.

Dokumen ini tidak mengubah semantic inti dari `Money`, arithmetic, rounding, policy, tax, atau correction. Dokumen ini menetapkan bagaimana semantic tersebut harus dijaga secara konsisten di seluruh implementasi.

## Normative Scope

Dokumen ini adalah owner untuk:

- semantic authority model
- larangan shadow semantics
- reuse dan equivalence rule
- transport-neutral semantic preservation
- protection of core invariants
- compliance evidence minimum
- change control pada level implementasi

## Single Semantic Authority

Setiap ekosistem implementasi **MUST** memiliki satu authority semantik yang jelas.

Authority ini **MUST** menetapkan secara konsisten:

- representasi `Money`
- arithmetic invariants
- normalization rules
- rounding rules
- policy interpretation
- determinism expectations
- compliance vectors

Authority semantik ini **MAY** diwujudkan sebagai:

- satu library inti
- satu executable specification
- satu compliance suite
- satu reference implementation
- atau kombinasi yang setara

Yang normatif adalah kesatuan semantic behavior, bukan bentuk teknis authority tersebut.

## No Shadow Semantics

Implementasi turunan **MUST NOT** mendefinisikan ulang invariant inti secara diam-diam.

Secara khusus, implementasi turunan **MUST NOT**:

- mengubah makna `amount`, `currency`, `scale`, atau invariant representasi inti
- mengubah equality, comparison, normalization, atau rounding behavior di luar aturan standard
- memperkenalkan shortcut numerik yang menghasilkan semantic berbeda
- melemahkan invariant inti dengan alasan transport, parsing, persistence, atau convenience lokal

Perilaku tambahan **MUST** dibangun di atas semantic inti, bukan dengan mengubah semantic inti tersebut.

## Approved Reuse or Proven Equivalence

Jika komponen inti resmi tersedia, implementasi turunan **SHOULD** menggunakannya.

Jika implementasi turunan tidak menggunakan komponen inti resmi, implementasi tersebut **MUST** membuktikan equivalence semantik melalui:

- compliance test yang sama
- golden vector yang sama
- invariant checks yang sama
- hasil deterministik yang sama untuk input yang sama

Multiple implementation **MAY** ada, tetapi semuanya **MUST** tunduk pada authority semantik yang sama.

## Transport Neutrality with Semantic Preservation

Standard ini bersifat transport-neutral.

Karena itu:

- encoding transport **MAY** berbeda antar protocol
- semantic value **MUST** tetap identik setelah parsing
- parsing dan serialization **MUST NOT** menyebabkan truncation, reinterpretation, precision drift, atau semantic fork

Jika suatu transport tidak dapat menjamin preservasi lossless untuk semantic integer yang diwajibkan standard, implementasi **MUST** memakai alternate encoding yang lossless.

## Invariant Protection

Invariant berikut **MUST** diperlakukan sebagai protected invariants:

- strict equality
- normalization requirement
- currency isolation
- zero discipline
- explicit rounding requirement
- append-only correction semantics
- policy-version traceability
- deterministic replay

Implementasi turunan **MUST NOT** meng-override invariant tersebut pada level runtime, adapter, service, consumer logic, atau transport boundary.

## Compliance Evidence

Implementasi yang mengklaim compliance **MUST** mampu menyediakan evidence minimum berikut:

- versi standard yang diikuti
- versi policy atau adapter yang aktif
- hasil compliance test
- daftar known deviations, jika ada
- batas compatibility transport yang didukung

Jika terdapat deviasi dari semantic inti, deviasi tersebut **MUST** dinyatakan eksplisit dan **MUST NOT** diperlakukan sebagai compliant behavior tanpa disclosure.

## Change Control

Perubahan yang memengaruhi semantic result **MUST** diperlakukan sebagai controlled change.

Perubahan berikut **MUST** dianggap breaking secara semantic jika mengubah hasil numerik atau interpretasi contract:

- representasi `Money`
- normalization rule
- rounding rule
- tax basis rule
- correction semantics
- equality or comparison behavior

Perubahan semacam itu **MUST** disertai salah satu dari:

- standard version baru
- policy version baru
- adapter version baru
- atau disclosure compatibility yang setara

## Derived System Obligations

Setiap derived system yang menerima, menyimpan, mentransformasi, atau menghitung nilai finansial **MUST**:

- menjaga invariant inti tetap utuh
- tidak membuat semantic fork lokal
- tidak memperkenalkan reinterpretation diam-diam
- tidak menyamarkan normalization atau conversion sebagai manipulasi integer biasa

Derived system **SHOULD** memperlakukan authority semantik sebagai dependency eksplisit, bukan asumsi implisit.

## Relationship to Compliance Review

Dokumen ini menetapkan governance normatif.

Dokumen [13-Compliance-Test](13-Compliance-Test.md) berfungsi sebagai:

- checklist kepatuhan minimum
- review form untuk governance review
- evidence capture untuk sign-off implementasi

Jika terjadi konflik pembacaan, dokumen governance ini menjadi sumber authority normatif, sedangkan `Compliance-Test` menjadi alat verifikasi operasional.

Untuk peta ownership dokumen secara ringkas, lihat [24-Document-Map](24-Document-Map.md).

## Concise Normative Summary

- Harus ada satu authority semantik.
- Implementasi turunan **MUST NOT** membuat shadow semantics.
- Reuse komponen inti **SHOULD** dilakukan jika tersedia.
- Jika reuse tidak dilakukan, equivalence semantik **MUST** dibuktikan.
- Transport boleh berbeda, tetapi semantic result **MUST** tetap identik.
- Invariant inti **MUST** dilindungi.
- Klaim compliance **MUST** didukung evidence yang dapat diverifikasi.
