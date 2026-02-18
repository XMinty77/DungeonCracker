use crate::dungeon::dungeon_data_parser::DungeonDataParser;
use crate::dungeon::reverser_instruction::{InstructionType, ReverserInstruction};
use crate::lcg::lcg::LCG;
use crate::lcg::rand::Rand;
use crate::math::mth;
use crate::mc::chunk_rand::{ChunkRand, MCVersion};
use crate::mc::next_long_reverser;
use crate::mc::population_reverser;
use crate::reverser::filtered_skip::FilteredSkip;
use crate::reverser::random_reverser::JavaRandomReverser;
use std::collections::HashSet;

/// Biome type affecting salt values.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BiomeType {
    NotDesert,
    Desert,
    Unknown,
}

/// Floor size options.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FloorSize {
    _9x9,
    _7x9,
    _9x7,
    _7x7,
}

impl FloorSize {
    pub fn x_min(&self) -> usize {
        match self {
            FloorSize::_7x7 | FloorSize::_7x9 => 1,
            _ => 0,
        }
    }

    pub fn z_min(&self) -> usize {
        match self {
            FloorSize::_7x7 | FloorSize::_9x7 => 1,
            _ => 0,
        }
    }

    pub fn x_max(&self) -> usize {
        match self {
            FloorSize::_7x7 | FloorSize::_7x9 => 8,
            _ => 9,
        }
    }

    pub fn z_max(&self) -> usize {
        match self {
            FloorSize::_7x7 | FloorSize::_9x7 => 8,
            _ => 9,
        }
    }
}

/// The result of a dungeon cracking operation.
pub struct CrackResult {
    pub dungeon_seeds: Vec<i64>,
    pub structure_seeds: Vec<i64>,
    pub world_seeds: Vec<i64>,
}

/// Info about the search space, returned by the prepare step.
pub struct PrepareResult {
    pub total_branches: i64,
    pub possibilities: usize,
    pub dimensions: usize,
    pub info_bits: f32,
}

/// Convert a 2D floor grid (row-major: [z][x], 9x9) into the column-major sequence string.
/// This mirrors Floor.getSequence() from Java: x outer, z inner, reading floorPattern[z][x].
pub fn get_sequence(floor: &[[u8; 9]; 9], floor_size: FloorSize) -> String {
    let mut seq = String::new();
    for x in floor_size.x_min()..floor_size.x_max() {
        for z in floor_size.z_min()..floor_size.z_max() {
            seq.push_str(&floor[z][x].to_string());
        }
    }
    seq
}

