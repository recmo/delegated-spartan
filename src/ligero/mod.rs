use {
    ark_bn254::Fr,
    ark_poly::{EvaluationDomain, Radix2EvaluationDomain},
};

/// Reed-Solomon encoding
/// Rate is `x.len()/e.len()`
pub fn encode(x: &[Fr], e: &mut [Fr]) {}

pub fn commit_vec(_x: &[Fr]) -> Fr {
    todo!()
}
