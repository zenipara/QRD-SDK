use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use qrd_core::encoding::*;
use qrd_core::utils::simd::SimdOps;

fn encoding_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding");

    // Test data sizes
    let sizes = [1_000, 10_000, 100_000, 1_000_000];

    for &size in &sizes {
        // Random data
        let random_data: Vec<i32> = (0..size).map(|i| i as i32 % 1000).collect();

        // Repetitive data (good for RLE)
        let repetitive_data: Vec<i32> = vec![42; size];

        // Delta-friendly data (good for DELTA_BINARY)
        let delta_data: Vec<i32> = (0..size).map(|i| i as i32).collect();

        group.bench_with_input(BenchmarkId::new("plain/random", size), &random_data, |b, data| {
            b.iter(|| {
                let mut encoder = PlainEncoder::new();
                let mut output = Vec::new();
                encoder.encode(black_box(data), &mut output).unwrap();
                black_box(output);
            });
        });

        group.bench_with_input(BenchmarkId::new("rle/repetitive", size), &repetitive_data, |b, data| {
            b.iter(|| {
                let mut encoder = RleEncoder::new();
                let mut output = Vec::new();
                encoder.encode(black_box(data), &mut output).unwrap();
                black_box(output);
            });
        });

        group.bench_with_input(BenchmarkId::new("delta_binary/sequential", size), &delta_data, |b, data| {
            b.iter(|| {
                let mut encoder = DeltaBinaryEncoder::new();
                let mut output = Vec::new();
                encoder.encode(black_box(data), &mut output).unwrap();
                black_box(output);
            });
        });

        group.bench_with_input(BenchmarkId::new("bit_packed/random", size), &random_data, |b, data| {
            b.iter(|| {
                let mut encoder = BitPackedEncoder::new();
                let mut output = Vec::new();
                encoder.encode(black_box(data), &mut output).unwrap();
                black_box(output);
            });
        });
    }

    group.finish();
}

fn decoding_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("decoding");

    // Create encoded data for decoding benchmarks
    let original_data: Vec<i32> = (0..100_000).map(|i| i as i32 % 100).collect();

    // Encode with different algorithms
    let mut plain_encoded = Vec::new();
    PlainEncoder::new().encode(&original_data, &mut plain_encoded).unwrap();

    let mut rle_encoded = Vec::new();
    RleEncoder::new().encode(&original_data, &mut rle_encoded).unwrap();

    let mut delta_encoded = Vec::new();
    DeltaBinaryEncoder::new().encode(&original_data, &mut delta_encoded).unwrap();

    let mut bitpacked_encoded = Vec::new();
    BitPackedEncoder::new().encode(&original_data, &mut bitpacked_encoded).unwrap();

    group.bench_function("plain/decode", |b| {
        b.iter(|| {
            let mut decoder = PlainDecoder::new();
            let mut output = Vec::new();
            decoder.decode(&plain_encoded, &mut output).unwrap();
            black_box(output);
        });
    });

    group.bench_function("rle/decode", |b| {
        b.iter(|| {
            let mut decoder = RleDecoder::new();
            let mut output = Vec::new();
            decoder.decode(&rle_encoded, &mut output).unwrap();
            black_box(output);
        });
    });

    group.bench_function("delta_binary/decode", |b| {
        b.iter(|| {
            let mut decoder = DeltaBinaryDecoder::new();
            let mut output = Vec::new();
            decoder.decode(&delta_encoded, &mut output).unwrap();
            black_box(output);
        });
    });

    group.bench_function("bit_packed/decode", |b| {
        b.iter(|| {
            let mut decoder = BitPackedDecoder::new();
            let mut output = Vec::new();
            decoder.decode(&bitpacked_encoded, &mut output).unwrap();
            black_box(output);
        });
    });

    group.finish();
}

