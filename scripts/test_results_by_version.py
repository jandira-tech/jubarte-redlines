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


def test_docx_to_pdf_speed_skips_incomplete_timing_rows(monkeypatch, tmp_path):
    _fresh(monkeypatch, tmp_path)
    (tmp_path / 'docx_to_pdf_speed').mkdir()
    row = {
        'tool': 'jubarte',
        'version': 'jubarte 0.9.2@65ce9de',
        'corpus': 'fixtures_500',
        'run_ts': '2026-09-27T00-31-42Z',
        'n': 2,
        'mean': 50.0,
        'median': 40.0,
    }
    broken = [
        {k: v for k, v in row.items() if k != field} for field in ('tool', 'version', 'run_ts', 'n', 'mean', 'median')
    ]
    broken += [{**row, 'run_ts': '27/09/2026'}, {**row, 'version': None}, {**row, 'mean': 'fast'}]
    lines = [json.dumps(r) for r in broken] + [json.dumps(row)]
    (tmp_path / 'docx_to_pdf_speed' / 'speed.jsonl').write_text('\n'.join(lines) + '\n')
    rv.docx_to_pdf_speed()
    assert [(r.tool, r.version, r.median) for r in rv.RUNS] == [('jubarte', 'jubarte@65ce9de', 40.0)]


def test_render_ranks_by_mean_and_drops_unranked_runs(monkeypatch, tmp_path):
    _fresh(monkeypatch, tmp_path)
    key = rv.metric('m', title='T', kind='redline markup', reference='Word', docs='redlines', unit='u')
    when = rv.when_of('2026-09-26T00:00:00Z')
    rv.add(metric=key, tool='low', version='1', when=when, mean=1.0, median=1.0, n=1)
    rv.add(metric=key, tool='high', version='2', when=when, mean=2.0, median=2.0, n=1)
    rv.add(metric=key, tool='none', version='3', when=when, mean=None, median=None, n=0)
    rows = [line for line in rv.render().splitlines() if line.startswith('| 1 |') or line.startswith('| 2 |')]
    assert [r.split(' | ')[1] for r in rows] == ['high', 'low']


def test_jsonl_readers_skip_lines_that_are_not_objects(monkeypatch, tmp_path):
    _fresh(monkeypatch, tmp_path)
    junk = 'null\n[1]\n"x"\n7\n{"corpus": null}\n'
    (tmp_path / 'docx_to_pdf_speed').mkdir()
    (tmp_path / 'docx_to_pdf_speed' / 'speed.jsonl').write_text(junk)
    (tmp_path / 'speed.jsonl').write_text(junk)
    (tmp_path / 'bench.jsonl').write_text(junk)
    rv.docx_to_pdf_speed()
    rv.speed_rows(tmp_path / 'speed.jsonl')
    rv.bench_jsonl()
    assert rv.RUNS == []


@pytest.mark.parametrize('stem', [
    'A_redline_harness', 'A_redline-_harness', 'A_redline-v1_unknown',
    'A_redline-v1_jobs', 'A_redline-v1_missing', 'Z_unknown-v1_harness', '',
])
def test_wordpdf_row_rejects_non_result_names(stem):
    assert rv.wordpdf_row(stem) is None


@pytest.mark.parametrize(('stem', 'expected'), [
    ('en_C_convert~docxide-0.17.1-rc_2_docxide',
     ('en_', 'conversion', 'docxide PDF (Word redline)', 'docxide 0.17.1-rc_2', 'docxide')),
    ('B_convert-fresh0926f_harness',
     ('', 'conversion', 'jubarte PDF (Word redline)', 'jubarte 0.9.2', 'harness')),
    ('D_e2e-abc123_harness',
     ('', 'end to end', 'jubarte redline + jubarte PDF', 'jubarte@abc123', 'harness')),
])
def test_wordpdf_row_preserves_version_delimiters_and_resolves_known_tags(stem, expected):
    assert rv.wordpdf_row(stem) == expected


def test_jsonl_objects_recovers_after_bad_records_and_accepts_empty_objects():
    lines = iter(['{"n": 1}', '{"broken":', '', 'null', '[1]', '"text"', '0', '{}', '{"n": 2}'])
    assert list(rv.jsonl_objects(lines)) == [{'n': 1}, {}, {'n': 2}]


