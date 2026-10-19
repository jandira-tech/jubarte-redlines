@coderabbitai plan against `src/markdown/from_docx/critic.rs` (`same_author` 187, `tree` 474, `fold_on` 611, `render` 836-860), `src/markdown/from_docx/agent.rs` (`join_tags`), `docs/DIALECT.md` line 69; be strict on static analysis (`cargo clippy --all-targets -- -D warnings`, no `#[allow]`), and argue for a different split, shape or refusal if the evidence in the named tests and the Word fixtures supports it.

# Plan 17: view attribution (#1111)

**Stack.** Step E7 of the jubarte 0.13 contract-redlining plan; branch `fix/view-attribution-1111`; PR over PR on the previous step, never merged until asked. Tracking issue: TRACKING_PLACEHOLDER. The whole plan (23 plans, conventions, Word experiments E1–E7) is `IMPLEMENTATION_PLAN.md` in the planning folder; the chosen alternative and its scores are in `PROPOSALS_PLAN.md` §A-detail 17.

**Problem.** s2-t2-g03c p47: XML order is LargeCo `w:ins` "limited, ",
our `w:del` "non-exclusive", our `w:ins` "nonexclusive"; the view prints
`{~~non-exclusive~>limited, ~~}{>>r524 @AL<<}{>>r141 @TX<<}{++nonexclusive++}`,
fusing our deletion with their insertion and reversing the order.
s1-t3-g03a p8: a counterparty deletion inside a multi-run insertion
prints `{--applicable--}{>>r16 @TX2<<}` without ` on r13` (brief item 17).

**Design (§A-detail 17, A).** `render` (`src/markdown/from_docx/critic.rs:836-860`)
pairs an insertion followed by a deletion into a substitution whenever
neither names an outer change; add `same_author(new_by, old_by)` (187)
to the guard, so a pair by two authors stays two changes in document
order. Always print ` on rN` for a change nested in another, whatever
the outer change's remaining text (`fold_on` 611 attaches the outer only
when the inner is the sole content; `tree` 474 must carry the outer id
on every nested span). Doc change: `docs/DIALECT.md` line 69
(`{~~a~>b~~}{>>r1 @AC<<}{>>r2 @JD<<}` "substitution by two authors")
becomes a read-only form: the reader still accepts it, the view no
longer prints it. Arthur: this changes a line of the settled dialect;
say if you want the two-author pair kept printable.

**What the agent sees.** Ids and handles are illustrative; the shapes
are the two bench cases. `TL` is the counterparty, `RC` is us.

Case A, adjacent changes by two authors (s2-t2-g03c p47). Source: "a
non-exclusive licence". The counterparty inserted "limited, " in front
of it; we then replaced "non-exclusive" with "nonexclusive". XML order:
their `w:ins` "limited, ", our `w:del` "non-exclusive", our `w:ins`
"nonexclusive". Before (0.12.0):

```
a {~~non-exclusive~>limited, ~~}{>>r141 @RC<<}{>>r524 @TL<<}{++nonexclusive++}{>>r142 @RC<<} licence
```

The view says "non-exclusive became limited, ", then "nonexclusive" was
added: document order reversed, a replacement nobody made, two authors
on one mark. To strike the counterparty's "limited, " the agent must
know that a substitution's second tag is its insertion side; agents got
this wrong, and `changes` lists the three in document order, so the two
outputs disagree. After:

```
a {++limited, ++}{>>r524 @TL<<}{--non-exclusive--}{>>r141 @RC<<}{++nonexclusive++}{>>r142 @RC<<} licence
```

Document order, one author per mark. Our own two marks keep pairing as
today when the view joins them (`{~~non-exclusive~>nonexclusive~~}{>>r141+142 @RC<<}`
with plan 11's grouping); same-author pairs are untouched.

Case B, lost nesting (s1-t3-g03a p8). We inserted "the applicable law of
England"; the counterparty deleted "applicable" inside it. Before:

```
{++the ++}{>>r13 @RC<<}{--applicable--}{>>r16 @TL<<}{++law of England++}{>>r13 @RC<<}
```

Nothing says the deletion sits inside our proposal; `changes` reports
`r16 … inside r13`, the view does not, so the agent cannot see that the
counterparty rejected part of what we wrote. After:

```
{++the ++}{>>r13 @RC<<}{--applicable--}{>>r16 @TL on r13<<}{++law of England++}{>>r13 @RC<<}
```

In the settled `re` dialect the note reads `{r16 @TL re r13 @RC}`.

The alternative considered and rejected: pairing across authors only
when the deletion precedes the insertion would print "they struck 30
days, we proposed 45 days" as `{~~30 days~>45 days~~}{>>r50 @TL<<}{>>r60 @RC<<}`.
The order is right, but two acts still read as one replacement, and
today's code never pairs a deletion-then-insertion at all, so that
option adds a pairing instead of removing one. Line 69 of
`docs/DIALECT.md` stays a form the reader accepts; the view stops
producing it.

**Files.** `src/markdown/from_docx/critic.rs` (`same_author` 187,
`tree` 474, `unnest` 572, `fold_on` 611, `render` 836),
`src/markdown/from_docx/agent.rs` (`join_tags` 245),
`tests/agent_view_attribution.rs` (new; fixtures synthesized from the
two cases), `tests/agent_text_view.rs`, `docs/DIALECT.md`,
`docs/MARKDOWN.md`.

**Steps.**

- [ ] Red: `two_authors_never_pair_as_a_substitution` (expected:
      `{++limited, ++}{>>r524 @AL<<}{--non-exclusive--}{>>r141 @TX<<}{++nonexclusive++}{>>r142 @TX<<}`,
      with the plan-11 grouping where it applies),
      `same_author_pair_still_prints_a_substitution`,
      `nested_deletion_always_names_its_outer_insertion` (`{--applicable--}{>>r16 @TX2 re r13 @AC<<}`
      in the `re` dialect, or ` on r13` in the current one: match the
      dialect the branch prints), `reader_still_accepts_a_two_author_pair`.
- [ ] Implement; coordinate with `fix/nested-on-1111` (take its tests).
- [ ] fmt, clippy, `cargo nextest run --test agent_view_attribution
      --test agent_text_view`; commit on `fix/view-attribution-1111`.

**Acceptance.** For every fixture, each change the view prints names the
author `changes` lists for it, and every `inside rN` in `changes` has a
nesting note in the view.

**Related.** #1111, #1135, brief item 17.

---
