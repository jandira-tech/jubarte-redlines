// The sizes of the jubarte-wasm builds the site ships, measured from the
// installed package when the build (or a test) loads the pages, so the copy
// cannot drift from the files. "Over the wire" is brotli at quality 4, which
// tracks what Cloudflare serves (2026-10-01: 1.31 MB and 5.23 MB served,
// against 1.29 MB and 4.97 MB here).

import { readFileSync } from "node:fs";
import { brotliCompressSync, constants } from "node:zlib";

const PACKAGE = new URL("../node_modules/jubarte-wasm/", import.meta.url);

const mb = (bytes: number) => `${(bytes / 1e6).toFixed(1)} MB`;

/** A build's size on disk and over the wire, as the page prints them. */
export type WasmSize = { raw: string; wire: string };

function measure(dir: string): WasmSize {
  const bytes = readFileSync(new URL(`${dir}/jubarte_wasm_bg.wasm`, PACKAGE));
  const wire = brotliCompressSync(bytes, {
    params: { [constants.BROTLI_PARAM_QUALITY]: 4 },
  }).length;
  return { raw: mb(bytes.length), wire: mb(wire) };
}

/** `slim` compares; `full` also renders PDF. */
export const WASM_SIZE: { slim: WasmSize; full: WasmSize } = {
  slim: measure("web-slim"),
  full: measure("web"),
};
