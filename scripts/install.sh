#!/usr/bin/env bash

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

# Installs the jubarte CLI and its fonts. The fonts are not embedded in the
# binary: they go to jubarte's per-user font folder, which `jubarte convert`
# searches after the system and Word font folders.
#
#   scripts/install.sh              binary (cargo install) + fonts
#   scripts/install.sh --fonts-only fonts only
#
# Font folder: $JUBARTE_FONT_DIR, else
#   macOS   ~/Library/Application Support/jubarte/fonts
#   Linux   $XDG_DATA_HOME/jubarte/fonts (default ~/.local/share/jubarte/fonts)
set -euo pipefail

repo="$(cd "$(dirname "$0")/.." && pwd)"

if [[ "${1:-}" != "--fonts-only" ]]; then
  cargo install --path "$repo" --features cli --locked --force
fi

if [[ -n "${JUBARTE_FONT_DIR:-}" ]]; then
  dest="$JUBARTE_FONT_DIR"
elif [[ "$(uname -s)" == "Darwin" ]]; then
  dest="$HOME/Library/Application Support/jubarte/fonts"
else
  dest="${XDG_DATA_HOME:-$HOME/.local/share}/jubarte/fonts"
fi

mkdir -p "$dest"
cp "$repo"/assets/fonts/extra/*.ttf "$dest"/
cp "$repo"/LICENSES/OFL-1.1.txt "$repo"/LICENSES/Apache-2.0.txt "$dest"/
echo "fonts installed in $dest"