def test_wordpdf_empty_scores_do_not_register_metrics_and_zero_scores_count(monkeypatch, tmp_path):
    out = _fresh(monkeypatch, tmp_path)
    (out / 'A_redline-empty_harness.json').write_text('{"bad": null, "text": "90"}')
    (out / 'B_convert-empty_docxide.json').write_text('[]')
    (out / 'en_A_redline-zero_harness.json').write_text('{"failed": 0, "good": 100}')
    (out / 'en_B_convert-zero_docxide.json').write_text('[{}, {"jaccard": 0}, {"jaccard": 0.9}]')
    rv.redline_wordpdf()
    by_metric = {r.metric: r for r in rv.RUNS}
    assert set(rv.METRICS) == set(by_metric) == {
        'wordpdf:en_:redlining:harness', 'wordpdf:en_:conversion:docxide',
    }
    harness = by_metric['wordpdf:en_:redlining:harness']
    assert (harness.n, harness.mean, harness.median) == (2, 50.0, 50.0)
    docxide = by_metric['wordpdf:en_:conversion:docxide']
    assert docxide.n == 3
    assert docxide.mean == pytest.approx(0.3)
    assert docxide.median == 0.0


@pytest.mark.parametrize(('value', 'unit', 'expected'), [
    (None, 'ms per document', '—'), (0.0, 'ms per document', '0.00'),
    (0.125, 'harness score 0-100', '0.12'), (0.125, 'Jaccard 0-1', '0.1250'),
    (1.0, 'Jaccard 0-1', '1.0000'), (100.0, 'harness score 0-100', '100.00'),
])
def test_metric_units_determine_display_precision(value, unit, expected):
    assert rv.fmt(value, unit) == expected


def test_conversion_speed_keeps_competitor_versions_and_unknown_corpus(monkeypatch, tmp_path):
    _fresh(monkeypatch, tmp_path)
    speed = tmp_path / 'docx_to_pdf_speed'
    speed.mkdir()
    rows = [
        {'corpus': 'custom_set', 'tool': 'other', 'version': 'other@v2',
         'n': 0, 'mean': 0, 'median': 0, 'run_ts': '2026-09-27T01-02-03Z'},
        {'corpus': 'word_redline_en', 'tool': 'jubarte', 'version': 'jubarte 0.9.2',
         'n': 3, 'mean': 2, 'median': 1, 'run_ts': '2026-09-27T01-02-03Z', 'mode': 'warm'},
    ]
    (speed / 'speed.jsonl').write_text('\n'.join(map(json.dumps, rows)))
    rv.docx_to_pdf_speed()
    assert [r.version for r in rv.RUNS] == ['other@v2', 'jubarte 0.9.2']
    assert (rv.RUNS[0].n, rv.RUNS[0].mean, rv.RUNS[0].median) == (0, 0, 0)
    assert rv.RUNS[0].when.isoformat() == '2026-09-27T01:02:03+00:00'
    assert rv.METRICS['speed:docx2pdf:custom_set'].docs == 'clean'
    assert rv.METRICS['speed:docx2pdf:warm:word_redline_en'].docs == 'redlines'


def test_render_separates_pipeline_stages_and_ranks_speed_lowest_first(monkeypatch, tmp_path):
    out = _fresh(monkeypatch, tmp_path)
    for row in ('D_e2e', 'C_convert', 'A_redline'):
        (out / f'{row}-abc123_harness.json').write_text('{"one": 80}')
    rv.redline_wordpdf()
    key = rv.metric('speed:test', title='Timing', kind='docx->pdf speed', reference='-',
                    docs='clean', unit='ms per document', lower_is_better=True)
    for tool, mean in [('slow', 10.0), ('fast', 0.5)]:
        rv.add(metric=key, tool=tool, version='v1', when=rv.when_of('2026-09-27'),
               mean=mean, median=mean, n=1)
    rendered = rv.render()
    headings = [line for line in rendered.splitlines() if line.startswith('### ')]
    assert [heading.split(' vs ')[0] for heading in headings[:3]] == [
        '### Redlining', '### Conversion', '### End to end',
    ]
    timing = rendered.split('### Timing\n', 1)[1]
    rows = [line for line in timing.splitlines() if line.startswith(('| 1 |', '| 2 |'))]
    assert [line.split(' | ')[1] for line in rows] == ['fast', 'slow']
    assert '| 0.50 | 0.50 |' in rows[0]
