# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only

"""What the installed `jubarte-redlines` distribution ships (plan T5.3).

PyPI's `jubarte` belongs to a third party (a study-topic CLI whose wheel
ships a `jubarte/` package and a `jubarte` console script), so this
distribution ships neither: `import jubarte` and the `jubarte` command
stay theirs. The import package is `jubarte_redlines`, run as
`python -m jubarte_redlines` or the `jubarte-redlines` script.
"""
import subprocess
import sys
from importlib import metadata

import pytest

DIST = "jubarte-redlines"


def test_imports_jubarte_redlines_with_compare_documents():
    import jubarte_redlines

    assert jubarte_redlines.compare_documents is not None
    assert callable(jubarte_redlines.compare_documents)


@pytest.mark.integration
def test_module_help_exits_zero():
    run = subprocess.run(
        [sys.executable, "-m", "jubarte_redlines", "--help"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert run.returncode == 0, run.stderr
    assert "compare" in run.stdout


def test_ships_no_jubarte_import_package():
    files = metadata.distribution(DIST).files
    assert files is not None, f"{DIST} has no RECORD: its files cannot be checked"
    clashing = [str(f) for f in files if str(f).split("/")[0] == "jubarte"]
    assert clashing == [], f"{DIST} would overwrite the third-party `jubarte` package: {clashing}"


def test_console_scripts_do_not_shadow_the_third_party_jubarte_command():
    scripts = {
        ep.name: ep.value
        for ep in metadata.distribution(DIST).entry_points
        if ep.group == "console_scripts"
    }
    assert "jubarte" not in scripts, scripts
    assert scripts["jubarte-redlines"] == "jubarte_redlines.__main__:main"
    assert scripts["jubarte-mcp"] == "jubarte_redlines.mcp_server:main"
