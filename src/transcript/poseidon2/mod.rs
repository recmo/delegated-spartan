//! # Poseidon2 implementation
//!
//! # References
//!
//! * Lorenzo Grassi, Dmitry Khovratovich, Markus Schofnegger (2023).
//!   Poseidon2: A Faster Version of the Poseidon Hash Function.
//!   <https://eprint.iacr.org/2023/323>
//! * Tomer Ashur, Thomas Buschman, Mohammad Mahzoun (2023).
//!   Algebraic Cryptanalysis of HADES Design Strategy: Application to POSEIDON and Poseidon2.
//!   <https://eprint.iacr.org/2023/537>
//! * Elena Andreeva, Rishiraj Bhattacharyya, Arnab Roy, Stefano Trevisani (2024).
//!   On Efficient and Secure Compression Modes for Arithmetization-Oriented Hashing.
//!   <https://eprint.iacr.org/2024/047>
//!
//! See also:
//! * https://github.com/HorizenLabs/poseidon2/blob/bb476b9ca38198cf5092487283c8b8c5d4317c4e/plain_implementations/src/poseidon2/poseidon2.rs
//! * https://github.com/Plonky3/Plonky3/blob/eeb4e37b20127c4daa871b2bad0df30a7c7380db/poseidon2/src/lib.rs

// https://hackmd.io/@hackmdhl/B1DdpVmK2
// https://extgit.iaik.tugraz.at/krypto/zkfriendlyhashzoo/-/blob/master/plain_impls/src/poseidon2/poseidon2_instance_bn256.rs?ref_type=heads
// https://eprint.iacr.org/2024/310.pdf

mod constants;

use {
    self::constants::{MAT_DIAG16, RC16, RC3},
    ark_bn254::Fr,
    ark_ff::Field,
};

// Compress arbitrary length inputs.
// Compute 16-arry Merkle tree over input.
// Compresses nodes using truncated width-16 Poseidon2.
pub fn compress(input: &[Fr]) -> Fr {
    if input.len() <= 16 {
        let mut state = [Fr::ZERO; 16];
        state[..input.len()].copy_from_slice(input);
        permute_16(&mut state);
        state[0]
    } else {
        // Allocation free depth-first recursive computation
        let mut state = [Fr::ZERO; 16];
        // Compute the largest power of 16 < input.len();
        let chunk = 1 << (4 * ((input.len() - 1).ilog2() / 4));
        for (s, chunk) in state.iter_mut().zip(input.chunks(chunk)) {
            *s = compress(chunk);
        }
        permute_16(&mut state);
        state[0]
    }
}

pub fn permute_3(state: &mut [Fr; 3]) {
    mat_full_3(state);
    for rc in RC3.0 {
        state.iter_mut().zip(rc).for_each(|(x, rc)| *x += rc);
        state
            .iter_mut()
            .for_each(|x| *x *= x.square().square_in_place());
        mat_full_3(state);
    }
    for rc in RC3.1 {
        state[0] += rc;
        state[0] *= state[0].square().square_in_place();

        // TODO: Why is this one more operations than the MDS matrix?
        let sum: Fr = state.iter().sum();
        state[2].double_in_place();
        state.iter_mut().for_each(|s| *s += sum);
    }
    for rc in RC3.2 {
        state.iter_mut().zip(rc).for_each(|(x, rc)| *x += rc);
        state
            .iter_mut()
            .for_each(|x| *x *= x.square().square_in_place());
        mat_full_3(state);
    }
}

pub fn permute_16(state: &mut [Fr; 16]) {
    mat_full_16(state);
    for rc in RC16.0 {
        // TODO: Combine passes?
        // Should be able to fold the linear layer into the Montgomery reduction.
        state.iter_mut().zip(rc).for_each(|(x, rc)| *x += rc);
        state
            .iter_mut()
            .for_each(|x| *x *= x.square().square_in_place());
        mat_full_16(state);
    }
    for rc in RC16.1 {
        state[0] += rc;
        state[0] *= state[0].square().square_in_place();

        // Matmul partial: ones(N, N) + diag(MAT_DIAG16)
        // TODO: This one is also more expensive than mat_full_16!
        let sum: Fr = state.iter().sum();
        state.iter_mut().zip(MAT_DIAG16).for_each(|(s, d)| {
            *s *= d;
            *s += sum;
        });
    }
    for rc in RC16.2 {
        state.iter_mut().zip(rc).for_each(|(x, rc)| *x += rc);
        state
            .iter_mut()
            .for_each(|x| *x *= x.square().square_in_place());
        mat_full_16(state);
    }
}

