/* tslint:disable */
/* eslint-disable */

/**
 * Run a partial crack for branches [branch_start, branch_end).
 * Returns JSON with dungeon_seeds, structure_seeds, world_seeds.
 */
export function crack_dungeon_partial_wasm(spawner_x: number, spawner_y: number, spawner_z: number, version: string, biome: string, floor_size: string, floor_grid: Uint8Array, branch_start: number, branch_end: number): string;

/**
 * Original single-shot entry point (non-parallel, kept for compatibility).
 */
export function crack_dungeon_wasm(spawner_x: number, spawner_y: number, spawner_z: number, version: string, biome: string, floor_size: string, floor_grid: Uint8Array): string;

/**
 * Prepare step: parse floor, build reverser, LLL reduce, get branch count.
 * Returns JSON with total_branches (for splitting work), dimensions, etc.
 */
export function prepare_crack_wasm(spawner_x: number, spawner_y: number, spawner_z: number, version: string, biome: string, floor_size: string, floor_grid: Uint8Array): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly crack_dungeon_partial_wasm: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number) => [number, number];
    readonly crack_dungeon_wasm: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => [number, number];
    readonly prepare_crack_wasm: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => [number, number];
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
