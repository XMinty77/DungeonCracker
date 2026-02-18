use crate::math::big_fraction::BigFraction;
use crate::math::big_matrix::BigMatrix;
use crate::math::big_vector::BigVector;
use num_bigint::BigInt;
use num_traits::Zero;

/// LLL lattice basis reduction parameters.
/// Faithful port of LattiCG's Params.java.
pub struct LLLParams {
    pub delta: BigFraction,
    pub max_stage: i32,
}

impl LLLParams {
    pub fn recommended() -> Self {
        LLLParams {
            delta: BigFraction::new(99i64, 100i64),
            max_stage: -1,
        }
    }
}

impl Default for LLLParams {
    fn default() -> Self {
        LLLParams {
            delta: BigFraction::new(75i64, 100i64),
            max_stage: -1,
        }
    }
}

/// Result of LLL reduction.
pub struct LLLResult {
    pub num_dependant_vectors: usize,
    pub reduced_basis: BigMatrix,
    pub transformations: BigMatrix,
}

/// LLL lattice basis reduction.
/// Faithful port of LattiCG's LLL.java, based on
/// Cohen's "A Course in Computational Algebraic Number Theory", page 95.
pub fn reduce(lattice: &BigMatrix, params: &LLLParams) -> LLLResult {
    let nb_rows = lattice.row_count();
    let nb_cols = lattice.col_count();

    let mut basis = lattice.clone();
    let mut base_gso = BigMatrix::new(nb_rows, nb_cols);
    let mut mu = BigMatrix::new(nb_rows, nb_rows);
    let mut norms = BigVector::new(nb_rows);
    let mut coordinates = BigMatrix::identity(nb_rows);

    // Initialize first GSO vector
    base_gso.set_row(0, &basis.get_row(0));
    norms.set(0, basis.get_row(0).magnitude_sq());

    let mut k: usize = 1;
    let mut kmax: usize = 0;
    let mut update_gso = true;
    let n = if params.max_stage == -1 { nb_rows } else { params.max_stage as usize };
    let mut iteration: u64 = 0;

    while k < n {
        iteration += 1;
        if iteration % 1000 == 0 {
            eprintln!("[lll]     iteration {}, k={}/{}", iteration, k, n);
        }
        if k > kmax && update_gso {
            kmax = k;
            update_gso_at(&basis, &mut base_gso, &mut mu, &mut norms, k);
        }

        // RED(k, k-1)
        red(&mut basis, &mut coordinates, &mut mu, k, k - 1);

        // Test LLL condition
        if test_condition(&mu, &norms, k, &params.delta) {
            // SWAP
            swapg(&mut basis, &mut coordinates, &mut base_gso, &mut mu, &mut norms, k, kmax);
            k = if k > 1 { k - 1 } else { 1 };
            update_gso = false;
        } else {
            if k >= 2 {
                for l in (0..=(k - 2)).rev() {
                    red(&mut basis, &mut coordinates, &mut mu, k, l);
                }
            }
            k += 1;
            update_gso = true;
        }
    }

    // Remove zero rows
    let p = count_zero_rows(&basis);
    if p > 0 {
        let new_rows = nb_rows - p;
        basis = basis.submatrix(p, 0, new_rows, nb_cols);
        coordinates = coordinates.submatrix(p, 0, new_rows, coordinates.col_count());
    }

    LLLResult {
        num_dependant_vectors: p,
        reduced_basis: basis,
        transformations: coordinates,
    }
}

/// Convenience: reduce with recommended delta (99/100).
pub fn reduce_default(lattice: &BigMatrix) -> LLLResult {
    reduce(lattice, &LLLParams::recommended())
}

fn count_zero_rows(basis: &BigMatrix) -> usize {
    let mut p = 0;
    for i in 0..basis.row_count() {
        if basis.get_row(i).is_zero() {
            p += 1;
        }
    }
    p
}

fn update_gso_at(
    basis: &BigMatrix,
    base_gso: &mut BigMatrix,
    mu: &mut BigMatrix,
    norms: &mut BigVector,
    k: usize,
) {
    let mut new_row = basis.get_row(k);
    for j in 0..k {
        if !norms.get(j).is_zero() {
            let mu_kj = basis.get_row(k).dot(&base_gso.get_row(j)).div_frac(norms.get(j));
            mu.set(k, j, mu_kj.clone());
            let scaled = base_gso.get_row(j).multiply_scalar(&mu_kj);
            new_row.subtract_assign(&scaled);
        } else {
            mu.set(k, j, BigFraction::zero());
        }
    }
    base_gso.set_row(k, &new_row);
    norms.set(k, new_row.magnitude_sq());
}

