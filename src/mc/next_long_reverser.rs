use crate::mc::jrand::JRand;

/// Port of mc_core's NextLongReverser.
/// Converts 48-bit structure seeds to 64-bit world seeds by reversing nextLong().

/// Returns seeds which produce nextLongs congruent to the structure seed.
pub fn get_seeds(structure_seed: i64) -> Vec<i64> {
    let mut seeds = Vec::with_capacity(2);
    add_seeds_to_list(structure_seed, &mut seeds);
    seeds
}

/// Returns the nextLong equivalents (world seeds) for a 48-bit structure seed.
pub fn get_next_long_equivalents(structure_seed: i64) -> Vec<i64> {
    let mut next_longs = Vec::with_capacity(2);
    for seed in get_seeds(structure_seed) {
        let mut r = JRand::of_internal_seed(seed);
        next_longs.push(r.next_long());
    }
    next_longs
}

fn add_seeds_to_list(structure_seed: i64, seed_list: &mut Vec<i64>) {
    let lower_bits = structure_seed & 0xffff_ffffi64;
    let mut upper_bits = (structure_seed as u64 >> 32) as i64;

    // Did the lower bits affect the upper bits
    if (lower_bits & 0x8000_0000i64) != 0 {
        upper_bits += 1;
    }

    let bits_of_danger: i32 = 1;

    let low_min = lower_bits << (16 - bits_of_danger);
    let low_max = ((lower_bits + 1) << (16 - bits_of_danger)) - 1;
    let upper_min = ((upper_bits << 16) - 107048004364969i64) >> bits_of_danger;

    let m1lv = floor_div(
        low_max.wrapping_mul(-33441).wrapping_add(upper_min.wrapping_mul(17549)),
        1i64 << (31 - bits_of_danger),
    ) + 1;
    let m2lv = floor_div(
        low_min.wrapping_mul(46603).wrapping_add(upper_min.wrapping_mul(39761)),
        1i64 << (32 - bits_of_danger),
    ) + 1;

    // (0,0)
    let seed = (-39761i64).wrapping_mul(m1lv).wrapping_add(35098i64.wrapping_mul(m2lv));
    if (46603i64.wrapping_mul(m1lv).wrapping_add(66882i64.wrapping_mul(m2lv)).wrapping_add(107048004364969i64) as u64 >> 16) as i64 == upper_bits {
        if (seed as u64 >> 16) as i64 == lower_bits {
            seed_list.push(
                (254681119335897i64.wrapping_mul(seed).wrapping_add(120305458776662i64))
                    & 0xffff_ffff_ffffi64,
            );
        }
    }

    // (1,0)
    let seed = (-39761i64).wrapping_mul(m1lv + 1).wrapping_add(35098i64.wrapping_mul(m2lv));
    if (46603i64.wrapping_mul(m1lv + 1).wrapping_add(66882i64.wrapping_mul(m2lv)).wrapping_add(107048004364969i64) as u64 >> 16) as i64 == upper_bits {
        if (seed as u64 >> 16) as i64 == lower_bits {
            seed_list.push(
                (254681119335897i64.wrapping_mul(seed).wrapping_add(120305458776662i64))
                    & 0xffff_ffff_ffffi64,
            );
        }
    }

    // (0,1)
    let seed = (-39761i64).wrapping_mul(m1lv).wrapping_add(35098i64.wrapping_mul(m2lv + 1));
    if (46603i64.wrapping_mul(m1lv).wrapping_add(66882i64.wrapping_mul(m2lv + 1)).wrapping_add(107048004364969i64) as u64 >> 16) as i64 == upper_bits {
        if (seed as u64 >> 16) as i64 == lower_bits {
            seed_list.push(
                (254681119335897i64.wrapping_mul(seed).wrapping_add(120305458776662i64))
                    & 0xffff_ffff_ffffi64,
            );
        }
    }
}

/// Java's Math.floorDiv
fn floor_div(x: i64, y: i64) -> i64 {
    let r = x / y;
    // If the signs are different and there's a remainder, subtract 1
    if (x ^ y) < 0 && (r * y != x) {
        r - 1
    } else {
        r
    }
}
