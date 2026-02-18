use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::Zero;

pub const MASK_8: i64 = 0xFF;
pub const MASK_16: i64 = 0xFFFF;
pub const MASK_32: i64 = 0xFFFF_FFFF;
pub const MASK_48: i64 = 0xFFFF_FFFF_FFFF;

pub fn get_pow2(bits: u32) -> i64 {
    1i64 << bits
}

pub fn get_mask(bits: u32) -> i64 {
    if bits >= 64 {
        !0i64
    } else {
        (1i64 << bits) - 1
    }
}

pub fn mask(value: i64, bits: u32) -> i64 {
    value & get_mask(bits)
}

pub fn mask_signed(value: i64, bits: u32) -> i64 {
    (value << (64 - bits)) >> (64 - bits)
}

/// Modular inverse mod 2^bits using Newton's method
pub fn mod_inverse(value: i64, bits: u32) -> i64 {
    let mut x = ((((value << 1) ^ value) & 4) << 1) ^ value;
    x = x.wrapping_mul(2i64.wrapping_sub(value.wrapping_mul(x)));
    x = x.wrapping_mul(2i64.wrapping_sub(value.wrapping_mul(x)));
    x = x.wrapping_mul(2i64.wrapping_sub(value.wrapping_mul(x)));
    x = x.wrapping_mul(2i64.wrapping_sub(value.wrapping_mul(x)));
    mask(x, bits)
}

/// Modular inverse mod 2^16 (simpler version used in PopulationReverser)
pub fn mod_inverse_16(x: i64) -> i64 {
    if (x & 1) == 0 {
        panic!("x is not coprime with the modulus");
    }
    let mut inv: i64 = 0;
    let mut b: i64 = 1;
    for i in 0..16 {
        if (b & 1) == 1 {
            inv |= 1i64 << i;
            b = (b - x) >> 1;
        } else {
            b >>= 1;
        }
    }
    inv
}

pub fn lcm_bigint(a: &BigInt, b: &BigInt) -> BigInt {
    let g = a.gcd(b);
    if g.is_zero() {
        BigInt::zero()
    } else {
        a * (b / g)
    }
}