fn test_condition(mu: &BigMatrix, norms: &BigVector, k: usize, delta: &BigFraction) -> bool {
    let mu_temp = mu.get(k, k - 1);
    let factor = delta.sub_frac(&mu_temp.mul_frac(mu_temp));
    *norms.get(k) < norms.get(k - 1).mul_frac(&factor)
}

fn red(
    basis: &mut BigMatrix,
    coordinates: &mut BigMatrix,
    mu: &mut BigMatrix,
    i: usize,
    j: usize,
) {
    let r = mu.get(i, j).round();
    if r == BigInt::zero() {
        return;
    }

    // basis[i] -= r * basis[j]
    let row_j = basis.get_row(j).multiply_bigint(&r);
    let mut row_i = basis.get_row(i);
    row_i.subtract_assign(&row_j);
    basis.set_row(i, &row_i);

    // coordinates[i] -= r * coordinates[j]
    let coord_j = coordinates.get_row(j).multiply_bigint(&r);
    let mut coord_i = coordinates.get_row(i);
    coord_i.subtract_assign(&coord_j);
    coordinates.set_row(i, &coord_i);

    // mu[i][j] -= r
    let new_mu = mu.get(i, j).sub_frac(&BigFraction::from_bigint(r.clone()));
    mu.set(i, j, new_mu);

    for col in 0..j {
        let new_val = mu.get(i, col).sub_frac(&mu.get(j, col).mul_frac(&BigFraction::from_bigint(r.clone())));
        mu.set(i, col, new_val);
    }
}

/// SWAP subroutine - exact port of LLL.java swapg()
fn swapg(
    basis: &mut BigMatrix,
    coordinates: &mut BigMatrix,
    base_gso: &mut BigMatrix,
    mu: &mut BigMatrix,
    norms: &mut BigVector,
    k: usize,
    kmax: usize,
) {
    basis.swap_rows(k, k - 1);
    coordinates.swap_rows(k, k - 1);

    if k > 1 {
        for j in 0..=(k - 2) {
            mu.swap_elements(k, j, k - 1, j);
        }
    }

    let tmu = mu.get(k, k - 1).clone();
    let tb = norms.get(k).add_frac(&tmu.mul_frac(&tmu).mul_frac(norms.get(k - 1)));

    if tb.is_zero() {
        // Case 1: tB == 0
        norms.set(k, norms.get(k - 1).clone());
        norms.set(k - 1, BigFraction::zero());
        base_gso.swap_rows(k, k - 1);
        for i in (k + 1)..=kmax {
            mu.set(i, k, mu.get(i, k - 1).clone());
            mu.set(i, k - 1, BigFraction::zero());
        }
    } else if norms.get(k).is_zero() && !tmu.is_zero() {
        // Case 2: B[k] == 0 and tmu != 0
        norms.set(k - 1, tb);
        let row = base_gso.get_row(k - 1).multiply_scalar(&tmu);
        base_gso.set_row(k - 1, &row);
        mu.set(k, k - 1, tmu.reciprocal());
        for i in (k + 1)..=kmax {
            let val = mu.get(i, k - 1).div_frac(&tmu);
            mu.set(i, k - 1, val);
        }
    } else {
        // Case 3: normal case
        let t = norms.get(k - 1).div_frac(&tb);
        mu.set(k, k - 1, tmu.mul_frac(&t));

        // b = gso[k-1].copy()  (save before overwrite)
        let b = base_gso.get_row(k - 1);
        let gso_k = base_gso.get_row(k);

        // gso[k-1] = gso[k] + b * tmu
        let new_gso_km1 = gso_k.add(&b.multiply_scalar(&tmu));
        // gso[k] = b * (B[k] / tB) - gso[k] * mu(k,k-1)
        let bk_over_tb = norms.get(k).div_frac(&tb);
        let new_mu_kk1 = mu.get(k, k - 1).clone(); // = tmu * t (already set above)
        let new_gso_k = b.multiply_scalar(&bk_over_tb).subtract(&gso_k.multiply_scalar(&new_mu_kk1));

        base_gso.set_row(k - 1, &new_gso_km1);
        base_gso.set_row(k, &new_gso_k);

        // B[k] = B[k] * t
        let new_bk = norms.get(k).mul_frac(&t);
        norms.set(k, new_bk);
        // B[k-1] = tB
        norms.set(k - 1, tb);

        for i in (k + 1)..=kmax {
            let t_val = mu.get(i, k).clone();
            let new_ik = mu.get(i, k - 1).sub_frac(&tmu.mul_frac(&t_val));
            let new_ikm1 = t_val.add_frac(&mu.get(k, k - 1).mul_frac(&new_ik));
            mu.set(i, k, new_ik);
            mu.set(i, k - 1, new_ikm1);
        }
    }
}
