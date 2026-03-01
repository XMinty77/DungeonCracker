/**
 * URL hash fragment serialization for multi-dungeon state.
 *
 * Three encoding tiers, chosen automatically:
 *
 * 1. **Text** (1 dungeon):
 *    `sizeIndex:pattern:x,y,z:version:biome`
 *    Backward-compatible with the original single-dungeon format.
 *    Floor pattern is plain digit string.
 *
 * 2. **Text with binary floor patterns** (≥2 dungeons):
 *    Dungeons separated by `|`. Floor pattern per dungeon is encoded as:
 *      - `E`           — empty floor (all UnknownSolid)
 *      - `B<hex>`      — simplified: only Mossy(0)/Cobble(1), 1 bit per tile
 *      - `C<hex>`      — complete:  3 bits per tile (values 0–4)
 *    Remaining bits in the last byte are zero-padded.
 *
 * 3. **Full binary/hex** (if text exceeds 2048 chars):
 *    Whole hash starts with `X` followed by hex digits.
 *    First byte is a protocol version (currently 0x01).
 *    See `encodeBinary` / `decodeBinary` for the byte layout.
 */

import {
  Tile,
  TILE_COUNT,
  FLOOR_SIZES,
  MC_VERSIONS,
  BIOMES,
  type MCVersion,
  type Biome,
  type DungeonEntry,
} from "@/lib/types";

const MAX_TEXT_URL_LEN = 2048;

// ─── Helpers ────────────────────────────────────────────────────────────

function createEmptyFloor(): Tile[][] {
  return Array.from({ length: 9 }, () =>
    new Array<Tile>(9).fill(Tile.UnknownSolid)
  );
}

function floorToPattern(floor: Tile[][], sizeIndex: number): string {
  const fs = FLOOR_SIZES[sizeIndex];
  const chars: string[] = [];
  for (let z = fs.zMin; z < fs.zMax; z++) {
    for (let x = fs.xMin; x < fs.xMax; x++) {
      chars.push(String(floor[z][x]));
    }
  }
  return chars.join("");
}

function patternToFloor(
  pattern: string,
  sizeIndex: number
): Tile[][] | null {
  const fs = FLOOR_SIZES[sizeIndex];
  const expectedLen = (fs.xMax - fs.xMin) * (fs.zMax - fs.zMin);
  if (pattern.length !== expectedLen) return null;

  for (const ch of pattern) {
    const n = parseInt(ch, 10);
    if (isNaN(n) || n < 0 || n >= TILE_COUNT) return null;
  }

  const grid = createEmptyFloor();
  let i = 0;
  for (let z = fs.zMin; z < fs.zMax; z++) {
    for (let x = fs.xMin; x < fs.xMax; x++) {
      grid[z][x] = parseInt(pattern[i], 10) as Tile;
      i++;
    }
  }
  return grid;
}

// ─── Binary floor pattern encoding ──────────────────────────────────────
//
// E           → empty floor (all UnknownSolid)
// B<hex>      → simplified: only Mossy(0) and Cobble(1), 1 bit per tile
// C<hex>      → complete:   3 bits per tile (tile values 0–4)
// Trailing bits in the last byte are zero-padded.

function packBitsToHex(bits: number[], bitsPerItem: number): string {
  const bytes: number[] = [];
  let buf = 0;
  let count = 0;
  for (const v of bits) {
    buf = (buf << bitsPerItem) | v;
    count += bitsPerItem;
    if (count >= 8) {
      count -= 8;
      bytes.push((buf >> count) & 0xff);
      buf &= (1 << count) - 1;
    }
  }
  if (count > 0) {
    bytes.push((buf << (8 - count)) & 0xff);
  }
  return bytes.map((b) => b.toString(16).padStart(2, "0")).join("");
}

function unpackBitsFromHex(
  hex: string,
  totalItems: number,
  bitsPerItem: number
): number[] | null {
  if (hex.length % 2 !== 0) return null;
  const bytes: number[] = [];
  for (let i = 0; i < hex.length; i += 2) {
    const b = parseInt(hex.slice(i, i + 2), 16);
    if (isNaN(b)) return null;
    bytes.push(b);
  }

  const items: number[] = [];
  let buf = 0;
  let count = 0;
  let byteIdx = 0;
  const mask = (1 << bitsPerItem) - 1;
  while (items.length < totalItems) {
    if (count < bitsPerItem) {
      if (byteIdx >= bytes.length) return null;
      buf = (buf << 8) | bytes[byteIdx++];
      count += 8;
    }
    count -= bitsPerItem;
    items.push((buf >> count) & mask);
    buf &= (1 << count) - 1;
  }
  return items;
}

