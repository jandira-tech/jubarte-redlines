# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only

"""What the installed `jubarte-docx` distribution ships.

The import package is `jubarte` and the command is `jubarte`; the PyPI
distribution is `jubarte-docx`, as `python-dotenv` ships `dotenv`, because
PyPI's `jubarte` belongs to a third party. `jubarte-redlines` is a separate
pure-Python shim (`jubarte-python/compat`, tested in test_compat_shim.py)
that owns `jubarte_redlines/` and the `jubarte-redlines` command, so no
file belongs to two of our distributions.
"""

import subprocess
import sys
import re
from importlib import metadata
from pathlib import Path

import pytest

DIST = "jubarte-docx"


def test_imports_jubarte_with_compare_documents():
    import jubarte

    assert callable(jubarte.compare_documents)


def test_the_native_module_lives_under_jubarte():
    from jubarte import _native

    assert _native.__name__ == "jubarte._native"


@pytest.mark.integration
def test_module_help_exits_zero():
    run = subprocess.run(
        [sys.executable, "-m", "jubarte", "--help"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert run.returncode == 0, run.stderr
    assert "compare" in run.stdout
    assert "Usage: jubarte " in run.stdout, run.stdout


def test_ships_the_jubarte_package_and_not_the_shims():
    # maturin packs python-source; an editable install's RECORD names only
    # a .pth, so check the tree it packs as well as what was installed.
    root = Path(__file__).resolve().parent.parent
    config = (root / "pyproject.toml").read_text()
    source = root / re.search(r'^python-source = "([^"]+)"', config, re.M).group(1)
    assert re.search(r'^module-name = "jubarte\._native"$', config, re.M), "the native module is jubarte._native"
    assert sorted(p.name for p in source.iterdir() if p.is_dir()) == ["jubarte"]
    files = metadata.distribution(DIST).files
    assert files is not None, f"{DIST} has no RECORD: its files cannot be checked"
    tops = {str(f).split("/")[0] for f in files}
    assert "jubarte_redlines" not in tops, f"{DIST} must leave jubarte_redlines/ to the shim: {tops}"


def test_console_scripts_are_jubarte_and_jubarte_mcp():
    scripts = {ep.name: ep.value for ep in metadata.distribution(DIST).entry_points if ep.group == "console_scripts"}
    assert scripts == {
        "jubarte": "jubarte.__main__:main",
        "jubarte-mcp": "jubarte.mcp_server:main",
    }
