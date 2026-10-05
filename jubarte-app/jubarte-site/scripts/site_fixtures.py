# /// script
# requires-python = ">=3.11"
# dependencies = ["pymupdf>=1.26", "pillow>=11", "huggingface_hub>=0.34"]
# ///
"""Fixtures for the jubarte.pro Cases viewer: one set of files, on Hugging Face and on the site.

The site never shows a document Hugging Face does not hold. Publishing a new set:

    uv run scripts/site_fixtures.py stage   # pick cases, hard-link their files into .fixtures/stage
    uv run scripts/site_fixtures.py upload  # push .fixtures/stage to the dataset under site/;
                                            # writes the new revision to fixtures.lock
    uv run scripts/site_fixtures.py render  # WebP page strips + data JSON for public/

Rebuilding the site from a clean checkout needs no benchmark checkout:

    uv run scripts/site_fixtures.py fetch   # download site/ at the fixtures.lock revision
    uv run scripts/site_fixtures.py render

`stage` reads neurotic_docx_bench (``--bench``): the DOCX-to-PDF pick in
``results/site_fixtures_860.csv`` (Word export, jubarte ``--jubarte``, LibreOffice 26.8.0.3,
docxide-pdf 0.17.1) and a stratified pick of the redline bench ``redlines_0929_full``
(Word's compare, jubarte's ``--redline-tool`` lane, Docxodus, superdoc, each opened in Word).
Tool outputs the bench pruned locally come from the results dataset itself. After a jubarte
release has run through the bench (its ``scripts/release_jubarte.py``), stage again with
``--jubarte <version>`` and, once its redline lane is uploaded, ``--redline-tool jubarte-<version>``.

`render` checks every staged file against the LFS sha256 Hugging Face reports at the
uploaded revision, then rasterises the PDFs. The data JSON links each file to that pinned
revision, so a link always resolves to the exact bytes the page images were drawn from.
"""

from __future__ import annotations

import argparse
import collections
import concurrent.futures as cf
import csv
import hashlib
import io
import json
import math
import os
import random
import shutil
from pathlib import Path

REPO = "arthrod/neurotic_docx_bench"
PREFIX = "site"
HERE = Path(__file__).resolve().parent.parent
STAGE = HERE / ".fixtures" / "stage" / PREFIX
PUBLIC = HERE / "public"
LOCK = HERE / "fixtures.lock"  # committed: the dataset revision the site is built from
STRIPS = HERE / ".fixtures" / "strips"  # page offsets of each rendered strip

MAX_PAGES = 12  # pages drawn per document and engine (the viewer's "more pages" strip)
WIDTH = 640  # px; a letter page is 640 x 828
QUALITY = 72
# Page one of every strip is also written on its own: the home page and the Cases
# viewer open on page one and fetch this single page (about 5 KB), not the whole
# strip (147 KB on average, up to 240 KB).
FIRST = "-p1"