/** Encode a floor grid's visible tiles into a B/C/E prefixed hex string. */
function encodeFloorBinary(floor: Tile[][], sizeIndex: number): string {
  const fs = FLOOR_SIZES[sizeIndex];
  const tiles: number[] = [];
  for (let z = fs.zMin; z < fs.zMax; z++) {
    for (let x = fs.xMin; x < fs.xMax; x++) {
      tiles.push(floor[z][x]);
    }
  }

  // Check if floor is entirely UnknownSolid (empty)
  if (tiles.every((t) => t === Tile.UnknownSolid)) {
    return "E";
  }

  // Check if only Mossy(0) and Cobble(1) are present → 1 bit per tile
  const onlyMossyCobble = tiles.every(
    (t) => t === Tile.Mossy || t === Tile.Cobble
  );
  if (onlyMossyCobble) {
    return "B" + packBitsToHex(tiles, 1);
  }

  // General case: 3 bits per tile
  return "C" + packBitsToHex(tiles.map((t) => t & 0x07), 3);
}

/** Decode a B/C/E prefixed hex string back into the visible tiles as a flat array, or null on failure. */
function decodeFloorBinary(
  encoded: string,
  sizeIndex: number
): number[] | null {
  const fs = FLOOR_SIZES[sizeIndex];
  const totalTiles = (fs.xMax - fs.xMin) * (fs.zMax - fs.zMin);

  if (encoded === "E") {
    return new Array(totalTiles).fill(Tile.UnknownSolid);
  }

  if (encoded.startsWith("B")) {
    return unpackBitsFromHex(encoded.slice(1), totalTiles, 1);
  }

  if (encoded.startsWith("C")) {
    return unpackBitsFromHex(encoded.slice(1), totalTiles, 3);
  }

  return null;
}

// ─── Version / Biome index mapping ──────────────────────────────────────

function versionIndex(v: MCVersion): number {
  const idx = (MC_VERSIONS as readonly string[]).indexOf(v);
  return idx >= 0 ? idx : 5; // default 1.13
}

function biomeIndex(b: Biome): number {
  const idx = (BIOMES as readonly string[]).indexOf(b);
  return idx >= 0 ? idx : 1; // default notdesert
}

// ─── Text serialization (tier 1 & 2) ───────────────────────────────────

function serializeDungeonText(d: DungeonEntry, useBinaryFloor: boolean): string {
  const patternPart = useBinaryFloor
    ? encodeFloorBinary(d.floorData, d.floorSizeIndex)
    : floorToPattern(d.floorData, d.floorSizeIndex);
  const coordsPart =
    d.spawnerX || d.spawnerY || d.spawnerZ
      ? `${d.spawnerX},${d.spawnerY},${d.spawnerZ}`
      : "";
  const labelPart = d.label ? encodeURIComponent(d.label) : "";
  return `${d.floorSizeIndex}:${patternPart}:${coordsPart}:${d.version}:${d.biome}:${labelPart}`;
}

function parseDungeonText(
  s: string,
  fallbackId: string
): DungeonEntry | null {
  const parts = s.split(":");
  if (parts.length < 2) return null;

  const sizeIndex = parseInt(parts[0], 10);
  if (isNaN(sizeIndex) || sizeIndex < 0 || sizeIndex >= FLOOR_SIZES.length)
    return null;

  // Pattern: may be binary-encoded (B/C/E prefix) or plain digit string
  const rawPattern = parts[1];
  let grid: Tile[][] | null = null;

  if (
    rawPattern.startsWith("B") ||
    rawPattern.startsWith("C") ||
    rawPattern === "E"
  ) {
    const tiles = decodeFloorBinary(rawPattern, sizeIndex);
    if (!tiles) return null;
    const fs = FLOOR_SIZES[sizeIndex];
    grid = createEmptyFloor();
    let ti = 0;
    for (let z = fs.zMin; z < fs.zMax; z++) {
      for (let x = fs.xMin; x < fs.xMax; x++) {
        const val = tiles[ti++];
        grid[z][x] = val < TILE_COUNT ? (val as Tile) : Tile.UnknownSolid;
      }
    }
  } else {
    grid = patternToFloor(rawPattern, sizeIndex);
  }

  if (!grid) return null;

  let spawnerX = "",
    spawnerY = "",
    spawnerZ = "";
  if (parts[2]) {
    const coords = parts[2].split(",");
    if (coords.length === 3) {
      spawnerX = coords[0];
      spawnerY = coords[1];
      spawnerZ = coords[2];
    }
  }

  let version: MCVersion = "1.13";
  if (parts[3] && (MC_VERSIONS as readonly string[]).includes(parts[3])) {
    version = parts[3] as MCVersion;
  }

  let biome: Biome = "notdesert";
  if (parts[4] && (BIOMES as readonly string[]).includes(parts[4])) {
    biome = parts[4] as Biome;
  }

  let label = `Dungeon ${fallbackId}`;
  if (parts[5]) {
    try {
      label = decodeURIComponent(parts[5]);
    } catch {
      // ignore
    }
  }

  return {
    id: fallbackId,
    label,
    floorData: grid,
    floorSizeIndex: sizeIndex,
    spawnerX,
    spawnerY,
    spawnerZ,
    version,
    biome,
  };
}

