#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""Shared pytest fixtures for the held-out pilot's harness tests."""

from __future__ import annotations

import os
import subprocess
from pathlib import Path

import pytest

HERE = Path(__file__).resolve().parent
PKG_ROOT = HERE.parent
REPO_ROOT = PKG_ROOT.parent.parent

DEFAULT_BINARY_PATH = REPO_ROOT / "target" / "debug" / "jubarte"


def _resolve_binary() -> Path | None:
    candidate = os.environ.get("JUBARTE_CLI")
    if candidate:
        p = Path(candidate).resolve()
        if p.is_file():
            return p
    if DEFAULT_BINARY_PATH.is_file():
        return DEFAULT_BINARY_PATH
    return None


JUBARTE_CLI = _resolve_binary()


def _cargo_seq() -> Path:
    return Path("/tmp/cargo-seq")


def _ensure_binary() -> Path:
    if JUBARTE_CLI is not None:
        return JUBARTE_CLI
    if not _cargo_seq().is_file():
        pytest.skip("jubarte binary not built and /tmp/cargo-seq missing")
    cwd = REPO_ROOT
    result = subprocess.run(
        [str(_cargo_seq()), "build", "--bin", "jubarte"],
        cwd=cwd,
        capture_output=True,
        text=True,
        timeout=600,
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"cargo-seq build failed: {result.returncode}\n{result.stderr}"
        )
    if not DEFAULT_BINARY_PATH.is_file():
        raise RuntimeError("build did not produce target/debug/jubarte")
    return DEFAULT_BINARY_PATH


@pytest.fixture(scope="session")
def jubarte_binary() -> Path:
    return _ensure_binary()


@pytest.fixture()
def repo_root() -> Path:
    return REPO_ROOT


@pytest.fixture()
def tasks_dir() -> Path:
    return PKG_ROOT / "tasks"


@pytest.fixture()
def harness_pkg() -> Path:
    return PKG_ROOT / "harness"


@pytest.hookimpl(trylast=True)
def pytest_terminal_summary(terminalreporter):
    """Emit the required independent line and branch percentages."""
    import io
    import json
    from contextlib import redirect_stdout

    plugin = terminalreporter.config.pluginmanager.getplugin("_cov")
    if plugin and plugin.cov_controller and plugin.cov_controller.cov:
        buf = io.StringIO()
        with redirect_stdout(buf):
            plugin.cov_controller.cov.json_report(outfile="-")
        totals = json.loads(buf.getvalue())["totals"]
        lines = 100 * totals["covered_lines"] / max(totals["num_statements"], 1)
        branches = (
            100
            * totals.get("covered_branches", 0)
            / max(totals.get("num_branches", 0), 1)
        )
        terminalreporter.write_line(
            f"Coverage: {lines:.2f}% lines, {branches:.2f}% branches"
        )
