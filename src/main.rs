use dungeon_cracker::dungeon::reverse_dungeon::{
    self, BiomeType, FloorSize,
};
use dungeon_cracker::mc::chunk_rand::MCVersion;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::time::Instant;

// ─── JSON I/O types ─────────────────────────────────────────────────────

/// A single dungeon's input specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DungeonInput {
    /// Spawner X coordinate.
    spawner_x: i32,
    /// Spawner Y coordinate.
    spawner_y: i32,
    /// Spawner Z coordinate.
    spawner_z: i32,
    /// Minecraft version string, e.g. "1.13".
    version: String,
    /// Biome type: "desert", "notdesert", or "unknown".
    biome: String,
    /// Floor size key: "9x9", "7x9", "9x7", "7x7".
    #[serde(default = "default_floor_size")]
    floor_size: String,
    /// Optional label for the dungeon.
    #[serde(default)]
    label: String,
    /// Floor rows as array of digit-strings (e.g. ["111110111", ...]).
    /// Either `floor_rows` or `floor_sequence` must be provided.
    #[serde(default)]
    floor_rows: Vec<String>,
    /// Pre-computed floor sequence string (column-major). If provided,
    /// `floor_rows` and `floor_size` are ignored.
    #[serde(default)]
    floor_sequence: String,
}

fn default_floor_size() -> String {
    "9x9".to_string()
}

/// Top-level JSON input: an array of dungeons.
#[derive(Debug, Serialize, Deserialize)]
struct JsonInput {
    dungeons: Vec<DungeonInput>,
}

/// Per-dungeon result in the output JSON.
#[derive(Debug, Serialize, Deserialize)]
struct DungeonOutput {
    #[serde(skip_serializing_if = "String::is_empty")]
    label: String,
    spawner_x: i32,
    spawner_y: i32,
    spawner_z: i32,
    version: String,
    biome: String,
    dungeon_seeds: Vec<i64>,
    structure_seeds: Vec<i64>,
    world_seeds: Vec<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    elapsed_ms: u64,
}

/// Top-level JSON output.
#[derive(Debug, Serialize, Deserialize)]
struct JsonOutput {
    dungeons: Vec<DungeonOutput>,
    /// World seeds common to all successfully cracked dungeons.
    common_world_seeds: Vec<i64>,
    total_elapsed_ms: u64,
}

// ─── Constants matching the web UI ──────────────────────────────────────

const HASH_MC_VERSIONS: &[&str] = &[
    "1.8", "1.9", "1.10", "1.11", "1.12", "1.13", "1.14", "1.15", "1.16", "1.17",
];

const HASH_BIOMES: &[&str] = &["desert", "notdesert", "unknown"];

struct FloorSizeDef {
    key: &'static str,
    x_min: usize,
    z_min: usize,
    x_max: usize,
    z_max: usize,
}

const HASH_FLOOR_SIZES: &[FloorSizeDef] = &[
    FloorSizeDef { key: "9x9", x_min: 0, z_min: 0, x_max: 9, z_max: 9 },
    FloorSizeDef { key: "7x9", x_min: 1, z_min: 0, x_max: 8, z_max: 9 },
    FloorSizeDef { key: "9x7", x_min: 0, z_min: 1, x_max: 9, z_max: 8 },
    FloorSizeDef { key: "7x7", x_min: 1, z_min: 1, x_max: 8, z_max: 8 },
];

// ─── Argument parsing ───────────────────────────────────────────────────

enum InputMode {
    /// Legacy positional args: <x> <y> <z> <version> <biome> [floor_size] [floor_rows...]
    Legacy(Vec<String>),
    /// --json <path>
    JsonFile(String),
    /// --hash <fragment>
    UrlHash(String),
}

struct CliArgs {
    input: InputMode,
    output_file: Option<String>,
    verbose: bool,
}

