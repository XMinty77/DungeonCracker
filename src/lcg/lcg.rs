/// Linear Congruential Generator parameters.
/// Models: seed_{n+1} = (seed_n * multiplier + addend) mod modulus
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LCG {
    pub multiplier: i64,
    pub addend: i64,
    pub modulus: i64,
}

impl LCG {
    /// Java's java.util.Random LCG
    pub const JAVA: LCG = LCG {
        multiplier: 0x5DEECE66D,
        addend: 0xB,
        modulus: 1 << 48,
    };

    pub fn new(multiplier: i64, addend: i64, modulus: i64) -> Self {
        LCG {
            multiplier,
            addend,
            modulus,
        }
    }

    pub fn next_seed(&self, seed: i64) -> i64 {
        self.modop(seed.wrapping_mul(self.multiplier).wrapping_add(self.addend))
    }

    pub fn modop(&self, n: i64) -> i64 {
        // Modulus is always a power of 2 for Java LCG
        if self.modulus > 0 && (self.modulus & (self.modulus.wrapping_neg())) == self.modulus {
            n & (self.modulus - 1)
        } else {
            // Unsigned remainder for non-power-of-2
            ((n as u64) % (self.modulus as u64)) as i64
        }
    }

    /// Combine this LCG with itself `steps` times.
    /// Equivalent to advancing the LCG by `steps` calls in one operation.
    pub fn combine(&self, steps: i64) -> LCG {
        let mut multiplier: i64 = 1;
        let mut addend: i64 = 0;

        let mut im = self.multiplier;
        let mut ia = self.addend;

        let mut k = steps;
        while k != 0 {
            if (k & 1) != 0 {
                multiplier = multiplier.wrapping_mul(im);
                addend = im.wrapping_mul(addend).wrapping_add(ia);
            }
            ia = (im.wrapping_add(1)).wrapping_mul(ia);
            im = im.wrapping_mul(im);
            k = ((k as u64) >> 1) as i64;
        }

        multiplier = self.modop(multiplier);
        addend = self.modop(addend);

        LCG::new(multiplier, addend, self.modulus)
    }

    /// Invert: combine(-1)
    pub fn invert(&self) -> LCG {
        self.combine(-1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_lcg_basic() {
        let lcg = LCG::JAVA;
        // Seed 0 -> next
        let s1 = lcg.next_seed(0);
        assert_eq!(s1, 0xB);
        let s2 = lcg.next_seed(s1);
        assert_eq!(s2, (0xB_i64.wrapping_mul(0x5DEECE66D) + 0xB) & ((1 << 48) - 1));
    }

    #[test]
    fn test_combine_identity() {
        let lcg = LCG::JAVA;
        let combined = lcg.combine(1);
        assert_eq!(combined.multiplier, lcg.multiplier);
        assert_eq!(combined.addend, lcg.addend);
    }

    #[test]
    fn test_combine_invert() {
        let lcg = LCG::JAVA;
        let inv = lcg.invert();
        let seed: i64 = 12345;
        let next = lcg.next_seed(seed);
        let back = inv.next_seed(next);
        assert_eq!(back, seed);
    }
}
