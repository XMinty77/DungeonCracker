use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{One, Signed, Zero};
use std::cmp::Ordering;
use std::fmt;

/// An exact rational number represented as numerator/denominator.
/// Invariants:
/// - denominator > 0
/// - gcd(|numerator|, denominator) == 1
/// - if numerator == 0 then denominator == 1
#[derive(Clone, Debug)]
pub struct BigFraction {
    ntor: BigInt,
    dtor: BigInt,
}

impl BigFraction {
    pub fn zero() -> Self {
        BigFraction {
            ntor: BigInt::zero(),
            dtor: BigInt::one(),
        }
    }

    pub fn one() -> Self {
        BigFraction {
            ntor: BigInt::one(),
            dtor: BigInt::one(),
        }
    }

    pub fn minus_one() -> Self {
        BigFraction {
            ntor: -BigInt::one(),
            dtor: BigInt::one(),
        }
    }

    pub fn half() -> Self {
        BigFraction::new_raw(BigInt::one(), BigInt::from(2))
    }

    pub fn new(numerator: impl Into<BigInt>, denominator: impl Into<BigInt>) -> Self {
        let mut f = BigFraction {
            ntor: numerator.into(),
            dtor: denominator.into(),
        };
        if f.dtor.is_zero() {
            panic!("Division by zero");
        }
        f.simplify();
        f
    }

    pub fn from_i64(n: i64) -> Self {
        BigFraction {
            ntor: BigInt::from(n),
            dtor: BigInt::one(),
        }
    }

    pub fn from_bigint(n: BigInt) -> Self {
        BigFraction {
            ntor: n,
            dtor: BigInt::one(),
        }
    }

    fn new_raw(ntor: BigInt, dtor: BigInt) -> Self {
        let mut f = BigFraction { ntor, dtor };
        f.simplify();
        f
    }

    fn simplify(&mut self) {
        if self.ntor.is_zero() {
            self.dtor = BigInt::one();
            return;
        }
        if self.dtor.is_negative() {
            self.ntor = -&self.ntor;
            self.dtor = -&self.dtor;
        }
        let g = self.ntor.gcd(&self.dtor);
        self.ntor = &self.ntor / &g;
        self.dtor = &self.dtor / &g;
    }

    pub fn numerator(&self) -> &BigInt {
        &self.ntor
    }

    pub fn denominator(&self) -> &BigInt {
        &self.dtor
    }

    pub fn add_frac(&self, other: &BigFraction) -> BigFraction {
        BigFraction::new_raw(
            &self.ntor * &other.dtor + &other.ntor * &self.dtor,
            &self.dtor * &other.dtor,
        )
    }

    pub fn add_bigint(&self, other: &BigInt) -> BigFraction {
        BigFraction::new_raw(&self.ntor + other * &self.dtor, self.dtor.clone())
    }

    pub fn sub_frac(&self, other: &BigFraction) -> BigFraction {
        BigFraction::new_raw(
            &self.ntor * &other.dtor - &other.ntor * &self.dtor,
            &self.dtor * &other.dtor,
        )
    }

    pub fn sub_bigint(&self, other: &BigInt) -> BigFraction {
        BigFraction::new_raw(&self.ntor - other * &self.dtor, self.dtor.clone())
    }

    pub fn mul_frac(&self, other: &BigFraction) -> BigFraction {
        BigFraction::new_raw(&self.ntor * &other.ntor, &self.dtor * &other.dtor)
    }

    pub fn mul_bigint(&self, other: &BigInt) -> BigFraction {
        BigFraction::new_raw(&self.ntor * other, self.dtor.clone())
    }

    pub fn div_frac(&self, other: &BigFraction) -> BigFraction {
        BigFraction::new_raw(&self.ntor * &other.dtor, &self.dtor * &other.ntor)
    }

    pub fn div_bigint(&self, other: &BigInt) -> BigFraction {
        BigFraction::new_raw(self.ntor.clone(), &self.dtor * other)
    }

    pub fn negate(&self) -> BigFraction {
        BigFraction {
            ntor: -&self.ntor,
            dtor: self.dtor.clone(),
        }
    }

    pub fn reciprocal(&self) -> BigFraction {
        BigFraction::new_raw(self.dtor.clone(), self.ntor.clone())
    }

    pub fn abs(&self) -> BigFraction {
        if self.ntor.is_negative() {
            self.negate()
        } else {
            self.clone()
        }
    }

    pub fn signum(&self) -> i32 {
        if self.ntor.is_positive() {
            1
        } else if self.ntor.is_negative() {
            -1
        } else {
            0
        }
    }

    /// Floor: largest integer k such that k <= self
    pub fn floor(&self) -> BigInt {
        if self.dtor.is_one() {
            self.ntor.clone()
        } else if self.ntor.is_negative() {
            // For negative: divide and subtract 1
            &self.ntor / &self.dtor - BigInt::one()
        } else {
            &self.ntor / &self.dtor
        }
    }

    /// Ceil: smallest integer k such that k >= self
    pub fn ceil(&self) -> BigInt {
        if self.dtor.is_one() {
            self.ntor.clone()
        } else if self.ntor.is_positive() {
            &self.ntor / &self.dtor + BigInt::one()
        } else {
            &self.ntor / &self.dtor
        }
    }

    /// Round: closest integer, rounding 0.5 up (towards +inf)
    pub fn round(&self) -> BigInt {
        self.add_frac(&BigFraction::half()).floor()
    }

    pub fn is_zero(&self) -> bool {
        self.ntor.is_zero()
    }
}

impl PartialEq for BigFraction {
    fn eq(&self, other: &Self) -> bool {
        self.ntor == other.ntor && self.dtor == other.dtor
    }
}

impl Eq for BigFraction {}

impl PartialOrd for BigFraction {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BigFraction {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.ntor * &other.dtor).cmp(&(&other.ntor * &self.dtor))
    }
}

impl fmt::Display for BigFraction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.dtor.is_one() {
            write!(f, "{}", self.ntor)
        } else {
            write!(f, "{}/{}", self.ntor, self.dtor)
        }
    }
}

impl From<i64> for BigFraction {
    fn from(n: i64) -> Self {
        BigFraction::from_i64(n)
    }
}

impl From<BigInt> for BigFraction {
    fn from(n: BigInt) -> Self {
        BigFraction::from_bigint(n)
    }
}
