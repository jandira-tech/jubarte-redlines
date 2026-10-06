#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Write hand_edited.docx: review/clean.docx plus one silent edit.

The last run of the payment paragraph gains " All amounts are exclusive of
VAT." through python-docx, with no w:ins/w:del anywhere - the negative
control for the tracked-changes check.

    python3 make_hand_edit.py     # after review/ exists
"""

from __future__ import annotations

from pathlib import Path

import docx

SRC = Path(__file__).resolve().parent / "review" / "clean.docx"
OUT = Path(__file__).resolve().parent / "hand_edited.docx"


def main() -> None:
    d = docx.Document(SRC)
    for p in d.paragraphs:
        if p.text.startswith("The Company will invoice"):
            p.runs[-1].text += " All amounts are exclusive of VAT."
            break
    else:
        raise SystemExit("payment paragraph not found")
    d.save(OUT)
    print(f"wrote {OUT.name} (one silent, untracked edit)")


if __name__ == "__main__":
    main()