fn parse_cli_args() -> CliArgs {
    let args: Vec<String> = env::args().collect();
    let mut output_file: Option<String> = None;
    let mut input_mode: Option<InputMode> = None;
    let mut verbose = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --output requires a filename argument");
                    std::process::exit(1);
                }
                output_file = Some(args[i].clone());
            }
            "--json" | "-j" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --json requires a filename argument");
                    std::process::exit(1);
                }
                input_mode = Some(InputMode::JsonFile(args[i].clone()));
            }
            "--hash" | "-u" => {
                i += 1;
                if i >= args.len() {
                    eprintln!("Error: --hash requires a URL hash fragment argument");
                    std::process::exit(1);
                }
                // Strip leading '#' if present
                let fragment = args[i].trim_start_matches('#').to_string();
                input_mode = Some(InputMode::UrlHash(fragment));
            }
            "--verbose" | "--log" => {
                verbose = true;
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                // Must be start of legacy positional args; collect the rest
                if input_mode.is_none() {
                    let rest: Vec<String> = args[i..].to_vec();
                    input_mode = Some(InputMode::Legacy(rest));
                    // Skip the rest since we consumed them
                    i = args.len();
                    continue;
                } else {
                    eprintln!("Error: unexpected argument '{}' (input mode already specified)", args[i]);
                    std::process::exit(1);
                }
            }
        }
        i += 1;
    }

    if input_mode.is_none() {
        print_help();
        std::process::exit(1);
    }

    CliArgs {
        input: input_mode.unwrap(),
        output_file,
        verbose,
    }
}

fn print_help() {
    let prog = env::args().next().unwrap_or_else(|| "dungeon_cracker".to_string());
    eprintln!("Dungeon Cracker");
    eprintln!();
    eprintln!("USAGE:");
    eprintln!("  {prog} [OPTIONS] <input>");
    eprintln!();
    eprintln!("INPUT MODES:");
    eprintln!("  <x> <y> <z> <ver> <biome> [size] [rows...]   Legacy single-dungeon positional args");
    eprintln!("  --json <file>  | -j <file>                    Read dungeons from a JSON file");
    eprintln!("  --hash <frag>  | -u <frag>                    Parse a URL hash fragment from the web UI");
    eprintln!();
    eprintln!("OPTIONS:");
    eprintln!("  --output <file> | -o <file>    Write results to a JSON file");
    eprintln!("  --verbose       | --log        Show detailed internal logs");
    eprintln!("  --help          | -h           Show this help message");
    eprintln!();
    eprintln!("LEGACY POSITIONAL ARGS:");
    eprintln!("  version: 1.8, 1.9, ..., 1.17");
    eprintln!("  biome:   desert, notdesert, unknown");
    eprintln!("  size:    9x9, 7x9, 9x7, 7x7  (default: 9x9)");
    eprintln!("  rows:    digit strings (0=mossy, 1=cobble, 2=air, 3=unknown, 4=unknown_solid)");
    eprintln!();
    eprintln!("JSON FILE FORMAT:");
    eprintln!(r#"  {{
    "dungeons": [
      {{
        "spawner_x": 320, "spawner_y": 29, "spawner_z": -418,
        "version": "1.13", "biome": "notdesert",
        "floor_size": "9x7",
        "floor_rows": ["000001000","000000000","000000010","001101000","000000110","000000011","100010000"],
        "label": "Example Dungeon"
      }}
    ]
  }}"#);
    eprintln!();
    eprintln!("URL HASH:");
    eprintln!("  Copy the #fragment from the web UI address bar and pass it to --hash.");
    eprintln!("  Supports single-dungeon, multi-dungeon (pipe-separated), and binary (X...) formats.");
    eprintln!();
    eprintln!("EXAMPLES:");
    eprintln!("  # Legacy single dungeon:");
    eprintln!("  {prog} 320 29 -418 1.13 notdesert 000001000 000000000 000000010 001101000 000000110 000000011 100010000");
    eprintln!();
    eprintln!("  # From JSON file, output to JSON:");
    eprintln!("  {prog} --json dungeons.json --output results.json");
    eprintln!();
    eprintln!("  # From web UI URL hash:");
    eprintln!("  {prog} --hash '0:B001f6103860082c0980580:-5,17,506:1.11:notdesert:9x9|3:B84040454010000:266,33,692:1.11:notdesert:7x7'");
}

