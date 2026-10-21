# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""The hand-written ``_native.pyi`` stub declares every native function."""

import re
from pathlib import Path

from jubarte_redlines import _native

STUB = Path(__file__).parents[1] / "python" / "jubarte_redlines" / "_native.pyi"


def test_the_stub_declares_every_native_function() -> None:
    native = {name for name in dir(_native) if not name.startswith("_") and callable(getattr(_native, name))}
    declared = set(re.findall(r"^(?:def|class) (\w+)", STUB.read_text(encoding="utf-8"), re.M))
    assert native - declared == set(), "missing from _native.pyi"
    assert declared - native == set(), "declared in _native.pyi but not native"
