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
