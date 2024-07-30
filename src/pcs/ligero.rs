use {
    crate::{
        merkle_tree::MerkleTree,
        ntt::{intt, ntt, transpose},
        poseidon::compress,
        reed_solomon::encode,
    },
    ark_bn254::Fr,
    ark_ff::Field,
};

pub fn commit(f: &[Fr]) -> MerkleTree {
    assert!(f.len().is_power_of_two());

    // Pick rows and cols
    let rows = 1 << (f.len().trailing_zeros() / 2);
    let cols = f.len() / rows;

    // Pick expansion
    let expansion = 4;
    let ecols = cols * expansion;

    // Encode values
    let mut buffer = vec![Fr::ZERO; cols];
    let mut encoded = vec![Fr::ZERO; f.len() * 4];
    for (f, e) in f.chunks_exact(cols).zip(encoded.chunks_exact_mut(ecols)) {
        buffer.copy_from_slice(f);
        encode(&mut buffer, e);
    }

    // Hash columns and construct merkle tree.
    transpose(&mut encoded, rows, ecols);
    MerkleTree::new(encoded.chunks_exact(rows).map(compress).collect())
}
