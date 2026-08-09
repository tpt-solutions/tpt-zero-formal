//! Criterion micro-benchmarks for `tpt-zero-linalg`.
#![allow(clippy::all, clippy::pedantic)]
#![allow(missing_docs)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tpt_zero_linalg::{cross, dot, mat_vec_mul, norm_l2, normalize};
use tpt_zero_tensor::{Tensor, Tensor2};

fn bench_linalg(c: &mut Criterion) {
    let mut group = c.benchmark_group("linalg");

    let a = Tensor::<f64, 1024>::from_fn(|i| i as f64);
    let b = Tensor::<f64, 1024>::from_fn(|i| (i as f64) * 0.5);
    group.bench_function("dot_1024", |bench| bench.iter(|| black_box(dot(&a, &b))));

    let v = Tensor::<f64, 1024>::from_fn(|i| (i as f64).sin());
    group.bench_function("norm_l2_1024", |bench| {
        bench.iter(|| black_box(norm_l2(&v)))
    });
    group.bench_function("normalize_1024", |bench| {
        bench.iter(|| black_box(normalize(&v)))
    });

    let u = Tensor::<f64, 3>::from_fn(|i| i as f64 + 1.0);
    let w = Tensor::<f64, 3>::from_fn(|i| (i as f64 + 2.0).sqrt());
    group.bench_function("cross_3", |bench| bench.iter(|| black_box(cross(&u, &w))));

    let mat = Tensor2::<f64, 64, 64>::from_fn(|r, c| (r * 64 + c) as f64);
    let vec = Tensor::<f64, 64>::from_fn(|i| i as f64);
    group.bench_function("mat_vec_mul_64", |bench| {
        bench.iter(|| black_box(mat_vec_mul(&mat, &vec)))
    });
    group.finish();
}

criterion_group!(benches, bench_linalg);
criterion_main!(benches);
