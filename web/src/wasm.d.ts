// Auto-generated from wasm-pack output by scripts/copy-assets.sh
// DO NOT EDIT — re-run the script after changing the Rust API.

declare module "@/lib/wasm-glue.js" {
  export function initWasm(
    moduleOrPath: string | URL | WebAssembly.Module
  ): Promise<void>;

  export function crack_dungeon_partial_wasm(spawner_x: number, spawner_y: number, spawner_z: number, version: string, biome: string, floor_size: string, floor_grid: Uint8Array, branch_start: number, branch_end: number): string;
  export function crack_dungeon_wasm(spawner_x: number, spawner_y: number, spawner_z: number, version: string, biome: string, floor_size: string, floor_grid: Uint8Array): string;
  export function prepare_crack_wasm(spawner_x: number, spawner_y: number, spawner_z: number, version: string, biome: string, floor_size: string, floor_grid: Uint8Array): string;
}
