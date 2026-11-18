# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only

"""The `jubarte-redlines` shim (`jubarte-python/compat`).

It owns `jubarte_redlines/` and the `jubarte-redlines` command, depends on
`jubarte-docx` at the engine's own version, and aliases `jubarte`'s objects
rather than copying them, so old code sees the same classes as new code.
"""

import os
import shutil
import subprocess
import sys
import re
import zipfile
from email.parser import Parser
from pathlib import Path

import pytest

COMPAT = Path(__file__).resolve().parent.parent / "compat"
ENGINE = Path(__file__).resolve().parent.parent / "Cargo.toml"


def engine_version() -> str:
    return re.search(r'^version = "([^"]+)"', ENGINE.read_text(), re.M).group(1)


@pytest.fixture(scope="module")
def wheel(tmp_path_factory: pytest.TempPathFactory) -> zipfile.ZipFile:
    uv = shutil.which("uv")
    if uv is None:
        pytest.skip("uv builds the shim")
    out = tmp_path_factory.mktemp("shim")
    subprocess.run([uv, "build", "--wheel", "--quiet", str(COMPAT), "-o", str(out)], check=True)
    (built,) = out.glob("jubarte_redlines-*.whl")
    return zipfile.ZipFile(built)


def metadata(wheel: zipfile.ZipFile):
    (name,) = [n for n in wheel.namelist() if n.endswith(".dist-info/METADATA")]
    return Parser().parsestr(wheel.read(name).decode())


@pytest.mark.integration
def test_the_shim_ships_only_jubarte_redlines(wheel: zipfile.ZipFile) -> None:
    tops = {n.split("/")[0] for n in wheel.namelist() if not n.split("/")[0].endswith(".dist-info")}
    assert tops == {"jubarte_redlines"}, tops


@pytest.mark.integration
def test_the_shim_pins_jubarte_docx_at_the_engine_version(wheel: zipfile.ZipFile) -> None:
    meta = metadata(wheel)
    version = engine_version()
    assert meta["Version"] == version
    requires = [r.replace(" ;", ";") for r in meta.get_all("Requires-Dist")]
    assert f"jubarte-docx=={version}" in requires, requires
    assert f"jubarte-docx[mcp]=={version}; extra == 'mcp'" in requires, requires


@pytest.mark.integration
def test_the_shim_keeps_the_jubarte_redlines_command(wheel: zipfile.ZipFile) -> None:
    (name,) = [n for n in wheel.namelist() if n.endswith(".dist-info/entry_points.txt")]
    text = wheel.read(name).decode()
    assert "jubarte-redlines = jubarte.__main__:main" in text, text
    assert "jubarte-mcp" not in text, "jubarte-docx owns jubarte-mcp"


def run_with_shim(code: str) -> subprocess.CompletedProcess[str]:
    env = {**os.environ, "PYTHONPATH": os.pathsep.join([str(COMPAT), os.environ.get("PYTHONPATH", "")])}
    return subprocess.run([sys.executable, "-c", code], capture_output=True, text=True, env=env, check=False)


def test_old_imports_are_jubarte_objects() -> None:
    run = run_with_shim(
        "import jubarte, jubarte_redlines\n"
        "from jubarte_redlines import models, document, _native\n"
        "import jubarte.models, jubarte.document\n"
        "assert jubarte_redlines.compare_documents is jubarte.compare_documents\n"
        "assert jubarte_redlines.Document is jubarte.Document\n"
        "assert jubarte_redlines.JubarteError is jubarte.JubarteError\n"
        "assert models is jubarte.models and document is jubarte.document\n"
        "assert _native is jubarte._native\n"
        "import jubarte_redlines.models as m; assert m is jubarte.models\n"
        "assert jubarte_redlines.__version__ == jubarte.__version__\n"
        "assert sorted(jubarte_redlines.__all__) == sorted(jubarte.__all__)\n"
    )
    assert run.returncode == 0, run.stderr


@pytest.mark.integration
def test_python_m_jubarte_redlines_still_runs() -> None:
    env = {**os.environ, "PYTHONPATH": os.pathsep.join([str(COMPAT), os.environ.get("PYTHONPATH", "")])}
    run = subprocess.run(
        [sys.executable, "-m", "jubarte_redlines", "--help"], capture_output=True, text=True, env=env, check=False
    )
    assert run.returncode == 0, run.stderr
    assert "compare" in run.stdout
