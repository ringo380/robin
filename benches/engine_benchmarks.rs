use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_math_operations(c: &mut Criterion) {
    c.bench_function("vector2_add", |b| {
        b.iter(|| {
            let v1 = robin::engine::math::Vec2::new(1.0, 2.0);
            let v2 = robin::engine::math::Vec2::new(3.0, 4.0);
            black_box(v1 + v2)
        })
    });
}

criterion_group!(benches, benchmark_math_operations);
criterion_main!(benches);