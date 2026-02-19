// ============================================================
// Auto-generated from wasm-pack output by scripts/copy-assets.sh
// DO NOT EDIT — re-run the script after changing the Rust API.
//
// Replaces wasm-pack's default init with initWasm(url) which
// works in both the main thread and Web Workers.
// ============================================================

/**
 * Run a partial crack for branches [branch_start, branch_end).
 * Returns JSON with dungeon_seeds, structure_seeds, world_seeds.
 * @param {number} spawner_x
 * @param {number} spawner_y
 * @param {number} spawner_z
 * @param {string} version
 * @param {string} biome
 * @param {string} floor_size
 * @param {Uint8Array} floor_grid
 * @param {number} branch_start
 * @param {number} branch_end
 * @returns {string}
 */
export function crack_dungeon_partial_wasm(spawner_x, spawner_y, spawner_z, version, biome, floor_size, floor_grid, branch_start, branch_end) {
    let deferred5_0;
    let deferred5_1;
    try {
        const ptr0 = passStringToWasm0(version, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(biome, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(floor_size, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passArray8ToWasm0(floor_grid, wasm.__wbindgen_malloc);
        const len3 = WASM_VECTOR_LEN;
        const ret = wasm.crack_dungeon_partial_wasm(spawner_x, spawner_y, spawner_z, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3, branch_start, branch_end);
        deferred5_0 = ret[0];
        deferred5_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred5_0, deferred5_1, 1);
    }
}

/**
 * Original single-shot entry point (non-parallel, kept for compatibility).
 * @param {number} spawner_x
 * @param {number} spawner_y
 * @param {number} spawner_z
 * @param {string} version
 * @param {string} biome
 * @param {string} floor_size
 * @param {Uint8Array} floor_grid
 * @returns {string}
 */
export function crack_dungeon_wasm(spawner_x, spawner_y, spawner_z, version, biome, floor_size, floor_grid) {
    let deferred5_0;
    let deferred5_1;
    try {
        const ptr0 = passStringToWasm0(version, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(biome, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(floor_size, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passArray8ToWasm0(floor_grid, wasm.__wbindgen_malloc);
        const len3 = WASM_VECTOR_LEN;
        const ret = wasm.crack_dungeon_wasm(spawner_x, spawner_y, spawner_z, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
        deferred5_0 = ret[0];
        deferred5_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred5_0, deferred5_1, 1);
    }
}

/**
 * Prepare step: parse floor, build reverser, LLL reduce, get branch count.
 * Returns JSON with total_branches (for splitting work), dimensions, etc.
 * @param {number} spawner_x
 * @param {number} spawner_y
 * @param {number} spawner_z
 * @param {string} version
 * @param {string} biome
 * @param {string} floor_size
 * @param {Uint8Array} floor_grid
 * @returns {string}
 */
export function prepare_crack_wasm(spawner_x, spawner_y, spawner_z, version, biome, floor_size, floor_grid) {
    let deferred5_0;
    let deferred5_1;
    try {
        const ptr0 = passStringToWasm0(version, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(biome, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(floor_size, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passArray8ToWasm0(floor_grid, wasm.__wbindgen_malloc);
        const len3 = WASM_VECTOR_LEN;
        const ret = wasm.prepare_crack_wasm(spawner_x, spawner_y, spawner_z, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
        deferred5_0 = ret[0];
        deferred5_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred5_0, deferred5_1, 1);
    }
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./dungeon_cracker_bg.js": import0,
    };
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasm;
function __wbg_finalize_init(instance) {
    wasm = instance.exports;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}




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
