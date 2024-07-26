pub mod hyrax;
pub mod ligero;
pub mod mle;
mod transcript;

pub use transcript::{poseidon2, poseidon_permute, ProverTranscript, Sponge, VerifierTranscript};
use {
    ark_bn254::Fr,
    ark_ff::{Field, Zero},
    hyrax::HyraxCommiter,
    mle::{prove_sumcheck_product, prove_sumcheck_r1cs},
    rand::Rng,
};

pub fn prove_r1cs(
    rng: &mut impl Rng,
    transcript: &mut ProverTranscript,
    size: usize,
    A: &[(usize, usize, Fr)],
    B: &[(usize, usize, Fr)],
    C: &[(usize, usize, Fr)],
    z: &[Fr],
) {
    let hyrax = HyraxCommiter::new(size);

    // Commit to z
    let z_commitment = hyrax.commit(rng, transcript, z);

    // Compute A ⋅ z, B ⋅ z, C ⋅ z
    let mut az = vec![Fr::zero(); size];
    let mut bz = vec![Fr::zero(); size];
    let mut cz = vec![Fr::zero(); size];

    // Compute MLE of eq(r, x)
    let e = todo!();

    // Prove the sum equals zero
    // let (r, rs) = prove_sumcheck_r1cs(transcript, size, e, az, bz, cz, Fr::zero());

    // Random linear combination of Az, Bz, Cz

    // Prove M ⋅ z
    // let (r, rs) = prove_sumcheck_product(transcript, size, f, g, sum);

    todo!()
}
