# Arithmetic and Allocation

## Objective

Dokumen ini mendefinisikan semantic operasi inti untuk monetary computation yang generik dan deterministik.

## Normative Scope

Dokumen ini adalah owner untuk:

- penjumlahan dan pengurangan monetary amount
- apply-rate dan multiply semantics
- division semantics
- allocation dan apportionment
- residual handling
- valid vs invalid operation boundary

## Operation Preconditions

Operasi `add`, `subtract`, `compare`, dan `allocate` hanya valid bila:

- `currency` identik
- `scale` identik
- effective monetary context sudah resolved secara deterministik
- policy tambahan yang diperlukan tersedia

Jika precondition gagal, implementasi harus:

- menolak operasi, atau
- menerima operand yang sudah lebih dulu dinormalisasi secara eksplisit sesuai policy yang terdokumentasi

Core arithmetic pada dokumen ini tidak boleh melakukan normalisasi implicit untuk menebak target `scale`.
Pemilihan target `scale` adalah tanggung jawab consumer, contract, atau policy owner di luar operasi inti ini.

## Normalization and Comparison Semantics

Dokumen ini mengakui bahwa normalisasi representasi dapat diperlukan sebelum arithmetic atau comparison dilakukan,
tetapi dokumen ini bukan owner dari algoritme normalisasi generik lintas `scale`.

Normalisasi dalam konteks dokumen ini hanya berfungsi sebagai pre-step eksplisit yang dilakukan di luar core operation
sebelum operand masuk ke `add`, `subtract`, `compare`, atau `allocate`.

Karena itu:

- core tidak boleh melakukan auto-normalization
- core tidak boleh menebak target `scale`
- hasil operasi inti hanya valid setelah semua operand memiliki `currency` dan `scale` efektif yang identik

Normalisasi hanya boleh diperlakukan valid bila:

- semantic unit tetap sama
- transformasi dilakukan secara deterministik
- hasilnya lossless

Jika normalisasi lintas `scale` membutuhkan truncation, rounding, atau perubahan representasi yang tidak lossless,
maka hasil tersebut tidak boleh dipakai sebagai dasar comparison generik pada dokumen ini.

Kebutuhan semacam itu harus dimodelkan sebagai concern policy atau rounding domain yang terpisah.

### Comparison

Operasi comparison pada dokumen ini menghasilkan salah satu dari:

- `LESS_THAN`
- `EQUAL`
- `GREATER_THAN`

Pada level model generik, comparison monetary tetap membentuk partial order, bukan total order universal.
Namun operasi inti `compare` pada dokumen ini hanya menerima operand yang sudah comparable.

Comparison numerik hanya valid bila:

- `currency` identik
- `scale` identik

Jika dua operand awalnya memiliki `scale` berbeda, consumer wajib melakukan normalisasi mandiri terlebih dahulu.
Core tidak boleh menerima mismatch itu lalu memilih sendiri representasi pembandingnya.

Jika salah satu syarat comparison tidak terpenuhi, operasi harus ditolak sebagai invalid comparison.

## Addition and Subtraction

Aturan dasar:

```text
result.amount = left.amount + right.amount
```

dan

```text
result.amount = left.amount - right.amount
```

Invariants:

- `currency` hasil harus sama dengan operand
- `scale` hasil harus sama dengan operand
- overflow harus diperlakukan sebagai error, bukan wraparound diam-diam

## Multiply by Rate

Perkalian dengan rate digunakan untuk:

- derived price amount
- tax
- fee
- discount
- proportional allocation
- field lain yang membutuhkan perhitungan berbasis rate

Standar ini merekomendasikan rate direpresentasikan sebagai rasio **integer** atau fixed-point policy-defined, bukan floating point.

Reference model:

```text
raw_amount = amount * numerator / denominator
```

Operasi `multiply by rate` dapat menghasilkan `raw_amount` sebagai intermediate result.

Jika hasil tersebut menurut policy aktif masih memerlukan rounding atau quantization tambahan, hasil itu belum boleh
diperlakukan sebagai final amount.

Jika hasilnya sudah exact pada representasi yang berlaku dan policy aktif tidak mensyaratkan transformasi tambahan,
hasil tersebut boleh diperlakukan sebagai final amount.

## Rounding Policy

Dokumen ini mengakui rounding sebagai boundary operasional yang diperlukan ketika hasil `multiply by rate`, division
finalization, atau transformasi lain masih membutuhkan representasi hasil yang lebih sempit daripada raw computation
menurut policy yang aktif.

Namun dokumen ini tidak menjadi owner untuk:

- target settlement quantum
- target regulatory quantum
- target reporting precision final

Concern tersebut boleh dimiliki oleh contract, policy implementasi, atau domain rule di layer atas core arithmetic.

Dalam ruang lingkup dokumen ini:

