use {
    ark_bn254::{Fr, G1Affine, G1Projective},
    ark_ec::VariableBaseMSM,
    rand::{Rng, SeedableRng},
    rand_chacha::ChaCha20Rng,
};

const SEED: [u8; 32] = *b"pedersen::PedersenCommitter::new";

pub struct PedersenCommitter {
    // Generators h, g_1, g_2, ..., g_n for the Pedersen commitment scheme.
    generators: Vec<G1Affine>,
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
    pub fn commit(&self, values: &[Fr], rng: &mut impl Rng) -> (Fr, G1Affine) {
        assert_eq!(1 + values.len(), self.generators.len());
        let secret: Fr = rng.gen();
        let mut commitment = G1Projective::from(self.generators[0]) * secret;
        // This uses Pipenger, but for Hyrax we could also do WNAF over the columns.
        // Benchmarking shows that take about equal time, and pipenger is easier here.
        // but there should be a way to combine both.
        commitment += G1Projective::msm(&self.generators[1..], values).expect("Lengths are equal");
        (secret, commitment.into())
    }
}
