use {
    criterion::{black_box, criterion_group, criterion_main, Criterion},
    delegated_spartan::hyrax::pedersen::PedersenCommitter,
};

fn bench_pedersen_new(c: &mut Criterion) {
    const SIZE: usize = 1 << 10;
    c.bench_function("pedersen_new", |b| {
        b.iter(|| PedersenCommitter::new(black_box(SIZE)))
    });
}

criterion_group!(benches, bench_pedersen_new);
criterion_main!(benches);