- rounding mode harus ditentukan secara eksplisit oleh policy yang relevan
- dokumen ini tidak menetapkan satu daftar mode rounding universal yang wajib untuk semua project
- policy yang aktif boleh dan sebaiknya membatasi mode rounding yang legal untuk domain tersebut
- implementasi tidak boleh memakai rounding mode di luar yang diizinkan oleh policy aktif
- core arithmetic tidak boleh menebak quantum atau precision final milik domain tertentu
- hasil rounding untuk satu domain tidak boleh otomatis dianggap sebagai canonical representation untuk domain lain

Jika suatu operasi menghasilkan `raw_amount` yang masih membutuhkan rounding atau quantization domain-specific, maka
nilai tersebut boleh hidup sebagai intermediate computational value.

Intermediate value semacam itu:

- belum boleh diperlakukan sebagai final amount
- belum boleh dipakai sebagai authoritative settlement amount
- belum boleh dipakai sebagai authoritative reporting amount
- belum boleh menggantikan canonical stored amount tanpa policy domain yang relevan

Jika policy rounding yang relevan sudah tersedia, implementasi harus memastikan:

- mode rounding yang dipakai deterministik
- stage penerapan rounding terdokumentasi
- hasil replayable untuk input dan policy yang sama

Rounding yang berasal dari concern regulatory, settlement, tax, cash handling, atau reporting domain harus diperlakukan
sebagai transformasi domain-specific, bukan sebagai perilaku implicit dari primitive `Money` itu sendiri.

## Division Semantics

Division finansial hampir selalu menghasilkan sisa. Karena itu:

- division pada core monetary harus mengikuti integer atau fixed-point semantics yang deterministik
- hasil quotient harus deterministik
- residual harus punya aturan distribusi
- division tidak boleh menyembunyikan loss secara implicit
- implementasi tidak boleh memakai floating point binary computation lalu membulatkan balik sebagai source of truth

Dalam dokumen ini, division tidak boleh diasumsikan selalu menghasilkan satu `Money` final.
Primitive `Money` tetap sederhana pada level representasi data, tetapi hasil aritmatika division harus tetap menjaga
conservation of amount secara eksplisit.

Reference model:

```text
quotient = dividend / divisor
remainder = dividend % divisor
```

Bentuk hasil normatif minimum untuk division adalah:

```text
DivisionResult {
  quotient,
  remainder
}
```

`quotient` dan `remainder` harus memakai `currency` dan `scale` efektif yang sama dengan dividend.

Implementasi atau policy boleh menambahkan metadata turunan seperti `carry`, note, atau presentation field lain,
tetapi field semacam itu tidak boleh menggantikan `remainder` sebagai source of truth untuk conservation.

Invariant rekonsiliasi minimum untuk `DivisionResult` adalah:

```text
dividend = divisor * quotient + remainder
```

`quotient` dan `remainder` harus ditentukan secara deterministik sehingga invariant tersebut dapat direplay untuk input
yang sama.

`remainder` harus:

- ditolak bila context tidak mengizinkan residual
- atau didistribusikan secara eksplisit oleh allocation policy

Hasil tunggal berupa `Money` hanya boleh diperlakukan final bila `remainder = 0` atau residual sudah diselesaikan
secara eksplisit oleh policy yang relevan.

## Allocation

Allocation adalah pembagian satu amount atau residual ke banyak target dengan total konservatif.

Invariant allocation:

- jumlah seluruh hasil allocation harus sama dengan original amount
- tidak boleh ada unit value hilang atau tercipta
- urutan distribusi residual harus deterministik

Reference algorithm family yang diizinkan:

- largest remainder
- stable index order
- weighted allocation berdasarkan integer weight

Project boleh memilih algoritme, tetapi harus:

- menamai policy-nya
- menjaga hasil replayable
- mendokumentasikan aturan tie-break residual

## Residual Handling

Residual handling minimum harus menyatakan:

- residual unit minimum dalam skala saat ini
- siapa yang menerima residual pertama
- urutan tie-break
- apakah residual boleh tetap tersisa sebagai carry

Contoh policy:

```text
allocation_policy = WEIGHTED_LARGEST_REMAINDER_V1
residual_tiebreak = INPUT_ORDER
```

## Invalid Operations

Operasi berikut tidak valid tanpa policy tambahan:

- add/subtract lintas currency
- add/subtract lintas scale
- division by zero
- divide tanpa definisi residual handling
- multiply by floating point binary number sebagai source of truth
- mempromosikan raw result yang masih membutuhkan rounding domain-specific menjadi final amount tanpa policy yang relevan
- memperlakukan hasil division dengan `remainder != 0` sebagai satu `Money` final tanpa penyelesaian residual yang eksplisit

## Compliance Outline

Implementasi yang patuh harus dapat membuktikan:

- addition/subtraction tidak menerima mismatch representation
- multiply/divide tidak kehilangan residual secara diam-diam
- allocation menjaga conservation of amount
- algoritme residual dapat direplay dengan input yang sama
