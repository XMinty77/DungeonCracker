use crate::lcg::lcg::LCG;

/// Port of mcseed's JRand - Java's java.util.Random equivalent.
/// This is separate from latticg's Rand - this is used for MC world gen simulation.
#[derive(Clone, Debug)]
pub struct JRand {
    seed: i64,
}

impl JRand {
    pub fn new(seed: i64) -> Self {
        JRand {
            seed: (seed ^ LCG::JAVA.multiplier) & ((1i64 << 48) - 1),
        }
    }

    pub fn of_internal_seed(seed: i64) -> Self {
        JRand {
            seed: seed & ((1i64 << 48) - 1),
        }
    }

    pub fn set_seed(&mut self, seed: i64, scramble: bool) {
        if scramble {
            self.seed = (seed ^ LCG::JAVA.multiplier) & ((1i64 << 48) - 1);
        } else {
            self.seed = seed & ((1i64 << 48) - 1);
        }
    }

    pub fn get_seed(&self) -> i64 {
        self.seed
    }

    pub fn next(&mut self, bits: i32) -> i32 {
        self.seed = LCG::JAVA.next_seed(self.seed);
        (self.seed >> (48 - bits)) as i32
    }

    pub fn next_int(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            panic!("bound must be positive");
        }

        if (bound & (-bound)) == bound {
            // power of 2
            return ((bound as i64).wrapping_mul(self.next(31) as i64) >> 31) as i32;
        }

        loop {
            let bits = self.next(31);
            let value = bits % bound;
            if bits - value + (bound - 1) >= 0 {
                return value;
            }
        }
    }

    pub fn next_long(&mut self) -> i64 {
        ((self.next(32) as i64) << 32).wrapping_add(self.next(32) as i64)
    }

    pub fn next_float(&mut self) -> f32 {
        self.next(24) as f32 / (1 << 24) as f32
    }

    pub fn next_double(&mut self) -> f64 {
        let hi = (self.next(26) as i64) << 27;
        let lo = self.next(27) as i64;
        (hi + lo) as f64 * (1.0f64 / (1i64 << 53) as f64)
    }

    pub fn advance(&mut self, calls: i64) {
        let skip = LCG::JAVA.combine(calls);
        self.seed = skip.next_seed(self.seed);
    }
}
