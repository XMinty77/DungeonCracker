#!/usr/bin/env bash
# ─────────────────────────────────────────────
# Copy WASM build output from wasm-pack into
# the Next.js project.
#
# Copies:
#   pkg/dungeon_cracker_bg.wasm  →  public/wasm/
#   pkg/dungeon_cracker.js       →  src/lib/wasm-glue.js  (patched)
#   pkg/dungeon_cracker.d.ts     →  src/wasm.d.ts         (patched)
#
# The JS glue is patched to replace wasm-pack's
# default init with an explicit initWasm(url)
# that works in both main thread and Web Workers.
#
# Run from the web/ directory:
#   ./scripts/copy-assets.sh
# ─────────────────────────────────────────────
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WEBUI_DIR="$(dirname "$SCRIPT_DIR")"
ROOT_DIR="$(dirname "$WEBUI_DIR")"

PKG_DIR="$ROOT_DIR/pkg"
WASM_DST="$WEBUI_DIR/public/wasm"
GLUE_DST="$WEBUI_DIR/src/lib/wasm-glue.js"
TYPES_DST="$WEBUI_DIR/src/wasm.d.ts"

if [ ! -d "$PKG_DIR" ]; then
  echo "⚠️  WASM pkg not found at $PKG_DIR"
  echo "   Run 'wasm-pack build --target web -- --features wasm' in the root first."
  exit 1
fi

# ── 1. Copy .wasm binary ──
mkdir -p "$WASM_DST"
echo "📦 Copying WASM binary → $WASM_DST/"
cp "$PKG_DIR/dungeon_cracker_bg.wasm" "$WASM_DST/"

# ── 2. Generate wasm-glue.js from the wasm-pack JS output ──
echo "🔧 Generating $GLUE_DST from wasm-pack output…"

# Take everything from the generated JS except the init/default exports,
# then append our custom initWasm that accepts an explicit URL.
# Remove the `import.meta.url` default and re-export initWasm instead.
{
  cat <<'HEADER'
// ============================================================
// Auto-generated from wasm-pack output by scripts/copy-assets.sh
// DO NOT EDIT — re-run the script after changing the Rust API.
//
// Replaces wasm-pack's default init with initWasm(url) which
// works in both the main thread and Web Workers.
// ============================================================
HEADER

  # Extract the body: everything from the generated JS, stripping:
  #   - the `/* @ts-self-types=... */` pragma
  #   - `export { initSync, __wbg_init as default };`
  #   - the `__wbg_init` function (we replace it)
  #   - the `initSync` function (unused)
  sed \
    -e '/\/\* @ts-self-types/d' \
    -e '/^export { initSync/d' \
    -e 's/^let wasmModule, wasm;/let wasm;/' \
    -e '/^    wasmModule = module;$/d' \
    -e 's/function __wbg_finalize_init(instance, module)/function __wbg_finalize_init(instance)/' \
    -e 's/__wbg_finalize_init(instance, module)/__wbg_finalize_init(instance)/' \
    "$PKG_DIR/dungeon_cracker.js" \
  | awk '
    # Skip initSync function block
    /^function initSync\(/ { skip=1 }
    # Skip __wbg_init function block
    /^async function __wbg_init\(/ { skip=1 }
    skip && /^}$/ { skip=0; next }
    !skip { print }
  '

  # Append the custom initWasm export
  cat <<'INIT'

/**
 * Initialize the WASM module.
 * Accepts a URL string or a pre-compiled WebAssembly.Module.
 * An explicit URL is required — workers cannot resolve relative paths.
 *
 * @param {string | URL | WebAssembly.Module} moduleOrPath
 * @returns {Promise<void>}
 */
export async function initWasm(moduleOrPath) {
  if (wasm !== undefined) return;

  const imports = __wbg_get_imports();

  if (moduleOrPath instanceof WebAssembly.Module) {
    const instance = await WebAssembly.instantiate(moduleOrPath, imports);
    __wbg_finalize_init(instance);
    return;
  }

  if (typeof moduleOrPath === 'string' || (typeof URL === 'function' && moduleOrPath instanceof URL)) {
    moduleOrPath = fetch(moduleOrPath);
  }

  const { instance } = await __wbg_load(await moduleOrPath, imports);
  __wbg_finalize_init(instance);
}
INIT
} > "$GLUE_DST"

# ── 3. Generate type declarations ──
echo "📝 Generating $TYPES_DST from wasm-pack output…"
{
  cat <<'TYPES_HEADER'
// Auto-generated from wasm-pack output by scripts/copy-assets.sh
// DO NOT EDIT — re-run the script after changing the Rust API.

declare module "@/lib/wasm-glue.js" {
  export function initWasm(
    moduleOrPath: string | URL | WebAssembly.Module
  ): Promise<void>;

TYPES_HEADER

  # Extract function declarations from the .d.ts, skipping the
  # default export, initSync, and the InitInput/InitOutput types.
  grep -E '^export function ' "$PKG_DIR/dungeon_cracker.d.ts" \
    | grep -v '__wbg_init\|initSync' \
    | sed 's/^/  /'

  echo "}"
} > "$TYPES_DST"

echo "✅ Done! WASM glue is in sync with the Rust API."
