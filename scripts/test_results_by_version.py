# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
# SPDX-License-Identifier: AGPL-3.0-only
"""Tests for results_by_version.py's Word-rendered redline loader and ranking."""

from __future__ import annotations

import json
import sys
from pathlib import Path

import pytest

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
    (out / 'en_A_redline-cb33ec3_harness.json').write_text(json.dumps({'a': 90.0, 'b': 80.0}))
    (out / 'en_A_redline-cb33ec3_docxide.json').write_text(json.dumps([{'jaccard': 0.5}, {'jaccard': None}]))
    (out / 'en_A_redline-cb33ec3_jobs.json').write_text('[]')
    (out / 'en_A_redline-cb33ec3_missing.json').write_text('[]')
    (out / 'Z_unknown-cb33ec3_harness.json').write_text(json.dumps({'a': 1.0}))
    rv.redline_wordpdf()
    by_metric = {r.metric: r for r in rv.RUNS}
    assert set(by_metric) == {'wordpdf:en_:redlining:harness', 'wordpdf:en_:redlining:docxide'}
    harness = by_metric['wordpdf:en_:redlining:harness']
    assert (harness.tool, harness.version, harness.n, harness.mean) == (
        'jubarte redline (Word PDF)',
        'jubarte@cb33ec3',
        2,
        85.0,
    )
    docxide = by_metric['wordpdf:en_:redlining:docxide']
    assert (docxide.n, docxide.mean) == (2, 0.25)  # a missing Jaccard scores 0, it is not dropped
    assert rv.METRICS['wordpdf:en_:redlining:harness'].kind == 'redline markup'


def test_wordpdf_rows_name_the_side_they_swap():
    assert rv.wordpdf_row('A_redline~docxodus-12.6.2_harness') == (
        '',
        'redlining',
        'docxodus redline (Word PDF)',
        'docxodus 12.6.2',
        'harness',
    )
    assert rv.wordpdf_row('C_soffice-cb33ec3_docxide') == (
        '',
        'conversion',
        'soffice PDF (Word redline)',
        rv.EN_COMPETITORS['soffice'],
        'docxide',
    )
    assert rv.wordpdf_row('en_E_e2e_soffice-cb33ec3_harness') == (
        'en_',
        'end to end',
        'jubarte redline + soffice PDF',
        'jubarte@cb33ec3',
        'harness',
    )
    assert rv.wordpdf_row('A_redline-cb33ec3_missing') is None


def test_docx_to_pdf_speed_skips_a_torn_line_and_splits_warm_from_cold(monkeypatch, tmp_path):
    _fresh(monkeypatch, tmp_path)
    (tmp_path / 'docx_to_pdf_speed').mkdir()
    row = {
        'unit': 'ms_per_docx',
        'tool': 'jubarte',
        'version': 'jubarte 0.9.2@65ce9de',
        'corpus': 'fixtures_500',
        'run_ts': '2026-09-27T00-31-42Z',
        'n': 2,
        'failed': 0,
        'mean': 50.0,
        'median': 40.0,
    }
    warm = {**row, 'mean': 5.0, 'median': 4.0, 'mode': 'warm'}
    (tmp_path / 'docx_to_pdf_speed' / 'speed.jsonl').write_text(
        json.dumps(row) + '\n' + json.dumps(warm) + '\n' + '{"unit": "ms_per_docx", "tool": "jub'
    )
    rv.docx_to_pdf_speed()
    by_metric = {r.metric: r for r in rv.RUNS}
    assert set(by_metric) == {'speed:docx2pdf:fixtures_500', 'speed:docx2pdf:warm:fixtures_500'}
    assert by_metric['speed:docx2pdf:warm:fixtures_500'].median == pytest.approx(4.0)
    assert 'long-lived worker' in rv.METRICS['speed:docx2pdf:warm:fixtures_500'].note


def test_render_ranks_by_mean_and_drops_unranked_runs(monkeypatch, tmp_path):
    _fresh(monkeypatch, tmp_path)
    key = rv.metric('m', title='T', kind='redline markup', reference='Word', docs='redlines', unit='u')
    when = rv.when_of('2026-09-26T00:00:00Z')
    rv.add(metric=key, tool='low', version='1', when=when, mean=1.0, median=1.0, n=1)
    rv.add(metric=key, tool='high', version='2', when=when, mean=2.0, median=2.0, n=1)
    rv.add(metric=key, tool='none', version='3', when=when, mean=None, median=None, n=0)
    rows = [line for line in rv.render().splitlines() if line.startswith('| 1 |') or line.startswith('| 2 |')]
    assert [r.split(' | ')[1] for r in rows] == ['high', 'low']
