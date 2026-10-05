// Just enough ZIP to read a .docx package in the browser: the central
// directory, stored and deflated entries (DecompressionStream "deflate-raw").
// Also a store-only writer (the tests, the demo documents, PNG bundles) and a
// part rewriter that copies every untouched part still compressed.

const EOCD = 0x06054b50;
const CENTRAL = 0x02014b50;
const LOCAL = 0x04034b50;

/**
 * @typedef {{ name: string, nameBytes: Uint8Array, method: number, flags: number,
 *   time: number, date: number, crc: number, offset: number, size: number, csize: number }} Entry
 */

/** @param {Uint8Array} bytes @returns {Map<string, Entry>} */
export function entries(bytes) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  let eocd = -1;
  for (let i = bytes.length - 22; i >= Math.max(0, bytes.length - 22 - 65535); i--) {
    if (view.getUint32(i, true) === EOCD) {
      eocd = i;
      break;
    }
  }
  if (eocd < 0) throw new Error("not a zip file");
  const count = view.getUint16(eocd + 10, true);
  let p = view.getUint32(eocd + 16, true);
  const out = new Map();
  const utf8 = new TextDecoder();
  for (let n = 0; n < count; n++) {
    if (view.getUint32(p, true) !== CENTRAL) throw new Error("bad central directory");
    const nameLen = view.getUint16(p + 28, true);
    const nameBytes = bytes.subarray(p + 46, p + 46 + nameLen);
    const name = utf8.decode(nameBytes);
    out.set(name, {
      name,
      nameBytes,
      flags: view.getUint16(p + 8, true),
      method: view.getUint16(p + 10, true),
      time: view.getUint16(p + 12, true),
      date: view.getUint16(p + 14, true),
      crc: view.getUint32(p + 16, true),
      csize: view.getUint32(p + 20, true),
      size: view.getUint32(p + 24, true),
      offset: view.getUint32(p + 42, true),
    });
    const extraLen = view.getUint16(p + 30, true);
    const commentLen = view.getUint16(p + 32, true);
    p += 46 + nameLen + extraLen + commentLen;
  }
  return out;
}

/** An entry's bytes as stored in the package (compressed when it is). */
function rawData(/** @type {Uint8Array} */ bytes, /** @type {Entry} */ entry) {
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  if (view.getUint32(entry.offset, true) !== LOCAL) throw new Error("bad local header");
  const start =
    entry.offset +
    30 +
    view.getUint16(entry.offset + 26, true) +
    view.getUint16(entry.offset + 28, true);
  return bytes.subarray(start, start + entry.csize);
}

/** @param {Uint8Array} data @param {CompressionFormat} format @param {"in" | "out"} way */
async function codec(data, format, way) {
  const transform = way === "in" ? new DecompressionStream(format) : new CompressionStream(format);
  const stream = new Blob([/** @type {Uint8Array<ArrayBuffer>} */ (data)])
    .stream()
    .pipeThrough(transform);
  return new Uint8Array(await new Response(stream).arrayBuffer());
}

/** @param {Uint8Array} bytes @param {Entry} entry */
async function inflate(bytes, entry) {
  const data = rawData(bytes, entry);
  if (entry.method === 0) return data.slice();
  if (entry.method !== 8) throw new Error(`unsupported compression ${entry.method}`);
  return codec(data, "deflate-raw", "in");
}

/** The bytes of one entry, or null when the package has no such part. */
export async function readEntry(/** @type {Uint8Array} */ bytes, /** @type {string} */ name) {
  const entry = entries(bytes).get(name);
  return entry ? inflate(bytes, entry) : null;
}

/** One entry as UTF-8 text, or null. */
export async function readText(/** @type {Uint8Array} */ bytes, /** @type {string} */ name) {
  const data = await readEntry(bytes, name);
  return data ? new TextDecoder().decode(data) : null;
}

const CRC_TABLE = (() => {
  const t = new Uint32Array(256);
  for (let n = 0; n < 256; n++) {
    let c = n;
    for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
    t[n] = c >>> 0;
  }
  return t;
})();

/** @param {Uint8Array} data */
export function crc32(data) {
  let c = 0xffffffff;
  for (let i = 0; i < data.length; i++) c = CRC_TABLE[(c ^ data[i]) & 0xff] ^ (c >>> 8);
  return (c ^ 0xffffffff) >>> 0;
}

/**
 * @typedef {{ nameBytes: Uint8Array, method: number, flags: number, time: number,
 *   date: number, crc: number, size: number, data: Uint8Array }} Packed
 */

