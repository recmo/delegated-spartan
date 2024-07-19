use {
    crate::{transcript, ProverTranscript, VerifierTranscript},
    ark_bn254::Fr,
    ark_ff::One,
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

// TODO: This is destructive on coefficients, but only overwrites first half.
// We can restore the original requires n/2 space.
pub fn prove_sumcheck(transcript: &mut ProverTranscript, mut coefficients: &mut [Fr], size: usize) {
    assert_eq!(coefficients.len(), 1 << size);
    for _ in 0..size {
        let (c0, c1) = coefficients.split_at_mut(coefficients.len() / 2);
        transcript.write(c0.iter().sum());
        transcript.write(c1.iter().sum());
        let r = transcript.read();
        // TODO: Single pass update and compute next sums.
        // eq(r, 0) * c0 + eq(r, 1) * c1 = c0 + r * (c1 - c0)
        c0.iter_mut()
            .zip(c1.iter())
            .for_each(|(c0, c1)| *c0 += r * (*c1 - *c0));
        coefficients = c0;
    }
}

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
        let mut f = (0..1 << size).map(|_| rng.gen()).collect::<Vec<Fr>>();

        // Prove
        let mut transcript = ProverTranscript::new();
        transcript.write(f.iter().sum());
        prove_sumcheck(&mut transcript, &mut f, size);
        let proof = transcript.finish();
        dbg!(proof.len() * std::mem::size_of::<Fr>());

        // Verify
        let mut transcript = VerifierTranscript::new(&proof);
        let e = transcript.read();
        let _ = verify_sumcheck(&mut transcript, size, e);
    }
}
