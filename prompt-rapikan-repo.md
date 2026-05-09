# PROMPT: Rapikan Repositori & Dokumentasi QRD-SDK

---

## KONTEKS

Kamu adalah senior software engineer yang bertugas melakukan **repository cleanup dan documentation consolidation** pada proyek **QRD-SDK** — sebuah Rust SDK untuk format biner kolumnar. Proyek ini telah melalui beberapa fase pengembangan yang tidak tersinkronisasi, menghasilkan dokumentasi yang saling bertentangan dan struktur yang inkonsisten.

Hasil audit teknis menunjukkan 5 inkonsistensi kritis:
1. Status phase yang berbeda-beda di 4 file berbeda
2. Klaim jumlah test yang tidak akurat (klaim 115, aktual 208+)
3. Java binding disebut JNI di satu file, JNA di file lain
4. Status enkripsi: diklaim selesai di README, diklaim belum di CHANGELOG dan ROADMAP
5. URL repositori berbeda antara `Cargo.toml` dan `README.md`

---

## TUGASMU

Lakukan cleanup berikut secara menyeluruh dan sistematis. Ikuti setiap instruksi dengan teliti.

---

## BAGIAN 1: AUDIT CEPAT SEBELUM MULAI

Sebelum mengubah apapun, jalankan perintah berikut dan catat hasilnya:

```bash
# Hitung jumlah test aktual
grep -rn "#\[test\]" core/qrd-core/src/ | wc -l
grep -rn "#\[test\]" core/qrd-core/tests/ | wc -l

# Cek daftar member workspace
grep -A 10 "\[workspace\]" Cargo.toml

# Cek semua URL repo yang tersebut
grep -rn "github.com" . --include="*.toml" --include="*.md" | grep -v ".git"

# Cek klaim status di setiap dokumen
grep -n "Phase\|phase\|COMPLETE\|complete\|production.ready\|Production Ready" \
  README.md SDK_STATUS.md IMPLEMENTATION_STATUS.md CHANGELOG.md ROADMAP.md Phase.md
```

Gunakan output perintah di atas sebagai **sumber kebenaran** untuk semua perubahan dokumentasi.

---

## BAGIAN 2: BUAT SATU SUMBER KEBENARAN — `STATUS.md`

Buat file baru `STATUS.md` di root repositori. File ini adalah **satu-satunya sumber kebenaran** untuk status proyek. Gunakan template berikut dan isi dengan kondisi aktual berdasarkan audit cepat di Bagian 1:

