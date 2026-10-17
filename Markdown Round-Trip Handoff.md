# Markdown round-trip handoff

jubarte-redlines, 2026-10-16. The state of branch `fix/md-roundtrip-fidelity`, PR #515, and the five-agent comparison with pandiff (pinned at commit `2631bb93`, `src/index.ts`, 536 lines).

**Branch**`fix/md-roundtrip-fidelity`\
0759b1e1, cab8ab73 (pushed)

**PR**#515, draft, into `feat/md-story-format`

**CI**security red: zip 8.x yanked, red on main too. Other checks had not reported.

**Tests**Per-file nextest green. Final combined run not done. `md_fixed_point` red at base too.

## What PR #515 does

- `convert X.docx -o Y.md` writes the agent view by default; `--commonmark` asks for plain CommonMark + CriticMarkup.
- Plain revision notes carry their id: `{>>r5 Ana Lima (date)<<}`. Comments keep `{>>Name (date): text<<}` with no number.
- A change across paragraph breaks names every piece (`r8+9+10`); the writer gives a paragraph mark its own id; the diff reads a numbered note as the author, not a comment.
- AGENTS.md: cargo-nextest day to day; `cargo miri nextest run` only for the rare full run.

The `security` failure is not this PR's: `scripts/security-check.sh:10-11` runs cargo-deny on `fuzz/Cargo.toml`, whose lockfile is gitignored (`.gitignore:99`), and every `zip` 8.x is yanked. Comment posted on #515 with two fixes: commit a seeded `fuzz/Cargo.lock` (quick), or move to `zip` 9.0.3+ together with `rdocx-opc` 0.5+ (which drops zip; an API migration).

## jubarte bugs found by the comparison

Ranked by harm. "Verified" means I reproduced it with `target/debug/jubarte` myself; the rest are the agents' reproductions, not yet rechecked.

1. **A deletion inside an insertion is lost on the way back to Word.**verified\
   The Word reader writes `{++one {--two--}{>>r3 Ben Ode …<<} three++}`, but the Markdown parser forbids a change inside a change (`inside()`, `src/markdown/critic.rs:176-182`). Result: `{--two--}` becomes inserted literal text, Ben's deletion disappears, and an empty comment by "r3 Ben Ode" appears. Evidence: `parsing/out/evidence_extra.txt`.
2. **Literal delimiters in a Word code block become a tracked change, in the agent view too.**verified\
   Code-block text is written unescaped (`from_docx/critic.rs:545-546`), but the writer still opens CriticMarkup inside fences (`critic.rs:522-526`). A 0-change docx with `fenced {++x++} code` comes back with `r1 insertion "x"`.
3. **`--commonmark` does not escape CriticMarkup when the document has no revisions.**verified\
   `from_docx/mod.rs:420-424` sets `plain`; `from_docx/critic.rs:550` returns the text as is. Literal `{\++a++}` comes back as `{++a++}` and the next convert makes a real insertion. Found by three agents independently.
4. **`--commonmark` does not escape Markdown syntax.**verified\
   Word text `- not a list` becomes a bullet list, `*not em*` becomes italics. Only the agent view escapes (`from_docx/mod.rs:2658-2662`, `if self.agent`). Fix: apply `escape_markdown` and `escape_line_starts` (`agent.rs:1231`, `1362`) in the plain view too.
5. **A comment nested in a change that contains `~>` or a closer breaks the change.**verified\
   `{~~a{>>x ~> y<<} b~>c~~}` becomes `r2 deletion "a{>>x "` plus `r1 insertion " y<<} b~>c"`, 0 comments. The closer search (`critic.rs:534-536`) does not skip nested spans, though jubarte claims one level of nesting (`critic.rs:136-143`). pandiff fails the same way.
