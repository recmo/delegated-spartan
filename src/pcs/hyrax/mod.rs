pub mod pedersen;

use {
    crate::{Prover, Verifier},
    ark_bn254::{Fr, G1Affine, G1Projective},
    ark_ec::VariableBaseMSM,
    ark_ff::Zero,
    pedersen::PedersenCommitter,
    rand::Rng,
};

/// Reference implementation of the contraction function.
/// Computes $a \cdot F \cdot b$ where $f$ is taken in row-major order.
pub fn compute_contraction(f: &[Fr], a: &[Fr], b: &[Fr]) -> Fr {
    assert_eq!(f.len(), a.len() * b.len());
    f.chunks_exact(b.len())
        .zip(a)
        .map(|(row, &a)| a * row.iter().zip(b).map(|(f, b)| f * b).sum::<Fr>())
        .sum()
}

pub struct HyraxCommiter {
    pub pedersen: PedersenCommitter,
}

impl HyraxCommiter {
    pub fn new(size: usize) -> Self {
        Self {
            pedersen: PedersenCommitter::new(size),
        }
    }

    pub fn commit(&self, rng: &mut impl Rng, transcript: &mut Prover, f: &[Fr]) -> Vec<Fr> {
        self.pedersen.batch_commit(rng, transcript, f)
    }

    pub fn proof_contraction(
        &self,
        rng: &mut impl Rng,
        transcript: &mut Prover,
        f: (&[Fr], &[Fr]), // Secrets and values
        a: &[Fr],          // Values
        b: &[Fr],          // Values
        c: Fr,             // Secret
    ) {
        assert_eq!(f.1.len(), a.len() * b.len());
        assert_eq!(f.0.len(), a.len());

        // Linearly combine the rows of the matrix and their secrets.
        let s = f.0.iter().zip(a.iter()).map(|(s, a)| s * a).sum();
        let f = f.1.chunks_exact(b.len()).zip(a.iter()).fold(
            vec![Fr::zero(); b.len()],
            |mut acc, (row, a)| {
                acc.iter_mut().zip(row).for_each(|(acc, f)| *acc += a * f);
                acc
            },
        );

        // Prove dot product relation
        self.pedersen
            .prove_dot_product(rng, transcript, (s, &f), b, c);
    }

    pub fn verify_contraction(
        &self,
        transcript: &mut Verifier,
        commitments: &[G1Affine],
        a: &[Fr],
        b: &[Fr],
        c: G1Affine,
    ) {
        assert_eq!(commitments.len(), a.len());

        // Linearly combine the commitments.
        let a = G1Projective::msm_unchecked(commitments, a).into();

        // Verify dot product relation
        self.pedersen
            .verify_dot_product(transcript, a, b, c)
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use {super::*, rand::SeedableRng, rand_chacha::ChaCha20Rng};

    #[test]
    fn test_contraction() {
        let mut rng = ChaCha20Rng::from_entropy();
        let (rows, cols) = (10, 20);
        let hyrax = HyraxCommiter::new(cols);
        let f = (0..rows * cols).map(|_| rng.gen()).collect::<Vec<Fr>>();
        let a = (0..rows).map(|_| rng.gen()).collect::<Vec<Fr>>();
        let b = (0..cols).map(|_| rng.gen()).collect::<Vec<Fr>>();
        let c = compute_contraction(&f, &a, &b);

        // Prove
        let mut transcript = Prover::new();
        let s = hyrax.commit(&mut rng, &mut transcript, &f);
        let (sc, cc) = hyrax.pedersen.commit(&mut rng, &[c]);
        transcript.write_g1(cc);
        hyrax.proof_contraction(&mut rng, &mut transcript, (&s, &f), &a, &b, sc);
        let proof = transcript.finish();
        dbg!(proof.len() * std::mem::size_of::<Fr>());

        // Verify
        let mut transcript = Verifier::new(&proof);
        let cs = (0..rows).map(|_| transcript.read_g1()).collect::<Vec<_>>();
        let cc = transcript.read_g1();
        hyrax.verify_contraction(&mut transcript, &cs, &a, &b, cc);
    }
}
