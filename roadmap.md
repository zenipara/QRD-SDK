# QRD-SDK Roadmap

Dokumen ini merangkum lima fase pengembangan QRD-SDK berdasarkan hasil audit v4.
Roadmap ini dibuat untuk mencerminkan status aktual fitur, integrasi keamanan, dan kebutuhan dokumentasi.

## Cara Menggunakan Roadmap
1. Baca status fase untuk memahami prioritas saat ini.
2. Ikuti checklist implementasi yang tercantum di setiap fase.
3. Gunakan `validate_*.sh` dan `quick_validate.sh` untuk memverifikasi target fase.
4. Perbarui dokumen ini saat milestone selesai agar roadmap tetap satu sumber kebenaran.

## Phase 1: Core Engine
Status: ✅ COMPLETE

Tujuan utama:
- Desain engine inti format QRD.
- Implementasi schema engine dengan 20 tipe logis.
- Integrasi 8 algoritma encoding.
- Implementasi kompresi ZSTD dan LZ4.
- Writer/reader dasar untuk file dan streaming.
- Footer parser/builder dengan CRC32.
- Infrastruktur I/O dan transposisi baris-ke-kolom.

Checklist:
- [x] Schema engine stabil
- [x] Encoding dan decoding bekerja untuk semua tipe logis
- [x] Compression pipeline terintegrasi
- [x] Footer builder dan parser selesai
- [x] Streaming writer/reader dasar selesai

Catatan audit v4:
- Core engine telah stabil dan menjadi fondasi yang valid untuk fase selanjutnya.

## Phase 2: Security & SIMD
Status: ✅ COMPLETE

Tujuan utama:
- Integrasi AES-256-GCM end-to-end di writer dan reader.
- Derivasi kunci dengan HKDF dan Argon2id untuk fitur password.
- Implementasi Reed-Solomon ECC dengan konfigurasi parity 1–32.
- Optimasi SIMD untuk memcpy, XOR, delta, dan operasi byte.
- Validasi checksum footer dan deteksi korupsi dasar.

Checklist:
- [x] End-to-end encryption pipeline aktif
- [x] Per-column key derivation tersedia
- [x] ECC encode/decode terintegrasi
- [x] SIMD optimasi hadir di utilitas core
- [x] Footer validation dengan CRC32 ada
- [ ] Per-column encryption diperluas ke seluruh kolom
- [ ] Unsafe SIMD transmute diganti dengan `bytemuck`

Catatan audit v4:
- Enkripsi dan ECC kini terintegrasi penuh di pipeline.
- Per-column encryption perlu penyempurnaan karena dukungan saat ini masih parsial.
- Unsafe SIMD transmute masih tersisa dan bisa diganti dengan `bytemuck`.

## Phase 3: Language Bindings
Status: ⚠️ IN PROGRESS

Tujuan utama:
- Python binding dengan PyO3.
- TypeScript/WASM binding untuk browser dan Node.
- Go binding via CGO dengan header `qrd.h`.
- Java binding melalui JNI atau JNA.
- FFI thread-safety dengan `Arc<Mutex<>>`.
- Konsistensi lintas bahasa untuk read/write roundtrip.

Checklist:
- [x] Python binding (`sdk/python`) tersedia
- [x] TypeScript/WASM binding (`sdk/typescript`) tersedia
- [x] Go binding (`sdk/go`) tersedia
- [x] Java binding (`sdk/java`) tersedia
- [x] FFI layer thread-safe
- [ ] Lingkup read/write roundtrip lintas bahasa diverifikasi
- [ ] Contoh lintas bahasa disiapkan di `examples/`
- [ ] Dokumentasi binding dibersihkan dari klaim status yang tidak konsisten

Catatan audit v4:
- Bahasa binding sudah tersedia, namun status produksi belum final.
- `SDK_USAGE.md` dan `validate_phase4.sh` perlu diselaraskan agar klaim status tidak menyesatkan.