```markdown
# QRD-SDK — Status Proyek

**Terakhir diperbarui:** [TANGGAL HARI INI]  
**Versi:** 0.1.0  
**Format QRD:** v1.0.0-draft

---

## Status Keseluruhan

> **[ALPHA / BETA / RC / STABLE]** — [satu kalimat deskripsi jujur kondisi saat ini]

---

## Komponen Core (qrd-core)

| Modul | Status | Catatan |
|---|---|---|
| Schema Engine | ✅ Lengkap | 20 tipe logis, deterministic hashing |
| Encoding (7 algoritma) | ✅ Lengkap | Auto-selection aktif |
| Kompresi (ZSTD + LZ4) | ✅ Lengkap | Entropy-based selection |
| Enkripsi (AES-256-GCM) | ⚠️ Parsial | Unit test lulus; belum terintegrasi di pipeline I/O |
| ECC (Reed-Solomon) | ⚠️ Parsial | Unit test lulus; belum terintegrasi di pipeline I/O |
| Writer (FileWriter) | ⚠️ Parsial | Tulis dasar berfungsi; enkripsi/ECC belum aktif |
| Writer (StreamingWriter) | ⚠️ Parsial | Sama dengan FileWriter |
| Reader (FileReader) | ⚠️ Parsial | Baca dasar berfungsi; dekripsi/ECC belum aktif |
| Reader (PartialReader) | ✅ Lengkap | Column-selective reads berfungsi |
| Footer Parser | ✅ Lengkap | |
| SIMD Utilities | ✅ Lengkap | Fallback scalar tersedia |
| Validasi (CRC32) | ✅ Lengkap | |

---

## Language Bindings

| Binding | Status | Catatan |
|---|---|---|
| Rust (native) | ✅ Dapat digunakan | Full feature access |
| FFI / C ABI | ✅ Dapat digunakan | `qrd-ffi` crate |
| Python (PyO3) | 🔄 Stub | Build via maturin; API terbatas |
| TypeScript (WASM) | 🔄 Stub | Writer berfungsi; reader belum implementasi |
| Go (CGO) | ❌ Broken | Bug include path; belum dapat di-build |
| Java (JNA) | 🔄 Stub | Struktur dasar ada; belum teruji end-to-end |

---

## Jumlah Test

| Lokasi | Jumlah |
|---|---|
| `core/qrd-core/src/` (unit tests) | [ISI DARI AUDIT] |
| `core/qrd-core/tests/` (integration tests) | [ISI DARI AUDIT] |
| **Total** | **[ISI DARI AUDIT]** |

---

## Known Issues

Lihat [AUDIT.md](./AUDIT.md) untuk daftar lengkap temuan.

**Blocker kritis sebelum beta:**
- [ ] Integrasikan enkripsi ke writer/reader pipeline
- [ ] Integrasikan ECC ke writer/reader pipeline
- [ ] Perbaiki Go CGO include path
- [ ] Daftarkan `qrd-wasm` ke Cargo workspace
- [ ] Perbaiki endianness FLAGS field di spesifikasi

---

## Roadmap

Lihat [ROADMAP.md](./ROADMAP.md).
```

---

## BAGIAN 3: PERBARUI `README.md`

Lakukan perubahan berikut pada `README.md`. **Jangan ubah konten teknis** (arsitektur, format, fitur) — hanya bagian status dan klaim.

### 3a. Perbarui badge status

Ganti:
```
[![Status](https://img.shields.io/badge/status-Phase%202%20Complete-brightgreen.svg)]
```
Dengan:
```
[![Status](https://img.shields.io/badge/status-Alpha-orange.svg)](./STATUS.md)
```

### 3b. Perbarui section "Project Status"

Ganti seluruh block progress bar phase dengan:
```markdown
## Project Status

**→ Lihat [STATUS.md](./STATUS.md) untuk status lengkap dan up-to-date.**

Ringkasan singkat:
- ✅ Rust core engine: fungsional dengan test komprehensif
- ✅ Spesifikasi format: lengkap (v1.0.0-draft)
- ⚠️ Enkripsi & ECC: diimplementasikan tapi belum terintegrasi ke pipeline I/O
- ❌ Language bindings: mayoritas masih stub atau broken
```

### 3c. Perbarui URL repositori

Cek URL yang benar dari remote git:
```bash
git remote get-url origin
```
Ganti semua URL repositori di README agar konsisten dengan hasil perintah di atas.

### 3d. Perbarui section "Language Bindings"

Tambahkan catatan jujur di setiap binding:
- Python: tambahkan `*(Stub — API terbatas)*`
- TypeScript: tambahkan `*(Reader belum diimplementasikan)*`
- Go: tambahkan `*(Broken — bug include path, lihat STATUS.md)*`
- Java: tambahkan `*(Stub — belum teruji end-to-end)*`

---

## BAGIAN 4: HAPUS ATAU ARSIPKAN FILE DOKUMENTASI YANG REDUNDAN

### 4a. File yang perlu diarsipkan

Buat direktori `docs/archive/` dan pindahkan file-file berikut ke sana:

```bash
mkdir -p docs/archive

# File yang berisi klaim status lama yang kini digantikan STATUS.md
mv SDK_STATUS.md docs/archive/SDK_STATUS_v1.md
mv IMPLEMENTATION_STATUS.md docs/archive/IMPLEMENTATION_STATUS_v1.md
mv Phase.md docs/archive/Phase_v1.md
```

