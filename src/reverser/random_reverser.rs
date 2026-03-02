use crate::lcg::lcg::LCG;
use crate::lcg::rand::Rand;
use crate::lattice::enumerate;
use crate::lattice::lll;
use crate::math::big_fraction::{BigFraction, FracOps};
use crate::math::big_matrix::BigMatrix;
use crate::math::big_vector::BigVector;
use crate::math::int_type::{Int, IntOps};
use crate::math::lu_decomposition;
use crate::math::mth;
use crate::reverser::filtered_skip::FilteredSkip;

/// Combined RandomReverser + JavaRandomReverser.
/// Builds lattice constraints from java.util.Random call observations,
/// then uses LLL reduction + enumeration to find matching seeds.
pub struct JavaRandomReverser {
    modulus: Int,
    mult: Int,
    lcg: LCG,
    mins: Vec<Int>,
    maxes: Vec<Int>,
    call_indices: Vec<i64>,
    filtered_skips: Vec<FilteredSkip>,
    lattice: Option<BigMatrix>,
    current_call_index: i64,
    dimensions: usize,
    success_chance: f64,
}

impl JavaRandomReverser {
    pub fn new(filtered_skips: Vec<FilteredSkip>) -> Self {
        let lcg = LCG::JAVA;
        let modulus = Int::int_from_i64(lcg.modulus);
        let mult = Int::int_from_i64(lcg.multiplier).int_rem(&modulus);
        JavaRandomReverser {
            modulus,
            mult,
            lcg,
            mins: Vec::new(),
            maxes: Vec::new(),
            call_indices: Vec::new(),
            filtered_skips,
            lattice: None,
            current_call_index: 0,
            dimensions: 0,
            success_chance: 1.0,
        }
    }

    /// Add a constraint on the measured seed value (in internal 48-bit representation).
    pub fn add_measured_seed(&mut self, min: i64, max: i64) {
        self.add_measured_seed_big(Int::int_from_i64(min), Int::int_from_i64(max));
    }

    pub fn add_measured_seed_big(&mut self, min: Int, max: Int) {
        let min = mod_big(&min, &self.modulus);
        let mut max = mod_big(&max, &self.modulus);
        if max < min {
            max = max.int_add(&self.modulus);
        }

        self.mins.push(min);
        self.maxes.push(max);
        self.dimensions += 1;
        self.current_call_index += 1;
        self.call_indices.push(self.current_call_index);

        let dim = self.dimensions;
        let mut new_lattice = BigMatrix::new(dim + 1, dim);

        if dim != 1 {
            if let Some(ref old_lattice) = self.lattice {
                for row in 0..dim {
                    for col in 0..(dim - 1) {
                        new_lattice.set(row, col, old_lattice.get(row, col).clone());
                    }
                }
            }
        }

        let exp = Int::int_from_i64(self.call_indices[dim - 1] - self.call_indices[0]);
        let temp_mult = self.mult.int_modpow(&exp, &self.modulus);
        new_lattice.set(0, dim - 1, BigFraction::frac_from_bigint(temp_mult));
        new_lattice.set(dim, dim - 1, BigFraction::frac_from_bigint(self.modulus.clone()));
        self.lattice = Some(new_lattice);
    }

    /// Add a constraint on the seed modulo a different modulus.
    pub fn add_modulo_measured_seed(&mut self, min: i64, max: i64, measured_mod: i64) {
        self.add_modulo_measured_seed_big(
            Int::int_from_i64(min),
            Int::int_from_i64(max),
            Int::int_from_i64(measured_mod),
        );
    }

