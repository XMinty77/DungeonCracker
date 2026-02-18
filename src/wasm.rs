use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

use crate::dungeon::reverse_dungeon::{self, BiomeType, FloorSize};
use crate::mc::chunk_rand::MCVersion;

#[derive(Serialize, Deserialize)]
pub struct WasmCrackResult {
    pub dungeon_seeds: Vec<String>,
    pub structure_seeds: Vec<String>,
    pub world_seeds: Vec<String>,
    pub error: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct WasmPrepareResult {
    pub total_branches: i64,
    pub possibilities: usize,
    pub dimensions: usize,
    pub info_bits: f32,
    pub error: Option<String>,
}

/// Parse a version string into MCVersion.
fn parse_version(s: &str) -> Result<MCVersion, String> {
    match s {
        "1.8" => Ok(MCVersion::V1_8),
        "1.9" => Ok(MCVersion::V1_9),
        "1.10" => Ok(MCVersion::V1_10),
        "1.11" => Ok(MCVersion::V1_11),
        "1.12" => Ok(MCVersion::V1_12),
        "1.13" => Ok(MCVersion::V1_13),
        "1.14" => Ok(MCVersion::V1_14),
        "1.15" => Ok(MCVersion::V1_15),
        "1.16" => Ok(MCVersion::V1_16),
        "1.17" => Ok(MCVersion::V1_17),
        _ => Err(format!("Unknown version: {}", s)),
    }
}

/// Parse a biome string into BiomeType.
fn parse_biome(s: &str) -> Result<BiomeType, String> {
    match s.to_lowercase().as_str() {
        "desert" => Ok(BiomeType::Desert),
        "notdesert" | "not_desert" | "mountains" => Ok(BiomeType::NotDesert),
        "unknown" => Ok(BiomeType::Unknown),
        _ => Err(format!("Unknown biome: {}", s)),
    }
}

/// Parse a floor size string into FloorSize.
fn parse_floor_size(s: &str) -> Result<FloorSize, String> {
    match s.to_lowercase().as_str() {
        "9x9" => Ok(FloorSize::_9x9),
        "7x9" => Ok(FloorSize::_7x9),
        "9x7" => Ok(FloorSize::_9x7),
        "7x7" => Ok(FloorSize::_7x7),
        _ => Err(format!("Unknown floor size: {}", s)),
    }
}

/// Build the floor sequence string from a flat grid + floor size.
fn build_sequence(floor_grid: &[u8], floor_size_str: &str) -> Result<String, String> {
    let floor_size = parse_floor_size(floor_size_str)?;
    if floor_grid.len() != 81 {
        return Err(format!("Expected 81 floor values, got {}", floor_grid.len()));
    }

    let mut floor = [[4u8; 9]; 9];
    for z in 0..9 {
        for x in 0..9 {
            floor[z][x] = floor_grid[z * 9 + x];
        }
    }

    Ok(reverse_dungeon::get_sequence(&floor, floor_size))
}

/// Original single-shot entry point (non-parallel, kept for compatibility).
#[wasm_bindgen]
pub fn crack_dungeon_wasm(
    spawner_x: i32,
    spawner_y: i32,
    spawner_z: i32,
    version: &str,
    biome: &str,
    floor_size: &str,
    floor_grid: &[u8],
) -> String {
    let result = crack_dungeon_inner(spawner_x, spawner_y, spawner_z, version, biome, floor_size, floor_grid);
    serde_json::to_string(&result).unwrap_or_else(|e| {
        format!(r#"{{"error":"Serialization error: {}","dungeon_seeds":[],"structure_seeds":[],"world_seeds":[]}}"#, e)
    })
}

fn crack_dungeon_inner(
    spawner_x: i32,
    spawner_y: i32,
    spawner_z: i32,
    version: &str,
    biome: &str,
    floor_size_str: &str,
    floor_grid: &[u8],
) -> WasmCrackResult {
    let version = match parse_version(version) {
        Ok(v) => v,
        Err(e) => return WasmCrackResult {
            dungeon_seeds: vec![], structure_seeds: vec![], world_seeds: vec![],
            error: Some(e),
        },
    };

    let biome = match parse_biome(biome) {
        Ok(b) => b,
        Err(e) => return WasmCrackResult {
            dungeon_seeds: vec![], structure_seeds: vec![], world_seeds: vec![],
            error: Some(e),
        },
    };

    let sequence = match build_sequence(floor_grid, floor_size_str) {
        Ok(s) => s,
        Err(e) => return WasmCrackResult {
            dungeon_seeds: vec![], structure_seeds: vec![], world_seeds: vec![],
            error: Some(e),
        },
    };

    match reverse_dungeon::crack_dungeon(spawner_x, spawner_y, spawner_z, version, biome, &sequence) {
        Ok(result) => WasmCrackResult {
            dungeon_seeds: result.dungeon_seeds.iter().map(|s| s.to_string()).collect(),
            structure_seeds: result.structure_seeds.iter().map(|s| s.to_string()).collect(),
            world_seeds: result.world_seeds.iter().map(|s| s.to_string()).collect(),
            error: None,
        },
        Err(e) => WasmCrackResult {
            dungeon_seeds: vec![], structure_seeds: vec![], world_seeds: vec![],
            error: Some(e),
        },
    }
}

/// Prepare step: parse floor, build reverser, LLL reduce, get branch count.
/// Returns JSON with total_branches (for splitting work), dimensions, etc.
#[wasm_bindgen]
pub fn prepare_crack_wasm(
    spawner_x: i32,
    spawner_y: i32,
    spawner_z: i32,
    version: &str,
    biome: &str,
    floor_size: &str,
    floor_grid: &[u8],
) -> String {
    let result = prepare_crack_inner(spawner_x, spawner_y, spawner_z, version, biome, floor_size, floor_grid);
    serde_json::to_string(&result).unwrap_or_else(|e| {
        format!(r#"{{"error":"Serialization error: {}","total_branches":0,"possibilities":0,"dimensions":0,"info_bits":0}}"#, e)
    })
}

fn prepare_crack_inner(
    spawner_x: i32,
    spawner_y: i32,
    spawner_z: i32,
    version: &str,
    biome: &str,
    floor_size_str: &str,
    floor_grid: &[u8],
) -> WasmPrepareResult {
    let version = match parse_version(version) {
        Ok(v) => v,
        Err(e) => return WasmPrepareResult {
            total_branches: 0, possibilities: 0, dimensions: 0, info_bits: 0.0,
            error: Some(e),
        },
    };

    let biome = match parse_biome(biome) {
        Ok(b) => b,
        Err(e) => return WasmPrepareResult {
            total_branches: 0, possibilities: 0, dimensions: 0, info_bits: 0.0,
            error: Some(e),
        },
    };

    let sequence = match build_sequence(floor_grid, floor_size_str) {
        Ok(s) => s,
        Err(e) => return WasmPrepareResult {
            total_branches: 0, possibilities: 0, dimensions: 0, info_bits: 0.0,
            error: Some(e),
        },
    };

    match reverse_dungeon::prepare_crack(spawner_x, spawner_y, spawner_z, version, biome, &sequence) {
        Ok(result) => WasmPrepareResult {
            total_branches: result.total_branches,
            possibilities: result.possibilities,
            dimensions: result.dimensions,
            info_bits: result.info_bits,
            error: None,
        },
        Err(e) => WasmPrepareResult {
            total_branches: 0, possibilities: 0, dimensions: 0, info_bits: 0.0,
            error: Some(e),
        },
    }
}

/// Run a partial crack for branches [branch_start, branch_end).
/// Returns JSON with dungeon_seeds, structure_seeds, world_seeds.
#[wasm_bindgen]
pub fn crack_dungeon_partial_wasm(
    spawner_x: i32,
    spawner_y: i32,
    spawner_z: i32,
    version: &str,
    biome: &str,
    floor_size: &str,
    floor_grid: &[u8],
    branch_start: i32,
    branch_end: i32,
) -> String {
    let result = crack_partial_inner(
        spawner_x, spawner_y, spawner_z, version, biome, floor_size, floor_grid,
        branch_start as i64, branch_end as i64,
    );
    serde_json::to_string(&result).unwrap_or_else(|e| {
        format!(r#"{{"error":"Serialization error: {}","dungeon_seeds":[],"structure_seeds":[],"world_seeds":[]}}"#, e)
    })
}

fn crack_partial_inner(
    spawner_x: i32,
    spawner_y: i32,
    spawner_z: i32,
    version: &str,
    biome: &str,
    floor_size_str: &str,
    floor_grid: &[u8],
    branch_start: i64,
    branch_end: i64,
) -> WasmCrackResult {
    let version = match parse_version(version) {
        Ok(v) => v,
        Err(e) => return WasmCrackResult {
            dungeon_seeds: vec![], structure_seeds: vec![], world_seeds: vec![],
            error: Some(e),
        },
    };

    let biome = match parse_biome(biome) {
        Ok(b) => b,
        Err(e) => return WasmCrackResult {
            dungeon_seeds: vec![], structure_seeds: vec![], world_seeds: vec![],
            error: Some(e),
        },
    };

    let sequence = match build_sequence(floor_grid, floor_size_str) {
        Ok(s) => s,
        Err(e) => return WasmCrackResult {
            dungeon_seeds: vec![], structure_seeds: vec![], world_seeds: vec![],
            error: Some(e),
        },
    };

    match reverse_dungeon::crack_dungeon_partial(
        spawner_x, spawner_y, spawner_z, version, biome, &sequence,
        branch_start, branch_end,
    ) {
        Ok(result) => WasmCrackResult {
            dungeon_seeds: result.dungeon_seeds.iter().map(|s| s.to_string()).collect(),
            structure_seeds: result.structure_seeds.iter().map(|s| s.to_string()).collect(),
            world_seeds: result.world_seeds.iter().map(|s| s.to_string()).collect(),
            error: None,
        },
        Err(e) => WasmCrackResult {
            dungeon_seeds: vec![], structure_seeds: vec![], world_seeds: vec![],
            error: Some(e),
        },
    }
}
