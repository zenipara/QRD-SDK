# Contributing to QRD-SDK

Terima kasih atas keinginan Anda untuk berkontribusi pada QRD-SDK! Panduan ini menjelaskan cara berkontribusi secara efektif pada proyek ini.

## Sebelum Mulai

Baca [STATUS.md](./STATUS.md) untuk memahami kondisi proyek saat ini. Banyak bagian masih dalam pengembangan aktif, dan beberapa fitur masih parsial.

## Known Issues yang Butuh Kontribusi

Lihat [AUDIT.md](./AUDIT.md) untuk daftar lengkap temuan. Issues dengan label 🔴 adalah prioritas tertinggi:

- 🔴 **Integrasikan enkripsi ke writer/reader pipeline** — unit tests sudah ada, belum di-hook ke I/O
- 🔴 **Integrasikan ECC ke writer/reader pipeline** — sama seperti enkripsi
- 🔴 **Perbaiki Go CGO include path** — sudah diperbaiki (lihat commit)
- 🔴 **Daftarkan qrd-wasm ke workspace** — sudah dilakukan
- 🔴 **Perbaiki FLAGS field di spesifikasi** — sudah diperbaiki (U32BE → U32LE)

## Aturan Dokumentasi — Satu Sumber Kebenaran

**STATUS.md adalah satu-satunya file yang mencatat status komponen. Jangan update status di file lain.**

Saat kamu menyelesaikan implementasi sebuah komponen:

1. **Update STATUS.md** dengan status terbaru
2. **Jangan** update README.md, SDK_STATUS.md, IMPLEMENTATION_STATUS.md untuk status komponen
3. Buat PR dengan judul: `[status] Komponen X selesai diimplementasikan`

Contoh:

```markdown
## Perubahan pada STATUS.md

| Modul | Sebelum | Sesudah | Catatan |
|---|---|---|---|
| Enkripsi di pipeline | ⚠️ Parsial | ✅ Lengkap | Sekarang terintegrasi ke writer/reader |
```

## Aturan Kode

### Format & Linting

```bash
# Sebelum commit, pastikan:
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all
```

Semua command harus pass sebelum push.

### Type System

- ✅ Gunakan `Result<T, E>` untuk operasi yang bisa gagal
- ✅ Pesan error yang jelas dan actionable
- ❌ Jangan gunakan `.unwrap()` di kode produksi
- ❌ Jangan `panic!` di perpustakaan publik

### Testing Requirements

Setiap PR harus:

- [ ] **Semua test existing tetap lulus:** `cargo test --all`
- [ ] **Perubahan format biner:** wajib sertakan golden test vector baru
- [ ] **Fitur baru:** wajib ada unit test
- [ ] **Perubahan enkripsi/ECC:** wajib ada security test

Jalankan test baru dengan:

```bash
cargo test --all --release 2>&1 | tail -20
```

### Dokumentasi Inline

Setiap fungsi publik harus punya `///` documentation:

```rust
/// Encode data dengan algoritma yang dipilih secara otomatis.
///
/// # Arguments
/// * `data` - Data mentah untuk di-encode
///
/// # Returns
/// Encoded data atau error jika encoding gagal.
///
/// # Panics
/// Tidak panic — selalu mengembalikan Result.
pub fn encode(data: &[u8]) -> Result<Vec<u8>> { }
```

## Format Commit

```
type(scope): deskripsi singkat

[optional body]

type: feat, fix, docs, test, refactor, chore
scope: core, ffi, go, python, ts, java, spec, docs

Contoh:
feat(core): integrasikan enkripsi ke writer pipeline
fix(go): perbaiki CGO include path dari .rs ke .h
docs(status): update STATUS.md setelah perbaikan Go binding
test(core): tambah security test untuk enkripsi
```

## Submission Workflow

1. **Fork** repositori
2. **Buat branch** dari `main`:
   ```bash
   git checkout -b fix/your-feature
   ```
3. **Commit** dengan format di atas
4. **Push** ke fork Anda
5. **Buat Pull Request** ke `main`
6. **Tunggu review** dan address feedback

## Code Review Checklist

Reviewer akan melihat:

- ✅ Format code dengan `cargo fmt`
- ✅ Semua test passing dengan `cargo test --all`
- ✅ Tidak ada `unwrap()` di produksi
- ✅ Pesan commit yang jelas
- ✅ STATUS.md di-update jika ada perubahan status
- ✅ Documentation inline lengkap
- ✅ Thread safety (use of `Sync`/`Send` jika perlu)

## Security Considerations

Jika Anda memodifikasi:

- **Encryption module** — pastikan tidak ada side-channel leaks, gunakan `subtle` crate untuk comparison
- **ECC module** — test recovery dari berbagai pola korupsi
- **Validation** — jangan skip CRC32 checks
- **SIMD** — fallback scalar implementations harus identical

## Performance Guidelines

Setiap perubahan kritis harus diverifikasi dengan benchmark:

```bash
cargo bench --package qrd-core
```

Target throughput:

| Operation | Target |
|---|---|
| Write | 1–5 GB/s |
| Read full | 2–10 GB/s |
| Read partial | 5–20 GB/s |
| Compression | 500 MB–2 GB/s |

Jika PR berdampak pada encoding atau compression, sertakan hasil benchmark dalam deskripsi PR.

## Questions?

- **Status proyek:** Lihat [STATUS.md](./STATUS.md)
- **Spesifikasi format:** Lihat [SPECIFICATION.md](./SPECIFICATION.md)
- **Architecture:** Lihat [docs/ARCHITECTURE.md](./docs/ARCHITECTURE.md)
- **Issues/Bugs:** Buka GitHub Issue dengan label yang tepat

## License

Semua kontribusi harus under MIT License. Dengan submit PR, Anda agree untuk license kode Anda under MIT.

---

**Terima kasih atas kontribusi Anda untuk membuat QRD-SDK lebih baik!** 🚀
