#!/usr/bin/env python3
"""Build the finding map: all-ids.json + per-folder dedup-NNNN.json.

One pass over .scan/findings.json. Never prints finding bodies.
Membership rule (union):
  - a provenance.sourceFindingIds entry starts with a claimedWorkerId declared
    in the folder's prompt.md reducer block, OR
  - an attackPath.corroboratingPaths[].evidenceRefs entry starts with
    "dedup" + the folder number without the dash (e.g. dedup-0010 -> "dedup0010").
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path("/Users/arthrod/temp/T/jubarte-redlines/.scan")
FINDINGS = ROOT / "findings.json"
DEDUP = ROOT / "artifacts" / "deep_discovery" / "dedup"
OUTDIR = ROOT / "remediation" / "findings"


def flatten_severity(sev):
    """Severity may be an object ({level, rationale, ...}) or a string."""
    if isinstance(sev, dict):
        for key in ("level", "severity", "rating", "value"):
            if key in sev and sev[key]:
                return str(sev[key]).strip().lower()
        return "unknown"
    if isinstance(sev, str):
        return sev.strip().lower()
    return "unknown"


def parse_claimed_workers(prompt_path):
    text = prompt_path.read_text(encoding="utf-8", errors="replace")
    m = re.search(r'"claimedWorkerIds"\s*:\s*\[([^\]]*)\]', text)
    if not m:
        return None
    return re.findall(r'"([0-9a-fA-F][0-9a-fA-F-]*)"', m.group(1))


def main():
    folders = sorted(p.name for p in DEDUP.iterdir()
                     if p.is_dir() and re.fullmatch(r"dedup-\d{4}", p.name))
    if len(folders) != 33:
        print(f"WARN: expected 33 dedup folders, found {len(folders)}")

    # folder -> (set of claimed worker uuids, evidence prefix)
    folder_workers = {}
    folder_prefix = {}
    for f in folders:
        workers = parse_claimed_workers(DEDUP / f / "prompt.md")
        if workers is None:
            print(f"ERROR: no claimedWorkerIds block in {f}/prompt.md")
            sys.exit(1)
        folder_workers[f] = set(workers)
        folder_prefix[f] = f.replace("-", "", 1)  # dedup-0010 -> dedup0010

    data = json.loads(FINDINGS.read_text(encoding="utf-8"))
    findings = data["findings"]
    print(f"findings total: {len(findings)}")

    all_ids = []
    folder_findings = {f: [] for f in folders}
    unmapped = []
    weird_refs = set()

    for fd in findings:
        fid = fd["findingId"]
        sev = flatten_severity(fd.get("severity"))
        prov_ids = fd.get("provenance", {}).get("sourceFindingIds") or []
        corrob = fd.get("attackPath", {}).get("corroboratingPaths") or []
        refs = []
        for cp in corrob:
            refs.extend(cp.get("evidenceRefs") or [])
            for extra in (cp.get("evidence") or []):
                if isinstance(extra, str):
                    refs.append(extra)
        for r in refs:
            if "dedup" in r and not r.startswith("dedup"):
                weird_refs.add(r)

        matched = set()
        for f in folders:
            by_worker = any(
                any(sid.startswith(w) for w in folder_workers[f])
                for sid in prov_ids
            )
            by_ref = any(r.startswith(folder_prefix[f]) for r in refs)
            if by_worker or by_ref:
                matched.add(f)

        all_ids.append({
            "findingId": fid,
            "title": fd.get("title"),
            "severity": sev,
            "folders": sorted(matched),
        })
        if not matched:
            unmapped.append(fid)
        for f in matched:
            folder_findings[f].append({
                "findingId": fid,
                "title": fd.get("title"),
                "severity": sev,
                "summary": fd.get("summary"),
                "remediation": fd.get("remediation"),
                "locations": fd.get("locations"),
            })

    OUTDIR.mkdir(parents=True, exist_ok=True)
    (OUTDIR / "all-ids.json").write_text(
        json.dumps(all_ids, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    for f in folders:
        (OUTDIR / f"{f}.json").write_text(
            json.dumps({"folder": f, "findings": folder_findings[f]},
                       indent=2, ensure_ascii=False) + "\n",
            encoding="utf-8",
        )

    total_mapped = sum(len(v) for v in folder_findings.values())
    print("per-folder counts:")
    for f in folders:
        print(f"  {f}: {len(folder_findings[f])}")
    print(f"mapped finding-folder entries: {total_mapped}")
    print(f"unmapped findings (all-ids.json only): {len(unmapped)}")
    for fid in unmapped:
        print(f"  UNMAPPED: {fid}")
    if weird_refs:
        print(f"WARN: {len(weird_refs)} evidenceRefs contain 'dedup' not at start: {sorted(weird_refs)[:5]}")
    print(f"folder files written: {len(folders)}")


if __name__ == "__main__":
    main()