Tambahkan header di setiap file yang diarsipkan:

```markdown
> ⚠️ **DOKUMEN ARSIP** — File ini tidak lagi dipertahankan.
> Status proyek saat ini ada di [STATUS.md](../../STATUS.md).
> File ini disimpan untuk referensi historis.
```

### 4b. File yang perlu diperbarui (bukan dihapus)

**`CHANGELOG.md`** — Perbaiki inkonsistensi status enkripsi:
- Di section `[Future] v1.2.0`, enkripsi masih `[ ]`. Ini benar — biarkan.
- Tambahkan entry `[Unreleased]` yang mencatat bahwa modul enkripsi dan ECC sudah ada sebagai unit-tested modules, belum terintegrasi ke pipeline.

**`ROADMAP.md`** — Pastikan konsisten dengan STATUS.md:
- Periksa item yang sudah selesai (`[x]`) vs belum (`[ ]`)
- Sinkronkan dengan kondisi aktual dari audit Bagian 1

---

## BAGIAN 5: PERBAIKI `Cargo.toml` WORKSPACE

### 5a. Tambahkan qrd-wasm ke workspace

Buka `Cargo.toml` di root, temukan section `[workspace]`, dan tambahkan:

```toml
members = [
    "core/qrd-core",
    "core/qrd-ffi",
    "core/qrd-wasm",    # ← Tambahkan ini
]
```

Verifikasi setelah perubahan:
```bash
cargo build --workspace 2>&1 | head -30
```

### 5b. Sinkronkan URL repositori

Di `Cargo.toml`, perbarui:
```toml
[workspace.package]
repository = "[URL YANG BENAR DARI git remote get-url origin]"
```

Pastikan URL ini sama persis dengan yang digunakan di README.md.

### 5c. Bersihkan dependensi yang tidak digunakan

Periksa apakah dependensi berikut benar-benar digunakan:
```bash
# Cek penggunaan tracing
grep -rn "tracing::\|#\[instrument\]" core/qrd-core/src/ | wc -l

# Cek penggunaan rand (bukan dari aes-gcm)
grep -rn "use rand::" core/qrd-core/src/ | wc -l
```

Jika tidak ditemukan, hapus dari `[workspace.dependencies]` dan tambahkan komentar `# Dihapus: tidak digunakan`.

---

## BAGIAN 6: PERBAIKI SPESIFIKASI BINER

**File:** `specs/binary-layout.md`

Temukan baris ini:
```
28      4     U32BE   FLAGS                Reserved
```

Ganti dengan:
```
28      4     U32LE   FLAGS                Reserved (harus 0x00000000)
```

Tambahkan catatan di bawahnya:
```markdown
> **Catatan implementasi:** Field FLAGS saat ini selalu 0 (reserved).
> Meski namanya "FLAGS", field ini ditulis little-endian konsisten dengan
> seluruh format. Spesifikasi sebelumnya keliru mencantumkan U32BE.
> Diperbaiki: [TANGGAL HARI INI].
```

---

## BAGIAN 7: PERBAIKI BUG GO BINDING

**File:** `sdk/go/qrd.go`

Temukan:
```go
/*
#cgo LDFLAGS: -L../../target/release -lqrd_ffi
#include "../../core/qrd-ffi/src/lib.rs"
*/
```

Ganti dengan:
```go
/*
#cgo LDFLAGS: -L../../target/release -lqrd_ffi
#include "qrd.h"
*/
```

File `qrd.h` sudah ada di `sdk/go/qrd.h`. Verifikasi:
```bash
ls sdk/go/qrd.h  # harus ada
```

Setelah fix, tambahkan catatan di `sdk/go/README.md`:
```markdown
## Build Requirements

1. Build Rust FFI library terlebih dahulu:
   ```bash
   cargo build --release -p qrd-ffi
   ```
2. Pastikan `libqrd_ffi.so` (Linux) atau `libqrd_ffi.dylib` (macOS) ada di `../../target/release/`
3. Build Go:
   ```bash
   go build ./...
   go test ./...
   ```
```

