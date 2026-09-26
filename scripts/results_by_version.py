#!/usr/bin/env python3
# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Write RESULTS.md: the best result of every tool version, per metric.

Reads every results store we keep and lists, for each metric:

- what it measures (redline markup, redline speed, docx->pdf, pdf->docx);
- the reference PDFs (Microsoft Word exports, or soffice-converted);
- for docx->pdf, whether the documents are clean or redlined (comparison) docs;
- per tool and version, the best run of each 7-day window (newest window
  first), ranked by its mean (lower is better for speed). A run without a
  mean is ranked by the average of its per-document scores.

Sources (in neurotic_docx_bench, $NEUROTIC_DOCX_BENCH): results/bench.jsonl, results/speed.jsonl, results/redline_speed_bench,
results/docx_to_pdf*.json, results/docxide_metrics*.json,
results/soffice_vs_word_redlines_randomized, grok_run/docxide_metrics, and the
jubarte loop's English-corpus, redlined-corpus and docxide-suite runs.

Usage: uv run python scripts/results_by_version.py [--out RESULTS.md]
"""

from __future__ import annotations

import argparse
import glob
import json
import os
import statistics
from dataclasses import dataclass
from datetime import datetime, timedelta, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
ROOT = Path(os.environ.get('NEUROTIC_DOCX_BENCH', '/Users/arthrod/temp/T/neurotic_docx_bench'))
RES = ROOT / 'results'
GROK = ROOT / 'grok_run'
LOOP = Path(os.environ.get('JUBARTE_LOOP', '/Users/arthrod/temp/T/jubarte-loop'))
DXSUITE = Path(os.environ.get('DOCXIDE_SUITE', '/Users/arthrod/temp/T/docxide_suite'))

# The English-corpus full runs record a tag, not the binary: en_full.sh BIN TAG.
EN_TAGS = {
    'full0926b': 'jubarte@1da7e08',
    'full0926c': 'jubarte@020d909',
    'full0926d': 'jubarte@ef2a739',
    'full0926e': 'jubarte@23abeb2',
    'saved23abeb2': 'jubarte@23abeb2',
    'h80d6f94': 'jubarte@80d6f94',
    'd2aa5db': 'jubarte@d2aa5db',
    '6742f5c': 'jubarte@6742f5c',
    '7c8d376_now': 'jubarte@7c8d376',
    'macfix': 'jubarte@macfix',
    # Clean rebuild of 23abeb2 (older binaries deleted first), 2026-09-26.
    'fresh0926f': 'jubarte@23abeb2 (fresh build)',
    'fresh0926g': 'jubarte@23abeb2 (fresh build)',
}
# Competitor versions of the English corpus (installed latest, 2026-09-25).
EN_COMPETITORS = {
    'docxide': 'docxide-pdf 0.17.1',
    'soffice': 'soffice 26.8.0',
    'rdocx': 'rdocx 0.14.0',
    'office2pdf': 'office2pdf 0.7.0',
    'minipdf': 'minipdf 0.7.0',
}
# Competitors on the redlined compared_a_100_vs_b_10 set (2026-09-24).
REDL_COMPETITORS = {'docxide': 'docxide-pdf 0.17.1', 'soffice': 'soffice 26.8.0'}


@dataclass
class Run:
    metric: str  # section key
    tool: str
    version: str
    when: datetime
    mean: float | None
    median: float | None
    n: int
    scores: list[float] | None = None

    @property
    def rank_value(self) -> float | None:
        if self.mean is not None:
            return self.mean
        if self.scores:
            return statistics.mean(self.scores)
        return None


@dataclass
class Metric:
    key: str
    title: str
    kind: str  # "redline markup", "redline speed", "docx->pdf", "pdf->docx"
    reference: str
    docs: str  # "clean", "redlines", or "-"
    unit: str
    lower_is_better: bool = False


METRICS: dict[str, Metric] = {}
RUNS: list[Run] = []


def metric(key: str, **kw) -> str:
    METRICS.setdefault(key, Metric(key=key, **kw))
    return key


def when_of(value: str | None, path: Path | None = None) -> datetime:
    if value:
        try:
            dt = datetime.fromisoformat(value.replace('Z', '+00:00'))
            return dt if dt.tzinfo else dt.replace(tzinfo=timezone.utc)
        except ValueError:
            pass
    if path is not None and path.exists():
        return datetime.fromtimestamp(path.stat().st_mtime, tz=timezone.utc)
    return datetime(1970, 1, 1, tzinfo=timezone.utc)


def add(**kw) -> None:
    RUNS.append(Run(**kw))


# --- redline markup benches (results/bench.jsonl) ------------------------------------------


def bench_jsonl() -> None:
    path = RES / 'bench.jsonl'
    if not path.exists():
        return
    with path.open() as fh:
        for line in fh:
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                continue
            bench = row.get('benchmark')
            if not bench:
                continue
            env = row.get('environment_config') or {}
            truth = str(env.get('source_of_truth') or '')
            runs = env.get('runs') or []
            render = runs[0].get('render') if runs else None
            reference = 'Word' if 'word' in truth.lower() else (truth or 'unknown')
            if render:
                reference += f' (tool output rendered by {render})'
            n_docs = row.get('itt_n_docs') or row.get('n_docs') or 0
            if n_docs < 20:
                continue  # smoke runs
            corpus = row.get('corpus_revision') or 'unstamped'
            key = metric(
                f'redline:{bench}:{reference}:{corpus}',
                title=f'Redline markup — {bench}' + (f' ({render} render)' if render else '') + f', corpus {corpus}',
                kind='redline markup',
                reference=reference,
                docs='redlines',
                unit='score 0-100',
            )
            version = row.get('tool_version') or (runs[0].get('package') if runs else None) or 'unversioned'
            mean = row.get('itt_mean') if row.get('itt_mean') is not None else row.get('overall_mean')
            median = row.get('itt_median') if row.get('itt_median') is not None else row.get('overall_median')
            add(
                metric=key,
                tool=row.get('vendor') or '?',
                version=str(version),
                when=when_of(row.get('timestamp')),
                mean=mean,
                median=median,
                n=n_docs,
            )


# --- redline speed ---------------------------------------------------------------------------


def speed_rows(path: Path) -> None:
    if not path.exists():
        return
    for line in path.read_text().splitlines():
        try:
            row = json.loads(line)
        except json.JSONDecodeError:
            continue
        if row.get('unit') != 'ms_per_redline':
            continue
        key = metric(
            'speed:redlines',
            title='Redline speed — ms per redline',
            kind='redline speed',
            reference='-',
            docs='redlines',
            unit='ms per redline (lower is better)',
            lower_is_better=True,
        )
        dist = row.get('dist') or ''
        version = Path(dist).name if dist else (row.get('engine') or 'unversioned')
        add(
            metric=key,
            tool=row.get('tool') or row.get('engine') or '?',
            version=f'{version} [{row.get("runtime") or "?"}]',
            when=when_of(row.get('run_ts'), path),
            mean=row.get('mean'),
            median=row.get('median'),
            n=row.get('n') or 0,
        )


def speed() -> None:
    speed_rows(RES / 'speed.jsonl')
    for path in sorted((RES / 'redline_speed_bench').rglob('speed.jsonl')):
        speed_rows(path)


# --- docx -> pdf, neurotic harness -----------------------------------------------------------


def harness_docx_to_pdf() -> None:
    for path in sorted(RES.glob('docx_to_pdf*.json')):
        try:
            doc = json.loads(path.read_text())
        except json.JSONDecodeError, OSError:
            continue
        track = doc.get('track') or ''
        clean = 'no_redline' in track
        oracle = doc.get('oracle') or ''
        reference = 'Word' if 'word' in oracle.lower() else oracle or 'unknown'
        n_docs = doc.get('n') or 0
        key = metric(
            f'harness:{track}:{n_docs}',
            title=f'docx→pdf — neurotic harness `{track}` ({n_docs} docs)',
            kind='docx->pdf',
            reference=reference,
            docs='clean' if clean else 'redlines',
            unit='harness score 0-100',
        )
        for tool, t in (doc.get('tools') or {}).items():
            per_doc = t.get('per_doc') or {}
            scores = [v for v in per_doc.values() if isinstance(v, (int, float))]
            add(
                metric=key,
                tool=tool,
                version=str(t.get('version') or t.get('converter') or 'unversioned'),
                when=when_of(doc.get('generated_at'), path),
                mean=t.get('mean'),
                median=t.get('median'),
                n=t.get('n_scored') or n_docs,
                scores=scores or None,
            )


def docxide_metric_reports() -> None:
    reports = [(p, None) for p in sorted(RES.glob('docxide_metrics*.json'))]
    reports.append((GROK / 'docxide_metrics' / 'report.json', 'fixtures_500'))
    for path, label in reports:
        if not path.exists():
            continue
        doc = json.loads(path.read_text())
        track = label or doc.get('fixture_track') or doc.get('track') or 'docxide_metrics'
        n_docs = doc.get('n') or 0
        oracle = doc.get('oracle') or ''
        key = metric(
            f'dxm:{track}:{n_docs}',
            title=f'docx→pdf — docxide-metrics Jaccard on `{track}` ({n_docs} docs)',
            kind='docx->pdf',
            reference='Word' if 'word' in oracle.lower() else oracle or 'unknown',
            docs='clean' if 'no_redline' in track or track == 'fixtures_500' else 'redlines',
            unit='Jaccard 0-100',
        )
        for tool, t in (doc.get('tools') or {}).items():
            jac = (t.get('metrics') or {}).get('jaccard') or {}
            add(
                metric=key,
                tool=tool,
                version=str(t.get('version') or 'unversioned'),
                when=when_of(doc.get('generated_at'), path),
                mean=jac.get('mean'),
                median=jac.get('median'),
                n=t.get('n_scored') or n_docs,
            )


def soffice_vs_word() -> None:
    path = RES / 'soffice_vs_word_redlines_randomized' / 'summary.json'
    if not path.exists():
        return
    doc = json.loads(path.read_text())
    key = metric(
        'soffice_vs_word',
        title='docx→pdf — soffice render of Word redlines vs Word export',
        kind='docx->pdf',
        reference='Word',
        docs='redlines',
        unit='visual score 0-100',
    )
    add(
        metric=key,
        tool='soffice',
        version=doc.get('renderer_candidate') or 'soffice',
        when=when_of(None, path),
        mean=doc.get('mean'),
        median=doc.get('median'),
        n=doc.get('n_scored') or 0,
    )


# --- docx -> pdf, jubarte loop corpora -------------------------------------------------------


def stats(values: list[float]) -> tuple[float, float]:
    return statistics.mean(values), statistics.median(values)


def english_corpus() -> None:
    parts = {'a': 'part a (500 originals)', 'b': 'part b (500 originals)', 'r': 'redlines a vs b'}
    seen_competitors: set[tuple[str, str]] = set()
    for p, label in parts.items():
        key = metric(
            f'en:{p}',
            title=f'docx→pdf — English corpus {label}',
            kind='docx->pdf',
            reference='Word',
            docs='redlines' if p == 'r' else 'clean',
            unit='docxide-metrics Jaccard 0-1',
        )
        paths = glob.glob(str(LOOP / 'runs' / f'enfull_{p}_*' / 'rows.json'))
        paths += glob.glob(str(LOOP / 'runs' / f'en{p}_full_*' / 'rows.json'))
        if p == 'r':
            paths += glob.glob(str(LOOP / 'runs' / 'en_r_base_*' / 'rows.json'))
        for raw in sorted(paths):
            path = Path(raw)
            rows = json.loads(path.read_text())
            if len(rows) < 400:
                continue
            name = path.parent.name
            tag = name.split(f'enfull_{p}_')[-1].split(f'en{p}_full_')[-1].split('en_r_base_')[-1]
            version = EN_TAGS.get(tag, f'jubarte@{tag}')
            when = when_of(None, path)
            vals = [r['jubarte'] for r in rows.values()]
            mean, median = stats(vals)
            add(metric=key, tool='jubarte', version=version, when=when, mean=mean, median=median, n=len(vals))
            for tool, tver in EN_COMPETITORS.items():
                if (p, tool) in seen_competitors:
                    continue
                cvals = [r[tool] for r in rows.values() if tool in r]
                if not cvals:
                    continue
                seen_competitors.add((p, tool))
                comp_path = LOOP / 'runs' / f'en_{p}_competitors.json'
                mean, median = stats(cvals)
                add(
                    metric=key,
                    tool=tool,
                    version=tver,
                    when=when_of(None, comp_path if comp_path.exists() else path),
                    mean=mean,
                    median=median,
                    n=len(cvals),
                )


def redlined_compared_set() -> None:
    redl = LOOP / 'redl'
    base = redl / 'all_scores.json'
    if not base.exists():
        return
    key = metric(
        'redl:compared_a_100_vs_b_10',
        title='docx→pdf — Word-redlined `compared_a_100_vs_b_10` (965 pairs)',
        kind='docx->pdf',
        reference='Word',
        docs='redlines',
        unit='docxide-metrics Jaccard 0-1',
    )
    doc = json.loads(base.read_text())
    for tool, ver in REDL_COMPETITORS.items():
        vals = list((doc.get(tool) or {}).values())
        if vals:
            mean, median = stats(vals)
            add(metric=key, tool=tool, version=ver, when=when_of(None, base), mean=mean, median=median, n=len(vals))
    for path in sorted(redl.glob('*_rows.json')):
        rows = json.loads(path.read_text())
        if len(rows) < 900:
            continue
        tag = path.name.removesuffix('_rows.json').removeprefix('rl_')
        vals = [v for v in rows.values() if isinstance(v, (int, float))]
        mean, median = stats(vals)
        add(
            metric=key,
            tool='jubarte',
            version=EN_TAGS.get(tag, f'jubarte@{tag}'),
            when=when_of(None, path),
            mean=mean,
            median=median,
            n=len(vals),
        )


def docxide_suite() -> None:
    key = metric(
        'dxsuite',
        title="docx→pdf — docxide-pdf's own 208-case suite",
        kind='docx->pdf',
        reference="Word (Word for Mac online export, docxide's references)",
        docs='clean',
        unit='Jaccard 0-1',
    )
    published = DXSUITE / 'published_scores.json'
    if published.exists():
        cases = json.loads(published.read_text())
        by_engine: dict[str, list[float]] = {}
        for case in cases:
            for engine, s in (case.get('scores') or {}).items():
                if isinstance(s, dict) and s.get('jaccard') is not None:
                    by_engine.setdefault(engine, []).append(s['jaccard'] / 100.0)
        for engine, vals in by_engine.items():
            mean, median = stats(vals)
            name = 'docxide' if engine == 'generated' else engine
            add(
                metric=key,
                tool=name,
                version='published (sverrejb.github.io/docxide-pdf)',
                when=when_of(None, published),
                mean=mean,
                median=median,
                n=len(vals),
            )
    for path in sorted((LOOP / 'dxruns').glob('*/result.json')):
        rows = json.loads(path.read_text())
        if len(rows) < 200:
            continue
        vals = [r['new'] for r in rows.values() if isinstance(r, dict) and r.get('new') is not None]
        mean, median = stats(vals)
        tag = path.parent.name.removeprefix('dx_')
        add(
            metric=key,
            tool='jubarte',
            version=EN_TAGS.get(tag, f'jubarte@{tag}'),
            when=when_of(None, path),
            mean=mean,
            median=median,
            n=len(vals),
        )


def fixtures_500_loop_runs() -> None:
    """jubarte-loop score.py full runs on fixtures_500 (0-1 Jaccard, shown 0-100 like report.json)."""
    for path in sorted((LOOP / 'runs').glob('full_*/result.json')):
        rows = json.loads(path.read_text())
        vals = [r['new'] * 100 for r in rows.values() if isinstance(r, dict) and r.get('new') is not None]
        if len(vals) < 400 or not any(vals):
            continue
        mean, median = stats(vals)
        tag = path.parent.name.removeprefix('full_')
        add(
            metric=f'dxm:fixtures_500:{len(vals)}',
            tool='jubarte',
            version=EN_TAGS.get(tag, f'jubarte@{tag}'),
            when=when_of(None, path),
            mean=mean,
            median=median,
            n=len(vals),
        )


def pdf_to_docx() -> None:
    metric('pdf_to_docx', title='pdf→docx', kind='pdf->docx', reference='-', docs='-', unit='-')


# --- report ----------------------------------------------------------------------------------


def best_per_window(runs: list[Run], lower: bool) -> list[Run]:
    """Best run per (tool, version) and 7-day window, windows counted back from the newest."""
    out: list[Run] = []
    groups: dict[tuple[str, str], list[Run]] = {}
    for r in runs:
        if r.rank_value is None:
            continue
        groups.setdefault((r.tool, r.version), []).append(r)
    for group in groups.values():
        group.sort(key=lambda r: r.when, reverse=True)
        while group:
            start = group[0].when - timedelta(days=7)
            window = [r for r in group if r.when > start]
            group = [r for r in group if r.when <= start]
            pick = min if lower else max
            out.append(pick(window, key=lambda r: r.rank_value))
    return out


def fmt(v: float | None) -> str:
    if v is None:
        return '—'
    return f'{v:.4f}' if abs(v) < 2 else f'{v:.2f}'


KIND_ORDER = ['redline markup', 'redline speed', 'docx->pdf', 'pdf->docx']


def render() -> str:
    lines = [
        # REUSE-IgnoreStart
        '<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->',
        '<!-- SPDX-License-Identifier: AGPL-3.0-only -->',
        # REUSE-IgnoreEnd
        '<!-- Generated by scripts/results_by_version.py — do not edit by hand. -->',
        '# Results by tool version',
        '',
        f'Generated {datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")}. Each table keeps, for every',
        'tool version, the best run of each 7-day window (newest first) and ranks rows by that',
        "run's mean (speed: lowest first). Rows without a mean rank by the average of their",
        'per-document scores. Scores are only comparable inside one table.',
        '',
        '| Metric | Kind | Reference PDFs | Documents | Unit |',
        '| --- | --- | --- | --- | --- |',
    ]
    ordered = sorted(METRICS.values(), key=lambda m: (KIND_ORDER.index(m.kind), m.docs, m.title))
    lines.extend(
        f'| [{m.title}](#{anchor(m.title)}) | {m.kind} | {m.reference} | {m.docs} | {m.unit} |' for m in ordered
    )
    for kind in KIND_ORDER:
        mets = [m for m in ordered if m.kind == kind]
        if not mets:
            continue
        lines += ['', f'## {kind}']
        if kind == 'docx->pdf':
            lines.append('')
            lines.append('Split by documents: **clean** = source documents without tracked changes;')
            lines.append('**redlines** = comparison (redlined) documents.')
        for m in mets:
            lines += ['', f'### {m.title}', '']
            lines.append(f'Reference PDFs: **{m.reference}**. Documents: **{m.docs}**. Unit: {m.unit}.')
            runs = best_per_window([r for r in RUNS if r.metric == m.key], m.lower_is_better)
            if not runs:
                lines += ['', '_No run measured yet._']
                continue
            runs.sort(key=lambda r: r.rank_value, reverse=not m.lower_is_better)
            lines += [
                '',
                '| Rank | Tool | Version | Date | Docs | Mean | Median |',
                '| --- | --- | --- | --- | --- | --- | --- |',
            ]
            for i, r in enumerate(runs, 1):
                mean = fmt(r.mean) if r.mean is not None else f'{fmt(r.rank_value)} (avg)'
                lines.append(
                    f'| {i} | {r.tool} | {r.version} | {r.when.strftime("%Y-%m-%d")} | {r.n} | {mean} | {fmt(r.median)} |'
                )
    return '\n'.join(lines) + '\n'


def anchor(title: str) -> str:
    keep = ''.join(c for c in title.lower() if c.isalnum() or c in ' -_')
    return keep.replace(' ', '-')


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument('--out', default=str(REPO / 'RESULTS.md'))
    args = ap.parse_args()
    for load in (
        bench_jsonl,
        speed,
        harness_docx_to_pdf,
        docxide_metric_reports,
        fixtures_500_loop_runs,
        soffice_vs_word,
        english_corpus,
        redlined_compared_set,
        docxide_suite,
        pdf_to_docx,
    ):
        load()
    Path(args.out).write_text(render())
    print(f'wrote {args.out}: {len(METRICS)} metrics, {len(RUNS)} runs')


if __name__ == '__main__':
    main()
