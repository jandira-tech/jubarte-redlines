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

# Superseded --compress runs, dropped: the writer that embedded whole fonts (23abeb2), and
# 1b72452, whose device-scale glyphs MuPDF hinted differently (fixed in 530f46c).
SUPERSEDED = 'superseded --compress'
SUPERSEDED_FILES = {
    'docxide_metrics_jubarte-23abeb2-compressed.json',
    'docxide_metrics_jubarte-1b72452-compressed.json',
}
# The harness records the binary's own version; 530f46c is the 0.9.2 line (Cargo.toml lags).
FILE_VERSIONS = {'docxide_metrics_jubarte-530f46c-compressed.json': 'jubarte 0.9.2 --compress'}

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
    'fresh0926f': 'jubarte 0.9.2',
    'fresh0926g': 'jubarte 0.9.2',
    'compressed0926': SUPERSEDED,
    'compressed0926b': SUPERSEDED,
    # Subset fonts, narrowed /W, shared text objects (530f46c), 2026-09-26.
    'compressed0926c': 'jubarte 0.9.2 --compress',
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
    corpora: str = ''

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


# --- docx -> pdf, docxide-metrics Jaccard vs Word, pooled over every corpus -------------------

# Corpus -> label. Every corpus's references are Microsoft Word PDF exports.
CLEAN_CORPORA = {
    'en_a': 'English part a',
    'en_b': 'English part b',
    'fixtures_500': 'fixtures_500',
    'dxsuite': "docxide's 208-case suite",
    'nb398': 'neurotic no-redline 398',
}
REDLINE_CORPORA = {'en_r': 'English redlines a vs b', 'compared': 'compared_a_100_vs_b_10'}
TOOL_NAMES = {'docxide-pdf': 'docxide', 'generated': 'docxide', 'libreoffice': 'soffice', 'LibreOffice': 'soffice'}


@dataclass
class Sample:
    """One tool run on one corpus: per-document docxide-metrics Jaccard, 0-1."""

    corpus: str
    tool: str
    version: str
    when: datetime
    scores: dict[str, float]

    @property
    def mean(self) -> float:
        return statistics.mean(self.scores.values())


SAMPLES: list[Sample] = []


def jub(tag: str) -> str:
    """A run tagged `compressed` (convert --compress) is its own tool."""
    return 'jubarte-compressed' if 'compressed' in tag else 'jubarte'


def sample(corpus: str, tool: str, version: str, when: datetime, scores: dict[str, float]) -> None:
    scores = {k: float(v) for k, v in scores.items() if isinstance(v, (int, float))}
    if scores and version != SUPERSEDED:
        SAMPLES.append(Sample(corpus, TOOL_NAMES.get(tool, tool), version, when, scores))


def english_corpus() -> None:
    for p in 'abr':
        corpus = f'en_{p}'
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
            sample(corpus, jub(tag), version, when_of(None, path), {k: r['jubarte'] for k, r in rows.items()})
        comp = LOOP / 'runs' / f'en_{p}_competitors.json'
        if comp.exists():
            for tool, rows in json.loads(comp.read_text()).items():
                sample(corpus, tool, EN_COMPETITORS.get(tool, tool), when_of(None, comp), rows)


def fixtures_500() -> None:
    report = GROK / 'docxide_metrics' / 'report.json'
    if report.exists():
        doc = json.loads(report.read_text())
        for tool, t in (doc.get('tools') or {}).items():
            per_doc = {
                k: v['jaccard'] / 100 for k, v in (t.get('per_doc') or {}).items() if v.get('jaccard') is not None
            }
            sample(
                'fixtures_500',
                tool,
                str(t.get('version') or 'unversioned'),
                when_of(doc.get('generated_at'), report),
                per_doc,
            )
    for path in sorted((LOOP / 'runs').glob('full_*/result.json')):
        rows = json.loads(path.read_text())
        scores = {k: r['new'] for k, r in rows.items() if isinstance(r, dict) and r.get('new') is not None}
        if len(scores) < 400 or not any(scores.values()):
            continue  # partial runs, and runs whose output folder was missing
        tag = path.parent.name.removeprefix('full_')
        sample('fixtures_500', jub(tag), EN_TAGS.get(tag, f'jubarte@{tag}'), when_of(None, path), scores)


def docxide_suite() -> None:
    published = DXSUITE / 'published_scores.json'
    if published.exists():
        by_engine: dict[str, dict[str, float]] = {}
        for case in json.loads(published.read_text()):
            for engine, sc in (case.get('scores') or {}).items():
                if isinstance(sc, dict) and sc.get('jaccard') is not None:
                    by_engine.setdefault(engine, {})[case['case']] = sc['jaccard'] / 100
        for engine, scores in by_engine.items():
            sample('dxsuite', engine, 'published (sverrejb.github.io/docxide-pdf)', when_of(None, published), scores)
    for path in sorted((LOOP / 'dxruns').glob('*/result.json')):
        rows = json.loads(path.read_text())
        if len(rows) < 200:
            continue
        tag = path.parent.name.removeprefix('dx_')
        scores = {k: r['new'] for k, r in rows.items() if isinstance(r, dict) and r.get('new') is not None}
        sample('dxsuite', jub(tag), EN_TAGS.get(tag, f'jubarte@{tag}'), when_of(None, path), scores)


