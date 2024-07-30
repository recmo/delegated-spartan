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
    self::constants::{RC16, RC3},
    ark_bn254::Fr,
    ark_ff::Field,
    std::sync::atomic::{AtomicU32, Ordering},
};

pub static COUNT_3: AtomicU32 = AtomicU32::new(0);
pub static COUNT_16: AtomicU32 = AtomicU32::new(0);

// Compress arbitrary length inputs.
// Compute 16-arry Merkle tree over input.
// Layers are zero padded.
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
    COUNT_3.fetch_add(1, Ordering::Relaxed);
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

// OPT: Time spend: 53% in `mat_partial_16`, 31% in x^5, 11% in `mat_full_16`.
pub fn permute_16(state: &mut [Fr; 16]) {
    COUNT_16.fetch_add(1, Ordering::Relaxed);
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
        mat_partial_16(state);
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

/// Computes a 16x16 partial matrix.
/// These are not MDS, but meet requirements set out in Poseidon2 paper.
/// Ones + Diag [-8, -7, -6, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 8, 9]
// OPT: This would benefit from delayed reduction. If we do this using
// additional bits in each limb, say base 2^43 over [u64; 6], we can do
// ~six rounds before needing to propagate carries and reductions. These
// can then be done in SIMD.
pub fn mat_partial_16(state: &mut [Fr; 16]) {
    let sum: Fr = state.iter().sum();

    // -8
    state[0]
        .neg_in_place()
        .double_in_place()
        .double_in_place()
        .double_in_place();
    // -7
    state[1] -= state[1].double().double_in_place().double_in_place();
    // -6
    state[2].neg_in_place().double_in_place();
    state[2] += state[2].double();
    // -4
    state[3].neg_in_place().double_in_place().double_in_place();
    // -3
    state[4].neg_in_place();
    state[4] += state[4].double();
    // -2
    state[5].neg_in_place().double_in_place();
    // -1
    state[6].neg_in_place();
    // 0
    state[7] = Fr::ZERO;
    // 1
    // 2
    state[9].double_in_place();
    // 3
    state[10] += state[10].double();
    // 4
    state[11].double_in_place().double_in_place();
    // 5
    state[12] += state[12].double().double_in_place();
    // 6
    state[13].double_in_place();
    state[13] += state[13].double();
    // 8
    state[14]
        .double_in_place()
        .double_in_place()
        .double_in_place();
    // 9
    state[15] += state[15].double().double_in_place().double_in_place();

    state.iter_mut().for_each(|s| *s += sum);
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
                    "21826543399356400550661234518745697083314121792488689649921600144027529528864"
                ),
                MontFp!(
                    "1136614509231762972747684409955833585795356890395544147561079338856329844882"
                ),
                MontFp!(
                    "7846203944249237147415415937111974637559001966398790080552410754200558542174"
                ),
                MontFp!(
                    "20373295329607786911155447976406333283890521720422897656483348909518994545302"
                ),
                MontFp!(
                    "10248258064547453812040141589882860977603883038550485389282160470216447363165"
                ),
                MontFp!(
                    "12098580505374776657887220870209868366964886865846185818028508340624519625074"
                ),
                MontFp!(
                    "21657356642314632644205483275755543092354720301692787626365690180578446876738"
                ),
                MontFp!(
                    "6478671057384004322304903741220876585829134366815215569349583061873769419284"
                ),
                MontFp!(
                    "9309480533852668150139011847800666441744286725509447134677492726769953022832"
                ),
                MontFp!(
                    "5985411547970865813552103081855308627655089133168907392759728995974124438178"
                ),
                MontFp!(
                    "21301812487058380531079694312167712492008196576067867213781569387743743843182"
                ),
                MontFp!(
                    "1763211660252669208569231157523230802504062806987793786534192016715027324977"
                ),
                MontFp!(
                    "21553224731557588082645730283660340925372392244504971109792566303605003960515"
                ),
                MontFp!(
                    "7999909471375171029299048969340621865761306943552761985378629049678432780509"
                ),
                MontFp!(
                    "9302130865923327285820247173837807967532413594680096813945954902727336131357"
                ),
                MontFp!(
                    "2154283930331108770651074355844169723002810959432355459893611637953760401958"
                ),
            ]
        );
    }

    #[test]
    fn test_vector_compress_100() {
        let input: [Fr; 100] = array::from_fn(|i| Fr::from(i as u64));
        assert_eq!(
            compress(&input),
            MontFp!("2665897937932724574846245531196671064769192363661323320407720095098758813730"),
        )
    }

    #[test]
    fn test_vector_compress_10000() {
        let input: [Fr; 10_000] = array::from_fn(|i| Fr::from(i as u64));
        assert_eq!(
            compress(&input),
            MontFp!(
                "13856578634472258607721631967114693152972466009291698529880637216466519128197"
            ),
        )
    }
}
