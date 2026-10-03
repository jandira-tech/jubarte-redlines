#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
#
# SPDX-License-Identifier: AGPL-3.0-only

"""scripts/library_readmes.py: cutting, link rewriting, stamps and gates."""

from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import library_readmes as lr  # noqa: E402

# REUSE-IgnoreStart
ROOT_README = """<!--
SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

SPDX-License-Identifier: AGPL-3.0-only
-->

> banner line

# jubarte

Tagline.

[![CI](https://ci/badge.svg)](https://ci/run)
[![license](https://img/license.svg)](./LICENSE)
[![PyPI](https://img/pypi.svg)](https://pypi.org/project/x/)

Intro with [layout rules](docs/WORD_LAYOUT_RULES.md) and [`docs/MARKDOWN.md`](docs/MARKDOWN.md).

## Install

```sh
# not a heading
cargo install jubarte-redlines
```

See [rendering](#rendering-and-fonts) and [contributing](#contributing).

## Rendering and fonts

![diagram](assets/flow.png) and `[not](a/link.md)` in code.

## Contributing

Run the gates.

## License

[AGPL](LICENSE)
"""
# REUSE-IgnoreEnd


class Cutting(unittest.TestCase):
    def test_split_keeps_fenced_hash_lines_inside_their_section(self) -> None:
        banner, lead, sections = lr.split_root(ROOT_README)
        self.assertEqual(banner, "> banner line")
        self.assertTrue(lead.startswith("Tagline."))
        names = [name for name, _ in sections]
        self.assertEqual(names, ["Install", "Rendering and fonts", "Contributing", "License"])
        self.assertIn("# not a heading", dict(sections)["Install"])

    def test_badges_are_filtered_in_spec_order(self) -> None:
        _, lead, _ = lr.split_root(ROOT_README)
        badges, rest = lr.filter_badges(lead, ("PyPI", "CI"))
        self.assertEqual(
            badges.split("\n"),
            [
                "[![PyPI](https://img/pypi.svg)](https://pypi.org/project/x/)",
                "[![CI](https://ci/badge.svg)](https://ci/run)",
            ],
        )
        self.assertNotIn("[![", rest)


class Links(unittest.TestCase):
    def rewrite(self, text: str, base: str = "", kept: set[str] | None = None) -> str:
        return lr.rewrite_links(text, base, "v1.2.3", kept or set())

    def test_relative_links_pin_to_the_tag(self) -> None:
        out = self.rewrite("[r](docs/X.md#part) [l](./LICENSE)")
        self.assertIn(f"({lr.REPO}/blob/v1.2.3/docs/X.md#part)", out)
        self.assertIn(f"({lr.REPO}/blob/v1.2.3/LICENSE)", out)

    def test_images_go_through_raw(self) -> None:
        out = self.rewrite("![d](assets/flow.png)")
        self.assertIn(f"({lr.RAW}/v1.2.3/assets/flow.png)", out)

    def test_a_badge_link_target_is_rewritten(self) -> None:
        out = self.rewrite("[![license](https://img/l.svg)](./LICENSE)")
        self.assertEqual(
            out, f"[![license](https://img/l.svg)]({lr.REPO}/blob/v1.2.3/LICENSE)"
        )

    def test_code_text_link_is_rewritten_but_code_spans_are_not(self) -> None:
        out = self.rewrite("[`docs/M.md`](docs/M.md) and `[not](a/link.md)`")
        self.assertIn(f"[`docs/M.md`]({lr.REPO}/blob/v1.2.3/docs/M.md)", out)
        self.assertIn("`[not](a/link.md)`", out)

    def test_fenced_code_is_untouched(self) -> None:
        text = "```md\n[x](docs/X.md)\n```"
        self.assertEqual(self.rewrite(text), text)

    def test_fragment_links_resolve_from_their_directory(self) -> None:
        out = self.rewrite("[root](../LICENSE) [here](python/x.py)", base="jubarte-python")
        self.assertIn(f"({lr.REPO}/blob/v1.2.3/LICENSE)", out)
        self.assertIn(f"({lr.REPO}/blob/v1.2.3/jubarte-python/python/x.py)", out)

    def test_anchors_stay_when_kept_and_point_home_when_cut(self) -> None:
        out = self.rewrite("[a](#kept) [b](#cut)", kept={"kept"})
        self.assertIn("[a](#kept)", out)
        self.assertIn(f"[b]({lr.REPO}/blob/v1.2.3/README.md#cut)", out)

    def test_absolute_and_mail_links_are_kept(self) -> None:
        text = "[a](https://x.y/z) [m](mailto:a@b.c)"
        self.assertEqual(self.rewrite(text), text)

    def test_a_link_out_of_the_repository_fails(self) -> None:
        with self.assertRaises(SystemExit):
            self.rewrite("[x](../../etc/passwd)")


