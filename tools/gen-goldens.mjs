// Golden generator — runs the TypeScript docxodus oracle to produce canonical
// redline outputs that the Rust port is tested against.
//
// Run from the jubarte-first dir so `tsx` + deps (jszip, …) resolve:
//   cd jubarte-first && node --import tsx ../ooxmlsdk/crates/jubarte/tools/gen-goldens.mjs
//
// Uses WmlComparer.Compare directly (NOT DocumentComparer.CompareDocumentsWithOptions)
// because the latter stamps DateTimeForRevisions = new Date().toISOString() (non-deterministic).
// We pin author + date so goldens are byte-stable.
import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
// Source-relative imports; `.js` resolves to the `.ts` source under tsx.
import {
  wireDocxodusNodeAdapter,
  wireWmlComparerNodeAdapter,
} from "../../../../jubarte-first/src/docxodus/lib/ooxml-package-jszip.js";
import { WmlComparer, WmlComparerSettings } from "../../../../jubarte-first/src/docxodus/WmlComparer.js";
import { WmlDocument } from "../../../../jubarte-first/src/docxodus/WmlDocument.js";

// Wire every injectable boundary the comparer reaches (both required).
wireDocxodusNodeAdapter();
wireWmlComparerNodeAdapter();

const here = dirname(fileURLToPath(import.meta.url));
const crate = resolve(here, "..");                 // jubarte/
const FIX = resolve(here, "../../../../jubarte-first/src/docxodus/tests/fixtures");
const outDir = resolve(crate, "tests/goldens");

const PINNED_AUTHOR = "Test Author";
const PINNED_DATE = "2020-01-01T00:00:00Z";

const pairs = [
  ["redline", `${FIX}/redline/original.docx`, `${FIX}/redline/modified.docx`],
  ["inpi", `${FIX}/redline-inpi/original-new.docx`, `${FIX}/redline-inpi/modified-new.docx`],
  ["inpi2", `${FIX}/redline-inpi/original-new-2.docx`, `${FIX}/redline-inpi/modified-new-2.docx`],
];

mkdirSync(outDir, { recursive: true });
for (const [name, o, m] of pairs) {
  const original = new WmlDocument(new Uint8Array(readFileSync(o)));
  original.FileName = "original.docx";
  const modified = new WmlDocument(new Uint8Array(readFileSync(m)));
  modified.FileName = "modified.docx";

  const settings = new WmlComparerSettings();
  settings.AuthorForRevisions = PINNED_AUTHOR;
  settings.DateTimeForRevisions = PINNED_DATE;
  settings.DetailThreshold = 0.15;
  settings.CaseInsensitive = false;

  const result = WmlComparer.Compare(original, modified, settings);
  const bytes = result.DocumentByteArray;
  writeFileSync(`${outDir}/${name}.redline.docx`, bytes);
  console.log(`wrote ${name}.redline.docx (${bytes.length} bytes)`);
}
