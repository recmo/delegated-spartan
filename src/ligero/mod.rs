use {
    ark_bn254::Fr,
    ark_poly::{
        polynomial::{univariate::DensePolynomial, DenseUVPolynomial},
        EvaluationDomain, Radix2EvaluationDomain,
    },
};

/// Reed-Solomon encoding
/// Rate is x.len()/e.len()
pub fn encode(x: &[Fr], e: &mut [Fr]) {
    assert_eq!(
        e.len() % x.len(),
        0,
        "e.len() must be a multiple of x.len()"
    );
    let domain = Radix2EvaluationDomain::<Fr>::new(x.len()).unwrap();
    let coefficients = domain.fft(&x);
}

pub fn commit_vec(x: &[Fr]) -> Fr {
    todo!()
}
