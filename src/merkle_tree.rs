use {
    crate::{
        poseidon::compress,
        transcript::{Prover, Verifier},
    },
    ark_bn254::Fr,
    ark_ff::Field,
    std::iter::repeat,
};

// Compute 16-ary merkle tree over given leaves.
pub fn merkle_tree(leaves: Vec<Fr>) -> Vec<Vec<Fr>> {
    let mut tree: Vec<Vec<Fr>> = vec![leaves];
    loop {
        let leaves = &tree.last().unwrap();
        if leaves.len() == 1 {
            break;
        }
        let layer = leaves.chunks(16).map(compress).collect();
        tree.push(layer);
    }
    tree
}

pub fn prove(transcript: &mut Prover, tree: &[Vec<Fr>], mut index: usize) {
    for layer in tree.iter() {
        if layer.len() == 1 {
            break;
        }
        let family = layer.chunks(16).nth(index / 16).unwrap_or_default();
        if family.len() < index % 16 {
            panic!("Invalid index");
        }
        // Zero-pad the family to 16 elements and reveal siblings.
        let family = family.iter().copied().chain(repeat(Fr::ZERO)).take(16);
        let siblings = family.enumerate().filter(|(i, _)| *i != index % 16);
        siblings.for_each(|(_, sibling)| transcript.reveal(sibling));
        index /= 16;
    }
}

pub fn verify(_transcript: &mut Verifier, _leaves: Fr, _index: usize, _root: &[Fr]) {}
