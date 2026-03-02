// Abstraction layer over exact rational numbers (fractions).
//
// When the `gmp` feature is enabled (native builds), this uses `rug::Rational`
// which is backed by GMP's mpq for performance — canonicalization (GCD) is
// done in highly-optimized C/assembly.
//
// When `gmp` is not enabled (WASM builds), this uses a hand-rolled
// `BigFractionInner` backed by `num_bigint::BigInt`.

use super::int_type::Int;
#[cfg(not(feature = "gmp"))]
use super::int_type::IntOps;
#[cfg(not(feature = "gmp"))]
use std::cmp::Ordering;
#[cfg(not(feature = "gmp"))]
use std::fmt;

// ─── Type alias ──────────────────────────────────────────────────────────────

#[cfg(feature = "gmp")]
pub type BigFraction = rug::Rational;

#[cfg(not(feature = "gmp"))]
pub type BigFraction = BigFractionInner;

// ─── Unified trait ───────────────────────────────────────────────────────────

/// Trait providing a unified API over both rational backends.
pub trait FracOps: Sized {
    fn frac_zero() -> Self;
    fn frac_one() -> Self;
    fn frac_minus_one() -> Self;
    fn frac_half() -> Self;
    fn frac_new(numerator: i64, denominator: i64) -> Self;
    fn frac_from_i64(n: i64) -> Self;
    fn frac_from_int(n: Int) -> Self;

    /// Backwards-compatible alias for `frac_from_int`.
    fn frac_from_bigint(n: Int) -> Self {
        Self::frac_from_int(n)
    }

    fn add_frac(&self, other: &Self) -> Self;
    fn sub_frac(&self, other: &Self) -> Self;
    fn mul_frac(&self, other: &Self) -> Self;
    fn div_frac(&self, other: &Self) -> Self;

    fn add_int(&self, other: &Int) -> Self;
    fn sub_int(&self, other: &Int) -> Self;
    fn mul_int(&self, other: &Int) -> Self;
    fn div_int(&self, other: &Int) -> Self;

    /// Backwards-compatible aliases.
    fn add_bigint(&self, other: &Int) -> Self { self.add_int(other) }
    fn sub_bigint(&self, other: &Int) -> Self { self.sub_int(other) }
    fn mul_bigint(&self, other: &Int) -> Self { self.mul_int(other) }
    fn div_bigint(&self, other: &Int) -> Self { self.div_int(other) }

    fn negate(&self) -> Self;
    fn reciprocal(&self) -> Self;
    fn frac_abs(&self) -> Self;
    fn signum(&self) -> i32;
    fn is_zero(&self) -> bool;

    fn numerator_int(&self) -> Int;

    /// Floor: largest integer k such that k <= self.
    fn floor(&self) -> Int;
    /// Ceil: smallest integer k such that k >= self.
    fn ceil(&self) -> Int;
    /// Round: closest integer, rounding 0.5 up (towards +inf).
    fn round(&self) -> Int;
}

// ─── rug / GMP backend (rug::Rational) ──────────────────────────────────────

#[cfg(feature = "gmp")]
mod rug_frac_impl {
    use super::*;
    use rug::{Integer, Rational};

    impl FracOps for Rational {
        fn frac_zero() -> Self { Rational::new() }
        fn frac_one() -> Self { Rational::from(1) }
        fn frac_minus_one() -> Self { Rational::from(-1) }
        fn frac_half() -> Self { Rational::from((1, 2)) }

        fn frac_new(numerator: i64, denominator: i64) -> Self {
            assert!(denominator != 0, "Division by zero");
            Rational::from((numerator, denominator))
        }

        fn frac_from_i64(n: i64) -> Self { Rational::from(n) }

        fn frac_from_int(n: Int) -> Self { Rational::from(n) }

        fn add_frac(&self, other: &Self) -> Self { Rational::from(self + other) }
        fn sub_frac(&self, other: &Self) -> Self { Rational::from(self - other) }
        fn mul_frac(&self, other: &Self) -> Self { Rational::from(self * other) }
        fn div_frac(&self, other: &Self) -> Self { Rational::from(self / other) }

