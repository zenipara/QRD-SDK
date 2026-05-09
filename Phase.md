Phase ini memperbaiki tiga masalah performa dan correctness:

1. **SIMD palsu** — `SimdOps` klaim vectorized tapi semua scalar
2. **StreamingWriter bug laten** — `W: Write` tanpa `Seek` tapi coba patch header
3. **Footer statistics placeholder** — statistik dikumpulkan tapi tidak ditulis ke binary

---

## TASK 1: Real SIMD Implementation

### Background

Di `utils/simd.rs`, semua fungsi adalah scalar dengan comment "placeholder":
```rust
pub fn memcpy(&self, dst: &mut [u8], src: &[u8]) -> Result<()> {
    dst.copy_from_slice(src);  // ← bukan SIMD
}
pub fn xor(&self, dst: &mut [u8], src: &[u8]) -> Result<()> {
    for i in 0..dst.len() { dst[i] ^= src[i]; }  // ← bukan SIMD
}
```

### Tambahkan ke `Cargo.toml` workspace dependencies:
```toml
# SIMD portable (stable Rust, no nightly needed)
wide = "0.7"
```

### Implementasi baru `utils/simd.rs`:

**Strategy:** Gunakan `wide` crate untuk portable SIMD yang works di stable Rust.
Tetap pertahankan scalar fallback untuk platform tanpa SIMD.

```rust
use wide::{u8x16, u8x32, i32x8};

impl SimdOps {
    /// SIMD-accelerated XOR — gunakan u8x32 (AVX2) jika tersedia, fallback u8x16 (SSE2), lalu scalar
    pub fn xor(&self, dst: &mut [u8], src: &[u8]) -> Result<()> {
        if dst.len() != src.len() {
            return Err(Error::InvalidInput("Length mismatch".to_string()));
        }
        
        if !self.enabled {
            // Scalar fallback
            dst.iter_mut().zip(src.iter()).for_each(|(d, s)| *d ^= s);
            return Ok(());
        }
        
        let len = dst.len();
        let chunks_32 = len / 32;
        let chunks_16 = (len % 32) / 16;
        let remainder = len % 16;
        
        // Process 32 bytes at a time (AVX2 width)
        for i in 0..chunks_32 {
            let offset = i * 32;
            let a = u8x32::from(&dst[offset..offset+32]);
            let b = u8x32::from(&src[offset..offset+32]);
            let result = a ^ b;
            dst[offset..offset+32].copy_from_slice(result.as_array_ref());
        }
        
        // Process 16 bytes at a time (SSE2 width)
        let base = chunks_32 * 32;
        for i in 0..chunks_16 {
            let offset = base + i * 16;
            let a = u8x16::from(&dst[offset..offset+16]);
            let b = u8x16::from(&src[offset..offset+16]);
            let result = a ^ b;
            dst[offset..offset+16].copy_from_slice(result.as_array_ref());
        }
        
        // Scalar tail
        let tail_start = base + chunks_16 * 16;
        dst[tail_start..].iter_mut().zip(src[tail_start..].iter()).for_each(|(d, s)| *d ^= s);
        
        Ok(())
    }
    
    /// SIMD-accelerated delta encoding untuk i32
    pub fn delta_encode_i32(&self, data: &[i32]) -> Result<Vec<i32>> {
        if data.is_empty() { return Ok(Vec::new()); }
        
        let mut result = Vec::with_capacity(data.len());
        result.push(data[0]);
        
        if !self.enabled || data.len() < 8 {
            // Scalar
            for i in 1..data.len() {
                result.push(data[i].wrapping_sub(data[i-1]));
            }
            return Ok(result);
        }
        
        // SIMD: proses 8 i32 sekaligus
        let chunks = (data.len() - 1) / 8;
        for chunk in 0..chunks {
            let base = chunk * 8 + 1;
            let curr = i32x8::from(&data[base..base+8]);
            let prev = i32x8::from(&data[base-1..base+7]);
            let deltas = curr - prev;
            result.extend_from_slice(deltas.as_array_ref());
        }
        
        // Scalar tail
        let processed = chunks * 8 + 1;
        for i in processed..data.len() {
            result.push(data[i].wrapping_sub(data[i-1]));
        }
        
        Ok(result)
    }
    
    /// SIMD-accelerated delta decoding untuk i32
    pub fn delta_decode_i32(&self, deltas: &[i32]) -> Result<Vec<i32>> {
        if deltas.is_empty() { return Ok(Vec::new()); }
        
        let mut result = Vec::with_capacity(deltas.len());
        result.push(deltas[0]);
        
        // NOTE: Delta decoding sequential by nature (prefix sum),
        // true SIMD prefix sum is complex. Use scalar with LLVM auto-vectorization hint.
        for i in 1..deltas.len() {
            result.push(result[i-1].wrapping_add(deltas[i]));
        }
        Ok(result)
    }
    
    /// SIMD byte counting
    pub fn count_bytes(&self, data: &[u8], target: u8) -> usize {
        if !self.enabled {
            return data.iter().filter(|&&b| b == target).count();
        }
        
        let target_vec = u8x16::splat(target);
        let mut count = 0usize;
        let chunks = data.len() / 16;
        
        for i in 0..chunks {
            let chunk = u8x16::from(&data[i*16..(i+1)*16]);
            let eq = chunk.cmp_eq(target_vec);
            // Count non-zero lanes
            count += eq.as_array_ref().iter().filter(|&&b| b != 0).count();
        }
        
        // Scalar tail
        count += data[chunks*16..].iter().filter(|&&b| b == target).count();
        count
    }
}
```

