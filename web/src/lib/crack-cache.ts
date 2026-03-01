/**
 * localStorage-backed LRU cache for recent crack results.
 * Keyed by a hash of the dungeon inputs so identical re-cracks are instant.
 */

import type { CrackResult, DungeonEntry } from "@/lib/types";
import { FLOOR_SIZES } from "@/lib/types";

const STORAGE_KEY = "dungeon_crack_cache";
const MAX_ENTRIES = 20;

interface CacheEntry {
  key: string;
  result: CrackResult;
  timestamp: number;
}

/** Build a deterministic cache key from the dungeon inputs. */
export function buildCacheKey(d: DungeonEntry): string {
  const fs = FLOOR_SIZES[d.floorSizeIndex];
  // Flatten just the visible region of the floor
  const tiles: number[] = [];
  for (let z = fs.zMin; z < fs.zMax; z++) {
    for (let x = fs.xMin; x < fs.xMax; x++) {
      tiles.push(d.floorData[z][x]);
    }
  }
  return [
    d.spawnerX,
    d.spawnerY,
    d.spawnerZ,
    d.version,
    d.biome,
    fs.key,
    tiles.join(""),
  ].join("|");
}

function loadCache(): CacheEntry[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    return JSON.parse(raw) as CacheEntry[];
  } catch {
    return [];
  }
}

function saveCache(entries: CacheEntry[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(entries));
  } catch {
    // quota exceeded – trim more aggressively
    try {
      const trimmed = entries.slice(0, Math.floor(MAX_ENTRIES / 2));
      localStorage.setItem(STORAGE_KEY, JSON.stringify(trimmed));
    } catch {
      // give up
    }
  }
}

/** Look up a cached result, returning null if miss. Promotes on hit (LRU). */
export function getCachedResult(d: DungeonEntry): CrackResult | null {
  const key = buildCacheKey(d);
  const entries = loadCache();
  const idx = entries.findIndex((e) => e.key === key);
  if (idx === -1) return null;

  // Promote to front (most recent)
  const [hit] = entries.splice(idx, 1);
  hit.timestamp = Date.now();
  entries.unshift(hit);
  saveCache(entries);

  return hit.result;
}

/** Store a result in the cache, evicting oldest if full. */
export function setCachedResult(d: DungeonEntry, result: CrackResult) {
  const key = buildCacheKey(d);
  const entries = loadCache().filter((e) => e.key !== key); // deduplicate
  entries.unshift({ key, result, timestamp: Date.now() });

  // Trim to max size
  if (entries.length > MAX_ENTRIES) {
    entries.length = MAX_ENTRIES;
  }

  saveCache(entries);
}

/** Clear the entire crack cache. */
export function clearCrackCache() {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch {
    // ignore
  }
}
