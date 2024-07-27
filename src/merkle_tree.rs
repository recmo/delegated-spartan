use {
    crate::{
        poseidon::compress,
        transcript::{Prover, Verifier},
    },
    ark_bn254::Fr,
    ark_ff::Field,
    std::{array, iter::repeat},
};

// TODO: Determine optimal arity for proof-size / verifier complexity trade-off.
const ARITY: usize = 16;

// Compute Merkle tree over given leaves.
pub fn merkle_tree(leaves: Vec<Fr>) -> Vec<Vec<Fr>> {
    let mut tree: Vec<Vec<Fr>> = vec![leaves];
    loop {
        let leaves = &tree.last().unwrap();
        if leaves.len() == 1 {
            break;
        }
        let layer = leaves.chunks(ARITY).map(compress).collect();
        tree.push(layer);
    }
    tree
}

pub fn prove(transcript: &mut Prover, tree: &[Vec<Fr>], mut index: usize) {
    for layer in tree.iter() {
        if layer.len() == 1 {
            break;
        }
        let family = layer.chunks(ARITY).nth(index / ARITY).unwrap_or_default();
        if family.len() < index % ARITY {
            panic!("Invalid index");
        }
        // Zero-pad the family and reveal siblings.
        let family = family.iter().copied().chain(repeat(Fr::ZERO)).take(ARITY);
        let siblings = family.enumerate().filter(|(i, _)| *i != index % ARITY);
        siblings.for_each(|(_, sibling)| transcript.reveal(sibling));
        index /= ARITY;
    }
}

pub fn verify(transcript: &mut Verifier, root: Fr, mut index: usize, mut leaf: Fr) {
    // TODO: Maybe pass tree size so we can terminate early on failure.
    while leaf != root {
        let family: [Fr; ARITY] = array::from_fn(|i| {
            if i == index % ARITY {
                leaf
            } else {
                transcript.reveal()
            }
        });
        leaf = compress(&family);
        index /= ARITY;
    }
}

#[cfg(test)]
mod test {
    use {super::*, std::mem::size_of};

    #[test]
    fn test_merkle_tree_10000() {
        let leafs: Vec<_> = (0..10_000).map(Fr::from).collect();
        let index = 5123;
        let leaf = leafs[index];

        // Proof
        let mut transcript = Prover::new();
        let tree = merkle_tree(leafs);
        let root = *tree.last().unwrap().first().unwrap();
        transcript.write(root);
        transcript.write(leaf);
        prove(&mut transcript, &tree, index);
        let proof = transcript.finish();
        dbg!(proof.len() * size_of::<Fr>());

        // Verify
        let mut transcript = Verifier::new(&proof);
        let vroot = transcript.read();
        let vleaf = transcript.read();
        verify(&mut transcript, vroot, index, vleaf);
    }
}
