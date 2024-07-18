use {
    ark_bn254::{Fr, G1Affine, G1Projective},
    ark_ec::scalar_mul::{fixed_base::FixedBase, variable_base::VariableBaseMSM},
    criterion::{black_box, criterion_group, criterion_main, Criterion},
    delegated_spartan::hyrax::{pedersen::PedersenCommitter, HyraxCommiter},
    rand::{Rng, SeedableRng},
    rand_chacha::ChaCha20Rng,
    std::array,
};

fn bench_hyrax_commit(c: &mut Criterion) {
    const SIZE: usize = 1 << 20;
    const COLS: usize = 1 << 10;
    let mut rng = ChaCha20Rng::from_entropy();
    let commiter = HyraxCommiter::new(COLS);
    let scalars = (0..SIZE).map(|_| rng.gen::<Fr>()).collect::<Vec<_>>();
    c.benchmark_group("dummy")
        .sample_size(10)
        .bench_function("hyrax_commit", |b| {
            b.iter(|| commiter.commit(&mut rng, black_box(&scalars)))
        });
}

criterion_group!(benches, bench_hyrax_commit,);
criterion_main!(benches);
