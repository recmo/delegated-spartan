use {
    ark_bn254::Fr,
    criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput},
    delegated_spartan::{poseidon2, poseidon_permute},
    std::array,
};

fn bench_poseidon_permute(c: &mut Criterion) {
    let mut state: [Fr; 3] = array::from_fn(|i| Fr::from(i as u64));
    c.bench_function("poseidon", |b| b.iter(|| poseidon_permute(&mut state)));
}

fn bench_poseidon2(c: &mut Criterion) {
    let mut group = c.benchmark_group("poseidon2");

    let mut state: [Fr; 3] = array::from_fn(|i| Fr::from(i as u64));
    group.throughput(Throughput::Elements(3));
    group.bench_function(BenchmarkId::new("permute", 3), |b| {
        b.iter(|| poseidon2::permute_3(&mut state))
    });

    let mut state: [Fr; 16] = array::from_fn(|i| Fr::from(i as u64));
    group.throughput(Throughput::Elements(16));
    group.bench_function(BenchmarkId::new("permute", 16), |b| {
        b.iter(|| poseidon2::permute_16(&mut state))
    });

    for size in [100, 1000, 1024, 2048, 4096, 10_000] {
        let input: Vec<Fr> = (0_u64..size).map(Fr::from).collect();
        group.throughput(Throughput::Elements(size));
        group.bench_function(BenchmarkId::new("compress", size), |b| {
            b.iter(|| poseidon2::compress(&input))
        });
    }
}

criterion_group!(benches, bench_poseidon_permute, bench_poseidon2);
criterion_main!(benches);
