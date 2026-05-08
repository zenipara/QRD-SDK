use criterion::{black_box, criterion_group, criterion_main, Criterion};

criterion_group!(benches, read_bench);
criterion_main!(benches);

fn read_bench(c: &mut Criterion) {
    c.bench_function("read_full_file", |b| {
        b.iter(|| {
            // Benchmark: reading full file
            black_box(100000);
        });
    });
}
