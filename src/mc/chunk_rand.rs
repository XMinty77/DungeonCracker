use super::jrand::JRand;
use crate::math::mth;

/// Minecraft version enum (relevant for population seed calculation).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MCVersion {
    V1_8,
    V1_9,
    V1_10,
    V1_11,
    V1_12,
    V1_13,
    V1_14,
    V1_15,
    V1_16,
    V1_17,
}

impl MCVersion {
    pub fn is_older_than(&self, other: MCVersion) -> bool {
        (*self as u8) < (other as u8)
    }

    pub fn is_newer_than(&self, other: MCVersion) -> bool {
        (*self as u8) > (other as u8)
    }

    pub fn is_between(&self, lower: MCVersion, upper: MCVersion) -> bool {
        (*self as u8) >= (lower as u8) && (*self as u8) <= (upper as u8)
    }
}

/// Port of mc_core's ChunkRand.
#[derive(Clone, Debug)]
pub struct ChunkRand {
    pub jrand: JRand,
}

impl ChunkRand {
    pub fn new() -> Self {
        ChunkRand {
            jrand: JRand::of_internal_seed(0),
        }
    }

    /// Set the population seed. For 1.13+, uses |1L; for older, uses /2*2+1.
    /// `x` and `z` are the block coordinates of the negative-most corner of the chunk.
    pub fn set_population_seed(&mut self, world_seed: i64, x: i32, z: i32, version: MCVersion) -> i64 {
        self.jrand.set_seed(world_seed, true);
        let a: i64;
        let b: i64;

        if version.is_older_than(MCVersion::V1_13) {
            a = self.jrand.next_long() / 2 * 2 + 1;
            b = self.jrand.next_long() / 2 * 2 + 1;
        } else {
            a = self.jrand.next_long() | 1;
            b = self.jrand.next_long() | 1;
        }

        let seed = (x as i64).wrapping_mul(a).wrapping_add((z as i64).wrapping_mul(b)) ^ world_seed;
        self.jrand.set_seed(seed, true);
        seed & mth::MASK_48
    }

    /// Set the decorator seed. Only for 1.13+.
    pub fn set_decorator_seed(&mut self, population_seed: i64, salt: i32, _version: MCVersion) -> i64 {
        let seed = population_seed.wrapping_add(salt as i64);
        self.jrand.set_seed(seed, true);
        seed & mth::MASK_48
    }
}
