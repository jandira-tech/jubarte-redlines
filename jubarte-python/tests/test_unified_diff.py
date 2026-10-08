# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only

"""Memory-only regressions; RED starts at the existing unsupported format."""
import pytest

import jubarte_redlines as jubarte
from docx_fixture import docx, para


@pytest.mark.parametrize("format", ["github", "unified", "text"])
def test_git_patch_aliases_keep_complete_lines_and_all_hunks(format):
    old = "".join(f"old {i} {'é' * 300}\nseparator {i}\n" for i in range(8))
    new = old.replace("old ", "new ")
    result = jubarte.diff(old, new, format=format, context=0)
    assert result.hunks == ()
    assert result.text.startswith("diff --git a/old.md b/new.md\n--- a/old.md\n+++ b/new.md\n")
    assert result.text.count("@@ -") == 8
    for i in range(8):
        assert f"-old {i} {'é' * 300}\n+new {i} {'é' * 300}\n" in result.text
    assert "…" not in result.text


def test_context_and_identical_documents():
    result = jubarte.diff("Intro\nOld\nEnd\n", "Intro\nNew\nEnd\n", format="github", context=1)
    assert "@@ -1,3 +1,3 @@\n Intro\n-Old\n+New\n End\n" in result.text
    assert jubarte.diff("same\n", "same\n", format="github").text == ""
    document = jubarte.from_markdown("Due in {~~30~>45~~} days.\n")
    assert document.diff(document, format="github").text == ""


def test_docx_preserves_existing_marks_and_header_story():
    old = jubarte.from_markdown("Due in {~~30~>45~~} days.\n")
    new = jubarte.from_markdown("Due in {~~30~>60~~} days.\n")
    patch = old.diff(new, format="github", context=0).text
    assert "[-30-]{+45+}" in patch and "[-30-]{+60+}" in patch
    patch = jubarte.diff(docx(para("same"), header="OLD HEADER"), docx(para("same"), header="NEW HEADER"), format="github").text
    assert "OLD HEADER" in patch and "NEW HEADER" in patch


@pytest.mark.parametrize("context", [True, False, -1, 1.5, 2**32, 2**100, "3", None])
def test_invalid_context_is_rejected_before_diffing(context):
    with pytest.raises(ValueError, match="context"):
        jubarte.diff("a\n", "b\n", format="github", context=context)


def test_maximum_u32_context_and_native_labels():
    assert " a\n" in jubarte.diff("a\nb\n", "a\nc\n", format="github", context=2**32 - 1).text
    patch = jubarte._native.diff_unified("old\n", "new\n", old_name="before.md", new_name="after.md", context=0)
    assert "--- a/before.md\n+++ b/after.md\n" in patch


@pytest.mark.parametrize("context", [True, False, -1, 1.5, 2**32, 2**100, "3", None])
def test_native_context_cannot_bypass_validation(context):
    with pytest.raises(ValueError, match="context"):
        jubarte._native.diff_unified("a\n", "b\n", context=context)
