use {
    crate::ntt::{intt, ntt},
    ark_bn254::Fr,
    ark_ff::Field,
};

/// Reed-Solomon encoding.
/// Rate is `x.len()/e.len()`.
pub fn encode(x: &mut [Fr], e: &mut [Fr]) {
    // TODO: Support arbitrary input and output length.

    // Convert x to coefficients.
    intt(x);

    // Compute coset evaluations
    let mut current_coset = Fr::ONE;
    for e in e.chunks_exact_mut(x.len()) {
        // Move to a different coset. P(X) -> P(c * X)
        let coset = Fr::from(5);
        let mut coset_i = coset;
        for x in x.iter_mut().skip(1) {
            *x *= coset_i;
            coset_i *= coset;
        }

        // Apply coset factors
        e.copy_from_slice(x);
        ntt(e);
    }
}
