use crate::lcg::lcg::LCG;
use crate::math::mth;
use crate::mc::chunk_rand::{ChunkRand, MCVersion};
use crate::mc::hensel;
use std::collections::HashSet;

/// Port of mc_reversal's PopulationReverser + ChunkRandomReverser.reversePopulationSeed.

const M1: i64 = LCG::JAVA.multiplier;

// Precompute LCG combine(2) and combine(4). We compute these once per call
// instead of using lazy_static, since const fn isn't available for combine.
fn lcg_params() -> (i64, i64, i64, i64) {
    let lcg2 = LCG::JAVA.combine(2);
    let lcg4 = LCG::JAVA.combine(4);
    (lcg2.multiplier, lcg2.addend, lcg4.multiplier, lcg4.addend)
}

/// Reverse population seed to world seeds.
/// This is the main entry point, matching ChunkRandomReverser.reversePopulationSeed.
pub fn reverse_population_seed(population_seed: i64, x: i32, z: i32, version: MCVersion) -> Vec<i64> {
    let pop_seed = population_seed & mth::MASK_48;

    if version.is_older_than(MCVersion::V1_13) {
        return get_seed_from_chunkseed_pre13(pop_seed, x, z);
    }

    reverse(pop_seed, x, z, version)
}

fn reverse(population_seed: i64, x: i32, z: i32, version: MCVersion) -> Vec<i64> {
    let (m2_val, a2_val, m4_val, a4_val) = lcg_params();

    // Precompute lookup tables (same as Java's static init)
    // For efficiency we compute on the fly instead of full 65536 tables.

    let mut world_seeds = Vec::new();
    let mut rand = ChunkRand::new();

    let e = population_seed & mth::MASK_32;
    let f = population_seed & mth::MASK_16;

    let free_bits = (x as i64 | z as i64).trailing_zeros();
    let mut c: i64 = mth::mask(population_seed, free_bits);
    let next_bit = if free_bits == 64 {
        0
    } else {
        (x as i64 ^ z as i64 ^ population_seed) & mth::get_pow2(free_bits)
    };
    c |= next_bit;
    let free_bits = free_bits + 1;
    let increment = mth::get_pow2(free_bits) as i64;

    let first_multiplier = (m2_val.wrapping_mul(x as i64).wrapping_add(m4_val.wrapping_mul(z as i64))) & mth::MASK_16;
    let mult_trailing_zeroes = first_multiplier.trailing_zeros();

    if mult_trailing_zeroes >= 16 {
        // Special case: use Hensel lifting
        let pop_hash = |value: i64| -> i64 {
            let mut r = ChunkRand::new();
            r.set_population_seed(value, x, z, version)
        };

        if free_bits >= 16 {
            hensel::lift(c, (free_bits as i32) - 16, population_seed, 32, 16, &pop_hash, &mut world_seeds);
        } else {
            let mut c_iter = c;
            while c_iter < (1i64 << 16) {
                hensel::lift(c_iter, 0, population_seed, 32, 16, &pop_hash, &mut world_seeds);
                c_iter += increment;
            }
        }

        return world_seeds;
    }

    let first_mult_inv = mth::mod_inverse_16(first_multiplier >> mult_trailing_zeroes);

    let offsets = get_offsets(x, z, version);

    while c < (1i64 << 16) {
        let target = (c ^ f) & mth::MASK_16;
        let x_term = ((m2_val.wrapping_mul((c ^ M1) & mth::MASK_16).wrapping_add(a2_val)) as u64 >> 16) as i64;
        let z_term = ((m4_val.wrapping_mul((c ^ M1) & mth::MASK_16).wrapping_add(a4_val)) as u64 >> 16) as i64;
        let magic = (x as i64).wrapping_mul(x_term).wrapping_add((z as i64).wrapping_mul(z_term));

        for &offset in &offsets {
            add_world_seeds(
                target.wrapping_sub((magic.wrapping_add(offset)) & mth::MASK_16),
                mult_trailing_zeroes,
                first_mult_inv,
                c,
                e,
                x,
                z,
                population_seed,
                &mut world_seeds,
                &mut rand,
                version,
            );
        }

        c += increment;
    }

    world_seeds
}

