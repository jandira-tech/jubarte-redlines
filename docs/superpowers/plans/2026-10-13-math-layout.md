<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

# Office Math layout, step 1

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Lay out OMML bar fractions (`m:f`), radicals (`m:rad`), and stretchy delimiters (`m:d`) as a 2D box, using the OpenType MATH table of the font actually in use, and paint them as real glyph runs plus rules.

**Architecture:** Two new modules under `src/convert/`. `math_font` reads MATH constants and vertical glyph variants from a `ttf-parser` 0.25 face (or a documented TeX fallback when the face has no MATH table). `math_layout` walks the OMML subtree into a box (width, ascent, descent, positioned glyphs, rules) and the PDF painter draws that box. No LaTeX. The markdown tables in `src/markdown/math.rs` stay the markdown path.

**Tech stack:** Rust, `ttf-parser` 0.25 (already a dependency, default features include the MATH table), the existing PDF `Op::Text` / `Op::FillRect` painter. Layout code adapted from markdown2pdf `src/lib/render/math/{font,layout}.rs` (MIT, Copyright 2023 Ismael Shakverdiev). Their engine draws outlines. Ours emits glyph ids so the PDF stays searchable and the subsetter keeps the variant glyphs.

---

## Where this disagrees with the sketch

1. **Do not port their whole TeX engine in this step.** `layout.rs` also does atom classes, mu spacing, limits, accents, and arrays. Turning that on would move every equation, including superscripts, n-ary operators, and noBar stacks that this step is supposed to leave alone. Port the MATH reader, vertical stretch (variants, then assembly), and the three constructors (fraction, radical, delimiter).

2. **A superscript inside a fraction has to be a box, or the bar is wrong.** Top-level `m:sSup` / `m:sSub` / `m:nary` stay on today's `VertAlign` path. When one of those sits *inside* an `m:f`, `m:rad`, or `m:d`, the layout places it with today's ratios (script size 0.65, superscript +0.35 em, subscript −0.15 em of the enclosing size) so the parent can measure it. Full MATH script constants are the later scripts step. Same for a noBar stack, `lin`, or `skw` that is nested inside one of the three: reproduce today's geometry as a box. At the top level those three fraction types stay on the collector they have now.

3. **One hard-coded ratio stays even when the font has a MATH table.** MATH does not say how tall a delimiter must be. The size rule is TeX's: at least twice the larger of (content ascent − axis) and (content descent + axis), and at least 0.9 em, so a parenthesis around one letter is taller than the letter. Fraction side padding is the font's own rule thickness, not TeX's 0.12 em. Every other spacing number comes from the MATH table, scaled by `value * font_size / units_per_em`.

4. **Cambria Math is not the Cambria text face.** `Fonts::mapped_face` folds any name containing "cambria" onto Cambria Regular, which has no MATH table. `add_installed_faces` already loads "Cambria Math" as its own face when Word is installed: `catalogue_paints_family("Cambria Math")` is false, because the catalogue pdf name `Cambria` does not start with `cambriamath`. Face index 1 of `/Applications/Microsoft Word.app/Contents/Resources/DFonts/Cambria.ttc` is Cambria Math (glyf + MATH). Index 0 is Cambria. Production keeps using that loader. A test that opens the collection skips when the file is absent. The font is never committed.

## What stays as it is