/// Main cracking function.
/// `floor_sequence` is the sequence string (from get_sequence or directly provided).
pub fn crack_dungeon(
    spawner_x: i32,
    spawner_y: i32,
    spawner_z: i32,
    version: MCVersion,
    biome: BiomeType,
    floor_sequence: &str,
) -> Result<CrackResult, String> {
    let salts = get_salts(version, biome);

    let possibilities = DungeonDataParser::get_all_possibilities(floor_sequence)
        .ok_or_else(|| "Too many possibilities (>128 unknown permutations)".to_string())?;

    eprintln!("[info] Generated {} floor interpretation(s)", possibilities.len());

    let offset_x = spawner_x & 15;
    let y = spawner_y;
    let offset_z = spawner_z & 15;
    eprintln!("[info] Offsets: x={}, y={}, z={}", offset_x, y, offset_z);

    let mut struct_seeds_set = HashSet::new();
    let mut dungeon_seeds_set = HashSet::new();

    for (poss_idx, program) in possibilities.iter().enumerate() {
        eprintln!("[progress] Processing possibility {}/{} ({} instructions)...", poss_idx + 1, possibilities.len(), program.len());
        // Build the DynamicProgram equivalent
        let mut filtered_skips: Vec<FilteredSkip> = Vec::new();
        let mut call_sequence: Vec<CallEntry> = Vec::new();
        let mut current_index: i64 = 0;

        // Spawner position calls
        if version.is_between(MCVersion::V1_8, MCVersion::V1_14) {
            // x, y, z order
            call_sequence.push(CallEntry::NextInt { bound: 16, value: offset_x });
            current_index += 1;
            call_sequence.push(CallEntry::NextInt { bound: 256, value: y });
            current_index += 1;
            call_sequence.push(CallEntry::NextInt { bound: 16, value: offset_z });
            current_index += 1;
        } else {
            // x, z, y order (1.15+)
            call_sequence.push(CallEntry::NextInt { bound: 16, value: offset_x });
            current_index += 1;
            call_sequence.push(CallEntry::NextInt { bound: 16, value: offset_z });
            current_index += 1;
            call_sequence.push(CallEntry::NextInt { bound: 256, value: y });
            current_index += 1;
        }

        // Skip 2 calls
        call_sequence.push(CallEntry::Skip { count: 2 });
        current_index += 2;

        // Floor calls
        let mut info_bits: f32 = 16.0;
        for instr in program {
            match instr.instruction_type {
                InstructionType::NextInt => {
                    call_sequence.push(CallEntry::NextIntEq { bound: 4, value: 0 });
                    info_bits += 2.0;
                    current_index += 1;
                }
                InstructionType::FilteredSkip => {
                    let idx = current_index;
                    filtered_skips.push(FilteredSkip::new(
                        idx,
                        Box::new(|r: &mut Rand| r.next_int(4) != 0),
                    ));
                    call_sequence.push(CallEntry::Skip { count: 1 });
                    info_bits += 0.4;
                    current_index += 1;
                }
                InstructionType::Skip => {
                    let count = instr.max_call_count as i64;
                    call_sequence.push(CallEntry::Skip { count });
                    current_index += count;
                }
                InstructionType::MutableSkip => {
                    // Should not appear after expansion
                    return Err("Mutable skip encountered during reverser setup".to_string());
                }
            }
        }

        if info_bits <= 32.0 {
            return Err("Not enough information in the floor pattern".to_string());
        }

        // Build the JavaRandomReverser
        let mut reverser = JavaRandomReverser::new(filtered_skips);
        for entry in &call_sequence {
            match entry {
                CallEntry::NextInt { bound, value } => {
                    reverser.add_next_int_call(*bound, *value, *value);
                }
                CallEntry::NextIntEq { bound, value } => {
                    reverser.add_next_int_call(*bound, *value, *value);
                }
                CallEntry::Skip { count } => {
                    reverser.add_unmeasured_seeds(*count);
                }
            }
        }

        eprintln!("[progress]   Built reverser with {} dimensions, info_bits={:.1}, success_chance={:.6}",
                 reverser.dimensions(), info_bits, reverser.success_chance());
        eprintln!("[progress]   Running find_all_valid_seeds (lattice reduction + enumeration)...");
        let dungeon_seeds_xored = reverser.find_all_valid_seeds();
        eprintln!("[progress]   Found {} candidate dungeon seed(s)", dungeon_seeds_xored.len());
        let mut rand = ChunkRand::new();

        for (ds_idx, seed) in dungeon_seeds_xored.iter().enumerate() {
            if ds_idx % 100 == 0 && ds_idx > 0 {
                eprintln!("[progress]   Processing dungeon seed {}/{}...", ds_idx, dungeon_seeds_xored.len());
            }
            dungeon_seeds_set.insert(*seed);

            for &salt in &salts {
                rand.jrand.set_seed(*seed, false);

                for _ in 0..8 {
                    let pop_seed = (rand.jrand.get_seed() ^ LCG::JAVA.multiplier) - salt;
                    let chunk_x = (spawner_x >> 4) << 4;
                    let chunk_z = (spawner_z >> 4) << 4;

                    let partial_struct_seeds =
                        population_reverser::reverse_population_seed(pop_seed, chunk_x, chunk_z, MCVersion::V1_14);

                    for ss in partial_struct_seeds {
                        let masked = ss & mth::MASK_48;
                        struct_seeds_set.insert(masked);
                    }

                    rand.jrand.advance(-5);
                }
            }
        }
    }

    // Convert structure seeds to world seeds
    eprintln!("[progress] All possibilities processed. {} dungeon seed(s), {} structure seed(s).",
             dungeon_seeds_set.len(), struct_seeds_set.len());
    eprintln!("[progress] Converting structure seeds to world seeds...");
    let mut world_seeds_set = HashSet::new();
    for struct_seed in &struct_seeds_set {
        let equivalents = next_long_reverser::get_next_long_equivalents(*struct_seed);
        for ws in equivalents {
            world_seeds_set.insert(ws);
        }
    }

    Ok(CrackResult {
        dungeon_seeds: dungeon_seeds_set.into_iter().collect(),
        structure_seeds: struct_seeds_set.into_iter().collect(),
        world_seeds: world_seeds_set.into_iter().collect(),
    })
}

