# Policy and Versioning

## Objective

Dokumen ini mendefinisikan bagaimana policy finansial dibentuk, disnapshot, dan diversikan agar hasil komputasi tetap replayable.

## Normative Scope

Dokumen ini adalah owner untuk:

- semantic `policy_version`
- policy snapshot minimum
- effective version selection
- compatibility rules
- migration expectation ketika policy berubah

## Core Rule

Setiap rule yang dapat mengubah hasil numerik harus berada di bawah policy yang versioned.

Contoh area policy:

- scale policy
- rounding policy
- allocation policy
- tax policy binding
- normalization policy

## Policy Snapshot

Sebuah computation context minimum sebaiknya dapat menunjuk atau membawa snapshot policy yang relevan.

Minimum semantics:

```text
policy_version
effective_at
```

Snapshot dapat:

- disimpan penuh bersama event/record, atau
- direferensikan melalui identifier yang stabil dan dapat direkonstruksi

Project harus memilih salah satu pendekatan secara eksplisit.

## Versioning Rules

- Breaking semantic change harus menghasilkan major policy version baru atau identifier baru yang ekuivalen.
- Minor extension yang tidak mengubah hasil existing boleh memakai increment non-breaking.
- Historical event tidak boleh direinterpretasi memakai policy terbaru secara diam-diam.

## Effective Version Selection

Implementasi harus dapat menjawab:

- policy mana yang berlaku pada waktu transaksi dibuat
- policy mana yang digunakan saat komputasi dijalankan
- apakah sistem mengizinkan recomputation dengan policy historis

Jika `effective_at` dan `computed_at` menghasilkan policy berbeda, contract domain harus menentukan mana yang authoritative.

## Migration Guidance

Ketika policy berubah, project harus memilih salah satu:

- freeze historical result
- recompute dengan policy lama
- recompute dengan policy baru melalui event koreksi eksplisit

Pemilihan strategi ini tidak boleh dibiarkan implicit.

## Compatibility Outline

Implementasi yang patuh harus dapat membuktikan:

- setiap hasil final dapat ditelusuri ke `policy_version`
- perubahan policy tidak mengubah historical meaning secara diam-diam
- compatibility contract dijaga lintas release
