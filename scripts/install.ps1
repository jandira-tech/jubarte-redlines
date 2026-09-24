# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

# Installs the jubarte CLI and its fonts on Windows. The fonts are not
# embedded in the binary: they go to %APPDATA%\jubarte\fonts (or
# $env:JUBARTE_FONT_DIR), which `jubarte convert` searches.
#
#   scripts\install.ps1              binary (cargo install) + fonts
#   scripts\install.ps1 -FontsOnly   fonts only
param([switch]$FontsOnly)
$ErrorActionPreference = "Stop"

$repo = Split-Path -Parent $PSScriptRoot

if (-not $FontsOnly) {
  cargo install --path $repo --features cli --locked --force
  if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

$dest = if ($env:JUBARTE_FONT_DIR) { $env:JUBARTE_FONT_DIR } else { Join-Path $env:APPDATA "jubarte\fonts" }
New-Item -ItemType Directory -Force -Path $dest | Out-Null
Copy-Item (Join-Path $repo "assets\fonts\extra\*.ttf") $dest
Copy-Item (Join-Path $repo "LICENSES\OFL-1.1.txt"), (Join-Path $repo "LICENSES\Apache-2.0.txt") $dest
Write-Output "fonts installed in $dest"
