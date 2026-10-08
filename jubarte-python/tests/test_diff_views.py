# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only

"""Pure adapter contracts and explicitly marked host CLI integration."""
import pytest

import jubarte_redlines as jubarte
from jubarte_redlines.__main__ import build_parser, main

VIEWS = ("github", "unified", "text", "word", "normal", "context", "side-by-side")


@pytest.mark.parametrize("format", VIEWS)
def test_views_return_text_without_paragraph_hunks(format):
    result = jubarte.diff("Due in 30 days.\n", "Due in 45 days.\n", format=format, context=0)
    assert result.hunks == ()
    assert "30" in result.text and "45" in result.text
    if format == "word":
        assert "{~~30~>45~~}" in result.text
    document = jubarte.from_markdown("Due in 30 days.\n")
    assert document.diff("Due in 45 days.\n", format=format, full_lines=True).hunks == ()


@pytest.mark.parametrize("format", ("github", "normal", "context", "side-by-side"))
def test_accept_changes_removes_historical_differences(format):
    old = jubarte.from_markdown("Due in {~~30~>45~~} days.\n")
    new = jubarte.from_markdown("Due in {~~60~>45~~} days.\n")
    assert old.diff(new, format=format).text
    clean = jubarte.from_markdown("Due in 45 days.\n")
    accepted = old.diff(new, format=format, accept_changes=True).text
    assert accepted == clean.diff(clean, format=format).text
    assert "30" not in accepted and "60" not in accepted


def test_word_always_accepts_both_inputs_before_new_markup():
    old = jubarte.from_markdown("Due in {~~30~>45~~} days.\n")
    new = jubarte.from_markdown("Due in {~~60~>45~~} days.\n")
    text = old.diff(new, format="word").text
    assert text == ""
    assert "{~~" not in text and "{++" not in text and "{--" not in text
    changed = old.diff(jubarte.from_markdown("Due in {~~60~>90~~} days.\n"), format="word").text
    assert "{~~45~>90~~}" in changed
    assert "30" not in changed and "60" not in changed


@pytest.mark.parametrize("format", ("github", "normal", "context", "side-by-side"))
def test_unicode_display_window_and_full_lines(format):
    prefix = "é🙂" * 100
    old, new = f"{prefix} old tail\n", f"{prefix} new tail\n"
    clipped = jubarte.diff(old, new, format=format).text
    full = jubarte.diff(old, new, format=format, full_lines=True).text
    assert prefix not in clipped
    assert prefix in full and "old tail" in full and "new tail" in full
    assert "�" not in clipped
    assert len(clipped) < len(full)
    assert prefix in jubarte._native.diff_unified(old, new)


@pytest.mark.parametrize("context", (True, False, -1, 1.5, 2**32, "3", None))
@pytest.mark.parametrize("format", VIEWS)
def test_view_context_is_strict(context, format):
    with pytest.raises(ValueError, match="context"):
        jubarte.diff("a", "b", format=format, context=context)
    with pytest.raises(ValueError, match="context"):
        jubarte._native.diff_view("a", "b", format="normal", context=context)


def test_native_view_labels_and_maximum_context():
    text = jubarte._native.diff_view("a\nb\n", "a\nc\n", format="github", context=2**32 - 1,
                                     old_name="folder/before.md", new_name="folder/after.md", full_lines=True)
    assert "--- a/folder/before.md\n+++ b/folder/after.md\n" in text
    assert " a\n" in text
    with pytest.raises(ValueError, match="format"):
        jubarte._native.diff_view("a", "b", format="critic")


@pytest.mark.parametrize("suffix,expected", (("txt", "md"), ("mdown", "md"), ("unknown", None)))
def test_shared_format_metadata(suffix, expected):
    args = build_parser().parse_args(["diff", "a.txt", "b.markdown", "-o", f"out.{suffix}"])
    assert args.old_format == args.new_format == "md"
    assert args.output_format == expected
    assert args.accept_changes is False and args.full_lines is False


