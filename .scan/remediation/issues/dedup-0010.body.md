@coderabbitai plan
Please enrich, fix and assess the plan below.

## Findings tackled
csf_9e40f23e742aa43493a0fb92 | Rendering a small DOCX table can exhaust host memory | medium
csf_0e2b085a251b34412cd71a4c | Reading or converting nested legacy DOC fields can monopolize the processing thread | medium
csf_a69309b0b64d1048c5a4e319 | Untrusted Markdown reference documents reach unchecked recursive XML parsing | low
csf_85d9c340b10d981559925227 | A shared font-index directory permits staging symlink overwrites | low
csf_b45e3ea11925b4220c435786 | Python CLI output/plan and npm direct-output symlinks can overwrite unrelated files | low
csf_68acfaf376a60cf8c3d8e2d4 | Cyclic Markdown drawing dependencies terminate conversion through unbounded recursion | medium
csf_5ad7cba70ec2176a33e66f2a | Tiny Markdown table metadata can trigger unbounded cell allocation | medium
csf_dea8102957ca1306a0f7756b | Unbounded list levels cause billions of iterations or allocations | medium
csf_6a40bc41bdf34a737ceac40d | Entity decoding repeatedly scans the input suffix | medium
csf_c2ab52b1c8614c7fd5356cf8 | Python and Node plan recording can overwrite unrelated files through staging symlinks | low
csf_6128bc397a829afd60ea9390 | Untrusted Markdown reference documents reach unchecked recursive XML parsing | low

Full texts of the findings above (summary and remediation for every findingId) are posted as follow-up comments on this issue.
The full TDD red/green plan is posted as a follow-up comment on this issue.