class Render(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = Path(tempfile.mkdtemp(prefix="library_readmes_"))
        (self.tmp / "pkg").mkdir()
        (self.tmp / "pkg/README.fragment.md").write_text(
            # REUSE-IgnoreStart
            "<!--\nSPDX-License-Identifier: AGPL-3.0-only\n-->\n\n## Usage\n\n[ex](examples/a.py)\n",
            # REUSE-IgnoreEnd
            encoding="utf-8",
        )

    def tearDown(self) -> None:
        shutil.rmtree(self.tmp, ignore_errors=True)

    def lib(self, **kw: object) -> lr.Library:
        base = dict(
            name="t",
            out="pkg/README.md",
            title="pkg",
            badges=("CI",),
            keep=("Rendering and fonts", "License"),
            fragment="pkg/README.fragment.md",
        )
        base.update(kw)
        return lr.Library(**base)  # type: ignore[arg-type]

    def test_output_is_header_stamp_banner_title_badges_fragment_shared(self) -> None:
        out = lr.render(self.lib(), ROOT_README, "1.2.3", root=self.tmp)
        self.assertTrue(out.startswith("<!--\nSPDX-FileCopyrightText"))
        order = [
            "Generated by scripts/library_readmes.py from README.md and pkg/README.fragment.md for v1.2.3",
            "> banner line",
            "# pkg",
            "[![CI]",
            "## Usage",
            f"({lr.REPO}/blob/v1.2.3/pkg/examples/a.py)",
            "## Rendering and fonts",
            "## License",
        ]
        at = [out.index(marker) for marker in order]
        self.assertEqual(at, sorted(at), out)
        self.assertNotIn("## Install", out)
        self.assertNotIn("## Contributing", out)
        self.assertNotIn("Tagline.", out)

    def test_drop_mode_keeps_every_other_section_and_the_lead(self) -> None:
        lib = self.lib(keep=None, drop=("Contributing",), fragment=None, lead=True)
        out = lr.render(lib, ROOT_README, "1.2.3", root=self.tmp)
        for heading in ("## Install", "## Rendering and fonts", "## License", "Tagline."):
            self.assertIn(heading, out)
        self.assertNotIn("## Contributing", out)
        # The cut section's anchor points back to the root README.
        self.assertIn(f"({lr.REPO}/blob/v1.2.3/README.md#contributing)", out)
        self.assertIn("(#rendering-and-fonts)", out)

    def test_a_missing_kept_section_is_an_error(self) -> None:
        with self.assertRaises(SystemExit):
            lr.render(self.lib(keep=("Nope",)), ROOT_README, "1.2.3", root=self.tmp)

    def test_the_stamp_moves_with_the_root_readme(self) -> None:
        a = lr.render(self.lib(), ROOT_README, "1.2.3", root=self.tmp)
        b = lr.render(self.lib(), ROOT_README + "\nMore.\n", "1.2.3", root=self.tmp)
        self.assertNotEqual(a.split("\n")[6], b.split("\n")[6])


class Repository(unittest.TestCase):
    """The committed READMEs are current and document every public name."""

    def test_check_passes_on_the_committed_files(self) -> None:
        r = subprocess.run(
            [sys.executable, str(HERE / "library_readmes.py"), "--check"],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    def test_surfaces_are_found(self) -> None:
        py = lr.python_surface()
        self.assertIn("compare_documents", py)
        self.assertIn("scrub", py)
        wasm = lr.wasm_surface()
        self.assertIn("compareDocuments", wasm)
        self.assertIn("scrubDocument", wasm)
        # Getters of returned objects are not top-level exports.
        self.assertNotIn("redline", wasm)

    def test_a_missing_name_is_a_gap(self) -> None:
        lib = next(lib for lib in lr.LIBRARIES if lib.surface == "wasm")
        self.assertIn("compareDocuments", lr.surface_gaps(lib, "nothing here"))


if __name__ == "__main__":
    unittest.main()
