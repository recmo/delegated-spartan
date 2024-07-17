use {
    crate::Transcript,
    ark_bn254::{Fr, G1Affine, G1Projective},
    ark_ec::{AffineRepr, CurveGroup, Group},
    ark_ff::{BigInteger, MontFp, PrimeField, UniformRand, Zero},
    rand::{Rng, SeedableRng},
    rand_chacha::ChaCha20Rng,
    std::{array, iter, ops::Mul},
};

const SEED: [u8; 32] = *b"pedersen::PedersenCommitter::new";

pub struct PedersenCommitter {
    // Generators h, g_1, g_2, ..., g_n for the Pedersen commitment scheme.
    // Represented by g^1, g^2, g^4, ..., g^{2^255}
    bit_generators: Vec<[G1Affine; 255]>,
}

impl PedersenCommitter {
    pub fn new(size: usize) -> Self {
        // Generators are PRNG generated.
        // TODO: Include a constants table instead.
        let mut rng = ChaCha20Rng::from_seed(SEED);
        Self {
            bit_generators: (0..=size)
                .map(|_| {
                    let mut g: G1Projective = rng.gen();
                    array::from_fn(|_| {
                        let result = g.into_affine();
                        g.double_in_place();
                        result
                    })
                })
                .collect(),
        }
    }

    /// Commit to a value using the Pedersen commitment scheme.
    /// Returns the prover secret and the commitment.
    pub fn commit(&self, value: &[Fr], rng: &mut impl Rng) -> (Fr, G1Affine) {
        assert_eq!(1 + value.len(), self.bit_generators.len());
        let secret: Fr = rng.gen();
        let mut result = G1Projective::zero();
        for (scalar, bitgens) in iter::once(&secret)
            .chain(value.iter())
            .zip(self.bit_generators.iter())
        {
            let scalar = scalar.into_bigint();
            for (i, bitgen) in bitgens.iter().enumerate() {
                if scalar.get_bit(i) {
                    result += bitgen;
                }
            }
        }
        (secret, result.into())
    }
}
