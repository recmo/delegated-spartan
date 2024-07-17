use {
    crate::{ProverTranscript, VerifierTranscript},
    ark_bn254::{Fr, G1Affine, G1Projective},
    ark_ec::VariableBaseMSM,
    rand::{Rng, SeedableRng},
    rand_chacha::ChaCha20Rng,
    thiserror::Error,
};

const SEED: [u8; 32] = *b"pedersen::PedersenCommitter::new";

pub struct PedersenCommitter {
    // Generators h, g_1, g_2, ..., g_n for the Pedersen commitment scheme.
    generators: Vec<G1Affine>,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("Pedersen commitment failed to verify.")]
    PedersenVerificationFailed,
    #[error("Pedersen commitment equality proof failed to verify.")]
    PedersenEqualityVerificationFailed,
}

impl PedersenCommitter {
    pub fn new(size: usize) -> Self {
        // Generators are PRNG generated.
        let mut rng = ChaCha20Rng::from_seed(SEED);
        Self {
            generators: (0..=size).map(|_| rng.gen()).collect(),
        }
    }

    /// Commit to a value using the Pedersen commitment scheme.
    /// Returns the prover secret and the commitment.
    pub fn commit(&self, rng: &mut impl Rng, values: &[Fr]) -> (Fr, G1Affine) {
        let secret = rng.gen();
        (secret, self.compute_commitment(secret, values))
    }

    pub fn compute_commitment(&self, secret: Fr, values: &[Fr]) -> G1Affine {
        assert!(
            values.len() < self.generators.len(),
            "Too many values for the generators."
        );
        let mut commitment = self.generators[0] * secret;
        commitment += G1Projective::msm_unchecked(&self.generators[1..], values);
        commitment.into()
    }

    /// Batch commitment as in Hyrax.
    pub fn batch_commit(&self, rng: &mut impl Rng, values: &[Fr]) -> Vec<(Fr, G1Affine)> {
        let batch_size = values.len() / (self.generators.len() - 1);
        assert_eq!((self.generators.len() - 1) * batch_size, values.len());
        // This uses Pipenger, but for Hyrax we could also do WNAF over the columns.
        // Benchmarking shows that take about equal time.
        // TODO: there should be a way to combine both.
        values
            .chunks_exact(self.generators.len() - 1)
            .map(|chunk| self.commit(rng, chunk))
            .collect()
    }

    pub fn verify(&self, commitment: G1Affine, secret: Fr, values: &[Fr]) -> Result<(), Error> {
        if commitment == self.compute_commitment(secret, values) {
            Ok(())
        } else {
            Err(Error::PedersenVerificationFailed)
        }
    }

    pub fn proof_equal(
        &self,
        rng: &mut impl Rng,
        transcript: &mut ProverTranscript,
        a: (Fr, G1Affine),
        b: (Fr, G1Affine),
    ) {
        let (s, c) = self.commit(rng, &[]);
        transcript.write_g1(c);
        let r = transcript.read();
        let z = r * (a.0 - b.0) + s;
        transcript.write(z);
    }

    pub fn verify_equal(
        &self,
        transcript: &mut VerifierTranscript,
        a: G1Affine,
        b: G1Affine,
    ) -> Result<(), Error> {
        let c = transcript.read_g1();
        let r = transcript.generate();
        let z = transcript.read();
        let left = self.generators[0] * z;
        let right = (a - b) * r + c;
        if left == right {
            Ok(())
        } else {
            Err(Error::PedersenEqualityVerificationFailed)
        }
    }

    /// Proof that c = a . b.
    /// From a only the secret and values are used, from b nothing is used, and from c only the secret is used.
    pub fn proof_dot_product(
        &self,
        rng: &mut impl Rng,
        transcript: &mut ProverTranscript,
        a: (Fr, &[Fr]),
        c: Fr,
    ) {
        let secret: Vec<Fr> = (0..a.1.len()).map(|_| rng.gen()).collect();
        let secret_dot = a.1.iter().zip(secret.iter()).map(|(a, b)| a * b).sum();
        let (s_b, b) = self.commit(rng, &[secret_dot]);
        let (s_d, d) = self.commit(rng, &secret);
        transcript.write_g1(b);
        transcript.write_g1(d);
        let r = transcript.read();
        secret.into_iter().zip(a.1.iter()).for_each(|(s, a)| {
            transcript.write(s + r * a);
        });
        transcript.write(s_b + r * a.0);
        transcript.write(s_d + r * c);
    }

    // Verify that c = a . b.
    pub fn verify_dot_product(
        &self,
        transcript: &mut VerifierTranscript,
        a: G1Affine,
        b: &[Fr],
        c: G1Affine,
    ) -> Result<(), Error> {
        let cb = transcript.read_g1();
        let cd = transcript.read_g1();
        let r = transcript.generate();
        let z: Vec<Fr> = (0..b.len()).map(|_| transcript.read()).collect();
        let z_b = transcript.read();
        let z_d = transcript.read();

        let secret_dot = z.iter().zip(b.iter()).map(|(z, b)| z * b).sum();
        let lhs = cb + c * r;
        let rhs = self.compute_commitment(z_b, &[secret_dot]);
        if lhs != rhs {
            return Err(Error::PedersenVerificationFailed);
        }
        let lhs = cd + a * r;
        let rhs = self.compute_commitment(z_d, &z);
        if lhs != rhs {
            return Err(Error::PedersenVerificationFailed);
        }
        Ok(())
    }
}
