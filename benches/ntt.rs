use {
    ark_bn254::Fr,
    ark_poly::{EvaluationDomain, Radix2EvaluationDomain},
    criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput},
    delegated_spartan::{intt, ntt},
};

fn bench_ntt(c: &mut Criterion) {
    let mut group = c.benchmark_group("ntt");

    for size in [2, 3, 4, 16, 64, 256, 1024, 3072, 4096, 8192, 16384] {
        let mut input: Vec<Fr> = (0_u64..size).map(Fr::from).collect();
        group.throughput(Throughput::Elements(size));
        group.bench_function(BenchmarkId::new("ntt", size), |b| {
            b.iter(|| ntt(&mut input))
        });
        group.bench_function(BenchmarkId::new("intt", size), |b| {
            b.iter(|| intt(&mut input))
        });
    }
}

fn bench_ark_poly(c: &mut Criterion) {
    let mut group = c.benchmark_group("ark_poly");

    for size in [16, 64, 256, 1024, 4096, 8192, 16384] {
        let mut input: Vec<Fr> = (0_u64..size).map(Fr::from).collect();
        let domain = Radix2EvaluationDomain::<Fr>::new(size as usize).unwrap();
        group.throughput(Throughput::Elements(size));
        group.bench_function(BenchmarkId::new("fft_in_place", size), |b| {
            b.iter(|| domain.fft_in_place(&mut input))
        });
    }
}

criterion_group!(benches, bench_ntt, bench_ark_poly);
criterion_main!(benches);
