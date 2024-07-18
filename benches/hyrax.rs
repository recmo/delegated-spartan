use {
    criterion::{black_box, criterion_group, criterion_main, Criterion},
    delegated_spartan::hyrax::HyraxCommiter,
    rand::{Rng, SeedableRng},
    rand_chacha::ChaCha20Rng,
};

fn bench_hyrax_commit(c: &mut Criterion) {
    const SIZE: usize = 1 << 20;
    const COLS: usize = 1 << 10;
    let mut rng = ChaCha20Rng::from_entropy();
    let hyrax = HyraxCommiter::new(COLS);
    let scalars = (0..SIZE).map(|_| rng.gen::<Fr>()).collect::<Vec<_>>();
    c.benchmark_group("dummy")
        .sample_size(10)
        .bench_function("hyrax_commit", |b| {
            b.iter(|| hyrax.commit(&mut rng, black_box(&scalars)))
        });
}

fn bench_hyrax_prove(c: &mut Criterion) {
    const SIZE: usize = 1 << 20;
    const COLS: usize = 1 << 10;
    let mut rng = ChaCha20Rng::from_entropy();
    let hyrax = HyraxCommiter::new(COLS);
    let scalars = (0..SIZE).map(|_| rng.gen::<Fr>()).collect::<Vec<_>>();
    c.benchmark_group("dummy")
        .sample_size(10)
        .bench_function("hyrax_commit", |b| {
            b.iter(|| hyrax.commit(&mut rng, black_box(&scalars)))
        });
}

criterion_group!(benches, bench_hyrax_commit,);
criterion_main!(benches);