// ─── Input resolvers ────────────────────────────────────────────────────

fn resolve_input(mode: InputMode) -> Vec<DungeonInput> {
    match mode {
        InputMode::Legacy(args) => vec![parse_legacy_args(&args)],
        InputMode::JsonFile(path) => parse_json_file(&path),
        InputMode::UrlHash(fragment) => parse_url_hash(&fragment),
    }
}

/// Parse legacy positional CLI arguments into a single DungeonInput.
fn parse_legacy_args(args: &[String]) -> DungeonInput {
    if args.len() < 5 {
        eprintln!("Error: legacy mode requires at least: <x> <y> <z> <version> <biome>");
        std::process::exit(1);
    }

    let spawner_x: i32 = args[0].parse().unwrap_or_else(|_| {
        eprintln!("Error: invalid spawner X '{}'", args[0]);
        std::process::exit(1);
    });
    let spawner_y: i32 = args[1].parse().unwrap_or_else(|_| {
        eprintln!("Error: invalid spawner Y '{}'", args[1]);
        std::process::exit(1);
    });
    let spawner_z: i32 = args[2].parse().unwrap_or_else(|_| {
        eprintln!("Error: invalid spawner Z '{}'", args[2]);
        std::process::exit(1);
    });

    let version = args[3].clone();
    let biome = args[4].clone();

    let (floor_size, floor_start) = if args.len() > 5 && is_floor_size(&args[5]) {
        (args[5].clone(), 6)
    } else {
        ("9x9".to_string(), 5)
    };

    let floor_rows: Vec<String> = args[floor_start..].to_vec();

    DungeonInput {
        spawner_x,
        spawner_y,
        spawner_z,
        version,
        biome,
        floor_size,
        label: String::new(),
        floor_rows,
        floor_sequence: String::new(),
    }
}

fn is_floor_size(s: &str) -> bool {
    matches!(s.to_lowercase().as_str(), "9x9" | "7x9" | "9x7" | "7x7")
}

/// Parse a JSON file into a list of DungeonInputs.
fn parse_json_file(path: &str) -> Vec<DungeonInput> {
    let content = fs::read_to_string(path).unwrap_or_else(|e| {
        eprintln!("Error: could not read '{}': {}", path, e);
        std::process::exit(1);
    });

    // Try parsing as { "dungeons": [...] } first, then as a bare array
    if let Ok(input) = serde_json::from_str::<JsonInput>(&content) {
        return input.dungeons;
    }
    if let Ok(dungeons) = serde_json::from_str::<Vec<DungeonInput>>(&content) {
        return dungeons;
    }
    // Try as a single dungeon object
    if let Ok(dungeon) = serde_json::from_str::<DungeonInput>(&content) {
        return vec![dungeon];
    }

    eprintln!("Error: could not parse '{}' as dungeon JSON", path);
    eprintln!("Expected format: {{ \"dungeons\": [...] }} or a bare array of dungeon objects");
    std::process::exit(1);
}

// ─── URL hash parsing (matching the web UI's hash-serialization.ts) ─────

/// Parse a URL hash fragment into dungeon inputs.
fn parse_url_hash(fragment: &str) -> Vec<DungeonInput> {
    if fragment.is_empty() {
        eprintln!("Error: empty URL hash fragment");
        std::process::exit(1);
    }

    // Tier 3: full binary (starts with 'X')
    if fragment.starts_with('X') {
        return parse_hash_binary(&fragment[1..]);
    }

    // Tier 1 or 2: text, possibly pipe-separated
    let segments: Vec<&str> = fragment.split('|').collect();
    let mut dungeons = Vec::new();

    for (i, seg) in segments.iter().enumerate() {
        match parse_hash_text_segment(seg, i + 1) {
            Some(d) => dungeons.push(d),
            None => {
                eprintln!("Error: could not parse hash segment #{}: '{}'", i + 1, seg);
                std::process::exit(1);
            }
        }
    }

    if dungeons.is_empty() {
        eprintln!("Error: no dungeons parsed from URL hash");
        std::process::exit(1);
    }

    dungeons
}

