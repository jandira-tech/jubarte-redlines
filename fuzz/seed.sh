#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
#
# Seed every target's corpus with the repository's .docx fixtures (files under
# 2 MiB, de-duplicated by content). `compare` gets pairs: a 4-byte little
# endian length of the first document, then both documents.
set -euo pipefail
cd "$(dirname "$0")/.."
for t in admit strict_to_transitional compare relationships; do mkdir -p "fuzz/corpus/$t"; done
find tests -name '*.docx' -size -2048k | sort | while read -r f; do
  h=$(sha1sum "$f" | cut -c1-16)
  cp -n "$f" "fuzz/corpus/admit/$h" || true
  cp -n "$f" "fuzz/corpus/strict_to_transitional/$h" || true
done
python3 - <<'PY'
import hashlib, pathlib, struct
root = pathlib.Path("tests")
docs = sorted(p for p in root.rglob("*.docx") if p.stat().st_size < 256 * 1024)[:40]
out = pathlib.Path("fuzz/corpus/compare")
for a, b in zip(docs, docs[1:]):
    da, db = a.read_bytes(), b.read_bytes()
    blob = struct.pack("<I", len(da)) + da + db
    (out / hashlib.sha1(blob).hexdigest()[:16]).write_bytes(blob)
# Direct-parser regressions, including a late duplicate after a large tag.
rels = pathlib.Path("fuzz/corpus/relationships")
prefix = b'<Relationships><Relationship Id="rId1" Type="image" Target="media/x.png"'
attrs = b''.join(f' a{i}="x"'.encode() for i in range(16384))
for name, extra in [('large', attrs), ('duplicate', attrs + b' a0="again"')]:
    (rels / name).write_bytes(prefix + extra + b'/></Relationships>')
PY