def neurotic_398() -> None:
    for path in sorted(RES.glob('docxide_metrics*.json')):
        if path.name in SUPERSEDED_FILES:
            continue
        doc = json.loads(path.read_text())
        if 'word' not in str(doc.get('oracle') or '').lower():
            continue
        for tool, t in (doc.get('tools') or {}).items():
            per_doc = {
                k: v['jaccard'] / 100 for k, v in (t.get('per_doc') or {}).items() if v.get('jaccard') is not None
            }
            version = str(t.get('version') or 'unversioned')
            # jubarte-first (the 0.2.0 docxToPdf adapter) is its own tool, not jubarte.
            name = 'jubarte-first' if 'jubarte-first' in version else tool
            if name == 'jubarte':
                name = jub(path.name)
            version = FILE_VERSIONS.get(path.name, version)
            sample('nb398', name, version, when_of(doc.get('generated_at'), path), per_doc)


def redlined_compared_set() -> None:
    redl = LOOP / 'redl'
    base = redl / 'all_scores.json'
    if base.exists():
        doc = json.loads(base.read_text())
        for tool, ver in REDL_COMPETITORS.items():
            sample('compared', tool, ver, when_of(None, base), doc.get(tool) or {})
    for path in sorted(redl.glob('*_rows.json')):
        rows = json.loads(path.read_text())
        if len(rows) < 900:
            continue
        tag = path.name.removesuffix('_rows.json').removeprefix('rl_')
        sample('compared', jub(tag), EN_TAGS.get(tag, f'jubarte@{tag}'), when_of(None, path), rows)


def ours(tool: str) -> bool:
    return tool.startswith('jubarte')


def pooled(corpora: dict[str, str], key: str, title: str, docs: str) -> None:
    """One table over every corpus in `corpora`: a run's per-document scores pooled across them.

    A competitor pools its best run per corpus. Jubarte gets one row per 7-day window: per
    corpus, its best run inside the window, else its latest run before it.
    """
    metric(key, title=title, kind='docx->pdf', reference='Word', docs=docs, unit='docxide-metrics Jaccard 0-1')
    mine = [s for s in SAMPLES if s.corpus in corpora]

    def emit(tool: str, picks: dict[str, Sample]) -> None:
        scores = [v for s in picks.values() for v in s.scores.values()]
        if not scores:
            return
        mean, median = stats(scores)
        versions = sorted({s.version for s in picks.values()})
        cover = ', '.join(f'{corpora[c]} {len(picks[c].scores)}' for c in corpora if c in picks)
        add(
            metric=key,
            tool=tool,
            version='; '.join(versions),
            when=max(s.when for s in picks.values()),
            mean=mean,
            median=median,
            n=len(scores),
            corpora=cover,
        )

    for tool in sorted({s.tool for s in mine if not ours(s.tool)}):
        picks: dict[str, Sample] = {}
        for s in mine:
            if s.tool == tool and (s.corpus not in picks or s.mean > picks[s.corpus].mean):
                picks[s.corpus] = s
        emit(tool, picks)
    for tool in sorted({s.tool for s in mine if ours(s.tool)}):
        jub = sorted((s for s in mine if s.tool == tool), key=lambda s: s.when, reverse=True)
        end = jub[0].when
        oldest = jub[-1].when
        while end >= oldest:
            start = end - timedelta(days=7)
            picks = {}
            for c in corpora:
                inside = [s for s in jub if s.corpus == c and start < s.when <= end]
                before = [s for s in jub if s.corpus == c and s.when <= start]
                if inside:
                    picks[c] = max(inside, key=lambda s: s.mean)
                elif before:
                    picks[c] = before[0]
            if any(start < s.when <= end for s in picks.values()):
                emit(tool, picks)
            end = start


def stats(values: list[float]) -> tuple[float, float]:
    return statistics.mean(values), statistics.median(values)


def docx_to_pdf_pooled() -> None:
    english_corpus()
    fixtures_500()
    docxide_suite()
    neurotic_398()
    redlined_compared_set()
    pooled(CLEAN_CORPORA, 'pool:clean', 'docx→pdf — every clean corpus pooled (no redlines)', 'clean')
    pooled(REDLINE_CORPORA, 'pool:redlines', 'docx→pdf — every redlined corpus pooled (redlines only)', 'redlines')


def pdf_to_docx() -> None:
    metric('pdf_to_docx', title='pdf→docx', kind='pdf->docx', reference='-', docs='-', unit='-')


# --- report ----------------------------------------------------------------------------------


def best_per_window(runs: list[Run], lower: bool) -> list[Run]:
    """Best run per 7-day window (windows counted back from the newest run): per tool for
    jubarte, whatever its version; per tool and version for a competitor."""
    out: list[Run] = []
    groups: dict[tuple[str, str], list[Run]] = {}
    for r in runs:
        if r.rank_value is None:
            continue
        groups.setdefault((r.tool, '' if ours(r.tool) else r.version), []).append(r)
    for group in groups.values():
        group.sort(key=lambda r: r.when, reverse=True)
        while group:
            start = group[0].when - timedelta(days=7)
            window = [r for r in group if r.when > start]
            group = [r for r in group if r.when <= start]
            pick = min if lower else max
            out.append(pick(window, key=lambda r: r.rank_value))
    return out


