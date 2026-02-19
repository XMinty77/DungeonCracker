# Dungeon Cracker (Rust)

The Minecraft dungeon floor seed cracker, ported to Rust with Copilot.
Supports Minecraft **1.8 – 1.17** and compiles to both a native CLI binary and a WebAssembly module for in-browser use.

If you have any modification you'd like to make, please contact me via Discord to ensure I receive a notification (@xminty77). I'll be happy to add you as a contributor.

## Usage

```
dungeon_cracker <x> <y> <z> <version> <biome> [floor_size] [floor_rows...]
```

**Arguments:**
| Argument | Description |
|---|---|
| `version` | `1.8`, `1.9`, … `1.17` |
| `biome` | `desert`, `notdesert`, `unknown` |
| `floor_size` | `9x9` (default), `7x9`, `9x7`, `7x7` |
| Floor digits | `0` = mossy, `1` = cobble, `2` = air, `3` = unknown, `4` = unknown solid |

**Example:**

```bash
dungeon_cracker 320 29 -418 1.13 notdesert 9x7 \
  000001000 000000000 000000010 001101000 000000110 000000011 100010000
```

## Building

```bash
# Native binary
cargo build --release

# WebAssembly (requires wasm-pack)
wasm-pack build --target web -- --features wasm
```

## Credits

The code is ported from the following projects, all credit goes to them for the brains of the cracker:

- **[Kludwisz/DungeonCracker](https://github.com/Kludwisz/DungeonCracker)** — original dungeon cracker
- **[Kinomora/DungeonCracker](https://github.com/Kinomora/DungeonCracker)** — another dungeon cracker
- **[mjtb49/LattiCG](https://github.com/mjtb49/LattiCG)** - core math library
- **[mjtb49/mc_seed_java](https://github.com/mjtb49/mc_seed_java)**
- **[mjtb49/mc_math_java](https://github.com/mjtb49/mc_math_java)**
- **[mjtb49/mc_reversal_java](https://github.com/mjtb49/mc_reversal_java)**
- **[L3g73/RustiCG](https://github.com/L3g73/RustiCG)**

I wasn't aware of RustiCG when I started this project, so it's not used here, I thought it's worth mentioning either case.

## License

MIT