    pub fn add_modulo_measured_seed_big(&mut self, min: Int, max: Int, measured_mod: Int) {
        let min = mod_big(&min, &measured_mod);
        let mut max = mod_big(&max, &measured_mod);
        if max < min {
            max = max.int_add(&measured_mod);
        }

        let residue = self.modulus.int_rem(&measured_mod);
        if !residue.int_is_zero() {
            self.success_chance *= 1.0 - residue.int_to_f64_approx() / self.lcg.modulus as f64;

            // First condition: is the seed real
            self.mins.push(Int::int_zero());
            self.maxes.push(self.modulus.int_sub(&residue));
            self.current_call_index += 1;
            self.call_indices.push(self.current_call_index);

            // Second condition: does the seed satisfy bounds
            self.mins.push(min);
            self.maxes.push(max);
            self.call_indices.push(self.current_call_index); // same call index

            self.dimensions += 2;

            let dim = self.dimensions;
            let mut new_lattice = BigMatrix::new(dim + 1, dim);

            if dim != 2 {
                if let Some(ref old) = self.lattice {
                    for row in 0..(dim - 1) {
                        for col in 0..(dim - 2) {
                            new_lattice.set(row, col, old.get(row, col).clone());
                        }
                    }
                }
            }

            let exp = Int::int_from_i64(self.call_indices[dim - 1] - self.call_indices[0]);
            let temp_mult = self.mult.int_modpow(&exp, &self.modulus);
            new_lattice.set(0, dim - 2, BigFraction::frac_from_bigint(temp_mult.clone()));
            new_lattice.set(0, dim - 1, BigFraction::frac_from_bigint(temp_mult));
            new_lattice.set(dim - 1, dim - 1, BigFraction::frac_from_bigint(self.modulus.clone()));
            new_lattice.set(dim - 1, dim - 2, BigFraction::frac_from_bigint(self.modulus.clone()));
            new_lattice.set(dim, dim - 1, BigFraction::frac_from_bigint(measured_mod));
            self.lattice = Some(new_lattice);
        } else {
            // Modulus divides evenly
            self.mins.push(min);
            self.maxes.push(max);
            self.dimensions += 1;
            self.current_call_index += 1;
            self.call_indices.push(self.current_call_index);

            let dim = self.dimensions;
            let mut new_lattice = BigMatrix::new(dim + 1, dim);

            if dim != 1 {
                if let Some(ref old) = self.lattice {
                    for row in 0..dim {
                        for col in 0..(dim - 1) {
                            new_lattice.set(row, col, old.get(row, col).clone());
                        }
                    }
                }
            }

            let exp = Int::int_from_i64(self.call_indices[dim - 1] - self.call_indices[0]);
            let temp_mult = self.mult.int_modpow(&exp, &self.modulus);
            new_lattice.set(0, dim - 1, BigFraction::frac_from_bigint(temp_mult));
            new_lattice.set(dim, dim - 1, BigFraction::frac_from_bigint(measured_mod));
            self.lattice = Some(new_lattice);
        }
    }

    /// Skip some unmeasured seeds (advance the call index without adding constraints).
    pub fn add_unmeasured_seeds(&mut self, num_seeds: i64) {
        self.current_call_index += num_seeds;
    }

    /// Get the current number of lattice dimensions.
    pub fn dimensions(&self) -> usize {
        self.dimensions
    }

    /// Get the estimated success chance.
    pub fn success_chance(&self) -> f64 {
        self.success_chance
    }

    // ---- JavaRandomReverser-specific methods ----

    /// Add a nextInt(n) call with known result (min == max) or range.
    pub fn add_next_int_call(&mut self, n: i32, min: i32, max: i32) {
        assert!(n > 0, "nextInt bound must be positive");

        if (n & (-n)) == n {
            // n is a power of 2
            let log = n.trailing_zeros() as i64;
            self.add_measured_seed(
                min as i64 * (1i64 << (48 - log)),
                max as i64 * (1i64 << (48 - log)) + (1i64 << (48 - log)) - 1,
            );
        } else {
            self.add_modulo_measured_seed(
                min as i64 * (1i64 << 17),
                (max as i64 * (1i64 << 17)) | 0x1ffff,
                n as i64 * (1i64 << 17),
            );
        }
    }

    /// Add a nextInt() call (unbounded 32-bit) with known range.
    pub fn add_next_int_unbounded_call(&mut self, min: i32, max: i32) {
        self.add_measured_seed(
            min as i64 * (1i64 << 16),
            max as i64 * (1i64 << 16) + (1i64 << 16) - 1,
        );
    }

    /// Consume nextInt calls without observing them.
    pub fn consume_next_int_calls(&mut self, num_calls: i32, bound: i32) {
        let residue = (1i64 << 48) % ((1i64 << 17) * bound as i64);
        if residue != 0 {
            self.success_chance *= f64::powi(
                1.0 - residue as f64 / (1i64 << 48) as f64,
                num_calls,
            );
        }
        self.add_unmeasured_seeds(num_calls as i64);
    }

    /// Find all valid seeds by building the lattice, reducing with LLL, and enumerating.
    pub fn find_all_valid_seeds(&mut self) -> Vec<i64> {
        if self.dimensions == 0 {
            // Degenerate: no constraints
            return (0..self.lcg.modulus).collect();
        }

        eprintln!("[lattice]   Creating lattice ({} dimensions)...", self.dimensions);
        self.create_lattice();
        eprintln!("[lattice]   Lattice created and LLL-reduced.");

        let (lattice, lower, upper, offset) = self.prepare_enumerate_params();

        eprintln!("[lattice]   Enumerating lattice points...");
        let results = enumerate::enumerate_bounds(&lattice, &lower, &upper, &offset);
        eprintln!("[lattice]   Enumeration found {} candidate(s).", results.len());

        self.filter_results(&results)
    }

