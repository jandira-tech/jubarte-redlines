#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
#
# re-dialect-read.sh: read each #1135 fixture three ways; fail when a
# command fails or its output differs from the view written beside it in the settled syntax:
#
#   jubarte F.docx > a.md           the agent view on stdout
#   jubarte F.docx -o b.md
#   jubarte convert F.docx -o c.md
#
# tests/fixtures/re-dialect/F.md is the expected view of F.docx. JUBARTE
# names the binary (default: target/debug/jubarte). The diff prints the
# expected view (-) against jubarte's (+). Exit 1 on failure or difference.
set -u
root=$(cd "$(dirname "$0")/.." && pwd)
jubarte=${JUBARTE:-$root/target/debug/jubarte}
fixtures=$root/tests/fixtures/re-dialect
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
pass=0
fail=0
for expected in "$fixtures"/*.md; do
  name=$(basename "$expected" .md)
  [ -f "$fixtures/$name.docx" ] || continue
  dir=$work/$name
  mkdir -p "$dir"
  cp "$fixtures/$name.docx" "$dir/"
  (
    cd "$dir" || exit 1
    "$jubarte" "$name.docx" > a.md 2> a.err
    echo $? > a.status
    "$jubarte" -q "$name.docx" -o b.md 2> b.err
    echo $? > b.status
    "$jubarte" -q convert "$name.docx" -o c.md 2> c.err
    echo $? > c.status
  )
  for out in a b c; do
    case $out in
      a) cmd="jubarte $name.docx > a.md" ;;
      b) cmd="jubarte $name.docx -o b.md" ;;
      c) cmd="jubarte convert $name.docx -o c.md" ;;
    esac
    if [ "$(cat "$dir/$out.status")" -eq 0 ] && [ -f "$dir/$out.md" ] && diff -u --label "expected $name.md" --label "$cmd" "$expected" "$dir/$out.md" > "$dir/$out.diff"; then
      echo "ok    $cmd"
      pass=$((pass + 1))
    else
      echo "FAIL  $cmd (exit $(cat "$dir/$out.status"))"
      [ -f "$dir/$out.diff" ] && cat "$dir/$out.diff"
      [ -s "$dir/$out.err" ] && sed 's/^/  stderr: /' "$dir/$out.err"
      fail=$((fail + 1))
    fi
  done
done
echo "read: $pass ok, $fail failed"
[ "$fail" -eq 0 ]
