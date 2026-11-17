/* @ts-self-types="./jubarte_wasm.d.ts" */

/**
 * What [`renderPdf`](render_pdf) returns.
 */
export class RenderOutput {
    static __wrap(ptr) {
        const obj = Object.create(RenderOutput.prototype);
        obj.__wbg_ptr = ptr;
        RenderOutputFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        RenderOutputFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_renderoutput_free(ptr, 0);
    }
    /**
     * `{"page_count", "page_starts"}`: the pages written and each one's
     * first line (60 characters at most), as an edit bundle's `render`
     * line lists them.
     * @returns {string}
     */
    get json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
            wasm.renderoutput_json(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * The PDF.
     * @returns {Uint8Array}
     */
    get pdf() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
            wasm.renderoutput_pdf(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_export(r0, r1 * 1, 1);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
if (Symbol.dispose) RenderOutput.prototype[Symbol.dispose] = RenderOutput.prototype.free;

/**
 * Render a DOCX package (bytes) → PDF bytes (Word-style layout).
 *
 * Mirrors `jubarte::convert::docx_to_pdf`. Fonts come from the embedded
 * Carlito / Liberation set; the native system/cloud font overrides are
 * no-ops under wasm (no filesystem), which only changes glyph sourcing,
 * never layout metrics.
 * `compress` (optional, default `false`) deflates the PDF's streams
 * (`/FlateDecode`): much smaller output, no longer plain text.
 * `revisions` (optional, default `"conventional"`) paints tracked changes:
 * `"conventional"`, `"word"` (Microsoft Word's markup) or `"custom"` with
 * `revisionPalette` (`"deleted=#AA0000:strike,..."`).
 * `moveComments` (optional, default `false`) lists the comments after the
 * last page instead of in balloons beside the text; `changedOnly`
 * (optional, default `false`) keeps only the pages a tracked change
 * touches, the pages a comment is anchored on and each section's first
 * page on which a revised header or footer paints (else the first page).
 * @param {Uint8Array | number[]} docx
 * @param {boolean | null} [compress]
 * @param {string | null} [revisions]
 * @param {string | null} [revision_palette]
 * @param {boolean | null} [move_comments]
 * @param {boolean | null} [changed_only]
 * @returns {Uint8Array}
 */
export function docxToPdf(docx, compress, revisions, revision_palette, move_comments, changed_only) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        let ptr0, len0, ptr1, len1, ptr2, len2;
        try {
            ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export4);
            len0 = WASM_VECTOR_LEN;
            ptr1 = isLikeNone(revisions) ? 0 : passStringToWasm0(revisions, wasm.__wbindgen_export4, wasm.__wbindgen_export5);
            len1 = WASM_VECTOR_LEN;
            ptr2 = isLikeNone(revision_palette) ? 0 : passStringToWasm0(revision_palette, wasm.__wbindgen_export4, wasm.__wbindgen_export5);
            len2 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr2) { wasm.__wbindgen_export(ptr2, len2 * 1, 1); }
            if (ptr1) { wasm.__wbindgen_export(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_export(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.docxToPdf(retptr, ptr0, len0, isLikeNone(compress) ? 0xFFFFFF : compress ? 1 : 0, ptr1, len1, ptr2, len2, isLikeNone(move_comments) ? 0xFFFFFF : move_comments ? 1 : 0, isLikeNone(changed_only) ? 0xFFFFFF : changed_only ? 1 : 0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
        if (r3) {
            throw takeFromExternrefTable0(r2);
        }
        var v4 = getArrayU8FromWasm0(r0, r1).slice();
        wasm.__wbindgen_export(r0, r1 * 1, 1);
        return v4;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}

/**
 * One-shot init: panic hook → `console.error`. Safe to call multiple times.
 */
export function initPanicHook() {
    wasm.initPanicHook();
}

/**
 * Number of pages in a PDF (cheap object scan; `0` if the bytes are not a
 * readable PDF).
 *
 * Mirrors `jubarte::convert::pdf_page_count`.
 * @param {Uint8Array | number[]} pdf
 * @returns {number}
 */
export function pdfPageCount(pdf) {
    const ptr0 = passArray8ToWasm0(pdf, wasm.__wbindgen_export4);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.pdfPageCount(ptr0, len0);
    return ret >>> 0;
}

/**
 * An edited document's PDF, compressed, its changes painted as
 * `revisions` (`conventional`, `word` or `custom` with
 * `revisionPalette`), with the page facts `edit --pdf` reports. Full build
 * only.
 *
 * Mirrors `jubarte::convert::render`.
 * @param {Uint8Array | number[]} docx
 * @param {string | null} [revisions]
 * @param {string | null} [revision_palette]
 * @returns {RenderOutput}
 */
export function renderPdf(docx, revisions, revision_palette) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16) >>> 0;
        let ptr0, len0, ptr1, len1, ptr2, len2;
        try {
            ptr0 = passArray8ToWasm0(docx, wasm.__wbindgen_export4);
            len0 = WASM_VECTOR_LEN;
            ptr1 = isLikeNone(revisions) ? 0 : passStringToWasm0(revisions, wasm.__wbindgen_export4, wasm.__wbindgen_export5);
            len1 = WASM_VECTOR_LEN;
            ptr2 = isLikeNone(revision_palette) ? 0 : passStringToWasm0(revision_palette, wasm.__wbindgen_export4, wasm.__wbindgen_export5);
            len2 = WASM_VECTOR_LEN;
        } catch (e) {
            if (ptr2) { wasm.__wbindgen_export(ptr2, len2 * 1, 1); }
            if (ptr1) { wasm.__wbindgen_export(ptr1, len1 * 1, 1); }
            if (ptr0) { wasm.__wbindgen_export(ptr0, len0 * 1, 1); }
            throw e;
        }
        wasm.renderPdf(retptr, ptr0, len0, ptr1, len1, ptr2, len2);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeFromExternrefTable0(r1);
        }
        return RenderOutput.__wrap(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_throw_4c88d8bf264fd984: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_error_757e9472f8410341: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_getRandomValues_8b6f1f663e6f188d: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_getUTCDate_663b90c995fdf8b7: function(arg0) {
            const ret = arg0.getUTCDate();
            return ret;
        },
        __wbg_getUTCFullYear_a63feba390b3baf5: function(arg0) {
            const ret = arg0.getUTCFullYear();
            return ret;
        },
        __wbg_getUTCHours_3e5c258e11ac0d90: function(arg0) {
            const ret = arg0.getUTCHours();
            return ret;
        },
        __wbg_getUTCMinutes_e309e461528477de: function(arg0) {
            const ret = arg0.getUTCMinutes();
            return ret;
        },
        __wbg_getUTCMonth_135d8f5aa938eadd: function(arg0) {
            const ret = arg0.getUTCMonth();
            return ret;
        },
        __wbg_getUTCSeconds_8eb6d834a30e92cf: function(arg0) {
            const ret = arg0.getUTCSeconds();
            return ret;
        },
        __wbg_new_0_36d738a0c8cfac43: function() {
            const ret = new Date();
            return ret;
        },
        __wbg_new_bb74001155cb8ca8: function() {
            const ret = new Error();
            return ret;
        },
        __wbg_stack_575c4e0918581b9c: function(arg0, arg1) {
            const ret = arg1.stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export4, wasm.__wbindgen_export5);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 1, len1, true);
            getDataViewMemory0().setInt32((arg0 >>> 0) + 4 * 0, ptr1, true);
        },
        __wbindgen_generic_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            table.set(0, undefined);
            table.set(1024, undefined);
            table.set(1025, null);
            table.set(1026, true);
            table.set(1027, false);
        },
    };
    return {
        __proto__: null,
        "./jubarte_wasm_bg.js": import0,
    };
}

const RenderOutputFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_renderoutput_free(ptr, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__wbindgen_export3();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_export2(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
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

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__wbindgen_export6(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(bytes) {
    numBytesDecoded += bytes.length;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true });
        cachedTextDecoder.decode();
        numBytesDecoded = bytes.length;
    }
    return cachedTextDecoder.decode(bytes);
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

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (!module.ok) {
            throw new Error(`failed to fetch Wasm: ${module.status} ${module.statusText} fetching '${module.url}'`);
        }

        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = expectedResponseType(module.type);

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

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

let __wbg_init_promise;
async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;
    if (__wbg_init_promise === undefined) {
        __wbg_init_promise = __wbg_init_once(module_or_path);
        __wbg_init_promise.catch(() => { __wbg_init_promise = undefined; });
    }
    return __wbg_init_promise;
}

async function __wbg_init_once(module_or_path) {

    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('jubarte_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
