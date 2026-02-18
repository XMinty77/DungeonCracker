use super::big_fraction::BigFraction;
use super::big_matrix::BigMatrix;
use super::big_vector::BigVector;
use super::gauss_jordan;

/// Linear programming optimizer using the simplex method over BigFractions.
/// This is a faithful port of the Java Optimize class from LattiCG.
#[derive(Clone)]
pub struct Optimize {
    table: BigMatrix,
    basics: Vec<usize>,
    nonbasics: Vec<usize>,
    transform: BigMatrix,
    rows: usize,
    cols: usize,
}

impl Optimize {
    fn new(table: BigMatrix, basics: Vec<usize>, nonbasics: Vec<usize>, transform: BigMatrix) -> Self {
        let rows = table.row_count();
        let cols = table.col_count();
        Optimize {
            table,
            basics,
            nonbasics,
            transform,
            rows,
            cols,
        }
    }

    pub fn table_size(&self) -> (usize, usize) {
        (self.rows, self.cols)
    }

    fn transform_for_table(&self, lhs: &BigVector, rhs: &BigFraction) -> BigVector {
        let tcols = self.transform.col_count();
        let mut transformed = BigVector::new(tcols);
        transformed.set(tcols - 1, rhs.clone());

        for row in 0..self.transform.row_count() {
            let x = lhs.get(row).clone();
            let tr_row = self.transform.get_row(row);
            let scaled = tr_row.multiply_scalar(&x);
            transformed.subtract_assign(&scaled);
        }

        let mut eliminated = BigVector::new(self.cols);
        for col in 0..(self.cols - 1) {
            eliminated.set(col, transformed.get(self.nonbasics[col]).clone());
        }
        eliminated.set(self.cols - 1, transformed.get(tcols - 1).clone());

        for row in 0..(self.rows - 1) {
            let x = transformed.get(self.basics[row]).clone();
            let t_row = self.table.get_row(row);
            let scaled = t_row.multiply_scalar(&x);
            eliminated.subtract_assign(&scaled);
        }

        eliminated
    }

    pub fn maximize(&mut self, gradient: &BigVector) -> (BigVector, BigFraction) {
        let neg = BigVector::from_data(
            (0..gradient.dimension())
                .map(|i| gradient.get(i).negate())
                .collect(),
        );
        let (result, val) = self.minimize(&neg);
        (result, val.negate())
    }

    pub fn minimize(&mut self, gradient: &BigVector) -> (BigVector, BigFraction) {
        assert_eq!(gradient.dimension(), self.transform.row_count());

        // Set objective row
        let obj_row = BigVector::new(self.cols);
        self.table.set_row(self.rows - 1, &obj_row);

        let neg_transformed = self.transform_for_table(gradient, &BigFraction::zero());
        // Subtract from objective row
        for c in 0..self.cols {
            let val = self.table.get(self.rows - 1, c).sub_frac(neg_transformed.get(c));
            self.table.set(self.rows - 1, c, val);
        }

        self.solve();

        let tcols = self.transform.col_count();
        let mut result = self.transform.get_col(tcols - 1);

        for row in 0..(self.rows - 1) {
            let v0 = self.basics[row];
            let scale = self.table.get(row, self.cols - 1);
            let col_vec = self.transform.get_col(v0);
            let scaled = col_vec.multiply_scalar(scale);
            result.subtract_assign(&scaled);
        }

        let obj_val = self.table.get(self.rows - 1, self.cols - 1).clone();
        (result, obj_val)
    }

    fn solve(&mut self) {
        let mut iters = 0u64;
        while self.step() {
            iters += 1;
            if iters % 10000 == 0 {
                eprintln!("[simplex]     solve iteration {}, table {}x{}", iters, self.rows, self.cols);
            }
            if iters > 1_000_000 {
                eprintln!("[simplex]     WARNING: over 1M iterations, likely cycling. Aborting.");
                break;
            }
        }
    }