VERSION_WIDTH = 20


def short_version(v: str) -> str:
    """A Version cell of at most VERSION_WIDTH chars; a cut ends in an ellipsis."""
    return v if len(v) <= VERSION_WIDTH else v[: VERSION_WIDTH - 1] + '…'


def fmt(v: float | None) -> str:
    if v is None:
        return '—'
    return f'{v:.4f}' if abs(v) < 2 else f'{v:.2f}'


KIND_ORDER = ['docx->pdf', 'redline markup', 'redline speed', 'pdf->docx']
# Reference groups, in page order: Word exports first, then soffice renders, then the rest.
GROUPS = [
    ('Microsoft Word® reference PDFs', lambda m: m.reference.startswith('Word') and 'soffice' not in m.reference),
    ('soffice-rendered PDFs', lambda m: 'soffice' in m.reference),
    ('No reference PDFs (speed, pdf→docx)', lambda m: True),
]


def group_of(m: Metric) -> int:
    return next(i for i, (_, test) in enumerate(GROUPS) if test(m))


def render() -> str:
    lines = [
        # REUSE-IgnoreStart
        '<!-- SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC -->',
        '<!-- SPDX-License-Identifier: AGPL-3.0-only -->',
        # REUSE-IgnoreEnd
        '<!-- Generated by scripts/results_by_version.py — do not edit by hand. -->',
        '# Results by tool version',
        '',
        f'Generated {datetime.now(timezone.utc).strftime("%Y-%m-%d %H:%M UTC")}. Jubarte keeps one row per',
        '7-day window (newest first): its best run of that week, whatever the version. A competitor',
        'keeps one row per version. Rows rank by mean (speed: lowest first); a row without a mean',
        'ranks by the average of its per-document scores. Scores are only comparable inside one table.',
        '',
        'The docx→pdf docxide-metrics tables pool the per-document Jaccard of every corpus the tools',
        'converted, each scored against the Microsoft Word® PDF export of the same document: one table',
        'for the clean corpora, one for the redlined documents only. A competitor pools its best run',
        'per corpus; the Corpora column shows which corpora (and how many documents) each row covers,',
        'so a row missing a corpus is averaged over fewer documents.',
        '',
        '| Metric | Kind | Reference PDFs | Documents | Unit |',
        '| --- | --- | --- | --- | --- |',
    ]
    ordered = sorted(
        METRICS.values(),
        key=lambda m: (group_of(m), KIND_ORDER.index(m.kind), not m.key.startswith('pool:'), m.docs, m.title),
    )
    lines.extend(
        f'| [{m.title}](#{anchor(m.title)}) | {m.kind} | {m.reference} | {m.docs} | {m.unit} |' for m in ordered
    )
    for gi, (heading, _) in enumerate(GROUPS):
        mets = [m for m in ordered if group_of(m) == gi]
        if not mets:
            continue
        lines += ['', f'## {heading}']
        for m in mets:
            lines += ['', f'### {m.title}', '']
            lines.append(
                f'Kind: {m.kind}. Reference PDFs: **{m.reference}**. Documents: **{m.docs}**. Unit: {m.unit}.'
            )
            runs = [r for r in RUNS if r.metric == m.key]
            if not m.key.startswith('pool:'):  # pooled runs are already one per week
                runs = best_per_window(runs, m.lower_is_better)
            runs = [r for r in runs if r.rank_value is not None]
            if not runs:
                lines += ['', '_No run measured yet._']
                continue
            runs.sort(key=lambda r: r.rank_value, reverse=not m.lower_is_better)
            pool = m.key.startswith('pool:')
            head = '| Rank | Tool | Version | Date | Docs | Mean | Median |' + (' Corpora |' if pool else '')
            lines += ['', head, '|' + ' --- |' * (8 if pool else 7)]
            for i, r in enumerate(runs, 1):
                mean = fmt(r.mean) if r.mean is not None else f'{fmt(r.rank_value)} (avg)'
                row = f'| {i} | {r.tool} | {short_version(r.version)} | {r.when.strftime("%Y-%m-%d")} | {r.n} | {mean} | {fmt(r.median)} |'
                lines.append(row + (f' {r.corpora} |' if pool else ''))
    return '\n'.join(lines) + '\n'


def anchor(title: str) -> str:
    keep = ''.join(c for c in title.lower() if c.isalnum() or c in ' -_')
    return keep.replace(' ', '-')


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument('--out', default=str(REPO / 'RESULTS.md'))
    args = ap.parse_args()
    for load in (bench_jsonl, speed, harness_docx_to_pdf, soffice_vs_word, docx_to_pdf_pooled, pdf_to_docx):
        load()
    Path(args.out).write_text(render())
    print(f'wrote {args.out}: {len(METRICS)} metrics, {len(RUNS)} runs')


if __name__ == '__main__':
    main()
