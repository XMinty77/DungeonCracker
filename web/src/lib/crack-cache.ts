/**
 * localStorage-backed cache for recent crack results.
 *
 * Keyed by a deterministic hash of *all* dungeon inputs (pattern tiles,
 * coordinates, version, biome, floor size).  Two dungeons are considered
 * identical only if every single setting and tile matches.
 *
 * Constraints:
 *  - At most MAX_ENTRIES cached results are kept.
 *  - Entries older than MAX_AGE_MS are pruned on page load.
 *  - Each entry stores the timestamp of when it was written.
 */

import type { CrackResult, DungeonEntry } from "@/lib/types";
import { FLOOR_SIZES } from "@/lib/types";

const STORAGE_KEY = "dungeon_crack_cache";
const MAX_ENTRIES = 10;
const MAX_AGE_MS = 48 * 60 * 60 * 1000; // 48 hours

interface CacheEntry {
  key: string;
  result: CrackResult;
  /** Unix-ms when this entry was stored. */
  storedAt: number;
}

/* ── Key generation ── */

/**
 * Build a deterministic cache key from the dungeon inputs.
 *
 * Includes every field that affects the crack: spawner coords, MC version,
 * biome, floor dimensions, and every visible tile.  Two dungeons produce
 * the same key if and only if they would give the same crack result.
 */
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
    d.spawnerX.trim(),
    d.spawnerY.trim(),
    d.spawnerZ.trim(),
    d.version,
    d.biome,
    `${d.floorSizeIndex}`,
    tiles.join(""),
  ].join("|");
}

/* ── Persistence helpers ── */

function loadCache(): CacheEntry[] {
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return [];
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed as CacheEntry[];
  } catch {
    return [];
  }
}

function saveCache(entries: CacheEntry[]) {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(entries));
  } catch {
    // Quota exceeded – trim more aggressively, then retry
    try {
      const trimmed = entries.slice(0, Math.floor(MAX_ENTRIES / 2));
      localStorage.setItem(STORAGE_KEY, JSON.stringify(trimmed));
    } catch {
      // give up
    }
  }
}

/* ── Pruning ── */

/**
 * Remove entries that are expired or exceed the size cap.
 * Call this once on page load to keep localStorage tidy.
 */
export function pruneCache(): void {
  const now = Date.now();
  let entries = loadCache();

  // Drop anything older than 48 hours
  entries = entries.filter((e) => now - e.storedAt < MAX_AGE_MS);

  // Keep only the most recent MAX_ENTRIES (sorted newest-first)
  entries.sort((a, b) => b.storedAt - a.storedAt);
  if (entries.length > MAX_ENTRIES) {
    entries.length = MAX_ENTRIES;
  }

  saveCache(entries);
}

/* ── Public API ── */

/**
 * Look up a cached result by dungeon settings.
 * Returns `null` on miss or if the entry has expired.
 */
export function getCachedResult(d: DungeonEntry): CrackResult | null {
  const key = buildCacheKey(d);
  const now = Date.now();
  const entries = loadCache();
  const entry = entries.find((e) => e.key === key);

  if (!entry) return null;

  // Expired?
  if (now - entry.storedAt >= MAX_AGE_MS) return null;

  return entry.result;
}

/**
 * Store a result in the cache.
 *
 * **Important:** `d` must be a snapshot of the dungeon settings that were
 * used to start the crack, NOT the current (possibly edited) dungeon.
 */
export function setCachedResult(d: DungeonEntry, result: CrackResult): void {
  const key = buildCacheKey(d);
  const now = Date.now();
  let entries = loadCache();

  // Remove any existing entry with the same key (dedup)
  entries = entries.filter((e) => e.key !== key);

  // Insert at front
  entries.unshift({ key, result, storedAt: now });

  // Sort newest-first and enforce size cap
  entries.sort((a, b) => b.storedAt - a.storedAt);
  if (entries.length > MAX_ENTRIES) {
    entries.length = MAX_ENTRIES;
  }

  saveCache(entries);
}

/** Clear the entire crack cache. */
export function clearCrackCache(): void {
  try {
    localStorage.removeItem(STORAGE_KEY);
  } catch {
    // ignore
  }
}