    fn step(&mut self) -> bool {
        let mut bland = false;

        for row in 0..(self.rows - 1) {
            if self.table.get(row, self.cols - 1).signum() == 0 {
                bland = true;
                break;
            }
        }

        let mut entering: Option<usize> = None;
        let mut candidate = BigFraction::zero();

        for col in 0..(self.cols - 1) {
            let x = self.table.get(self.rows - 1, col);
            if x.signum() <= 0 {
                continue;
            }
            if entering.is_some() && *x <= candidate {
                continue;
            }
            entering = Some(col);
            candidate = x.clone();
            if bland {
                break;
            }
        }

        let entering = match entering {
            Some(e) => e,
            None => return false,
        };

        let mut exiting: Option<usize> = None;
        candidate = BigFraction::zero(); // reuse for ratio test

        for row in 0..(self.rows - 1) {
            let x = self.table.get(row, entering);
            if x.signum() <= 0 {
                continue;
            }
            let y = self.table.get(row, self.cols - 1).div_frac(x);
            if exiting.is_some() && y >= candidate {
                continue;
            }
            exiting = Some(row);
            candidate = y;
        }

        let exiting = exiting.expect("Unbounded LP");
        self.pivot(entering, exiting);
        true
    }

    fn pivot(&mut self, entering: usize, exiting: usize) {
        let rows = self.rows;
        let cols = self.cols;

        let pivot = self.table.get(exiting, entering).clone();

        // Scale pivot row
        for col in 0..cols {
            if col == entering {
                continue;
            }
            let val = self.table.get(exiting, col).div_frac(&pivot);
            self.table.set(exiting, col, val);
        }

        // Eliminate entering column from other rows
        for row in 0..rows {
            if row == exiting {
                continue;
            }
            let x = self.table.get(row, entering).clone();
            for col in 0..cols {
                if col == entering {
                    continue;
                }
                let y = self.table.get(exiting, col);
                let val = self.table.get(row, col).sub_frac(&x.mul_frac(y));
                self.table.set(row, col, val);
            }
            let val = x.div_frac(&pivot).negate();
            self.table.set(row, entering, val);
        }

        let recip = pivot.reciprocal();
        self.table.set(exiting, entering, recip);

        // Swap basic/nonbasic
        let tmp = self.nonbasics[entering];
        self.nonbasics[entering] = self.basics[exiting];
        self.basics[exiting] = tmp;
    }

    pub fn with_strict_bound(&self, lhs: &BigVector, rhs: &BigFraction) -> Optimize {
        let mut new_table = BigMatrix::new(self.rows + 1, self.cols);

        for row in 0..(self.rows - 1) {
            for col in 0..self.cols {
                new_table.set(row, col, self.table.get(row, col).clone());
            }
        }

        let bound_row = self.transform_for_table(lhs, rhs);
        for col in 0..self.cols {
            new_table.set(self.rows - 1, col, bound_row.get(col).clone());
        }

        if new_table.get(self.rows - 1, self.cols - 1).signum() < 0 {
            new_table.row_multiply(self.rows - 1, &BigFraction::minus_one());
        }

        let mut new_basics = self.basics.clone();
        new_basics.push((self.rows - 1) + (self.cols - 1));

        let new_nonbasics = self.nonbasics.clone();

        Optimize::from_table(new_table, new_basics, new_nonbasics, 1, &self.transform)
    }

