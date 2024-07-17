mod constants;

use {
    self::constants::{MDS, RC},
    ark_bn254::Fr,
    ark_ff::Field,
};

pub fn permute(state: &mut [Fr; 3]) {
    for (i, rc) in RC.iter().enumerate() {
        // Add round constants
        state[0] += rc[0];
        state[1] += rc[1];
        state[2] += rc[2];

        // Non-linear layer: x -> x^5 (full and half rounds)
        state[0] *= state[0].square().square_in_place();
        if !(4..60).contains(&i) {
            state[1] *= state[1].square().square_in_place();
            state[2] *= state[2].square().square_in_place();
        }

        // MDS layer
        let new_state = [
            MDS[0][0] * state[0] + MDS[0][1] * state[1] + MDS[0][2] * state[2],
            MDS[1][0] * state[0] + MDS[1][1] * state[1] + MDS[1][2] * state[2],
            MDS[2][0] * state[0] + MDS[2][1] * state[1] + MDS[2][2] * state[2],
        ];
        state.copy_from_slice(&new_state);
    }
}

#[cfg(test)]
mod test {
    use {
        super::*,
        ark_ff::{BigInteger, PrimeField},
        hex,
    };

    #[test]
    fn assert_hardcoded_field() {
        assert_eq!(
            hex::encode(Fr::MODULUS.to_bytes_be()),
            "30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001"
        );
    }

    #[test]
    fn test_reference_vector() {
        // Generated using the reference implementation
        // sage ./poseidonperm_x5_254_3.sage
        // Using the script from https://extgit.iaik.tugraz.at/krypto/hadeshash/-/blob/208b5a164c6a252b137997694d90931b2bb851c5/code/poseidonperm_x5_254_3.sage
        let mut state = [Fr::from(0), Fr::from(1), Fr::from(2)];
        permute(&mut state);
        assert_eq!(
            state.map(|n| hex::encode(n.into_bigint().to_bytes_be())),
            [
                "1c1e457a6ef28389aa023b17aa23a8bd43abdb381d28b59e62dba7cde8321155",
                "01d23eabcf873cf73fb12b28cb99bc88f27e7fc4ef7076bd18462d4efc907340",
                "179e2b28b7cffb8b989f97aa8391baec323adad97de4e6f57bee6a76408b5f87"
            ]
        );
    }
}
