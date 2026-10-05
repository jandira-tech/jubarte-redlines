# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""asc_loader.load_module_from_path loads a script the way an import would.

Run with:
    uv run --with pytest pytest scripts/test_asc_loader.py -q

Covers CodeRabbit #4156672352 on PR #20: a loaded module must sit in
sys.modules while it runs, or a dataclass it defines under postponed
annotations cannot resolve its own module.
"""

import sys
from pathlib import Path

from asc_loader import load_module_from_path

SCRIPTS = Path(__file__).resolve().parent


DATACLASS_MODULE = """\
from __future__ import annotations
from dataclasses import dataclass

@dataclass
class Build:
    version: str
    processed: bool = False
"""


def test_a_loaded_module_can_define_dataclasses(tmp_path: Path) -> None:
    path = tmp_path / "with-dataclass.py"
    path.write_text(DATACLASS_MODULE)
    mod = load_module_from_path("asc_loader_dataclass_probe", path)
    assert mod.Build("0.10.1").version == "0.10.1"
    assert sys.modules["asc_loader_dataclass_probe"] is mod
