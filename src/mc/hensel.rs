use crate::math::mth;

/// Hensel lifting for seed reversal.
/// Port of mc_math's Hensel.java.
pub fn lift(
    value: i64,
    bit: i32,
    target: i64,
    bits: i32,
    offset: i32,
    hash: &dyn Fn(i64) -> i64,
    result: &mut Vec<i64>,
) {
    if bit >= bits {
        if mth::mask(target, (bit + offset) as u32) == mth::mask(hash(value), (bit + offset) as u32) {
            result.push(value);
        }
    } else if mth::mask(target, bit as u32) == mth::mask(hash(value), bit as u32) {
        lift(value, bit + 1, target, bits, offset, hash, result);
        lift(
            value | mth::get_pow2((bit + offset) as u32),
            bit + 1,
            target,
            bits,
            offset,
            hash,
            result,
        );
    }
}
