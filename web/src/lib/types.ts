// ── Tile types matching Rust/WASM: 0=mossy, 1=cobble, 2=air, 3=unknown, 4=unknown_solid ──

/** Next.js basePath, used to prefix public asset URLs in static export. */
export const BASE_PATH = process.env.__NEXT_ROUTER_BASEPATH || "DungeonCracker";

export enum Tile {
  Mossy = 0,
  Cobble = 1,
  Air = 2,
  Unknown = 3,
  UnknownSolid = 4,
}

export const TILE_COUNT = 5;

export const TILE_NAMES: Record<Tile, string> = {
  [Tile.Mossy]: "Mossy",
  [Tile.Cobble]: "Cobblestone",
  [Tile.Air]: "Air",
  [Tile.Unknown]: "Unknown",
  [Tile.UnknownSolid]: "Unknown Solid",
};

export const TILE_IMAGES: Record<Tile, string> = {
  [Tile.Mossy]: `${BASE_PATH}/tiles/mossy.png`,
  [Tile.Cobble]: `${BASE_PATH}/tiles/cobble.png`,
  [Tile.Air]: `${BASE_PATH}/tiles/air.png`,
  [Tile.Unknown]: `${BASE_PATH}/tiles/unknown.png`,
  [Tile.UnknownSolid]: `${BASE_PATH}/tiles/unknown_solid.png`,
};

// ── Floor sizes ──
export interface FloorSize {
  label: string;
  key: string;
  xMin: number;
  zMin: number;
  xMax: number;
  zMax: number;
}

export const FLOOR_SIZES: FloorSize[] = [
  { label: "9 × 9", key: "9x9", xMin: 0, zMin: 0, xMax: 9, zMax: 9 },
  { label: "7 × 9", key: "7x9", xMin: 1, zMin: 0, xMax: 8, zMax: 9 },
  { label: "9 × 7", key: "9x7", xMin: 0, zMin: 1, xMax: 9, zMax: 8 },
  { label: "7 × 7", key: "7x7", xMin: 1, zMin: 1, xMax: 8, zMax: 8 },
];

// ── MC versions ──
export const MC_VERSIONS = [
  "1.8",
  "1.9",
  "1.10",
  "1.11",
  "1.12",
  "1.13",
  "1.14",
  "1.15",
  "1.16",
  "1.17",
] as const;
export type MCVersion = (typeof MC_VERSIONS)[number];

// ── Biomes ──
export const BIOMES = ["desert", "notdesert", "unknown"] as const;
export type Biome = (typeof BIOMES)[number];

export const BIOME_LABELS: Record<Biome, string> = {
  desert: "Desert",
  notdesert: "Not Desert",
  unknown: "Unknown",
};

// ── Cracker result ──
export interface CrackResult {
  dungeon_seeds: string[];
  structure_seeds: string[];
  world_seeds: string[];
}

// ── Prepare result ──
export interface PrepareResult {
  total_branches: number;
  dimensions?: number;
  info_bits?: number;
  possibilities?: number;
  error?: string;
}

// ── App state ──
export type CrackStatus =
  | "idle"
  | "loading"
  | "preparing"
  | "cracking"
  | "done"
  | "error";
