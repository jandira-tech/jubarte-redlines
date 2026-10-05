# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

The Mac app is a Tauri 2 window whose UI is HTML/CSS/JS (`src/`). It follows
macOS conventions (Finder Quick Actions, Open With, the Dock, the App Store),
but its design language is the web one it shares with jubarte.pro.

## Users

**Primary: lawyers and legal staff.** They hold two versions of a Word
document, usually a contract draft and the counterparty's turn of it. They
need a redline that Microsoft Word opens cleanly as genuine tracked changes,
to review, accept, reject and send back. They reach Jubarte through the Mac
app and through jubarte.pro's in-browser Demo.

**Secondary: developers.** These are legal-tech and document-pipeline
engineers who embed the same engine as a CLI, a Rust crate, Python wheels or
WebAssembly packages. jubarte.pro's Download and Benchmark pages serve them.

## Product Purpose

Jubarte does three things with a Word document: it compares two `.docx`
files into native Word revisions, it resolves those revisions, and it renders
documents to PDF. All of it runs without Microsoft Word. The goal is output
that Word treats as its own: a redline that opens with no warning or repair
prompt, and a PDF that matches Word's own export.

The surfaces covered by this record:
- **Mac app** (this repository, proprietary). Drop two documents and get a
  redline, or convert a document to PDF, from the window or from Finder.
- **jubarte.pro** (`jubarte-site/`). The public site: Home, Demo (the engine
  as wasm in the browser), Benchmark and Cases, App, Download, Contact,
  Privacy and Terms. Live is "coming soon".
- **The old redline tool's hosts** (`redlines-site/`): redlines.free,
  www.redlines.free, redlines.jubarte.pro and redlines.arthur.law. The
  standalone web redliner is retired; each host redirects to jubarte.pro/demo
  (its policies to jubarte.pro's).

The engine itself is `jubarte-redlines` (AGPL-3.0, Rust path `jubarte::`),
developed in the parent checkout.

## Positioning

**Measured Word fidelity.** Jubarte is scored publicly against Microsoft
Word's own output in `neurotic_docx_bench`:
- Word's PDF export is the reference for conversion.
- Word's own compare is the reference for redlines.
- Redlines are opened in Word before they are scored.
- A failure counts as zero.
- Jubarte is marked author-affiliated (†) and held to the same rules as
  every other tool.

Two supporting claims follow: documents never leave the machine (the app and
the browser Demo process files locally), and the engine is open (AGPL) and
embeddable.

## Operating Context

- Microsoft Word is the oracle. Every reference PDF and reference redline
  comes from Word, never from LibreOffice. "Word valid" means Word opens the
  file with no warning, error or offer to repair it.
- A legal workflow: the drafts arrive by email or a document system, and the
  redline goes back out as a `.docx` the counterparty opens in Word.
- The Mac app runs in the App Sandbox. It writes into its own container and
  exports with "Save a copy". "Revisions by" defaults to the modified
  document's author.
- The Demo runs jubarte-wasm in a Web Worker in the visitor's tab; the
  Worker never receives a document.

## Capabilities and Constraints

- **Engine jobs:**
  - compare (a redline with insertions, deletions, moves and formatting
    changes);
  - resolve (accept or reject revisions by ID);
  - edit (validated JSON edit plans);
  - render (PDF or PNG, with the revision styles conventional, word and
    custom).
- **Pricing truth:**
  - The Mac App Store listing is id6790926615. Today 0.7.0 is a one-time
    $99.99 purchase.
  - The next release is a free download with five free uses (a redline or a
    PDF each, from one shared pool), then $99.99 a year.
  - The site, the CLI, the crate, the wheels and the wasm are free.
- **Benchmark figures:** they come only from `jubarte-site/site/data/bench.ts`,
  which is copied from neurotic_docx_bench's RESULTS.md and checked by
  `pnpm bench:check`. Never round or restate them by hand.
- **Case fixtures:** they live on Cloudflare assets and on Hugging Face
  (`arthrod/neurotic_docx_bench`, `site/`), pinned by `jubarte-site/fixtures.lock`.
  A link must point to the same bytes on Hugging Face.
- **jubarte.pro limits:** page requests are limited to 10 per minute per IP.
  Live mode is not built and stays "coming soon".
- **Undecided:**
  - the direct-download DMG (marked "coming soon");
  - routing App Store Server Notifications to verify-worker, which would
    change the privacy policy's wording.

## Brand Commitments

- **Names:**
  - Jubarte is the product.
  - `jubarte-redlines` is the engine's crate and repository.
  - JUBARTE.DOCX is the site's wordmark.
  - Jandira Technologies, LLC is the company (New York · São Paulo).
- **Voice: truthful and specific.** Show the scoreboard instead of making a
  claim ("Not a claim. A scoreboard."). Say honestly when Jubarte is not the
  right answer. Pricing and Terms state exactly what the store does.
- **The whale** (jubarte is Portuguese for humpback) is the mark, in both the
  app and the site.
- **Binding palette:** Arthur chose it on 2026-10-02. Graphite (light) and
  Night (dark) are OKLCH tokens, with maximum text contrast and the brand
  colour as the primary button. The site follows the browser's theme until
  the footer switch pins one; the app follows macOS. Document pages stay
  white paper in both modes.

## Evidence on Hand

- **Benchmark results:** `jubarte-site/site/data/bench.ts`, generated
  2026-10-01 from neurotic_docx_bench 0.7. It covers 6,427 documents for
  conversion and 3,502 pairs for redlines.
- **Published cases:** each one has Word's page, jubarte's and LibreOffice's,
  the scores and the source files. Browse them at `/cases`; the files are on
  Hugging Face.
- **The benchmark viewer:**
  https://jandira-tech.github.io/neurotic_docx_bench/
- **The working product:** the in-browser Demo, the Mac app on the App Store,
  and the engine releases on GitHub with SHA256SUMS.
- **Absent, and never to be fabricated:** customer names, testimonials,
  press, usage statistics, and any performance or accuracy claim that
  bench.ts does not hold.

## Product Principles

1. **Word is the judge.** Fidelity to Microsoft Word outranks speed,
   novelty and taste. A result Word would reject is a failure, not a
   variant.
2. **Show the evidence.** Every claim links to scored, reproducible results;
   failures are shown as failures.
3. **The document is the user's.** Process locally, never upload, and keep
   the user's output untouched by the interface's styling.
4. **Lawyers first, developers through the same door.** Lead with the
   redline a lawyer sends. The engine surfaces stay one step away.
5. **Truthful commerce.** Prices, free uses and terms read exactly as the
   store and the app enforce them.
