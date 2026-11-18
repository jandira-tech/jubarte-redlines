#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
#
# Build one ReadTheDocs book with the pinned mdBook:
#
#   docs/rtd/build.sh PROJECT [OUT_DIR]
#
# PROJECT is a folder of docs/rtd/ (`jubarte`, `jubarte-dom`, ...). OUT_DIR
# defaults to .scratch/rtd/books/PROJECT; each project's .readthedocs.yaml
# passes "$READTHEDOCS_OUTPUT/html".
#
# mdBook is downloaded as the release's prebuilt binary, once per version
# and platform, into .scratch/rtd/mdbook-VERSION-TARGET/, and its archive is
# checked against the sha256 pinned below before it is unpacked or reused.
#
# The repository URL (the book's GitHub link and its "edit this page"
# link) is read from `[workspace.package] repository` in Cargo.toml, the
# one place it is written; no book.toml repeats it.
#
# A warning from mdBook fails the build.

set -euo pipefail
cd "$(dirname "$0")/../.."
ROOT=$PWD

MDBOOK_VERSION=0.5.4

die() {
  echo "docs/rtd/build.sh: $*" >&2
  exit 1
}

[ $# -ge 1 ] && [ $# -le 2 ] || die "usage: docs/rtd/build.sh PROJECT [OUT_DIR]"
project=$1
book=docs/rtd/$project
[ -f "$book/book.toml" ] || die "no book at $book (no book.toml)"
out=${2:-$ROOT/.scratch/rtd/books/$project}

# --- mdBook --------------------------------------------------------------------
case "$(uname -s)-$(uname -m)" in
  Linux-x86_64)
    target=x86_64-unknown-linux-gnu
    sha256=3f28de05dafca9d0f2eab99c662116b0e37b89b1d96a08f8f430b9eeae958cd7 ;;
  Linux-aarch64 | Linux-arm64)
    target=aarch64-unknown-linux-musl
    sha256=753e5c5c363ee8a56972344dcf91466f005a51db84a7aeffe427ae3ef83d6d44 ;;
  Darwin-arm64)
    target=aarch64-apple-darwin
    sha256=03e8a6d8b13a2971e0b3280affd03b388373c1485e26f73407c3a76b0b1838df ;;
  Darwin-x86_64)
    target=x86_64-apple-darwin
    sha256=a47d7bf0d5d670cff9ee6cce95537cbeb62dc10704d9e7131ffbd13e2b59a5de ;;
  *) die "no pinned mdBook $MDBOOK_VERSION build for $(uname -s)-$(uname -m)" ;;
esac

sha256_of() {
  if command -v sha256sum >/dev/null; then
    sha256sum "$1" | cut -d' ' -f1
  else
    shasum -a 256 "$1" | cut -d' ' -f1
  fi
}

tool_dir=$ROOT/.scratch/rtd/mdbook-$MDBOOK_VERSION-$target
archive=$tool_dir/mdbook-v$MDBOOK_VERSION-$target.tar.gz
mdbook=$tool_dir/mdbook
if [ ! -f "$archive" ]; then
  mkdir -p "$tool_dir"
  url=https://github.com/rust-lang/mdBook/releases/download/v$MDBOOK_VERSION/mdbook-v$MDBOOK_VERSION-$target.tar.gz
  echo "docs/rtd/build.sh: downloading $url" >&2
  curl --proto '=https' --tlsv1.2 -sSfL --retry 3 -o "$archive.part" "$url"
  mv "$archive.part" "$archive"
fi
actual=$(sha256_of "$archive")
if [ "$actual" != "$sha256" ]; then
  rm -f "$archive" "$mdbook"
  die "mdBook archive sha256 $actual is not the pinned $sha256"
fi
# Unpack from the verified archive every time: a binary left in the folder
# is never trusted on its own.
tar -xzf "$archive" -C "$tool_dir" mdbook
"$mdbook" --version | grep -qx "mdbook v$MDBOOK_VERSION" || die "unexpected $("$mdbook" --version)"

# --- Repository URL ----------------------------------------------------------
repository=$(sed -n '/^\[workspace\.package\]/,/^\[/{
s/^repository[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p
}' Cargo.toml)
case "$repository" in
  https://*) ;;
  *) die "no [workspace.package] repository URL in Cargo.toml" ;;
esac
export MDBOOK_OUTPUT__HTML__GIT_REPOSITORY_URL=$repository
export MDBOOK_OUTPUT__HTML__EDIT_URL_TEMPLATE=$repository/edit/main/$book/{path}

# --- Build ---------------------------------------------------------------------
mkdir -p "$out"
out=$(cd "$out" && pwd)
log=$(mktemp)
trap 'rm -f "$log"' EXIT
if ! "$mdbook" build "$book" --dest-dir "$out" 2>"$log"; then
  cat "$log" >&2
  die "mdBook failed for $project"
fi
cat "$log" >&2
if grep -Eq '(^|[^A-Za-z])(WARN|ERROR)([^A-Za-z]|$)' "$log"; then
  die "mdBook warned while building $project; fix the warning"
fi
echo "docs/rtd/build.sh: $project -> $out"
