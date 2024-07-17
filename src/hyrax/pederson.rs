use {
    ark_bn254::{Fr, G1Affine, G1Projective},
    rand::Rng,
    std::{array, iter, ops::Mul},
};

pub struct PedersenCommitter {
    // Generators h, g_1, g_2, ..., g_n for the Pedersen commitment scheme.
    // Represented by g^1, g^2, g^4, ..., g^{2^255}
    bit_generators: Vec<[G1Affine; 255]>,
}

impl PedersenCommitter {
    pub fn new(size: usize, rng: impl Rng) -> Self {
        Self {
            bit_generators: (0..=size)
                .map(|_| {
                    let mut g = G1Affine::rand(rng);
                    array::from_fn(|_| {
                        let result = g;
                        g = g.double();
                        result
                    })
                })
                .collect(),
        }
    }

    /// Commit to a value using the Pedersen commitment scheme.
    /// Returns the prover secret and the commitment.
    pub fn commit(&self, value: &[Fr], rng: impl Rng) -> (Fr, G1Affine) {
        assert_eq!(1 + value.len(), self.bit_generators.len());
        let secret = Fr::rand(rng);
        let mut result = G1Projective::zero();
        for (scalar, bitgens) in iter::once(&secret)
            .chain(value.iter())
            .zip(self.bit_generators.iter())
        {
            for (i, bitgen) in bitgens.iter().enumerate() {
                if scalar.get_bit(i) {
                    result += bitgen;
                }
            }
        }
        (secret, result.into())
    }
}