/** A zip of ready entries, in order: local headers with sizes, no data descriptors. */
function pack(/** @type {Packed[]} */ items) {
  const locals = [];
  const centrals = [];
  let offset = 0;
  for (const it of items) {
    // Bit 3 (sizes in a trailing data descriptor) is cleared: the sizes are
    // written in the local header below, and no descriptor follows the data.
    const flags = it.flags & ~0x0008;
    const local = new Uint8Array(30 + it.nameBytes.length);
    const lv = new DataView(local.buffer);
    lv.setUint32(0, LOCAL, true);
    lv.setUint16(4, 20, true);
    lv.setUint16(6, flags, true);
    lv.setUint16(8, it.method, true);
    lv.setUint16(10, it.time, true);
    lv.setUint16(12, it.date, true);
    lv.setUint32(14, it.crc, true);
    lv.setUint32(18, it.data.length, true);
    lv.setUint32(22, it.size, true);
    lv.setUint16(26, it.nameBytes.length, true);
    local.set(it.nameBytes, 30);
    const central = new Uint8Array(46 + it.nameBytes.length);
    const cv = new DataView(central.buffer);
    cv.setUint32(0, CENTRAL, true);
    cv.setUint16(4, 20, true);
    cv.setUint16(6, 20, true);
    cv.setUint16(8, flags, true);
    cv.setUint16(10, it.method, true);
    cv.setUint16(12, it.time, true);
    cv.setUint16(14, it.date, true);
    cv.setUint32(16, it.crc, true);
    cv.setUint32(20, it.data.length, true);
    cv.setUint32(24, it.size, true);
    cv.setUint16(28, it.nameBytes.length, true);
    cv.setUint32(42, offset, true);
    central.set(it.nameBytes, 46);
    locals.push(local, it.data);
    centrals.push(central);
    offset += local.length + it.data.length;
  }
  const cdSize = centrals.reduce((n, c) => n + c.length, 0);
  const end = new Uint8Array(22);
  const ev = new DataView(end.buffer);
  ev.setUint32(0, EOCD, true);
  ev.setUint16(8, items.length, true);
  ev.setUint16(10, items.length, true);
  ev.setUint32(12, cdSize, true);
  ev.setUint32(16, offset, true);
  const out = new Uint8Array(offset + cdSize + 22);
  let p = 0;
  for (const chunk of [...locals, ...centrals, end]) {
    out.set(chunk, p);
    p += chunk.length;
  }
  return out;
}

/**
 * A stored (uncompressed) zip of the given parts, in order. Word opens stored
 * packages; `[Content_Types].xml` should come first.
 * @param {[string, Uint8Array | string][]} parts
 */
export function writeZip(parts) {
  const enc = new TextEncoder();
  return pack(
    parts.map(([name, content]) => {
      const data = typeof content === "string" ? enc.encode(content) : content;
      // 1980-01-01 00:00, the zip epoch: deterministic output.
      return {
        nameBytes: enc.encode(name),
        method: 0,
        flags: 0x0800,
        time: 0,
        date: (0 << 9) | (1 << 5) | 1,
        crc: crc32(data),
        size: data.length,
        data,
      };
    }),
  );
}

/**
 * The package with some of its text parts rewritten, or the same bytes when
 * nothing changed. `edit(name, text)` sees each part `wants(name)` selects and
 * returns its new text, or null to keep it. Every other part is copied byte
 * for byte, still compressed; an edited part is deflated again.
 * @param {Uint8Array} bytes
 * @param {(name: string) => boolean} wants
 * @param {(name: string, text: string) => string | null} edit
 */
export async function rewriteParts(bytes, wants, edit) {
  const utf8 = new TextDecoder();
  const enc = new TextEncoder();
  /** @type {Packed[]} */
  const items = [];
  let changed = false;
  for (const e of entries(bytes).values()) {
    /** @type {Packed} */
    const item = { ...e, data: rawData(bytes, e) };
    if (wants(e.name)) {
      const text = utf8.decode(await inflate(bytes, e));
      const next = edit(e.name, text);
      if (next !== null && next !== text) {
        const plain = enc.encode(next);
        Object.assign(item, {
          method: 8,
          crc: crc32(plain),
          size: plain.length,
          data: await codec(plain, "deflate-raw", "out"),
        });
        changed = true;
      }
    }
    items.push(item);
  }
  return changed ? pack(items) : bytes;
}
