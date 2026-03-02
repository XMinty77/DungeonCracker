// Abstraction layer over arbitrary-precision integers.
//
// When the `gmp` feature is enabled (native builds), this uses `rug::Integer`
// which is backed by GMP for performance.
//
// When `gmp` is not enabled (WASM builds), this uses `num_bigint::BigInt`,
// which is pure Rust and portable.

#[cfg(feature = "gmp")]
pub use rug::Integer as Int;

#[cfg(not(feature = "gmp"))]
pub use num_bigint::BigInt as Int;

// ─── Unified trait for operations that differ between backends ────────────────

/// Trait providing a unified API over `num_bigint::BigInt` and `rug::Integer`.
pub trait IntOps: Sized {
    fn int_zero() -> Self;
    fn int_one() -> Self;
    fn int_from_i64(v: i64) -> Self;
    fn int_is_zero(&self) -> bool;
    fn int_is_positive(&self) -> bool;
    fn int_is_negative(&self) -> bool;
    fn int_is_one(&self) -> bool;
    fn int_abs(&self) -> Self;
    fn int_neg(&self) -> Self;
    fn int_gcd(&self, other: &Self) -> Self;
    fn int_modpow(&self, exp: &Self, modulus: &Self) -> Self;
    fn int_shr(&self, bits: u32) -> Self;
    fn int_to_i64(&self) -> i64;
    fn int_to_f64_approx(&self) -> f64;

    // Arithmetic (returns new value)
    fn int_add(&self, other: &Self) -> Self;
    fn int_sub(&self, other: &Self) -> Self;
    fn int_mul(&self, other: &Self) -> Self;
    fn int_div(&self, other: &Self) -> Self;
    fn int_rem(&self, other: &Self) -> Self;

    fn int_add_i64(&self, other: i64) -> Self {
        self.int_add(&Self::int_from_i64(other))
    }
    fn int_sub_i64(&self, other: i64) -> Self {
        self.int_sub(&Self::int_from_i64(other))
    }
}

// ─── num-bigint backend ──────────────────────────────────────────────────────

#[cfg(not(feature = "gmp"))]
mod num_impl {
    use super::*;
    use num_bigint::BigInt;
    use num_integer::Integer;
    use num_traits::{One, Signed, ToPrimitive, Zero};

    impl IntOps for BigInt {
        fn int_zero() -> Self { BigInt::zero() }
        fn int_one() -> Self { BigInt::one() }
        fn int_from_i64(v: i64) -> Self { BigInt::from(v) }
        fn int_is_zero(&self) -> bool { self.is_zero() }
        fn int_is_positive(&self) -> bool { self.is_positive() }
        fn int_is_negative(&self) -> bool { self.is_negative() }
        fn int_is_one(&self) -> bool { *self == BigInt::one() }
        fn int_abs(&self) -> Self { Signed::abs(self) }
        fn int_neg(&self) -> Self { -self }
        fn int_gcd(&self, other: &Self) -> Self { Integer::gcd(self, other) }
        fn int_modpow(&self, exp: &Self, modulus: &Self) -> Self { self.modpow(exp, modulus) }

        fn int_shr(&self, bits: u32) -> Self { self >> bits as usize }

        fn int_to_i64(&self) -> i64 {
            // For seed values we may need wrapping behaviour.
            let bytes = self.to_signed_bytes_le();
            let mut result: i64 = 0;
            for (i, &b) in bytes.iter().enumerate().take(8) {
                result |= (b as u8 as i64) << (i * 8);
            }
            if self.is_negative() && bytes.len() < 8 {
                for i in bytes.len()..8 {
                    result |= 0xFFi64 << (i * 8);
                }
            }
            result
        }

        fn int_to_f64_approx(&self) -> f64 {
            ToPrimitive::to_f64(self).unwrap_or(0.0)
        }

        fn int_add(&self, other: &Self) -> Self { self + other }
        fn int_sub(&self, other: &Self) -> Self { self - other }
        fn int_mul(&self, other: &Self) -> Self { self * other }
        fn int_div(&self, other: &Self) -> Self { self / other }
        fn int_rem(&self, other: &Self) -> Self { self % other }
    }
}

// ─── rug / GMP backend ──────────────────────────────────────────────────────

#[cfg(feature = "gmp")]
mod rug_impl {
    use super::*;
    use rug::Integer;

    impl IntOps for Integer {
        fn int_zero() -> Self { Integer::from(0) }
        fn int_one() -> Self { Integer::from(1) }
        fn int_from_i64(v: i64) -> Self { Integer::from(v) }
        fn int_is_zero(&self) -> bool { *self == 0 }
        fn int_is_positive(&self) -> bool { *self > 0 }
        fn int_is_negative(&self) -> bool { *self < 0 }
        fn int_is_one(&self) -> bool { *self == 1 }
        fn int_abs(&self) -> Self { Integer::from(self.abs_ref()) }
        fn int_neg(&self) -> Self { Integer::from(-self) }
        fn int_gcd(&self, other: &Self) -> Self { Integer::from(self.gcd_ref(other)) }

        fn int_modpow(&self, exp: &Self, modulus: &Self) -> Self {
            Integer::from(self.pow_mod_ref(exp, modulus).unwrap())
        }

        fn int_shr(&self, bits: u32) -> Self { Integer::from(self >> bits) }

        fn int_to_i64(&self) -> i64 {
            self.to_i64().unwrap_or_else(|| {
                // Fallback: extract low 64 bits with sign
                let abs = Integer::from(self.abs_ref());
                let mask = Integer::from(u64::MAX);
                let low = Integer::from(&abs & &mask);
                let v = low.to_u64().unwrap_or(0) as i64;
                if *self < 0 { -v } else { v }
            })
        }

        fn int_to_f64_approx(&self) -> f64 {
            self.to_f64()
        }

        fn int_add(&self, other: &Self) -> Self { Integer::from(self + other) }
        fn int_sub(&self, other: &Self) -> Self { Integer::from(self - other) }
        fn int_mul(&self, other: &Self) -> Self { Integer::from(self * other) }
        fn int_div(&self, other: &Self) -> Self { Integer::from(self / other) }
        fn int_rem(&self, other: &Self) -> Self { Integer::from(self % other) }
    }
}
