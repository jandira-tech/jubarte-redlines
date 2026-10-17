# Duplicated logic in `src/`

Places where the crate does one job in more than one way. Each entry was found by a tool (tree-sitter syntax-tree comparison, see [DUPLICATE_CANDIDATES.md](DUPLICATE_CANDIDATES.md)) and then confirmed by reading the code. Line numbers are at the commit this was generated from. No source file was changed.

How to read the verdicts:

- **Copies**: same behaviour written out more than once. Merging is mechanical.
- **Diverged**: the copies have drifted and now disagree on some input. Merging means choosing which behaviour wins, so it is a decision, not a refactor.
- **Overlap with a dependency**: hand-written code for something a crate in `Cargo.toml` already does.

Summary, most valuable first:

| # | Job | Ways it is done | Verdict |
|---|---|---|---|
| 1 | "Is this a `rsid` attribute?" | 9 sites, 4 different rules | Diverged |
| 2 | Heading level of a paragraph style | 5 functions, 5 rules | Diverged |
| 3 | Find the main document part | 6 functions, 4 fallback policies | Diverged |
| 4 | XML escaping and unescaping | 7 functions | Diverged |
| 5 | Collapse whitespace | 4 functions, 3 definitions of "whitespace" | Diverged |
| 6 | Base64 decoding | 2 hand-written decoders, plus the `base64` crate | Overlap with a dependency |
| 7 | `xml:space="preserve"` decision | 3 copies, 2 rules | Diverged |
| 8 | CriticMarkup delimiter escaping | 2 identical copies, 2 related variants | Copies |
| 9 | Style pairing key | 2 copies | Copies |
| 10 | Load and store an XML part as a DOM | 4 functions | Copies and variants |
| 11 | RGB to hue/saturation/lightness | 2 implementations | Diverged conventions |
| 12 | Small exact copies | 5 pairs | Copies |

---

## 1. "Is this a `rsid` attribute?" (9 sites, 4 rules)

`rsid*` attributes are Word's editing-session bookkeeping. The code needs to drop or ignore them in many places and asks the question four different ways:

| Rule | Where |
|---|---|
| Closed list of 8 names, in the `w:` namespace only | `comparer/preprocess.rs:48`, `markup_simplifier.rs:136` (the two lists are identical) |
| Any namespace, local name lower-cased, `starts_with("rsid")` | `comparer/formatchg.rs:40`, `document_comparer.rs:1494` and `:1737`, `comparer/finalize.rs:3307` |
| Any namespace, case-sensitive `starts_with("rsid")` | `comparer/produce.rs:1584`, `comparer/finalize.rs:11613`, `comparer/mod.rs:1553`, `debug/diff.rs:513` |

They disagree on three inputs: an `rsidXyz` attribute that is not in the closed list, an `rsid*` attribute in another namespace or none, and a name with different capitalisation (`RSIDR`). Two stages can therefore disagree about whether the same attribute is noise. If the closed list is the correct definition, every prefix test is wider than it should be; if the prefix is, the two list-based functions miss attributes.

Suggested direction: one `is_rsid_attr(&XName)` in `namespaces.rs` (or next to `W`), and the call sites that filter by name use it.

## 2. Heading level of a paragraph style (5 functions)

| Function | Rule |
|---|---|
| `audit.rs:502` `heading_level` | style name `heading N`, else style id starting `heading`, via `level_after` |
| `fields.rs:1065` `heading_level` | style id `HeadingN` with N in 1 to 9, else style name `heading N` |
| `markdown/patch.rs:380` `heading_level` | style id `HeadingN` with N in 1 to 9 only (no name lookup) |
| `markdown/from_docx/mod.rs:1825` `Styles::heading_level` | walks the style's `basedOn` chain, `title` is level 1, `heading N` clamped to 1 to 6, then the outline level |
| `markdown/from_docx/mod.rs:1642` `heading_level(styles, p, agent)` | paragraph-level `outlineLvl` first (value 9 clears it), then the style |

