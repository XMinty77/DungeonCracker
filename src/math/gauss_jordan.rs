use super::big_matrix::BigMatrix;

/// Gauss-Jordan elimination. Returns pivot_rows array:
/// pivot_rows[col] = row that has pivot in that column, or -1 if none.
pub fn reduce(
    matrix: &mut BigMatrix,
    others: &mut [&mut BigMatrix],
    predicate: &dyn Fn(usize, &[i32]) -> bool,
) -> Vec<i32> {
    let rows = matrix.row_count();
    let cols = matrix.col_count();
    let mut pivot_rows = vec![-1i32; cols];

    let mut row = 0usize;
    let mut pivot_col = 0usize;

    while row < rows && pivot_col < cols {
        // Find pivot row
        let mut pivot_row = None;
        for pr in row..rows {
            if !matrix.get(pr, pivot_col).is_zero() {
                pivot_row = Some(pr);
                break;
            }
        }

        if let Some(pr) = pivot_row {
            let pivot = matrix.get(pr, pivot_col).clone();

            // Divide pivot row by pivot value
            matrix.row_divide(pr, &pivot);
            for other in others.iter_mut() {
                other.row_divide(pr, &pivot);
            }

            // Eliminate column in all other rows
            for i in 0..rows {
                if i == pr {
                    continue;
                }
                let scale = matrix.get(i, pivot_col).clone();
                if !scale.is_zero() {
                    matrix.row_subtract_scaled(i, pr, &scale);
                    for other in others.iter_mut() {
                        other.row_subtract_scaled(i, pr, &scale);
                    }
                }
            }

            // Swap rows
            if pr != row {
                matrix.swap_rows(row, pr);
                for other in others.iter_mut() {
                    other.swap_rows(row, pr);
                }
            }

            pivot_rows[pivot_col] = row as i32;
            row += 1;
        }

        // Advance pivot column, checking predicate
        loop {
            pivot_col += 1;
            if pivot_col >= cols || predicate(pivot_col, &pivot_rows) {
                break;
            }
        }
    }

    pivot_rows
}

pub fn reduce_all(matrix: &mut BigMatrix) -> Vec<i32> {
    reduce(matrix, &mut [], &|_, _| true)
}