/// Prepare the cracking: parse floor, build reverser, get branch count.
/// Returns the total number of depth-0 branches that can be split across workers.
pub fn prepare_crack(
    spawner_x: i32,
    spawner_y: i32,
    spawner_z: i32,
    version: MCVersion,
    _biome: BiomeType,
    floor_sequence: &str,
) -> Result<PrepareResult, String> {
    let possibilities = DungeonDataParser::get_all_possibilities(floor_sequence)
        .ok_or_else(|| "Too many possibilities (>128 unknown permutations)".to_string())?;

    if possibilities.is_empty() {
        return Err("No valid floor interpretations".to_string());
    }

    // We only parallelize the first possibility's enumeration (the main one).
    // Multiple possibilities are rare and handled sequentially.
    let program = &possibilities[0];

    let (reverser, info_bits) = build_reverser(spawner_x, spawner_y, spawner_z, version, program)?;
    let mut reverser = reverser;
    let branch_count = reverser.get_branch_count();

    Ok(PrepareResult {
        total_branches: branch_count,
        possibilities: possibilities.len(),
        dimensions: reverser.dimensions(),
        info_bits,
    })
}

/// Crack dungeon for a specific range of depth-0 branches.
/// Each worker calls this with a different [branch_start, branch_end) range.
pub fn crack_dungeon_partial(
    spawner_x: i32,
    spawner_y: i32,
    spawner_z: i32,
    version: MCVersion,
    biome: BiomeType,
    floor_sequence: &str,
    branch_start: i64,
    branch_end: i64,
) -> Result<CrackResult, String> {
    let salts = get_salts(version, biome);

    let possibilities = DungeonDataParser::get_all_possibilities(floor_sequence)
        .ok_or_else(|| "Too many possibilities (>128 unknown permutations)".to_string())?;

    let mut struct_seeds_set = HashSet::new();
    let mut dungeon_seeds_set = HashSet::new();

    for (poss_idx, program) in possibilities.iter().enumerate() {
        let (mut reverser, info_bits) = build_reverser(spawner_x, spawner_y, spawner_z, version, program)?;

        if info_bits <= 32.0 {
            return Err("Not enough information in the floor pattern".to_string());
        }

        eprintln!("[worker] Processing possibility {}/{}, branches [{}, {})",
                 poss_idx + 1, possibilities.len(), branch_start, branch_end);

        let dungeon_seeds_xored = reverser.find_seeds_for_branches(branch_start, branch_end);
        eprintln!("[worker] Found {} candidate dungeon seed(s)", dungeon_seeds_xored.len());

        let mut rand = ChunkRand::new();

        for seed in &dungeon_seeds_xored {
            dungeon_seeds_set.insert(*seed);

            for &salt in &salts {
                rand.jrand.set_seed(*seed, false);

                for _ in 0..8 {
                    let pop_seed = (rand.jrand.get_seed() ^ LCG::JAVA.multiplier) - salt;
                    let chunk_x = (spawner_x >> 4) << 4;
                    let chunk_z = (spawner_z >> 4) << 4;

                    let partial_struct_seeds =
                        population_reverser::reverse_population_seed(pop_seed, chunk_x, chunk_z, MCVersion::V1_14);

                    for ss in partial_struct_seeds {
                        let masked = ss & mth::MASK_48;
                        struct_seeds_set.insert(masked);
                    }

                    rand.jrand.advance(-5);
                }
            }
        }
    }

    // Convert structure seeds to world seeds
    let mut world_seeds_set = HashSet::new();
    for struct_seed in &struct_seeds_set {
        let equivalents = next_long_reverser::get_next_long_equivalents(*struct_seed);
        for ws in equivalents {
            world_seeds_set.insert(ws);
        }
    }

    Ok(CrackResult {
        dungeon_seeds: dungeon_seeds_set.into_iter().collect(),
        structure_seeds: struct_seeds_set.into_iter().collect(),
        world_seeds: world_seeds_set.into_iter().collect(),
    })
}

