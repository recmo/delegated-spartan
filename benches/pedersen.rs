use {
    ark_bn254::{Fr, G1Affine, G1Projective},
    ark_ec::scalar_mul::{fixed_base::FixedBase, variable_base::VariableBaseMSM},
    criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput},
    delegated_spartan::hyrax::pedersen::PedersenCommitter,
    rand::{Rng, SeedableRng},
    rand_chacha::ChaCha20Rng,
    std::array,
};

fn bench_pedersen_new(c: &mut Criterion) {
    const SIZE: usize = 1 << 10;
    c.bench_function("pedersen_new", |b| {
        b.iter(|| PedersenCommitter::new(black_box(SIZE)))
    });
}

fn bench_pedersen_commit(c: &mut Criterion) {
    const SIZE: usize = 1 << 10;
    let mut rng = ChaCha20Rng::from_entropy();
    let commiter = PedersenCommitter::new(SIZE);
    let scalars = (0..SIZE).map(|_| rng.gen::<Fr>()).collect::<Vec<_>>();
    c.bench_function("pedersen_commit", |b| {
        b.iter(|| commiter.commit(&mut rng, black_box(&scalars)))
    });

    let mut rng = ChaCha20Rng::from_entropy();
    let mut group = c.benchmark_group("pedersen");
    let commiter = PedersenCommitter::new(10_000);
    for size in [100, 1000, 1024, 2048, 4096, 10_000] {
        let input: Vec<Fr> = (0_u64..size).map(|_| rng.gen()).collect();
        group.throughput(Throughput::Elements(size));
        group.bench_function(BenchmarkId::new("commit", size), |b| {
            b.iter(|| commiter.commit(&mut rng, black_box(&input)))
        });
    }
}

fn ref_wnaf(c: &mut Criterion) {
    const SIZE: usize = 1 << 10;
    // assert_eq!(rayon::current_num_threads(), 1);
    let mut rng = ChaCha20Rng::from_entropy();
    let generator: G1Projective = rng.gen();
    let scalars = (0..SIZE).map(|_| rng.gen::<Fr>()).collect::<Vec<_>>();
    let window_size = FixedBase::get_mul_window_size(SIZE); // returns 6 for size 1000
    let windows = FixedBase::get_window_table(254, window_size, generator);
    c.bench_function("reference_wnaf", |b| {
        b.iter(|| FixedBase::msm::<G1Projective>(254, window_size, &windows, black_box(&scalars)))
    });
}

fn ref_pipenger(c: &mut Criterion) {
    const SIZE: usize = 1 << 10;
    // assert_eq!(rayon::current_num_threads(), 1);
    let mut rng = ChaCha20Rng::from_entropy();
    let generators: [G1Affine; SIZE] = array::from_fn(|_| rng.gen());
    let scalars = (0..SIZE).map(|_| rng.gen::<Fr>()).collect::<Vec<_>>();
    c.bench_function("reference_pipenger", |b| {
        b.iter(|| G1Projective::msm(&generators, black_box(&scalars)))
    });
}

criterion_group!(
    benches,
    bench_pedersen_new,
    bench_pedersen_commit,
    ref_wnaf,
    ref_pipenger
);
criterion_main!(benches);
