<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->
<!-- SPDX-License-Identifier: AGPL-3.0-only -->

# 00: what the sandbox carries

Each row is a fresh `ubuntu:24.04` (linux/amd64) container. The size is
the bytes `du -sxb /` counts after the install minus before, with the apt
lists and the apt, pip and npm caches removed. The pip rows start from an
image that already has Python and pip, so they count the package alone.
Measured on 2026-10-04 ([`sizes_jubarte_vs_replaced.tsv`](sizes_jubarte_vs_replaced.tsv)):

| Install | Size |
|---|---|
| jubarte 0.11.2 release binary (`linux-x86_64` tarball) | 36 MB (14 MB download) |
| jubarte 0.11.2 wheel (`pip install jubarte-redlines`) | 22 MB |
| LibreOffice, `libreoffice-writer-nogui --no-install-recommends` | 392 MB |
| LibreOffice, `apt-get install libreoffice` | 1,753 MB |
| Poppler (`poppler-utils`, for `pdftoppm`) | 25 MB |
| pandoc | 200 MB |
| Node.js, npm and `docx` (docx-js) | 233 MB |
| Python 3, pip and python-docx | 72 MB |
| python-docx with lxml, Python already present | 15 MB |

Per skill:

- Anthropic's `docx` runs LibreOffice, Poppler, pandoc and docx-js:
  850 MB with the smallest LibreOffice, v 36 MB. It drops 814 MB.
- OpenAI's `doc` asks for `apt-get install -y libreoffice poppler-utils`
  and uses python-docx: 1,793 MB v 36 MB. It drops 1,757 MB (396 MB with
  the smallest LibreOffice).

## Verdict

jubarte is the smaller install by an order of magnitude. Rows are
separate containers, so shared libraries are counted in each; the sum
over-counts by whatever two tools share (little: LibreOffice brings its
own libraries). The sizes say nothing about output quality; the other
folders do.

## Run it

```bash
JUBARTE_MEASURE_SIZES=1 bash examples/adoption/00-install-size/run.sh
```

It needs Docker and downloads about 2 GB, so without
`JUBARTE_MEASURE_SIZES=1` it prints the committed table instead. On an
arm64 host the amd64 containers run emulated (about 40 minutes).
