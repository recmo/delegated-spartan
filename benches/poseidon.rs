use {
    ark_bn254::Fr,
    criterion::{criterion_group, criterion_main, Criterion},
    delegated_spartan::{poseidon2, poseidon_permute},
    std::array,
};

fn bench_poseidon_permute(c: &mut Criterion) {
    let mut state: [Fr; 3] = array::from_fn(|i| Fr::from(i as u64));
    c.bench_function("poseidon", |b| b.iter(|| poseidon_permute(&mut state)));
}

fn bench_poseidon2_permute_3(c: &mut Criterion) {
    let mut state: [Fr; 3] = array::from_fn(|i| Fr::from(i as u64));
    c.bench_function("poseidon2/3", |b| {
        b.iter(|| poseidon2::permute_3(&mut state))
    });
}

fn bench_poseidon2_permute_16(c: &mut Criterion) {
    let mut state: [Fr; 16] = array::from_fn(|i| Fr::from(i as u64));
    c.bench_function("poseidon2/16", |b| {
        b.iter(|| poseidon2::permute_16(&mut state))
    });
}

criterion_group!(
    benches,
    bench_poseidon_permute,
    bench_poseidon2_permute_3,
    bench_poseidon2_permute_16
);
criterion_main!(benches);