---

## BAGIAN 8: TAMBAHKAN `STRUCT` / `ENUM` yang Hilang di Go Binding

**File:** `sdk/go/qrd.go`

Temukan definisi `FieldType` constants, lalu tambahkan dua yang hilang:

```go
const (
    FieldTypeBoolean   FieldType = iota  // 0
    FieldTypeInt8                         // 1
    FieldTypeInt16                        // 2
    FieldTypeInt32                        // 3
    FieldTypeInt64                        // 4
    FieldTypeUint8                        // 5
    FieldTypeUint16                       // 6
    FieldTypeUint32                       // 7
    FieldTypeUint64                       // 8
    FieldTypeFloat32                      // 9
    FieldTypeFloat64                      // 10
    FieldTypeTimestamp                    // 11
    FieldTypeDate                         // 12
    FieldTypeTime                         // 13
    FieldTypeDuration                     // 14 ← TAMBAHKAN
    FieldTypeString                       // 15
    FieldTypeEnum                         // 16 ← TAMBAHKAN
    FieldTypeUuid                         // 17
    FieldTypeBlob                         // 18
    FieldTypeDecimal                      // 19
)
```

Pastikan urutan numerik konsisten dengan `ffi/src/lib.rs` fungsi `field_type_from_ffi()`.

---

## BAGIAN 9: PERBAIKI INKONSISTENSI JAVA (JNI vs JNA)

**File:** `README.md`

Cari semua penyebutan "JNI" yang merujuk pada Java binding dan ganti dengan "JNA":

```bash
# Cek dulu
grep -n "JNI\|JNA" README.md

# Ganti secara selektif (hanya yang merujuk Java SDK, bukan penjelasan umum)
# Baris yang biasanya bermasalah:
# "Java SDK (JNI + Stream API)" → "Java SDK (JNA + Stream API)"
```

Tambahkan penjelasan singkat di `sdk/java/README.md`:
```markdown
## Tentang Implementasi

Java binding menggunakan **JNA (Java Native Access)** — bukan JNI.
JNA dipilih karena tidak memerlukan kode glue native di sisi Java,
sehingga lebih mudah di-maintain. Trade-off: overhead sedikit lebih
tinggi dibanding JNI murni, tapi tidak signifikan untuk use-case ini.
```

---

## BAGIAN 10: PERBARUI `CONTRIBUTING.md` / BUAT JIKA BELUM ADA

Buat atau perbarui `CONTRIBUTING.md` di root repositori:

```markdown
# Contributing to QRD-SDK

## Sebelum Mulai

Baca [STATUS.md](./STATUS.md) untuk memahami kondisi proyek saat ini.
Banyak bagian masih dalam pengembangan aktif.

## Known Issues yang Butuh Kontribusi

Lihat [AUDIT.md](./AUDIT.md) section "Temuan Kritis & Rekomendasi".
Issues dengan label 🔴 adalah prioritas tertinggi.

## Aturan Dokumentasi

**Satu sumber kebenaran:** STATUS.md adalah satu-satunya file yang
mencatat status komponen. Jangan update status di file lain.

Saat kamu menyelesaikan implementasi sebuah komponen:
1. Update STATUS.md
2. Jangan update README.md, SDK_STATUS.md, atau file lain untuk status
3. Buat PR dengan judul: `[status] Komponen X selesai diimplementasikan`

## Format Commit

```
type(scope): deskripsi singkat

Tipe: feat, fix, docs, test, refactor, chore
Scope: core, ffi, go, python, ts, java, spec, docs

