import pathlib, sys

def sub(text, old, new, label):
    n = text.count(old)
    if n != 1:
        print(f"MISMATCH {label} ({n})", file=sys.stderr)
        sys.exit(1)
    return text.replace(old, new)

# --- Python CLI: exclusive unpredictable staging ---
p = pathlib.Path("new/jubarte-python/python/jubarte_redlines/__main__.py")
text = p.read_text()
text = sub(text, """    if not dry_run:
        # Through a symbolic link to the file it names, written yet or not;
        # via a hidden partial file named for this process, removed if the
        # write fails; bytes as they are, so no platform newline translation.
        target = Path(os.path.realpath(plan))
        partial = target.with_name(f".{target.name}.{os.getpid()}.partial")
        try:
            partial.write_bytes(recorded["json"].encode("utf-8"))
            os.replace(partial, target)
        except OSError as exc:
            partial.unlink(missing_ok=True)
            raise CliError(f"writing {plan}: {exc}") from exc""",
"""    if not dry_run:
        # Through a symbolic link to the file it names, written yet or not;
        # via a hidden partial file beside it, removed if the write fails;
        # bytes as they are, so no platform newline translation. The partial
        # is created exclusively under an unpredictable name, as the native
        # publisher stages it (docs/FILESYSTEM_PUBLICATION.md): a local
        # writer of the directory cannot preplace it with a link to a
        # victim file this process may write.
        target = Path(os.path.realpath(plan))
        partial = target.with_name(f".{target.name}.{os.urandom(16).hex()}.partial")
        try:
            fd = os.open(partial, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o666)
            try:
                with os.fdopen(fd, "wb") as f:
                    f.write(recorded["json"].encode("utf-8"))
            except BaseException:
                partial.unlink(missing_ok=True)
                raise
            os.replace(partial, target)
        except OSError as exc:
            partial.unlink(missing_ok=True)
            raise CliError(f"writing {plan}: {exc}") from exc""", "python _record")
p.write_text(text)
print("__main__.py applied")

# --- Python test ---
p = pathlib.Path("new/jubarte-python/tests/test_cli.py")
text = p.read_text()
anchor = """    assert not plan.exists(), "--consume deletes the applied plan"
    out = capsys.readouterr().out
    assert "{~~his or her~>their~~}" in out and "Why 3?" in out, out


def test_plan_paths_as_typed_and_strict_utf8_as_the_binary("""
addition = """    assert not plan.exists(), "--consume deletes the applied plan"
    out = capsys.readouterr().out
    assert "{~~his or her~>their~~}" in out and "Why 3?" in out, out


def test_plan_recording_uses_exclusive_staging_a_preplaced_link_cannot_divert(letter: Path, tmp_path: Path, capsys: pytest.CaptureFixture[str]) -> None:
    # The recorder's old partial name was the plan's basename and this
    # process's pid; a local writer of the directory could preplace it with
    # a symlink to a victim this process may write, and the recording
    # truncated that victim. Staging is now an exclusive create under an
    # unpredictable name (native publication's design), so only the plan
    # is touched.
    plan = tmp_path / "plan.json"
    victim = tmp_path / "victim.txt"
    victim.write_text("do not touch")
    planted = tmp_path / f".plan.json.{os.getpid()}.partial"
    planted.symlink_to(victim)
    assert main(["edit", str(letter), "--plan", str(plan), "-p", "p1", "--content", "Their."]) == 0
    capsys.readouterr()
    assert victim.read_text() == "do not touch"
    assert planted.is_symlink(), "the planted link itself is untouched"
    planted.unlink()
    assert plan.is_file() and len(json.loads(plan.read_text())["operations"]) == 1
    assert not list(tmp_path.glob(".*.partial")), "staging is renamed into the plan"


def test_plan_paths_as_typed_and_strict_utf8_as_the_binary("""
text = sub(text, anchor, addition, "python test anchor")
p.write_text(text)
print("test_cli.py applied")