JUBARTE = "0.10.1"  # the jubarte release whose DOCX-to-PDF run the convert cases show
REDLINE_TOOL_JUBARTE = "jubarte-0.10.1"  # the redlines_0929_full lane the redline cases show
CONVERT_ENGINES = {
    "word": "Word export · oracle",
    "jubarte": f"jubarte {JUBARTE}",
    "soffice": "LibreOffice 26.8.0.3",
    "docxide": "docxide-pdf 0.17.1",
}
REDLINE_ENGINES = {
    "word": "Word compare · oracle",
    "jubarte": "jubarte 0.10.1 (release)",
    "docxodus": "Docxodus 12.6.5",
    "superdoc": "superdoc-sdk 2.16.0",
}
NOTICES = ("NOTICE", "LICENSE-ODC-BY-1.0.txt")  # docx-corpus attribution, shipped with the files
SCORE_BINS = [(0, 50), (50, 70), (70, 90), (90, 100.01)]


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fh:
        for block in iter(lambda: fh.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def git_blob_id(path: Path) -> str:
    data = path.read_bytes()
    return hashlib.sha1(b"blob %d\0" % len(data) + data, usedforsecurity=False).hexdigest()


def unique_cid(cid: str, stem: str, seen: set[str]) -> str:
    """`cid`, or `cid` plus a hash of `stem` when `cid` is taken.

    Case ids name the staging folders, so two cases sharing one would overwrite
    each other's files. Stops the run when the suffixed id is taken as well.
    """
    if cid in seen:
        cid = f"{cid}-{hashlib.sha1(stem.encode(), usedforsecurity=False).hexdigest()[:6]}"
        if cid in seen:
            raise SystemExit(f"case id {cid} is used twice; widen the id")
    seen.add(cid)
    return cid


def link(src: Path, dst: Path) -> None:
    dst.parent.mkdir(parents=True, exist_ok=True)
    if dst.exists():
        dst.unlink()
    try:
        os.link(src.resolve(), dst)
    except OSError:
        shutil.copy2(src.resolve(), dst)


def checkpoint(path: Path) -> dict[str, dict]:
    out = {}
    if not path.exists():
        return out
    for line in path.read_text().splitlines():
        try:
            row = json.loads(line)
        except json.JSONDecodeError:
            continue
        out[row["key"]] = row["result"]
    return out


def r2(v):
    return None if v is None else round(float(v), 2)


# ── stage ─────────────────────────────────────────────────────────────────────


def stage_convert(bench: Path, cases: list, jubarte: str = JUBARTE) -> None:
    res, corpus = bench / "results", bench / "corpus" / "word"
    work = res / f"jubarte_{jubarte}_docx_to_pdf_work"
    if not work.is_dir():
        raise SystemExit(f"{work} is missing: run the bench's release_jubarte.py {jubarte} first")
    pdfs = {
        "word": work / "oracle",
        "jubarte": work / "jubarte" / "candidate",
        "soffice": res / "soffice_26.8.0.3_work" / "candidate",
        "docxide": res / "docxide_0.17.1_work" / "candidate",
    }
    pixel = {
        "jubarte": checkpoint(work / "jubarte" / "scores.checkpoint.jsonl"),
        "soffice": checkpoint(res / "docx_to_pdf_soffice_26.8.0.3.checkpoint.jsonl"),
        "docxide": checkpoint(res / "docx_to_pdf_docxide_0.17.1.checkpoint.jsonl"),
    }
    metric_files = {
        "jubarte": f"docxide_metrics_jubarte_{jubarte}.json",
        "soffice": "docxide_metrics_soffice_26.8.0.3_corpus-all.json",
        "docxide": "docxide_metrics_docxide_0.17.1_corpus-all.json",
    }
    metrics = {
        k: next(iter(json.loads((res / f).read_text())["tools"].values()))["per_doc"] for k, f in metric_files.items()
    }
    seen: set[str] = set()
    # A score or a Word reference with no PDF behind it, or a case without its
    # source document, means the bench lost files: staging on would publish
    # blank panels or cases nobody can reproduce, so stop and name them.
    lost: dict[str, list[Path]] = collections.defaultdict(list)
    for row in csv.DictReader((res / "site_fixtures_860.csv").open()):
        stem = row["stem"]
        state, rest = stem.split("__", 1)
        cid = unique_cid(f"{state.replace('_', '-')}-{rest[:10]}", stem, seen)
        files = {"source.docx": corpus / state / "docx" / f"{rest}.docx"}
        files |= {f"{k}.pdf": d / f"{stem}.pdf" for k, d in pdfs.items()}
        engines = {}
        for key in CONVERT_ENGINES:
            if key == "word":
                continue
            px, mt = pixel[key].get(stem), metrics[key].get(stem)
            engines[key] = {
                "score": r2(px["overall_score"]) if px else 0.0,
                "failed": px is None,
                "page_scores": [r2(p["score"]) for p in px["pages"]][:MAX_PAGES] if px else [],
                "jaccard": r2(mt["jaccard"]) if mt else None,
                "text_boundary": r2(mt["text_boundary"]) if mt else None,
            }
        for name, src in list(files.items()):
            if src.exists():
                link(src, STAGE / "convert" / cid / name)
                continue
            files.pop(name)
            key = name.removesuffix(".pdf")
            if key in ("word", "source.docx"):
                lost[key.removesuffix(".docx")].append(src)
            elif not engines.get(key, {"failed": True})["failed"]:
                lost[f"scored {key}"].append(src)
        cases.append(
            {
                "id": cid,
                "bench": "convert",
                "stem": stem,
                "state": state,
                "files": sorted(files),
                "engines": engines,
            }
        )
    if lost:
        raise SystemExit(
            "; ".join(f"{len(v)} {k} file(s) missing, e.g. {v[0]}" for k, v in sorted(lost.items()))
            + ": restore them in the bench before staging"
        )


def pick_redlines(bench: Path, n: int, seed: int, available: set[str], tool: str = REDLINE_TOOL_JUBARTE) -> list[dict]:
    run = bench / "results" / "redlines_0929_full"
    rows = json.loads((run / f"scores_{tool}.json").read_text())["rows"]
    pool = {r["key"]: r for r in csv.DictReader((run / "pool_pairs.csv").open())}
    keys = [k for k, v in rows.items() if v.get("oracle") == "corpus" and k in pool and k in available]
    cells = collections.defaultdict(list)
    for k in keys:
        score = rows[k]["overall_score"] or 0
        band = next(i for i, (lo, hi) in enumerate(SCORE_BINS) if lo <= score <= hi)
        cells[pool[k]["state"], band].append(k)
    quota = {c: min(len(v), 2) for c, v in cells.items()}
    left = n - sum(quota.values())
    weight = {c: math.sqrt(len(v)) for c, v in cells.items()}
    total = sum(weight.values())
    for c in sorted(cells):
        quota[c] = min(len(cells[c]), quota[c] + int(left * weight[c] / total))
    rng = random.Random(seed)
    picked = []
    for c in sorted(cells):
        picked += rng.sample(sorted(cells[c]), quota[c])
    return [pool[k] | {"_scores": rows[k]} for k in sorted(picked)[:n]]


def stage_redline(bench: Path, cases: list, n: int, seed: int, tool: str = REDLINE_TOOL_JUBARTE) -> None:
    from huggingface_hub import HfApi, hf_hub_download

    api = HfApi()
    tree = set(api.list_repo_files(REPO, repo_type="dataset"))
    base = "outputs/redlines_0929_full"

    def hub(tool: str, kind: str, key: str) -> str | None:
        ext = "docx" if kind == "docx" else "pdf"
        path = f"{base}/{tool}/{kind}/{key}_{tool}.{ext}"
        return path if path in tree else None

    available = {
        p.rsplit("/", 1)[1].removesuffix(f"_{tool}.pdf") for p in tree if p.startswith(f"{base}/{tool}/pdf_by_word/")
    }
    if not available:
        raise SystemExit(f"{REPO} has no {base}/{tool}/pdf_by_word: upload the lane first (hub_upload.py)")
    redline_tool = {"jubarte": tool, "docxodus": "docxodus", "superdoc": "superdoc"}
    run = bench / "results" / "redlines_0929_full"
    other = {t: json.loads((run / f"scores_{t}.json").read_text())["rows"] for t in ("docxodus", "superdoc")}
    corpus = bench / "corpus" / "word"
    seen: set[str] = set()
    for row in pick_redlines(bench, n, seed, available, tool):
        key = row["key"]
        cid = unique_cid(f"r-{key.rsplit('_', 1)[1][:10]}", key, seen)
        dest = STAGE / "redline" / cid
        local = {
            "original.docx": corpus / f"{row['base']}.docx",
            "modified.docx": corpus / f"{row['next']}.docx",
            "word.docx": corpus / row["docx"],
            "word.pdf": corpus / row["pdf"],
        }
        files = []
        for name, src in local.items():
            if src.exists():
                link(src, dest / name)
                files.append(name)
        engines = {}
        for short, lane in redline_tool.items():
            for kind, name in (("docx", f"{short}.docx"), ("pdf_by_word", f"{short}.pdf")):
                path = hub(lane, kind, key)
                if path:
                    link(Path(hf_hub_download(REPO, path, repo_type="dataset")), dest / name)
                    files.append(name)
            s = row["_scores"] if short == "jubarte" else other[lane].get(key)
            scored = bool(s) and s.get("overall_score") is not None and f"{short}.pdf" in files
            engines[short] = {
                "score": r2(s["overall_score"]) if scored else 0.0,
                "failed": not scored,
                "page_scores": [],
                "jaccard": r2(100 * s["ink_jaccard"]) if scored and s.get("ink_jaccard") is not None else None,
                "text_boundary": r2(100 * s["text_boundary"])
                if scored and s.get("text_boundary") is not None
                else None,
                "pages_mismatch": bool(s.get("page_count_mismatch")) if scored else None,
            }
        cases.append(
            {
                "id": cid,
                "bench": "redline",
                "stem": key,
                "state": row["state"],
                "files": sorted(files),
                "engines": engines,
            }
        )


def cmd_stage(a) -> None:
    if STAGE.exists():
        shutil.rmtree(STAGE)
    STAGE.mkdir(parents=True)
    cases: list[dict] = []
    stage_convert(a.bench, cases, a.jubarte)
    stage_redline(a.bench, cases, a.redlines, a.seed, a.redline_tool)
    # The engine labels name the runs actually staged: versions.json is the redline run's own record.
    lanes = json.loads((a.bench / "results/redlines_0929_full/versions.json").read_text())
    engines = {
        "convert": CONVERT_ENGINES | {"jubarte": f"jubarte {a.jubarte}"},
        "redline": REDLINE_ENGINES | {"jubarte": lanes[a.redline_tool]},
    }
    shas = {}
    for c in cases:
        for f in c["files"]:
            shas[f"{c['bench']}/{c['id']}/{f}"] = sha256(STAGE / c["bench"] / c["id"] / f)
    manifest = {
        "about": "Documents shown on jubarte.pro/cases. Scores: neurotic_docx_bench pixel scorer v1 vs Microsoft Word "
        "(0-100, a failure counts 0); jaccard and text_boundary: docxide-pdf's metrics at 150 dpi (0-100).",
        "engines": engines,
        "cases": cases,
        "sha256": shas,
    }
    (STAGE / "manifest.json").write_text(json.dumps(manifest, indent=1))
    (STAGE / "README.md").write_text(README.replace("{jubarte}", a.jubarte))
    for name in NOTICES:
        shutil.copyfile(a.bench / "corpus/word/notices" / name, STAGE / name)
    by = collections.Counter(c["bench"] for c in cases)
    print(f"staged {dict(by)} cases, {len(shas)} files under {STAGE}")


README = """# site/ — the documents behind jubarte.pro/use-cases

Every page image on [jubarte.pro/cases](https://jubarte.pro/cases) is drawn from a file in this
folder, at the revision the site links to.

- `convert/<id>/`: `source.docx`, then Microsoft Word's PDF export of it (`word.pdf`, the oracle)
  and the PDFs of jubarte {jubarte}, LibreOffice 26.8.0.3 and docxide-pdf 0.17.1. Picked from the
  6,427-document `corpus/word:all` set by `results/site_fixtures_860.csv` (stratified by corpus
  state, page count and jubarte score).
- `redline/<id>/`: `original.docx`, `modified.docx`, Word's own compare of the pair (`word.docx`,
  `word.pdf`), and each tool's redline with the PDF Word made of it after opening it
  (`<tool>.docx`, `<tool>.pdf`). A missing PDF means the tool produced no redline or Word could
  not open it; the bench scores that pair 0.
- `manifest.json`: per case, the corpus stem, the scores and the sha256 of every file.

Scores come from [neurotic_docx_bench](https://github.com/jandira-tech/neurotic_docx_bench):
pixel scorer v1 against Word (0-100, intent-to-treat), plus docxide-pdf's Jaccard and
text-boundary metrics. jubarte is author-affiliated and held to the same rules.

## Source documents

Contains information from [docx-corpus](https://huggingface.co/datasets/superdoc-dev/docx-corpus)
(superdoc-dev/docx-corpus, built by SuperDoc), made available under the ODC Attribution License
(ODC-By v1.0; `LICENSE-ODC-BY-1.0.txt`, `NOTICE`). ODC-By covers the collection, not copyright in
the individual .docx files, which were collected from the public web and may carry their own
rights.
"""


# ── upload ────────────────────────────────────────────────────────────────────


def cmd_upload(a) -> None:
    from huggingface_hub import HfApi

    api = HfApi()
    info = api.upload_folder(
        repo_id=REPO,
        repo_type="dataset",
        folder_path=str(STAGE),
        path_in_repo=PREFIX,
        commit_message="site: documents behind jubarte.pro/use-cases",
        delete_patterns=["**"],
    )
    rev = info.oid
    LOCK.write_text(f"{REPO}@{rev}\n")
    print(f"uploaded {STAGE} -> {REPO}/{PREFIX} @ {rev}; commit fixtures.lock")


def locked_revision() -> str:
    repo, _, rev = LOCK.read_text().strip().partition("@")
    if repo != REPO or len(rev) != 40:
        raise SystemExit(f"{LOCK.name} must read {REPO}@<40-hex revision>")
    return rev


def cmd_fetch(a) -> None:
    from huggingface_hub import snapshot_download

    rev = locked_revision()
    if STAGE.exists():
        shutil.rmtree(STAGE)
    snapshot_download(REPO, repo_type="dataset", revision=rev, allow_patterns=[f"{PREFIX}/**"], local_dir=STAGE.parent)
    print(f"fetched {REPO}/{PREFIX} @ {rev} -> {STAGE}")


# ── render ────────────────────────────────────────────────────────────────────


def first_page_path(strip_path: Path) -> Path:
    return strip_path.with_name(f"{strip_path.stem}{FIRST}.webp")


def save_webp(image, out: Path) -> None:
    out.parent.mkdir(parents=True, exist_ok=True)
    buf = io.BytesIO()
    image.save(buf, "WEBP", quality=QUALITY, method=6)
    out.write_bytes(buf.getvalue())


def strip(pdf: Path, out: Path, first: bool = False) -> dict:
    """Stack up to MAX_PAGES pages into one WebP; return page offsets in strip pixels.

    With ``first``, page one is also written on its own (see FIRST)."""
    import pymupdf
    from PIL import Image

    doc = pymupdf.open(pdf)
    pages = []
    # doc.pages() loads only the pages drawn; list(doc) built every page of a long document.
    for page in doc.pages(0, min(MAX_PAGES, doc.page_count)):
        zoom = WIDTH / page.rect.width
        pix = page.get_pixmap(matrix=pymupdf.Matrix(zoom, zoom), alpha=False)
        pages.append(Image.frombytes("RGB", (pix.width, pix.height), pix.samples))
    total = doc.page_count
    doc.close()
    height = sum(p.height for p in pages)
    sheet = Image.new("RGB", (WIDTH, height), "white")
    y, offsets = 0, []
    for p in pages:
        sheet.paste(p, (0, y))
        offsets.append([y, p.height])
        y += p.height
    save_webp(sheet, out)
    if first and pages:
        save_webp(pages[0], first_page_path(out))
    return {"pages": total, "h": height, "offsets": offsets}


def render_one(job):
    pdf, out, first = job
    try:
        return str(out), strip(Path(pdf), Path(out), first)
    # Deliberately broad: mupdf raises plain Exception subclasses, PIL even MemoryError,
    # and one bad PDF must be recorded as a failed render, not stop a pool of thousands.
    except Exception as exc:  # noqa: BLE001
        return str(out), {"error": str(exc)}


def crop_first(job) -> None:
    """Page one cut from a strip rendered before FIRST existed (``render --keep``)."""
    from PIL import Image

    strip_path, height = job
    with Image.open(strip_path) as sheet:
        save_webp(sheet.convert("RGB").crop((0, 0, WIDTH, height)), first_page_path(Path(strip_path)))


def case_record(c: dict, strips: dict) -> dict:
    """A case as the viewer reads it: scores, the strips' page offsets, its file names."""
    base = f"{c['bench']}/{c['id']}"
    renders = {
        f[:-4]: strips.get(f"{base}/{f[:-4]}", {"error": "not rendered"}) for f in c["files"] if f.endswith(".pdf")
    }
    return {
        "id": c["id"],
        "stem": c["stem"],
        "state": c["state"],
        "engines": c["engines"],
        "renders": renders,
        "files": sorted(c["files"]),
    }


def cmd_render(a) -> None:
    from huggingface_hub import HfApi, RepoFile

    rev = locked_revision()
    manifest = json.loads((STAGE / "manifest.json").read_text())
    api = HfApi()
    # Large files are LFS objects and report their sha256; small ones are plain git blobs
    # and are checked by their blob id instead. Either way every byte is compared.
    remote = {}
    for entry in api.list_repo_tree(REPO, repo_type="dataset", path_in_repo=PREFIX, recursive=True, revision=rev):
        if not isinstance(entry, RepoFile):
            continue  # a folder
        lfs = entry.lfs
        remote[entry.path.removeprefix(f"{PREFIX}/")] = ("sha256", lfs.sha256) if lfs else ("blob", entry.blob_id)

    def matches(path: str, digest: str) -> bool:
        kind, value = remote[path]
        return value == (digest if kind == "sha256" else git_blob_id(STAGE / path))

    missing = [p for p in manifest["sha256"] if p not in remote]
    bad = [p for p, d in manifest["sha256"].items() if p in remote and not matches(p, d)]
    if bad or missing:
        raise SystemExit(f"staged files differ from {REPO}@{rev}: {len(bad)} changed, {len(missing)} missing")
    print(f"{len(manifest['sha256'])} files match {REPO}@{rev}")

    out_root = PUBLIC / "fixtures"
    if not a.keep:
        for d in (out_root, STRIPS):
            if d.exists():
                shutil.rmtree(d)

    def meta_of(target: Path) -> Path:
        return STRIPS / target.relative_to(out_root).with_suffix(".json")

    jobs = []
    for c in manifest["cases"]:
        for f in c["files"]:
            if f.endswith(".pdf"):
                target = out_root / c["bench"] / c["id"] / f.replace(".pdf", ".webp")
                if not (a.keep and target.exists() and meta_of(target).exists()):
                    jobs.append((str(STAGE / c["bench"] / c["id"] / f), str(target), True))
    strips = {}
    with cf.ProcessPoolExecutor() as pool:
        for i, (out, meta) in enumerate(pool.map(render_one, jobs, chunksize=4), 1):
            meta_of(Path(out)).parent.mkdir(parents=True, exist_ok=True)
            meta_of(Path(out)).write_text(json.dumps(meta))
            if i % 200 == 0:
                print(f"  {i}/{len(jobs)}")
    for meta_file in STRIPS.rglob("*.json"):
        strips[str(meta_file.relative_to(STRIPS).with_suffix(""))] = json.loads(meta_file.read_text())
    crops = [
        (str(out_root / f"{key}.webp"), meta["offsets"][0][1])
        for key, meta in strips.items()
        if meta.get("offsets") and not first_page_path(out_root / f"{key}.webp").exists()
    ]
    with cf.ProcessPoolExecutor() as pool:
        list(pool.map(crop_first, crops, chunksize=16))

    hub = f"https://huggingface.co/datasets/{REPO}"
    data = {
        "repo": REPO,
        "revision": rev,
        # A case's folder is {hub}/{bench}/{id}, a file in it {resolve}/{bench}/{id}/{name}:
        # the viewer builds both, so each case lists only its file names.
        "hub": f"{hub}/tree/{rev}/{PREFIX}",
        "resolve": f"{hub}/resolve/{rev}/{PREFIX}",
        "engines": manifest["engines"],
        "cases": {"convert": [], "redline": []},
    }
    for c in manifest["cases"]:
        data["cases"][c["bench"]].append(case_record(c, strips))
    (PUBLIC / "data").mkdir(parents=True, exist_ok=True)
    for bench, cases in data["cases"].items():
        body = {k: v for k, v in data.items() if k != "cases"} | {"bench": bench, "cases": cases}
        (PUBLIC / "data" / f"cases-{bench}.json").write_text(json.dumps(body, separators=(",", ":")))
    print(f"rendered {len(jobs)} strips, cut {len(crops)} first pages; data -> {PUBLIC / 'data'}")


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    s = sub.add_parser("stage")
    s.add_argument("--bench", type=Path, default=Path.home() / "temp/T/neurotic_docx_bench")
    s.add_argument("--redlines", type=int, default=240)
    s.add_argument("--seed", type=int, default=20261001)
    s.add_argument("--jubarte", default=JUBARTE, help="jubarte release of the DOCX-to-PDF cases")
    s.add_argument("--redline-tool", default=REDLINE_TOOL_JUBARTE, help="redlines_0929_full lane of the redline cases")
    sub.add_parser("upload")
    sub.add_parser("fetch")
    r = sub.add_parser("render")
    r.add_argument("--keep", action="store_true", help="reuse strips already rendered")
    a = ap.parse_args()
    {"stage": cmd_stage, "upload": cmd_upload, "fetch": cmd_fetch, "render": cmd_render}[a.cmd](a)


if __name__ == "__main__":
    main()
