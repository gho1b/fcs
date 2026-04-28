# Money

## Objective

Dokumen ini mendefinisikan cara merepresentasikan nilai uang secara stabil di JSON, database, event, dan reporting.

Definisi ini bersifat **netral terhadap currency manapun** — berlaku untuk IDR, USD, EUR, JPY, dan currency lainnya.

## Canonical Representation

Semua monetary value direpresentasikan sebagai fixed-point integer:

```text
decimal_value = amount / scale
```

> **Contoh berikut bersifat ilustratif.** Currency dan scale dipilih oleh policy project yang bersangkutan.

Contoh dengan IDR (3 digit precision):

```text
amount   = 150000000
currency = IDR
scale    = 1000
decimal_value = 150000.000
```

Contoh dengan USD (2 digit precision):

```text
amount   = 10099
currency = USD
scale    = 100
decimal_value = 100.99
```

## Monetary Value and Monetary Header

`amount` adalah bagian dari **monetary value**, bukan bagian wajib dari setiap header atau envelope metadata.

Shape minimum untuk satu nilai uang:

```json
{
  "amount": <integer>,
  "currency": "<currency-code>",
  "scale": <integer>
}
```

Jika project memisahkan metadata representasi dari nilai, maka header minimum dapat berbentuk:

```json
{
  "currency": "<currency-code>",
  "scale": <integer>
}
```

Dalam model ini, `amount` boleh berada di field detail, line item, atau record value yang memakai header tersebut.

> `currency` mengikuti ISO 4217 atau kode yang disepakati project. `scale` ditentukan oleh policy
> yang aktif, bukan oleh standar ini secara hardcoded.

## Custom Unit Boundary

Standar ini mengizinkan `currency` berupa:

- kode ISO 4217
- kode currency non-ISO yang disepakati project
- unit nilai internal yang diperlakukan seperti monetary unit oleh contract project

Namun project tidak boleh mencampur ketiganya secara ambigu.

Jika unit internal atau pseudo-currency digunakan, project harus memastikan:

- identifier unit stabil
- semantic unit terdokumentasi
- unit tidak diperlakukan setara dengan legal tender tanpa contract eksplisit

Contoh yang termasuk kategori ini:

- store credit
- loyalty point
- internal settlement unit
- token bernilai tetap

Format `currency` harus mengikuti spesifikasi identifier yang dipilih oleh project.

Contoh:

- jika project memilih ISO 4217, maka format `currency` harus mengikuti format ISO 4217
- jika project memilih namespace internal, maka formatnya harus mengikuti standard internal tersebut secara konsisten

Standar ini tidak menetapkan satu format universal di atas semua namespace, tetapi mewajibkan project memilih satu aturan format yang stabil dan terdokumentasi.

Project-specific policy boleh menambahkan:

```json
{
  "policy_version": "<policy-name>/<version>"
}
```

## Normative Scope

Dokumen ini adalah owner untuk:

- semantic `amount`
- semantic `currency`
- semantic `scale`
- sign semantics untuk monetary value
- aturan validitas dasar sebelum arithmetic dilakukan

Dokumen ini **sengaja** mempertahankan `Money` sebagai primitive generik.

Karena itu, dokumen ini tidak memasukkan context domain seperti:

- authority source
- valuation context
- settlement context
- reporting role
- balance state

Concern tersebut harus dimodelkan pada contract, policy, atau object domain di layer atas `Money`, bukan di primitive `Money` itu sendiri.

## Invariants

- `amount` adalah integer.
- `scale` adalah integer positif.
- Operasi aritmetika hanya valid bila `currency` dan `scale` sama.
- Stored fixed-point amount adalah source of truth.
- Normalized presentation value tidak boleh disimpan sebagai authority pengganti.
- Implementasi boleh memisahkan `amount` dari header representasi selama relasi nilainya tetap eksplisit dan tidak ambigu.
- `currency` harus diperlakukan sebagai identifier kanonik yang case-sensitive menurut contract project yang aktif.
- format kanonik `currency` harus mengikuti spesifikasi identifier yang dipilih oleh project.

## Currency Availability Boundary

Pada boundary `Money` yang dianggap valid oleh standard ini:

- `currency` harus sudah diketahui
- `scale` harus sudah diketahui

Karena itu, unknown currency atau deferred currency assignment tidak termasuk bentuk `Money` final yang valid.

Project boleh memiliki payload ingestion sementara yang belum lengkap, tetapi payload tersebut:

