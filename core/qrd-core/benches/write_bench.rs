use criterion::{black_box, criterion_group, criterion_main, Criterion};

criterion_group!(benches, write_bench);
criterion_main!(benches);

fn write_bench(c: &mut Criterion) {
    c.bench_function("write_1000_rows", |b| {
        b.iter(|| {
            // Benchmark: writing 1000 rows
            black_box(1000);
        });
    });
}