## Phase 4: Ecosystem & CLI
Status: ⚠️ IN PROGRESS

Tujuan utama:
- Bangun CLI dan tool interoperabilitas yang dapat dipakai.
- Sediakan contoh penggunaan lintas bahasa dan repositori integrasi.
- Perbaiki dokumentasi ekosistem: `ECOSYSTEM_TOOLS.md`, `docs/ARCHITECTURE.md`, `docs/QUICKSTART.md`.
- Selaraskan roadmap, README, CHANGELOG, dan `SDK_USAGE.md`.
- Validasi end-to-end dengan skrip fase dan test vectors.
- Konsolidasikan metadata repositori seperti URL dan lisensi.

Checklist:
- [x] `tools/` diindikasikan sebagai bagian fase 4
- [ ] `ECOSYSTEM_TOOLS.md` mencerminkan fase implementasi aktual
- [ ] `docs/QUICKSTART.md` merujuk ke status fase terbaru
- [ ] `README.md` path roadmap diperbarui agar konsisten
- [ ] `SDK_USAGE.md` diperbarui menjadi "Phase 4: In Progress" atau lebih spesifik
- [ ] `validate_phase4.sh` diperbarui agar jumlah tes dinamis dan status akurat
- [ ] URL repositori diseragamkan di semua dokumen
- [ ] Contoh CLI dan tool cross-language tersedia
- [ ] `PRODUCTION_GUIDE.md` dikoreksi jika masih menyatakan hanya Phase 2

Catatan audit v4:
- Roadmap di README masih menandai enkripsi/ECC sebagai belum selesai padahal sudah diimplementasikan.
- Dokumentasi status masih terpecah antara README dan `SDK_USAGE.md`.

## Phase 5: Production Readiness & Audit Closure
Status: 🔜 NEXT

Tujuan utama:
- Tutup semua temuan audit dan verifikasi kualitas produksi.
- Tambahkan validasi korupsi yang lebih kuat: per-row-group checksum dan per-layer ECC deteksi.
- Kurangi penggunaan unsafe pada level crate.
- Sesuaikan skrip validasi dan klaim jumlah tes dengan output real.
- Finalisasi golden vector dan regression tests.
- Perbarui release notes, CHANGELOG, dan semua dokumen status.

Checklist:
- [ ] `validate_phase4.sh` dan `validate_core_stable.sh` menyajikan status akurat
- [ ] `SDK_USAGE.md` diubah menjadi status "In Progress" untuk Phase 4 dan Phase 5
- [ ] `CHANGELOG.md` dipindahkan item enkripsi/ECC ke rilis yang sudah ada atau Unreleased
- [ ] `README.md` diselaraskan dengan roadmap baru dan status fase
- [ ] Keseluruhan test suite minimal 238 test diverifikasi atau tingkatan yang relevan
- [ ] Audit hardening dengan fokus pada `unsafe` dan checksum row-group
- [ ] Final release candidate didefinisikan untuk dokumentasi publik

Catatan audit v4:
- Audit menunjukkan kebutuhan fase akhir untuk memastikan stabilitas nyata sebelum klaim produksi.
- Fase ini fokus pada hardening, konsistensi status, dan pelaporan yang akurat.

## Ringkasan Status
- Phase 1: COMPLETE
- Phase 2: COMPLETE
- Phase 3: IN PROGRESS
- Phase 4: IN PROGRESS
- Phase 5: NEXT

## Tujuan Utama Roadmap
1. Menjadikan QRD-SDK sebagai engine inti yang stabil dan benar.
2. Menyelesaikan integrasi keamanan dan ECC secara end-to-end.
3. Menghadirkan binding bahasa lintas platform yang dapat diandalkan.
4. Menyediakan ekosistem dan tooling yang konsisten dan terdokumentasi.
5. Menutup cycle audit dengan validasi produksi dan dokumentasi yang jujur.