- belum boleh diperlakukan sebagai `Money` yang sah untuk arithmetic atau comparison
- harus diselesaikan menjadi `currency` dan `scale` yang final sebelum masuk ke financial computation kernel

## Zero Semantics

- `amount = 0` adalah nilai yang valid.
- Nilai nol tetap harus membawa `currency` dan `scale` yang jelas.
- Nilai nol tidak menghapus kewajiban contract untuk mendefinisikan context representasinya.
- Implementasi tidak boleh memperlakukan nilai nol sebagai "tidak ada nilai" kecuali contract spesifik memang menyatakannya.
- Nilai nol tetap mengikuti aturan equality dan comparison yang sama seperti nilai non-nol.

Contoh:

Valid:

```text
amount=0, currency=USD, scale=100
amount=0, currency=USD, scale=1000
=> dapat dibandingkan atau dianggap setara hanya setelah normalisasi eksplisit
```

Invalid:

```text
amount=0, currency=USD, scale=100
amount=0, currency=EUR, scale=100
=> tidak boleh dianggap setara tanpa conversion policy terpisah
```

## Sign Semantics

- Standar ini mengizinkan `amount` bernilai positif, nol, atau negatif.
- Makna sign harus ditentukan oleh contract atau event type yang menggunakannya.
- `Money` sebagai value object tidak membawa semantic debit/credit secara mandiri.
- Implementasi tidak boleh mengasumsikan `negative == invalid` kecuali contract spesifik memang melarangnya.
- Jika suatu record hanya mengizinkan amount non-negatif, larangan tersebut harus dinyatakan di contract owner-nya.

## Equality and Comparison

### Strict Equality

Dua `Money` dianggap **strictly equal** hanya bila:

- `amount` identik
- `currency` identik
- `scale` identik

Jika `currency` atau `scale` diwariskan dari header, maka strict equality dihitung setelah context efektif untuk masing-masing value berhasil di-resolve secara deterministik.

### Normalized Equality

Dua `Money` hanya boleh dianggap setara setelah normalisasi bila:

- hasil normalisasi mempertahankan semantic unit yang sama

Normalisasi representasi pada dasarnya valid selama:

- semantic unit tetap sama
- transformasi scale dilakukan secara deterministik

Namun project harus tetap menyadari bahwa normalisasi dapat mengubah nilai representasi akhir bila:

- input memakai scale yang lebih presisi daripada scale internal
- proses normalisasi memerlukan pembagian yang menghasilkan truncation atau rounding

Dalam kasus seperti itu, hasil normalisasi tetap valid sebagai hasil transformasi policy, tetapi tidak boleh diasumsikan identik dengan representasi input awal.

Contoh normalisasi lintas scale:

Valid:

```text
input  = amount=1000, currency=USD, scale=1000
target = scale=100
result = amount=100, currency=USD, scale=100
=> valid bila dilakukan sebagai normalisasi eksplisit
```

Valid dengan perubahan representasi:

```text
input  = amount=15, currency=USD, scale=1000
target = scale=100
result = amount=1, currency=USD, scale=100
=> valid bila policy normalisasi memang mengizinkan truncation atau rounding
```

Invalid:

```text
input  = amount=1000, currency=USD, scale=1000
other  = amount=100, currency=USD, scale=100
=> tidak boleh langsung dianggap setara tanpa langkah normalisasi eksplisit
```

Tanpa normalisasi eksplisit, implementasi tidak boleh menganggap nilai berikut setara:

```text
amount=100,  currency=USD, scale=100
amount=1000, currency=USD, scale=1000
```

Jika `Money` berasal dari header inheritance, normalized equality hanya boleh dievaluasi setelah semua field efektif (`amount`, `currency`, `scale`) pada masing-masing value sudah resolved.

### Comparison

- Comparison numerik valid bila `currency` dan `scale` identik.
- Comparison lintas `scale` hanya valid setelah normalisasi eksplisit.
- Comparison lintas `currency` tidak valid tanpa conversion policy yang terpisah.
- Dokumen ini tidak mendefinisikan FX conversion; normalisasi bukan conversion mata uang.

Dengan aturan ini, `Money` secara umum membentuk **partial order**, bukan total order universal.

Artinya:

- dua nilai dengan context identik dapat dibandingkan langsung
- dua nilai dengan scale berbeda dapat dibandingkan setelah normalisasi eksplisit
- dua nilai dengan currency berbeda tidak memiliki ordering generik di dokumen ini