// ─── Binary serialization (tier 3) ─────────────────────────────────────
//
// Byte layout (protocol v1):
//   [0]     protocol version = 0x01
//   [1]     number of dungeons (N)
//   For each dungeon:
//     [+0]    floorSizeIndex (4 bits) | versionIndex (4 bits)
//     [+1]    biomeIndex (4 bits) | labelLength (4 bits, max 15)
//     [+2..+2+labelLen-1]  label (UTF-8 bytes, up to 15 bytes)
//     [+next] spawnerX as int16 (big-endian)  — 0x8000 = empty
//     [+next] spawnerY as int16 (big-endian)  — 0x8000 = empty
//     [+next] spawnerZ as int16 (big-endian)  — 0x8000 = empty
//     [+next] floor tiles, 3 bits per tile packed sequentially
//             Total bits = width * height * 3, padded to full bytes.

const BINARY_PROTOCOL = 0x01;
const EMPTY_COORD = 0x8000; // sentinel for "no coordinate"

function encodeBinary(dungeons: DungeonEntry[]): string {
  const bytes: number[] = [];
  bytes.push(BINARY_PROTOCOL);
  bytes.push(dungeons.length & 0xff);

  for (const d of dungeons) {
    // Byte 0: floorSizeIndex (hi nibble) | versionIndex (lo nibble)
    const vi = versionIndex(d.version);
    bytes.push(((d.floorSizeIndex & 0x0f) << 4) | (vi & 0x0f));

    // Byte 1: biomeIndex (hi nibble) | labelLength (lo nibble)
    const bi = biomeIndex(d.biome);
    // Truncate label to 15 UTF-8 bytes
    const labelBytes = new TextEncoder().encode(d.label).slice(0, 15);
    bytes.push(((bi & 0x0f) << 4) | (labelBytes.length & 0x0f));

    // Label bytes
    for (const b of labelBytes) bytes.push(b);

    // Spawner coordinates as int16 big-endian
    for (const coordStr of [d.spawnerX, d.spawnerY, d.spawnerZ]) {
      if (coordStr === "" || isNaN(parseInt(coordStr))) {
        bytes.push((EMPTY_COORD >> 8) & 0xff);
        bytes.push(EMPTY_COORD & 0xff);
      } else {
        const val = parseInt(coordStr) & 0xffff; // truncate to 16 bits
        bytes.push((val >> 8) & 0xff);
        bytes.push(val & 0xff);
      }
    }

    // Floor tiles: 3 bits per tile (values 0–4 fit in 3 bits)
    const fs = FLOOR_SIZES[d.floorSizeIndex];
    const tiles: number[] = [];
    for (let z = fs.zMin; z < fs.zMax; z++) {
      for (let x = fs.xMin; x < fs.xMax; x++) {
        tiles.push(d.floorData[z][x] & 0x07);
      }
    }

    // Pack 3-bit values into bytes
    let bitBuf = 0;
    let bitCount = 0;
    for (const t of tiles) {
      bitBuf = (bitBuf << 3) | t;
      bitCount += 3;
      if (bitCount >= 8) {
        bitCount -= 8;
        bytes.push((bitBuf >> bitCount) & 0xff);
        bitBuf &= (1 << bitCount) - 1;
      }
    }
    // Flush remaining bits
    if (bitCount > 0) {
      bytes.push((bitBuf << (8 - bitCount)) & 0xff);
    }
  }

  return bytes.map((b) => b.toString(16).padStart(2, "0")).join("");
}