/// Build a JavaRandomReverser from a program (one possibility).
/// Returns (reverser, info_bits).
fn build_reverser(
    spawner_x: i32,
    spawner_y: i32,
    spawner_z: i32,
    version: MCVersion,
    program: &[ReverserInstruction],
) -> Result<(JavaRandomReverser, f32), String> {
    let offset_x = spawner_x & 15;
    let y = spawner_y;
    let offset_z = spawner_z & 15;

    let mut filtered_skips: Vec<FilteredSkip> = Vec::new();
    let mut call_sequence: Vec<CallEntry> = Vec::new();
    let mut current_index: i64 = 0;

    // Spawner position calls
    if version.is_between(MCVersion::V1_8, MCVersion::V1_14) {
        call_sequence.push(CallEntry::NextInt { bound: 16, value: offset_x });
        current_index += 1;
        call_sequence.push(CallEntry::NextInt { bound: 256, value: y });
        current_index += 1;
        call_sequence.push(CallEntry::NextInt { bound: 16, value: offset_z });
        current_index += 1;
    } else {
        call_sequence.push(CallEntry::NextInt { bound: 16, value: offset_x });
        current_index += 1;
        call_sequence.push(CallEntry::NextInt { bound: 16, value: offset_z });
        current_index += 1;
        call_sequence.push(CallEntry::NextInt { bound: 256, value: y });
        current_index += 1;
    }

    // Skip 2 calls
    call_sequence.push(CallEntry::Skip { count: 2 });
    current_index += 2;

    // Floor calls
    let mut info_bits: f32 = 16.0;
    for instr in program {
        match instr.instruction_type {
            InstructionType::NextInt => {
                call_sequence.push(CallEntry::NextIntEq { bound: 4, value: 0 });
                info_bits += 2.0;
                current_index += 1;
            }
            InstructionType::FilteredSkip => {
                let idx = current_index;
                filtered_skips.push(FilteredSkip::new(
                    idx,
                    Box::new(|r: &mut Rand| r.next_int(4) != 0),
                ));
                call_sequence.push(CallEntry::Skip { count: 1 });
                info_bits += 0.4;
                current_index += 1;
            }
            InstructionType::Skip => {
                let count = instr.max_call_count as i64;
                call_sequence.push(CallEntry::Skip { count });
                current_index += count;
            }
            InstructionType::MutableSkip => {
                return Err("Mutable skip encountered during reverser setup".to_string());
            }
        }
    }

    // Build the JavaRandomReverser
    let mut reverser = JavaRandomReverser::new(filtered_skips);
    for entry in &call_sequence {
        match entry {
            CallEntry::NextInt { bound, value } => {
                reverser.add_next_int_call(*bound, *value, *value);
            }
            CallEntry::NextIntEq { bound, value } => {
                reverser.add_next_int_call(*bound, *value, *value);
            }
            CallEntry::Skip { count } => {
                reverser.add_unmeasured_seeds(*count);
            }
        }
    }

    Ok((reverser, info_bits))
}

fn get_salts(version: MCVersion, biome: BiomeType) -> Vec<i64> {
    if version.is_newer_than(MCVersion::V1_15) {
        match biome {
            BiomeType::Desert => vec![30003],
            BiomeType::NotDesert => vec![30002],
            BiomeType::Unknown => vec![30002, 30003],
        }
    } else {
        vec![20003]
    }
}

/// Internal representation of call sequence entries.
enum CallEntry {
    NextInt { bound: i32, value: i32 },
    NextIntEq { bound: i32, value: i32 },
    Skip { count: i64 },
}
