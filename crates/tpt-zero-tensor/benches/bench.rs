//! Criterion micro-benchmarks for `tpt-zero-tensor`.
#![allow(clippy::all, clippy::pedantic)]
#![allow(missing_docs)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tpt_zero_tensor::{Tensor, Tensor2};

const N: usize = 1024;
const M: usize = 64;

fn bench_tensor(c: &mut Criterion) {
    let mut group = c.benchmark_group("tensor");

    let a = Tensor::<f64, N>::from_fn(|i| i as f64);
    let b = Tensor::<f64, N>::from_fn(|i| (i as f64) * 0.5);
    group.bench_function("dot_1024", |bench| bench.iter(|| black_box(a.dot(&b))));

    let cvec = Tensor::<f64, N>::from_fn(|i| (i as f64).sin());
    group.bench_function("add_1024", |bench| bench.iter(|| black_box(a.add(&cvec))));
    group.bench_function("map_1024", |bench| {
        bench.iter(|| black_box(a.map(|x| x * 2.0)))
    });

    let mat = Tensor2::<f64, M, M>::from_fn(|r, c| (r * M + c) as f64);
    let t = mat.transpose();
    group.bench_function("mat_mul_64", |bench| bench.iter(|| black_box(mat.mul(&t))));
    group.bench_function("transpose_64", |bench| {
        bench.iter(|| black_box(mat.transpose()))
    });
    group.finish();
}

criterion_group!(benches, bench_tensor);
criterion_main!(benches);
