use {
    ark_bn254::Fr,
    criterion::{black_box, criterion_group, criterion_main, Criterion},
    delegated_spartan::pcs::ligero::commit,
    rand::{Rng, SeedableRng},
    rand_chacha::ChaCha20Rng,
    std::time::Instant,
};

fn bench_ligero_commit(c: &mut Criterion) {
    const SIZE: usize = 1 << 20;
    let mut rng = ChaCha20Rng::from_entropy();
    let scalars = (0..SIZE).map(|_| rng.gen::<Fr>()).collect::<Vec<_>>();

    c.benchmark_group("dummy")
        .sample_size(10)
        .bench_function("ligero_commit", |b| {
            b.iter_custom(|iters| {
                let start = Instant::now();
                for _ in 0..iters {
                    commit(black_box(&scalars));
                }
                start.elapsed()
            });
        });
}

criterion_group!(benches, bench_ligero_commit);
criterion_main!(benches);
