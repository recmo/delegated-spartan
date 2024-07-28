use {
    ark_bn254::Fr,
    ark_ff::{FftField, Field, MontFp},
    std::{iter, sync::RwLock},
};

/// Hardcoded roots of unity
/// Taking 5 as the generator of the multilicative group.
const OMEGA_4_1: Fr =
    MontFp!("21888242871839275217838484774961031246007050428528088939761107053157389710902");
const HALF_OMEGA_3_1_PLUS_2: Fr =
    MontFp!("10944121435919637611123202872628637544274182200208017171849102093287904247808");
const HALF_OMEGA_3_1_MIN_2: Fr =
    MontFp!("10944121435919637615531123842924881386667549415214173256765571550433748226270");
const OMEGA_2415919104: Fr =
    MontFp!("8001236115608269688640730372558895144313937963023562728862538587154079436142");

/// Cache of precomputed roots of unity.
static ROOTS: RwLock<Vec<Fr>> = RwLock::new(Vec::new());

pub fn ntt(values: &mut [Fr]) {
    // Precompute roots of unity if necessary.
    let roots = ROOTS.read().unwrap();
    let roots = if roots.len() < values.len() || roots.len() % values.len() != 0 {
        // Obtain write lock to update the cache.
        drop(roots);
        let mut roots = ROOTS.write().unwrap();

        // Minimal size to support all sizes seen so far.
        let size = if roots.is_empty() {
            values.len()
        } else {
            lcm(roots.len(), values.len())
        };
        roots.clear();
        roots.reserve_exact(size);

        // Compute powers of roots of unity.
        let root = root(size).unwrap();
        let mut root_i = Fr::ONE;
        while roots.len() < size {
            roots.push(root_i);
            root_i *= root;
        }

        // Back to read lock.
        drop(roots);
        ROOTS.read().unwrap()
    } else {
        roots
    };

    ntt_batch_inner(values, &roots, values.len());
}

fn ntt_batch_inner(values: &mut [Fr], roots: &[Fr], size: usize) {
    debug_assert_eq!(values.len() % size, 0);
    debug_assert!(roots.len() >= values.len());
    debug_assert_eq!(roots.len() % values.len(), 0);

    match size {
        0 | 1 => {}
        2 => {
            for v in values.chunks_exact_mut(2) {
                (v[0], v[1]) = (v[0] + v[1], v[0] - v[1]);
            }
        }
        3 => {
            for v in values.chunks_exact_mut(3) {
                // Rader NTT:
                let v0 = v[0];
                (v[1], v[2]) = (v[1] + v[2], v[1] - v[2]);
                v[0] += v[1];
                v[1] *= HALF_OMEGA_3_1_PLUS_2;
                v[2] *= HALF_OMEGA_3_1_MIN_2;
                v[1] += v0;
                (v[1], v[2]) = (v[1] + v[2], v[1] - v[2]);
            }
        }
        4 => {
            for v in values.chunks_exact_mut(4) {
                (v[0], v[2]) = (v[0] + v[2], v[0] - v[2]);
                (v[1], v[3]) = (v[1] + v[3], v[1] - v[3]);
                v[3] *= OMEGA_4_1;
                (v[0], v[1]) = (v[0] + v[1], v[0] - v[1]);
                (v[2], v[3]) = (v[2] + v[3], v[2] - v[3]);
                (v[1], v[2]) = (v[2], v[1]);
            }
        }
        n => {
            let n1 = sqrt_factor(size);
            let n2 = size / n1;
            let step = roots.len() / size;
            for values in values.chunks_exact_mut(size) {
                transpose(values, n1, n2);
                ntt_batch_inner(values, roots, n1);
                transpose(values, n2, n1);

                for i in 1..n1 {
                    let step = (i * step) % roots.len();
                    let mut index = step;
                    for j in 1..n2 {
                        index %= roots.len();
                        values[i * n2 + j] *= roots[index];
                        index += step;
                    }
                }

                ntt_batch_inner(values, roots, n2);
                transpose(values, n1, n2);
            }
        }
    }
}