The same paragraph can be a heading to `fields` (TOC building) and not to `patch`, or level 8 in one and level 6 in another. `markdown/write.rs:5594` also has a `heading_level` but that one converts a pulldown-cmark enum and is unrelated.

## 3. Find the main document part (6 functions, 4 policies)

`PartFs::main_document_part()` (in `opc`) already answers this. Five callers wrap it, each with its own fallback when the answer is `None`:

| Function | Fallback |
|---|---|
| `append.rs:323` `main_part` | none; error `no main document part`, also requires the part to have bytes |
| `changes.rs:398` `main_part` | the constant `word/document.xml`, without checking the part exists |
| `inspect.rs:683` `main_part` | `word/document.xml` only if that part exists, else `InspectError::MissingDocument` |
| `validate.rs:1084` `main_part` | the same as `inspect.rs`, returning `Option` (exact copy modulo the error type) |
| `markdown/from_docx/ooxml.rs:140` `main_part(fallback)` | reads the relationships itself (`first_of_type("/officeDocument")`), then the caller's fallback if the part is missing |
| `admission.rs:717` `main_part` | does not use `PartFs` at all: scans `_rels/.rels` from raw bytes against the Transitional and Strict relationship-type list, and requires `[Content_Types].xml` |

`changes.rs` is the odd one out: on a package with no `officeDocument` relationship and no `word/document.xml` it returns a path that does not exist, where `inspect` and `validate` report a missing document.

## 4. XML escaping and unescaping (7 functions)

Escaping:

- `edit/watermark.rs:81` `escape` and `convert/altchunk.rs:765` `xml_escape`: identical four-`replace` chains (`& < > "`).
- `comparer/preprocess.rs:1191` `escape_xml_attr` and `:1202` `escape_xml_text`: the same replacements with a fast path that returns early when nothing needs escaping (the text variant leaves `"` alone).
- `markdown/xml.rs:426` `escape`: returns `Cow`, and also drops characters that are illegal in XML 1.0.
- `xmllinq/serialize.rs:537` `write_escape_attr` and `:555` `write_escape_text`: the serializer's own, writing into a buffer.

Unescaping:

- `xmllinq/parse.rs:704` `unescape_xml_text`: named and numeric references.
- `convert/altchunk.rs:724` `decode_entities`: five named entities plus `&nbsp;` and numeric references.
- `comparer/parts.rs:44` `decode_xml_attribute`: the four basic entities only; `&apos;` and numeric references pass through untouched.

The `xmllinq` pair is the serializer's own and the parser's `unescape_xml_text` is its inverse. The others reimplement subsets, and `decode_xml_attribute` leaves an attribute value containing `&apos;` or `&#x20;` undecoded.

## 5. Collapse whitespace (4 functions, 3 definitions)

| Function | Treats as whitespace |
|---|---|
| `comparer/moves.rs:118` and `markdown/from_docx/ooxml.rs:499` | Unicode `White_Space`, including NBSP (`split_whitespace` then `join(" ")`); identical |
| `convert/altchunk.rs:704` | Unicode whitespace except NBSP |
| `convert/mod.rs:16220` | ASCII blanks only (space, CR, VT, FF); keeps `\n` and treats NBSP and the ideographic space as text |

The `convert` rule is deliberate (its comment says so). The other three are the same job with a different edge case each, and the `moves.rs` copy feeds the similarity score used for move detection, so its NBSP behaviour reaches redline output.

## 6. Base64 decoding (2 hand-written, 1 crate)

- `convert/altchunk.rs:97` `base64_decode`: hand-written, lenient (skips unknown bytes, stops at `=`), returns `Vec<u8>`.
- `edit/images.rs:38` `decode_base64`: hand-written, strict (ignores whitespace, errors on any other byte or data after padding), returns `Result`.
- `markdown/from_docx/drawing.rs:1171` to `:1195` and `markdown/write.rs:2163`: use the `base64` crate, which `Cargo.toml` declares as a direct dependency.