fn compression_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("compression");

    let data = vec![0u8; 1_000_000]; // 1MB of compressible data

    group.bench_function("zstd/level1", |b| {
        b.iter(|| {
            let compressed = qrd_core::compression::compress(&data, qrd_core::compression::CompressionType::Zstd(1)).unwrap();
            black_box(compressed);
        });
    });

    group.bench_function("zstd/level6", |b| {
        b.iter(|| {
            let compressed = qrd_core::compression::compress(&data, qrd_core::compression::CompressionType::Zstd(6)).unwrap();
            black_box(compressed);
        });
    });

    group.bench_function("lz4/default", |b| {
        b.iter(|| {
            let compressed = qrd_core::compression::compress(&data, qrd_core::compression::CompressionType::Lz4(4)).unwrap();
            black_box(compressed);
        });
    });

    group.finish();
}

fn simd_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd");
    let ops = SimdOps::new();

    let large_data = vec![42u8; 1_000_000];
    let mut dst = vec![0u8; 1_000_000];

    group.bench_function("memcpy/large", |b| {
        b.iter(|| {
            ops.memcpy(&mut dst, &large_data).unwrap();
            black_box(&dst);
        });
    });

    let xor_data = vec![1u8; 1_000_000];
    let mut xor_dst = vec![255u8; 1_000_000];

    group.bench_function("xor/large", |b| {
        b.iter(|| {
            ops.xor(&mut xor_dst, &xor_data).unwrap();
            black_box(&xor_dst);
        });
    });

    let count_data = vec![1u8; 1_000_000];

    group.bench_function("count_bytes", |b| {
        b.iter(|| {
            let count = ops.count_bytes(&count_data, 1);
            black_box(count);
        });
    });

    let delta_data: Vec<i32> = (0..100_000).map(|i| i as i32).collect();

    group.bench_function("delta_encode_i32", |b| {
        b.iter(|| {
            let encoded = ops.delta_encode_i32(&delta_data).unwrap();
            black_box(encoded);
        });
    });

    let encoded_delta = ops.delta_encode_i32(&delta_data).unwrap();

    group.bench_function("delta_decode_i32", |b| {
        b.iter(|| {
            let decoded = ops.delta_decode_i32(&encoded_delta).unwrap();
            black_box(decoded);
        });
    });

    group.finish();
}

fn encryption_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("encryption");

    let key = qrd_core::encryption::EncryptionConfig::generate_key();
    let config = qrd_core::encryption::EncryptionConfig::new(key).unwrap();
    let data = vec![42u8; 100_000]; // 100KB

    group.bench_function("aes_gcm/encrypt", |b| {
        b.iter(|| {
            let encrypted = qrd_core::encryption::encrypt(&data, &config).unwrap();
            black_box(encrypted);
        });
    });

    let encrypted = qrd_core::encryption::encrypt(&data, &config).unwrap();

    group.bench_function("aes_gcm/decrypt", |b| {
        b.iter(|| {
            let decrypted = qrd_core::encryption::decrypt(&encrypted, &config).unwrap();
            black_box(decrypted);
        });
    });

    group.finish();
}

fn ecc_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("ecc");

    let config = qrd_core::ecc::EccConfig::with_chunk_size(4, 4096).unwrap();
    let data = vec![42u8; 100_000]; // 100KB

    group.bench_function("reed_solomon/encode", |b| {
        b.iter(|| {
            let mut codec = qrd_core::ecc::EccCodec::new(config.clone()).unwrap();
            let encoded = codec.encode(&data).unwrap();
            black_box(encoded);
        });
    });

    let mut codec = qrd_core::ecc::EccCodec::new(config.clone()).unwrap();
    let encoded = codec.encode(&data).unwrap();

    group.bench_function("reed_solomon/decode", |b| {
        b.iter(|| {
            let shards = encoded.shards_as_options();
            let recovered = qrd_core::ecc::decode_and_recover(&shards, &config).unwrap();
            black_box(recovered);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    encoding_benchmarks,
    decoding_benchmarks,
    compression_benchmarks,
    simd_benchmarks,
    encryption_benchmarks,
    ecc_benchmarks
);
criterion_main!(benches);
        });
    });
}