## Validation Guidance

Minimal validasi `Money`:

- `currency` wajib ada
- `scale > 0`
- `amount` harus berada dalam rentang tipe integer yang dipilih implementasi
- comparison atau arithmetic lintas `currency` atau `scale` tidak valid tanpa normalisasi eksplisit
- jika header dipisah dari value, aturan inheritance dan override harus terdokumentasi

## Reporting Rule

- Reporting tidak boleh mengagregasi raw amount lintas transaksi kecuali `currency` dan `scale` identik.
- Jika `scale` berbeda, reporting harus menormalisasi setiap transaksi sebelum agregasi.

## Header Inheritance and Override

Jika project memakai shared monetary header, precedence minimum harus jelas.

Aturan yang direkomendasikan:

1. field yang berada pada monetary value lokal mengalahkan header induk
2. jika value tidak membawa `currency` atau `scale`, header induk boleh menjadi source of truth
3. jika beberapa header berlaku sekaligus, contract project harus menentukan urutan precedence-nya
4. conflict antara header dan value lokal tidak boleh diselesaikan diam-diam

Project harus memilih salah satu perilaku ketika terjadi conflict:

- reject input
- normalize secara eksplisit
- gunakan local override sesuai contract yang terdokumentasi

Implicit inheritance diperbolehkan hanya jika source header yang dipakai dapat ditentukan tanpa ambigu.

Jika inheritance terjadi pada lebih dari satu level, contract project harus menentukan precedence dari yang paling dekat ke value hingga yang paling jauh.

Aturan yang direkomendasikan:

1. local value
2. nearest parent header
3. enclosing aggregate header
4. outermost envelope header

Jika dua header pada level yang sama memberi context yang bertentangan, input harus ditolak kecuali contract secara eksplisit mendefinisikan tie-break rule.

Contoh:

Valid:

```json
{
  "currency": "USD",
  "scale": 100,
  "lines": [
    {
      "amount": 1000
    }
  ]
}
```

`line.amount` mewarisi `currency=USD` dan `scale=100` dari header.

Valid jika contract mengizinkan local override:

```json
{
  "currency": "USD",
  "scale": 100,
  "lines": [
    {
      "amount": 1000,
      "scale": 1000
    }
  ]
}
```

Valid hanya bila contract project menyatakan field lokal mengalahkan header.

Invalid:

```json
{
  "currency": "USD",
  "scale": 100,
  "lines": [
    {
      "amount": 1000,
      "currency": "EUR"
    }
  ]
}
```

Invalid bila contract tidak mendefinisikan override currency lintas header atau conflict tersebut tidak ditangani secara eksplisit.

Invalid pada nested inheritance tanpa precedence yang jelas:

```json
{
  "currency": "USD",
  "scale": 100,
  "batch": {
    "currency": "EUR",
    "lines": [
      {
        "amount": 1000
      }
    ]
  }
}
```

Jika project tidak mendefinisikan apakah `line.amount` mewarisi `batch.currency` atau `root.currency`, payload ini ambigu dan harus ditolak.

## Large Integer and Serialization Safety

Karena dokumen ini menargetkan JSON, event, database, dan integrasi lintas bahasa, implementasi harus memperhatikan keamanan integer besar.

Minimum guardrail:

- `amount` tidak boleh silently truncated saat serialisasi atau deserialisasi
- consumer tidak boleh mengubah `amount` menjadi floating point binary number sebagai source of truth
- project harus mendokumentasikan apakah integer dikirim sebagai number atau string pada boundary yang rawan precision loss

Rekomendasi interoperability minimum:

- project sebaiknya menentukan satu baseline signed integer compatibility untuk contract publik
- jika ada consumer JavaScript atau runtime lain yang tidak aman untuk integer besar, representasi string sebaiknya dipilih pada boundary tersebut
- intermediate representation internal boleh lebih lebar dari contract publik selama semantic result tidak berubah

Baseline kompatibilitas minimum yang direkomendasikan oleh standard ini adalah **signed 64-bit integer semantics**.

Artinya:

- semua implementasi sebaiknya mampu merepresentasikan dan memproses `amount` dalam rentang signed 64-bit tanpa kehilangan nilai
- representasi transport tidak harus selalu berupa literal `int64`, tetapi semantic nilai signed 64-bit harus tetap terjaga
- implementasi internal boleh memakai `i128`, bigint, atau representasi lain yang lebih lebar selama kompatibilitas baseline signed 64-bit tetap dijaga