/// Parse a single text-format hash segment:
/// `sizeIndex:pattern:x,y,z:version:biome:label`
fn parse_hash_text_segment(s: &str, fallback_num: usize) -> Option<DungeonInput> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() < 2 {
        return None;
    }

    let size_index: usize = parts[0].parse().ok()?;
    if size_index >= HASH_FLOOR_SIZES.len() {
        return None;
    }
    let fs_def = &HASH_FLOOR_SIZES[size_index];

    // Decode floor pattern
    let raw_pattern = parts[1];
    let floor_sequence = decode_hash_floor_pattern(raw_pattern, fs_def)?;

    // Coordinates
    let (spawner_x, spawner_y, spawner_z) = if parts.len() > 2 && !parts[2].is_empty() {
        let coords: Vec<&str> = parts[2].split(',').collect();
        if coords.len() == 3 {
            let x: i32 = coords[0].parse().ok()?;
            let y: i32 = coords[1].parse().ok()?;
            let z: i32 = coords[2].parse().ok()?;
            (x, y, z)
        } else {
            (0, 0, 0)
        }
    } else {
        (0, 0, 0)
    };

    // Version
    let version = if parts.len() > 3 && HASH_MC_VERSIONS.contains(&parts[3]) {
        parts[3].to_string()
    } else {
        "1.13".to_string()
    };

    // Biome
    let biome = if parts.len() > 4 && HASH_BIOMES.contains(&parts[4]) {
        parts[4].to_string()
    } else {
        "notdesert".to_string()
    };

    // Label
    let label = if parts.len() > 5 && !parts[5].is_empty() {
        urlencoding_decode(parts[5])
    } else {
        format!("Dungeon {}", fallback_num)
    };

    Some(DungeonInput {
        spawner_x,
        spawner_y,
        spawner_z,
        version,
        biome,
        floor_size: fs_def.key.to_string(),
        label,
        floor_rows: Vec::new(),
        floor_sequence,
    })
}

/// Decode a floor pattern from the hash format.
/// The pattern is in row-major order (z outer, x inner) within the visible region.
/// We need to convert to column-major (x outer, z inner) for the sequence string.
fn decode_hash_floor_pattern(raw: &str, fs: &FloorSizeDef) -> Option<String> {
    let width = fs.x_max - fs.x_min;
    let height = fs.z_max - fs.z_min;
    let total = width * height;

    // Decode raw into flat tile array (row-major: z outer, x inner)
    let tiles: Vec<u8>;

    if raw == "E" {
        tiles = vec![4; total]; // All UnknownSolid
    } else if raw.starts_with('B') {
        // Simplified binary: 1 bit per tile (0=Mossy, 1=Cobble)
        tiles = unpack_bits_from_hex(&raw[1..], total, 1)?;
    } else if raw.starts_with('C') {
        // Complete: 3 bits per tile
        tiles = unpack_bits_from_hex(&raw[1..], total, 3)?;
    } else {
        // Plain digit string
        if raw.len() != total {
            return None;
        }
        tiles = raw.bytes().map(|b| {
            if b >= b'0' && b <= b'4' { b - b'0' } else { 4 }
        }).collect();
    }

    // Convert from row-major (z outer, x inner) to column-major (x outer, z inner)
    Some(build_sequence_from_flat_tiles(&tiles, width, height))
}