fn add_world_seeds(
    first_addend: i64,
    mult_trailing_zeroes: u32,
    first_mult_inv: i64,
    c: i64,
    e: i64,
    x: i32,
    z: i32,
    population_seed: i64,
    world_seeds: &mut Vec<i64>,
    rand: &mut ChunkRand,
    version: MCVersion,
) {
    if (first_addend.trailing_zeros()) < mult_trailing_zeroes {
        return;
    }

    let mask = mth::get_mask(16 - mult_trailing_zeroes);
    let increment = mth::get_pow2(16 - mult_trailing_zeroes);

    let mut b = (((first_mult_inv.wrapping_mul(first_addend)) >> mult_trailing_zeroes) ^ (M1 >> 16)) & mask;

    while b < (1i64 << 16) {
        let k = (b << 16) + c;
        let target2 = (k ^ e) >> 16;
        let second_addend = get_partial_addend(k, x, z, 32, version) & mth::MASK_16;

        if (target2.wrapping_sub(second_addend)).trailing_zeros() < mult_trailing_zeroes {
            b += increment;
            continue;
        }

        let mut a = (((first_mult_inv.wrapping_mul(target2.wrapping_sub(second_addend))) >> mult_trailing_zeroes) ^ (M1 >> 32)) & mask;

        while a < (1i64 << 16) {
            let ws = (a << 32) + k;
            if rand.set_population_seed(ws, x, z, version) == population_seed {
                world_seeds.push(ws);
            }
            a += increment;
        }

        b += increment;
    }
}

fn get_offsets(x: i32, z: i32, version: MCVersion) -> HashSet<i64> {
    let mut offsets = HashSet::new();

    if version.is_older_than(MCVersion::V1_13) {
        for i in 0..3i64 {
            for j in 0..3i64 {
                offsets.insert((x as i64).wrapping_mul(i).wrapping_add((z as i64).wrapping_mul(j)));
            }
        }
    } else {
        for i in 0..2i64 {
            for j in 0..2i64 {
                offsets.insert((x as i64).wrapping_mul(i).wrapping_add((z as i64).wrapping_mul(j)));
            }
        }
    }

    offsets
}

fn get_partial_addend(partial_seed: i64, x: i32, z: i32, bits: u32, version: MCVersion) -> i64 {
    let (m2_val, a2_val, m4_val, a4_val) = lcg_params();

    let mask = mth::get_mask(bits);
    let a = ((m2_val.wrapping_mul((partial_seed ^ M1) & mask).wrapping_add(a2_val)) & mth::MASK_48) >> 16;
    let b = ((m4_val.wrapping_mul((partial_seed ^ M1) & mask).wrapping_add(a4_val)) & mth::MASK_48) >> 16;

    if version.is_older_than(MCVersion::V1_13) {
        return (x as i64).wrapping_mul(a / 2 * 2 + 1)
            .wrapping_add((z as i64).wrapping_mul(b / 2 * 2 + 1));
    }

    ((x as i64).wrapping_mul(a | 1).wrapping_add((z as i64).wrapping_mul(b | 1))) >> 16
}

// ---- Pre-1.13 reversal ----

fn get_chunkseed_pre13(seed: i64, x: i32, z: i32) -> i64 {
    use crate::mc::jrand::JRand;
    let mut r = JRand::new(seed);
    let a = r.next_long() / 2 * 2 + 1;
    let b = r.next_long() / 2 * 2 + 1;
    ((x as i64).wrapping_mul(a).wrapping_add((z as i64).wrapping_mul(b)) ^ seed) & ((1i64 << 48) - 1)
}

fn get_partial_addend_pre13(partial_seed: i64, x: i32, z: i32, bits: u32) -> i64 {
    let (m2_val, a2_val, m4_val, a4_val) = lcg_params();
    let mask = mth::get_mask(bits);

    let av = ((m2_val.wrapping_mul((partial_seed ^ M1) & mask).wrapping_add(a2_val)) & mth::MASK_48) >> 16;
    let bv = ((m4_val.wrapping_mul((partial_seed ^ M1) & mask).wrapping_add(a4_val)) & mth::MASK_48) >> 16;

    (x as i64).wrapping_mul(av as i32 as i64 / 2 * 2 + 1)
        .wrapping_add((z as i64).wrapping_mul(bv as i32 as i64 / 2 * 2 + 1))
}

