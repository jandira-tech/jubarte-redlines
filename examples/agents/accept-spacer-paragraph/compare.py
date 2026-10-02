#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Accept every case with ``jubarte accept`` and with LibreOffice; print both.

    python3 compare.py [WORKDIR]

Writes each case, ``<case>.jubarte.docx`` and ``<case>.libreoffice.docx`` to
WORKDIR (default ``./out``) and prints one row per case: the body paragraphs
each tool leaves, ``#`` marking a numbered paragraph and ``_`` an empty one.
An empty numbered paragraph (``#_``) is the stray bullet the skill warns of.
Exits 1 if the two tools disagree on any case or either leaves ``#_``.

Needs ``jubarte`` on PATH (or ``JUBARTE=/path/to/jubarte``), ``soffice`` with
Writer, and the system Python's ``uno`` module for the LibreOffice side.
"""

from __future__ import annotations

import os
import subprocess
import sys
import xml.etree.ElementTree as ET
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import make_redline

W = "{http://schemas.openxmlformats.org/wordprocessingml/2006/main}"


def paragraphs(docx: Path) -> list[str]:
    """Body paragraphs as text, `#` when numbered (numId other than 0), `_` when empty."""
    root = ET.fromstring(zipfile.ZipFile(docx).read("word/document.xml"))
    out = []
    for para in root.find(f"{W}body").findall(f"{W}p"):
        text = "".join(t.text or "" for t in para.iter(f"{W}t"))
        num = para.find(f"{W}pPr/{W}numPr/{W}numId")
        numbered = num is not None and num.get(f"{W}val") != "0"
        out.append(("#" if numbered else "") + (text or "_"))
    return out


def main() -> int:
    work = Path(sys.argv[1]) if len(sys.argv) > 1 else HERE / "out"
    work.mkdir(parents=True, exist_ok=True)
    jubarte = os.environ.get("JUBARTE", "jubarte")
    bad = False
    for case, body in make_redline.CASES.items():
        src = work / f"{case}.docx"
        make_redline.write(src, body)
        ours = work / f"{case}.jubarte.docx"
        subprocess.run(
            [jubarte, "accept", str(src), "-o", str(ours), "--force"], check=True
        )
        theirs = work / f"{case}.libreoffice.docx"
        subprocess.run(
            [
                "/usr/bin/python3",
                str(HERE / "libreoffice_accept.py"),
                str(src),
                str(theirs),
            ],
            check=True,
        )
        a, b = paragraphs(ours), paragraphs(theirs)
        flag = "" if a == b and "#_" not in a + b else "  <-- differs"
        bad |= bool(flag)
        print(f"{case:22} jubarte={a}  libreoffice={b}{flag}")
    return 1 if bad else 0


if __name__ == "__main__":
    raise SystemExit(main())
