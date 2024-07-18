pub mod pedersen;

use {
    crate::ProverTranscript,
    ark_bn254::{Fr, G1Affine},
    pedersen::PedersenCommitter,
    rand::Rng,
};

pub struct HyraxCommiter {
    pedersen: PedersenCommitter,
}

impl HyraxCommiter {
    pub fn new(size: usize) -> Self {
        Self {
            pedersen: PedersenCommitter::new(size),
        }
    }

    pub fn commit(&self, rng: &mut impl Rng, coefficients: &[Fr]) -> Vec<(Fr, G1Affine)> {
        self.pedersen.batch_commit(rng, coefficients)
    }

    pub fn proof_evaluation(
        &self,
        rng: &mut impl Rng,
        transcript: &mut ProverTranscript,
        secrets: &[Fr],
        coefficients: &[Fr],
        point: &[Fr],
        value: Fr,
    ) {
        todo!("Implement proof evaluation")
    }
}
