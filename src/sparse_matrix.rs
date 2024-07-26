use ark_bn254::Fr;

pub struct SparseMatrix {
    pub rows: usize,
    pub cols: usize,
    pub entries: Vec<(usize, Fr)>,
}

impl SparseMatrix {
    /// Compute Self ⋅ v
    pub fn mul_left(&self, v: &[Fr]) -> Vec<Fr> {
        assert_eq!(v.len(), self.cols);
        let mut res = vec![Fr::zero(); self.rows];
        for (i, val) in &self.entries {
            res[i / self.cols] += val * v[i % self.cols];
        }
        res
    }
}
