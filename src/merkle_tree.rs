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

pub struct MerkleTree(Vec<Vec<Fr>>);

/// TODO: Blinding?
impl MerkleTree {
    pub fn new(leaves: Vec<Fr>) -> Self {
        // TODO: Maybe flatten the tree to a single vector?
        let mut tree: Vec<Vec<Fr>> = vec![leaves];
        loop {
            let leaves = &tree.last().unwrap();
            if leaves.len() == 1 {
                break;
            }
            let layer = leaves.chunks(ARITY).map(compress).collect();
            tree.push(layer);
        }
        Self(tree)
    }

    pub fn root(&self) -> Fr {
        self.0.last().unwrap().first().copied().unwrap()
    }

    pub fn leaves(&self) -> &[Fr] {
        self.0.first().unwrap()
    }

    pub fn reveal(&self, transcript: &mut Prover, mut index: usize) {
        for layer in self.0.iter() {
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
    fn test_merkle_tree_1000() {
        let leafs: Vec<_> = (0..1000).map(Fr::from).collect();
        let index = 531;
        let leaf = leafs[index];

        // Proof
        let mut transcript = Prover::new();
        let tree = MerkleTree::new(leafs);
        transcript.write(tree.root());
        transcript.write(tree.leaves()[index]);
        tree.reveal(&mut transcript, index);
        let proof = transcript.finish();
        dbg!(proof.len() * size_of::<Fr>());

        // Verify
        let mut transcript = Verifier::new(&proof);
        let vroot = transcript.read();
        let vleaf = transcript.read();
        assert_eq!(vroot, tree.root());
        assert_eq!(vleaf, leaf);
        verify(&mut transcript, vroot, index, vleaf);
    }
}