        fn add_int(&self, other: &Int) -> Self { Rational::from(self + other) }
        fn sub_int(&self, other: &Int) -> Self { Rational::from(self - other) }
        fn mul_int(&self, other: &Int) -> Self { Rational::from(self * other) }
        fn div_int(&self, other: &Int) -> Self { Rational::from(self / other) }

        fn negate(&self) -> Self { Rational::from(-self) }

        fn reciprocal(&self) -> Self { Rational::from(self.recip_ref()) }

        fn frac_abs(&self) -> Self { Rational::from(self.abs_ref()) }

        fn signum(&self) -> i32 {
            use std::cmp::Ordering::*;
            match self.cmp0() {
                Greater => 1,
                Less => -1,
                Equal => 0,
            }
        }

        fn is_zero(&self) -> bool { *self == 0 }

        fn numerator_int(&self) -> Int {
            Integer::from(self.numer())
        }

        fn floor(&self) -> Int {
            // rug's trunc_ref truncates toward zero.
            // floor = trunc if non-negative or exact, else trunc - 1.
            let (fract, trunc) = Rational::from(self).fract_trunc(Integer::new());
            if fract >= 0 {
                trunc
            } else {
                trunc - 1
            }
        }

        fn ceil(&self) -> Int {
            let (fract, trunc) = Rational::from(self).fract_trunc(Integer::new());
            if fract <= 0 {
                trunc
            } else {
                trunc + 1
            }
        }

        fn round(&self) -> Int {
            let half_added = self.add_frac(&Self::frac_half());
            FracOps::floor(&half_added)
        }
    }

    // Note: rug::Rational already implements Display, From<i64>, From<Integer>, etc.
}

// ─── num-bigint backend (BigFractionInner) ───────────────────────────────────

/// Pure-Rust exact rational backed by `num_bigint::BigInt`.
/// Invariants after `simplify()`:
/// - denominator > 0
/// - gcd(|numerator|, denominator) == 1
/// - if numerator == 0 then denominator == 1
#[cfg(not(feature = "gmp"))]
#[derive(Clone, Debug)]
pub struct BigFractionInner {
    ntor: Int,
    dtor: Int,
}

#[cfg(not(feature = "gmp"))]
impl BigFractionInner {
    fn new_raw(ntor: Int, dtor: Int) -> Self {
        let mut f = BigFractionInner { ntor, dtor };
        f.simplify();
        f
    }

    fn simplify(&mut self) {
        if self.ntor.int_is_zero() {
            self.dtor = Int::int_one();
            return;
        }
        if self.dtor.int_is_negative() {
            self.ntor = self.ntor.int_neg();
            self.dtor = self.dtor.int_neg();
        }
        let g = self.ntor.int_gcd(&self.dtor);
        self.ntor = self.ntor.int_div(&g);
        self.dtor = self.dtor.int_div(&g);
    }
}

#[cfg(not(feature = "gmp"))]
impl FracOps for BigFractionInner {
    fn frac_zero() -> Self {
        BigFractionInner { ntor: Int::int_zero(), dtor: Int::int_one() }
    }

    fn frac_one() -> Self {
        BigFractionInner { ntor: Int::int_one(), dtor: Int::int_one() }
    }

    fn frac_minus_one() -> Self {
        BigFractionInner { ntor: Int::int_one().int_neg(), dtor: Int::int_one() }
    }

    fn frac_half() -> Self {
        BigFractionInner::new_raw(Int::int_one(), Int::int_from_i64(2))
    }

    fn frac_new(numerator: i64, denominator: i64) -> Self {
        let mut f = BigFractionInner {
            ntor: Int::int_from_i64(numerator),
            dtor: Int::int_from_i64(denominator),
        };
        assert!(!f.dtor.int_is_zero(), "Division by zero");
        f.simplify();
        f
    }

    fn frac_from_i64(n: i64) -> Self {
        BigFractionInner { ntor: Int::int_from_i64(n), dtor: Int::int_one() }
    }

    fn frac_from_int(n: Int) -> Self {
        BigFractionInner { ntor: n, dtor: Int::int_one() }
    }