- Top-level `m:sSup`, `m:sSub`, `m:sSubSup`, `m:sPre`, `m:nary`, `m:func`, `m:limLow`, `m:limUpp`, `m:acc`, `m:groupChr`, `m:m`, `m:eqArr`, `m:borderBox`, `m:bar`.
- Top-level `m:f` with `m:type` `noBar`, `lin`, or `skw`. Absent `m:type` is `bar` (Office default) and takes the new layout.
- `m:d` with `m:grow` off still becomes a box, centered on the axis, but the glyph is not stretched.
- An `m:d` inside a superscript is still raised with its contents. The shift is baked into the box (today's +0.35 em, size × 0.65) and the run stays on the baseline, so the painter does not shift it twice.
- Searchable text stays the characters in reading order. `omml_d_draws_its_delimiters` still joins to `(𝑥)`. The run is one box, not three runs.

## File map

| File | Responsibility |
| --- | --- |
| `src/convert/math_font.rs` | MATH constants, glyph metrics, `stretch_vertical`. Owns font bytes. |
| `src/convert/math_layout.rs` | OMML node → `MathBox`. |
| `src/convert/mod.rs` | Build the font for the conversion, emit a `TextRun` carrying the box, paint it, measure it. |
| `tests/fixtures/fonts/STIXTwoMath.otf` | OFL math font for geometry tests. Not used in production PDFs (it is CFF; the PDF embedder keeps glyf faces only). |
| `tests/fixtures/fonts/STIXTwoMath-OFL.txt` | Upstream OFL text, unmodified. |
| `REUSE.toml` | Annotation for those two files. |

`MathBox` (in `math_layout`, stored on `TextRun`):

```rust
pub(crate) struct MathGlyph {
    pub text: String, // searchable; empty on assembly extenders
    pub gid: u16,     // 0 when the face has no glyph
    pub x: f32,       // from the box origin, points
    pub y: f32,       // baseline offset, +up, points
    pub size: f32,    // point size this glyph is drawn at
}
pub(crate) struct MathRule {
    pub x: f32,
    pub y_top: f32, // top edge, +up from the box baseline
    pub w: f32,
    pub thickness: f32,
}
pub(crate) struct MathBox {
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
    pub glyphs: Vec<MathGlyph>,
    pub rules: Vec<MathRule>,
}
```

Coordinates match the painter: `y` grows up, the run baseline is the line baseline. A rule is `Op::FillRect` at `baseline + y_top - thickness`.

## Fallback constants

Used only when `face.tables().math` is missing. Stored in font units (`fraction * upem`). Comment in `math_font.rs` cites TeXbook Appendix G / plain TeX at 10pt (cmsy axis 2.5pt, cmex rule 0.4pt).

| Field | em |
| --- | --- |
| `script_percent` | 0.70 |
| `script_script_percent` | 0.50 |
| `axis_height` | 0.25 |
| `fraction_rule_thickness` | 0.04 |
| `fraction_num_display_shift_up` | 0.676 |
| `fraction_num_shift_up` | 0.394 |
| `fraction_denom_display_shift_down` | 0.686 |
| `fraction_denom_shift_down` | 0.345 |
| `fraction_num_display_gap_min` | 0.30 |
| `fraction_num_gap_min` | 0.10 |
| `fraction_denom_display_gap_min` | 0.30 |
| `fraction_denom_gap_min` | 0.10 |
| `radical_vertical_gap` | 0.05 |
| `radical_display_vertical_gap` | 0.14 |
| `radical_rule_thickness` | 0.04 |
| `radical_extra_ascender` | 0.04 |
| `radical_kern_before_degree` | 0.05 |
| `radical_kern_after_degree` | −0.10 |
| `radical_degree_bottom_raise_percent` | 0.60 |
| stack shifts (noBar nested only) | today's 0.45 / 0.40 of the base size, not these |

When the face has no vertical variants, a radical or delimiter that is too short is drawn at a larger point size so its height + depth meets the target. That scale is a fallback, not a TeX ratio.

## Layout rules (the three constructors)

Display style when the `m:oMath` has an `m:oMathPara` ancestor. Otherwise text style. Numerator and denominator of a display fraction stay at the base size (TeX display → text, and text size is the base size). An inline fraction's numerator and denominator use `script_percent`.

**Bar fraction.** Numerator and denominator centered. The rule is centered on `axis_height`, thickness `fraction_rule_thickness`. Shifts are the display or text pair (`fraction_numerator*_shift_up`, `fraction_denominator*_shift_down`), raised until the gap constants clear. The rule is wider than the wider piece by one rule-thickness on each side. `noBar` at the top level does not come here.

**Radical.** Body in the current style. Gap is `radical_display_vertical_gap` or `radical_vertical_gap`, rule is `radical_rule_thickness`, extra ascent is `radical_extra_ascender`. Target height is body ascent + descent + gap + rule. `stretch_vertical` on U+221A. The body sits to the right of the sign. The rule's bottom is at least `body_top + gap`. `m:deg` is laid out at script-script percent and raised by `radical_degree_bottom_raise_percent` of the rule top, with `radical_kern_before_degree` and `radical_kern_after_degree`, unless `m:degHide` is on. The first radical glyph's text is U+221A. This is the bug where `$\sqrt[3]{x}$` paints `3 x`.

**Delimiter.** `begChr` / `endChr` / `sepChr` default to `(`, `)`, `|`. An empty `m:val` draws nothing. Each fence stretches to the target above. Fences are centered on the axis. `m:e` items are a row, `sepChr` between them. `m:grow` off uses the base glyph, still centered.

**Row.** Children abut. No mu spacing in this step. `m:r` / `w:r` text uses `math_italic` unless `m:sty` is `p` or `b` or `m:nor` is set, same as `math_text_style` today. A character the face cannot draw stays ASCII and is flagged so the painter uses the italic face and the PDF text stays the math letter (`math_ascii`).

**Nesting.** `layout_node` recurses. Depth cap 64; past that the node contributes nothing.

## Painter and line box

`TextRun` gains `math: Option<MathBox>`.

- `run_width_pt` returns `math.width` for the run's own text.
- `wrap_runs_segment` treats a math run as one unsplittable unit of that width. The wrap loop measures ordinary runs with `width_pt_kern`, which would split a fraction into the letters of both the numerator and the denominator.
- `line_face_metrics` takes the box ascent and descent, so the line grows and the baseline leaves room above and below.
- `paint_run` draws each glyph with `Op::text` at `(x + glyph.x, baseline + glyph.y)` using the glyph's own size and gid, then each rule. The face is `fonts.resolve("Cambria Math", false, false)`, the same face the constants came from. Assembly extenders carry an empty string so the text layer has one radical sign. The subsetter already keeps every gid in an `Op::Text`.

The conversion builds one `MathFont` from that face's bytes and installs it in a thread-local for the conversion, the same way `MathAlphanumerics` is installed. Tests that do not set it get the fallback (relations still hold; gids may be 0). Geometry tests call `layout` with STIX directly and do not use the thread-local.

## Fixtures (Word PDFs already exist)

Rendered with `jubarte convert --revisions word`. Side by side via `sbs.py`.

| Fixture | Why |
| --- | --- |
| `tests/fixtures/math/quadratic-in-table.docx` | Display bar fraction whose numerator is a radical whose body has a superscript. |
| `tests/fixtures/math/word2007-equations.docx` | Parentheses around real content, and bar fractions. noBar binomials in it stay on the old path. |
| `tests/fixtures/math/inline-symbol-runs.docx` | Inline radicals (the missing U+221A) and a fraction in a superscript. |

`brackets-and-braces.docx` is only empty fences, so it does not show stretch. `mismatched-delimiters.docx` puts the fraction inside an `m:eqArr`, which this step does not stack, so its brace will still be short.

## Later steps (not this task)

- **Scripts.** Top-level `m:sSub` / `m:sSup` / `m:sSubSup` / `m:sPre` on MATH shifts (`superscript_shift_up`, `subscript_shift_down`, gap constants, italic correction). Replace the 0.65 / 0.35 / 0.15 ratios inside boxes.
- **N-ary and limits.** `m:nary`, `m:limLow`, `m:limUpp`: grow the operator to `display_operator_min_height`, limits above and below in display style, sub/sup beside it otherwise.
- **Matrices.** `m:m` column alignment (`m:mcJc`) and row baselines.
- **Equation arrays.** `m:eqArr` rows, `m:rSp`, alignment points. Unblocks `mismatched-delimiters.docx`.
- **Accents.** `m:acc` / `m:groupChr` with `top_accent` and horizontal variants (`widen`).
- **Spacing.** Atom classes and mu skips, so `x + y` matches Word's gaps.

---

### Task 1: Vendor STIX Two Math

**Files:**
- Create: `tests/fixtures/fonts/STIXTwoMath.otf` (copy, do not edit)
- Create: `tests/fixtures/fonts/STIXTwoMath-OFL.txt` (copy of the upstream OFL)
- Modify: `REUSE.toml`

- [ ] Copy the font and `OFL.txt` from the markdown2pdf checkout. Do not rename the font (OFL reserved name "STIX Two").
- [ ] Add two `REUSE.toml` annotations, `precedence = "override"`, license `OFL-1.1`, copyright the STI Pub Companies (2001-2021) and Tiro Typeworks (2013-2020) for the font.
- [ ] Run `uv tool run --from 'reuse[charset-normalizer]' reuse lint`.
- [ ] Commit `chore(convert): vendor STIX Two Math for math layout tests`.

### Task 2: MATH font reader (red, then green)

**Files:**
- Create: `src/convert/math_font.rs`
- Modify: `src/convert/mod.rs` (`mod math_font;`)

- [ ] **Red.** Tests in `math_font.rs`:
  - STIX `axis_height` is between 0.15 and 0.40 em, `fraction_rule_thickness` > 0, `radical_vertical_gap` > 0, `has_math_table` is true.
  - U+221A stretched to three times its own height is either a taller variant or an assembly of at least two parts.
  - `(` stretched the same way grows.
  - `MathFont::fallback(1000.0)` has `has_math_table` false, axis 250, rule thickness 40, and `stretch_vertical` returns the base gid.
  - If `Cambria.ttc` exists, face index 1 has a MATH table and family name Cambria Math. If the file is missing, the test returns. Never fail on a machine without Word.

  Run `cargo nextest run --all-features --lib -E 'test(math_font)'` and watch the STIX assertions fail because the reader returns zeros.

- [ ] **Green.** Port `MathConstants` and `stretch_vertical` from markdown2pdf `font.rs`. SPDX copyright line for Ismael Shakverdiev next to ours. `from_bytes` parses face 0. A face with no MATH table still returns `Some`, flagged, constants from the fallback table, cmap still live. Re-run the same nextest filter. Pass.

- [ ] Commit the red tests, then the green reader. `cargo fmt` before each commit.

### Task 3: Box layout (red, then green)

**Files:**
- Create: `src/convert/math_layout.rs`
- Modify: `src/convert/mod.rs` (`mod math_layout;`)

- [ ] **Red.** Tests call `layout_omml(font, 12.0, display, omml)` on STIX. `layout_omml` builds a tiny `w:document` and lays out the first `m:oMath` child.

  ```rust
  // bar: rule center on the axis; numerator baseline above the rule;
  // denominator baseline below it; rule at least as wide as either piece.
  let omml = r#"<m:f><m:num><m:r><m:t>a</m:t></m:r></m:num><m:den><m:r><m:t>b</m:t></m:r></m:den></m:f>"#;

  // radical: a glyph whose text is U+221A sits left of the body;
  // the rule's bottom is at least body_top + radical_vertical_gap.
  let omml = r#"<m:rad><m:radPr><m:degHide m:val="1"/></m:radPr><m:deg/><m:e><m:r><m:t>x</m:t></m:r></m:e></m:rad>"#;

  // degree: U+221A is present, and the degree glyph is left of it and above the body baseline.
  let omml = r#"<m:rad><m:deg><m:r><m:t>3</m:t></m:r></m:deg><m:e><m:r><m:t>x</m:t></m:r></m:e></m:rad>"#;

  // delimiters grow: fences around a fraction are taller than fences around "x",
  // and the tall fence covers the fraction's ascent + descent.
  // nesting: fraction inside radical inside brackets has both rules,
  // the radical left of the fraction, the brackets outside both, all finite.
  ```

  A fallback font (no MATH table) still puts the numerator above the bar and the denominator below it.

  Run the filter `test(math_layout)`. Fail because `layout_omml` returns an empty box.

- [ ] **Green.** Implement the three constructors and the row walker. Port the fraction, radical, and delimiter arithmetic from markdown2pdf `layout.rs` (`frac`, `sqrt`, `delimited`, `place_delim`, `assemble_vertical`), with the disagreements above. Same SPDX copyright line. Re-run the filter. Pass.

- [ ] Commit red, then green.

### Task 4: Wire the box into the PDF renderer

**Files:**
- Modify: `src/convert/mod.rs`
- Modify: `src/convert/math_layout.rs` only if the collector needs a smaller entry point

- [ ] **Red.** In the convert test module:
  - `omml_rad_emits_a_radical_sign`: `m:rad` of `x` with `degHide` joins to a string containing U+221A. Fails today (`x` only).
  - `omml_bar_fraction_is_a_box`: `m:f` of `n` over `d` is one run, `math` is `Some`, it has a rule, glyph text order is `n` then `d`, and the following `end` run stays on the baseline. The existing `fractions_with_and_without_bars_preserve_operand_order` bar arm asserts three baseline runs; change that arm to the box, leave the noBar arm as it is.
  - `omml_delimiter_in_a_superscript_is_raised`: the `(n)` box inside `m:sSup` has a glyph `y` above 0. Replace `omml_delimiters_take_the_script_placement`, which looks for three Super runs.
  - `omml_d_draws_its_delimiters` keeps the joined strings. Point the family assertion at the single run that contains `(`.

- [ ] **Green.**
  - In `collect_runs_rec`, a bar `m:f`, any `m:rad`, and any `m:d` emit one `TextRun` from `layout_node`. Other nodes unchanged.
  - Install `MathFont` from the resolved Cambria Math face for the whole conversion, and restore the previous one on drop.
  - `paint_run`, `run_width_pt`, `line_face_metrics`, `wrap_runs_segment` as above.
  - Run the new tests plus `test(omml_)` and `test(fractions_with_and_without)`. Pass.

- [ ] Commit `feat(convert): lay out bar fractions, radicals, and stretchy delimiters`.

### Task 5: Verify

- [ ] `cargo fmt`
- [ ] `cargo clippy --all-features --all-targets -- -D warnings`
- [ ] `cargo nextest run --all-features --lib -E 'test(math_font) or test(math_layout) or test(omml_) or test(fractions_with_and_without)'`
- [ ] One full `cargo nextest run --all-features --lib` after the filter is green. Then the CLI `--help` smoke test.
- [ ] Before every cargo command, `pgrep -x cargo` and `pgrep -x cargo-nextest` must both print nothing. `CARGO_INCREMENTAL=0`. Never set `CARGO_TARGET_DIR`.
- [ ] Render the three fixtures with `--revisions word` into the job's `ours/` directory. Run `sbs.py`. Write `REPORT.md`: commits, test commands and results, image paths, what is still off versus Word (scripts still on the old ratios, noBar/lin/skw, n-ary, matrices, eqArr, accents, no mu spacing, assembly vs Word's glyph on a tall radical).

Do not run Word, osascript, or soffice. If a reference PDF is missing, name the docx in the report.