/// Unpack N items of `bits_per_item` bits each from a hex string.
fn unpack_bits_from_hex(hex: &str, total_items: usize, bits_per_item: u32) -> Option<Vec<u8>> {
    if hex.len() % 2 != 0 {
        return None;
    }
    let bytes: Vec<u8> = (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<_>, _>>()
        .ok()?;

    let mask = (1u32 << bits_per_item) - 1;
    let mut items = Vec::with_capacity(total_items);
    let mut buf: u32 = 0;
    let mut count: u32 = 0;
    let mut byte_idx = 0;

    while items.len() < total_items {
        if count < bits_per_item {
            if byte_idx >= bytes.len() {
                return None;
            }
            buf = (buf << 8) | bytes[byte_idx] as u32;
            byte_idx += 1;
            count += 8;
        }
        count -= bits_per_item;
        items.push(((buf >> count) & mask) as u8);
        buf &= (1 << count) - 1;
    }

    Some(items)
}

/// Parse the full binary hash format (tier 3).
fn parse_hash_binary(hex: &str) -> Vec<DungeonInput> {
    let bytes: Vec<u8> = match (0..hex.len())
        .step_by(2)
        .map(|i| {
            if i + 2 > hex.len() {
                Err(())
            } else {
                u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| ())
            }
        })
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(b) => b,
        Err(_) => {
            eprintln!("Error: invalid hex in binary hash");
            std::process::exit(1);
        }
    };

    let mut pos = 0usize;
    let read = |pos: &mut usize| -> u8 {
        if *pos >= bytes.len() {
            0
        } else {
            let v = bytes[*pos];
            *pos += 1;
            v
        }
    };

    let protocol = read(&mut pos);
    if protocol != 0x01 {
        eprintln!("Error: unknown binary hash protocol version {}", protocol);
        std::process::exit(1);
    }

    let count = read(&mut pos) as usize;
    if count == 0 || count > 100 {
        eprintln!("Error: invalid dungeon count {} in binary hash", count);
        std::process::exit(1);
    }

    let mut dungeons = Vec::with_capacity(count);

    for di in 0..count {
        // Byte 0: floorSizeIndex (hi nibble) | versionIndex (lo nibble)
        let b0 = read(&mut pos);
        let floor_size_idx = ((b0 >> 4) & 0x0f) as usize;
        let version_idx = (b0 & 0x0f) as usize;

        if floor_size_idx >= HASH_FLOOR_SIZES.len() {
            eprintln!("Error: invalid floor size index {} in binary hash", floor_size_idx);
            std::process::exit(1);
        }

        let version = if version_idx < HASH_MC_VERSIONS.len() {
            HASH_MC_VERSIONS[version_idx].to_string()
        } else {
            "1.13".to_string()
        };

        // Byte 1: biomeIndex (hi nibble) | labelLength (lo nibble)
        let b1 = read(&mut pos);
        let biome_idx = ((b1 >> 4) & 0x0f) as usize;
        let label_len = (b1 & 0x0f) as usize;

        let biome = if biome_idx < HASH_BIOMES.len() {
            HASH_BIOMES[biome_idx].to_string()
        } else {
            "notdesert".to_string()
        };

        // Label
        let label_bytes: Vec<u8> = (0..label_len).map(|_| read(&mut pos)).collect();
        let label = String::from_utf8(label_bytes)
            .unwrap_or_else(|_| format!("Dungeon {}", di + 1));

        // Spawner coordinates (int16 big-endian, 0x8000 = empty)
        let read_i16 = |pos: &mut usize| -> i32 {
            let hi = read(pos) as u16;
            let lo = read(pos) as u16;
            let val = (hi << 8) | lo;
            if val == 0x8000 {
                0 // empty coordinate
            } else if val >= 0x8000 {
                (val as i32) - 0x10000
            } else {
                val as i32
            }
        };

        let spawner_x = read_i16(&mut pos);
        let spawner_y = read_i16(&mut pos);
        let spawner_z = read_i16(&mut pos);

        // Floor tiles: 3 bits per tile
        let fs = &HASH_FLOOR_SIZES[floor_size_idx];
        let width = fs.x_max - fs.x_min;
        let height = fs.z_max - fs.z_min;
        let total_tiles = width * height;

        let mut tiles: Vec<u8> = Vec::with_capacity(total_tiles);
        let mut bit_buf: u32 = 0;
        let mut bit_count: u32 = 0;

        while tiles.len() < total_tiles {
            if bit_count < 3 {
                bit_buf = (bit_buf << 8) | read(&mut pos) as u32;
                bit_count += 8;
            }
            bit_count -= 3;
            tiles.push(((bit_buf >> bit_count) & 0x07) as u8);
            bit_buf &= (1 << bit_count) - 1;
        }

        // Convert row-major tiles to column-major sequence
        let floor_sequence = build_sequence_from_flat_tiles(&tiles, width, height);

        dungeons.push(DungeonInput {
            spawner_x,
            spawner_y,
            spawner_z,
            version,
            biome,
            floor_size: fs.key.to_string(),
            label: if label.is_empty() { format!("Dungeon {}", di + 1) } else { label },
            floor_rows: Vec::new(),
            floor_sequence,
        });
    }

    dungeons
}