fn add_world_seed_pre13(
    first_addend: i64,
    mult_trailing_zeroes: u32,
    first_mult_inv: i64,
    c: i64,
    x: i32,
    z: i32,
    chunkseed: i64,
    worldseeds: &mut Vec<i64>,
) {
    let bottom32 = chunkseed & mth::MASK_32;

    if first_addend.trailing_zeros() >= mult_trailing_zeroes {
        let mut b = ((first_mult_inv.wrapping_mul(first_addend) >> mult_trailing_zeroes) ^ (M1 >> 16)) & mth::get_mask(16 - mult_trailing_zeroes);

        if mult_trailing_zeroes != 0 {
            let small_mask = mth::get_mask(mult_trailing_zeroes);
            let small_mult_inverse = small_mask & first_mult_inv;
            let target = (((b ^ (bottom32 >> 16)) & small_mask)
                .wrapping_sub(get_partial_addend_pre13((b << 16) + c, x, z, 32 - mult_trailing_zeroes) >> 16))
                & small_mask;
            b += (((target.wrapping_mul(small_mult_inverse)) ^ (M1 >> (32 - mult_trailing_zeroes))) & small_mask)
                << (16 - mult_trailing_zeroes);
        }

        let bottom32_seed = (b << 16) + c;
        let target2 = (bottom32_seed ^ bottom32) >> 16;
        let second_addend = (get_partial_addend_pre13(bottom32_seed, x, z, 32) >> 16) & mth::MASK_16;

        let mut top_bits = (((first_mult_inv.wrapping_mul(target2.wrapping_sub(second_addend))) >> mult_trailing_zeroes) ^ (M1 >> 32)) & mth::get_mask(16 - mult_trailing_zeroes);

        while top_bits < (1i64 << 16) {
            let ws = (top_bits << 32) + bottom32_seed;
            if get_chunkseed_pre13(ws, x, z) == chunkseed {
                worldseeds.push(ws);
            }
            top_bits += 1i64 << (16 - mult_trailing_zeroes);
        }
    }
}

fn get_seed_from_chunkseed_pre13(chunkseed: i64, x: i32, z: i32) -> Vec<i64> {
    let mut worldseeds = Vec::new();

    if x == 0 && z == 0 {
        worldseeds.push(chunkseed);
        return worldseeds;
    }

    let _e = chunkseed & mth::MASK_32;
    let f = chunkseed & mth::MASK_16;

    let (m2_val, a2_val, m4_val, a4_val) = lcg_params();

    let first_multiplier = (m2_val.wrapping_mul(x as i64).wrapping_add(m4_val.wrapping_mul(z as i64))) & mth::MASK_16;
    let mult_trailing_zeroes = first_multiplier.trailing_zeros();
    let first_mult_inv = mth::mod_inverse_16(first_multiplier >> mult_trailing_zeroes);

    let xcount = (x as i64).trailing_zeros();
    let zcount = (z as i64).trailing_zeros();
    let total_count = (x as i64 | z as i64).trailing_zeros();

    let mut possible_offsets = HashSet::new();
    for i in 0..3i64 {
        for j in 0..3i64 {
            possible_offsets.insert((x as i64).wrapping_mul(i).wrapping_add(j.wrapping_mul(z as i64)));
        }
    }

    let mut c: i64 = if xcount == zcount {
        chunkseed & ((1 << (xcount + 1)) - 1)
    } else {
        (chunkseed & ((1 << (total_count + 1)) - 1)) ^ (1 << total_count)
    };

    while c < (1i64 << 16) {
        let target = (c ^ f) & mth::MASK_16;
        let magic = (x as i64).wrapping_mul(((m2_val.wrapping_mul((c ^ M1) & mth::MASK_16).wrapping_add(a2_val)) as u64 >> 16) as i64)
            .wrapping_add((z as i64).wrapping_mul(((m4_val.wrapping_mul((c ^ M1) & mth::MASK_16).wrapping_add(a4_val)) as u64 >> 16) as i64));

        for &offset in &possible_offsets {
            add_world_seed_pre13(
                target.wrapping_sub((magic.wrapping_add(offset)) & mth::MASK_16),
                mult_trailing_zeroes,
                first_mult_inv,
                c,
                x,
                z,
                chunkseed,
                &mut worldseeds,
            );
        }

        c += 1 << (total_count + 1);
    }

    worldseeds
}
