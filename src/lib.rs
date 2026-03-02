//! # Dungeon Cracker
//!
//! A Rust port of the Minecraft dungeon seed cracker. Recovers world seeds from
//! observed dungeon floor tile patterns by reversing Java's `java.util.Random` LCG
//! using lattice-based techniques (LLL reduction).
//!
//! ## Overview
//!
//! Given a spawner position and the floor pattern of a Minecraft dungeon, this
//! crate reconstructs the sequence of `nextInt()` calls that generated the floor,
//! builds a lattice of constraints on the internal RNG state, and uses LLL basis
//! reduction to efficiently enumerate candidate seeds.
//!
//! The pipeline is: **floor pattern → dungeon seeds → structure seeds → world seeds**.

use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag controlling verbose (internal) log output.
/// When `false` (the default), library-internal progress messages are suppressed.
pub static VERBOSE: AtomicBool = AtomicBool::new(false);

/// Set the global verbose logging flag.
pub fn set_verbose(v: bool) {
    VERBOSE.store(v, Ordering::Relaxed);
}

/// Check whether verbose logging is enabled.
#[inline]
pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

/// Like `eprintln!`, but only prints when the global `VERBOSE` flag is set.
#[macro_export]
macro_rules! verbose_eprintln {
    ($($arg:tt)*) => {
        if $crate::is_verbose() {
            eprintln!($($arg)*);
        }
    };
}

/// Exact rational arithmetic, matrix operations, LU decomposition, and linear programming.
pub mod math;
/// Linear congruential generator (LCG) types and Java `Random` state model.
pub mod lcg;
/// LLL lattice basis reduction and bounded lattice point enumeration.
pub mod lattice;
/// `java.util.Random` seed reverser using lattice techniques.
pub mod reverser;
/// Minecraft-specific RNG: `JRand`, `ChunkRand`, population/structure seed reversal.
pub mod mc;
/// Dungeon floor parsing and the top-level cracking entry points.
pub mod dungeon;

#[cfg(feature = "wasm")]
pub mod wasm;