    /// Get the number of depth-0 branches for parallel enumeration.
    /// Must be called after create_lattice().
    pub fn get_branch_count(&mut self) -> i64 {
        if self.dimensions == 0 {
            return 1;
        }
        self.create_lattice();
        let (lattice, lower, upper, offset) = self.prepare_enumerate_params();
        enumerate::get_branch_count(&lattice, &lower, &upper, &offset)
    }

    /// Find valid seeds for a subset of depth-0 branches [branch_start, branch_end).
    /// Each worker calls this with a different range.
    pub fn find_seeds_for_branches(&mut self, branch_start: i64, branch_end: i64) -> Vec<i64> {
        if self.dimensions == 0 {
            if branch_start == 0 {
                return (0..self.lcg.modulus).collect();
            }
            return vec![];
        }

        self.create_lattice();
        let (lattice, lower, upper, offset) = self.prepare_enumerate_params();

        eprintln!("[lattice]   Enumerating branches [{}, {})...", branch_start, branch_end);
        let results = enumerate::enumerate_bounds_partial(
            &lattice, &lower, &upper, &offset, branch_start, branch_end,
        );
        eprintln!("[lattice]   Partial enumeration found {} candidate(s).", results.len());

        self.filter_results(&results)
    }

    /// Prepare the enumeration parameters (lattice, lower, upper, offset).
    fn prepare_enumerate_params(&self) -> (BigMatrix, BigVector, BigVector, BigVector) {
        let dims = self.dimensions;
        let mut lower = BigVector::new(dims);
        let mut upper = BigVector::new(dims);
        let mut offset = BigVector::new(dims);
        let mut rand = Rand::of_internal_seed(&self.lcg, 0);

        for i in 0..dims {
            lower.set(i, BigFraction::frac_from_bigint(self.mins[i].clone()));
            upper.set(i, BigFraction::frac_from_bigint(self.maxes[i].clone()));
            offset.set(i, BigFraction::frac_from_bigint(Int::int_from_i64(rand.get_seed())));

            if i != dims - 1 {
                rand.advance(self.call_indices[i + 1] - self.call_indices[i]);
            }
        }

        let lattice = self.lattice.as_ref().unwrap().transpose();
        (lattice, lower, upper, offset)
    }

    /// Filter enumeration results through the LCG reversal and filtered skips.
    fn filter_results(&self, results: &[BigVector]) -> Vec<i64> {
        let r = self.lcg.combine(-self.call_indices[0]);

        let mut seeds: Vec<i64> = results
            .iter()
            .filter_map(|vec| {
                let n = vec.get(0).numerator_int();
                Some(r.next_seed(n.int_to_i64()))
            })
            .collect();

        // Filter by filtered skips
        if !self.filtered_skips.is_empty() {
            eprintln!("[lattice]   Filtering {} seed(s) with {} filtered skip(s)...", seeds.len(), self.filtered_skips.len());
            seeds.retain(|&seed| {
                for skip in &self.filtered_skips {
                    let mut rr = Rand::of_internal_seed(&self.lcg, seed);
                    if !skip.check_state(&mut rr) {
                        return false;
                    }
                }
                true
            });
        }

        seeds
    }

    fn create_lattice(&mut self) {
        let dims = self.dimensions;

        // Compute side lengths
        let mut side_lengths: Vec<Int> = Vec::with_capacity(dims);
        for i in 0..dims {
            side_lengths.push(self.maxes[i].int_sub(&self.mins[i]).int_add_i64(1));
        }

        // Compute LCM
        let mut lcm = Int::int_one();
        for sl in &side_lengths {
            lcm = mth::lcm_int(&lcm, sl);
        }

        // Scaling matrix
        let mut scales = BigMatrix::new(dims, dims);
        for i in 0..dims {
            scales.set(i, i, BigFraction::frac_from_bigint(lcm.int_div(&side_lengths[i])));
        }

        let unscaled = self.lattice.as_ref().unwrap().clone();
        let scaled = unscaled.multiply_matrix(&scales);

        // LLL reduction
        let params = lll::LLLParams::recommended();
        let result = lll::reduce(&scaled, &params);

        // Unscale
        let scales_inv = lu_decomposition::inverse(&scales);
        self.lattice = Some(result.reduced_basis.multiply_matrix(&scales_inv));
    }
}

/// Int modulo (always non-negative).
fn mod_big(a: &Int, m: &Int) -> Int {
    let r = a.int_rem(m);
    let shifted = r.int_add(m);
    shifted.int_rem(m)
}
