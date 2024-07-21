use {
    crate::{transcript, ProverTranscript, VerifierTranscript},
    ark_bn254::Fr,
    ark_ff::{One, Zero},
    rayon,
};

/// Evaluates a multilinear extension at a point (parallel version).
pub fn par_eval_mle(coefficients: &[Fr], eval: &[Fr]) -> Fr {
    const PAR_THRESHOLD: usize = 10;
    debug_assert_eq!(coefficients.len(), 1 << eval.len());
    if eval.len() < PAR_THRESHOLD {
        eval_mle(coefficients, eval)
    } else {
        let (&x, tail) = eval.split_first().unwrap(); // Eval is non-empty
        let (c0, c1) = coefficients.split_at(coefficients.len() / 2);
        let (e0, e1) = rayon::join(|| par_eval_mle(c0, tail), || par_eval_mle(c1, tail));
        (Fr::one() - x) * e0 + x * e1
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

/// Updates f(x, x') -> f(r, x') and returns f
fn update(f: &mut [Fr], r: Fr) -> &mut [Fr] {
    let (a, b) = f.split_at_mut(f.len() / 2);
    a.iter_mut().zip(b).for_each(|(a, b)| *a += r * (*b - *a));
    a
}

/// Prove sumcheck for $\sum_x f(x)$.
/// Returns $(e, r)$ for reduced claim $e = f(r)$.
// TODO: This is destructive on coefficients, but only overwrites first half.
// We can restore the original requires n/2 space.
pub fn prove_sumcheck(
    transcript: &mut ProverTranscript,
    size: usize,
    mut f: &mut [Fr],
    mut sum: Fr,
) -> (Fr, Vec<Fr>) {
    assert_eq!(f.len(), 1 << size);
    let mut rs = Vec::with_capacity(size);
    for _ in 0..size {
        // Compute $p(x) = \sum_y f(x, y) = eq(x, 0) \sum_y f(0, y) + eq(x, 1) \sum_y f(1, y)$
        let f0y = f.iter().take(f.len() / 2).sum();
        let f1y = f.iter().skip(f.len() / 2).sum();
        assert_eq!(f0y + f1y, sum); // TODO: Exploit redudancy to avoid computations.

        // Send $p(0)$ and $p(1)$ to verifier.
        // Note: This is not ZK.
        transcript.write(f0y);
        transcript.write(f1y);
        let r = transcript.read();
        rs.push(r);

        // TODO: Fold update with sum computation.
        f = update(f, r);
        // sum = p(r) = eq(r, 0) \sum_y f(0, y) + eq(r, 1) \sum_y f(1, y)
        sum = f0y + r * (f1y - f0y);
    }
    (f[0], rs)
}

/// Verify sumcheck for $e = \sum_x f(x)$.
/// Returns $(e, r)$ for reduced claim $e = f(r)$.
pub fn verify_sumcheck(
    transcript: &mut VerifierTranscript,
    size: usize,
    mut e: Fr,
) -> (Fr, Vec<Fr>) {
    let mut rs = Vec::with_capacity(size);
    for _ in 0..size {
        let e0 = transcript.read();
        let e1 = transcript.read();
        if e != e0 + e1 {
            panic!("Sumcheck failed");
        }
        let r = transcript.generate();
        rs.push(r);
        e = r * (e1 - e0) + e0;
    }
    (e, rs)
}

/// Prove sumcheck for $\sum_x f(x) ⋅ g(x)$.
/// Returns $(e, r)$ for reduced claim $e = f(r)⋅ g(r)$.
pub fn prove_sumcheck_product(
    transcript: &mut ProverTranscript,
    size: usize,
    mut f: &mut [Fr],
    mut g: &mut [Fr],
    mut sum: Fr,
) -> (Fr, Vec<Fr>) {
    assert_eq!(f.len(), 1 << size);
    assert_eq!(g.len(), 1 << size);
    let mut rs = Vec::with_capacity(size);
    for _ in 0..size {
        let (f0, f1) = f.split_at(f.len() / 2);
        let (g0, g1) = g.split_at(g.len() / 2);
        let p00: Fr = f0.iter().zip(g0.iter()).map(|(f, g)| f * g).sum();
        let p01: Fr = f0.iter().zip(g1.iter()).map(|(f, g)| f * g).sum();
        let p10: Fr = f1.iter().zip(g0.iter()).map(|(f, g)| f * g).sum();
        let p11: Fr = f1.iter().zip(g1.iter()).map(|(f, g)| f * g).sum();
        // p(x) = p0 + p1 ⋅ x + p2 ⋅ x^2
        let p0 = p00;
        let p1 = p01 + p10 - p00 - p00;
        let p2 = p00 + p11 - p01 - p10;
        // p(0) + p(1) = 2 ⋅ p0 + p1 + p2
        assert_eq!(p0 + p0 + p1 + p2, sum);
        transcript.write(p0);
        transcript.write(p1);
        transcript.write(p2);
        let r = transcript.read();
        rs.push(r);
        f = update(f, r);
        g = update(g, r);
        sum = p0 + r * (p1 + r * p2);
    }
    (sum, rs)
}

pub fn verify_sumcheck_quadratic(
    transcript: &mut VerifierTranscript,
    size: usize,
    mut e: Fr,
) -> (Fr, Vec<Fr>) {
    let mut rs = Vec::with_capacity(size);
    for i in 0..size {
        let p0 = transcript.read();
        let p1 = transcript.read();
        let p2 = transcript.read();
        if e != p0 + p0 + p1 + p2 {
            panic!("Sumcheck failed at step {i}");
        }
        let r = transcript.generate();
        rs.push(r);
        e = p0 + r * (p1 + r * p2);
    }
    (e, rs)
}

/// Sumcheck for $\sum_x e(x) ⋅ (a(x) ⋅ b(x) - c(x))$.
/// Returns $(e, r)$ for reduced claim $e = e(r) ⋅ (a(r) ⋅ b(r) - c(r))$.
pub fn prove_sumcheck_r1cs(
    transcript: &mut ProverTranscript,
    size: usize,
    mut e: &mut [Fr],
    mut a: &mut [Fr],
    mut b: &mut [Fr],
    mut c: &mut [Fr],
) {
}

#[cfg(test)]
mod test {
    use {
        super::*,
        rand::{Rng, SeedableRng},
        rand_chacha::ChaCha20Rng,
    };

    #[test]
    fn test_eval_mle_1() {
        // https://github.com/microsoft/Nova/blob/d2c52bd73e6a91c20f23ae4971f24ad70a9d0395/src/spartan/polys/multilinear.rs#L181C1-L206C1
        let f = [0, 0, 0, 1, 0, 1, 0, 2]
            .into_iter()
            .map(Fr::from)
            .collect::<Box<[_]>>();
        let e = [1, 1, 1].into_iter().map(Fr::from).collect::<Box<[_]>>();
        let r = Fr::from(2);
        assert_eq!(eval_mle(&f, &e), r)
    }

    #[test]
    fn test_eval_mle_2() {
        // https://github.com/microsoft/Nova/blob/d2c52bd73e6a91c20f23ae4971f24ad70a9d0395/src/spartan/polys/multilinear.rs#L259-L270
        let f = [Fr::from(8); 4];
        let e = [4, 3].into_iter().map(Fr::from).collect::<Box<[_]>>();
        let r = Fr::from(8);
        assert_eq!(eval_mle(&f, &e), r)
    }

    #[test]
    fn test_sumcheck() {
        let size = 10;
        let mut rng = ChaCha20Rng::from_entropy();
        let f = (0..1 << size).map(|_| rng.gen()).collect::<Vec<Fr>>();
        let s = f.iter().sum();

        // Prove
        let mut transcript = ProverTranscript::new();
        transcript.write(s);
        let mut copy = f.clone();
        let (e, rs) = prove_sumcheck(&mut transcript, size, &mut copy, s);
        assert_eq!(eval_mle(&f, &rs), e);
        let proof = transcript.finish();
        dbg!(proof.len() * std::mem::size_of::<Fr>());

        // Verify
        let mut transcript = VerifierTranscript::new(&proof);
        let e = transcript.read();
        let (e, rs) = verify_sumcheck(&mut transcript, size, e);
        assert_eq!(eval_mle(&f, &rs), e);
    }

    #[test]
    fn test_sumcheck_product() {
        let size = 10;
        let mut rng = ChaCha20Rng::from_entropy();
        let f = (0..1 << size).map(|_| rng.gen()).collect::<Vec<Fr>>();
        let g = (0..1 << size).map(|_| rng.gen()).collect::<Vec<Fr>>();
        let s = f.iter().zip(g.iter()).map(|(f, g)| f * g).sum();

        // Prove
        let mut transcript = ProverTranscript::new();
        transcript.write(s);
        let mut fc = f.clone();
        let mut gc = g.clone();
        let (e, rs) = prove_sumcheck_product(&mut transcript, size, &mut fc, &mut gc, s);
        assert_eq!(eval_mle(&f, &rs) * eval_mle(&g, &rs), e);
        let proof = transcript.finish();
        dbg!(proof.len() * std::mem::size_of::<Fr>());

        // Verify
        let mut transcript = VerifierTranscript::new(&proof);
        let vs = transcript.read();
        assert_eq!(vs, s);
        let (ve, vrs) = verify_sumcheck_quadratic(&mut transcript, size, s);
        assert_eq!(ve, e);
        assert_eq!(vrs, rs);
    }
}
