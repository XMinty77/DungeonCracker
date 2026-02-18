use super::lcg::LCG;

/// A random number generator state that mirrors Java's java.util.Random.
/// This is the LattiCG Rand equivalent.
#[derive(Clone, Debug)]
pub struct Rand {
    lcg: LCG,
    seed: i64,
}

impl Rand {
    pub fn of_internal_seed(lcg: &LCG, seed: i64) -> Self {
        Rand {
            lcg: lcg.clone(),
            seed: lcg.modop(seed),
        }
    }

    pub fn of_seed_scrambled(lcg: &LCG, seed: i64) -> Self {
        Rand {
            lcg: lcg.clone(),
            seed: lcg.modop(seed ^ lcg.multiplier),
        }
    }

    pub fn get_seed(&self) -> i64 {
        self.seed
    }

    pub fn set_seed(&mut self, seed: i64) {
        self.seed = self.lcg.modop(seed);
    }

    pub fn set_seed_scrambled(&mut self, seed: i64) {
        self.seed = self.lcg.modop(seed ^ self.lcg.multiplier);
    }

    pub fn next(&mut self, bits: i32) -> i32 {
        self.seed = self.lcg.next_seed(self.seed);
        (self.seed >> (48 - bits)) as i32
    }

    pub fn advance(&mut self, calls: i64) {
        let skip = self.lcg.combine(calls);
        self.seed = skip.next_seed(self.seed);
    }

    pub fn advance_lcg(&mut self, skip: &LCG) {
        self.seed = skip.next_seed(self.seed);
    }

    pub fn next_int(&mut self, bound: i32) -> i32 {
        if bound <= 0 {
            panic!("bound must be positive");
        }

        if (bound & (-bound)) == bound {
            // power of 2
            return ((bound as i64 * self.next(31) as i64) >> 31) as i32;
        }

        let mut bits;
        let mut value;
        loop {
            bits = self.next(31);
            value = bits % bound;
            if bits - value + (bound - 1) >= 0 {
                break;
            }
        }
        value
    }

    pub fn next_long(&mut self) -> i64 {
        ((self.next(32) as i64) << 32).wrapping_add(self.next(32) as i64)
    }

    pub fn next_float(&mut self) -> f32 {
        self.next(24) as f32 / (1 << 24) as f32
    }

    pub fn next_double(&mut self) -> f64 {
        (((self.next(26) as i64) << 27) + self.next(27) as i64) as f64 * (1.0f64 / (1i64 << 53) as f64)
    }
}