### Update `detect_simd_support()`:
```rust
fn detect_simd_support() -> (bool, SimdInstructionSet) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx2") {
            return (true, SimdInstructionSet::Avx2);
        }
        if is_x86_feature_detected!("sse4.1") {
            return (true, SimdInstructionSet::Sse4);
        }
    }
    #[cfg(target_arch = "aarch64")]
    {
        // NEON always available on AArch64
        return (true, SimdInstructionSet::Neon);
    }
    (false, SimdInstructionSet::None)
}
```

### Benchmark SIMD vs Scalar

Tambahkan ke `benches/comprehensive_bench.rs`:
```rust
fn bench_simd_xor(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd_xor");
    
    for size in [1_000, 10_000, 100_000, 1_000_000].iter() {
        let mut dst = vec![0u8; *size];
        let src = vec![0xFFu8; *size];
        
        group.bench_with_input(BenchmarkId::new("simd", size), size, |b, _| {
            let ops = SimdOps::new();
            b.iter(|| ops.xor(&mut dst, &src))
        });
        
        group.bench_with_input(BenchmarkId::new("scalar", size), size, |b, _| {
            b.iter(|| dst.iter_mut().zip(src.iter()).for_each(|(d, s)| *d ^= s))
        });
    }
}
```

---

## TASK 2: Fix StreamingWriter — Header Patch Issue

### Masalah
`StreamingWriter<W: Write>` tidak bisa patch row count di header karena `Write` tidak punya `Seek`.

```rust
// Saat ini di streaming_writer.rs — INI AKAN FAIL untuk non-seekable streams:
pub fn finish(mut self) -> Result<()> {
    // ...
    // BUG: tidak ada cara untuk update row count di header untuk Write-only streams!
}
```

### Solusi: Dua-pass approach + deferred header

**Opsi A (Recommended):** Tulis row count di footer saja, bukan di header. Header pakai sentinel `u32::MAX` untuk "row count di footer":

```rust
fn write_header(writer: &mut W, schema: &Schema) -> Result<()> {
    // ...
    // Tulis sentinel untuk row count — akan dibaca dari footer
    writer.write_u32::<LittleEndian>(u32::MAX)?;  // Sentinel: "baca dari footer"
    // ...
}

pub fn finish(mut self) -> Result<()> {
    self.flush_current_row_group()?;
    
    // Row count ada di footer — tidak perlu patch header
    let footer = Footer::with_metadata_index(
        self.schema.clone(),
        self.total_rows,  // ← ada di footer
        metadata_index,
    );
    
    let footer_bytes = footer.serialize()?;
    self.writer.write_all(&footer_bytes)?;
    self.writer.write_u32::<LittleEndian>(footer_bytes.len() as u32)?;
    
    // Flush writer buffer
    if let Some(ref mut w) = self.writer.as_mut() {
        w.flush()?;
    }
    
    Ok(())
}
```

**Update `FileReader` untuk handle sentinel:**
```rust
// Di reader/mod.rs saat parse header:
let row_count_header = u32::from_le_bytes([header[16], header[17], header[18], header[19]]);
let row_count = if row_count_header == u32::MAX {
    // Baca dari footer (sudah di-parse di bawah)
    footer.row_count
} else {
    row_count_header
};
```

**Opsi B:** Split trait bound:
```rust
pub struct StreamingWriter<W: Write> { ... }       // untuk streams
pub struct SeekableWriter<W: Write + Seek> { ... } // untuk files dengan header patch
```

**Implementasikan Opsi A**, lalu:

### Tambahkan test StreamingWriter:
```rust
#[test]
fn test_streaming_writer_to_vec() {
    // Tulis ke Vec<u8> (tidak seekable)
    let mut buf = Vec::new();
    let mut writer = StreamingWriter::new(Cursor::new(&mut buf), schema)?;
    for i in 0..1000 {
        writer.write_row(make_row(i))?;
    }
    writer.finish()?;
    
    // Baca kembali dari buf
    let reader = FileReader::from_bytes(buf)?;
    assert_eq!(reader.row_count(), 1000);
}

#[test]
fn test_streaming_writer_bounded_memory() {
    // Verifikasi memory tidak naik proporsional dengan data size
    // Gunakan 1M rows tapi row_group_size = 1000
    // Peak memory seharusnya < 10MB (bukan proportional ke 1M rows)
}

#[test]
fn test_streaming_writer_multiple_row_groups() {
    // row_group_size = 100, tulis 350 rows
    // Harus ada 4 row groups (100, 100, 100, 50)
}
```

---

## TASK 3: Wire Footer Statistics ke Binary

### Masalah
Di `rowgroup/mod.rs`, null_count dan distinct_count ditulis sebagai hardcoded 0:

```rust
result.write_u32::<LittleEndian>(0)?; // null_count (placeholder)
result.write_u32::<LittleEndian>(0)?; // distinct_count (placeholder)
```

Padahal `RowGroupStats` sudah dikumpulkan di writer. Tinggal di-wire.

### Implementasi

**Step 1:** Update `RowGroup` struct untuk terima statistics:
```rust
pub struct RowGroup {
    pub row_count: u32,
    pub columns: Vec<ColumnChunk>,
    pub column_stats: Option<Vec<ColumnChunkStats>>,  // ← TAMBAHKAN
}

pub struct ColumnChunkStats {
    pub null_count: u32,
    pub distinct_count: u32,
    pub min_value: Option<Vec<u8>>,
    pub max_value: Option<Vec<u8>>,
}
```

**Step 2:** Update serialization di `rowgroup/mod.rs` untuk pakai stats actual:
```rust
fn serialize_column_header(column: &ColumnChunk, stats: Option<&ColumnChunkStats>) -> Result<Vec<u8>> {
    let null_count = stats.map(|s| s.null_count).unwrap_or(0);
    let distinct_count = stats.map(|s| s.distinct_count).unwrap_or(0);
    result.write_u32::<LittleEndian>(null_count)?;
    result.write_u32::<LittleEndian>(distinct_count)?;
    // Tulis min/max jika ada
}
```

**Step 3:** Di `writer/mod.rs`, pass `RowGroupStats` ke `RowGroup::process_column()`:
```rust
// flush_row_group():
let rg_stats = self.current_row_group_stats.column_stats();
for (col_idx, column) in columns.iter().enumerate() {
    let col_stats = rg_stats.get(col_idx);
    row_group.process_column_with_stats(column, encoding, codec, level, col_stats)?;
}
```

### Test statistik:
```rust
#[test]
fn test_column_statistics_null_count() {
    // Tulis 10 rows, 3 di antaranya null di kolom Optional
    // Baca footer → null_count harus 3
}

#[test]
fn test_column_statistics_min_max() {
    // Tulis kolom Int64 dengan nilai 1-100
    // Footer stats harus: min=1, max=100
}
```

---

## VALIDASI PHASE 3

```bash
# Build dengan fitur SIMD
cargo build --package qrd-core --release 2>&1

# SIMD tests
cargo test --package qrd-core simd 2>&1

# Benchmark SIMD (lihat apakah ada speedup)
cargo bench --package qrd-core simd_xor 2>&1

# Streaming writer tests
cargo test --package qrd-core streaming 2>&1

# Statistics tests
cargo test --package qrd-core statistics 2>&1

# Full test suite
cargo test --package qrd-core 2>&1 | tail -30
```

**Expected:**
```
test utils::simd::tests::test_simd_xor_correctness ... ok
test utils::simd::tests::test_simd_delta_encode ... ok
test writer::streaming_writer::tests::test_streaming_writer_to_vec ... ok
test writer::streaming_writer::tests::test_streaming_writer_multiple_row_groups ... ok
```

**Benchmark output harus menunjukkan SIMD > scalar untuk data >= 10KB:**
```
simd_xor/simd/100000   time: [XX µs]
simd_xor/scalar/100000 time: [YY µs]
# SIMD should be 2-5x faster for large inputs
```

---

## CONSTRAINT

- `wide` crate versi 0.7+ untuk stable Rust SIMD
- Scalar fallback WAJIB ada untuk platform tanpa SIMD (WASM, old x86)
- SIMD code tidak boleh `unsafe` — `wide` crate adalah safe abstraction
- StreamingWriter fix harus backward compatible dengan FileReader existing
- Statistics null_count HARUS akurat — ini critical untuk query optimization

**Lanjut ke PHASE 4 setelah semua validasi passing.**