6. **`diff` puts marks inside inline code, image targets and math.**verified (code)\
   ``Call `{~~foo~>bar~~}()` now.`` Our reader treats code spans as literal, so accept or reject does nothing. Fix in `atoms()` (`diff.rs:532-571`): whole-range atom for code spans, `Tag::Image` beside `Tag::Link`, enable math and make it atomic (pandiff's own rule, `index.ts:L48-L56`).
7. **Big changed blocks pair lines by position.**verified\
   Above 400 comparisons (`diff.rs:350`, `MAX_PAIRINGS` at `diff.rs:30`), when every line changed and one was inserted, each item is shown as rewritten into its neighbour. One insertion into an otherwise unchanged 20-item list is fine (Patience handles it).
8. **Agent-reported, not rechecked:**
   - One word added to a hard-wrapped paragraph gives 5 delete/insert pairs (line-based alignment).
   - List numbers skew pairing: `diff.rs:356` scores whole lines; score `&line[marker_len(line)..]` instead.
   - A list item turned into a paragraph is glued to the deleted item (`diff.rs:748`).
   - The diff escaper adds `\` without counting backslashes or skipping code (`diff.rs:213-235`); `unified.rs:1021` already counts correctly.
   - A closer inside an HTML comment or image alt text is dropped (`write.rs:2909-2919`, `write.rs:2319`), so the insertion runs to the end of the document with no warning.
   - A blank line inside `{>>…<<}` splits the paragraph.

## pandiff against jubarte, by stage

| Stage | pandiff (index.ts @ 2631bb93) | jubarte | Where the grammars differ |
| --- | --- | --- | --- |
| Regex / tokens | Three global regexes, L334-L336 (`del`, `ins`, `sub`), applied in that fixed order. No `{==`, no `{>>`. No escapes; the only backslash regex is L256, which deletes every `\(` `\)` `\[` `\]`, code included. Wrapping: L373 sees only ```` ``` ```` fences, L379 wraps everything else at 72. | Closer index built once, escape-aware (odd backslash count, `critic.rs:124-132`, `203-208`). Unicode word tokens (`util/words.rs:83`) and whole-number atoms (`diff.rs:585-619`). Linear-time tested (`critic.rs:1212-1219`). | pandiff splits `naïve` and `$5,000,000` mid-word (node-htmldiff's ASCII class); jubarte keeps whole words and amounts. |
| Parsing | Never parses CriticMarkup into a structure. Reads it only in `normalise` (L530-L535, via `criticReject` L417-L421 and `criticAccept` L422-L426) and `postrender` (L464-L470). Word input: pandoc `--track-changes=all` (L527); L67-L73 unwraps every span that is not an insertion or deletion, so comment text is pasted into the body. | CriticMarkup is read before Markdown: `critic::encode` swaps delimiters for stand-ins (`critic.rs:57-74`), pulldown-cmark parses, then tokens are split back (`write.rs:2326-2336`). A `Literal` pre-scan keeps code spans and link destinations verbatim (`critic.rs:247-252`). | jubarte allows one nesting level; pandiff has none. jubarte reads CriticMarkup inside fences but not inside code spans. |
| Modification (diff) | Both sides rendered to HTML by pandoc (L298-L299), aligned by node-htmldiff (L300), then rewritten: math whole (L48-L56), code blocks as line diffs (L139-L146), tags hoisted (L157-L169), marks merged (L172-L178), del+ins paragraph split (L181-L191), del next to ins becomes a substitution (L194-L201). Threshold gives up (L305-L310). No move detection. | Patience over source lines (`diff.rs:74`), similarity LCS pairing up to 400 comparisons (`diff.rs:350`), Myers over words (`diff.rs:476`), atoms (`diff.rs:638`). The Word route (`compare -o x.docx`) detects moves (`comparer/mod.rs:1502`). | pandiff loses added emphasis, changed link and footnote text, and paragraph splits and merges; jubarte keeps them. jubarte loses on re-wrapping and syntax-only changes. |
| Reconstruction | pandoc writes Markdown with `-smart`, `-pipe_tables` and others (L316-L330); regexes write CriticMarkup (L362-L367) with no escaping. To docx (L408-L415, L467-L468): author `unknown`, no `w:date`, a del+ins pair shares one `w:id` (our `validate` reports DUPLICATE_REVISION_ID), paragraph marks untracked. | Delimiters escaped with a backslash (`from_docx/critic.rs:717-737`), notes carry `rN`, author and date. Except: bugs 2-4 above. | pandiff cannot read our notes: `{~~30~>45~~}{>>r2+1 Owner (date)<<}` becomes `{~~30{~>45{~~}\>\>r2+1 …`. |

## The Lezer tokenizer

Verdict: do not adopt it as the tokenizer. It has no escapes, no Markdown context, no nesting (our writer emits comment-in-change), recognises every closer everywhere, and has no grammar for `r3+4` notes. Only its generated tables exist, and lezer-generator only outputs JS. What it does better is report malformed markup (unclosed opener, stray closer, `{~~a~~}` with no `~>`, `{++a--}`); jubarte silently keeps those as text, and `{~~a~~}` becomes strikethrough that `--commonmark` then drops. Recommendation: have `convert` and `validate` warn on those cases (jubarte already detects each at `critic.rs:534-536`) and keep Lezer for editor highlighting.

## Decisions that are yours

1. Nested change: should the parser accept a deletion inside an insertion, or should the Word reader split the insertion around it? This decides the fix for bug 1.
2. Should `--commonmark` always escape CriticMarkup and Markdown syntax? My recommendation is yes: today it silently creates tracked changes and lists.
3. Should there be a bare spec-CriticMarkup output (no `rN` notes) for tools like pandiff?
4. Literal delimiters in code: fence rule (read inside) or code-span rule (literal)? Bug 2's fix depends on it.
5. zip: quick lockfile, or the zip 9 + rdocx-opc migration?
6. Is `MAX_PAIRINGS = 400` a deliberate performance bound?

## Backlog carried over

- B1, C1 (trace the lost insertion in fixture 007), D1; agent-only round-trip issues.
- A2: underline/highlight in the plain view (undecided).
- G: what Word does with a malformed `docProps/core.xml` (not started).
- Stack #507-#511: needs your merge or bypass, bottom-up.
- Optional: `cN` numbers on plain-view comments, if you want them.
- Before merging #515: run the combined nextest pass once.

### House rules (AGENTS.md)

Cargo runs sequentially in the default `target/`, never `CARGO_TARGET_DIR`; no `#[allow]`; fmt and clippy `-D warnings`; targeted `cargo nextest run --test NAME` (an `-E` filter applies to every binary in the run).

### Evidence

The five full reports, the line-numbered `index.ts` and the pasted Lezer parser are in `/home/user/handoff/pandiff-reports/`. The probe cases are under the session scratchpad (`agents/*/`). Both live in this container and go away when it is reclaimed.