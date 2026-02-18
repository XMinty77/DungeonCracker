use dungeon_cracker::dungeon::reverse_dungeon::{
    self, BiomeType, FloorSize,
};
use dungeon_cracker::mc::chunk_rand::MCVersion;
use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 6 {
        eprintln!("Usage: {} <spawner_x> <spawner_y> <spawner_z> <version> <biome> [floor_size] [floor_rows...]", args[0]);
        eprintln!();
        eprintln!("  version: 1.8, 1.9, ..., 1.17");
        eprintln!("  biome: desert, notdesert, unknown");
        eprintln!("  floor_size: 9x9, 7x9, 9x7, 7x7 (default: 9x9)");
        eprintln!();
        eprintln!("  Floor rows should be given as strings of digits:");
        eprintln!("    0 = mossy, 1 = cobble, 2 = air, 3 = unknown, 4 = unknown_solid");
        eprintln!();
        eprintln!("Example (test case):");
        eprintln!("  {} 320 29 -418 1.13 notdesert 9x7 111110111 111111111 111111101 110010111 111111001 111111100 011101111", args[0]);
        std::process::exit(1);
    }

    let spawner_x: i32 = args[1].parse().expect("Invalid spawner X");
    let spawner_y: i32 = args[2].parse().expect("Invalid spawner Y");
    let spawner_z: i32 = args[3].parse().expect("Invalid spawner Z");

    let version = parse_version(&args[4]);
    let biome = parse_biome(&args[5]);

    let floor_size = if args.len() > 6 {
        parse_floor_size(&args[6])
    } else {
        FloorSize::_9x9
    };

    // Build floor grid from remaining args
    let floor_start_idx = 7;
    let expected_rows = floor_size.z_max() - floor_size.z_min();
    let expected_cols = floor_size.x_max() - floor_size.x_min();

    if args.len() < floor_start_idx + expected_rows {
        eprintln!(
            "Expected {} floor rows of {} characters each",
            expected_rows, expected_cols
        );
        std::process::exit(1);
    }

    // Build 9x9 grid (fill unknown_solid for out-of-bounds)
    let mut floor = [[4u8; 9]; 9]; // default to unknown_solid

    for (row_idx, z) in (floor_size.z_min()..floor_size.z_max()).enumerate() {
        let row_str: &str = &args[floor_start_idx + row_idx];
        if row_str.len() != expected_cols {
            eprintln!(
                "Row {} has {} characters, expected {}",
                row_idx,
                row_str.len(),
                expected_cols
            );
            std::process::exit(1);
        }
        for (col_idx, x) in (floor_size.x_min()..floor_size.x_max()).enumerate() {
            let ch = row_str.as_bytes()[col_idx];
            floor[z][x] = ch - b'0';
        }
    }

    let sequence = reverse_dungeon::get_sequence(&floor, floor_size);
    println!("Floor sequence: {}", sequence);
    println!("Spawner: ({}, {}, {})", spawner_x, spawner_y, spawner_z);
    println!("Version: {:?}, Biome: {:?}", version, biome);
    println!();

    let start = Instant::now();

    match reverse_dungeon::crack_dungeon(
        spawner_x, spawner_y, spawner_z, version, biome, &sequence,
    ) {
        Ok(result) => {
            let elapsed = start.elapsed();
            println!("=== Results ===");
            println!("Dungeon seeds found: {}", result.dungeon_seeds.len());
            for seed in &result.dungeon_seeds {
                println!("  Dungeon seed: {}", seed);
            }
            println!();
            println!("Structure seeds found: {}", result.structure_seeds.len());
            for seed in &result.structure_seeds {
                println!("  Structure seed: {}", seed);
            }
            println!();
            println!("World seeds found: {}", result.world_seeds.len());
            for seed in &result.world_seeds {
                println!("  World seed: {}", seed);
            }
            println!();
            println!("Time elapsed: {:?}", elapsed);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}

fn parse_version(s: &str) -> MCVersion {
    match s {
        "1.8" => MCVersion::V1_8,
        "1.9" => MCVersion::V1_9,
        "1.10" => MCVersion::V1_10,
        "1.11" => MCVersion::V1_11,
        "1.12" => MCVersion::V1_12,
        "1.13" => MCVersion::V1_13,
        "1.14" => MCVersion::V1_14,
        "1.15" => MCVersion::V1_15,
        "1.16" => MCVersion::V1_16,
        "1.17" => MCVersion::V1_17,
        _ => {
            eprintln!("Unknown version: {}", s);
            std::process::exit(1);
        }
    }
}

fn parse_biome(s: &str) -> BiomeType {
    match s.to_lowercase().as_str() {
        "desert" => BiomeType::Desert,
        "notdesert" | "mountains" | "not_desert" => BiomeType::NotDesert,
        "unknown" => BiomeType::Unknown,
        _ => {
            eprintln!("Unknown biome type: {} (use desert, notdesert, or unknown)", s);
            std::process::exit(1);
        }
    }
}

fn parse_floor_size(s: &str) -> FloorSize {
    match s.to_lowercase().as_str() {
        "9x9" => FloorSize::_9x9,
        "7x9" => FloorSize::_7x9,
        "9x7" => FloorSize::_9x7,
        "7x7" => FloorSize::_7x7,
        _ => {
            eprintln!("Unknown floor size: {} (use 9x9, 7x9, 9x7, or 7x7)", s);
            std::process::exit(1);
        }
    }
}
