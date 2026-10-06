<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# Accepting and rejecting a redline: LibreOffice vs `jubarte accept` / `reject`

Anthropic's docx skill produces a clean copy with a LibreOffice macro that
dispatches `.uno:AcceptAllTrackedChanges` (`scripts/accept_changes.py`); the
adoption page offers `jubarte accept redline.docx -o clean.docx` (and
`reject`) as the replacement. This folder builds one redline and accepts it
with both, side by side, then rejects it with jubarte and checks the result
against the original.

## The inputs and the redline

`original.md` and `revised.md` are two versions of a one-page consulting
letter (date, day count, fee, and the termination clause changed). Both are
built with the jubarte Markdown writer, and the redline comes from the
two-document compare:

```bash
jubarte convert original.md -o original.docx --force
jubarte convert revised.md -o revised.docx --force
jubarte original.docx revised.docx -o redline.docx --author Ann --force
jubarte changes redline.docx --json     # 9 revisions, all by Ann (changes_jubarte.json)
```

## The commands

```bash
# The substituted tool: LibreOffice accept-all.
# Documented runner: Python-UNO (libreoffice_accept.py, .uno:AcceptAllTrackedChanges).
# On this machine neither interpreter named on the adoption pages can run uno:
#   /opt/homebrew/bin/python3 -> ModuleNotFoundError: No module named 'uno'
#   /usr/bin/python3          -> ModuleNotFoundError: No module named 'uno'
#   /Applications/LibreOffice.app/Contents/Resources/python -> Killed: 9 (exit 137,
#   macOS quarantine kills the bundled interpreter; full log: uno_probe.log)
# run.sh therefore dispatches the same .uno command from a Basic macro
# (Standard.Module1.AcceptAllTrackedChanges) installed in this folder's own
# LibreOffice profile - the mechanism Anthropic's script itself uses.
soffice -env:UserInstallation=file:///tmp/lo_adopt_08 --headless --norestore \
  "macro:///Standard.Module1.AcceptAllTrackedChanges(\"IN\",\"OUT\")"

# The replacement:
jubarte accept redline.docx -o accepted_jubarte.docx --force
jubarte reject redline.docx -o rejected_jubarte.docx --force
```

## Tool versions used here

- jubarte 0.11.2 (binary at `/Users/arthrod/temp/T/jr-adopt-bin/jubarte`)
- soffice: LibreOffice 26.8.0.3 (homebrew), own profile `lo_adopt_08`
- Python 3.14.7 (probe only; no usable `uno` module, see above)

## Outputs

| File | Made by |
|---|---|
| `original.md` / `revised.md`, `original.docx` / `revised.docx` | the two versions |
| `redline.docx`, `changes_jubarte.json` | `jubarte` compare; the 9 revisions |
| `accepted_libreoffice.docx`, `basic_macro.log` | LibreOffice accept-all |
| `accepted_jubarte.docx` | `jubarte accept` |
| `rejected_jubarte.docx` | `jubarte reject` |
| `accept_text_jubarte.txt`, `accept_text_libreoffice.txt`, `accept_text.diff` | `jubarte text` of both accepted files and their diff |
| `reject_text_jubarte.txt`, `text_original.txt`, `reject_text.diff` | reject vs original |
| `revisions_accepted_*.log` | tracked changes left in each accepted file (0 and 0) |
| `accept_page_1_jubarte.png`, `accept_page_1_libreoffice.png` | page 1 of each accepted file, rendered by `jubarte convert --png --dpi 72` |
| `uno_probe.log` | the interpreter probe quoted above |

## Result

- `jubarte revisions` reports **0 revisions** in both accepted files.
- `accept_text.diff` shows exactly four lines of difference, all the same
  kind: LibreOffice writes an explicit `Normal` paragraph style on the body
  paragraphs, so `jubarte text` tags them `[body:p:1 Normal]` where
  jubarte's own output has no style tag. The text itself is identical, word
  for word, in all four paragraphs.
- `reject_text.diff` is empty: `jubarte reject` returns the original text
  exactly.

## Verdict

On this redline, jubarte's accept matches LibreOffice's accept-all: same
accepted text, no surviving revisions, and page 1 renders of the two
outputs are visually the same. The one measurable difference is in
serialization, not acceptance: LibreOffice rewrites the whole package in its
own dialect (explicit `Normal` styles; a 6.5 kB file for a 4 kB source)
while jubarte's output stays closer to the source package. jubarte's
`reject` has no LibreOffice counterpart in this folder (the skill's macro
only accepts); the checked expectation - reject == original, diff empty -
holds.

## Discrepancies with the adoption pages

None in the commands' behavior. One environmental note, not a jubarte
fault: the adoption page's documented runner for the LibreOffice side is
Python-UNO, and on this macOS machine no available interpreter can import
`uno` (the bundled one is killed by Gatekeeper), so the macro dispatch shown
above is the only working LibreOffice accept path here; `run.sh` probes and
records this (`uno_probe.log`) before falling back.
