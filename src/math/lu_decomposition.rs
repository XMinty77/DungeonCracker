use super::big_fraction::BigFraction;
use super::big_matrix::BigMatrix;

/// LU Decomposition for BigMatrix (exact arithmetic with BigFraction).
/// Returns the inverse matrix.
pub fn inverse(matrix: &BigMatrix) -> BigMatrix {
    assert!(matrix.is_square(), "Matrix is not square");
    let size = matrix.row_count();

    let mut m = matrix.clone();
    let mut inv = BigMatrix::identity(size);

    // Decomposition
    for i in 0..size {
        let mut pivot = None;
        let mut biggest = BigFraction::zero();

        for row in i..size {
            let d = m.get(row, i).abs();
            if d > biggest {
                biggest = d;
                pivot = Some(row);
            }
        }

        let pivot = pivot.expect("Matrix is singular");

        inv.swap_rows(i, pivot);
        if pivot != i {
            m.swap_rows(i, pivot);
        }

        for row in (i + 1)..size {
            let val = m.get(row, i).div_frac(m.get(i, i));
            m.set(row, i, val);
        }

        for row in (i + 1)..size {
            for col in (i + 1)..size {
                let val = m.get(row, col).sub_frac(&m.get(row, i).mul_frac(m.get(i, col)));
                m.set(row, col, val);
            }
        }
    }

    // Inverse (forward substitution)
    for dcol in 0..size {
        for row in 0..size {
            for col in 0..row {
                let val = inv.get(row, dcol).sub_frac(&m.get(row, col).mul_frac(inv.get(col, dcol)));
                inv.set(row, dcol, val);
            }
        }
    }

    // Inverse (back substitution)
    for dcol in 0..size {
        for row in (0..size).rev() {
            for col in ((row + 1)..size).rev() {
                let val = inv.get(row, dcol).sub_frac(&m.get(row, col).mul_frac(inv.get(col, dcol)));
                inv.set(row, dcol, val);
            }
            let val = inv.get(row, dcol).div_frac(m.get(row, row));
            inv.set(row, dcol, val);
        }
    }

    inv
}
