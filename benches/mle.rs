use {
    ark_bn254::Fr,
    criterion::{black_box, criterion_group, criterion_main, Criterion},
    delegated_spartan::{
        mle::{eval_mle, par_eval_mle, prove_sumcheck},
        Prover,
    },
    rand::{Rng, SeedableRng},
    rand_chacha::ChaCha20Rng,
};

fn bench_eval_mle(c: &mut Criterion) {
    const SIZE: usize = 20;
    let mut rng = ChaCha20Rng::from_entropy();
    let f = (0..1 << SIZE).map(|_| rng.gen::<Fr>()).collect::<Vec<_>>();
    let e = (0..SIZE).map(|_| rng.gen::<Fr>()).collect::<Vec<_>>();
    c.bench_function("eval_mle", |b| b.iter(|| eval_mle(&f, black_box(&e))));
}

fn bench_par_eval_mle(c: &mut Criterion) {
    const SIZE: usize = 20;
    let mut rng = ChaCha20Rng::from_entropy();
    let f = (0..1 << SIZE).map(|_| rng.gen::<Fr>()).collect::<Vec<_>>();
    let e = (0..SIZE).map(|_| rng.gen::<Fr>()).collect::<Vec<_>>();
    c.bench_function("par_eval_mle", |b| {
        b.iter(|| par_eval_mle(&f, black_box(&e)))
    });
}

fn bench_prove_sumcheck(c: &mut Criterion) {
    const SIZE: usize = 20;
    let mut rng = ChaCha20Rng::from_entropy();
    let mut f = (0..1 << SIZE).map(|_| rng.gen::<Fr>()).collect::<Vec<_>>();
    c.bench_function("prove_sumcheck", |b| {
        b.iter(|| {
            let mut transcript = Prover::new();
            prove_sumcheck(&mut transcript, &mut f, SIZE);
            transcript.finish()
        })
    });
}

criterion_group!(
    benches,
    bench_eval_mle,
    bench_par_eval_mle,
    bench_prove_sumcheck
);
criterion_main!(benches);
