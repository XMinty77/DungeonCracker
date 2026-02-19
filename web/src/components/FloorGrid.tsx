"use client";

import { useState, useCallback, useRef, useEffect } from "react";
import { motion } from "framer-motion";
import Image from "next/image";
import { Grid3x3, Trash2, Copy, Check, Paintbrush, MousePointerClick, ImageIcon } from "lucide-react";
import {
  Tile,
  TILE_COUNT,
  TILE_IMAGES,
  TILE_NAMES,
  FLOOR_SIZES,
} from "@/lib/types";

type InputMode = "fast" | "cycle";

interface FloorGridProps {
  floorData: Tile[][];
  floorSizeIndex: number;
  patternString: string;
  onTileChange: (z: number, x: number, tile: Tile) => void;
  onFloorSizeChange: (index: number) => void;
  onPatternChange: (pattern: string) => void;
  onClear: () => void;
  onUsePicture?: () => void;
}

export function FloorGrid({
  floorData,
  floorSizeIndex,
  patternString,
  onTileChange,
  onFloorSizeChange,
  onPatternChange,
  onClear,
  onUsePicture,
}: FloorGridProps) {
  const fs = FLOOR_SIZES[floorSizeIndex];
  const [hoveredCell, setHoveredCell] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);
  const [inputMode, setInputMode] = useState<InputMode>("cycle");
  const [initialAnimDone, setInitialAnimDone] = useState(false);
  const patternRef = useRef<HTMLInputElement>(null);
  const cursorRef = useRef<number | null>(null);

  // Mark initial pop animation as done after tiles finish appearing
  useEffect(() => {
    const timer = setTimeout(() => setInitialAnimDone(true), 1200);
    return () => clearTimeout(timer);
  }, []);

  // Track which mouse button is held for fast-mode paint dragging
  // 0 = left, 2 = right, null = not painting
  const paintingRef = useRef<number | null>(null);
  // Track the cell where mousedown started, for tap animation
  const pressedCellRef = useRef<string | null>(null);
  const [tappedCell, setTappedCell] = useState<string | null>(null);

  // Release paint on global mouseup
  useEffect(() => {
    const handleUp = () => {
      paintingRef.current = null;
      pressedCellRef.current = null;
    };
    window.addEventListener("mouseup", handleUp);
    return () => window.removeEventListener("mouseup", handleUp);
  }, []);

  // Restore cursor position after React re-renders the input value
  useEffect(() => {
    if (cursorRef.current !== null && patternRef.current) {
      patternRef.current.setSelectionRange(
        cursorRef.current,
        cursorRef.current
      );
      cursorRef.current = null;
    }
  }, [patternString]);

  const cycleCell = useCallback(
    (z: number, x: number, direction: 1 | -1) => {
      const cur = floorData[z][x];
      const next =
        (((cur + direction) % TILE_COUNT) + TILE_COUNT) % TILE_COUNT;
      onTileChange(z, x, next as Tile);
    },
    [floorData, onTileChange]
  );

  /** Paint a tile in fast mode (left=mossy, right=cobble). */
  const paintCell = useCallback(
    (z: number, x: number, button: number) => {
      const tile = button === 2 ? Tile.Cobble : Tile.Mossy;
      onTileChange(z, x, tile);
    },
    [onTileChange]
  );

  /** Handle cell interaction based on current input mode. */
  const handleCellDown = useCallback(
    (z: number, x: number, button: number) => {
      pressedCellRef.current = `${z}-${x}`;
      if (inputMode === "fast") {
        paintingRef.current = button;
        paintCell(z, x, button);
      } else {
        cycleCell(z, x, button === 2 ? -1 : 1);
      }
    },
    [inputMode, paintCell, cycleCell]
  );

  const handleCellEnter = useCallback(
    (z: number, x: number) => {
      setHoveredCell(`${z}-${x}`);
      // If dragging to a different cell, cancel tap animation
      if (pressedCellRef.current !== null && pressedCellRef.current !== `${z}-${x}`) {
        pressedCellRef.current = null;
      }
      // Paint on drag in fast mode
      if (inputMode === "fast" && paintingRef.current !== null) {
        paintCell(z, x, paintingRef.current);
      }
    },
    [inputMode, paintCell]
  );

  /** Trigger tap animation only if released on the same cell (no drag). */
  const handleCellUp = useCallback(
    (z: number, x: number) => {
      if (pressedCellRef.current === `${z}-${x}`) {
        setTappedCell(`${z}-${x}`);
        setTimeout(() => setTappedCell(null), 150);
      }
      pressedCellRef.current = null;
    },
    []
  );

  const isCellVisible = (z: number, x: number) =>
    x >= fs.xMin && x < fs.xMax && z >= fs.zMin && z < fs.zMax;

  return (
    <motion.div
      initial={{ opacity: 0, x: -30 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ duration: 0.5, delay: 0.1, ease: "easeOut" }}
    >
      {/* Section header */}
      <div className="flex items-center gap-2 mb-3">
        <Grid3x3 className="w-4 h-4 text-mc-green-text" />
        <h2 className="font-minecraft text-sm text-mc-text-highlight tracking-wide">
          Dungeon Floor
        </h2>
      </div>

      {/* Panel */}
      <div className="mc-panel p-4">
        {/* Compass directions + Grid */}
        <div className="flex flex-col items-center gap-2">
          {/* North */}
          <span className="text-[10px] font-semibold text-mc-text-dim uppercase tracking-[0.2em]">
            North (−Z)
          </span>

          <div className="flex items-center gap-2">
            {/* West */}
            <span
              className="text-[10px] font-semibold text-mc-text-dim uppercase tracking-[0.2em]"
              style={{
                writingMode: "vertical-lr",
                transform: "rotate(180deg)",
              }}
            >
              West (−X)
            </span>

            {/* Grid cells */}
            <div
              className="bg-mc-bg-darker p-1 border border-mc-border"
              onContextMenu={(e) => e.preventDefault()}
            >
              <div className="grid grid-cols-9 gap-px">
                {Array.from({ length: 9 }, (_, z) =>
                  Array.from({ length: 9 }, (_, x) => {
                    const visible = isCellVisible(z, x);
                    const tile = floorData[z][x];
                    const key = `${z}-${x}`;
                    const isHovered = hoveredCell === key;
                    // Row-by-row stagger: top-left → right, then next row
                    const tileIndex = z * 9 + x;
                    const popDelay = 0.3 + tileIndex * 0.008;

                    return (
                      <motion.button
                        key={key}
                        initial={
                          !initialAnimDone && visible
                            ? { scale: 0, opacity: 0 }
                            : false
                        }
                        animate={{
                          scale: tappedCell === key ? 0.88 : 1,
                          opacity: 1,
                        }}
                        transition={
                          !initialAnimDone && visible
                            ? {
                                delay: popDelay,
                                duration: 0.15,
                                ease: [0.34, 1.56, 0.64, 1],
                              }
                            : { duration: 0.1 }
                        }
                        className={`
                          relative w-11 h-11 sm:w-12 sm:h-12 md:w-[52px] md:h-[52px]
                          outline-none transition-colors duration-200 select-none
                          ${
                            visible
                              ? "cursor-pointer"
                              : "invisible pointer-events-none"
                          }
                        `}
                        style={{
                          borderColor: isHovered
                            ? "#52A535"
                            : "transparent",
                          borderWidth: 2,
                          borderStyle: "solid",
                        }}
                        onMouseDown={(e) => {
                          if (visible) handleCellDown(z, x, e.button);
                        }}
                        onMouseUp={() => {
                          if (visible) handleCellUp(z, x);
                        }}
                        onMouseEnter={() =>
                          visible && handleCellEnter(z, x)
                        }
                        onMouseLeave={() => setHoveredCell(null)}
                        title={
                          visible
                            ? `${TILE_NAMES[tile]} (${x}, ${z})`
                            : undefined
                        }
                      >
                        {visible && (
                          <Image
                            src={TILE_IMAGES[tile]}
                            alt={TILE_NAMES[tile]}
                            fill
                            sizes="52px"
                            className="pixelated object-cover"
                            draggable={false}
                          />
                        )}
                      </motion.button>
                    );
                  })
                )}
              </div>
            </div>

            {/* East */}
            <span
              className="text-[10px] font-semibold text-mc-text-dim uppercase tracking-[0.2em]"
              style={{ writingMode: "vertical-lr" }}
            >
              East (+X)
            </span>
          </div>

          {/* South */}
          <span className="text-[10px] font-semibold text-mc-text-dim uppercase tracking-[0.2em]">
            South (+Z)
          </span>
        </div>

        {/* Controls row */}
        <div className="flex justify-center gap-4 mt-4">
          {/* Floor size selector */}
          <div className="flex items-center gap-2">
            {/* X dimension */}
            <div className="flex">
              {([9, 7] as const).map((val) => {
                const isActive =
                  val === (fs.xMax - fs.xMin);
                return (
                  <button
                    key={`x-${val}`}
                    onClick={() => {
                      const zDim = fs.zMax - fs.zMin;
                      const idx =
                        (val === 7 ? 1 : 0) +
                        (zDim === 7 ? 2 : 0);
                      onFloorSizeChange(idx);
                    }}
                    className={`h-8 px-3 text-xs font-semibold transition-colors duration-200 border-y-2 border-x first:border-l-2 last:border-r-2 ${
                      isActive
                        ? "bg-mc-green text-white border-mc-green border-t-mc-green-light border-b-mc-green-dark hover:bg-mc-green-hover"
                        : "bg-mc-bg-darker text-mc-text-dim border-mc-border hover:text-mc-text hover:bg-mc-tab-active"
                    }`}
                  >
                    {val}
                  </button>
                );
              })}
            </div>

            <span className="text-xs text-mc-text-dim font-bold select-none">
              ×
            </span>

            {/* Z dimension */}
            <div className="flex">
              {([9, 7] as const).map((val) => {
                const isActive =
                  val === (fs.zMax - fs.zMin);
                return (
                  <button
                    key={`z-${val}`}
                    onClick={() => {
                      const xDim = fs.xMax - fs.xMin;
                      const idx =
                        (xDim === 7 ? 1 : 0) +
                        (val === 7 ? 2 : 0);
                      onFloorSizeChange(idx);
                    }}
                    className={`h-8 px-3 text-xs font-semibold transition-colors duration-200 border-y-2 border-x first:border-l-2 last:border-r-2 ${
                      isActive
                        ? "bg-mc-green text-white border-mc-green border-t-mc-green-light border-b-mc-green-dark hover:bg-mc-green-hover"
                        : "bg-mc-bg-darker text-mc-text-dim border-mc-border hover:text-mc-text hover:bg-mc-tab-active"
                    }`}
                  >
                    {val}
                  </button>
                );
              })}
            </div>
          </div>

          {/* Input mode toggle */}
          <div className="flex">
            {(
              [
                { mode: "cycle" as InputMode, label: "Cycle", icon: MousePointerClick },
                { mode: "fast" as InputMode, label: "Paint", icon: Paintbrush },
              ] as const
            ).map(({ mode, label, icon: Icon }) => {
              const isActive = inputMode === mode;
              return (
                <button
                  key={mode}
                  onClick={() => setInputMode(mode)}
                  className={`h-8 px-3 text-xs font-semibold transition-colors duration-200 border-y-2 border-x first:border-l-2 last:border-r-2 flex items-center gap-1.5 ${
                    isActive
                      ? "bg-mc-green text-white border-mc-green border-t-mc-green-light border-b-mc-green-dark hover:bg-mc-green-hover"
                      : "bg-mc-bg-darker text-mc-text-dim border-mc-border hover:text-mc-text hover:bg-mc-tab-active"
                  }`}
                >
                  <Icon className="w-3 h-3" />
                  {label}
                </button>
              );
            })}
          </div>

          <button
            onClick={onClear}
            className="mc-btn h-8 !py-0 !px-3 !text-xs"
          >
            <Trash2 className="w-3.5 h-3.5" />
            Clear
          </button>

          {onUsePicture && (
            <button
              onClick={onUsePicture}
              className="mc-btn h-8 !py-0 !px-3 !text-xs"
              title="Import floor pattern from a screenshot"
            >
              <ImageIcon className="w-3.5 h-3.5" />
              Use Picture
            </button>
          )}
        </div>
        {/* Legend */}
        <div className="flex flex-wrap justify-center gap-x-4 gap-y-1.5 mt-3 pt-3 border-t border-mc-border">
          {(
            [
              Tile.Mossy,
              Tile.Cobble,
              Tile.Air,
              Tile.Unknown,
              Tile.UnknownSolid,
            ] as Tile[]
          ).map((tile) => (
            <div
              key={tile}
              className="flex items-center gap-1.5 text-[11px] text-mc-text-dim"
            >
              <Image
                src={TILE_IMAGES[tile]}
                alt={TILE_NAMES[tile]}
                width={14}
                height={14}
                className="pixelated"
              />
              <span>{TILE_NAMES[tile]}</span>
            </div>
          ))}
        </div>

        {/* Pattern input */}
        <div className="mt-3 pt-3 border-t border-mc-border">
          <label className="block text-[11px] font-semibold text-mc-text-dim mb-1.5 uppercase tracking-wider">
            Floor Pattern
          </label>
          <div className="flex gap-1.5">
            <input
              ref={patternRef}
              type="text"
              value={patternString}
              onChange={(e) => {
                const maxLen =
                  (fs.xMax - fs.xMin) * (fs.zMax - fs.zMin);
                // Only allow digits 0-4
                const filtered = e.target.value.replace(/[^0-4]/g, "");
                const cursor = e.target.selectionStart ?? filtered.length;
                if (filtered.length > maxLen) {
                  // Overwrite mode: remove the char right after cursor
                  const trimmed =
                    filtered.slice(0, cursor) +
                    filtered.slice(cursor + 1);
                  cursorRef.current = cursor;
                  onPatternChange(trimmed.slice(0, maxLen));
                } else {
                  cursorRef.current = cursor;
                  onPatternChange(filtered);
                }
              }}
              placeholder={
                "4".repeat(
                  (fs.xMax - fs.xMin) * (fs.zMax - fs.zMin)
                )
              }
              spellCheck={false}
              autoComplete="off"
              className="mc-input font-mono !text-xs tracking-widest flex-1 min-w-0"
            />
            <button
              onClick={() => {
                navigator.clipboard.writeText(patternString);
                setCopied(true);
                setTimeout(() => setCopied(false), 2000);
              }}
              className="mc-btn h-[38px] !py-0 !px-2.5 shrink-0"
              title="Copy pattern"
            >
              {copied ? (
                <Check className="w-3.5 h-3.5" />
              ) : (
                <Copy className="w-3.5 h-3.5" />
              )}
            </button>
          </div>
        </div>
      </div>
    </motion.div>
  );
}
