use {
    ark_bn254::Fr,
    criterion::{criterion_group, criterion_main, Criterion},
    delegated_spartan::poseidon::permute,
};

fn bench_posedion_permute(c: &mut Criterion) {
    let mut state = [Fr::from(0), Fr::from(1), Fr::from(2)];
    c.bench_function("poseidon", |b| b.iter(|| permute(&mut state)));
}

criterion_group!(benches, bench_posedion_permute);
criterion_main!(benches);
