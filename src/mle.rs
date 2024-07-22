use {
    crate::{transcript, ProverTranscript, VerifierTranscript},
    ark_bn254::Fr,
    ark_ff::{Field, MontFp, One, Zero},
    itertools::izip,
    rayon,
    std::array,
};

const HALF: Fr =
    MontFp!("10944121435919637611123202872628637544274182200208017171849102093287904247809");

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
        // Send p(x) = p0 + p1 ⋅ x to verifier
        let p0 = f.iter().take(f.len() / 2).sum();
        let p1 = sum - p0 - p0;
        transcript.write(p0);
        transcript.write(p1);
        let r = transcript.read();
        rs.push(r);

        // TODO: Fold update with sum computation.
        f = update(f, r);
        // sum = p(r) = eq(r, 0) \sum_y f(0, y) + eq(r, 1) \sum_y f(1, y)
        sum = p0 + r * p1;
    }
    (f[0], rs)
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
        // p(x) = p0 + p1 ⋅ x + p2 ⋅ x^2
        let mut p0 = Fr::zero();
        let mut p2 = Fr::zero();
        let (f0, f1) = f.split_at(f.len() / 2);
        let (g0, g1) = g.split_at(g.len() / 2);
        f0.iter()
            .zip(f1)
            .zip(g0.iter().zip(g1))
            .for_each(|((f0, f1), (g0, g1))| {
                // Evaluation at 0
                p0 += f0 * g0;
                // Evaluation at ∞
                p2 += (f1 - f0) * (g1 - g0);
            });
        // sum = p(0) + p(1) = 2 ⋅ p0 + p1 + p2
        let p1 = sum - p0 - p0 - p2;
        transcript.write(p0);
        transcript.write(p1);
        transcript.write(p2);
        let r = transcript.read();
        rs.push(r);
        f = update(f, r);
        g = update(g, r);
        // sum = p(r)
        sum = p0 + r * (p1 + r * p2);
    }
    (sum, rs)
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
    mut sum: Fr,
) -> (Fr, Vec<Fr>) {
    assert_eq!(e.len(), 1 << size);
    assert_eq!(a.len(), 1 << size);
    assert_eq!(b.len(), 1 << size);
    assert_eq!(c.len(), 1 << size);
    let mut rs = Vec::with_capacity(size);
    for _ in 0..size {
        // p(x) = p0 + p1 ⋅ x + p2 ⋅ x^2 + p3 ⋅ x^3
        let mut p0 = Fr::zero();
        let mut p2 = Fr::zero();
        let mut p3 = Fr::zero();
        let (e0, e1) = e.split_at(e.len() / 2);
        let (a0, a1) = a.split_at(a.len() / 2);
        let (b0, b1) = b.split_at(b.len() / 2);
        let (c0, c1) = c.split_at(c.len() / 2);
        izip!(
            e0.iter().zip(e1),
            a0.iter().zip(a1),
            b0.iter().zip(b1),
            c0.iter().zip(c1)
        )
        .for_each(|(e, a, b, c)| {
            // Evaluation at 0
            p0 += *e.0 * (a.0 * b.0 - c.0);
            // Evaluation at -1
            p2 += (e.0 + e.0 - e.1) * ((a.0 + a.0 - a.1) * (b.0 + b.0 - b.1) - (c.0 + c.0 - c.1));
            // Evaluation at ∞
            p3 += (e.1 - e.0) * (a.1 - a.0) * (b.1 - b.0);
        });
        // p(-1)             =     p0 - p1 + p2 - p3
        // sum = p(0) + p(1) = 2 ⋅ p0 + p1 + p2 + p3
        let p1 = sum - p0 - p0 - p2 - p3;
        transcript.write(p0);
        transcript.write(p1);
        transcript.write(p2);
        let r = transcript.read();
        rs.push(r);
        e = update(e, r);
        a = update(a, r);
        b = update(b, r);
        c = update(c, r);
        // sum = p(r)
        sum = p0 + r * (p1 + r * p2);
    }
    (sum, rs)
}

/// Verify sumcheck for $N$-term polynomials.
/// I.e. N = 2 for linear, 3 for quadratic, etc.
pub fn verify_sumcheck<const N: usize>(
    transcript: &mut VerifierTranscript,
    size: usize,
    mut e: Fr,
) -> (Fr, Vec<Fr>) {
    let mut rs = Vec::with_capacity(size);
    for i in 0..size {
        let p: [Fr; N] = array::from_fn(|_| transcript.read());
        // Check p'(r) = p(0) + p(1)
        if e != p[0] + p.iter().sum::<Fr>() {
            panic!("Sumcheck failed at step {i}");
        }
        assert_eq!(p[0], e - p.iter().sum::<Fr>());
        let r = transcript.generate();
        rs.push(r);
        e = p
            .into_iter()
            .rev()
            .reduce(|acc, p| p + r * acc)
            .expect("p not empty");
    }
    (e, rs)
}

#[cfg(test)]
mod test {
    use {
        super::*,
        rand::{Rng, SeedableRng},
        rand_chacha::ChaCha20Rng,
    };

    #[test]
    fn test_half() {
        assert_eq!(HALF.double(), Fr::ONE);
    }

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
        let (e, rs) = verify_sumcheck::<2>(&mut transcript, size, e);
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
        let (ve, vrs) = verify_sumcheck::<3>(&mut transcript, size, s);
        assert_eq!(ve, e);
        assert_eq!(vrs, rs);
    }
}
