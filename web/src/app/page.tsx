"use client";

import { useState, useCallback, useEffect, useMemo } from "react";
import { motion } from "framer-motion";
import { Loader2, Zap, Cpu, AlertTriangle } from "lucide-react";
import { Header } from "@/components/Header";
import { FloorGrid } from "@/components/FloorGrid";
import { OptionsForm } from "@/components/OptionsForm";
import { ResultsPanel } from "@/components/ResultsPanel";
import { ParticleBackground } from "@/components/ParticleBackground";
import { useCracker } from "@/hooks/useCracker";
import {
  Tile,
  TILE_COUNT,
  FLOOR_SIZES,
  type MCVersion,
  type Biome,
} from "@/lib/types";

function createEmptyFloor(): Tile[][] {
  return Array.from({ length: 9 }, () =>
    new Array<Tile>(9).fill(Tile.UnknownSolid)
  );
}

/** Serialize the visible portion of the floor grid into a digit string. */
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

/** Parse a digit string back into the 9×9 floor grid, filling visible cells. */
function patternToFloor(
  pattern: string,
  sizeIndex: number
): Tile[][] | null {
  const fs = FLOOR_SIZES[sizeIndex];
  const expectedLen = (fs.xMax - fs.xMin) * (fs.zMax - fs.zMin);
  if (pattern.length !== expectedLen) return null;

  // Validate every character is a valid tile digit
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

export default function Home() {
  const [floorData, setFloorData] = useState<Tile[][]>(createEmptyFloor);
  const [floorSizeIndex, setFloorSizeIndex] = useState(0);
  const [hydrated, setHydrated] = useState(false);

  const [spawnerX, setSpawnerX] = useState("");
  const [spawnerY, setSpawnerY] = useState("");
  const [spawnerZ, setSpawnerZ] = useState("");
  const [version, setVersion] = useState<MCVersion>("1.13");
  const [biome, setBiome] = useState<Biome>("notdesert");

  const cracker = useCracker();

  // ── Restore from URL hash on mount ──
  useEffect(() => {
    try {
      const hash = window.location.hash.slice(1); // remove "#"
      if (hash) {
        // Format: sizeIndex:pattern:x,y,z
        const parts = hash.split(":");
        if (parts.length >= 2) {
          const idx = parseInt(parts[0], 10);
          const pattern = parts[1];
          if (!isNaN(idx) && idx >= 0 && idx < FLOOR_SIZES.length) {
            const grid = patternToFloor(pattern, idx);
            if (grid) {
              setFloorSizeIndex(idx);
              setFloorData(grid);
            }
          }
          // Restore spawner coordinates if present
          if (parts[2]) {
            const coords = parts[2].split(",");
            if (coords.length === 3) {
              setSpawnerX(coords[0]);
              setSpawnerY(coords[1]);
              setSpawnerZ(coords[2]);
            }
          }
        } else {
          // Try as plain pattern with default size index 0
          const grid = patternToFloor(hash, 0);
          if (grid) {
            setFloorData(grid);
          }
        }
      }
    } catch {
      // ignore malformed hash
    }
    setHydrated(true);
  }, []);

  // ── Derived pattern string ──
  const patternString = useMemo(
    () => floorToPattern(floorData, floorSizeIndex),
    [floorData, floorSizeIndex]
  );

  // ── Persist to URL hash whenever state changes ──
  useEffect(() => {
    if (!hydrated) return;
    const coordsPart =
      spawnerX || spawnerY || spawnerZ
        ? `:${spawnerX},${spawnerY},${spawnerZ}`
        : "";
    const fragment = `${floorSizeIndex}:${patternString}${coordsPart}`;
    window.history.replaceState(null, "", `#${fragment}`);
  }, [patternString, floorSizeIndex, spawnerX, spawnerY, spawnerZ, hydrated]);

  const handleTileChange = useCallback(
    (z: number, x: number, tile: Tile) => {
      setFloorData((prev) => {
        const next = prev.map((row) => [...row]);
        next[z][x] = tile;
        return next;
      });
    },
    []
  );

  const handleClear = useCallback(() => {
    setFloorData(createEmptyFloor());
  }, []);

  const handlePatternChange = useCallback(
    (value: string) => {
      const grid = patternToFloor(value, floorSizeIndex);
      if (grid) {
        setFloorData(grid);
      }
    },
    [floorSizeIndex]
  );

  const isValid =
    spawnerX !== "" &&
    spawnerY !== "" &&
    spawnerZ !== "" &&
    !isNaN(parseInt(spawnerX)) &&
    !isNaN(parseInt(spawnerY)) &&
    !isNaN(parseInt(spawnerZ));

  const canCrack =
    isValid &&
    cracker.workersReady &&
    cracker.status !== "cracking" &&
    cracker.status !== "preparing";

  const handleCrack = useCallback(() => {
    if (!canCrack) return;

    const fs = FLOOR_SIZES[floorSizeIndex];

    // WASM expects the full 9×9 grid (81 values), not just the visible portion.
    const flatGrid = new Uint8Array(81);
    for (let z = 0; z < 9; z++) {
      for (let x = 0; x < 9; x++) {
        flatGrid[z * 9 + x] = floorData[z][x];
      }
    }

    cracker.crack({
      spawnerX: parseInt(spawnerX),
      spawnerY: parseInt(spawnerY),
      spawnerZ: parseInt(spawnerZ),
      version,
      biome,
      floorSize: fs.key,
      floorGrid: flatGrid,
    });
  }, [
    canCrack,
    floorData,
    floorSizeIndex,
    spawnerX,
    spawnerY,
    spawnerZ,
    version,
    biome,
    cracker,
  ]);

  const isCracking =
    cracker.status === "cracking" || cracker.status === "preparing";

  return (
    <div className="min-h-dvh flex flex-col bg-mc-bg-dark">
      <ParticleBackground />
      <Header />

      <main className="relative z-10 flex-1 max-w-7xl mx-auto w-full px-4 md:px-6 py-6 md:py-8">
        <div className="flex flex-col lg:flex-row gap-6 lg:gap-8">
          {/* ── Left column: Dungeon Floor ── */}
          <div className="lg:flex-shrink-0">
            <FloorGrid
              floorData={floorData}
              floorSizeIndex={floorSizeIndex}
              patternString={patternString}
              onTileChange={handleTileChange}
              onFloorSizeChange={setFloorSizeIndex}
              onPatternChange={handlePatternChange}
              onClear={handleClear}
            />
          </div>

          {/* ── Right column: Options + Crack + Status + Results ── */}
          <div className="flex-1 min-w-0 space-y-5">
            <OptionsForm
              spawnerX={spawnerX}
              spawnerY={spawnerY}
              spawnerZ={spawnerZ}
              version={version}
              biome={biome}
              onSpawnerXChange={setSpawnerX}
              onSpawnerYChange={setSpawnerY}
              onSpawnerZChange={setSpawnerZ}
              onVersionChange={setVersion}
              onBiomeChange={setBiome}
            />

            {/* ── Crack Button with progress bar background ── */}
            <motion.div
              initial={{ opacity: 0, x: 30 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.5, delay: 0.2, ease: "easeOut" }}
            >
              <button
                onClick={handleCrack}
                disabled={!canCrack}
                className={`mc-btn w-full !py-3 !text-sm relative overflow-hidden ${
                  isCracking ? "mc-btn-yellow" : ""
                }`}
              >
                {/* Progress bar as button background */}
                {(isCracking || cracker.status === "done") && (
                  <motion.div
                    className="absolute inset-0 bg-mc-green"
                    style={{ opacity: 0.35 }}
                    initial={{ width: 0 }}
                    animate={{ width: `${cracker.progress}%` }}
                    transition={{ duration: 0.4, ease: "easeOut" }}
                  />
                )}
                <span className="relative z-10 flex items-center justify-center gap-2">
                  {isCracking ? (
                    <>
                      <Loader2 className="w-4 h-4 animate-spin" />
                      Cracking… {cracker.progress}%
                    </>
                  ) : cracker.status === "loading" ? (
                    <>
                      <Cpu className="w-4 h-4 animate-pulse" />
                      Loading WASM…
                    </>
                  ) : (
                    <>
                      <Zap className="w-4 h-4" />
                      Crack Seed
                    </>
                  )}
                </span>
              </button>

              {/* Error message (inline) */}
              {cracker.status === "error" && cracker.error && (
                <div className="flex items-center gap-2 mt-2 p-2.5 border border-mc-red bg-mc-bg-darker">
                  <AlertTriangle className="w-3.5 h-3.5 text-mc-red-text flex-shrink-0" />
                  <p className="text-xs text-mc-red-text">{cracker.error}</p>
                </div>
              )}
            </motion.div>

            {/* ── Results ── */}
            {cracker.result && <ResultsPanel result={cracker.result} />}
          </div>
        </div>
      </main>
    </div>
  );
}