Both hand-written decoders could call the crate; the leniency of the first and the error message of the second are the two behaviours to preserve.

## 7. `xml:space="preserve"` decision (3 copies, 2 rules)

`xml_space_attr(text)` exists in `comparer/finalize.rs:1040`, `comparer/produce.rs:671` and `comparer/atomize.rs:661`. The first two are identical (first or last character is Unicode whitespace). The third only checks for an ASCII space. Text that starts or ends with a tab, newline or NBSP gets `xml:space="preserve"` in two stages and not in the third.

## 8. CriticMarkup delimiter escaping

- `edit/resolution.rs:27` `escape_critic` and `markdown/from_docx/critic.rs:1009` `escape` are the same function: the same 11-entry delimiter table and the same body. This is the cleanest exact copy in the crate.
- `markdown/critic.rs:129` `escaped` and `markdown/diff.rs:363` `escaped_at` are the same backslash-parity test.
- `markdown/diff.rs:294` and `markdown/unified.rs:1071` `escape` handle other delimiter families (`[-`, `{+`), and `markdown/view_stories.rs:774` `unescape` reverses one of them. These are related but different jobs.

## 9. Style pairing key

`append.rs:825` `style_key` and `document_comparer.rs:1843` `style_match_key` have identical syntax trees: Word's key for pairing two stylesheets' styles (type plus name, built-in names folded to lower case). Both doc comments describe the same rule in different words.

## 10. Load and store an XML part as a DOM

- `revision_processor/comments.rs:38` and `:51` (`Parsed::load`, `Parsed::store`) and `scrub.rs:189` and `:202` (`Part::load`, `Part::store`) are exact copies.
- `inspect.rs` `parse_part` and `append.rs` `parse` do the same job for a part of a `PartFs`, each with its own error type.

## 11. RGB to hue, saturation, lightness

`convert/mod.rs:19612` `rgb_to_hsl` (`f32`, returns `(h, s, l)`) with `hsl_to_rgb` at `:19631`, and `markdown/package.rs:816` `rgb_to_hls` (`f64`, returns `(h, l, s)`) with `hls_to_rgb` at `:839`. Same maths, opposite tuple order and different precision. A caller moving between the two modules has to remember which order it is holding.

## 12. Small exact copies

| Job | Where |
|---|---|
| Element children of a node | `comparer/preprocess.rs:506` `element_children`, `revision_processor.rs:817` `element_children_vec` |
| Build a `w:r` holding a `w:fldChar` | `comparer/finalize.rs:6409` `field_char_run`, `fields.rs:1286` `fld_char_run` |
| Eight-digit and two-digit hex check | `comments.rs:1098` `is_hex8`, `markdown/borders.rs:268` `is_hex_byte` |
| `Debug` for two option structs | `markdown/mod.rs:287` and `markdown/redline.rs:66` are identical hand-written `fmt` bodies, which `#[derive(Debug)]` would replace |

## What the tool found that is not a problem

- 38 of the 65 identical-tree groups are siblings in one file: `shift_op_x` and `shift_op_y`, `read_u16` and `read_u32`, `accept_revisions` and `reject_revisions`, the four digit-label tables in `convert/mod.rs`. Short and symmetric, so merging them would cost more than it saves.
- Many same-name functions are different jobs on different types (`paragraph_id`, `runs`, `line`, `mark`, `to_json`, `load`). They are listed in the candidates file and were not treated as duplicates.
- Inverse pairs (`rgb_to_hsl` and `hsl_to_rgb`, `base64_encode` and `base64_decode`) match the name test but are not copies.

## Limits

- Detection is syntactic. Two functions that compute the same result with different code (a loop and an iterator chain, say) are not found; only the ones that share a tree, a near-identical tree, or a name were reviewed by hand.
- Only functions with at least 40 syntax nodes were compared for body similarity, so tiny helpers are covered only by name.
- Test code is excluded.
- "Diverged" entries are based on reading the function bodies listed. Whether the divergence changes real documents was not tested.
