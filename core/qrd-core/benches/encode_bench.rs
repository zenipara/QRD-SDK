use criterion::{black_box, criterion_group, criterion_main, Criterion};

criterion_group!(benches, encode_bench);
criterion_main!(benches);

fn encode_bench(c: &mut Criterion) {
    c.bench_function("encode_plain_1mb", |b| {
        let data = vec![0u8; 1024 * 1024];
        b.iter(|| {
            black_box(&data);
        });
    });

    c.bench_function("encode_rle_repetitive", |b| {
        let mut data = Vec::with_capacity(1024 * 1024);
        for _ in 0..1024 * 512 {
            data.push(42u8);
        }
        b.iter(|| {
            black_box(&data);
        });
    });
}