/// Build a column-major floor sequence from a row-major flat tile array.
/// tiles[row * width + col] where row = z offset, col = x offset.
/// Output: x outer, z inner.
fn build_sequence_from_flat_tiles(tiles: &[u8], width: usize, height: usize) -> String {
    let mut seq = String::with_capacity(tiles.len());
    for col in 0..width {
        for row in 0..height {
            let tile = tiles[row * width + col];
            seq.push((b'0' + tile) as char);
        }
    }
    seq
}

/// Simple percent-decoding for URL-encoded labels.
fn urlencoding_decode(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(val) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""),
                16,
            ) {
                result.push(val as char);
                i += 3;
                continue;
            }
        }
        if bytes[i] == b'+' {
            result.push(' ');
        } else {
            result.push(bytes[i] as char);
        }
        i += 1;
    }
    result
}

// ─── Dungeon input resolution ───────────────────────────────────────────

/// Resolve a DungeonInput into its floor sequence, validating fields.
fn resolve_dungeon(d: &DungeonInput) -> Result<(i32, i32, i32, MCVersion, BiomeType, String), String> {
    let version = parse_version(&d.version)?;
    let biome = parse_biome(&d.biome)?;

    let sequence = if !d.floor_sequence.is_empty() {
        d.floor_sequence.clone()
    } else if !d.floor_rows.is_empty() {
        build_sequence_from_rows(&d.floor_rows, &d.floor_size)?
    } else {
        return Err("No floor data provided (need either floor_rows or floor_sequence)".to_string());
    };

    Ok((d.spawner_x, d.spawner_y, d.spawner_z, version, biome, sequence))
}

/// Build a column-major sequence string from row strings + floor size key.
fn build_sequence_from_rows(rows: &[String], floor_size_key: &str) -> Result<String, String> {
    let floor_size = parse_floor_size(floor_size_key)?;

    let expected_rows = floor_size.z_max() - floor_size.z_min();
    let expected_cols = floor_size.x_max() - floor_size.x_min();

    if rows.len() != expected_rows {
        return Err(format!(
            "Expected {} floor rows for size {}, got {}",
            expected_rows, floor_size_key, rows.len()
        ));
    }

    // Build 9x9 grid, defaulting to unknown_solid (4)
    let mut floor = [[4u8; 9]; 9];

    for (row_idx, z) in (floor_size.z_min()..floor_size.z_max()).enumerate() {
        let row_str = &rows[row_idx];
        if row_str.len() != expected_cols {
            return Err(format!(
                "Row {} has {} characters, expected {}",
                row_idx, row_str.len(), expected_cols
            ));
        }
        for (col_idx, x) in (floor_size.x_min()..floor_size.x_max()).enumerate() {
            let ch = row_str.as_bytes()[col_idx];
            if ch < b'0' || ch > b'4' {
                return Err(format!("Invalid tile '{}' at row {} col {}", ch as char, row_idx, col_idx));
            }
            floor[z][x] = ch - b'0';
        }
    }

    Ok(reverse_dungeon::get_sequence(&floor, floor_size))
}

