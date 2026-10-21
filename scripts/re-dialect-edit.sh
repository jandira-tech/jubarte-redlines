#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
#
# re-dialect-edit.sh: write and edit the #1135 fixtures; fail when a
# command fails or the view read back differs from the settled syntax.
#
# 1. Round trip: `jubarte convert F.md -o F.docx`, then `jubarte F.docx`.
#    The body read back must equal F.md's body.
# 2. Edits: each `edit` line below runs one `jubarte edit` on a fixture as
#    Carol Diaz (@CD) at a pinned time, then reads the result. Its body must
#    equal tests/fixtures/re-dialect/edits/NAME.md.
#
# Only the body is compared (everything after the YAML header): the header
# names the file and its sha256, which change with every write. JUBARTE
# names the binary (default: target/debug/jubarte). Exit 1 on any difference.
set -u
root=$(cd "$(dirname "$0")/.." && pwd)
jubarte=${JUBARTE:-$root/target/debug/jubarte}
fixtures=$root/tests/fixtures/re-dialect
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
pass=0
fail=0

body() { awk 'n >= 2 { print } /^---$/ { n++ }' "$1"; }

check() { # NAME EXPECTED_BODY GOT_VIEW ERR COMMAND STATUS
  [ -f "$3" ] && body "$3" > "$3.body"
  if [ "$6" -eq 0 ] && [ -s "$3" ] && diff -u --label "expected $1" --label "$5" "$2" "$3.body" > "$3.diff"; then
    echo "ok    $5"
    pass=$((pass + 1))
  else
    echo "FAIL  $5"
    [ -f "$3.diff" ] && cat "$3.diff"
    [ -s "$4" ] && sed 's/^/  stderr: /' "$4"
    fail=$((fail + 1))
  fi
}

echo "== round trip: md -> docx -> md"
for expected in "$fixtures"/*.md; do
  name=$(basename "$expected" .md)
  [ -f "$fixtures/$name.docx" ] || continue
  dir=$work/rt-$name
  mkdir -p "$dir"
  cp "$expected" "$dir/$name.md"
  (
    cd "$dir" || exit 1
    "$jubarte" -q convert "$name.md" -o "$name.docx" 2> convert.err &&
      "$jubarte" "$name.docx" > back.md 2>> convert.err
  )
  status=$?
  body "$expected" > "$dir/expected.body"
  check "$name.md" "$dir/expected.body" "$dir/back.md" "$dir/convert.err" "jubarte convert $name.md -o $name.docx; jubarte $name.docx" "$status"
done

echo "== edits as @CD"
edit() { # NAME FIXTURE EDIT_ARGS...
  local name=$1 fixture=$2
  shift 2
  local dir=$work/edit-$name
  mkdir -p "$dir"
  cp "$fixtures/$fixture.docx" "$dir/"
  (
    cd "$dir" || exit 1
    "$jubarte" -q edit "$fixture.docx" "$@" -a "Carol Diaz" -d 2026-10-18T10:00:00Z -o out.docx > edit.out 2> edit.err &&
      "$jubarte" out.docx > out.md 2>> edit.err
  )
  local status=$?
  local shown
  shown=$(printf '%q ' "$@")
  check "edits/$name.md" "$fixtures/edits/$name.md" "$dir/out.md" "$dir/edit.err" "jubarte edit $fixture.docx ${shown% }" "$status"
}

edit modify-inside-insertion step2-tracked -p p0 --anchor their --content its
edit add-inside-insertion step2-tracked -p p0 --anchor month. --content "month, in cash."
edit modify-substitution step2-tracked -p p0 --anchor insurers --content underwriters
edit reject-substitution step2-tracked -p r4+5 --reject
edit accept-substitution step2-tracked -p r4+5 --accept
edit modify-plain-text step2-untracked -p p0 --anchor insurers --content underwriters
edit reject-nested-deletion annex -p r4 --reject

echo "edit: $pass ok, $fail failed"
[ "$fail" -eq 0 ]
