use {
    ark_bn254::Fr, ark_crypto_primitives::sponge::poseidon::PoseidonSponge,
    num_traits::identities::Zero, rand::SeedableRng, rand_chacha::ChaCha20Rng,
};

const CONFIG: PoseidonConfig<Fr> = PoseidonConfig {
    /// Number of rounds in a full-round operation.
    pub full_rounds: usize,
    /// Number of rounds in a partial-round operation.
    pub partial_rounds: usize,
    /// Exponent used in S-boxes.
    pub alpha: u64,
    /// Additive Round keys. These are added before each MDS matrix application to make it an affine shift.
    /// They are indexed by `ark[round_num][state_element_index]`
    pub ark: Vec<Vec<F>>,
    /// Maximally Distance Separating (MDS) Matrix.
    pub mds: Vec<Vec<F>>,
    /// The rate (in terms of number of field elements).
    /// See [On the Indifferentiability of the Sponge Construction](https://iacr.org/archive/eurocrypt2008/49650180/49650180.pdf)
    /// for more details on the rate and capacity of a sponge.
    pub rate: usize,
    /// The capacity (in terms of number of field elements).
    pub capacity: usize,
}

pub struct Transcript {
    hasher: PoseidonSponge<Fr>,
    state: Fr,
}

impl Transcript {
    pub fn new() -> Self {
        let mut rng = ChaCha20Rng::seed_from_u64(SEED);
        let params = Fr::get
        Self {
            hasher: Poseidon::new(PoseidonParameters::generate(&mut rng)),
            state: Fr::zero(),
        }
    }

    pub fn send(&mut self, message: Fr) {
        self.state = self.hasher.hash_two(self.state, message);
    }

    pub fn challenge_scalar(&mut self, label: &str) -> u64 {
        println!("{}: 0", label);
        0
    }
}
