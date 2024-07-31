use {
    crate::ntt::{intt, ntt},
    ark_bn254::Fr,
    ark_ff::Field,
};

/// Reed-Solomon encoding.
/// Rate is `x.len()/e.len()`.
pub fn encode(m: &mut [Fr], c: &mut [Fr]) {
    // TODO: Support arbitrary input and output length.

    // Convert x to coefficients.
    // TODO: Is this necessary?
    intt(m);

    // Compute coset evaluations
    let mut current_coset = Fr::ONE;
    for c in c.chunks_exact_mut(m.len()) {
        // Move to a different coset. P(X) -> P(c * X)
        let coset = Fr::from(5);
        let mut coset_i = coset;
        for m in m.iter_mut().skip(1) {
            *m *= coset_i;
            coset_i *= coset;
        }

        // Apply coset factors
        c.copy_from_slice(m);
        ntt(c);
    }
}

/// Fold the codeword using a random factor.
pub fn fold(c: &mut [Fr], degree: usize, r: &Fr) {
    assert_eq!(degree, 2);
    assert_eq!(c.len() % degree, 0);

    let (a, b) = c.split_at_mut(c.len() / 2);
    a.iter_mut().zip(b.iter()).for_each(|(a, b)| *a += r * b);
}
