#!/usr/bin/env bash
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only
set -euo pipefail
cd "$(dirname "$0")/.."
command -v cargo-deny >/dev/null
command -v gitleaks >/dev/null
# Standalone workspaces: the root graph alone misses binding dependencies.
for manifest in Cargo.toml jubarte-python/Cargo.toml jubarte-wasm/Cargo.toml jubarte-rust-inproc/Cargo.toml; do
  cargo deny --manifest-path "$manifest" check advisories
done
# Full reachable history plus local changes; never print credential values.
gitleaks git --log-opts="--all" --redact --no-banner
# Scan only files eligible for publication, excluding ignored build artifacts.
# Gitleaks dir does not honor .gitignore, so snapshot git's file inventory.
python3 - <<'SCAN'
from pathlib import Path
import shutil
import subprocess
import tempfile

names = subprocess.check_output([
    "git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"
]).split(b"\0")
with tempfile.TemporaryDirectory(prefix="jubarte-secret-scan-") as scratch:
    for name in names:
        if not name:
            continue
        source = Path(name.decode("utf-8", "surrogateescape"))
        if source.is_symlink() or not source.is_file():
            continue
        destination = Path(scratch) / source
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source, destination)
    subprocess.run(["gitleaks", "dir", "--redact", "--no-banner", scratch], check=True)
SCAN
