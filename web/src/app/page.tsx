"use client";

import { useState, useCallback, useEffect, useRef } from "react";
import { motion } from "framer-motion";
import {
  Loader2,
  AlertTriangle,
  XCircle,
  Plus,
  X,
  Layers,
  ChevronRight,
  Cpu,
} from "lucide-react";
import { Header } from "@/components/Header";
import { DungeonPanel, createEmptyFloor } from "@/components/DungeonPanel";
import { MultiDungeonResults } from "@/components/MultiDungeonResults";
import { ParticleBackground } from "@/components/ParticleBackground";
import { PictureImportDialog } from "@/components/PictureImportDialog";
import { useCracker } from "@/hooks/useCracker";
import { useImageDrop } from "@/hooks/useImageDrop";
import {
  Tile,
  FLOOR_SIZES,
  type CrackResult,
  type DungeonEntry,
} from "@/lib/types";
import {
  serializeDungeons,
  deserializeDungeons,
} from "@/lib/hash-serialization";
import { hasAnimated, markAnimated } from "@/lib/initial-animation";

let nextDungeonId = 2; // starts at 2 because the initial dungeon is always id "1"
function makeDungeon(label?: string): DungeonEntry {
  const id = String(nextDungeonId++);
  return {
    id,
    label: label ?? `Dungeon ${id}`,
    floorData: createEmptyFloor(),
    floorSizeIndex: 0,
    spawnerX: "",
    spawnerY: "",
    spawnerZ: "",
    version: "1.13",
    biome: "notdesert",
  };
}

/** The very first dungeon — a stable constant so SSR and client always agree. */
const INITIAL_DUNGEON: DungeonEntry = {
  id: "1",
  label: "Dungeon 1",
  floorData: createEmptyFloor(),
  floorSizeIndex: 0,
  spawnerX: "",
  spawnerY: "",
  spawnerZ: "",
  version: "1.13",
  biome: "notdesert",
};

function isValid(d: DungeonEntry) {
  return (
    d.spawnerX !== "" &&
    d.spawnerY !== "" &&
    d.spawnerZ !== "" &&
    !isNaN(parseInt(d.spawnerX)) &&
    !isNaN(parseInt(d.spawnerY)) &&
    !isNaN(parseInt(d.spawnerZ))
  );
}

/** Special sentinel id for the multi-dungeons tab */
const MULTI_TAB_ID = "__multi__";

