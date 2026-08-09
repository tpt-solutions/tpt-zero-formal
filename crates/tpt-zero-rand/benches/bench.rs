//! Criterion micro-benchmarks for `tpt-zero-rand`.
#![allow(clippy::all, clippy::pedantic)]
#![allow(missing_docs)]

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use tpt_zero_rand::{Pcg32, Rng, SeedableRng, XorShift64};

fn bench_xorshift(c: &mut Criterion) {
    let mut group = c.benchmark_group("xorshift64");
    let mut rng = XorShift64::seed_from_u64(0xDEAD_BEEF);

    group.bench_function("next_u32", |b| b.iter(|| black_box(rng.next_u32())));
    group.bench_function("next_u64", |b| b.iter(|| black_box(rng.next_u64())));
    group.bench_function("next_f64", |b| b.iter(|| black_box(rng.next_f64())));

    let mut buf = [0u8; 64];
    group.bench_function("fill_bytes/64", |b| {
        b.iter(|| {
            rng.fill_bytes(black_box(&mut buf));
        })
    });
    group.finish();
}

fn bench_pcg(c: &mut Criterion) {
    let mut group = c.benchmark_group("pcg32");
    let mut rng = Pcg32::seed_from_u64(0x1234_5678);

    group.bench_function("next_u32", |b| b.iter(|| black_box(rng.next_u32())));
    group.bench_function("next_u64", |b| b.iter(|| black_box(rng.next_u64())));
    group.bench_function("next_f64", |b| b.iter(|| black_box(rng.next_f64())));
    group.finish();
}

fn bench_throughput(c: &mut Criterion) {
    let mut rng = XorShift64::seed_from_u64(1);
    c.bench_function("xorshift64/1e6_u32", |b| {
        b.iter(|| {
            for _ in 0..1024 {
                black_box(rng.next_u32());
            }
        })
    });
}

criterion_group!(benches, bench_xorshift, bench_pcg, bench_throughput);
criterion_main!(benches);