    fn from_table(
        mut table: BigMatrix,
        basics: Vec<usize>,
        nonbasics: Vec<usize>,
        artificials: usize,
        transform: &BigMatrix,
    ) -> Optimize {
        let rows = table.row_count();
        let cols = table.col_count();

        let real_variables = (rows - 1) + (cols - 1) - artificials;

        // Phase 1: add artificial rows to objective
        for basic_row in 0..(rows - 1) {
            if basics[basic_row] < real_variables {
                continue;
            }
            // Add row to objective row
            for col in 0..cols {
                let val = table.get(rows - 1, col).add_frac(table.get(basic_row, col));
                table.set(rows - 1, col, val);
            }
        }

        let mut opt = Optimize::new(table, basics.clone(), nonbasics.clone(), BigMatrix::new(1, 1));
        opt.solve();

        // Check feasibility
        if opt.table.get(opt.rows - 1, opt.cols - 1).signum() != 0 {
            panic!("Table has no basic feasible solutions");
        }

        // Pivot out artificial variables
        for row in 0..(opt.rows - 1) {
            if opt.basics[row] >= real_variables {
                for col in 0..(opt.cols - 1) {
                    if opt.nonbasics[col] >= real_variables || opt.table.get(row, col).signum() == 0 {
                        continue;
                    }
                    opt.pivot(col, row);
                    break;
                }
            }
        }

        // Remove artificial columns
        let final_cols = cols - artificials;
        let mut final_table = BigMatrix::new(rows, final_cols);

        let mut c0 = 0usize;
        let mut c1 = 0usize;
        let mut final_nonbasics = vec![0usize; final_cols - 1];

        while c0 < final_cols - 1 {
            while c1 < cols - 1 && opt.nonbasics[c1] >= real_variables {
                c1 += 1;
            }
            if c1 >= cols - 1 {
                break;
            }
            for row in 0..(rows - 1) {
                final_table.set(row, c0, opt.table.get(row, c1).clone());
            }
            final_nonbasics[c0] = opt.nonbasics[c1];
            c0 += 1;
            c1 += 1;
        }

        for row in 0..(rows - 1) {
            final_table.set(row, final_cols - 1, opt.table.get(row, cols - 1).clone());
        }

        Optimize::new(final_table, opt.basics.clone(), final_nonbasics, transform.clone())
    }

    fn from_inner_table(inner_table: &BigMatrix, transform: &BigMatrix) -> Optimize {
        let constraints = inner_table.row_count();
        let variables = inner_table.col_count() - 1;

        let mut inner = inner_table.clone();
        let mut basics = vec![usize::MAX; constraints];
        let mut nonbasic_list: Vec<usize> = Vec::new();

        // Ensure RHS is non-negative
        for row in 0..constraints {
            if inner.get(row, variables).signum() < 0 {
                inner.row_multiply(row, &BigFraction::minus_one());
            }
        }

        // Find initial basic variables (columns with single non-zero entry)
        for col in 0..variables {
            let mut count = 0;
            let mut index = 0;
            for row in 0..constraints {
                if inner.get(row, col).signum() != 0 {
                    count += 1;
                    index = row;
                }
            }
            if count == 1 && basics[index] == usize::MAX && inner.get(index, col).signum() > 0 {
                let pivot = inner.get(index, col).clone();
                inner.row_divide(index, &pivot);
                basics[index] = col;
            } else {
                nonbasic_list.push(col);
            }
        }

        let mut artificials = 0usize;
        for row in 0..constraints {
            if basics[row] == usize::MAX {
                basics[row] = variables + artificials;
                artificials += 1;
            }
        }

        let nonbasic_count = variables - constraints + artificials;
        let nonbasics: Vec<usize> = nonbasic_list.iter().copied().collect();
        let mut table = BigMatrix::new(constraints + 1, nonbasic_count + 1);

        for row in 0..constraints {
            // Eliminate basic columns from this row
            for basic_row in 0..constraints {
                if basic_row == row || basics[basic_row] >= variables {
                    continue;
                }
                let scale = inner.get(row, basics[basic_row]).clone();
                if !scale.is_zero() {
                    // row -= scale * basic_row
                    for c in 0..inner.col_count() {
                        let val = inner.get(row, c).sub_frac(&inner.get(basic_row, c).mul_frac(&scale));
                        inner.set(row, c, val);
                    }
                }
            }

            for col in 0..nonbasic_count {
                if col < nonbasics.len() {
                    table.set(row, col, inner.get(row, nonbasics[col]).clone());
                }
            }
            table.set(row, nonbasic_count, inner.get(row, variables).clone());
        }

        let mut final_nonbasics = vec![0usize; nonbasic_count];
        for i in 0..nonbasics.len().min(nonbasic_count) {
            final_nonbasics[i] = nonbasics[i];
        }

        Optimize::from_table(table, basics, final_nonbasics, artificials, transform)
    }
}

