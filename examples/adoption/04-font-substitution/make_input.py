# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Build input.docx: a letter that asks for three fonts, none of the
first two installed (the third is a fake name that only resembles one).

  Garamond Premier Pro  not installed anywhere here
  Fake Serif Pro        a made-up name
  Calibri               installed (the control)
"""
import docx
from docx.shared import Pt

d = docx.Document()

h = d.add_paragraph()
r = h.add_run("Font substitution probe")
r.font.name = "Georgia Pro"
r.font.size = Pt(16)
r.bold = True

p = d.add_paragraph()
r = p.add_run(
    "This paragraph is set in Garamond Premier Pro, which is not "
    "installed on this machine. If the renderer substitutes it, the "
    "letterforms below will come from some other face, and a reader "
    "comparing the proof with the author's screen sees different type."
)
r.font.name = "Garamond Premier Pro"
r.font.size = Pt(11)

p = d.add_paragraph()
r = p.add_run(
    "This paragraph is set in Fake Serif Pro, a name that exists on no "
    "system at all. It is here to show what each tool does with a made-"
    "up font name rather than a real but absent one."
)
r.font.name = "Fake Serif Pro"
r.font.size = Pt(11)

p = d.add_paragraph()
r = p.add_run(
    "This paragraph is set in Calibri, which is installed; it is the "
    "control. Whatever the tools report about the two paragraphs above, "
    "this one must come out unsubstituted."
)
r.font.name = "Calibri"
r.font.size = Pt(11)

d.save("input.docx")