// ─── Parsing helpers ────────────────────────────────────────────────────

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

fn parse_biome(s: &str) -> Result<BiomeType, String> {
    match s.to_lowercase().as_str() {
        "desert" => Ok(BiomeType::Desert),
        "notdesert" | "not_desert" | "mountains" => Ok(BiomeType::NotDesert),
        "unknown" => Ok(BiomeType::Unknown),
        _ => Err(format!("Unknown biome: {} (use desert, notdesert, or unknown)", s)),
    }
}

fn parse_floor_size(s: &str) -> Result<FloorSize, String> {
    match s.to_lowercase().as_str() {
        "9x9" => Ok(FloorSize::_9x9),
        "7x9" => Ok(FloorSize::_7x9),
        "9x7" => Ok(FloorSize::_9x7),
        "7x7" => Ok(FloorSize::_7x7),
        _ => Err(format!("Unknown floor size: {} (use 9x9, 7x9, 9x7, or 7x7)", s)),
    }
}

/// Format version for output JSON (user-friendly "1.13" style, not "V1_13").
fn format_version(v: MCVersion) -> String {
    match v {
        MCVersion::V1_8 => "1.8",
        MCVersion::V1_9 => "1.9",
        MCVersion::V1_10 => "1.10",
        MCVersion::V1_11 => "1.11",
        MCVersion::V1_12 => "1.12",
        MCVersion::V1_13 => "1.13",
        MCVersion::V1_14 => "1.14",
        MCVersion::V1_15 => "1.15",
        MCVersion::V1_16 => "1.16",
        MCVersion::V1_17 => "1.17",
    }.to_string()
}

fn format_biome(b: BiomeType) -> String {
    match b {
        BiomeType::Desert => "desert",
        BiomeType::NotDesert => "notdesert",
        BiomeType::Unknown => "unknown",
    }.to_string()
}

// ─── Main ───────────────────────────────────────────────────────────────

