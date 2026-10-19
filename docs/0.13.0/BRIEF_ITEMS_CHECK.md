# Brief items 10 and 14 to 23: does the implementation plan handle each?

Date: 2026-10-19, checked against `src/` at `59d8d96c` and the open PR
stack. Verdict: nine of the eleven are handled by the right plan with
the right mechanism. Two plans are stale against the stack and must be
cut down (Plans 10 and 20). Three carry a gap to fix before
implementation (Plans 12, 14, 15).

## Item 14, un-bolded defined terms (Plan 12)

Handled; diagnosis confirmed. `replace` resolves the whole `find` range
as one edit (`apply_text_edit`, `src/edit.rs:5555`); the emitter
(`emit_text`, `src/edit/tracked.rs:735`) deletes every run in it and
writes one new run in the first run's formatting, so a bold "CPRA"
inside the range loses bold. The comparer path then records that as a
formatting change. Only `rewrite` runs the word-level diff today
(`rewrite_ranges`, called once at `src/edit.rs:3463`). Plan 12's fix,
running the same diff inside `replace`, is the correct minimal change.

Correction: instead of "expose the source-to-normalised offset table",
normalise the diff keys in `units()` (`src/edit/rewrite.rs`) with the
anchor module's `reads_as` table, so a curly quote and a straight one
compare equal and the source's character stays. That covers item 22.

## Item 22, curly quotes and non-breaking spaces swapped (Plan 12)

Handled with the amendment above. The lenient match (`reading_hits`,
`src/edit/anchor.rs`) is correct; the defect is that the replacement is
diffed against the raw characters. Consequence to state: after the fix
an agent cannot change a curly quote to a straight one through
`replace`; `whole` stays available for it.

## Item 20, duplicate numbering ids and 17 re-pointed paragraphs (Plan 16)

Handled; diagnosis confirmed. `copy_missing_numbering`
(`src/comparer/footnotes.rs:364`) clones each `w:num` with
`clone_subtree` in both branches, so a fresh `numId` carries the old
`w16cid:durableId`. The re-pointing comes from the same function: when a
`numId` in the source references a different abstract definition than
the revised copy's mapped one, every paragraph using it is re-pointed
(`num_remap`). First suspect: `normalized_abstract_num_signature`
(line 326) strips only `abstractNumId`, `restartNumberingAfterBreak`,
`nsid` and `tmpl`; anything else that differs between an accepted
document and its edited copy splits one list into two. The three
delivered documents were turn-1 edits, which run through the comparer
under the default mode, so Plan 15's rerouting removes the symptom and
Plan 16 fixes the comparer for `compare` and `whole`. Both are needed.

## Item 10, accepting a deletion that holds a comment anchor (Plan 10)

Already handled on the stack; the plan is stale. PR #1146 removes such
comments including their replies, as Word does, and warns. PR #1191
reports every removed thread on the edit paths, corrects the advice, and
adds the opt-in plan field `removed_anchor_comments: keep`. Plan 10
re-specifies both. Cut it to what the stack lacks: the `changes` listing
naming the mark a deletion holds, the view showing the mark inside the
deletion, and the wrong refusal code (`OVERLAPPING_EDITS`) for a reply
to a comment in a paragraph the plan deletes. Extend PR #1191's JSON
shape (`removed_comments: [{id, author, replies}]`) rather than add a
second one.

## Item 15, rewriting the counterparty's comment under their name (Plan 14)

Handled. Refuse by default, allow under an explicit plan field, let the
Word experiment (E5) decide what the author and date carry afterwards.

## Item 16, bare accept renumbers comment ids (Plan 14)

Handled, but the plan hides a trade-off. The renumbering was added on
purpose in commit 48aae202 as Word parity (rule R5: Word's own Accept
All, read off 51 files, numbers bookmarks and comments with one dense
counter in document order). Plan 14 reverses that for id stability. The
reversal is right: `edit` never renumbers, so jubarte is already
inconsistent with itself, and a stale handle that lands in the wrong
thread is worse than a non-dense id. But the plan must say it deviates
from Word, re-pin `tests/m_accept_word_parity.rs` so ids are compared
structurally rather than literally, and record the choice in
`docs/WORD_DIFFERENCES.md`. As written it lands as a surprise parity
regression. The `accept_revisions_package_keeping_ids` variant already
exists for compare, so the mechanism is there.

## Item 17, misattributed substitution in `read` (Plan 17)

Handled. Same-author guard on the pairing in `critic.rs` `render`, and
the nested id carried on every span. Examples are in the plan.

## Item 18, `insert_paragraph` with style Normal takes the heading's numbering and bold (Plan 15)

Handled; diagnosis confirmed: `build_paragraph` (`src/edit.rs:6224`)
clones the anchor's `w:pPr`, dropping only `sectPr`, `pPrChange` and
`rPr`, sets the style on top, and copies the dominant run's `w:rPr`.
The plan's rule "style given means style only" fixes the reported case.

Gap: with no `style` and no `like`, the plan keeps "copy the anchor",
which after a numbered heading produces another numbered heading.
Word's Enter at the end of a heading follows the style's next style
(`w:style/w:next`), Normal for headings. The default should read
`w:next` from the styles part and fall back to the anchor's style only
when it has none. Nothing in `src/edit.rs` reads `w:next` today (only
`append.rs`, `debug.rs` and the comparer touch it). Add it to Plan 15.

## Item 19, the opening quote hops across the paragraph boundary (Plan 15)

Handled by the rerouting. The emitter already has `DeleteParagraph`
(`src/edit/tracked.rs:108`, `:545`) and `InsertParagraph` (`:396`) arms
used under `keep`; sending every supported kind through it in every
mode removes the comparer's cross-boundary diff. The red tests named in
the plan cover the definition case and the partial reject.

## Item 21, a paragraph added after a mark the counterparty deleted is glued on Accept All (Plan 19)

Handled correctly as a report, not a behaviour change. Accepting the
deletion of a paragraph mark merges the paragraphs, so the glue is what
Word does. The warning names the revision and the way out. Run E3 first.

## Item 23, tabs dropped from `changes --json` and 518 errors from `validate --original` (Plan 20)

Half handled, half stale. The tab fix is right and small. The
foreign-author half is already fixed on the stack by PR #1197, which
aligns the edited file's revisions with the original's in document
order (and fixes the eleven matcher defects found by #1200 and #1204).
Plan 20's identity of (id, author, kind, text) is weaker and breaks on
exactly item 16, where ids change. Remove that half from Plan 20 and
make Plan 6's `--baseline` edits rule reuse PR #1197's
`check_foreign_revisions`.

## Plan edits to apply (not yet applied)

1. Plan 10: rewrite the problem statement and scope on top of PRs #1146 and #1191.
2. Plan 20: drop the foreign-author half; point Plan 6 at PR #1197's matcher.
3. Plan 14: add the Word-parity deviation, the test re-pin, and the WORD_DIFFERENCES entry.
4. Plan 15: add the next-style rule for the no-style, no-`like` default.
5. Plan 12: replace the offset-table step with key normalisation in the diff units; note that a quote-style change needs `whole`.

Issues to re-sync after the edits: #1260 (Plan 10), #1271 (Plan 20),
#1264 (Plan 14), #1266 (Plan 15), #1262 (Plan 12).
