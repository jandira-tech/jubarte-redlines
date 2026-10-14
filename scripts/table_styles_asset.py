#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""Writes src/markdown/word_table_styles.tsv: Word's built-in Grid Table and
List Table styles (1 to 7, plain and Accent 1 to 6), as Word wrote them into
a document that uses all 98, one template per family.

  python3 scripts/table_styles_asset.py [SOURCE.docx]

Each family's six accents differ only in their theme colour, so a family is
one line, `FAMILY<TAB>accent<TAB>XML`, its colour `{C}`; a plain style is the
template on `text1`, except the Colorful families', which Word writes
without the accent's shade (`FAMILY<TAB>plain<TAB>XML`). Every hex colour
Word resolved from a theme colour is `@`: the writer resolves it from the
written document's theme. The script checks that it resolves each one as
Word did for the source's own theme, to within 1 per channel.
"""

from __future__ import annotations

import colorsys
import os
import re
import sys
import tempfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SOURCE = ROOT / (
    "tests/corpus/neurotic_docx_bench/grok_run/fixtures_500/"
    "00f45b1b7812db57f5a024f116de6c6f0cfab5b79c9bc33657a6be57657d181d.docx"
)
OUT = ROOT / "src/markdown/word_table_styles.tsv"
FAMILY = re.compile(r"((?:Grid|List)Table\d(?:Light|Dark|Colorful)?)(?:-Accent(\d))?1$")
SLOTS = {
    "text1": "dk1", "dark1": "dk1", "background1": "lt1", "light1": "lt1",
    "text2": "dk2", "dark2": "dk2", "background2": "lt2", "light2": "lt2",
    "hyperlink": "hlink", "followedHyperlink": "folHlink",
}


def theme(package: zipfile.ZipFile) -> dict[str, str]:
    name = next(n for n in package.namelist() if re.fullmatch(r"word/theme/theme\d*\.xml", n))
    xml = package.read(name).decode()
    colours = {}
    for slot in ("dk1", "lt1", "dk2", "lt2", "accent1", "accent2", "accent3", "accent4",
                 "accent5", "accent6", "hlink", "folHlink"):
        found = re.search(rf"<a:{slot}>.*?(?:srgbClr val|lastClr)=\"([0-9A-Fa-f]{{6}})\"", xml, re.S)
        colours[slot] = found.group(1).upper()
    return colours


def resolve(colours: dict[str, str], name: str, tint: str | None, shade: str | None) -> str:
    rgb = colours[SLOTS.get(name, name)]
    r, g, b = (int(rgb[i:i + 2], 16) / 255 for i in (0, 2, 4))
    h, lum, s = colorsys.rgb_to_hls(r, g, b)
    if tint is not None:
        t = int(tint, 16) / 255
        lum = lum * t + (1 - t)
    if shade is not None:
        lum *= int(shade, 16) / 255
    return "".join(f"{int(v * 255):02X}" for v in colorsys.hls_to_rgb(h, lum, s))


def check(holds: bool, problem: object) -> None:
    """Stops the script when `holds` is false: a check `python -O` keeps."""
    if not holds:
        raise SystemExit(f"table_styles_asset: {problem}")


def close(a: str, b: str) -> bool:
    return all(abs(int(a[i:i + 2], 16) - int(b[i:i + 2], 16)) <= 1 for i in (0, 2, 4))


def templated(style: str, colours: dict[str, str]) -> str:
    """The style with each theme-resolved hex colour `@`, checked first."""

    def tag(match: re.Match[str]) -> str:
        text = match.group(0)
        attrs = dict(re.findall(r'w:(\w+)="([^"]*)"', text))
        for key, theme_key, tint, shade in (
            ("color", "themeColor", "themeTint", "themeShade"),
            ("val", "themeColor", "themeTint", "themeShade"),
            ("fill", "themeFill", "themeFillTint", "themeFillShade"),
        ):
            if theme_key not in attrs or not re.fullmatch(r"[0-9A-F]{6}", attrs.get(key, "")):
                continue
            want = resolve(colours, attrs[theme_key], attrs.get(tint), attrs.get(shade))
            check(close(want, attrs[key]), (text, want))
            text = text.replace(f'w:{key}="{attrs[key]}"', f'w:{key}="@"')
        return text

    return re.sub(r"<w:\w+ [^>]*/>", tag, style)


def main() -> None:
    source = Path(sys.argv[1]) if len(sys.argv) > 1 else SOURCE
    package = zipfile.ZipFile(source)
    colours = theme(package)
    styles = package.read("word/styles.xml").decode()
    families: dict[str, dict[str, str]] = {}
    for match in re.finditer(r'<w:style [^>]*w:type="table"[^>]*>.*?</w:style>', styles, re.S):
        style = match.group(0)
        found = FAMILY.match(re.search(r'w:styleId="([^"]+)"', style).group(1))
        if not found:
            continue
        family, accent = found.group(1), found.group(2)
        body = templated(style, colours)
        body = re.sub(r' w:rsid\w*="[^"]*"|<w:rsid [^>]*/>|<w:name [^>]*/>', "", body)
        body = re.sub(r'<w:style [^>]*>', "", body).removesuffix("</w:style>")
        colour = f"accent{accent}" if accent else "text1"
        families.setdefault(family, {})[accent or "0"] = body.replace(f'"{colour}"', '"{C}"')
    lines = []
    for family in sorted(families):
        variants = families[family]
        accents = {variants[a] for a in "123456"}
        check(len(accents) == 1, f"{family}: the accents differ beyond their colour")
        (template,) = accents
        lines.append(f"{family}\taccent\t{template}")
        if variants["0"] != template:
            lines.append(f"{family}\tplain\t{variants['0'].replace('{C}', 'text1')}")
    check(len(families) == 14, sorted(families))
    # A whole asset or none: the build reads it with `include_str!`.
    fd, staged = tempfile.mkstemp(dir=OUT.parent, prefix=f".{OUT.name}.")
    try:
        with os.fdopen(fd, "w", encoding="utf-8") as out:
            out.write("\n".join(lines) + "\n")
            out.flush()
            os.fsync(out.fileno())
        os.replace(staged, OUT)
    except BaseException:
        Path(staged).unlink(missing_ok=True)
        raise
    print(f"{OUT.relative_to(ROOT)}: {len(families)} families, {len(lines)} lines")


if __name__ == "__main__":
    main()