# --- Node bin: crypto import + exclusive staging ---
p = pathlib.Path("new/jubarte-wasm/cli/bin/jubarte-redlines.mjs")
text = p.read_text()
text = sub(text, """import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";""",
"""import crypto from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";""", "node imports")
text = sub(text, """  if (!o.dry_run) {
    // Through a symbolic link to the file it names, written yet or not; via a hidden partial
    // file named for this process, removed if the write fails.
    let target = o.plan;
    try {
      target = fs.realpathSync(o.plan);
    } catch {
      // A link to a plan not written yet names where it goes.
      try {
        target = path.resolve(path.dirname(o.plan), fs.readlinkSync(o.plan));
      } catch {}
    }
    const partial = path.join(path.dirname(target), `.${path.basename(target)}.${process.pid}.partial`);
    try {
      fs.writeFileSync(partial, recorded.json);
      fs.renameSync(partial, target);
    } catch (e) {
      fs.rmSync(partial, { force: true });
      throw new CliError(`writing ${o.plan}: ${osError(e)}`);
    }
  }""",
"""  if (!o.dry_run) {
    // Through a symbolic link to the file it names, written yet or not; via a hidden partial
    // file beside it, removed if the write fails. The partial is created exclusively under
    // an unpredictable name, as the native publisher stages it
    // (docs/FILESYSTEM_PUBLICATION.md): a local writer of the directory cannot preplace it
    // with a link to a victim file this process may write.
    let target = o.plan;
    try {
      target = fs.realpathSync(o.plan);
    } catch {
      // A link to a plan not written yet names where it goes.
      try {
        target = path.resolve(path.dirname(o.plan), fs.readlinkSync(o.plan));
      } catch {}
    }
    const partial = path.join(path.dirname(target), `.${path.basename(target)}.${crypto.randomUUID()}.partial`);
    try {
      const fd = fs.openSync(partial, "wx");
      try {
        fs.writeFileSync(fd, recorded.json);
      } finally {
        fs.closeSync(fd);
      }
      fs.renameSync(partial, target);
    } catch (e) {
      fs.rmSync(partial, { force: true });
      throw new CliError(`writing ${o.plan}: ${osError(e)}`);
    }
  }""", "node recordPlan")
p.write_text(text)
print("jubarte-redlines.mjs applied")

# --- Node test ---
p = pathlib.Path("new/jubarte-wasm/cli/test/cli.test.mjs")
text = p.read_text()
anchor = """  const view = run("read", path.join(dir, "redline.docx")).out;
  assert.ok(view.includes(`{~~${word}~>Changed~~}`) && view.includes("Why?"), view);
});

test("--consume closes the plan descriptor when applyEdit throws (integration)", () => {"""
addition = """  const view = run("read", path.join(dir, "redline.docx")).out;
  assert.ok(view.includes(`{~~${word}~>Changed~~}`) && view.includes("Why?"), view);
});

test("plan recording stages exclusively: a preplaced partial symlink cannot divert it", () => {
  // The recorder's old partial name was the plan's basename and the recorder's pid; a
  // local writer of the directory could preplace it with a symlink to a victim the
  // process may write, and the recording truncated that victim. Staging is now an
  // exclusive create under an unpredictable name (native publication's design), so
  // only the plan is touched. The hook runs in the recorder's own process, so
  // process.pid names the pre-fix partial exactly.
  const paras = JSON.parse(run("read", untracked, "--json").out).paragraphs.filter((p) => p.text.length > 3);
  const file = copy(untracked, "exclusive.docx");
  const plan = path.join(tmp, "exclusive-plan.json");
  const victim = path.join(tmp, "exclusive-victim.txt");
  fs.writeFileSync(victim, "do not touch");
  const result = runWithPlanHook(plan, (fs, mock, assert, plan, context) => {
    const partial = `${context.dir}/.${context.base}.${process.pid}.partial`;
    fs.symlinkSync(context.victim, partial);
    return () => {
      assert.equal(fs.readFileSync(context.victim, "utf8"), "do not touch", "the planted link must not divert the recording");
      assert.ok(fs.lstatSync(partial).isSymbolicLink(), "the planted link itself is untouched");
      fs.rmSync(partial, { force: true });
      assert.deepEqual(fs.readdirSync(context.dir).filter((n) => n.endsWith(".partial")), []);
      assert.equal(JSON.parse(fs.readFileSync(plan, "utf8")).operations.length, 1);
    };
  }, ["edit", file, "--plan", plan, "-p", `p${paras[0].index}`, "--content", "Changed."], { dir: tmp, base: path.basename(plan), victim });
  assert.equal(result.code, 0, result.err);
});

test("--consume closes the plan descriptor when applyEdit throws (integration)", () => {"""
text = sub(text, anchor, addition, "node test anchor")
p.write_text(text)
print("cli.test.mjs applied")
