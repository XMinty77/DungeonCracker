"use client";

import { useCallback, useEffect, useRef, useState } from "react";
import type {
  CrackResult,
  CrackStatus,
  PrepareResult,
} from "@/lib/types";
import { BASE_PATH } from "@/lib/types";

const NUM_WORKERS =
  typeof navigator !== "undefined"
    ? Math.max(1, navigator.hardwareConcurrency || 4)
    : 4;

interface CrackerState {
  status: CrackStatus;
  progress: number; // 0–100
  error: string | null;
  result: CrackResult | null;
  prepareInfo: PrepareResult | null;
  workersReady: boolean;
}

interface CrackParams {
  spawnerX: number;
  spawnerY: number;
  spawnerZ: number;
  version: string;
  biome: string;
  floorSize: string;
  floorGrid: Uint8Array;
}

export function useCracker() {
  const [state, setState] = useState<CrackerState>({
    status: "loading",
    progress: 0,
    error: null,
    result: null,
    prepareInfo: null,
    workersReady: false,
  });

  const workersRef = useRef<Worker[]>([]);
  const workersReadyCount = useRef(0);
  const wasmGlueRef = useRef<{
    prepare_crack_wasm: (
      x: number,
      y: number,
      z: number,
      version: string,
      biome: string,
      floorSize: string,
      grid: Uint8Array
    ) => string;
  } | null>(null);

  // Initialize WASM on main thread (for prepare) + spawn workers
  useEffect(() => {
    let cancelled = false;

    async function init() {
      try {
        console.log("[DungeonCracker] Loading WebAssembly module & spawning workers…");

        // Build an absolute URL for the WASM file — workers can't
        // resolve root-relative paths like "/wasm/..." on their own.
        const wasmUrl = new URL(
          `${BASE_PATH}/wasm/dungeon_cracker_bg.wasm`,
          window.location.origin
        ).href;

        // Load the self-contained WASM glue on the main thread
        const glue = await import("@/lib/wasm-glue.js");
        await glue.initWasm(wasmUrl);
        wasmGlueRef.current = glue;

        // Spawn workers
        const workers: Worker[] = [];
        workersReadyCount.current = 0;

        const workerPromises: Promise<void>[] = [];

        for (let i = 0; i < NUM_WORKERS; i++) {
          const w = new Worker(
            new URL("../workers/cracker.worker.ts", import.meta.url)
          );
          workers.push(w);

          workerPromises.push(
            new Promise<void>((resolve, reject) => {
              w.onmessage = (e) => {
                if (e.data.type === "init_done") {
                  workersReadyCount.current++;
                  resolve();
                } else if (e.data.type === "error") {
                  reject(new Error(e.data.error));
                }
              };
            })
          );

          w.postMessage({
            type: "init",
            wasmUrl,
          });
        }

        await Promise.all(workerPromises);

        workersRef.current = workers;

        if (!cancelled) {
          console.log(`[DungeonCracker] WASM loaded, ${NUM_WORKERS} workers ready.`);
          setState((s) => ({
            ...s,
            status: "idle",
            workersReady: true,
          }));
        }
      } catch (err) {
        console.error("[DungeonCracker] Failed to initialize:", err);
        if (!cancelled) {
          setState((s) => ({
            ...s,
            status: "error",
            error: `Failed to initialize: ${err}`,
          }));
        }
      }
    }

    init();

    return () => {
      cancelled = true;
      workersRef.current.forEach((w) => w.terminate());
    };
  }, []);

  // Crack function
  const crack = useCallback(async (params: CrackParams) => {
    const glue = wasmGlueRef.current;
    if (!glue) return;

    setState((s) => ({
      ...s,
      status: "preparing",
      progress: 0,
      error: null,
      result: null,
      prepareInfo: null,
    }));

    console.log("[DungeonCracker] Preparing crack — parsing floor, building reverser…");

    try {
      // 1. Prepare step (lightweight, on main thread)
      const prepareJson = glue.prepare_crack_wasm(
        params.spawnerX,
        params.spawnerY,
        params.spawnerZ,
        params.version,
        params.biome,
        params.floorSize,
        params.floorGrid
      );
      const prepareResult: PrepareResult = JSON.parse(prepareJson);

      if (prepareResult.error) {
        throw new Error(prepareResult.error);
      }

      console.log("[DungeonCracker] Prepare done:", {
        total_branches: prepareResult.total_branches,
        info_bits: prepareResult.info_bits,
        possibilities: prepareResult.possibilities,
      });
      console.log(`[DungeonCracker] Cracking seeds across ${workersRef.current.length} workers…`);

      setState((s) => ({
        ...s,
        status: "cracking",
        prepareInfo: prepareResult,
      }));

      // 2. Split branches across workers
      const totalBranches = prepareResult.total_branches;
      const workers = workersRef.current;
      const numWorkers = workers.length;
      const branchesPerWorker = Math.ceil(totalBranches / numWorkers);

      const mergedResult: CrackResult = {
        dungeon_seeds: [],
        structure_seeds: [],
        world_seeds: [],
      };

      let completedChunks = 0;
      const totalChunks = numWorkers;

      await new Promise<void>((resolve, reject) => {
        let finished = 0;

        workers.forEach((w, i) => {
          const branchStart = i * branchesPerWorker;
          const branchEnd = Math.min(
            (i + 1) * branchesPerWorker,
            totalBranches
          );

          if (branchStart >= totalBranches) {
            finished++;
            completedChunks++;
            if (finished === totalChunks) resolve();
            return;
          }

          // Send floorGrid as a plain Array (not ArrayBuffer) so it
          // survives structured-clone without transfer issues.
          const gridArray = Array.from(params.floorGrid);

          w.onmessage = (e) => {
            const msg = e.data;
            if (msg.type === "crack_done") {
              mergedResult.dungeon_seeds.push(...msg.result.dungeon_seeds);
              mergedResult.structure_seeds.push(
                ...msg.result.structure_seeds
              );
              mergedResult.world_seeds.push(...msg.result.world_seeds);
              completedChunks++;
              const pct = Math.round((completedChunks / totalChunks) * 100);
              console.log(`[DungeonCracker] Worker ${i} done (${pct}% complete)`);
              setState((s) => ({
                ...s,
                progress: pct,
              }));
              finished++;
              if (finished === totalChunks) resolve();
            } else if (msg.type === "error") {
              reject(new Error(msg.error));
            }
          };

          w.postMessage({
            type: "crack",
            params: {
              spawnerX: params.spawnerX,
              spawnerY: params.spawnerY,
              spawnerZ: params.spawnerZ,
              version: params.version,
              biome: params.biome,
              floorSize: params.floorSize,
              floorGrid: gridArray,
              branchStart,
              branchEnd,
            },
          });
        });
      });

      // Deduplicate seeds
      mergedResult.dungeon_seeds = [...new Set(mergedResult.dungeon_seeds)];
      mergedResult.structure_seeds = [
        ...new Set(mergedResult.structure_seeds),
      ];
      mergedResult.world_seeds = [...new Set(mergedResult.world_seeds)];

      console.log("[DungeonCracker] Crack complete!", {
        dungeon_seeds: mergedResult.dungeon_seeds.length,
        structure_seeds: mergedResult.structure_seeds.length,
        world_seeds: mergedResult.world_seeds.length,
      });

      setState((s) => ({
        ...s,
        status: "done",
        progress: 100,
        result: mergedResult,
      }));
    } catch (err) {
      console.error("[DungeonCracker] Crack failed:", err);
      setState((s) => ({
        ...s,
        status: "error",
        error: `Crack failed: ${err}`,
      }));
    }
  }, []);

  const reset = useCallback(() => {
    setState((s) => ({
      ...s,
      status: s.workersReady ? "idle" : "loading",
      progress: 0,
      error: null,
      result: null,
      prepareInfo: null,
    }));
  }, []);

  return { ...state, crack, reset };
}