@pytest.mark.integration
@pytest.mark.parametrize("format", VIEWS)
def test_cli_views_write_only_text_file(tmp_path, capsys, format):
    a, b, out = (tmp_path / name for name in ("old.txt", "new.md", f"{format}.patch"))
    a.write_text("Due in 30 days.\n")
    b.write_text("Due in 45 days.\n")
    assert main(["diff", str(a), str(b), "--format", format, "--full-lines", "-U", "0"]) == 0
    text = capsys.readouterr().out
    assert "30" in text and "45" in text
    assert not (tmp_path / "old_v_new.docx").exists()
    assert main(["diff", str(a), str(b), "--format", format, "--full-lines", "-U", "0", "-o", str(out)]) == 0
    streams = capsys.readouterr()
    assert streams.out == "" and "wrote" in streams.err
    assert out.read_text() == text


@pytest.mark.integration
@pytest.mark.parametrize("suffix", ("txt", "unknown"))
def test_cli_patch_output_uses_native_format_inference(tmp_path, capsys, suffix):
    a, b, out = (tmp_path / name for name in ("old.md", "new.md", f"out.{suffix}"))
    a.write_text("Due in 30 days.\n")
    b.write_text("Due in 45 days.\n")
    assert main(["diff", str(a), str(b), "-o", str(out)]) == 0
    assert out.read_text() == "Due in {~~30~>45~~} days.\n"
    assert "[-30-]{+45+}" in capsys.readouterr().out


@pytest.mark.integration
@pytest.mark.parametrize("format", ("patch", "critic", *VIEWS))
def test_invalid_docx_utf8_is_never_markdown(tmp_path, capsys, format):
    a, b = (tmp_path / name for name in ("bad.docx", "valid.md"))
    a.write_text("plain UTF-8 pretending to be Word\n")
    b.write_text("new\n")
    assert main(["diff", str(a), str(b), "--format", format]) == 1
    assert capsys.readouterr().out == ""
    assert not (tmp_path / "bad_v_valid.docx").exists()


@pytest.mark.parametrize("format", VIEWS)
def test_public_word_bytes_cannot_fall_back_to_markdown(format):
    with pytest.raises(jubarte.JubarteError):
        jubarte.diff(b"UTF-8 pretending to be DOCX\n", "new\n", format=format)


@pytest.mark.integration
def test_cli_accept_changes_and_word_ignore_prior_histories(tmp_path, capsys):
    a, b = (tmp_path / name for name in ("history-old.docx", "history-new.docx"))
    a.write_bytes(jubarte.from_markdown("Due in {~~30~>45~~} days.\n").to_bytes())
    b.write_bytes(jubarte.from_markdown("Due in {~~60~>45~~} days.\n").to_bytes())
    assert main(["diff", str(a), str(b), "--format", "github"]) == 0
    preserved = capsys.readouterr().out
    assert "30" in preserved and "60" in preserved
    assert main(["diff", str(a), str(b), "--format", "github", "--accept-changes"]) == 0
    assert capsys.readouterr().out == ""
    assert main(["diff", str(a), str(b), "--format", "word"]) == 0
    accepted = capsys.readouterr().out
    assert accepted == ""


@pytest.mark.integration
def test_unknown_output_suffix_falls_back_to_word_for_word_inputs(tmp_path, capsys):
    a, b, out = (tmp_path / name for name in ("base.docx", "next.docx", "redline.unknown"))
    a.write_bytes(jubarte.from_markdown("old\n").to_bytes())
    b.write_bytes(jubarte.from_markdown("new\n").to_bytes())
    assert main(["diff", str(a), str(b), "-o", str(out)]) == 0
    assert out.read_bytes().startswith(b"PK\x03\x04")
    assert "[-old-]{+new+}" in capsys.readouterr().out


@pytest.mark.integration
def test_compare_declared_docx_cannot_be_read_as_markdown(tmp_path, capsys):
    a, b, out = (tmp_path / name for name in ("pretend.docm", "new.md", "never.docx"))
    a.write_text("not a DOCX zip\n")
    b.write_text("new\n")
    assert main(["compare", str(a), str(b), "-o", str(out)]) == 1
    assert capsys.readouterr().out == ""
    assert not out.exists()


@pytest.mark.integration
def test_explicit_markdown_input_and_bom_share_native_text_behavior(tmp_path, capsys):
    a, b = (tmp_path / name for name in ("declared-old.docx", "declared-new.docx"))
    a.write_text("\ufeffDue 30 days.\n", encoding="utf-8")
    b.write_text("Due 45 days.\n", encoding="utf-8")
    assert main(["diff", str(a), str(b), "--from", "md", "--format", "word", "--full-lines"]) == 0
    assert capsys.readouterr().out == "Due {~~30~>45~~} days.\n"
