# 09-FFI-Plan — Multi-language Integration Plan (Ringkas)

## Objective

Dokumen ini mendefinisikan rencana integrasi lintas bahasa (_multi-language_) untuk **Financial Computation Standard**
melalui FFI, tanpa mengubah prinsip inti kernel:

- deterministik
- audit-friendly
- versionable
- reusable lintas project

Saat ini FFI **belum diimplementasikan**. Dokumen ini berfungsi sebagai _guardrail_ desain API agar implementasi FFI di
masa depan tidak memerlukan refactor besar.

---

## Scope

Rencana FFI ini mencakup:

- strategi exposing API `fcs-core` ke bahasa lain
- aturan desain API agar FFI-friendly (DTO, error semantics, versioning)
- pendekatan packaging dan kompatibilitas rilis
- baseline testing untuk memastikan hasil lintas bahasa konsisten

Rencana FFI ini **tidak** mencakup:

- detail implementasi binding per bahasa (JVM, Go, .NET) sekarang
- keputusan toolchain final (mis. UniFFI vs C ABI manual) sebagai kontrak permanen
- keputusan distribusi artifact lintas platform (CI build matrix) pada fase awal

---

## Design Principle for FFI

### 1) API Surface MUST be Stable and Minimal

- API publik yang diekspos ke FFI **MUST** seminimal mungkin.
- Refactor internal Rust **MUST NOT** memaksa perubahan API FFI.

Prinsip: **internal detail boleh berubah; kontrak FFI tidak.**

### 2) Exposed Types SHOULD be Plain DTO

Tipe yang diekspos ke FFI sebaiknya berupa struktur data sederhana:

- integer (`i64`, `u64`)
- string (`String`)
- list (`Vec<T>`)
- enum sederhana tanpa generics

Hindari mengekspose secara langsung:

- lifetime / reference (`&T`, `&str`) pada API publik FFI
- generic types pada signature publik
- trait object sebagai parameter/return utama
- closure/callback kompleks (kecuali diperlukan dan diputuskan kemudian)

### 3) Computation MUST be Deterministic

Semua fungsi yang diekspos ke FFI:

- **MUST** pure/deterministik
- **MUST NOT** bergantung pada global mutable state
- **MUST** memiliki hasil yang identik untuk input yang identik

### 4) Errors MUST be Structured

Error yang diekspos ke FFI **MUST** punya struktur:

- `code` (enum atau string stabil)
- `message` (human-readable)
- `details` (opsional)

Tujuannya agar bahasa lain bisa memetakan error menjadi exception/Result tanpa kehilangan konteks audit.

---

## Planned Approach (Initial Direction)

Rencana awal FFI adalah menggunakan **Mozilla UniFFI (`uniffi-rs`)** untuk menghasilkan binding lintas bahasa dari
interface Rust yang deklaratif.

Catatan:

- Ini adalah **initial plan**, bukan keputusan final.
- Implementasi UniFFI **belum termasuk** dalam scope rilis awal.

---

## Packaging Strategy (Future)

Ketika FFI diimplementasikan, disarankan adanya crate terpisah:

- `fcs-core` tetap menjadi kernel logic dan model
- `fcs-ffi` (atau `fcs-bindings`) bertugas sebagai boundary FFI dan tooling bindings

Rationale:

- memisahkan stabilitas ABI/binding dari perubahan internal core
- memudahkan build artifact per platform tanpa “mengotori” `fcs-core`

---

## Versioning Rules for FFI Compatibility

Repo ini mem-versioning **standard/spec** melalui Git tag release (`vX.Y.Z`).

Konvensi versioning untuk FFI yang disarankan:

- **Standard Version**: mengikuti tag repo `vX.Y.Z`
- **Crate Version**: `fcs-core` mengikuti `X.Y.Z` yang sama
- **Bindings Version**: jika ada artifact binding, mengikuti `X.Y.Z` yang sama

### Compatibility Rules

- Breaking change pada kontrak data/semantics **MUST** menaikkan **major version**.
- Minor/Patch **SHOULD** mempertahankan kompatibilitas binding untuk API yang sudah ada.
- Jika terdapat perubahan implementasi tanpa mengubah spec, bump versi tetap mengikuti tag (patch) dengan catatan di
  CHANGELOG.

---

## FFI Contract Surface (Target Minimum)

FFI (jika diimplementasikan) sebaiknya menyediakan fungsi minimum berikut:

### Money

- validasi invariant `Money`
- operasi dasar yang aman (opsional, jika dibutuhkan)
- normalisasi/konversi scale (hanya jika policy mendefinisikan)

### Rounding

- `round_to_quantum(amount, mode, quantum) -> amount`
- daftar `RoundingMode` yang stabil

### Tax (Abstraction)

- kontrak `TaxPolicySnapshot` dan `TaxResult` (DTO)
- mekanisme menjalankan adapter (jika adapter di-host di Rust)
- **Catatan:** pemanggilan adapter eksternal lintas bahasa adalah ruang desain terpisah dan tidak diwajibkan di fase awal.

### Correction

- model record correction (DTO)
- helper invariant minimal (mis. reference id wajib, sign amount, dsb)

---

## Data Ownership & Memory (Guideline)

Agar aman lintas bahasa:

- API FFI **SHOULD** menghindari return pointer mentah.
- Semua data output **SHOULD** berupa nilai (owned) yang dapat diserialisasi.
- Jika diperlukan handle/object lifecycle, harus ada aturan eksplisit untuk:
  - create
  - use
  - free / drop

Aturan detail akan ditetapkan saat implementasi binding diputuskan.

---

## Serialization Strategy (Optional)

Untuk integrasi antar service, pendekatan yang disarankan:

- Kontrak DTO dapat diserialisasi ke JSON sesuai `07-Reference-Contracts.md`
- FFI boleh menyediakan helper encode/decode JSON **opsional**
- JSON **tidak** menggantikan fixed-point integer sebagai authority

Tujuannya: bahasa lain bisa integrasi cepat dengan contract yang sama.

---

## Testing Strategy (FFI Readiness)

Saat FFI diimplementasikan, kompatibilitas lintas bahasa **MUST** diuji dengan:

1.  **Golden tests** (vector input/output) untuk rounding, tax example, correction invariant
2.  **Cross-language parity tests**:
    - input sama → output sama (bitwise untuk integer)
3.  **Versioned contract tests**:
    - memastikan perubahan kontrak mengikuti aturan versioning

Golden tests akan ditempatkan di folder referensi (mis. `reference/golden/`) ketika siap.

---

## Non-Goals (Explicit)

- FFI bukan pengganti desain domain layer project.
- FFI tidak akan memaksakan style arsitektur (RPC/event sourcing/etc).
- FFI tidak akan memasukkan business workflow (order/invoice lifecycle).

---

## Roadmap (High Level)

### Phase 0 — Now (Spec & Core)

- stabilkan kontrak data dan invariant
- jaga API `fcs-core` tetap DTO-friendly
- susun golden tests (opsional, namun sangat disarankan)

### Phase 1 — FFI Prototype (Future)

- tambah crate `fcs-ffi`
- definisikan interface binding minimal (Money/Rounding dulu)
- generate binding (UniFFI atau alternatif)

### Phase 2 — Production Readiness

- release artifact lintas platform
- cross-language parity tests di CI
- dokumentasi integrasi per bahasa

---

## Decision Log

- Initial plan: UniFFI (`uniffi-rs`) untuk binding lintas bahasa.
- Implementasi ditunda sampai `fcs-core` stabil dan golden tests tersedia.