Contoh:
feat(core): integrasikan enkripsi ke writer pipeline
fix(go): perbaiki CGO include path dari .rs ke .h
docs(status): update STATUS.md setelah perbaikan Go binding
```

## Test Requirements

Setiap PR harus:
- [ ] Semua test existing tetap lulus: `cargo test --all`
- [ ] Perubahan format biner: wajib sertakan golden test vector baru
- [ ] Fitur baru: wajib ada unit test
- [ ] Perubahan enkripsi/ECC: wajib ada security test
```

---

## BAGIAN 11: VERIFIKASI AKHIR

Setelah semua perubahan dilakukan, jalankan checklist ini:

```bash
# 1. Workspace build bersih
cargo build --workspace
echo "Exit code: $?"

# 2. Semua test lulus
cargo test --all 2>&1 | tail -5

# 3. Tidak ada URL inkonsisten
grep -rn "github.com" . --include="*.md" --include="*.toml" \
  | grep -v ".git" | grep -v "docs/archive"

# 4. STATUS.md ada dan berisi tanggal hari ini
head -5 STATUS.md

# 5. Go include path sudah benar
grep "include" sdk/go/qrd.go

# 6. qrd-wasm terdaftar di workspace
grep "qrd-wasm" Cargo.toml

# 7. FLAGS field di spec sudah U32LE
grep "FLAGS" specs/binary-layout.md
```

**Semua checklist harus hijau sebelum commit.**

---

## BAGIAN 12: COMMIT STRATEGY

Lakukan commit secara terpisah per bagian — jangan satu commit besar:

```bash
git add STATUS.md
git commit -m "docs: tambah STATUS.md sebagai satu sumber kebenaran status proyek"

git add README.md
git commit -m "docs(readme): update status badge dan klaim binding ke kondisi aktual"

git add docs/archive/
git commit -m "docs: arsipkan SDK_STATUS, IMPLEMENTATION_STATUS, Phase ke docs/archive"

git add Cargo.toml
git commit -m "chore(workspace): tambah qrd-wasm ke members, sinkronkan URL repo"

git add specs/binary-layout.md
git commit -m "spec: perbaiki endianness FLAGS field dari U32BE ke U32LE"

git add sdk/go/qrd.go
git commit -m "fix(go): perbaiki CGO include path dari lib.rs ke qrd.h"

git add sdk/go/qrd.go
git commit -m "fix(go): tambah FieldTypeDuration dan FieldTypeEnum yang hilang"

git add CHANGELOG.md ROADMAP.md
git commit -m "docs: sinkronkan CHANGELOG dan ROADMAP dengan STATUS.md"

git add CONTRIBUTING.md
git commit -m "docs: tambah CONTRIBUTING.md dengan aturan dokumentasi satu sumber kebenaran"
```

---

## OUTPUT YANG DIHARAPKAN

Setelah prompt ini dieksekusi, repositori harus memiliki:

```
QRD-SDK/
├── STATUS.md              ← BARU: satu sumber kebenaran
├── AUDIT.md               ← Ada (dari audit sebelumnya)
├── README.md              ← Diperbarui: klaim akurat, URL konsisten
├── CONTRIBUTING.md        ← BARU atau diperbarui
├── CHANGELOG.md           ← Diperbarui: konsisten dengan STATUS.md
├── ROADMAP.md             ← Diperbarui: konsisten dengan STATUS.md
├── Cargo.toml             ← Diperbarui: qrd-wasm di workspace, URL konsisten
├── specs/
│   └── binary-layout.md   ← Diperbarui: FLAGS field U32LE
├── sdk/
│   └── go/
│       └── qrd.go         ← Diperbaiki: include path + FieldType lengkap
└── docs/
    └── archive/
        ├── SDK_STATUS_v1.md
        ├── IMPLEMENTATION_STATUS_v1.md
        └── Phase_v1.md
```

**Tidak ada lagi:**
- Empat file yang mengklaim status phase berbeda-beda
- Klaim "production-ready" yang tidak akurat
- Go binding yang tidak bisa di-build
- Endianness inconsistency di spesifikasi
- URL repositori yang berbeda-beda