Jika suatu transport atau runtime tidak dapat menjamin preservasi lossless untuk signed 64-bit integer dalam encoding native-nya, implementasi **harus** memakai alternate encoding yang lossless.

Contoh:

- JSON dapat memakai string literal `"1234435"`
- Protobuf dapat memakai `int64`
- format internal lain dapat memakai `i128` selama nilai baseline tetap kompatibel

Jika suatu runtime atau protocol tidak dapat merepresentasikan integer besar dengan aman, project harus:

- menggunakan representasi string, atau
- membatasi rentang nilai yang diperbolehkan secara eksplisit

Pilihan ini adalah contract concern dan tidak boleh dibiarkan implicit.

## Storage Precision and Economic Precision

`scale` pada dokumen ini mendefinisikan precision representasi `Money`, bukan otomatis:

- settlement quantum
- cash rounding quantum
- legal tender minimum quantum
- presentation precision final

Karena itu, project harus membedakan setidaknya dua concern:

- **storage precision**: precision yang dipakai untuk menyimpan dan mengangkut `Money`
- **economic or settlement precision**: precision atau quantum yang berlaku saat nilai dipakai untuk settlement, display final, atau rule ekonomi tertentu

Contoh:

```text
currency = USD
storage scale = 1000
settlement quantum = 100
```

Artinya:

- sistem boleh menyimpan nilai dengan precision 3 digit
- tetapi settlement atau customer-facing amount dapat tetap tunduk pada quantum 2 digit

Dokumen ini tidak menetapkan bagaimana settlement precision dipilih. Pemilihan tersebut dimiliki oleh policy, rounding, atau contract domain yang relevan.

Namun implementasi tidak boleh mengasumsikan bahwa `scale` penyimpanan otomatis sama dengan precision ekonomi final.

## FX and Conversion Boundary

Dokumen ini tidak mendefinisikan foreign exchange atau conversion antar mata uang.

Secara khusus, dokumen ini tidak mendefinisikan:

- exchange rate
- source of FX rate
- conversion timestamp
- cross-currency settlement rule
- gain/loss semantics akibat conversion

Karena itu:

- normalisasi hanya berlaku untuk representasi dalam semantic unit yang sama
- conversion antar currency harus diperlakukan sebagai operasi terpisah dengan policy terpisah
- hasil conversion tidak boleh disamarkan sebagai normalisasi biasa

Contoh:

Valid normalization:

```text
amount=1000, currency=USD, scale=1000
-> amount=100, currency=USD, scale=100
```

Bukan normalization:

```text
amount=100, currency=USD, scale=100
-> amount=155000, currency=IDR, scale=100
```

Transformasi kedua adalah currency conversion dan berada di luar scope dokumen ini.

## Domain Context Boundary

Beberapa sistem bisnis, seperti core banking, treasury, clearing, atau ledger-heavy platform, dapat membutuhkan context moneter tambahan yang sangat kuat.

Contohnya:

- booked balance vs available balance
- ledger amount vs settlement amount
- preview valuation vs final valuation
- internal amount vs externally authoritative amount

Dokumen ini tidak memasukkan distinction tersebut ke dalam identity normatif `Money`.

Dengan demikian:

- dua `Money` dengan `amount`, `currency`, dan `scale` yang sama tetap dianggap identik pada level primitive ini
- jika project membutuhkan distinction domain yang lebih kuat, distinction tersebut harus dimodelkan di object atau contract yang membungkus `Money`
- arithmetic generic pada kernel ini tetap beroperasi pada primitive `Money`, bukan pada context domain yang lebih kaya

Keputusan ini diambil untuk menjaga standard tetap:

- generik
- interoperable
- ringan untuk implementasi
- stabil lintas project yang tidak memiliki semantic moneter seberat core banking

## Consequences

Positif:

- deterministik
- tidak ada floating point drift
- aman untuk audit
- berlaku lintas currency dan sistem

Negatif:

- consumer harus memahami `scale`
- perubahan `scale` memerlukan versioning atau _**migration strategy**_

## Out of Scope

Dokumen ini tidak mendefinisikan:

- cara menambah atau mengurangi dua nilai uang
- cara allocation residual dibagikan
- rounding mode yang digunakan pada titik tertentu

Semua itu dimiliki oleh dokumen lain dalam standard ini.
