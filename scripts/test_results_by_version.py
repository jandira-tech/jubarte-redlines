# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Tests for results_by_version.py's Word-rendered redline loader and ranking."""

from __future__ import annotations

import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
if str(HERE) not in sys.path:
    sys.path.insert(0, str(HERE))

import results_by_version as rv


def _fresh(monkeypatch, tmp_path: Path) -> Path:
    monkeypatch.setattr(rv, 'RES', tmp_path)
    monkeypatch.setattr(rv, 'METRICS', {})
    monkeypatch.setattr(rv, 'RUNS', [])
    out = tmp_path / 'redline_wordpdf'
    out.mkdir()
    return out


def test_redline_wordpdf_reads_both_scorers_and_skips_jobs(monkeypatch, tmp_path):
    out = _fresh(monkeypatch, tmp_path)
    (out / 'en_redlines-cb33ec3_harness.json').write_text(json.dumps({'a': 90.0, 'b': 80.0}))
    (out / 'en_redlines-cb33ec3_docxide.json').write_text(json.dumps([{'jaccard': 0.5}, {'jaccard': None}]))
    (out / 'en_redlines-cb33ec3_jobs.json').write_text('[]')
    (out / 'unknown_set-cb33ec3_harness.json').write_text(json.dumps({'a': 1.0}))
    rv.redline_wordpdf()
    by_metric = {r.metric: r for r in rv.RUNS}
    assert set(by_metric) == {'wordpdf:en_redlines:harness', 'wordpdf:en_redlines:docxide'}
    harness = by_metric['wordpdf:en_redlines:harness']
    assert (harness.tool, harness.version, harness.n, harness.mean) == ('jubarte', 'jubarte@cb33ec3', 2, 85.0)
    docxide = by_metric['wordpdf:en_redlines:docxide']
    assert (docxide.n, docxide.mean) == (2, 0.25)  # a missing Jaccard scores 0, it is not dropped
    assert rv.METRICS['wordpdf:en_redlines:harness'].kind == 'redline markup'


def test_word_control_tag_is_its_own_tool(monkeypatch, tmp_path):
    out = _fresh(monkeypatch, tmp_path)
    (out / 'harness_fresh_word-wordcorpus_harness.json').write_text(json.dumps({'a': 97.0}))
    rv.redline_wordpdf()
    (run,) = rv.RUNS
    assert (run.tool, run.version) == ('word (older corpus redline)', 'wordcorpus')


def test_render_ranks_by_mean_and_drops_unranked_runs(monkeypatch, tmp_path):
    _fresh(monkeypatch, tmp_path)
    key = rv.metric('m', title='T', kind='redline markup', reference='Word', docs='redlines', unit='u')
    when = rv.when_of('2026-09-26T00:00:00Z')
    rv.add(metric=key, tool='low', version='1', when=when, mean=1.0, median=1.0, n=1)
    rv.add(metric=key, tool='high', version='2', when=when, mean=2.0, median=2.0, n=1)
    rv.add(metric=key, tool='none', version='3', when=when, mean=None, median=None, n=0)
    rows = [line for line in rv.render().splitlines() if line.startswith('| 1 |') or line.startswith('| 2 |')]
    assert [r.split(' | ')[1] for r in rows] == ['high', 'low']
