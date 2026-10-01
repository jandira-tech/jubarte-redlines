# Run logs — Python bindings demo & synthetic fixtures

Environment: Ubuntu sandbox, Python 3.12.14, Rust release build of
`jubarte-python` (PyO3 + maturin), engine `jubarte-redlines` 0.10.1.

## 1. Build — maturin develop --release

```text
$ python3 -m venv .venv && .venv/bin/pip install -q pytest maturin
$ VIRTUAL_ENV=$PWD/.venv .venv/bin/maturin develop --release
    Blocking waiting for file lock on build directory
    Finished `release` profile [optimized] target(s) in 1m 46s
$ cp target/release/libjubarte_python.so python/jubarte_redlines/_native.abi3.so
$ PYTHONPATH=python .venv/bin/python -c "import jubarte_redlines"
```

Note: maturin's wheel staging step failed with a copy error (staging dir
`target/maturin/` missing); the Rust build itself succeeded, so the native
library was staged manually from `target/release/`.

## 2. demo_bindings.py — every binding variation

```text
==============================================================
1. compare_documents (default author)
==============================================================
redline bytes: 6483
==============================================================
2. compare_documents (custom author + date)
==============================================================
redline bytes: 6485
==============================================================
3. get_revisions / get_revisions_json
==============================================================
  Inserted author='jubarte' text='DELTA'
  Deleted author='jubarte' text='Gamma'
  Deleted author='jubarte' text='Second paragraph here.\n'
  Inserted author='jubarte' text='Completely rewritten text.\n'
  JSON: 4 revisions
==============================================================
4. accept_revisions / reject_revisions
==============================================================
accepted markdown: [body:p:0] Alpha Beta DELTA.
[body:p:1] Completely rewritten text.
[body:p:2] Third line stays.
rejected markdown: [body:p:0] Alpha Beta Gamma.
[body:p:1] Second paragraph here.
[body:p:2] Third line stays.
==============================================================
5. Document API (compare, changes, inspect)
==============================================================
compare -> Document, sha256: b84f206e38069270 …
  change id=body:rev:1 kind=insertion text='DELTA'
  change id=body:rev:2 kind=deletion text='Gamma'
  change id=body:rev:3 kind=deletion text=''
  change id=body:rev:4 kind=deletion text='Second paragraph here.'
  change id=body:rev:5 kind=insertion text=''
  change id=body:rev:6 kind=insertion text='Completely rewritten text.'
snapshot paragraphs: 3
markdown: [body:p:0] Alpha Beta Gamma.
[body:p:1] Second paragraph here.
[body:p:2] Third line stays.
==============================================================
6. EditPlan (replace + insert + comment)
==============================================================
report: EditReport(schema_version=1, ok=True,
  source_sha256='bfdda325721b2146836b6e6d9ec99b1ce91a584c95e3f42da49d0238d95491d8',
  author='Editor', date='1970-01-01T00:00:00Z', existing_revisions='refuse',
  paragraphs=ParagraphDelta(from_=3, to=4), operations=(
    EditOutcome(id='op-1', kind='replace', status='ok', matches=1, paragraph='body:p:0', context='Alpha {Beta→REPLACED} Gamma.', ...),
    EditOutcome(id='op-2', kind='insert', status='ok', matches=1, paragraph='body:p:0', context='Alpha Beta{+Inserted run.} Gamma.', ...),
    EditOutcome(id='op-3', kind='insert_paragraph', status='ok', matches=1, paragraph='body:p:0', context='{+¶ A whole new paragraph.}', ...),
    EditOutcome(id='op-4', kind='comment', status='ok', matches=1, paragraph='body:p:0', context='{#Alpha} Beta Gamma.', comment_id=0, ...)),
  comments_added=1, revisions=RevisionCounts(inserted=2, deleted=1, moved=0, format_changed=0, total=3),
  resolved_revisions=ResolvedRevisions(accepted=(), rejected=()))
clean markdown:
[body:p:0] Alpha REPLACEDInserted run. Gamma.
[body:p:1] A whole new paragraph.
[body:p:2] Second paragraph here.
[body:p:3] Third line stays.
redline revisions: 3
==============================================================
7. docx_to_pdf / to_png
==============================================================
PDF bytes: 17430, header: b'%PDF-1.4'
PNG pages: 1, first PNG header: b'\x89PNG\r\n\x1a\n'
==============================================================
8. python -m jubarte_redlines --help
==============================================================
usage: python -m jubarte_redlines [-h] [--version]
                                  {inspect,text,edit,convert,compare,revisions,changes,accept,reject,capabilities} ...
DOCX compare, tracked editing, inspection and rendering (the jubarte engine).
==============================================================
9. JubarteError on garbage input
==============================================================
caught JubarteError: ZIP error: invalid Zip archive: Could not find EOCD

All variations OK ✅
```

## 3. make_fixtures.py — synthetic NDA fixtures

```text
$ PYTHONPATH=jubarte-python/python .venv/bin/python make_fixtures.py
revisions:
  Inserted 'three (3'
  Deleted 'two (2'
  Inserted ','
  Deleted ' or'
  Inserted ' or financial'
  Inserted 'The Receiving Party may not disclose Confidential Informatio'
```

## 4. Verification of delivered artifacts

```text
$ python - <<'EOF'   (compare against delivered files)
accepted == modified content: True
rejected == original content: True
redline revisions: 6
original.pdf b'%PDF-1.4'
modified.pdf b'%PDF-1.4'
redline.pdf b'%PDF-1.4'
EOF
```

## 5. Test suite

```text
$ PYTHONPATH=jubarte-python/python .venv/bin/python -m pytest jubarte-python/tests -q
........................................................................ [ 71%]
.............................                                            [100%]
101 passed in 0.33s
```
