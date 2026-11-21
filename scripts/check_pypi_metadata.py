#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: Apache-2.0

"""Refuse a Python distribution whose PyPI page would come out bare.

PyPI builds a project's page from the metadata of the release it is shown
for: the summary, the long description, the author, the links and the
classifiers come from `[project]` in pyproject.toml, and a release cannot be
changed once it is uploaded. `python3 scripts/check_pypi_metadata.py` reads
the two distributions this repository publishes (jubarte-docx and its
jubarte-redlines shim) and names every field that is missing or empty.
scripts/release.sh runs it before the publish dry-runs.

Exit 0 when both are complete; 1 naming what is missing. Plain Python 3
(3.11 or later, for tomllib), stdlib only.
"""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PROJECTS = ("jubarte-python/pyproject.toml", "jubarte-python/compat/pyproject.toml")
# the links PyPI lists in the sidebar
URLS = ("Homepage", "Documentation", "Repository", "Changelog", "Issues")
# the classifier groups PyPI's search filters on
CLASSIFIER_GROUPS = (
    "Development Status :: ",
    "Intended Audience :: ",
    "Operating System :: ",
    "Programming Language :: Python :: 3.",
    "Topic :: ",
)


def problems(pyproject: Path) -> list[str]:
    """What PyPI would show empty for the distribution PYPROJECT describes."""
    try:
        project = tomllib.loads(pyproject.read_text()).get("project", {})
    except (OSError, tomllib.TOMLDecodeError) as error:
        return [f"cannot be read: {error}"]
    plain = ("name", "description", "readme", "requires-python", "license")
    found = [f"`{field}` is missing" for field in plain if not project.get(field)]
    readme = project.get("readme")
    if isinstance(readme, str) and not (pyproject.parent / readme).is_file():
        found.append(f"readme `{readme}` is no file beside it")
    for field in ("authors", "maintainers"):
        people = project.get(field) or []
        if not people:
            found.append(f"`{field}` is missing")
        elif not all(person.get("name") and person.get("email") for person in people):
            found.append(f"every entry of `{field}` needs a name and an email")
    if len(project.get("keywords") or []) < 3:
        found.append("`keywords` needs at least three")
    classifiers = project.get("classifiers") or []
    found += [
        f"no classifier starts with `{group}`"
        for group in CLASSIFIER_GROUPS
        if not any(classifier.startswith(group) for classifier in classifiers)
    ]
    urls = project.get("urls") or {}
    found += [
        f"`[project.urls]` has no https `{label}`"
        for label in URLS
        if not str(urls.get(label, "")).startswith("https://")
    ]
    return found


def main(argv: list[str]) -> int:
    paths = [Path(arg) for arg in argv] or [ROOT / name for name in PROJECTS]
    bad = False
    for path in paths:
        for problem in problems(path):
            bad = True
            print(f"{path}: {problem}", file=sys.stderr)
    if bad:
        print(
            "PyPI shows these fields as they are uploaded and a release cannot be edited: fill them in first",
            file=sys.stderr,
        )
        return 1
    print(f"PyPI metadata complete: {', '.join(str(p) for p in paths)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
