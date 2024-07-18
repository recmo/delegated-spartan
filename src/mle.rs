use {ark_bn254::Fr, ark_ff::One, rayon};

/// Evaluates a multilinear extension at a point (parallel version).
pub fn par_eval_mle(coefficients: &[Fr], eval: &[Fr]) -> Fr {
    const PAR_THRESHOLD: usize = 10;
    debug_assert_eq!(coefficients.len(), 1 << eval.len());
    if eval.len() >= PAR_THRESHOLD {
        let (&x, tail) = eval.split_first().unwrap(); // Eval is non-empty
        let (c0, c1) = coefficients.split_at(coefficients.len() / 2);
        let (e0, e1) = rayon::join(|| par_eval_mle(c0, tail), || par_eval_mle(c1, tail));
        (Fr::one() - x) * e0 + x * e1
    } else {
        eval_mle(coefficients, eval)
    }
}

/// Evaluates a multilinear extension at a point.
/// Uses a cache-oblivious recursive algorithm.
pub fn eval_mle(coefficients: &[Fr], eval: &[Fr]) -> Fr {
    debug_assert_eq!(coefficients.len(), 1 << eval.len());
    if let Some((&x, tail)) = eval.split_first() {
        let (c0, c1) = coefficients.split_at(coefficients.len() / 2);
        (Fr::one() - x) * eval_mle(c0, tail) + x * eval_mle(c1, tail)
    } else {
        return coefficients[0];
    }
}