fn main() {
    let cli = parse_cli_args();
    dungeon_cracker::set_verbose(cli.verbose);
    let dungeons = resolve_input(cli.input);

    if dungeons.is_empty() {
        eprintln!("Error: no dungeons to crack");
        std::process::exit(1);
    }

    eprintln!("=== Dungeon Cracker ===");
    eprintln!("Dungeons to process: {}", dungeons.len());
    eprintln!();

    let total_start = Instant::now();
    let mut outputs: Vec<DungeonOutput> = Vec::new();
    let mut all_world_seed_sets: Vec<HashSet<i64>> = Vec::new();

    for (idx, dungeon) in dungeons.iter().enumerate() {
        let label = if dungeon.label.is_empty() {
            format!("Dungeon {}", idx + 1)
        } else {
            dungeon.label.clone()
        };

        eprintln!("─── {} ({}/{}) ───", label, idx + 1, dungeons.len());

        match resolve_dungeon(dungeon) {
            Err(e) => {
                eprintln!("  Error: {}", e);
                eprintln!();
                outputs.push(DungeonOutput {
                    label,
                    spawner_x: dungeon.spawner_x,
                    spawner_y: dungeon.spawner_y,
                    spawner_z: dungeon.spawner_z,
                    version: dungeon.version.clone(),
                    biome: dungeon.biome.clone(),
                    dungeon_seeds: vec![],
                    structure_seeds: vec![],
                    world_seeds: vec![],
                    error: Some(e),
                    elapsed_ms: 0,
                });
            }
            Ok((sx, sy, sz, version, biome, sequence)) => {
                eprintln!("  Spawner: ({}, {}, {})", sx, sy, sz);
                eprintln!("  Version: {}, Biome: {}", format_version(version), format_biome(biome));
                eprintln!("  Sequence: {} ({} tiles)", sequence, sequence.len());

                let start = Instant::now();
                match reverse_dungeon::crack_dungeon(sx, sy, sz, version, biome, &sequence) {
                    Ok(result) => {
                        let elapsed = start.elapsed();
                        let elapsed_ms = elapsed.as_millis() as u64;

                        eprintln!("  Dungeon seeds:   {}", result.dungeon_seeds.len());
                        eprintln!("  Structure seeds: {}", result.structure_seeds.len());
                        eprintln!("  World seeds:     {}", result.world_seeds.len());
                        eprintln!("  Time: {:?}", elapsed);
                        eprintln!();

                        let ws_set: HashSet<i64> = result.world_seeds.iter().copied().collect();
                        all_world_seed_sets.push(ws_set);

                        outputs.push(DungeonOutput {
                            label,
                            spawner_x: sx,
                            spawner_y: sy,
                            spawner_z: sz,
                            version: format_version(version),
                            biome: format_biome(biome),
                            dungeon_seeds: result.dungeon_seeds,
                            structure_seeds: result.structure_seeds,
                            world_seeds: result.world_seeds,
                            error: None,
                            elapsed_ms,
                        });
                    }
                    Err(e) => {
                        let elapsed = start.elapsed();
                        eprintln!("  Error: {}", e);
                        eprintln!("  Time: {:?}", elapsed);
                        eprintln!();

                        outputs.push(DungeonOutput {
                            label,
                            spawner_x: sx,
                            spawner_y: sy,
                            spawner_z: sz,
                            version: format_version(version),
                            biome: format_biome(biome),
                            dungeon_seeds: vec![],
                            structure_seeds: vec![],
                            world_seeds: vec![],
                            error: Some(e),
                            elapsed_ms: elapsed.as_millis() as u64,
                        });
                    }
                }
            }
        }
    }

    let total_elapsed = total_start.elapsed();

    // Compute intersection of world seeds across all successful dungeons
    let common_world_seeds: Vec<i64> = if all_world_seed_sets.is_empty() {
        vec![]
    } else if all_world_seed_sets.len() == 1 {
        let mut v: Vec<i64> = all_world_seed_sets[0].iter().copied().collect();
        v.sort();
        v
    } else {
        let mut intersection = all_world_seed_sets[0].clone();
        for set in &all_world_seed_sets[1..] {
            intersection = intersection.intersection(set).copied().collect();
        }
        let mut v: Vec<i64> = intersection.into_iter().collect();
        v.sort();
        v
    };

    // Print summary to stdout
    eprintln!("═══════════════════════════════════════");
    if dungeons.len() > 1 {
        eprintln!("Common world seeds: {}", common_world_seeds.len());
        for seed in &common_world_seeds {
            println!("{}", seed);
        }
    } else if !outputs.is_empty() && outputs[0].error.is_none() {
        eprintln!("World seeds found: {}", outputs[0].world_seeds.len());
        for seed in &outputs[0].world_seeds {
            println!("{}", seed);
        }
    }
    eprintln!("Total time: {:?}", total_elapsed);

    // Write JSON output if requested
    if let Some(output_path) = cli.output_file {
        let json_output = JsonOutput {
            dungeons: outputs,
            common_world_seeds,
            total_elapsed_ms: total_elapsed.as_millis() as u64,
        };

        let json_str = serde_json::to_string_pretty(&json_output).unwrap_or_else(|e| {
            eprintln!("Error: failed to serialize output: {}", e);
            std::process::exit(1);
        });

        fs::write(&output_path, &json_str).unwrap_or_else(|e| {
            eprintln!("Error: failed to write '{}': {}", output_path, e);
            std::process::exit(1);
        });

        eprintln!("Results written to: {}", output_path);
    }
}