    fn add_frac(&self, other: &Self) -> Self {
        Self::new_raw(
            self.ntor.int_mul(&other.dtor).int_add(&other.ntor.int_mul(&self.dtor)),
            self.dtor.int_mul(&other.dtor),
        )
    }

    fn sub_frac(&self, other: &Self) -> Self {
        Self::new_raw(
            self.ntor.int_mul(&other.dtor).int_sub(&other.ntor.int_mul(&self.dtor)),
            self.dtor.int_mul(&other.dtor),
        )
    }

    fn mul_frac(&self, other: &Self) -> Self {
        Self::new_raw(
            self.ntor.int_mul(&other.ntor),
            self.dtor.int_mul(&other.dtor),
        )
    }

    fn div_frac(&self, other: &Self) -> Self {
        Self::new_raw(
            self.ntor.int_mul(&other.dtor),
            self.dtor.int_mul(&other.ntor),
        )
    }

    fn add_int(&self, other: &Int) -> Self {
        Self::new_raw(
            self.ntor.int_add(&other.int_mul(&self.dtor)),
            self.dtor.clone(),
        )
    }

    fn sub_int(&self, other: &Int) -> Self {
        Self::new_raw(
            self.ntor.int_sub(&other.int_mul(&self.dtor)),
            self.dtor.clone(),
        )
    }

    fn mul_int(&self, other: &Int) -> Self {
        Self::new_raw(self.ntor.int_mul(other), self.dtor.clone())
    }

    fn div_int(&self, other: &Int) -> Self {
        Self::new_raw(self.ntor.clone(), self.dtor.int_mul(other))
    }

    fn negate(&self) -> Self {
        BigFractionInner { ntor: self.ntor.int_neg(), dtor: self.dtor.clone() }
    }

    fn reciprocal(&self) -> Self {
        Self::new_raw(self.dtor.clone(), self.ntor.clone())
    }

    fn frac_abs(&self) -> Self {
        if self.ntor.int_is_negative() { self.negate() } else { self.clone() }
    }

    fn signum(&self) -> i32 {
        if self.ntor.int_is_positive() { 1 }
        else if self.ntor.int_is_negative() { -1 }
        else { 0 }
    }

    fn is_zero(&self) -> bool {
        self.ntor.int_is_zero()
    }

    fn numerator_int(&self) -> Int {
        self.ntor.clone()
    }

    fn floor(&self) -> Int {
        if self.dtor.int_is_one() {
            self.ntor.clone()
        } else if self.ntor.int_is_negative() {
            self.ntor.int_div(&self.dtor).int_sub(&Int::int_one())
        } else {
            self.ntor.int_div(&self.dtor)
        }
    }

    fn ceil(&self) -> Int {
        if self.dtor.int_is_one() {
            self.ntor.clone()
        } else if self.ntor.int_is_positive() {
            self.ntor.int_div(&self.dtor).int_add(&Int::int_one())
        } else {
            self.ntor.int_div(&self.dtor)
        }
    }

    fn round(&self) -> Int {
        self.add_frac(&Self::frac_half()).floor()
    }
}

#[cfg(not(feature = "gmp"))]
impl PartialEq for BigFractionInner {
    fn eq(&self, other: &Self) -> bool {
        self.ntor == other.ntor && self.dtor == other.dtor
    }
}

#[cfg(not(feature = "gmp"))]
impl Eq for BigFractionInner {}

#[cfg(not(feature = "gmp"))]
impl PartialOrd for BigFractionInner {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[cfg(not(feature = "gmp"))]
impl Ord for BigFractionInner {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ntor.int_mul(&other.dtor).cmp(&other.ntor.int_mul(&self.dtor))
    }
}

#[cfg(not(feature = "gmp"))]
impl fmt::Display for BigFractionInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.dtor.int_is_one() {
            write!(f, "{}", self.ntor)
        } else {
            write!(f, "{}/{}", self.ntor, self.dtor)
        }
    }
}

#[cfg(not(feature = "gmp"))]
impl From<i64> for BigFractionInner {
    fn from(n: i64) -> Self { Self::frac_from_i64(n) }
}

#[cfg(not(feature = "gmp"))]
impl From<Int> for BigFractionInner {
    fn from(n: Int) -> Self { Self::frac_from_int(n) }
}
