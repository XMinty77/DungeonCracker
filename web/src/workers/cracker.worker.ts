// ============================================================
// Dungeon Cracker — Web Worker
// Each worker loads its own WASM instance and processes a
// range of depth-0 branches.
//
// Uses the self-contained wasm-glue module that fetches +
// instantiates WASM from the public URL at runtime.
// ============================================================

import {
  initWasm,
  crack_dungeon_partial_wasm,
} from "../lib/wasm-glue.js";

// We export {} so TypeScript treats this as a module.
export {};

let ready = false;

self.onmessage = async function (e: MessageEvent) {
  const msg = e.data;

  if (msg.type === "init") {
    try {
      await initWasm(msg.wasmUrl);
      ready = true;
      self.postMessage({ type: "init_done" });
    } catch (err) {
      self.postMessage({ type: "error", error: `Init failed: ${err}` });
    }
    return;
  }

  if (msg.type === "crack") {
    if (!ready) {
      self.postMessage({ type: "error", error: "WASM not initialized" });
      return;
    }
    try {
      const p = msg.params;
      const jsonStr = crack_dungeon_partial_wasm(
        p.spawnerX,
        p.spawnerY,
        p.spawnerZ,
        p.version,
        p.biome,
        p.floorSize,
        new Uint8Array(p.floorGrid),
        p.branchStart,
        p.branchEnd
      );
      const result = JSON.parse(jsonStr);
      self.postMessage({
        type: "crack_done",
        result,
        branchStart: p.branchStart,
        branchEnd: p.branchEnd,
      });
    } catch (err) {
      self.postMessage({ type: "error", error: `Crack failed: ${err}` });
    }
    return;
  }
};
