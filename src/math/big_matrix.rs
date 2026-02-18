use super::big_fraction::BigFraction;
use super::big_vector::BigVector;

/// A matrix of BigFraction values stored in row-major order.
#[derive(Clone, Debug)]
pub struct BigMatrix {
    data: Vec<BigFraction>,
    rows: usize,
    cols: usize,
}

impl BigMatrix {
    pub fn new(rows: usize, cols: usize) -> Self {
        BigMatrix {
            data: vec![BigFraction::zero(); rows * cols],
            rows,
            cols,
        }
    }

    pub fn identity(size: usize) -> Self {
        let mut m = BigMatrix::new(size, size);
        for i in 0..size {
            m.set(i, i, BigFraction::one());
        }
        m
    }

    pub fn row_count(&self) -> usize {
        self.rows
    }

    pub fn col_count(&self) -> usize {
        self.cols
    }

    pub fn is_square(&self) -> bool {
        self.rows == self.cols
    }

    pub fn get(&self, row: usize, col: usize) -> &BigFraction {
        &self.data[row * self.cols + col]
    }

    pub fn set(&mut self, row: usize, col: usize, value: BigFraction) {
        self.data[row * self.cols + col] = value;
    }

    pub fn get_row(&self, row: usize) -> BigVector {
        let start = row * self.cols;
        BigVector::from_data(self.data[start..start + self.cols].to_vec())
    }

    pub fn set_row(&mut self, row: usize, v: &BigVector) {
        assert_eq!(v.dimension(), self.cols);
        let start = row * self.cols;
        for i in 0..self.cols {
            self.data[start + i] = v.get(i).clone();
        }
    }

    pub fn get_col(&self, col: usize) -> BigVector {
        let mut v = BigVector::new(self.rows);
        for i in 0..self.rows {
            v.set(i, self.get(i, col).clone());
        }
        v
    }

    pub fn set_col(&mut self, col: usize, v: &BigVector) {
        assert_eq!(v.dimension(), self.rows);
        for i in 0..self.rows {
            self.set(i, col, v.get(i).clone());
        }
    }

    pub fn swap_rows(&mut self, r1: usize, r2: usize) {
        if r1 == r2 {
            return;
        }
        for col in 0..self.cols {
            let i1 = r1 * self.cols + col;
            let i2 = r2 * self.cols + col;
            self.data.swap(i1, i2);
        }
    }

    pub fn swap_elements(&mut self, r1: usize, c1: usize, r2: usize, c2: usize) {
        let i1 = r1 * self.cols + c1;
        let i2 = r2 * self.cols + c2;
        self.data.swap(i1, i2);
    }

    pub fn transpose(&self) -> BigMatrix {
        let mut m = BigMatrix::new(self.cols, self.rows);
        for r in 0..self.rows {
            for c in 0..self.cols {
                m.set(c, r, self.get(r, c).clone());
            }
        }
        m
    }

    pub fn multiply_matrix(&self, other: &BigMatrix) -> BigMatrix {
        assert_eq!(self.cols, other.rows);
        let mut result = BigMatrix::new(self.rows, other.cols);
        for r in 0..self.rows {
            for c in 0..other.cols {
                let mut sum = BigFraction::zero();
                for k in 0..self.cols {
                    sum = sum.add_frac(&self.get(r, k).mul_frac(other.get(k, c)));
                }
                result.set(r, c, sum);
            }
        }
        result
    }

    pub fn multiply_vector(&self, v: &BigVector) -> BigVector {
        assert_eq!(self.cols, v.dimension());
        let mut result = BigVector::new(self.rows);
        for r in 0..self.rows {
            let row = self.get_row(r);
            result.set(r, row.dot(v));
        }
        result
    }

    pub fn multiply_scalar(&self, scalar: &BigFraction) -> BigMatrix {
        let mut m = self.clone();
        for i in 0..m.data.len() {
            m.data[i] = m.data[i].mul_frac(scalar);
        }
        m
    }

    /// Get a submatrix view (copies data).
    pub fn submatrix(
        &self,
        start_row: usize,
        start_col: usize,
        row_count: usize,
        col_count: usize,
    ) -> BigMatrix {
        let mut m = BigMatrix::new(row_count, col_count);
        for r in 0..row_count {
            for c in 0..col_count {
                m.set(r, c, self.get(start_row + r, start_col + c).clone());
            }
        }
        m
    }

    /// Row operations for Gauss-Jordan / LU:
    pub fn row_subtract_scaled(&mut self, target_row: usize, source_row: usize, scale: &BigFraction) {
        for c in 0..self.cols {
            let val = self.get(target_row, c).sub_frac(&self.get(source_row, c).mul_frac(scale));
            self.set(target_row, c, val);
        }
    }

    pub fn row_divide(&mut self, row: usize, divisor: &BigFraction) {
        let recip = divisor.reciprocal();
        for c in 0..self.cols {
            let val = self.get(row, c).mul_frac(&recip);
            self.set(row, c, val);
        }
    }

    /// Multiply row by scalar in place
    pub fn row_multiply(&mut self, row: usize, scalar: &BigFraction) {
        for c in 0..self.cols {
            let val = self.get(row, c).mul_frac(scalar);
            self.set(row, c, val);
        }
    }

    /// Add scaled row to target
    pub fn row_add_scaled(&mut self, target_row: usize, source_row: usize, scale: &BigFraction) {
        for c in 0..self.cols {
            let val = self.get(target_row, c).add_frac(&self.get(source_row, c).mul_frac(scale));
            self.set(target_row, c, val);
        }
    }
}

impl std::fmt::Display for BigMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{")?;
        for r in 0..self.rows {
            if r > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", self.get_row(r))?;
        }
        write!(f, "}}")
    }
}