pub fn mat_full_3(state: &mut [Fr; 3]) {
    // Matrix circ(2, 1, 1)
    let sum: Fr = state.iter().sum();
    state.iter_mut().for_each(|s| *s += sum);
}

pub fn mat_full_4(state: &mut [Fr; 4]) {
    let t0 = state[0] + state[1];
    let t1 = state[2] + state[3];
    let t2 = state[1].double() + t1;
    let t3 = state[3].double() + t0;
    let t4 = t1.double().double() + t3;
    let t5 = t0.double().double() + t2;
    let t6 = t3 + t5;
    let t7 = t2 + t4;
    state[0] = t6;
    state[1] = t5;
    state[2] = t7;
    state[3] = t4;
}

pub fn mat_full_16(state: &mut [Fr; 16]) {
    // TODO: Use array_chunks_mut when it is stable
    let mut sum = [Fr::ZERO; 4];
    state.chunks_exact_mut(4).for_each(|s| {
        let s: &mut [Fr; 4] = s.try_into().unwrap();
        mat_full_4(s);
        sum.iter_mut().zip(s.iter()).for_each(|(sum, s)| *sum += s);
    });
    state.chunks_exact_mut(4).for_each(|s| {
        let s: &mut [Fr; 4] = s.try_into().unwrap();
        s.iter_mut().zip(sum.iter()).for_each(|(s, sum)| *s += sum);
    });
}

#[cfg(test)]
mod test {
    use {
        super::*,
        ark_ff::{BigInteger, MontFp, PrimeField},
        hex,
        std::array,
    };

    #[test]
    fn assert_hardcoded_field() {
        assert_eq!(
            hex::encode(Fr::MODULUS.to_bytes_be()),
            "30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001"
        );
    }

    #[test]
    fn test_testvector_3() {
        let mut state = array::from_fn(|i| Fr::from(i as u64));
        permute_3(&mut state);
        assert_eq!(
            state,
            [
                MontFp!(
                    "5297208644449048816064511434384511824916970985131888684874823260532015509555"
                ),
                MontFp!(
                    "21816030159894113985964609355246484851575571273661473159848781012394295965040"
                ),
                MontFp!(
                    "13940986381491601233448981668101586453321811870310341844570924906201623195336"
                ),
            ]
        );
    }

    #[test]
    fn test_vector_16() {
        let mut state = array::from_fn(|i| Fr::from(i as u64));
        permute_16(&mut state);
        assert_eq!(
            state,
            [
                MontFp!(
                    "7129053404014098913941583447102076532611276040718594073862066403012892177215"
                ),
                MontFp!(
                    "5458683216916715697310099658604278457911373519210593239261146303695981710820"
                ),
                MontFp!(
                    "11764907654416682971926471140388165312909351793032868507449176373009888376893"
                ),
                MontFp!(
                    "17363012907147515824232626923071954964539976031233523938322583063167173991942"
                ),
                MontFp!(
                    "16754602647566413012759386310550362661092317428428132757066277153406453157400"
                ),
                MontFp!(
                    "10442131742273378767812305849732860137449534508695657144865044457198204305243"
                ),
                MontFp!(
                    "13315916208806700309353847107954103794241355430909228633658159683794835480566"
                ),
                MontFp!(
                    "14675611827802190925530581036356245293764500457751312643178429199155385431971"
                ),
                MontFp!(
                    "3800671750689110886099899395588427301982955036566905831860793275457528754896"
                ),
                MontFp!(
                    "863058427093450397617252284543198432424871511785791089866952153042503171268"
                ),
                MontFp!(
                    "16110421480974327191214802248220528120081914075253666769021797524181818259452"
                ),
                MontFp!(
                    "3050248777345249982082587219460801555485024010345812479213241978893548171998"
                ),
                MontFp!(
                    "8005144369031495385854140476761376792991595443174132540148616210767138457404"
                ),
                MontFp!(
                    "193712991007063517677674367979478243863141973963118958643316643360558925992"
                ),
                MontFp!(
                    "6765341258738133397733055933640609905610288576122407133007925535267189590216"
                ),
                MontFp!(
                    "6411743912316957490668095751870764077217660758836562678571866082387292213586"
                ),
            ]
        );
    }

    #[test]
    fn test_vector_compress_100() {
        let input: [Fr; 100] = array::from_fn(|i| Fr::from(i as u64));
        assert_eq!(
            compress(&input),
            MontFp!("3816407482473555807302815129307981188409786394446515165748843258596324773823"),
        )
    }

    #[test]
    fn test_vector_compress_10000() {
        let input: [Fr; 10_000] = array::from_fn(|i| Fr::from(i as u64));
        assert_eq!(
            compress(&input),
            MontFp!("676003899498022687474120528382216106663447834229062399087364023543648939619"),
        )
    }
}
