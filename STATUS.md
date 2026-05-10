# QRD-SDK — Status Proyek

**Terakhir diperbarui:** 9 Mei 2026  
**Versi:** 0.1.0  
**Format QRD:** v1.0.0-draft

---

## Status Keseluruhan

> **ALPHA** — Core Rust engine fungsional dengan test komprehensif. Enkripsi & ECC unit-tested tapi belum terintegrasi ke pipeline I/O. Language bindings mayoritas masih dalam bentuk stub atau memiliki bug kritis.

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
| Footer Parser | ✅ Lengkap | - |
| SIMD Utilities | ✅ Lengkap | Fallback scalar tersedia |
| Validasi (CRC32) | ✅ Lengkap | - |

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
| `core/qrd-core/src/` (unit tests) | 177 |
| `core/qrd-core/tests/` (integration tests) | 96 |
| **Total** | **273** |

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