/// Builder for constructing Optimize instances with bounds constraints.
pub struct OptimizeBuilder {
    size: usize,
    slacks: Vec<i32>,
    lefts: Vec<BigVector>,
    rights: Vec<BigFraction>,
}

impl OptimizeBuilder {
    pub fn of_size(size: usize) -> Self {
        OptimizeBuilder {
            size,
            slacks: Vec::new(),
            lefts: Vec::new(),
            rights: Vec::new(),
        }
    }

    pub fn with_lower_bound_idx(mut self, idx: usize, rhs: &BigFraction) -> Self {
        let lhs = BigVector::basis_one(self.size, idx);
        self.slacks.push(-1);
        self.lefts.push(lhs);
        self.rights.push(rhs.clone());
        self
    }

    pub fn with_upper_bound_idx(mut self, idx: usize, rhs: &BigFraction) -> Self {
        let lhs = BigVector::basis_one(self.size, idx);
        self.slacks.push(1);
        self.lefts.push(lhs);
        self.rights.push(rhs.clone());
        self
    }

    pub fn build(self) -> Optimize {
        let variables = self.size + self.slacks.len();
        let mut constraint = 0usize;
        let mut slack = self.size;

        // Allocate table with extra room
        let max_rows = self.slacks.len() + self.size;
        let max_cols = variables + 2 * self.size + 1;
        let mut table = BigMatrix::new(max_rows, max_cols);

        // Fill constraints
        for i in 0..self.slacks.len() {
            for col in 0..self.size {
                table.set(constraint, col, self.lefts[i].get(col).clone());
            }
            table.set(constraint, variables + 2 * self.size, self.rights[i].clone());

            if self.slacks[i] != 0 {
                table.set(constraint, slack, BigFraction::from_i64(self.slacks[i] as i64));
                slack += 1;
            }
            constraint += 1;
        }

        // Reduce real variables out
        let mut pivot_rows = gauss_jordan::reduce(&mut table, &mut [], &|col, _| col < self.size);

        // For any real variables we couldn't remove, add slack pair
        for col in 0..self.size {
            if pivot_rows[col] != -1 {
                continue;
            }
            table.set(constraint, col, BigFraction::one());
            table.set(constraint, slack, BigFraction::one());
            table.set(constraint, slack + 1, BigFraction::minus_one());
            constraint += 1;
            slack += 2;
        }

        // Re-reduce
        pivot_rows = gauss_jordan::reduce_all(&mut table);

        // Check all real variables removed
        for col in 0..self.size {
            assert!(
                pivot_rows[col] != -1,
                "Could not remove column from table"
            );
        }

        constraint = 1 + pivot_rows.iter().copied().max().unwrap_or(-1) as usize;

        // Build transform and inner table
        let slack_count = slack - self.size;
        let mut transform = BigMatrix::new(self.size, slack_count + 1);
        let inner_rows = if constraint > self.size { constraint - self.size } else { 0 };
        let mut inner_table = BigMatrix::new(inner_rows.max(1), slack_count + 1);

        for row in 0..self.size {
            for col in 0..slack_count {
                transform.set(row, col, table.get(row, self.size + col).clone());
            }
            transform.set(row, slack_count, table.get(row, variables + 2 * self.size).clone());
        }

        for row in 0..inner_rows {
            for col in 0..slack_count {
                inner_table.set(row, col, table.get(self.size + row, self.size + col).clone());
            }
            inner_table.set(row, slack_count, table.get(self.size + row, variables + 2 * self.size).clone());
        }

        Optimize::from_inner_table(&inner_table, &transform)
    }
}