## Patch
```diff
diff --git a/jubarte-python/python/jubarte_redlines/__main__.py b/jubarte-python/python/jubarte_redlines/__main__.py
index 686d0f0..b92043c 100644
--- a/jubarte-python/python/jubarte_redlines/__main__.py
+++ b/jubarte-python/python/jubarte_redlines/__main__.py
@@ -32,7 +32,9 @@ from __future__ import annotations
 import argparse
 import json
 import os
+import stat
 import sys
+import tempfile
 from dataclasses import asdict
 from collections.abc import Sequence
 from pathlib import Path
@@ -123,13 +125,54 @@ def _check_side_file(side: Path, flag: str, others: tuple[tuple[Path, str], ...]
     _ensure_writable(side, force)
 
 
-def _write(path: Path, data: bytes | str) -> int:
-    """Writes ``data`` (text as UTF-8, newlines as given); returns the bytes written."""
+def _umask_default_mode() -> int:
+    """The mode a freshly created output takes — ``open``'s 0o666 masked by
+    the umask — which exclusive ``mkstemp`` staging must re-apply before the
+    staged file is installed as an output."""
+    mask = os.umask(0)
+    os.umask(mask)
+    return 0o666 & ~mask
+
+
+def _publish_mode(path: Path) -> int:
+    """The permissions an installed output takes: a regular file a forced
+    installation replaces keeps its own (a truncating write never changed
+    them), and any other destination is created with the umask default;
+    ``mkstemp``'s private 0600 would narrow every output the CLI writes."""
+    try:
+        info = path.lstat()
+    except OSError:
+        return _umask_default_mode()
+    return stat.S_IMODE(info.st_mode) if stat.S_ISREG(info.st_mode) else _umask_default_mode()
+
+
+def _write(path: Path, data: bytes | str, force: bool = False) -> int:
+    """Writes ``data`` (text as UTF-8, newlines as given) through exclusive
+    staging beside ``path``: a hard link that never clobbers when ``force``
+    is false (a name occupied after the check — a symlink included — is
+    refused), a rename that replaces only the output entry when true.
+    Returns the bytes written; a symlink planted at the output or a staging
+    name never has its referent written."""
     raw = data.encode("utf-8") if isinstance(data, str) else data
     try:
-        path.write_bytes(raw)
+        fd, staged = tempfile.mkstemp(dir=path.parent, prefix=f".{path.name}.", suffix=".partial")
     except OSError as exc:
         raise CliError(f"writing {path}: {_os(exc)}") from exc
+    try:
+        with os.fdopen(fd, "wb") as handle:
+            handle.write(raw)
+        os.chmod(staged, _publish_mode(path))
+        if force:
+            os.replace(staged, path)
+        else:
+            os.link(staged, path)
+            os.unlink(staged)
+    except OSError as exc:
+        try:
+            os.unlink(staged)
+        except OSError:
+            pass
+        raise CliError(f"writing {path}: {_os(exc)}") from exc
     return len(raw)
 
 
@@ -245,7 +288,7 @@ def _write_result(args: argparse.Namespace, data: bytes, *, source: str, view: d
         text = _native.read_json(data, **_selection(view)) + "\n"
     else:
         text = data
-    _wrote(args, output, kind, _write(output, text))
+    _wrote(args, output, kind, _write(output, text, args.force))
 
 
 def _not_written(args: argparse.Namespace) -> str:
@@ -402,15 +445,23 @@ def _record(args: argparse.Namespace, verb: str, doc: Document) -> int:
             print(f"note: {note}")
     if not dry_run:
         # Through a symbolic link to the file it names, written yet or not;
-        # via a hidden partial file named for this process, removed if the
-        # write fails; bytes as they are, so no platform newline translation.
+        # via a hidden partial beside it — named unpredictably and created
+        # exclusively, so a symlink planted at a predictable PID name cannot
+        # take the write — removed if the write fails; bytes as they are, so
+        # no platform newline translation.
         target = Path(os.path.realpath(plan))
-        partial = target.with_name(f".{target.name}.{os.getpid()}.partial")
+        staged: str | None = None
         try:
-            partial.write_bytes(recorded["json"].encode("utf-8"))
-            os.replace(partial, target)
+            fd, staged = tempfile.mkstemp(
+                dir=target.parent, prefix=f".{target.name}.", suffix=".partial"
+            )
+            with os.fdopen(fd, "wb") as handle:
+                handle.write(recorded["json"].encode("utf-8"))
+            os.chmod(staged, _umask_default_mode())
+            os.replace(staged, target)
         except OSError as exc:
-            partial.unlink(missing_ok=True)
+            if staged is not None:
+                Path(staged).unlink(missing_ok=True)
             raise CliError(f"writing {plan}: {exc}") from exc
     if not args.quiet:
         print(recorded["summary"])
@@ -510,11 +561,11 @@ def _apply_edit(args: argparse.Namespace, verb: str, doc: Document) -> int:
     out_dir.mkdir(parents=True, exist_ok=True)
     saved = []
     for name, data in outputs:
-        _write(out_dir / name, data)
+        _write(out_dir / name, data, True)
         saved.append({"f": name, "bytes": len(data), "sha256": _native.source_sha256(data)})
     lines.append(_jsonl({"ev": "save", "dir": str(out_dir), "outputs": saved}))
     lines.append(summary)
-    _write(out_dir / "report.jsonl", "\n".join(lines) + "\n")
+    _write(out_dir / "report.jsonl", "\n".join(lines) + "\n", True)
     if args.quiet:
         return EXIT_OK
     # A pure accept/reject prints the resolved lines on stdout, nothing
@@ -637,7 +688,7 @@ def _write_edited(args: argparse.Namespace, path: Path, result: EditResult, note
             print(f"error: output '{taken}' already exists (use --force to overwrite)", file=sys.stderr)
             raise SystemExit(EXIT_USAGE)
     for output, data in outputs:
-        _write(output, data)
+        _write(output, data, args.force)
     if args.quiet:
         return EXIT_OK
     print(f"wrote {path} (docx, {len(document)} bytes)", file=sys.stderr)
@@ -706,7 +757,7 @@ def cmd_convert(args: argparse.Namespace) -> int:
                     raise CliError(f"{flag} applies to PDF or PNG output only")
             docx_out = args.output or args.file.with_suffix(".docx")
             _ensure_writable(docx_out, args.force)
-            _wrote(args, docx_out, "docx", _write(docx_out, doc.to_bytes()))
+            _wrote(args, docx_out, "docx", _write(docx_out, doc.to_bytes(), args.force))
             return EXIT_OK
     else:
         if args.output_format == "docx" and not args.update_fields:
@@ -754,21 +805,21 @@ def _render_to(args: argparse.Namespace, doc: Document, output: Path) -> None:
             if side is not None and _same_file(side, path):
                 raise CliError(f"{flag} '{side}' is the same file as the PNG output")
     if rendered.pdf is not None:
-        _write(output, rendered.pdf)
+        _write(output, rendered.pdf, args.force)
         _wrote(args, output, "pdf", len(rendered.pdf), pages)
     if png:
         from . import _native
 
         for path, image in zip(png_paths, rendered.pngs):
-            _write(path, image)
+            _write(path, image, args.force)
         if not args.quiet:
             pattern = output.parent / f"{output.stem}-page-NN.png"
             print(_native.wrote_png_line(pattern, len(rendered.pngs), args.dpi), file=sys.stderr)
     if getattr(args, "font_report", None) is not None:
-        _write(args.font_report, json.dumps([asdict(f) for f in rendered.report.fonts]))
+        _write(args.font_report, json.dumps([asdict(f) for f in rendered.report.fonts]), args.force)
     if getattr(args, "report", None) is not None:
         report = {"page_count": pages, "document_pages": rendered.report.document_pages, "pages": [asdict(p) for p in rendered.report.pages], "fonts": [asdict(f) for f in rendered.report.fonts]}
-        _write(args.report, json.dumps(report, ensure_ascii=False))
+        _write(args.report, json.dumps(report, ensure_ascii=False), args.force)
 
 
 def _write_updated_fields(args: argparse.Namespace, doc: Document) -> int:
@@ -786,9 +837,9 @@ def _write_updated_fields(args: argparse.Namespace, doc: Document) -> int:
         print(f"{field.paragraph}\t{field.kind}\t{json.dumps(field.old, ensure_ascii=False)} -> {json.dumps(field.new, ensure_ascii=False)}")
     print(f"{len(updated.fields)} field(s) written; {updated.page_count} page(s)", file=sys.stderr)
     if args.report is not None:
-        _write(args.report, json.dumps({"page_count": updated.page_count, "fields": [asdict(f) for f in updated.fields]}, ensure_ascii=False))
+        _write(args.report, json.dumps({"page_count": updated.page_count, "fields": [asdict(f) for f in updated.fields]}, ensure_ascii=False), args.force)
     data = updated.document.to_bytes()
-    _write(output, data)
+    _write(output, data, args.force)
     _wrote(args, output, "docx", len(data))
     return EXIT_OK
 
@@ -828,7 +879,7 @@ def cmd_diff_render(args: argparse.Namespace) -> int:
             _ensure_writable(path, args.force)
         out.mkdir(parents=True, exist_ok=True)
         for path, data in files:
-            _write(path, data)
+            _write(path, data, args.force)
     if args.json:
         print(summary)
     else:
@@ -865,7 +916,7 @@ def cmd_compare(args: argparse.Namespace) -> int:
             raise SystemExit(EXIT_USAGE)
         _ensure_writable(output, args.force)
         critic = diff(original, modified, format="critic", author=args.author, date=args.datetime).text
-        _wrote(args, output, "md", _write(output, critic))
+        _wrote(args, output, "md", _write(output, critic, args.force))
         return EXIT_OK
     if args.output_format != "png":
         _ensure_writable(output, args.force)
@@ -900,7 +951,7 @@ def cmd_diff(args: argparse.Namespace) -> int:
     if output is None:
         sys.stdout.write(text)
         return EXIT_OK
-    size = _write(output, text)
+    size = _write(output, text, args.force)
     if not args.quiet:
         print(f"wrote {_spelled(args, 'output')} (md, {size} bytes)", file=sys.stderr)
     return EXIT_OK
@@ -969,7 +1020,7 @@ def cmd_validate(args: argparse.Namespace) -> int:
         _ensure_writable(args.repair, args.force)
         repaired = doc.repair()
         docx = repaired.document.to_bytes()
-        size = _write(args.repair, docx)
+        size = _write(args.repair, docx, args.force)
         if not args.json:
             print(f"repaired {len(repaired.repaired)} finding(s)")
         if not args.quiet:
diff --git a/jubarte-python/tests/test_cli.py b/jubarte-python/tests/test_cli.py
index 62469a0..6bbe0a0 100644
--- a/jubarte-python/tests/test_cli.py
+++ b/jubarte-python/tests/test_cli.py
@@ -854,3 +854,50 @@ def test_edit_resolution_bundle_prints_resolved_lines(tmp_path: Path, capsys: py
     assert captured.out == 'accepted r0 in p3: "{++quarterly++}" > "quarterly"\n'
     assert (bundle / "report.jsonl").is_file()
     assert (bundle / "clean.docx").is_file()
+
+
+def _symlink(link: Path, referent: Path) -> None:
+    try:
+        link.symlink_to(referent)
+    except (OSError, NotImplementedError):
+        pytest.skip("this account cannot create symlinks")
+
+
+def test_an_output_that_is_a_dangling_symlink_is_refused(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
+    # `exists()` follows links, so a dangling output name passes the
+    # preflight; the write itself must refuse it rather than create the
+    # file the link names.
+    out = tmp_path / "hole.pdf"
+    referent = tmp_path / "referent.pdf"
+    _symlink(out, referent)
+    assert main(["accept", str(letter), "-o", str(out)]) == 1
+    captured = capsys.readouterr()
+    assert captured.err.startswith(f"error: writing {out}"), captured.err
+    assert out.is_symlink() and not referent.exists(), "nothing is written through the link"
+
+
+def test_force_replaces_a_symlink_entry_not_its_referent(letter: Path, tmp_path: Path) -> None:
+    victim = tmp_path / "victim.pdf"
+    victim.write_bytes(b"Carol Diaz")
+    out = tmp_path / "out.pdf"
+    _symlink(out, victim)
+    assert main(["accept", str(letter), "-o", str(out), "--force"]) == 0
+    assert victim.read_bytes() == b"Carol Diaz", "the referent is never touched"
+    assert not out.is_symlink(), "the output entry itself is replaced"
+    assert out.read_bytes().startswith(b"%PDF"), "the new bytes are installed"
+    # An installed output keeps the permissions a truncating write gave it,
+    # not the staging file's private mkstemp mode.
+    mask = os.umask(0)
+    os.umask(mask)
+    assert out.stat().st_mode & 0o777 == 0o666 & ~mask, "the umask default survives staging"
+
+
+def test_a_predictably_named_plan_partial_cannot_redirect_the_write(letter: Path, tmp_path: Path) -> None:
+    victim = tmp_path / "victim.tsv"
+    victim.write_text("keep me")
+    plan = tmp_path / "plan.json"
+    planted = tmp_path / f".{plan.name}.{os.getpid()}.partial"
+    _symlink(planted, victim)
+    assert main(["edit", str(letter), "--plan", str(plan), "-p", "p1", "--anchor", "his or her", "--content", "their"]) == 0
+    assert victim.read_text() == "keep me", "the planted partial's referent is untouched"
+    assert json.loads(plan.read_text())["operations"], "the plan itself is recorded"
diff --git a/jubarte-wasm/cli/bin/jubarte-redlines.mjs b/jubarte-wasm/cli/bin/jubarte-redlines.mjs
index 77f5325..a6bffa4 100644
--- a/jubarte-wasm/cli/bin/jubarte-redlines.mjs
+++ b/jubarte-wasm/cli/bin/jubarte-redlines.mjs
@@ -13,6 +13,7 @@
 // 3 edit plan refused (its per-operation report is on stdout; nothing was
 // written).
 
+import crypto from "node:crypto";
 import fs from "node:fs";
 import { createRequire } from "node:module";
 import path from "node:path";
@@ -103,10 +104,38 @@ function markdownText(file, bytes) {
   }
 }
 
-function write(file, data) {
+/** Write `data` through exclusive staging beside `file`: a hard link that
+ * never clobbers when `force` is false (a name occupied after the check —
+ * a symlink included — is refused), a rename that replaces only the output
+ * entry, never a referent, when true. */
+function write(file, data, force = false) {
+  const staged = path.join(
+    path.dirname(path.resolve(file)),
+    `.${path.basename(file)}.${crypto.randomUUID()}.partial`
+  );
   try {
-    fs.writeFileSync(file, data);
+    const fd = fs.openSync(staged, "wx");
+    try {
+      fs.writeFileSync(fd, data);
+    } finally {
+      fs.closeSync(fd);
+    }
+    if (force) {
+      // A truncating write kept a replaced regular file's permissions; the
+      // rename that replaces the entry must put them back on the new bytes.
+      let mode = null;
+      try {
+        const info = fs.lstatSync(file);
+        if (info.isFile()) mode = info.mode & 0o777;
+      } catch {}
+      if (mode !== null) fs.chmodSync(staged, mode);
+      fs.renameSync(staged, file);
+    } else {
+      fs.linkSync(staged, file);
+      fs.unlinkSync(staged);
+    }
   } catch (e) {
+    try { fs.rmSync(staged, { force: true }); } catch {}
     throw new CliError(`writing ${file}: ${osError(e)}`);
   }
 }
@@ -141,7 +170,7 @@ function writeResult(o, docx, source, view) {
   const data = kind === "md" ? viewText(docx, view ?? {}, source)
     : kind === "json" ? `${wasm.readJson(docx, JSON.stringify(selection(view)))}\n`
     : docx;
-  write(output, data);
+  write(output, data, o.force);
   wrote(o, output, kind, typeof data === "string" ? Buffer.byteLength(data, "utf8") : data.length);
 }
 
@@ -149,7 +178,7 @@ function writeResult(o, docx, source, view) {
 function renderTo(o, docx, output) {
   ensureWritable(output, o.force);
   const pdf = wasm.docxToPdf(docx, Boolean(o.compress), o.revisions ?? undefined, paletteOf(o), Boolean(o.move_comments), Boolean(o.changed_pages));
-  write(output, pdf);
+  write(output, pdf, o.force);
   wrote(o, output, "pdf", pdf.length, wasm.pdfPageCount(pdf));
 }
 
@@ -176,7 +205,7 @@ const COMMANDS = {
         if (o.view_given) throw new UsageError("read options shape the view; two Markdown documents write CriticMarkup to a .md");
         ensureWritable(output, o.force);
         const critic = wasm.diffDocumentsCritic(a, b, o.author, o.datetime);
-        write(output, critic);
+        write(output, critic, o.force);
         return void wrote(o, output, "md", Buffer.byteLength(critic, "utf8"));
       }
       ensureWritable(output, o.force);
@@ -211,7 +240,7 @@ const COMMANDS = {
         } else text = JSON.parse(wasm.diffDocuments(a, b, o.author, date, o.columns, old, next)).text;
       }
       if (o.output == null) return void process.stdout.write(text);
-      write(o.output, text);
+      write(o.output, text, o.force);
       if (!o.quiet) console.error(`wrote ${o.output} (md, ${Buffer.byteLength(text, "utf8")} bytes)`);
     },
   },
@@ -256,7 +285,7 @@ const COMMANDS = {
           }
           const output = o.output ?? path.join(path.dirname(file), `${stem(file)}.docx`);
           ensureWritable(output, o.force);
-          write(output, docx);
+          write(output, docx, o.force);
           return void wrote(o, output, "docx", docx.length);
         }
       } else if (convertsToDocx(o)) {
@@ -317,10 +346,10 @@ function wordToWord(file, docx, o) {
       console.log(`${field.paragraph}\t${field.kind}\t${JSON.stringify(field.old)} -> ${JSON.stringify(field.new)}`);
     }
     console.error(`${report.fields.length} field(s) written; ${report.page_count} page(s)`);
-    if (o.report != null) write(o.report, updated.json);
+    if (o.report != null) write(o.report, updated.json, o.force);
     out = updated.docx;
   }
-  write(o.output, out);
+  write(o.output, out, o.force);
   wrote(o, o.output, "docx", out.length);
 }
 
@@ -331,7 +360,7 @@ function wordToMarkdown(docx, o) {
   if (warning) console.error(`warning: ${warning}`);
   if (o.output == null) return void process.stdout.write(markdown);
   ensureWritable(o.output, o.force);
-  write(o.output, markdown);
+  write(o.output, markdown, o.force);
   wrote(o, o.output, "md", Buffer.byteLength(markdown, "utf8"));
 }
 
@@ -417,9 +446,17 @@ function recordPlan(verb, file, docx, o) {
         target = path.resolve(path.dirname(o.plan), fs.readlinkSync(o.plan));
       } catch {}
     }
-    const partial = path.join(path.dirname(target), `.${path.basename(target)}.${process.pid}.partial`);
+    // A hidden partial beside it, named unpredictably and created
+    // exclusively: a symlink planted at a predictable PID name cannot
+    // take the write.
+    const partial = path.join(path.dirname(target), `.${path.basename(target)}.${crypto.randomUUID()}.partial`);
     try {
-      fs.writeFileSync(partial, recorded.json);
+      const fd = fs.openSync(partial, "wx");
+      try {
+        fs.writeFileSync(fd, recorded.json);
+      } finally {
+        fs.closeSync(fd);
+      }
       fs.renameSync(partial, target);
     } catch (e) {
       fs.rmSync(partial, { force: true });
@@ -545,11 +582,11 @@ function applyEdit(verb, file, docx, o) {
   for (const [name, r] of renders) outputs.push([`${name}.pdf`, Buffer.from(r.pdf)]);
   fs.mkdirSync(outDir, { recursive: true });
   const saved = outputs.map(([name, data]) => {
-    write(path.join(outDir, name), data);
+    write(path.join(outDir, name), data, true);
     return { f: name, bytes: data.length, sha256: wasm.sourceSha256(data) };
   });
   lines.push(jsonl({ ev: "save", dir: outDir, outputs: saved }), summary);
-  write(path.join(outDir, "report.jsonl"), `${lines.join("\n")}\n`);
+  write(path.join(outDir, "report.jsonl"), `${lines.join("\n")}\n`, true);
   if (o.quiet) return;
   // A pure accept/reject prints the resolved lines on stdout, nothing
   // else; the bundle's report.jsonl still holds the summary.
diff --git a/jubarte-wasm/cli/test/cli.test.mjs b/jubarte-wasm/cli/test/cli.test.mjs
index ee06baf..ba07f53 100644
--- a/jubarte-wasm/cli/test/cli.test.mjs
+++ b/jubarte-wasm/cli/test/cli.test.mjs
@@ -1488,3 +1488,37 @@ test("edit -p rN --accept/--reject prints the resolved changes, as the binary do
   assert.equal(pair.code, 0, pair.err);
   assert.equal(pair.out, 'accepted r1+2 in p3: "{~~weekly~>monthly~~}" > "monthly"\n');
 });
+
+test("an output that is a dangling symlink is refused, never written through", () => {
+  // existsSync follows links, so a dangling output name passes the check;
+  // the write itself must refuse it rather than create the referent.
+  const out = path.join(tmp, "hole.md");
+  fs.symlinkSync(path.join(tmp, "referent.md"), out);
+  const r = run("diff", tracked, untracked, "-o", out);
+  assert.equal(r.code, 1, r.err);
+  assert.match(r.err, /^error: writing /);
+  assert.ok(fs.lstatSync(out).isSymbolicLink(), "the link stays a link");
+  assert.ok(!fs.existsSync(path.join(tmp, "referent.md")), "the referent is never created");
+});
+
+test("--force replaces the symlink entry, not its referent", () => {
+  const victim = path.join(tmp, "victim.md");
+  fs.writeFileSync(victim, "Carol Diaz");
+  const out = path.join(tmp, "out.md");
+  fs.symlinkSync(victim, out);
+  const r = run("diff", tracked, untracked, "-o", out, "--force");
+  assert.equal(r.code, 0, r.err);
+  assert.equal(fs.readFileSync(victim, "utf8"), "Carol Diaz", "the referent is never touched");
+  assert.ok(!fs.lstatSync(out).isSymbolicLink(), "the output entry itself is replaced");
+  assert.ok(fs.readFileSync(out, "utf8").length > 0, "the new bytes are installed");
+});
+
+test("recording a plan leaves no partial beside it", () => {
+  const plan = path.join(tmp, "record-plan.json");
+  const r = run("edit", untracked, "--plan", plan, "-p", "p1", "--content", "Changed.");
+  assert.equal(r.code, 0, r.err);
+  const planJson = JSON.parse(fs.readFileSync(plan, "utf8"));
+  assert.equal(planJson.operations.length, 1);
+  const leftovers = fs.readdirSync(tmp).filter((name) => name.endsWith(".partial"));
+  assert.deepEqual(leftovers, []);
+});
diff --git a/src/convert/altchunk.rs b/src/convert/altchunk.rs
index 7def5f9..49f1c06 100644
--- a/src/convert/altchunk.rs
+++ b/src/convert/altchunk.rs
@@ -730,7 +730,16 @@ pub(crate) fn decode_entities(s: &str) -> String {
     while let Some(a) = rest.find('&') {
         out.push_str(&rest[..a]);
         let tail = &rest[a..];
-        let semi = tail.find(';').filter(|&e| e <= 10);
+        // A supported entity (`&#x1F600;`) ends within ten characters; a
+        // `;` past that window is not the end of one. Searching the whole
+        // remaining input per `&` made runs of bare ampersands quadratic.
+        // The window snaps back to a character boundary: an 11-byte cut
+        // through a multibyte character would fall back to the whole tail.
+        let mut window = 11.min(tail.len());
+        while !tail.is_char_boundary(window) {
+            window -= 1;
+        }
+        let semi = tail[..window].find(';');
         let decoded = semi.and_then(|e| {
             let ent = &tail[1..e];
             let ch = match ent {
@@ -930,6 +939,17 @@ mod tests {
             );
         }
 
+        #[test]
+        fn long_bare_ampersand_runs_stay_linear() {
+            // Only the eleven bytes after an `&` are searched for the `;`
+            // that ends a supported entity: a run of bare ampersands must
+            // not rescan the whole remaining input at each one.
+            let bare = "&".repeat(1_000_000);
+            assert_eq!(decode_entities(&bare), bare);
+            let distant = format!("{bare}x;");
+            assert_eq!(decode_entities(&distant), distant);
+        }
+
         #[test]
         fn css_lengths_convert_physical_units_and_decimal_commas() {
             for (length, points) in [
diff --git a/src/convert/font.rs b/src/convert/font.rs
index d60243a..03400e3 100644
--- a/src/convert/font.rs
+++ b/src/convert/font.rs
@@ -3219,14 +3219,18 @@ impl FontIndex {
             .unwrap_or_default();
         merged.extend(self.entries.iter().map(|(k, v)| (k.clone(), v.clone())));
         let text = format_font_index(&merged);
-        let tmp = path.with_extension(format!("tsv.{}.tmp", std::process::id()));
-        let written = path
+        if path
             .parent()
-            .is_some_and(|dir| fs::create_dir_all(dir).is_ok())
-            && fs::write(&tmp, text).is_ok()
-            && fs::rename(&tmp, path).is_ok();
-        if !written {
-            let _ = fs::remove_file(&tmp);
+            .is_some_and(|dir| fs::create_dir_all(dir).is_err())
+        {
+            return;
+        }
+        // Publish through exclusive, unpredictable private staging (the
+        // recorder's design): the old PID-named temporary followed a
+        // symlink planted in a shared index directory and truncated the
+        // file it named.
+        if let Ok(staged) = crate::publication::StagedFile::prepare(path, text.as_bytes(), true) {
+            let _ = staged.commit();
         }
     }
 }
@@ -5413,6 +5417,49 @@ mod tests {
             );
         }
 
+        #[test]
+        fn the_font_index_publishes_through_private_staging() {
+            let dir = tempfile::tempdir().unwrap();
+            let index = FontIndex {
+                path: Some(dir.path().join("font-index.tsv")),
+                entries: HashMap::new(),
+            };
+            index.save();
+            let written = fs::read_to_string(dir.path().join("font-index.tsv")).unwrap();
+            assert!(written.starts_with(FONT_INDEX_HEADER), "{written}");
+            let leftovers: Vec<_> = fs::read_dir(dir.path())
+                .unwrap()
+                .map(|entry| entry.unwrap().file_name())
+                .filter(|name| {
+                    let name = name.to_string_lossy();
+                    name.contains(".partial") || name.ends_with(".tmp")
+                })
+                .collect();
+            assert!(leftovers.is_empty(), "staging leftovers: {leftovers:?}");
+        }
+
+        #[cfg(unix)]
+        #[test]
+        fn a_planted_predictable_staging_symlink_never_redirects_the_index() {
+            let dir = tempfile::tempdir().unwrap();
+            let victim = dir.path().join("victim.tsv");
+            fs::write(&victim, "keep me").unwrap();
+            let index_path = dir.path().join("font-index.tsv");
+            // The staging name the index used to derive from its PID.
+            std::os::unix::fs::symlink(
+                &victim,
+                index_path.with_extension(format!("tsv.{}.tmp", std::process::id())),
+            )
+            .unwrap();
+            FontIndex {
+                path: Some(index_path.clone()),
+                entries: HashMap::new(),
+            }
+            .save();
+            assert_eq!(fs::read_to_string(&victim).unwrap(), "keep me");
+            assert!(index_path.is_file(), "the index itself is published");
+        }
+
         #[test]
         fn dfonts_index_finds_abbreviated_names_and_ignores_corrupt_fonts() {
             let root = tempfile::tempdir().unwrap();
diff --git a/src/convert/mod.rs b/src/convert/mod.rs
index d900e05..8e75298 100644
--- a/src/convert/mod.rs
+++ b/src/convert/mod.rs
@@ -6617,8 +6617,7 @@ fn parse_numbering_xml(xml: &str, media: impl Fn(&str) -> Option<Vec<u8>>) -> Nu
         let mut lvls = HashMap::new();
         for lvl in dom.descendants(abs, Some(&W::name("lvl"))) {
             let ilvl = attr_any(&dom, lvl, "ilvl")
-                .and_then(|s| s.parse().ok())
-                .unwrap_or(0);
+                .map_or(0, parse_ilvl);
             if let Some(v) = first_named(&dom, lvl, "lvlRestart")
                 .and_then(|n| attr_any(&dom, n, "val"))
                 .and_then(|s| s.parse().ok())
@@ -6674,8 +6673,7 @@ fn parse_numbering_xml(xml: &str, media: impl Fn(&str) -> Option<Vec<u8>>) -> Nu
             .into_iter()
             .filter_map(|ov| {
                 let ilvl = attr_any(&dom, ov, "ilvl")
-                    .and_then(|s| s.parse().ok())
-                    .unwrap_or(0);
+                    .map_or(0, parse_ilvl);
                 first_named(&dom, ov, "lvl").map(|lvl| (ilvl, parse_num_level(&dom, lvl)))
             })
             .collect();
@@ -6712,8 +6710,7 @@ fn parse_numbering_xml(xml: &str, media: impl Fn(&str) -> Option<Vec<u8>>) -> Nu
         numbering.instances.insert(nid.to_string(), aid);
         for ov in dom.descendants(num, Some(&W::name("lvlOverride"))) {
             let ilvl = attr_any(&dom, ov, "ilvl")
-                .and_then(|s| s.parse().ok())
-                .unwrap_or(0);
+                .map_or(0, parse_ilvl);
             let start = first_named(&dom, ov, "startOverride")
                 .and_then(|n| attr_any(&dom, n, "val"))
                 .and_then(|s| s.parse().ok())
@@ -12704,14 +12701,22 @@ fn resolve_num_pr(
     (None, 0)
 }
 
+/// An `w:ilvl` attribute inside Word's supported domain. Word defines nine
+/// numbering levels (0 through 8), and the level maps, fallback searches
+/// and counter loops stay inside that domain; a value past it is clamped
+/// rather than iterated over, so an attribute cannot size numbering work.
+fn parse_ilvl(val: &str) -> u32 {
+    const MAX_ILVL: u32 = 8;
+    val.parse::<u32>().unwrap_or(0).min(MAX_ILVL)
+}
+
 fn num_pr(dom: &Dom, ppr: NodeId) -> (Option<String>, u32) {
     let Some(num) = first_named(dom, ppr, "numPr") else {
         return (None, 0);
     };
     let ilvl = first_named(dom, num, "ilvl")
         .and_then(|n| dom.attribute(n, &W::val()))
-        .and_then(|v| v.parse::<u32>().ok())
-        .unwrap_or(0);
+        .map_or(0, parse_ilvl);
     let num_id = first_named(dom, num, "numId")
         .and_then(|n| dom.attribute(n, &W::val()))
         .map(str::to_string);
@@ -13213,7 +13218,7 @@ fn table_block(
                 p => p,
             };
             cells.push(grid_skip_cell(span, pref, tbl_pad_l, tbl_pad_r));
-            grid_at += span;
+            grid_at = grid_at.saturating_add(span);
         }
         for cell in wrapped_children(dom, row, "tc") {
             row_has_cell_del |= cell_is_deleted(dom, cell);
@@ -13601,7 +13606,12 @@ fn table_block(
     let mut occupancy = 0usize;
     let grid_len = cols.len();
     for row in &raw_rows {
-        occupancy = occupancy.max(row.iter().map(|c| c.colspan).sum());
+        // Checked and bounded: a row's spans size the grid vectors below,
+        // and no Word table spans more grid columns than MAX_GRID_SPAN.
+        let held = row
+            .iter()
+            .fold(0usize, |sum, c| sum.saturating_add(c.colspan));
+        occupancy = occupancy.max(held.min(MAX_GRID_SPAN));
     }
     if cols.len() < occupancy {
         if cols.len() == 1 && occupancy > 1 {
@@ -14172,7 +14182,9 @@ fn cell_pad_tb(dom: &Dom, cell: NodeId, table_t: f32, table_b: f32) -> (f32, f32
 fn stretch_stale_spans(rows: &mut [Vec<RawCell>], cols: &[f32]) {
     const STALE_SPAN_SLACK_PT: f32 = 20.0;
     for row in rows.iter_mut() {
-        let used: usize = row.iter().map(|c| c.colspan.max(1)).sum();
+        let used: usize = row
+            .iter()
+            .fold(0usize, |sum, c| sum.saturating_add(c.colspan.max(1)));
         if used >= cols.len() {
             continue;
         }
@@ -14185,7 +14197,8 @@ fn stretch_stale_spans(rows: &mut [Vec<RawCell>], cols: &[f32]) {
                 let mut extra = 0usize;
                 while extra < spare
                     && want - covered > STALE_SPAN_SLACK_PT
-                    && let Some(&next) = cols.get(col + span + extra)
+                    && let Some(&next) =
+                        cols.get(col.saturating_add(span).saturating_add(extra))
                     && (covered + next - want).abs() < (want - covered).abs()
                 {
                     covered += next;
@@ -14196,11 +14209,17 @@ fn stretch_stale_spans(rows: &mut [Vec<RawCell>], cols: &[f32]) {
                     spare -= extra;
                 }
             }
-            col += cell.colspan.max(1);
+            col = col.saturating_add(cell.colspan.max(1));
         }
     }
 }
 
+/// The widest grid any Word table lays out: Word's own UI stops at 63
+/// columns, and a `gridSpan`, `gridBefore` or `gridAfter` naming more
+/// columns than this is not a Word table. Every span-derived vector, loop
+/// and merge slot is bounded by this, never sized by the attribute.
+const MAX_GRID_SPAN: usize = 1024;
+
 fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
     let Some(pr) = first_named(dom, cell, "tcPr") else {
         return (1, VMerge::None);
@@ -14211,8 +14230,7 @@ fn cell_span(dom: &Dom, cell: NodeId) -> (usize, VMerge) {
     let colspan = direct_named(dom, pr, "gridSpan")
         .and_then(|n| attr_any(dom, n, "val"))
         .and_then(|s| s.parse().ok())
-        .unwrap_or(1)
-        .max(1);
+        .map_or(1, |span| span.clamp(1, MAX_GRID_SPAN));
     let vmerge = match direct_named(dom, pr, "vMerge") {
         None => VMerge::None,
         Some(n) => match dom.attribute(n, &W::val()) {
@@ -14270,21 +14288,21 @@ fn column_prefs(raw_rows: &[Vec<RawCell>], grid: &[f32], fixed: bool) -> Vec<Pre
         // A gridBefore/gridAfter placeholder is no cell: its grid width is
         // the row's own math, not a preference for the column.
         if cell.grid_skip {
-            col += span;
+            col = col.saturating_add(span);
             continue;
         }
-        let covered: f32 = (col..col + span)
+        let covered: f32 = (col..col.saturating_add(span))
             .map(|i| grid.get(i).copied().unwrap_or(0.0))
             .sum();
         let share = |i: usize| {
             if covered > 0.0 {
-                grid.get(col + i).copied().unwrap_or(0.0) / covered
+                grid.get(col.saturating_add(i)).copied().unwrap_or(0.0) / covered
             } else {
                 1.0 / span as f32
             }
         };
         for i in 0..span {
-            let Some(slot) = pref.get_mut(col + i) else {
+            let Some(slot) = pref.get_mut(col.saturating_add(i)) else {
                 continue;
             };
             match cell.pref {
@@ -14293,7 +14311,7 @@ fn column_prefs(raw_rows: &[Vec<RawCell>], grid: &[f32], fixed: bool) -> Vec<Pre
                 PrefWidth::Auto => {}
             }
         }
-        col += span;
+        col = col.saturating_add(span);
     }
     for row in raw_rows.iter().skip(1).filter(|_| fixed) {
         let mut col = 0usize;
@@ -14314,7 +14332,7 @@ fn column_prefs(raw_rows: &[Vec<RawCell>], grid: &[f32], fixed: bool) -> Vec<Pre
                     _ => {}
                 }
             }
-            col += span;
+            col = col.saturating_add(span);
         }
     }
     pref
@@ -14342,6 +14360,7 @@ fn row_grid_skip(dom: &Dom, row: NodeId, side: &str) -> Option<(usize, PrefWidth
     let span = direct_named(dom, pr, &format!("grid{side}"))
         .and_then(|n| attr_any(dom, n, "val"))
         .and_then(|v| v.parse::<usize>().ok())
+        .map(|n| n.min(MAX_GRID_SPAN))
         .filter(|n| *n > 0)?;
     let pref = direct_named(dom, pr, &format!("w{side}"))
         .filter(|n| attr_any(dom, *n, "type").is_none_or(|t| t == "dxa"))
@@ -14607,7 +14626,7 @@ fn resolve_table_merges(raw_rows: Vec<Vec<RawCell>>) -> Vec<Vec<TableCell>> {
                 {
                     origin.rowspan += 1;
                 }
-                col += span;
+                col = col.saturating_add(span);
                 continue;
             }
             let idx = origins[ri].len();
@@ -14634,14 +14653,15 @@ fn resolve_table_merges(raw_rows: Vec<Vec<RawCell>>) -> Vec<Vec<TableCell>> {
                 borders: raw.borders,
                 style_fill: false,
             });
-            if open.len() < col + span {
-                open.resize(col + span, None);
+            let end = col.saturating_add(span).min(MAX_GRID_SPAN);
+            if open.len() < end {
+                open.resize(end, None);
             }
             let mark = (raw.vmerge == VMerge::Restart).then_some((col, ri, idx));
             for slot in open.iter_mut().skip(col).take(span) {
                 *slot = mark;
             }
-            col += span;
+            col = col.saturating_add(span);
         }
     }
     origins
diff --git a/src/legacy_doc.rs b/src/legacy_doc.rs
index 1881053..0bfbeda 100644
--- a/src/legacy_doc.rs
+++ b/src/legacy_doc.rs
@@ -1594,6 +1594,11 @@ impl Story<'_> {
         // Field nesting: true while inside a field's code (before its
         // separator).
         let mut fields: Vec<bool> = Vec::new();
+        // How many frames are currently in their code phase. Asking
+        // `fields.iter().any` of every character made a story of
+        // begin+separator pairs quadratic; the count answers the same
+        // question in constant time.
+        let mut code_frames = 0usize;
         // Each finished row and whether it is marked as a header.
         let mut rows: Vec<(Vec<String>, bool)> = Vec::new();
         let mut row: Vec<String> = Vec::new();
@@ -1601,20 +1606,26 @@ impl Story<'_> {
 
         for story in chars {
             // Inside any field's code, even a nested field's result is code.
-            let in_code = fields.iter().any(|&code| code);
+            let in_code = code_frames > 0;
             let text = match story.ch {
                 '\u{13}' => {
                     fields.push(true);
+                    code_frames += 1;
                     continue;
                 }
                 '\u{14}' => {
                     if let Some(top) = fields.last_mut() {
+                        if *top {
+                            code_frames -= 1;
+                        }
                         *top = false;
                     }
                     continue;
                 }
                 '\u{15}' => {
-                    fields.pop();
+                    if fields.pop() == Some(true) {
+                        code_frames -= 1;
+                    }
                     continue;
                 }
                 _ if in_code => continue,
@@ -1967,6 +1978,36 @@ mod tests {
         assert_eq!(texts, ["aoneb"]);
     }
 
+    #[test]
+    fn deeply_nested_field_marks_stay_linear_and_keep_the_code_rule() {
+        // 600k begin+separator pairs leave an all-result stack; scanning
+        // every frame per character made this quadratic, and a field opened
+        // inside code still hides its text until its own end marker.
+        let (papx, chpx) = (FkpIndex { runs: Vec::new() }, FkpIndex { runs: Vec::new() });
+        let lists = Lists::default();
+        let mut text = String::with_capacity(1_200_010);
+        for _ in 0..600_000 {
+            text.push('\u{13}');
+            text.push('\u{14}');
+        }
+        text.push_str("shown\u{13}hidden\u{15}\r");
+        let chars = story_chars(&text);
+        let story = Story {
+            papx: &papx,
+            chpx: &chpx,
+            styles: &[],
+            lists: &lists,
+            section_marks: Vec::new(),
+        };
+        let blocks = story.blocks(&chars);
+        assert_eq!(blocks.len(), 1, "one paragraph closes after the field ends");
+        let text = match &blocks[0] {
+            Block::Paragraph(paragraph) => paragraph.text(),
+            Block::Table(_) => panic!("the story closes as one paragraph"),
+        };
+        assert_eq!(text, "shown");
+    }
+
     /// A supplementary character split across two Unicode pieces is one
     /// character, as Word reads it; a lone high surrogate is a replacement
     /// mark, never dropped.
diff --git a/src/markdown/from_docx/drawing.rs b/src/markdown/from_docx/drawing.rs
index a944602..53a823f 100644
--- a/src/markdown/from_docx/drawing.rs
+++ b/src/markdown/from_docx/drawing.rs
@@ -1887,11 +1887,14 @@ pub(crate) fn materialize(
     next_doc_pr: &mut u32,
 ) -> Option<String> {
     let mut xml = drawing.xml.clone();
+    let mut chain = Vec::new();
     for rel in &drawing.rels {
         let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
             (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
             (_, Some(index), _) if index < drawing.deps.len() => {
-                let name = dep_part(drawing, index, package, existing, used, next_doc_pr)?;
+                let name = dep_part(
+                    drawing, index, package, existing, used, next_doc_pr, &mut chain,
+                )?;
                 (rel.kind.clone(), relative_rel_target(part, &name), false)
             }
             (_, _, Some(raw)) => (rel.kind.clone(), relative_rel_target(part, raw), false),
@@ -1951,6 +1954,11 @@ fn renumber_doc_pr(xml: &mut String, next: &mut u32) {
     *xml = out;
 }
 
+/// The deepest chain of dependent parts a drawing may materialize. Word's
+/// own closures (a chart, its colors and its style) are three levels;
+/// anything past this is a hand-built graph, refused before recursion.
+const MAX_DEPENDENCY_DEPTH: usize = 64;
+
 /// Write dependent part `index` and its own dependency closure; its new
 /// part name.
 fn dep_part(
@@ -1960,10 +1968,18 @@ fn dep_part(
     existing: &HashSet<String>,
     used: &mut HashSet<String>,
     next_doc_pr: &mut u32,
+    chain: &mut Vec<usize>,
 ) -> Option<String> {
+    // An imported graph may point anywhere, including back at a part on
+    // this chain: refuse the revisit (or a chain deeper than any real
+    // dependency closure) instead of recursing on it.
+    if chain.len() >= MAX_DEPENDENCY_DEPTH || chain.contains(&index) {
+        return None;
+    }
     let dep = drawing.deps.get(index)?;
     let name = name_for(&dep.name, existing, used);
     used.insert(name.clone());
+    chain.push(index);
     // The part's own relationships name the parts the recursion is writing;
     // their ids land in a rewrite of its `d{index}.{n}` symbols.
     let mut xml = dep.xml.clone();
@@ -1971,7 +1987,8 @@ fn dep_part(
         let (kind, target, external) = match (&rel.u, rel.part, &rel.raw) {
             (Some(url), _, _) => (rel.kind.clone(), url.clone(), true),
             (_, Some(inner), _) if inner < drawing.deps.len() => {
-                let inner_name = dep_part(drawing, inner, package, existing, used, next_doc_pr)?;
+                let inner_name =
+                    dep_part(drawing, inner, package, existing, used, next_doc_pr, chain)?;
                 (
                     rel.kind.clone(),
                     relative_rel_target(&name, &inner_name),
@@ -1992,6 +2009,7 @@ fn dep_part(
         renumber_doc_pr(&mut xml, next_doc_pr);
         format!("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n{xml}").into_bytes()
     };
+    chain.pop();
     // Two payloads may depend on the one part (two drawings share an
     // image): a part already written byte for byte is the same one, not
     // a collision to rename around.
@@ -2097,6 +2115,127 @@ mod tests {
         assert!(at("![a][d1").is_none());
     }
 
+    fn internal(part: usize, sym: &str) -> PayRel {
+        PayRel {
+            sym: sym.into(),
+            kind: "http://schemas.openxmlformats.org/officeDocument/2006/relationships/image"
+                .into(),
+            part: Some(part),
+            ..PayRel::default()
+        }
+    }
+
+    fn part_dep(name: &str, rels: Vec<PayRel>) -> Dep {
+        Dep {
+            name: name.into(),
+            kind: "image/png".into(),
+            b64: base64_encode(name.as_bytes()),
+            rels,
+            ..Dep::default()
+        }
+    }
+
+    fn payload(rels: Vec<PayRel>, deps: Vec<Dep>) -> Drawing {
+        Drawing {
+            rels,
+            deps,
+            ..Drawing::default()
+        }
+    }
+
+    #[test]
+    fn cyclic_and_overdeep_drawing_dependencies_are_refused() {
+        // A dependency graph may point anywhere, including back at the
+        // part being written, or in a chain no Word package has: both are
+        // refused before any recursion, while an acyclic chain still
+        // materializes its parts.
+        fn run(
+            drawing: &Drawing,
+            package: &mut PartFs,
+            used: &mut HashSet<String>,
+            next: &mut u32,
+        ) -> Option<String> {
+            let existing = HashSet::new();
+            materialize(drawing, package, "word/document.xml", &existing, used, next)
+        }
+        let mut package = PartFs::new();
+        let mut used = HashSet::new();
+        let mut next = 1;
+        // A one-node self-reference.
+        assert!(
+            run(
+                &payload(
+                    vec![internal(0, "r1")],
+                    vec![part_dep("media/image1.png", vec![internal(0, "d0.1")])]
+                ),
+                &mut package,
+                &mut used,
+                &mut next
+            )
+            .is_none(),
+            "a self-referencing dependency is refused"
+        );
+        // A two-node cycle.
+        assert!(
+            run(
+                &payload(
+                    vec![internal(0, "r1")],
+                    vec![
+                        part_dep("media/image1.png", vec![internal(1, "d0.1")]),
+                        part_dep("media/image2.png", vec![internal(0, "d1.1")]),
+                    ]
+                ),
+                &mut package,
+                &mut used,
+                &mut next
+            )
+            .is_none(),
+            "a two-node dependency cycle is refused"
+        );
+        // A chain deeper than any real dependency closure.
+        let deep: Vec<Dep> = (0..70)
+            .map(|i| {
+                part_dep(
+                    &format!("media/part{i}.xml"),
+                    if i == 69 {
+                        Vec::new()
+                    } else {
+                        vec![internal(i + 1, "d0.1")]
+                    },
+                )
+            })
+            .collect();
+        assert!(
+            run(&payload(vec![internal(0, "r1")], deep), &mut package, &mut used, &mut next)
+                .is_none(),
+            "an over-deep dependency chain is refused"
+        );
+        // A bounded acyclic chain still materializes every part.
+        used.clear();
+        let three = run(
+            &payload(
+                vec![internal(0, "r1"), internal(1, "r2")],
+                vec![
+                    part_dep("media/image1.png", vec![internal(2, "d0.1")]),
+                    part_dep("media/image2.png", Vec::new()),
+                    part_dep("media/image3.png", Vec::new()),
+                ],
+            ),
+            &mut package,
+            &mut used,
+            &mut next,
+        )
+        .expect("an acyclic dependency graph materializes");
+        assert_eq!(three, "");
+        for name in ["media/image1.png", "media/image2.png", "media/image3.png"] {
+            assert_eq!(
+                package.part_bytes(name),
+                Some(name.as_bytes()),
+                "{name} carries its own bytes"
+            );
+        }
+    }
+
     #[test]
     fn unchanged_blank_box_keeps_controls_and_remains_editable() {
         let mut root = element(
diff --git a/src/markdown/package.rs b/src/markdown/package.rs
index 9162eb3..f969b3d 100644
--- a/src/markdown/package.rs
+++ b/src/markdown/package.rs
@@ -110,6 +110,16 @@ pub(super) fn assemble(
         Some(bytes) => bytes.to_vec(),
         None => default_template(options.page).map_err(err)?,
     };
+    // A reference is a conversion input and is admitted like one: its
+    // parts reach the section, style, separator and revision parsers, and
+    // the XML depth and DTD budgets refuse a template before any
+    // recursive parser sees it. The Python, WASM and CLI reference paths
+    // share this gate through `markdown_to_docx`.
+    if let Some(bytes) = options.reference {
+        crate::document_comparer::admit_package(bytes).map_err(|e| {
+            MarkdownError::Reference(format!("not a reference jubarte can read ({e})"))
+        })?;
+    }
     let mut package = PartFs::open(&template)
         .map_err(|e| MarkdownError::Reference(format!("not a readable .docx ({e})")))?;
     let main = package
diff --git a/src/markdown/view_notes.rs b/src/markdown/view_notes.rs
index adcd567..0183b1e 100644
--- a/src/markdown/view_notes.rs
+++ b/src/markdown/view_notes.rs
@@ -1015,8 +1015,13 @@ pub(super) fn table_line(line: &str) -> Option<TableLine> {
                 let (first, last) = range.split_once('-').unwrap_or((range, range));
                 let first = crate::ids::short_number(first, 'p')?;
                 let last = crate::ids::short_number(last, 'p')?;
+                // A `cells` row range is a paragraph range like the
+                // `empty` idiom's and is bounded the same way: a
+                // hand-edited `p0-p1000000000` must not become a billion
+                // padding cells.
                 last.checked_sub(first)?
                     .checked_add(1)
+                    .filter(|&count| count <= MOST_EMPTY)
                     .map(|count| (first, count))
             })
             .collect();
@@ -1702,6 +1707,22 @@ mod tests {
         assert_eq!(line.row_paragraphs, Some(vec![(0, 3), (3, 1)]));
     }
 
+    #[test]
+    fn a_table_cells_range_is_bounded_like_an_empty_range() {
+        // The `empty` idiom bounds a hand-edited range at MOST_EMPTY
+        // paragraphs; a `cells` row range must not pad a row by a billion
+        // cells either.
+        for line in [
+            "<!-- t0 1x1, cells t0.row0 p0-p100000 -->",
+            "<!-- t0 1x1, cells t0.row0 p0-p99999 t0.row1 p0-p100000 -->",
+        ] {
+            assert_eq!(table_line(line).unwrap().row_paragraphs, None, "{line}");
+        }
+        let line =
+            table_line("<!-- t0 1x1, cells t0.row0 p0-p99999 t0.row1 p100000 -->").unwrap();
+        assert_eq!(line.row_paragraphs, Some(vec![(0, 100_000), (100_000, 1)]));
+    }
+
     #[test]
     fn latest_is_not_a_shared_date_or_a_range() {
         let yaml = "authors:\n  AL: Ann Lee  # 2 revisions, latest 2026-10-17T16:59:00Z\n";
diff --git a/src/opc/mod.rs b/src/opc/mod.rs
index 9865bda..0216e4f 100644
--- a/src/opc/mod.rs
+++ b/src/opc/mod.rs
@@ -675,10 +675,12 @@ impl PartFs {
     /// absolute target ("/word/footer1.xml") was later re-prefixed into a
     /// "//word/…" relationship, and "word/../customXml/item1.xml" named no part.
     pub fn resolve_rel_target(&self, source_part: &str, rel_target: &str) -> String {
-        // PartFs receives raw relationship attributes; DOM consumers already
-        // decoded XML entities and call the pure resolver directly.
-        let target = crate::xmllinq::parse::unescape_xml_text(rel_target);
-        resolve_rel_target(source_part, &target)
+        // Relationship targets arrive already decoded once: the OPC reader
+        // normalizes attribute values, and programmatic relationships hold
+        // logical names. Decoding here again re-decoded literal `&amp;`
+        // part names and rescanned the suffix for every bare `&` in a
+        // target.
+        resolve_rel_target(source_part, rel_target)
     }
 
     /// `contentTypeFor(part)`.
@@ -1319,6 +1321,47 @@ mod tests {
         );
     }
 
+    #[test]
+    fn rel_targets_resolve_without_decoding_entities_a_second_time() {
+        // `Target="media/a&amp;amp;b.png"` names the literal part
+        // `media/a&amp;b.png` (one decode, by the OPC reader); resolving
+        // it must not decode the name again into a part that does not
+        // exist.
+        let fs = PartFs::open(&package_with_document_rels(
+            r#"<Relationship Id="rId7" Type="image" Target="media/a&amp;amp;b.png"/>"#,
+        ))
+        .unwrap();
+        let target = fs
+            .read_rels_for("word/document.xml")
+            .unwrap()
+            .items[0]
+            .target
+            .clone();
+        assert_eq!(target, "media/a&amp;b.png");
+        assert_eq!(
+            fs.resolve_rel_target("word/document.xml", &target),
+            "media/a&amp;b.png"
+        );
+    }
+
+    #[test]
+    fn rel_targets_full_of_ampersands_resolve_in_one_pass() {
+        // A target of a hundred thousand encoded ampersands decodes once
+        // at read; resolution must not walk (and re-decode) the suffix
+        // once per `&`.
+        let encoded = format!("media/{}b.png", "&amp;".repeat(500_000));
+        let xml = format!(r#"<Relationship Id="rId7" Type="image" Target="{encoded}"/>"#);
+        let fs = PartFs::open(&package_with_document_rels(&xml)).unwrap();
+        let target = fs
+            .read_rels_for("word/document.xml")
+            .unwrap()
+            .items[0]
+            .target
+            .clone();
+        assert_eq!(target, format!("media/{}b.png", "&".repeat(500_000)));
+        assert_eq!(fs.resolve_rel_target("word/document.xml", &target), target);
+    }
+
     #[test]
     fn rel_targets_are_relative_to_the_source_folder() {
         assert_eq!(
diff --git a/src/xmllinq/parse.rs b/src/xmllinq/parse.rs
index 236c54c..c40ce95 100644
--- a/src/xmllinq/parse.rs
+++ b/src/xmllinq/parse.rs
@@ -765,13 +765,20 @@ pub fn unescape_xml_text(s: &str) -> String {
     if !s.contains('&') {
         return s.to_string();
     }
+    // An entity this decoder resolves is a few characters long (`#x10FFFF;`
+    // is the longest predefined shape, and only zero-padded numeric
+    // references run past it); a `;` beyond the window is not the end of
+    // one. Scanning the whole remaining input for every `&` made runs of
+    // bare ampersands quadratic.
+    const MAX_ENTITY_CHARS: usize = 64;
     let chars: Vec<char> = s.chars().collect();
     let mut out = String::with_capacity(s.len());
     let mut i = 0;
     while i < chars.len() {
         if chars[i] == '&' {
             // find ';'
-            if let Some(semi_rel) = chars[i + 1..].iter().position(|&c| c == ';') {
+            let limit = (i + 1 + MAX_ENTITY_CHARS).min(chars.len());
+            if let Some(semi_rel) = chars[i + 1..limit].iter().position(|&c| c == ';') {
                 let body: String = chars[i + 1..i + 1 + semi_rel].iter().collect();
                 if let Some(decoded) = decode_entity(&body) {
                     out.push_str(&decoded);
@@ -1069,6 +1076,20 @@ mod xml_lexical_boundary_contract_tests {
         }
     }
 
+    #[test]
+    fn long_bare_ampersand_runs_decode_in_one_pass() {
+        // The search for a `;` reaches only the entity window past the
+        // `&`, never the whole remaining input: bare ampersands (with the
+        // only `;` far beyond them) stay linear instead of quadratic.
+        let mut input = "&".repeat(500_000);
+        input.push('x');
+        input.push(';');
+        assert_eq!(unescape_xml_text(&input), input);
+        // A body past the window is literal, exactly like an unknown name.
+        let over = format!("&{};", "a".repeat(100));
+        assert_eq!(unescape_xml_text(&over), over);
+    }
+
     #[test]
     fn literal_attribute_whitespace_reads_as_spaces_references_survive() {
         // XML 1.0 §2.11 + §3.3.3, the value a conforming reader (Word) sees:
diff --git a/tests/convert_docx_to_pdf.rs b/tests/convert_docx_to_pdf.rs
index 65815f1..01b4088 100644
--- a/tests/convert_docx_to_pdf.rs
+++ b/tests/convert_docx_to_pdf.rs
@@ -6257,6 +6257,82 @@ fn numbering_is_lgl_paints_parent_slots_as_decimal() {
     );
 }
 
+#[test]
+fn an_ilvl_past_word_s_nine_levels_resolves_in_bounded_work() {
+    // `w:ilvl` accepts any u32, but Word defines nine levels (0 through
+    // 8). A level past the domain falls back like an undefined one
+    // instead of scanning and counting the levels below it.
+    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
+        <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
+          <w:abstractNum w:abstractNumId=\"0\">\
+            <w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/>\
+              <w:lvlText w:val=\"%1.\"/>\
+              <w:pPr><w:ind w:left=\"360\" w:hanging=\"360\"/></w:pPr></w:lvl>\
+          </w:abstractNum>\
+          <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
+        </w:numbering>";
+    for ilvl in ["9", "1000000000", "4294967295"] {
+        let body = format!(
+            "<w:p><w:pPr><w:numPr><w:ilvl w:val=\"{ilvl}\"/><w:numId w:val=\"1\"/>\
+             </w:numPr></w:pPr><w:r><w:t>DeepHL</w:t></w:r></w:p><w:sectPr/>"
+        );
+        let pdf = docx_to_pdf(&numbering_docx(&body, Some(numbering)))
+            .unwrap_or_else(|e| panic!("ilvl {ilvl} converts: {e}"));
+        let text = pdf_winansi_text(&pdf);
+        let one = text
+            .find("1.")
+            .unwrap_or_else(|| panic!("the level-0 marker paints for ilvl {ilvl}; text={text:?}"));
+        let at = text.find("DeepHL").expect("DeepHL");
+        assert!(one < at, "ilvl {ilvl} numbers at level 0; text={text:?}");
+    }
+}
+
+#[test]
+fn a_numbering_level_defined_past_word_s_domain_stays_bounded() {
+    // The abstract defines and the paragraph asks for level 4294967294:
+    // both clamp into Word's 0..=8 domain, so the counters initialize
+    // for nine levels, not four billion.
+    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\
+        <w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\">\
+          <w:abstractNum w:abstractNumId=\"0\">\
+            <w:lvl w:ilvl=\"4294967294\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/>\
+              <w:lvlText w:val=\"%1.\"/>\
+              <w:pPr><w:ind w:left=\"360\" w:hanging=\"360\"/></w:pPr></w:lvl>\
+          </w:abstractNum>\
+          <w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num>\
+        </w:numbering>";
+    let body = "<w:p><w:pPr><w:numPr><w:ilvl w:val=\"4294967294\"/><w:numId w:val=\"1\"/>\
+           </w:numPr></w:pPr><w:r><w:t>DeepDF</w:t></w:r></w:p><w:sectPr/>";
+    let pdf = docx_to_pdf(&numbering_docx(body, Some(numbering)))
+        .expect("a level clamped into Word's domain converts");
+    let text = pdf_winansi_text(&pdf);
+    assert!(text.contains("DeepDF"), "the numbered body paints; text={text:?}");
+}
+
+#[test]
+fn a_table_with_absurd_grid_spans_and_skips_stays_bounded() {
+    // A hand-edited `w:gridSpan`/`w:gridBefore` names more columns than
+    // any Word table has (Word's UI stops at 63): the span must bound
+    // the grid vectors, not size them. Unpatched, these values asked
+    // for gigabytes of columns before the first page existed.
+    for span in ["1000000000", "1073741824", "4294967295"] {
+        let body = format!(
+            "<w:tbl><w:tblGrid><w:gridCol w:w=\"96\"/></w:tblGrid>\
+             <w:tr><w:trPr><w:gridBefore w:val=\"{span}\"/></w:trPr>\
+             <w:tc><w:tcPr><w:gridSpan w:val=\"{span}\"/></w:tcPr>\
+             <w:p><w:r><w:t>SpanBoom</w:t></w:r></w:p></w:tc></w:tr></w:tbl>\
+             <w:p><w:r><w:t>Tail</w:t></w:r></w:p><w:sectPr/>"
+        );
+        let pdf = docx_to_pdf(&numbering_docx(&body, None))
+            .unwrap_or_else(|e| panic!("grid span {span} converts: {e}"));
+        assert_eq!(
+            pdf_page_count(&pdf),
+            1,
+            "a bounded span keeps the tiny table on one page; span={span}"
+        );
+    }
+}
+
 #[test]
 fn numbering_num_style_link_binds_instance_to_style_link_abstract() {
     // xml_parts_plan: w:num/w:numStyleLink looks up the abstractNum whose
diff --git a/tests/markdown_to_docx.rs b/tests/markdown_to_docx.rs
index 95d127c..d7ce887 100644
--- a/tests/markdown_to_docx.rs
+++ b/tests/markdown_to_docx.rs
@@ -1022,6 +1022,32 @@ fn a_reference_that_is_not_a_docx_is_refused() {
     assert!(err.to_string().starts_with("reference document:"), "{err}");
 }
 
+#[test]
+fn a_reference_whose_xml_nests_past_the_budget_is_refused() {
+    // A reference's parts reach the recursive section, style and
+    // separator parsers; it is admitted like any conversion input first.
+    // 100k nested elements used to reach the recursive DOM parser
+    // unguarded.
+    let nest = format!(
+        "{}{}",
+        "<w:nest>".repeat(100_000),
+        "</w:nest>".repeat(100_000)
+    );
+    let reference = common::docx::docx(&nest);
+    let err = markdown_to_docx(
+        "Text.\n",
+        &DocxOptions {
+            reference: Some(&reference),
+            ..DocxOptions::default()
+        },
+    )
+    .unwrap_err();
+    assert!(
+        err.to_string().starts_with("reference document:"),
+        "{err}"
+    );
+}
+
 #[test]
 fn a_whole_block_after_a_changed_break_keeps_its_own_mark() {
     // The inserted break is the only paragraph mark the markup adds: B's

```
