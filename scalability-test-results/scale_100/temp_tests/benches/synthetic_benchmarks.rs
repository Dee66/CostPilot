use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_synthetic_1(c: &mut Criterion) {
    c.bench_function(&format!("synthetic_bench_{}", 1), |b| {
        b.iter(|| {
            black_box(42 * 1)
        });
    });
}

criterion_group!(
    benches,
    bench_synthetic_1,
);
criterion_main!(benches);