function decodeBinary(hex: string): DungeonEntry[] | null {
  if (hex.length % 2 !== 0) return null;
  const bytes: number[] = [];
  for (let i = 0; i < hex.length; i += 2) {
    const b = parseInt(hex.slice(i, i + 2), 16);
    if (isNaN(b)) return null;
    bytes.push(b);
  }

  let pos = 0;
  function read(): number {
    if (pos >= bytes.length) return 0;
    return bytes[pos++];
  }

  const protocol = read();
  if (protocol !== BINARY_PROTOCOL) return null; // unknown protocol

  const count = read();
  if (count === 0 || count > 100) return null;

  const dungeons: DungeonEntry[] = [];

  for (let di = 0; di < count; di++) {
    // Byte 0: floorSizeIndex | versionIndex
    const b0 = read();
    const floorSizeIndex = (b0 >> 4) & 0x0f;
    const vi = b0 & 0x0f;
    if (floorSizeIndex >= FLOOR_SIZES.length) return null;
    const version: MCVersion =
      vi < MC_VERSIONS.length ? MC_VERSIONS[vi] : "1.13";

    // Byte 1: biomeIndex | labelLength
    const b1 = read();
    const bi = (b1 >> 4) & 0x0f;
    const labelLen = b1 & 0x0f;
    const biome: Biome = bi < BIOMES.length ? BIOMES[bi] : "notdesert";

    // Label
    const labelBytes = new Uint8Array(labelLen);
    for (let i = 0; i < labelLen; i++) labelBytes[i] = read();
    let label: string;
    try {
      label = new TextDecoder().decode(labelBytes);
    } catch {
      label = `Dungeon ${di + 1}`;
    }

    // Spawner coordinates
    function readInt16(): string {
      const hi = read();
      const lo = read();
      const val = (hi << 8) | lo;
      if (val === EMPTY_COORD) return "";
      // Sign-extend 16-bit to JS number
      const signed = val >= 0x8000 ? val - 0x10000 : val;
      return String(signed);
    }
    const spawnerX = readInt16();
    const spawnerY = readInt16();
    const spawnerZ = readInt16();

    // Floor tiles
    const fs = FLOOR_SIZES[floorSizeIndex];
    const width = fs.xMax - fs.xMin;
    const height = fs.zMax - fs.zMin;
    const totalTiles = width * height;

    // Read 3-bit packed tiles
    const tiles: number[] = [];
    let bitBuf = 0;
    let bitCount = 0;
    while (tiles.length < totalTiles) {
      if (bitCount < 3) {
        bitBuf = (bitBuf << 8) | read();
        bitCount += 8;
      }
      bitCount -= 3;
      tiles.push((bitBuf >> bitCount) & 0x07);
      bitBuf &= (1 << bitCount) - 1;
    }

    const grid = createEmptyFloor();
    let ti = 0;
    for (let z = fs.zMin; z < fs.zMax; z++) {
      for (let x = fs.xMin; x < fs.xMax; x++) {
        const val = tiles[ti++];
        grid[z][x] = val < TILE_COUNT ? (val as Tile) : Tile.UnknownSolid;
      }
    }

    const id = String(di + 1);
    dungeons.push({
      id,
      label: label || `Dungeon ${id}`,
      floorData: grid,
      floorSizeIndex,
      spawnerX,
      spawnerY,
      spawnerZ,
      version,
      biome,
    });
  }

  return dungeons;
}

// ─── Public API ─────────────────────────────────────────────────────────

/**
 * Serialize an array of dungeons into a URL hash fragment string (without the `#`).
 * Automatically picks the most compact encoding that fits.
 */
export function serializeDungeons(dungeons: DungeonEntry[]): string {
  if (dungeons.length === 0) return "";

  // Tier 1: single dungeon — plain text, no binary floor, backward-compatible
  if (dungeons.length === 1) {
    const text = serializeDungeonText(dungeons[0], false);
    if (text.length <= MAX_TEXT_URL_LEN) return text;
  }

  // Tier 2: multiple dungeons — text with binary floor patterns, separated by |
  const useBinaryFloor = dungeons.length >= 2;
  const textParts = dungeons.map((d) => serializeDungeonText(d, useBinaryFloor));
  const textResult = textParts.join("|");
  if (textResult.length <= MAX_TEXT_URL_LEN) return textResult;

  // Tier 3: full binary encoding as hex
  return "X" + encodeBinary(dungeons);
}

/**
 * Deserialize dungeons from a URL hash fragment string (without the `#`).
 * Returns null if the hash is empty or unparseable.
 */
export function deserializeDungeons(
  hash: string
): DungeonEntry[] | null {
  if (!hash) return null;

  // Tier 3: full binary
  if (hash.startsWith("X")) {
    return decodeBinary(hash.slice(1));
  }

  // Tier 1 or 2: text (possibly pipe-separated)
  const segments = hash.split("|");
  const dungeons: DungeonEntry[] = [];

  for (let i = 0; i < segments.length; i++) {
    const d = parseDungeonText(segments[i], String(i + 1));
    if (!d) return null; // abort on any parse failure
    dungeons.push(d);
  }

  return dungeons.length > 0 ? dungeons : null;
}