export default function Home() {
  // ── Multi-dungeon state ──
  const [dungeons, setDungeons] = useState<DungeonEntry[]>([INITIAL_DUNGEON]);
  const [activeTabId, setActiveTabId] = useState<string>("1"); // dungeon id or MULTI_TAB_ID

  // Per-dungeon crack results (keyed by dungeon id)
  const [dungeonResults, setDungeonResults] = useState<
    Record<string, CrackResult>
  >({});

  // Multi-crack state
  const [multiCrackActive, setMultiCrackActive] = useState(false);
  const [currentCrackingId, setCurrentCrackingId] = useState<string | null>(null);
  const crackQueueRef = useRef<string[]>([]);

  const [pictureDialogOpen, setPictureDialogOpen] = useState(false);

  const cracker = useCracker();

  // Keep a ref to dungeons so async callbacks see the latest
  const dungeonsRef = useRef(dungeons);
  dungeonsRef.current = dungeons;

  const isMultiTab = activeTabId === MULTI_TAB_ID;
  const activeDungeon = isMultiTab
    ? null
    : (dungeons.find((d) => d.id === activeTabId) ?? dungeons[0]);

  // ── Restore state from URL hash on mount ──
  const initializedFromHash = useRef(false);
  useEffect(() => {
    if (initializedFromHash.current) return;
    initializedFromHash.current = true;

    const raw = window.location.hash.replace(/^#/, "");
    if (!raw) return;

    const restored = deserializeDungeons(raw);
    if (!restored || restored.length === 0) return;

    // Reset the id counter past any restored ids
    const maxId = restored.reduce(
      (max, d) => Math.max(max, parseInt(d.id) || 0),
      0
    );
    nextDungeonId = maxId + 1;

    setDungeons(restored);
    setActiveTabId(restored[0].id);
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  // ── Persist state to URL hash when dungeons change ──
  const isFirstRender = useRef(true);
  useEffect(() => {
    // Skip the very first render (before hash restore has a chance to run)
    if (isFirstRender.current) {
      isFirstRender.current = false;
      return;
    }
    const hash = serializeDungeons(dungeons);
    window.history.replaceState(null, "", hash ? `#${hash}` : window.location.pathname);
  }, [dungeons]);

  // ── Mark initial animation as done after first paint ──
  useEffect(() => {
    if (hasAnimated()) return;
    const timer = setTimeout(markAnimated, 800);
    return () => clearTimeout(timer);
  }, []);

  // ── Dungeon management ──
  const addDungeon = useCallback(() => {
    const d = makeDungeon();
    setDungeons((prev) => [...prev, d]);
    setActiveTabId(d.id);
  }, []);

  const removeDungeon = useCallback(
    (id: string) => {
      setDungeons((prev) => {
        if (prev.length <= 1) return prev;
        const next = prev.filter((d) => d.id !== id);
        if (activeTabId === id) {
          const removedIdx = prev.findIndex((d) => d.id === id);
          const newActive = next[Math.min(removedIdx, next.length - 1)];
          setActiveTabId(newActive.id);
        }
        return next;
      });
      setDungeonResults((prev) => {
        const next = { ...prev };
        delete next[id];
        return next;
      });
    },
    [activeTabId]
  );

  const updateDungeon = useCallback((updated: DungeonEntry) => {
    setDungeons((prev) =>
      prev.map((d) => (d.id === updated.id ? updated : d))
    );
  }, []);

  // ── Apply a detected floor from image analysis ──
  const handleImageApply = useCallback(
    (floor: Tile[][], sizeIndex: number) => {
      if (!activeDungeon) return;
      updateDungeon({ ...activeDungeon, floorData: floor, floorSizeIndex: sizeIndex });
    },
    [activeDungeon, updateDungeon]
  );

  // ── Global paste / drag-drop (when dialog is NOT open) ──
  useImageDrop({
    floorSizeIndex: activeDungeon?.floorSizeIndex ?? 0,
    dialogOpen: pictureDialogOpen,
    onApply: handleImageApply,
  });

  const isCracking =
    cracker.status === "cracking" || cracker.status === "preparing";

  // ── Start cracking a specific dungeon ──
  const startCrackForDungeon = useCallback(
    (dungeon: DungeonEntry) => {
      const fs = FLOOR_SIZES[dungeon.floorSizeIndex];
      const flatGrid = new Uint8Array(81);
      for (let z = 0; z < 9; z++) {
        for (let x = 0; x < 9; x++) {
          flatGrid[z * 9 + x] = dungeon.floorData[z][x];
        }
      }

      cracker.crack({
        spawnerX: parseInt(dungeon.spawnerX),
        spawnerY: parseInt(dungeon.spawnerY),
        spawnerZ: parseInt(dungeon.spawnerZ),
        version: dungeon.version,
        biome: dungeon.biome,
        floorSize: fs.key,
        floorGrid: flatGrid,
      });
    },
    [cracker]
  );

  // ── Crack only the active dungeon ──
  const handleCrackSingle = useCallback(() => {
    if (isCracking || multiCrackActive) {
      cracker.stop();
      setMultiCrackActive(false);
      crackQueueRef.current = [];
      setCurrentCrackingId(null);
      return;
    }

    if (!activeDungeon || !isValid(activeDungeon) || !cracker.workersReady) return;

    // Clear this dungeon's old result
    setDungeonResults((prev) => {
      const next = { ...prev };
      delete next[activeDungeon.id];
      return next;
    });

    setCurrentCrackingId(activeDungeon.id);
    setMultiCrackActive(false);
    crackQueueRef.current = [];

    startCrackForDungeon(activeDungeon);
  }, [isCracking, multiCrackActive, activeDungeon, cracker, startCrackForDungeon]);

  // ── Save single crack result when done (non-multi mode) ──
  useEffect(() => {
    if (
      !multiCrackActive &&
      cracker.status === "done" &&
      cracker.result &&
      currentCrackingId
    ) {
      setDungeonResults((prev) => ({
        ...prev,
        [currentCrackingId]: cracker.result!,
      }));
    }
  }, [multiCrackActive, cracker.status, cracker.result, currentCrackingId]);

  // ── Pick the next dungeon from the multi-crack queue ──
  const processNextInQueue = useCallback(() => {
    const queue = crackQueueRef.current;
    if (queue.length === 0) {
      setMultiCrackActive(false);
      setCurrentCrackingId(null);
      return;
    }

    const nextId = queue.shift()!;
    const nextDungeon = dungeonsRef.current.find((d) => d.id === nextId);

    if (!nextDungeon || !isValid(nextDungeon)) {
      processNextInQueue();
      return;
    }

    setCurrentCrackingId(nextId);

    setTimeout(() => {
      startCrackForDungeon(nextDungeon);
    }, 50);
  }, [startCrackForDungeon]);

  // ── Watch cracker status to drive multi-crack queue ──
  useEffect(() => {
    if (!multiCrackActive) return;

    if (cracker.status === "done" && cracker.result && currentCrackingId) {
      setDungeonResults((prev) => ({
        ...prev,
        [currentCrackingId]: cracker.result!,
      }));
      processNextInQueue();
    }

    if (cracker.status === "error" && currentCrackingId) {
      processNextInQueue();
    }
  }, [multiCrackActive, cracker.status, cracker.result, currentCrackingId, processNextInQueue]);

  // ── Crack all valid dungeons sequentially (skipping already-cracked) ──
  const handleCrackAll = useCallback(() => {
    if (multiCrackActive || isCracking) {
      cracker.stop();
      setMultiCrackActive(false);
      crackQueueRef.current = [];
      setCurrentCrackingId(null);
      return;
    }

    // Only queue valid dungeons that don't already have results
    const uncrackedValidIds = dungeons
      .filter((d) => isValid(d) && !dungeonResults[d.id])
      .map((d) => d.id);
    if (uncrackedValidIds.length === 0) return;

    const [firstId, ...rest] = uncrackedValidIds;
    crackQueueRef.current = rest;
    setMultiCrackActive(true);

    const firstDungeon = dungeons.find((d) => d.id === firstId)!;
    setCurrentCrackingId(firstId);
    startCrackForDungeon(firstDungeon);
  }, [multiCrackActive, isCracking, dungeons, dungeonResults, cracker, startCrackForDungeon]);

  // ── Computed results for multi-dungeon display ──
  const allResults = dungeons
    .filter((d) => dungeonResults[d.id])
    .map((d) => ({
      label: d.label,
      result: dungeonResults[d.id],
    }));

  const validCount = dungeons.filter(isValid).length;
  const uncrackedValidCount = dungeons.filter(
    (d) => isValid(d) && !dungeonResults[d.id]
  ).length;
  const crackedCount = Object.keys(dungeonResults).length;

  // ── Is the cracker active for the viewed dungeon? ──
  const isActiveDungeonCracking =
    isCracking && activeDungeon != null && currentCrackingId === activeDungeon.id;

  // True when a single-dungeon crack is running (not multi-crack)
  const singleCrackBusy = !multiCrackActive && isCracking;
  const singleCrackLabel = singleCrackBusy
    ? (dungeons.find((d) => d.id === currentCrackingId)?.label ?? "dungeon")
    : null;

  // Per-dungeon result for the active dungeon
  const activeDungeonResult = activeDungeon ? dungeonResults[activeDungeon.id] ?? null : null;
  // Show live cracker result if it's for the active dungeon and not yet saved
  const displayResult =
    activeDungeonResult ??
    (isActiveDungeonCracking || (cracker.status === "done" && currentCrackingId === activeDungeon?.id)
      ? cracker.result
      : null);

  // ── Editing the dungeon label ──
  const [editingLabelId, setEditingLabelId] = useState<string | null>(null);

  return (
    <div className="min-h-dvh flex flex-col bg-mc-bg-dark">
      <ParticleBackground />
      <Header />

      <main className="relative z-10 flex-1 max-w-7xl mx-auto w-full px-4 md:px-6 py-6 md:py-8">
        {/* ── Tab Bar ── */}
        <motion.div
          initial={hasAnimated() ? false : { opacity: 0, y: -10 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3, ease: "easeOut" }}
          className="flex items-stretch gap-0 mb-6"
        >
          {/* Scrollable dungeon tabs */}
          <div className="flex items-stretch gap-1 overflow-x-auto pb-1 min-w-0 flex-1">
            {dungeons.map((d) => {
              const isActive = d.id === activeTabId;

              return (
                <div
                  key={d.id}
                  className={`flex items-center gap-1 px-3 py-2 text-xs font-semibold transition-colors duration-200 border-b-2 cursor-pointer select-none flex-shrink-0 ${
                    isActive
                      ? "bg-mc-tab-active text-mc-text-highlight border-mc-green"
                      : "bg-mc-bg text-mc-text-dim hover:bg-mc-tab-active hover:text-mc-text border-transparent"
                  }`}
                  onClick={() => setActiveTabId(d.id)}
                >
                  {editingLabelId === d.id ? (
                    <input
                      autoFocus
                      type="text"
                      value={d.label}
                      onChange={(e) => updateDungeon({ ...d, label: e.target.value })}
                      onBlur={() => setEditingLabelId(null)}
                      onKeyDown={(e) => {
                        if (e.key === "Enter") setEditingLabelId(null);
                      }}
                      className="bg-transparent border-b border-mc-text-dim text-xs text-mc-text-highlight outline-none w-24"
                      onClick={(e) => e.stopPropagation()}
                    />
                  ) : (
                    <span
                      className="truncate max-w-[120px]"
                      onDoubleClick={(e) => {
                        e.stopPropagation();
                        setEditingLabelId(d.id);
                      }}
                      title="Double-click to rename"
                    >
                      {d.label}
                    </span>
                  )}

                  {dungeons.length > 1 && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        removeDungeon(d.id);
                      }}
                      className="p-0.5 hover:bg-mc-bg-darker rounded transition-colors duration-200 flex-shrink-0 cursor-pointer"
                      title="Remove dungeon"
                    >
                      <X className="w-3 h-3" />
                    </button>
                  )}
                </div>
              );
            })}

            <button
              onClick={addDungeon}
              className="flex items-center gap-1 px-3 py-2 text-xs font-semibold text-mc-text-dim hover:text-mc-text hover:bg-mc-tab-active transition-colors duration-200 cursor-pointer border-b-2 border-transparent flex-shrink-0"
              title="Add dungeon"
            >
              <Plus className="w-3.5 h-3.5" />
              <span className="hidden sm:inline">Add Dungeon</span>
            </button>
          </div>

          {/* Pinned Multi-Dungeons button */}
          <div className="flex-shrink-0 border-l border-mc-border pl-1 ml-1">
            <button
              onClick={() => setActiveTabId(MULTI_TAB_ID)}
              className={`flex items-center gap-1.5 px-3 py-2 text-xs font-semibold transition-colors duration-200 border-b-2 cursor-pointer select-none h-full ${
                isMultiTab
                  ? "bg-mc-tab-active text-mc-text-highlight border-mc-green"
                  : "bg-mc-bg text-mc-text-dim hover:bg-mc-tab-active hover:text-mc-text border-transparent"
              }`}
            >
              <Layers className="w-3.5 h-3.5" />
              <span className="hidden sm:inline">Multi-Dungeons</span>
              {crackedCount > 0 && (
                <span className="text-[10px] font-bold px-1.5 py-px border bg-[#6CC34922] text-[#6CC349] border-[#6CC34944]">
                  {crackedCount}
                </span>
              )}
            </button>
          </div>
        </motion.div>

        {/* ── Active dungeon editor ── */}
        {!isMultiTab && activeDungeon && (
          <DungeonPanel
            key={activeDungeon.id}
            dungeon={activeDungeon}
            onChange={updateDungeon}
            onUsePicture={() => setPictureDialogOpen(true)}
            onCrack={handleCrackSingle}
            crackerStatus={cracker.status}
            crackerProgress={cracker.progress}
            crackerError={cracker.error}
            crackerResult={displayResult}
            workersReady={cracker.workersReady}
            isCracking={!!isActiveDungeonCracking}
            multiCrackActive={multiCrackActive}
          />
        )}

        {/* ── Multi-Dungeons tab ── */}
        {isMultiTab && (
          <motion.div
            initial={hasAnimated() ? false : { opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ duration: 0.4, ease: "easeOut" }}
            className="space-y-5"
          >
            {/* Crack All button */}
            <motion.div
              initial={hasAnimated() ? false : { opacity: 0, x: 30 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ duration: 0.5, delay: 0.1, ease: "easeOut" }}
            >
              <button
                onClick={handleCrackAll}
                disabled={
                  ((!cracker.workersReady || uncrackedValidCount === 0) &&
                  !multiCrackActive) || singleCrackBusy
                }
                className={`mc-btn w-full !py-3 !text-sm relative overflow-hidden ${
                  multiCrackActive ? "mc-btn-red" : ""
                }`}
              >
                <span className="relative z-10 flex items-center justify-center gap-2">
                  {multiCrackActive ? (
                    <>
                      <XCircle className="w-4 h-4" />
                      Stop All
                    </>
                  ) : singleCrackBusy ? (
                    <>
                      <Loader2 className="w-4 h-4 animate-spin" />
                      Cracking {singleCrackLabel}…
                    </>
                  ) : cracker.status === "loading" ? (
                    <>
                      <Cpu className="w-4 h-4 animate-pulse" />
                      Loading WASM…
                    </>
                  ) : (
                    <>
                      <ChevronRight className="w-4 h-4" />
                      Crack All Dungeons
                      {uncrackedValidCount > 0 && (
                        <span className="text-[10px] font-bold px-1.5 py-px border bg-white/10 border-white/20">
                          {uncrackedValidCount} remaining
                        </span>
                      )}
                    </>
                  )}
                </span>
              </button>

              {/* Progress bar for multi-crack */}
              {(multiCrackActive || (crackedCount > 0 && !isCracking)) && validCount > 0 && (
                <div className="mt-2 space-y-1">
                  <div className="w-full h-1 bg-mc-bg-darker border border-mc-border overflow-hidden">
                    <motion.div
                      className="h-full bg-mc-green progress-bar-shimmer"
                      initial={{ width: 0 }}
                      animate={{
                        width: multiCrackActive
                          ? `${Math.round(((crackedCount + (isCracking ? cracker.progress / 100 : 0)) / validCount) * 100)}%`
                          : `${Math.round((crackedCount / validCount) * 100)}%`,
                      }}
                      transition={{ duration: 0.4, ease: "easeOut" }}
                    />
                  </div>
                  {multiCrackActive && (
                    <div className="flex items-center gap-2 text-xs text-mc-text-dim">
                      <Loader2 className="w-3.5 h-3.5 animate-spin" />
                      <span>
                        {currentCrackingId
                          ? `Cracking ${
                              dungeons.find((d) => d.id === currentCrackingId)?.label ?? "dungeon"
                            }… ${cracker.progress}%`
                          : "Preparing…"}
                        <span className="ml-1">
                          ({crackedCount}/{validCount} done)
                        </span>
                      </span>
                    </div>
                  )}
                  {!multiCrackActive && crackedCount > 0 && (
                    <div className="text-xs text-mc-text-dim">
                      {crackedCount}/{validCount} dungeons cracked
                      {uncrackedValidCount > 0 && ` — ${uncrackedValidCount} remaining`}
                    </div>
                  )}
                </div>
              )}

              {/* Progress bar for single-dungeon crack viewed from multi tab */}
              {singleCrackBusy && (
                <div className="mt-2 space-y-1">
                  <div className="w-full h-1 bg-mc-bg-darker border border-mc-border overflow-hidden">
                    <motion.div
                      className="h-full bg-mc-green progress-bar-shimmer"
                      initial={{ width: 0 }}
                      animate={{ width: `${cracker.progress}%` }}
                      transition={{ duration: 0.4, ease: "easeOut" }}
                    />
                  </div>
                  <div className="flex items-center gap-2 text-xs text-mc-text-dim">
                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                    <span>Cracking {singleCrackLabel}… {cracker.progress}%</span>
                  </div>
                </div>
              )}

              {/* Error message */}
              {cracker.status === "error" && cracker.error && multiCrackActive && (
                <div className="flex items-center gap-2 mt-2 p-2.5 border border-mc-red bg-mc-bg-darker">
                  <AlertTriangle className="w-3.5 h-3.5 text-mc-red-text flex-shrink-0" />
                  <p className="text-xs text-mc-red-text">{cracker.error}</p>
                </div>
              )}
            </motion.div>

            {/* Multi-dungeon combined results */}
            {allResults.length > 0 && <MultiDungeonResults results={allResults} />}

            {allResults.length === 0 && !multiCrackActive && (
              <div className="mc-panel p-6 text-center">
                <Layers className="w-8 h-8 text-mc-text-dim mx-auto mb-3 opacity-50" />
                <p className="text-sm text-mc-text-dim mb-1">No dungeons cracked yet</p>
                <p className="text-xs text-mc-text-dim opacity-70">
                  Crack individual dungeons from their tabs, or use the button above to crack all at once.
                  World seeds common to all dungeons will appear here.
                </p>
              </div>
            )}
          </motion.div>
        )}
      </main>

      {/* ── Picture Import Dialog ── */}
      <PictureImportDialog
        open={pictureDialogOpen}
        floorSizeIndex={activeDungeon?.floorSizeIndex ?? 0}
        onClose={() => setPictureDialogOpen(false)}
        onApply={handleImageApply}
      />
    </div>
  );
}
