mod utils;

use {
    self::utils::{human, time},
    ark_bn254::Fr,
    delegated_spartan::{pcs::hyrax::HyraxCommiter, transcript::Prover},
    rand::{Rng, SeedableRng},
    rand_chacha::ChaCha20Rng,
    std::hint::black_box,
};

fn main() {
    let mut rng = ChaCha20Rng::from_entropy();
    let mut transcript = Prover::new();

    for size_log2 in 10..24 {
        let size: usize = 1 << size_log2;
        let cols = 1 << (size.ilog2() / 2);
        let hyrax = HyraxCommiter::new(cols);
        let scalars = (0..size).map(|_| rng.gen::<Fr>()).collect::<Vec<_>>();
        let transcript = &mut transcript;

        let duration = time(|| {
            transcript.proof.clear();
            hyrax.commit(&mut rng, transcript, black_box(&scalars));
        });
        let proof_size = transcript.proof.len() * size_of::<Fr>();
        println!(
            "Hyrax commitment: size: 2^{size_log2} = {}𝔽, prover time: {}s, througput: {}𝔽/s, proof size: {}B",
            human(size as f64),
            human(duration),
            human(size as f64 / duration),
            human(proof_size as f64)
        );
    }
}
