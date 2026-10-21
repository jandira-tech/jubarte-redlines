#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Make the #1135 fixtures from Word-made files, keeping Word's XML.

    python3 -I make_fixtures.py OUT_DIR NAME=SOURCE.docx [NAME=SOURCE.docx ...]

Each SOURCE.docx was saved by Word 16 for Mac. Its parts are copied byte
for byte, in the same order and with the same compression, except:

- every author name is replaced by a synthetic one (AUTHORS below), in the
  body, the people part and the core properties;
- each w15:presenceInfo gets providerId "None" and the synthetic name as its
  userId, so no account id from the source file survives.

The script refuses a source that still names an author it has no
replacement for, and an output that still holds a source name.
"""
import re
import sys
import zipfile

# Source author -> synthetic author. The initials stay the same, so the
# view's handles (@AR, @ASR, @B) match the source files'.
AUTHORS = {
    "Arthur Souza Rodrigues": "Alma Sato Reyes",
    "Arthur Rodrigues": "Alma Reyes",
    "Bob": "Bob",
}

PRESENCE = re.compile(r'<w15:presenceInfo w15:providerId="[^"]*" w15:userId="[^"]*"\s*/>')


def rewrite(name: str, data: bytes) -> bytes:
    if not name.endswith(".xml") and not name.endswith(".rels"):
        return data
    text = data.decode("utf-8")
    # Longest names first: "Arthur Souza Rodrigues" before "Arthur Rodrigues".
    for old in sorted(AUTHORS, key=len, reverse=True):
        text = text.replace(old, AUTHORS[old])
    if name == "word/people.xml":
        def presence(person_match: re.Match) -> str:
            author = person_match.group(1)
            return (
                f'<w15:person w15:author="{author}">'
                f'<w15:presenceInfo w15:providerId="None" w15:userId="{author}"/>'
            )
        text = re.sub(
            r'<w15:person w15:author="([^"]*)">\s*' + PRESENCE.pattern,
            presence,
            text,
        )
    return text.encode("utf-8")


def authors_in(data: bytes) -> set[str]:
    text = data.decode("utf-8", "replace")
    names = set(re.findall(r'w(?:15)?:author="([^"]*)"', text))
    names |= set(re.findall(r"<(?:dc:creator|cp:lastModifiedBy)>([^<]*)<", text))
    return {n for n in names if n}


def make(source: str, target: str) -> None:
    with zipfile.ZipFile(source) as zin:
        seen = set()
        for info in zin.infolist():
            seen |= authors_in(zin.read(info))
        unknown = seen - set(AUTHORS) - set(AUTHORS.values())
        if unknown:
            sys.exit(f"{source}: no synthetic name for {sorted(unknown)}")
        with zipfile.ZipFile(target, "w") as zout:
            for info in zin.infolist():
                data = rewrite(info.filename, zin.read(info))
                out = zipfile.ZipInfo(info.filename, date_time=(1980, 1, 1, 0, 0, 0))
                out.compress_type = info.compress_type
                out.external_attr = info.external_attr
                zout.writestr(out, data)
    with zipfile.ZipFile(target) as check:
        for info in check.infolist():
            text = check.read(info).decode("utf-8", "replace")
            for old, new in AUTHORS.items():
                if old != new and old in text:
                    sys.exit(f"{target}: {info.filename} still names {old!r}")
            if info.filename == "word/people.xml" and 'providerId="None"' not in text:
                sys.exit(f"{target}: people.xml kept a provider id")


def main() -> None:
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    out_dir = sys.argv[1]
    for spec in sys.argv[2:]:
        name, _, source = spec.partition("=")
        make(source, f"{out_dir}/{name}.docx")
        print(f"wrote {out_dir}/{name}.docx")


if __name__ == "__main__":
    main()
