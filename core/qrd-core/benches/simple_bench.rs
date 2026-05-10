use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use qrd_core::utils::simd::SimdOps;

fn simd_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("simd");

    // Test data sizes
    let sizes = [1_000, 10_000, 100_000];

    for &size in &sizes {
        // Test delta encoding
        let data: Vec<i32> = (0..size as i32).collect();
        let simd = SimdOps::new();

        group.bench_with_input(BenchmarkId::new("delta_encode/i32", size), &size, |b, _| {
            b.iter(|| {
                let result = simd.delta_encode_i32(black_box(&data)).unwrap();
                black_box(result);
            });
        });

        group.bench_with_input(BenchmarkId::new("delta_decode/i32", size), &size, |b, _| {
            let encoded = simd.delta_encode_i32(&data).unwrap();
            b.iter(|| {
                let result = simd.delta_decode_i32(black_box(&encoded)).unwrap();
                black_box(result);
            });
        });

        // Test find runs
        group.bench_with_input(BenchmarkId::new("find_runs", size), &size, |b, _| {
            let runs_data: Vec<u8> = (0..size as u32).map(|i| ((i % 10) as u8)).collect();
            b.iter(|| {
                let result = simd.find_runs(black_box(&runs_data));
                black_box(result);
            });
        });

        // Test count bytes
        group.bench_with_input(BenchmarkId::new("count_bytes", size), &size, |b, _| {
            let bytes: Vec<u8> = (0..size as u32).map(|i| ((i % 255) as u8)).collect();
            b.iter(|| {
                let result = simd.count_bytes(black_box(&bytes), 42);
                black_box(result);
            });
        });

        // Test XOR operation
        group.bench_with_input(BenchmarkId::new("xor", size), &size, |b, _| {
            let dst = vec![0u8; size];
            let src: Vec<u8> = (0..size as u32).map(|i| ((i % 255) as u8)).collect();
            b.iter(|| {
                let mut dst_copy = dst.clone();
                simd.xor(black_box(&mut dst_copy), black_box(&src)).unwrap();
                black_box(dst_copy);
            });
        });

        // Test memcpy
        group.bench_with_input(BenchmarkId::new("memcpy", size), &size, |b, _| {
            let dst = vec![0u8; size];
            let src: Vec<u8> = (0..size as u32).map(|i| ((i % 255) as u8)).collect();
            b.iter(|| {
                let mut dst_copy = dst.clone();
                simd.memcpy(black_box(&mut dst_copy), black_box(&src))
                    .unwrap();
                black_box(dst_copy);
            });
        });
    }

    group.finish();
}

fn varint_benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("varint");

    // Test data sizes
    let sizes = [1_000, 10_000, 100_000];

    for &size in &sizes {
        group.bench_with_input(BenchmarkId::new("encode", size), &size, |b, _| {
            let values: Vec<u64> = (0..size as u64).map(|i| i * 1000).collect();
            b.iter(|| {
                let mut results = Vec::new();
                for &val in &values {
                    let encoded = qrd_core::utils::varint::encode(black_box(val));
                    results.push(encoded);
                }
                black_box(results);
            });
        });

        group.bench_with_input(BenchmarkId::new("decode", size), &size, |b, _| {
            let values: Vec<u64> = (0..size as u64).map(|i| i * 1000).collect();
            let encoded: Vec<_> = values
                .iter()
                .map(|&v| qrd_core::utils::varint::encode(v))
                .collect();

            b.iter(|| {
                let mut results = Vec::new();
                for data in &encoded {
                    let (decoded, _) = qrd_core::utils::varint::decode(black_box(data)).unwrap();
                    results.push(decoded);
                }
                black_box(results);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, simd_benchmarks, varint_benchmarks);
criterion_main!(benches);
