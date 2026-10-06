<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# 12 — Comments: comment on a phrase, reply, resolve

The task both tools are given: in `input.docx` (a one-page NDA), comment
on the phrase "who need it for the Permitted Purpose" as "Ann Counsel",
have "Ben Vendor" reply to that comment, and resolve the thread. The
substituted tool is python-docx 1.2.0, which gained `add_comment` in
1.2.0.

## The exact commands

Substituted tool (python-docx 1.2.0):

```bash
python3 comment_pydocx.py   # add_comment(runs, text=..., author=..., initials=...)
```

That is all python-docx can do here: `dir(Comment)` lists `author`,
`initials`, `timestamp`, `comment_id`, `paragraphs`, `tables`, `text`
and body-building methods — there is no reply and no resolve API, so
the reply and resolve halves of the task have no python-docx command to
show. (Reaching for raw XML would mean hand-writing
`word/commentsExtended.xml` paraId pairs, which is exactly the
marker-pasting the Anthropic skill does.)

jubarte 0.11.2 (two plans, the second bound to the first's output — a
reply can only name a comment already present in its source):

```bash
jubarte edit input.docx         --plan plan-1-comment.json --out-dir review-1 --png --dpi 72
jubarte edit review-1/clean.docx --plan plan-2-thread.json --out-dir review-2 --png --dpi 72
jubarte comments review-2/clean.docx --json
```

Tool versions used here: jubarte 0.11.2, python-docx 1.2.0. Both page-1
renders come from `jubarte convert --png --dpi 72`; comments are painted
(the comment-bearing renders are byte-different from the comment-free
render of `input.docx`).

## Outputs

| File | What it is |
|---|---|
| `input.md` / `input.docx` | source |
| `comment_pydocx.py`, `comment_pydocx.docx` | the python-docx script and its result |
| `pydocx_anchor.txt` | what the script anchored: 3 whole runs, highlighting the whole sentence-plus, not the phrase |
| `comments_pydocx.jsonl` | `jubarte comments` on the python-docx output: 1 comment, no `parent`, `done: false` |
| `comment_page_1_pydocx.png` | page 1: one balloon over a paragraph-wide highlight |
| `plan-1-comment.json` | the comment plan (`find` anchors the exact phrase) |
| `plan-2-thread.json` | the reply + resolve plan, bound to `review-1/clean.docx` by sha256 |
| `review-1/`, `review-2/` | the two `jubarte edit` outputs |
| `comments_jubarte.jsonl` | the finished thread: id 1 has `"parent": 0`, both records `"done": true` |
| `comment_page_1_jubarte.png` | page 1 of `review-2/clean.docx`: the resolved two-entry thread |

## Verdict

python-docx 1.2.0 can add the comment, with two real costs. The anchor
is run-granular: covering the phrase meant covering three whole runs, so
the comment highlights "Recipient may disclose … who need it for the
Permitted Purpose, provided each employee signs a" — the entire
paragraph minus its tail — where jubarte's `find` anchored exactly the
38-character phrase (compare `anchor_text` in the two jsonl files). And
the thread stops there: no reply, no resolve, no `done` flag — the other
two-thirds of the task are out of the library's reach.

jubarte did all three steps in two plans, and `jubarte comments --json`
reads the thread back with `parent` and `done` set. The two-plan split
is jubarte's own constraint, honestly: `reply_comment` can only name a
comment already in the document, so add-then-reply takes two `edit`
calls and the second plan is bound to the first output's sha256.

Other honest notes: python-docx comment bodies are rich (paragraphs,
tables); jubarte's `comment` takes plain `text` only. python-docx stamps
wall-clock time on the comment, so `comment_pydocx.docx` and its render
are not byte-reproducible across runs; jubarte takes the plan's fixed
`date`, so its outputs are.

Discrepancies with the adoption pages: none. The anthropic page's
comment-thread claims (`comment` with `find` + `text`; `reply_comment`,
`resolve_comment`; `jubarte comments FILE --json`; the reply-must-exist
two-plan flow) all behaved exactly as written; the `commentsExtended` /
`commentsIds` / `commentsExtensible` parts the page promises are present
in the outputs (see `review-2/clean.docx`).