fn transpose<T: Copy>(matrix: &mut [T], rows: usize, cols: usize) {
    debug_assert_eq!(matrix.len(), rows * cols);
    if rows == cols {
        for i in 0..rows {
            for j in (i + 1)..cols {
                matrix.swap(i * cols + j, j * rows + i);
            }
        }
    } else {
        let mut copy = matrix.to_vec();
        for i in 0..rows {
            for j in 0..cols {
                matrix[j * rows + i] = copy[i * cols + j];
            }
        }
    }
}

pub fn intt(values: &mut [Fr]) {
    let s = Fr::from(values.len() as u64).inverse().unwrap();
    values.iter_mut().for_each(|v| *v *= s);
    values[1..].reverse();
    ntt(values);
}

// Compute a root of unity of the given order.
// Fr^* is of order 2^28 * 3^2 * 13 * 29 * 983 * 11003 * 237073 * 405928799 * 1670836401704629 * 13818364434197438864469338081
// 2^28 * 3^2 = 2415919104
// TODO: 13 = 2^2 * 3^1 + 1 is a good candidate for Rader NTT.
fn root(order: usize) -> Option<Fr> {
    if 2415919104 % order == 0 {
        Some(OMEGA_2415919104.pow([(2415919104 / order) as u64]))
    } else {
        None
    }
}

// Compute a factor of n that is close to sqrt(n).
fn sqrt_factor(n: usize) -> usize {
    // TODO: Support non-powers-of-two.
    1 << (n.trailing_zeros() / 2)
}

fn lcm(a: usize, b: usize) -> usize {
    a * b / gcd(a, b)
}

fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        (a, b) = (b, a % b);
    }
    a
}

#[cfg(test)]
mod test {
    use {super::*, std::array};

    // O(n^2) Reference implementation
    pub fn ntt_ref(values: &mut [Fr]) {
        let Some(root) = root(values.len()) else {
            panic!("Root of unity not found for length {}", values.len());
        };
        let mut result = Vec::with_capacity(values.len());
        let mut root_i = Fr::ONE;
        for _i in 0..values.len() {
            let mut root_ij = Fr::ONE;
            let mut sum = Fr::ZERO;
            for &v in values.iter() {
                sum += v * root_ij;
                root_ij *= root_i;
            }
            result.push(sum);
            root_i *= root;
        }
        values.copy_from_slice(&result);
    }

    #[test]
    #[rustfmt::skip]
    fn test_transpose() {
        let mut values: [u8; 6] = [
            0, 1, 2,
            3, 4, 5
        ];
        transpose(&mut values, 2, 3);
        assert_eq!(values, [
            0, 3,
            1, 4,
            2, 5,
        ]);
    }

    #[test]
    fn test_roots() {
        // Ark-BN254 only supports powers of two.
        for size in [1, 2, 4, 8, 16, 32, 64] {
            assert_eq!(root(size), Fr::get_root_of_unity(size as u64));
        }
        assert_eq!(root(2415919104).unwrap(), OMEGA_2415919104);
        assert_eq!(root(4).unwrap(), OMEGA_4_1);
    }

    #[test]
    fn test_ntt_ref() {
        for size in [1, 2, 3, 4, 8, 16, 32, 64, 128, 256, 512, 1024] {
            let mut values: Vec<Fr> = (0..size).map(|i| Fr::from(i as u64)).collect();
            let mut expected = values.clone();
            ntt(&mut values);
            ntt_ref(&mut expected);
            assert_eq!(values, expected);
        }
    }

    #[test]
    fn test_ntt_intt() {
        let mut values: [Fr; 1024] = array::from_fn(|i| Fr::from(i as u64));
        let expected = values;
        ntt_ref(&mut values);
        intt(&mut values);
        assert_eq!(values, expected);
    }
}
