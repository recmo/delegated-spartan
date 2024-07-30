use {
    crate::{merkle_tree::MerkleTree, reed_solomon::encode},
    ark_bn254::Fr,
    ark_ff::Field,
};

pub fn commit(scalars: &[Fr]) -> MerkleTree {
    let mut buffer = scalars.to_vec();
    let mut codeword = vec![Fr::ZERO; 4 * scalars.len()];
    encode(&mut buffer, &mut codeword);

    MerkleTree::new(codeword)
}
