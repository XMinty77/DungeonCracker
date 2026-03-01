"use client";

import { useCallback, useMemo, useRef, useState } from "react";
import { motion } from "framer-motion";
import { Loader2, Zap, Cpu, AlertTriangle, XCircle } from "lucide-react";
import { FloorGrid } from "@/components/FloorGrid";
import { OptionsForm } from "@/components/OptionsForm";
import { ResultsPanel } from "@/components/ResultsPanel";
import {
  Tile,
  TILE_COUNT,
  FLOOR_SIZES,
  type MCVersion,
  type Biome,
  type DungeonEntry,
  type CrackResult,
  type CrackStatus,
} from "@/lib/types";
import { hasAnimated } from "@/lib/initial-animation";

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
function patternToFloor(pattern: string, sizeIndex: number): Tile[][] | null {
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

function createEmptyFloor(): Tile[][] {
  return Array.from({ length: 9 }, () =>
    new Array<Tile>(9).fill(Tile.UnknownSolid)
  );
}

interface DungeonPanelProps {
  dungeon: DungeonEntry;
  onChange: (updated: DungeonEntry) => void;
  onUsePicture: () => void;
  /** Crack controls for this dungeon */
  onCrack: () => void;
  crackerStatus: CrackStatus;
  crackerProgress: number;
  crackerError: string | null;
  crackerResult: CrackResult | null;
  workersReady: boolean;
  isCracking: boolean;
  /** True when a multi-dungeon crack-all run is in progress */
  multiCrackActive: boolean;
}

export function DungeonPanel({
  dungeon,
  onChange,
  onUsePicture,
  onCrack,
  crackerStatus,
  crackerProgress,
  crackerError,
  crackerResult,
  workersReady,
  isCracking,
  multiCrackActive,
}: DungeonPanelProps) {
  const dungeonRef = useRef(dungeon);
  dungeonRef.current = dungeon;

  const patternString = useMemo(
    () => floorToPattern(dungeon.floorData, dungeon.floorSizeIndex),
    [dungeon.floorData, dungeon.floorSizeIndex]
  );

  const handleTileChange = useCallback(
    (z: number, x: number, tile: Tile) => {
      const d = dungeonRef.current;
      const next = d.floorData.map((row) => [...row]);
      next[z][x] = tile;
      onChange({ ...d, floorData: next });
    },
    [onChange]
  );

  const handleFloorSizeChange = useCallback(
    (index: number) => {
      onChange({ ...dungeonRef.current, floorSizeIndex: index });
    },
    [onChange]
  );

  const handlePatternChange = useCallback(
    (value: string) => {
      const d = dungeonRef.current;
      const grid = patternToFloor(value, d.floorSizeIndex);
      if (grid) {
        onChange({ ...d, floorData: grid });
      }
    },
    [onChange]
  );

  const handleClear = useCallback(() => {
    onChange({ ...dungeonRef.current, floorData: createEmptyFloor() });
  }, [onChange]);

  const handleSpawnerXChange = useCallback(
    (v: string) => onChange({ ...dungeonRef.current, spawnerX: v }),
    [onChange]
  );
  const handleSpawnerYChange = useCallback(
    (v: string) => onChange({ ...dungeonRef.current, spawnerY: v }),
    [onChange]
  );
  const handleSpawnerZChange = useCallback(
    (v: string) => onChange({ ...dungeonRef.current, spawnerZ: v }),
    [onChange]
  );
  const handleVersionChange = useCallback(
    (v: MCVersion) => onChange({ ...dungeonRef.current, version: v }),
    [onChange]
  );
  const handleBiomeChange = useCallback(
    (v: Biome) => onChange({ ...dungeonRef.current, biome: v }),
    [onChange]
  );

  // Apply an image analysis result to this dungeon
  const handleImageApply = useCallback(
    (floor: Tile[][], sizeIndex: number) => {
      onChange({ ...dungeonRef.current, floorData: floor, floorSizeIndex: sizeIndex });
    },
    [onChange]
  );

  const valid =
    dungeon.spawnerX !== "" &&
    dungeon.spawnerY !== "" &&
    dungeon.spawnerZ !== "" &&
    !isNaN(parseInt(dungeon.spawnerX)) &&
    !isNaN(parseInt(dungeon.spawnerY)) &&
    !isNaN(parseInt(dungeon.spawnerZ));

  const canCrack = valid && workersReady && crackerStatus !== "preparing";

  // During multi-crack, another dungeon may be cracking (not this one)
  const multiCrackBusy = multiCrackActive && !isCracking;

  // ── Validation highlight state ──
  const [showValidation, setShowValidation] = useState(false);

  // Clear validation highlight when the user starts editing
  const prevValidRef = useRef(valid);
  if (!prevValidRef.current && valid && showValidation) {
    // field went from invalid → valid, clear highlight
    setShowValidation(false);
  }
  prevValidRef.current = valid;

  const handleCrackClick = useCallback(() => {
    // If currently cracking, delegate to parent (stop)
    if (isCracking) {
      onCrack();
      return;
    }
    // If inputs invalid, highlight them instead of cracking
    if (!valid) {
      setShowValidation(true);
      return;
    }
    setShowValidation(false);
    onCrack();
  }, [isCracking, valid, onCrack]);

  return (
    <div className="flex flex-col lg:flex-row gap-6 lg:gap-8">
      {/* Left column: Dungeon Floor */}
      <div className="lg:flex-shrink-0">
        <FloorGrid
          floorData={dungeon.floorData}
          floorSizeIndex={dungeon.floorSizeIndex}
          patternString={patternString}
          onTileChange={handleTileChange}
          onFloorSizeChange={handleFloorSizeChange}
          onPatternChange={handlePatternChange}
          onClear={handleClear}
          onUsePicture={onUsePicture}
        />
      </div>

      {/* Right column: Options + Crack + Results */}
      <div className="flex-1 min-w-0 space-y-5">
        <OptionsForm
          spawnerX={dungeon.spawnerX}
          spawnerY={dungeon.spawnerY}
          spawnerZ={dungeon.spawnerZ}
          version={dungeon.version}
          biome={dungeon.biome}
          showValidation={showValidation}
          onSpawnerXChange={handleSpawnerXChange}
          onSpawnerYChange={handleSpawnerYChange}
          onSpawnerZChange={handleSpawnerZChange}
          onVersionChange={handleVersionChange}
          onBiomeChange={handleBiomeChange}
        />

        {/* ── Crack Button ── */}
        <motion.div
          initial={hasAnimated() ? false : { opacity: 0, x: 30 }}
          animate={{ opacity: 1, x: 0 }}
          transition={{ duration: 0.5, delay: 0.2, ease: "easeOut" }}
        >
          <button
            onClick={handleCrackClick}
            tabIndex={70}
            disabled={(!workersReady && !isCracking) || multiCrackBusy || crackerStatus === "preparing"}
            className={`mc-btn w-full !py-3 !text-sm relative overflow-hidden ${
              isCracking ? "mc-btn-red" : ""
            }`}
          >
            <span className="relative z-10 flex items-center justify-center gap-2">
              {isCracking ? (
                <>
                  <XCircle className="w-4 h-4" />
                  Stop
                </>
              ) : multiCrackBusy ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin" />
                  Multi-crack in progress…
                </>
              ) : crackerStatus === "loading" ? (
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

          {/* Progress bar */}
          {(isCracking || crackerStatus === "done") && (
            <div className="mt-2 space-y-1">
              <div className="w-full h-1 bg-mc-bg-darker border border-mc-border overflow-hidden">
                <motion.div
                  className="h-full bg-mc-green progress-bar-shimmer"
                  initial={{ width: 0 }}
                  animate={{ width: `${crackerProgress}%` }}
                  transition={{ duration: 0.4, ease: "easeOut" }}
                />
              </div>
              {isCracking && (
                <div className="flex items-center gap-2 text-xs text-mc-text-dim">
                  <Loader2 className="w-3.5 h-3.5 animate-spin" />
                  <span>Cracking… {crackerProgress}%</span>
                </div>
              )}
            </div>
          )}

          {/* Error message */}
          {crackerStatus === "error" && crackerError && (
            <div className="flex items-center gap-2 mt-2 p-2.5 border border-mc-red bg-mc-bg-darker">
              <AlertTriangle className="w-3.5 h-3.5 text-mc-red-text flex-shrink-0" />
              <p className="text-xs text-mc-red-text">{crackerError}</p>
            </div>
          )}
        </motion.div>

        {/* ── Results ── */}
        {crackerResult && <ResultsPanel result={crackerResult} />}
      </div>
    </div>
  );
}

export { createEmptyFloor, floorToPattern, patternToFloor };
