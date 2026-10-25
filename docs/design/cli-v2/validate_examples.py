#!/usr/bin/env python3

# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""Validate design schemas and a deliberately limited, in-memory text model.

This DOES NOT execute the proposed CLI or edit DOCX packages. It cannot establish
tracking, revision settlement, comment anchoring, range preservation or rendering.
The model checks the bundled four-paragraph fixture, exact text composition,
working-text guards and the proposed paragraph-expectation rules only.
"""
from __future__ import annotations

from copy import deepcopy
from datetime import datetime
import hashlib
import json
from pathlib import Path
import re
import zipfile
import xml.etree.ElementTree as ET

from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parent
EXAMPLES = ROOT / 'examples'
NS = {'w': 'http://schemas.openxmlformats.org/wordprocessingml/2006/main'}


class ContractError(ValueError):
    pass


def require(condition, code):
    if not condition:
        raise ContractError(code)


def read_json(path):
    return json.loads(path.read_text(encoding='utf-8'))


def text_hash(text):
    return hashlib.sha256(text.encode('utf-8')).hexdigest()


def source_texts(path):
    with zipfile.ZipFile(path) as package:
        document = ET.fromstring(package.read('word/document.xml'))
    require(not document.findall('.//w:tbl', NS), 'ORACLE_TABLE_UNSUPPORTED')
    require(not document.findall('.//w:ins', NS), 'ORACLE_REVISIONS_UNSUPPORTED')
    require(not document.findall('.//w:del', NS), 'ORACLE_REVISIONS_UNSUPPORTED')
    return [''.join(node.text or '' for node in p.findall('.//w:t', NS))
            for p in document.findall('./w:body/w:p', NS)]


def target_index(target, texts):
    short = target['short_id']
    require(re.fullmatch(r'p[0-9]+', short) is not None, 'ORACLE_TARGET_UNSUPPORTED')
    index = int(short[1:])
    require(index < len(texts), 'UNKNOWN_TARGET')
    require(target['long_id'] == f'document.w:p.j:short.{short}', 'ID_PAIR_MISMATCH')
    return index


def literal_span(old, needle, occurrence=None):
    hits, cursor = [], 0
    require(needle != '', 'EMPTY_ANCHOR')
    while (start := old.find(needle, cursor)) != -1:
        hits.append((start, start + len(needle)))
        cursor = start + 1  # Include overlapping possible literal start positions.
    if occurrence is None:
        require(len(hits) != 0, 'ANCHOR_NOT_FOUND')
        require(len(hits) == 1, 'AMBIGUOUS_ANCHOR')
        selected = hits[0]
    else:
        require(1 <= occurrence <= len(hits), 'OCCURRENCE_OUT_OF_RANGE')
        selected = hits[occurrence - 1]
    return selected


def compose(old, spans):
    """All spans were resolved on the same working string, never intermediate text."""
    ordered = sorted(spans, key=lambda x: (x[0], x[3]))
    for pos, first in enumerate(ordered):
        a, b, _, _ = first
        for second in ordered[pos + 1:]:
            c, d, _, _ = second
            if a == b and c == d:
                continue  # Same-position insertions preserve request-array order.
            if a == b:
                require(not c <= a <= d, 'OVERLAPPING_OPERATIONS')
            elif c == d:
                require(not a <= c <= b, 'OVERLAPPING_OPERATIONS')
            else:
                require(b <= c or d <= a, 'OVERLAPPING_OPERATIONS')
    parts, cursor = [], 0
    for start, end, replacement, _ in ordered:
        parts.extend([old[cursor:start], replacement])
        cursor = end
    parts.append(old[cursor:])
    return ''.join(parts)


def fixture_oracle(plan, source, digest, working=None, one_to_one=True):
    """A text-only model. working override tests guards, not actual settlement."""
    require(plan['source']['sha256'] == digest, 'STALE_SOURCE')
    require(plan['resolve_revisions'] == {'accept': [], 'reject': []}, 'ORACLE_REVISIONS_UNSUPPORTED')
    require(plan['policy']['text_view'] == 'final', 'ORACLE_VIEW_UNSUPPORTED')
    working = source if working is None else working
    require(len(working) == len(source), 'ORACLE_MAPPING_UNSUPPORTED')
    selected = (set(range(len(source))) if plan['scope']['kind'] == 'all' else
                {target_index(t, source) for t in plan['scope']['targets']})

    for record in plan.get('source_context', []):
        require(record['kind'] == 'paragraph', 'ORACLE_CONTEXT_UNSUPPORTED')
        index = target_index(record['target'], source)
        require(record['text'] == source[index], 'READ_ONLY_CONTEXT_CHANGED')

    spans = {index: [] for index in range(len(source))}
    rewrites, deletions = {}, set()
    before = {index: [] for index in range(len(source))}
    after = {index: [] for index in range(len(source))}
    unchanged, active_body_requests, abstract_comments = 0, 0, 0
    for pointer, operation in enumerate(plan['operations']):
        index = target_index(operation['target'], source)
        require(index in selected, 'TARGET_OUT_OF_SCOPE')
        old, kind = working[index], operation['kind']
        if 'attached_comment' in operation:
            require(operation['text'] != '', 'EMPTY_COMMENT_RESULT')
            abstract_comments += 1  # Declaration only: the model never builds comment XML.
        if kind == 'rewrite':
            require(one_to_one, 'UNSUPPORTED_MAPPING')
            require(operation['before_sha256'] == text_hash(old), 'WORKING_BASE_CHANGED')
            if operation['text'] == old:
                unchanged += 1
                continue
            require(index not in rewrites, 'OVERLAPPING_OPERATIONS')
            rewrites[index] = operation['text']
        elif kind in {'replace', 'delete'}:
            start, end = literal_span(old, operation['find'], operation.get('occurrence'))
            replacement = operation.get('text', '') if kind == 'replace' else ''
            if old[start:end] == replacement:
                unchanged += 1
                continue
            spans[index].append((start, end, replacement, pointer))
        elif kind == 'insert':
            if 'edge' in operation:
                location = 0 if operation['edge'] == 'start' else len(old)
            else:
                key = 'before_text' if 'before_text' in operation else 'after_text'
                start, end = literal_span(old, operation[key], operation.get('occurrence'))
                location = start if key == 'before_text' else end
            spans[index].append((location, location, operation['text'], pointer))
        elif kind == 'insert_paragraph':
            (before if operation['position'] == 'before' else after)[index].append(operation['text'])
        elif kind == 'delete_paragraph':
            require(index not in deletions, 'OVERLAPPING_OPERATIONS')
            deletions.add(index)
        else:
            raise ContractError('ORACLE_REVIEW_OPERATION_UNSUPPORTED')
        active_body_requests += 1

    output, source_to_output = [], {}
    for index, old in enumerate(working):
        require(not (index in rewrites and spans[index]), 'OVERLAPPING_OPERATIONS')
        require(not (index in deletions and (index in rewrites or spans[index])), 'OVERLAPPING_OPERATIONS')
        require(not (index in deletions and (before[index] or after[index])), 'DELETED_INSERTION_ANCHOR')
        output.extend(before[index])
        if index not in deletions:
            source_to_output[index] = len(output)
            output.append(rewrites.get(index, compose(old, spans[index])))
        output.extend(after[index])
    return {'text': output, 'source_to_output': source_to_output,
            'active_body_requests': active_body_requests, 'unchanged_text_components': unchanged,
            'declared_attached_comments': abstract_comments}


def expectation_oracle(spec, source, digest, candidate):
    require(spec['source']['sha256'] == digest, 'EXPECTATION_SOURCE_MISMATCH')
    keys = [entry['key'] for entry in spec['requirements']]
    require(len(keys) == len(set(keys)), 'DUPLICATE_REQUIREMENT_KEY')
    passed = unassessed = 0
    for entry in spec['requirements']:
        if not entry['checks']:
            unassessed += 1
            continue
        for check in entry['checks']:
            index = target_index(check['target'], source)
            require(index in candidate['source_to_output'], 'EXPECTATION_TARGET_UNMAPPABLE')
            value = candidate['text'][candidate['source_to_output'][index]]
            wanted, kind = check['text'], check['kind']
            valid = (value == wanted if kind == 'text_equals' else
                     wanted in value if kind == 'text_contains' else wanted not in value)
            require(valid, 'EXPECTATION_FAILED')
            passed += 1
    return {'passed_checks': passed, 'unassessed_requirements': unassessed}


def expect_error(call, code):
    try:
        call()
    except ContractError as error:
        require(str(error) == code, f'WRONG_ERROR:{error}:{code}')
    else:
        raise AssertionError(f'Expected {code}')


def main():
    schema = read_json(ROOT / 'jubarte-edit-plan-v2-core.schema.json')
    expected_schema = read_json(ROOT / 'jubarte-expectations-v1.schema.json')
    for item in [schema, expected_schema]:
        Draft202012Validator.check_schema(item)
    formats = FormatChecker()
    @formats.checks('date-time', raises=ValueError)
    def timestamp(value):
        if not isinstance(value, str):
            return True  # Type validation belongs to the schema.
        parsed = datetime.fromisoformat(value.replace('Z', '+00:00'))
        return parsed.utcoffset() is not None
    validator = Draft202012Validator(schema, format_checker=formats)
    expectation_validator = Draft202012Validator(expected_schema, format_checker=formats)
    source_path = EXAMPLES / 'example-source.docx'
    digest = hashlib.sha256(source_path.read_bytes()).hexdigest()
    source = source_texts(source_path)
    assert source == ['Payment is due within 30 days.', 'The fee is due; the fee is payable.',
                      'One  space.', 'Notice is due within 30\u00a0days.']
    expected = read_json(EXAMPLES / 'expected-changes.json')
    spec = read_json(EXAMPLES / 'expectations.json')
    expectation_validator.validate(spec)
    plans, examples = {}, []
    for filename in ['template.generated.json', 'template.edited.json', 'template.exact-replace.json',
                     'template.composed.json', 'template.with-comment.json']:
        plan = read_json(EXAMPLES / filename)
        plans[filename] = plan
        validator.validate(plan)
        candidate = fixture_oracle(plan, source, digest)
        if filename == 'template.generated.json':
            assert candidate['text'] == source and candidate['active_body_requests'] == 0
            assert candidate['unchanged_text_components'] == 4
        elif filename == 'template.composed.json':
            assert candidate['text'][1] == 'The charge is due; the fee is due.'
            assert candidate['active_body_requests'] == 2 and candidate['unchanged_text_components'] == 1
        else:
            assert candidate['text'] == expected['expected_final_paragraph_texts']
            assert candidate['active_body_requests'] == 3
            assert expectation_oracle(spec, source, digest, candidate) == {
                'passed_checks': 3, 'unassessed_requirements': 1}
        examples.append({'file': filename, 'schema_valid': True,
                         'active_body_requests': candidate['active_body_requests'],
                         'unchanged_text_components': candidate['unchanged_text_components'],
                         'declared_attached_comments': candidate['declared_attached_comments']})

    generated = plans['template.generated.json']
    exact = plans['template.exact-replace.json']
    semantic_cases = []
    def negative(name, plan, code, **kwargs):
        validator.validate(plan)
        expect_error(lambda: fixture_oracle(plan, source, digest, **kwargs), code)
        semantic_cases.append(name)
    changed = deepcopy(generated); changed['source']['sha256'] = '0' * 64
    negative('stale source', changed, 'STALE_SOURCE')
    changed = deepcopy(generated); changed['operations'][0]['target']['long_id'] = 'document.w:p.j:short.p1'
    negative('mismatched ID pair', changed, 'ID_PAIR_MISMATCH')
    changed = deepcopy(generated); changed['operations'][0]['before_sha256'] = '0' * 64
    negative('unchanged row still verifies before hash', changed, 'WORKING_BASE_CHANGED')
    drift = list(source); drift[0] = 'Payment is due within 45 days.'
    negative('changed working basis never restores old scaffold', generated, 'WORKING_BASE_CHANGED', working=drift)
    negative('whole rewrite rejects non-one-to-one mapping', generated, 'UNSUPPORTED_MAPPING', one_to_one=False)
    changed = deepcopy(exact); changed['operations'][1].pop('occurrence')
    negative('ambiguous repeated literal', changed, 'AMBIGUOUS_ANCHOR')
    changed = deepcopy(exact); changed['operations'][1]['occurrence'] = 3
    negative('occurrence outside matches', changed, 'OCCURRENCE_OUT_OF_RANGE')
    overlapping_text = list(source); overlapping_text[0] = 'aaa'
    changed = deepcopy(exact); changed['operations'] = [
        {'kind': 'replace', 'target': generated['operations'][0]['target'], 'find': 'aa', 'text': 'X'}]
    negative('overlapping literal starts count as ambiguous', changed, 'AMBIGUOUS_ANCHOR', working=overlapping_text)
    for occurrence, wanted in [(1, 'Xa'), (2, 'aX')]:
        changed['operations'][0]['occurrence'] = occurrence
        assert fixture_oracle(changed, source, digest, working=overlapping_text)['text'][0] == wanted
        semantic_cases.append(f'overlapping literal occurrence {occurrence} selects exact start')
    changed = deepcopy(exact); changed['operations'] = [deepcopy(exact['operations'][0])]
    changed['operations'][0]['target'] = generated['operations'][3]['target']
    negative('NBSP is not normalized to ordinary space', changed, 'ANCHOR_NOT_FOUND')
    changed = deepcopy(exact); changed['operations'].append(deepcopy(exact['operations'][0]))
    negative('true overlapping edits', changed, 'OVERLAPPING_OPERATIONS')
    changed = deepcopy(exact); changed['scope'] = {'kind': 'selected', 'targets': [generated['operations'][0]['target']]}
    negative('direct target outside selected scope', changed, 'TARGET_OUT_OF_SCOPE')
    changed = deepcopy(generated); changed['source_context'] = [{'kind': 'paragraph', 'target': generated['operations'][0]['target'],
        'text': 'Modified read-only text', 'reason_codes': ['CONTEXT_ONLY']}]
    negative('modified original-source observation', changed, 'READ_ONLY_CONTEXT_CHANGED')

    # Exact whitespace and soft-break requests remain meaningful in this text model.
    changed = deepcopy(generated); changed['operations'][2]['text'] = 'One space.\tLine\nbreak.'
    assert fixture_oracle(changed, source, digest)['text'][2] == 'One space.\tLine\nbreak.'
    semantic_cases.append('exact whitespace request retained')
    # Remove unchanged scaffolds before body conflicts, but retain their explicit comment declarations.
    changed = deepcopy(plans['template.composed.json'])
    changed['operations'][0]['attached_comment'] = {'text': 'Discuss the revised wording.'}
    assert fixture_oracle(changed, source, digest)['declared_attached_comments'] == 1
    semantic_cases.append('unchanged rewrite retains attached comment declaration')
    changed = deepcopy(exact); changed['operations'] = [deepcopy(exact['operations'][0])]
    changed['operations'][0]['text'] = changed['operations'][0]['find']
    changed['operations'][0]['attached_comment'] = {'text': 'Confirm this period.'}
    result = fixture_oracle(changed, source, digest)
    assert result['text'] == source and result['active_body_requests'] == 0 and result['declared_attached_comments'] == 1
    semantic_cases.append('unchanged replacement retains attached comment declaration')
    changed = deepcopy(generated); changed['operations'] = []
    assert fixture_oracle(changed, source, digest)['text'] == source
    semantic_cases.append('empty plan is valid unchanged request')
    changed = deepcopy(generated); changed['operations'] = [
        {'kind': 'insert', 'target': generated['operations'][0]['target'], 'edge': 'start', 'text': 'A'},
        {'kind': 'insert', 'target': generated['operations'][0]['target'], 'edge': 'start', 'text': 'B'}]
    validator.validate(changed)
    assert fixture_oracle(changed, source, digest)['text'][0] == 'AB' + source[0]
    semantic_cases.append('same-position insertion order')
    changed = deepcopy(generated); changed['operations'] = [
        {'kind': 'delete_paragraph', 'target': generated['operations'][0]['target']}]
    missing = fixture_oracle(changed, source, digest)
    expect_error(lambda: expectation_oracle(spec, source, digest, missing), 'EXPECTATION_TARGET_UNMAPPABLE')
    semantic_cases.append('missing expectation target is not passing absence')
    expect_error(lambda: expectation_oracle(spec, source, digest, fixture_oracle(generated, source, digest)), 'EXPECTATION_FAILED')
    semantic_cases.append('failed expected text is visible')

    invalid_cases = []
    def schema_negative(name, change, base=generated):
        value = deepcopy(base); change(value)
        assert not validator.is_valid(value), name
        invalid_cases.append(name)
    schema_negative('run_operation rejected', lambda p: p['operations'][0].update(run_operation=False))
    schema_negative('missing before hash rejected', lambda p: p['operations'][0].pop('before_sha256'))
    schema_negative('missing selected-decision field rejected', lambda p: p.pop('resolve_revisions'))
    schema_negative('unknown root field rejected', lambda p: p.update(ready=True))
    schema_negative('third identity field rejected', lambda p: p['operations'][0]['target'].update(pid='1234'))
    schema_negative('zero occurrence rejected', lambda p: p['operations'][1].update(occurrence=0), base=exact)
    schema_negative('empty attached anchor rejected', lambda p: p['operations'][0].update(text='', attached_comment={'text': 'X'}))
    schema_negative('timezone-free date rejected', lambda p: p.update(date='2026-10-17T07:00:00'))
    schema_negative('invalid calendar date rejected', lambda p: p.update(date='2026-02-30T07:00:00Z'))
    schema_negative('source hash with trailing newline rejected', lambda p: p['source'].update(sha256=digest+'\n'))
    schema_negative('before hash with trailing newline rejected', lambda p: p['operations'][0].update(before_sha256=text_hash(source[0])+'\n'))
    schema_negative('whitespace author rejected', lambda p: p.update(author='  '))
    insert = deepcopy(generated); insert['operations'] = [
        {'kind': 'insert', 'target': generated['operations'][0]['target'], 'text': 'X', 'edge': 'start'}]
    schema_negative('two insertion positions rejected', lambda p: p['operations'][0].update(before_text='Payment'), base=insert)
    schema_negative('occurrence on edge rejected', lambda p: p['operations'][0].update(occurrence=1), base=insert)
    comment = deepcopy(generated); comment['operations'] = [
        {'kind': 'comment', 'target': generated['operations'][0]['target'], 'text': 'X'}]
    validator.validate(comment)  # Shape only, not comment XML behavior.
    schema_negative('comment occurrence without find rejected', lambda p: p['operations'][0].update(occurrence=1), base=comment)
    schema_negative('comment find plus through rejected', lambda p: p['operations'][0].update(find='Payment', through=generated['operations'][1]['target']), base=comment)
    bad_expect = deepcopy(spec); bad_expect['requirements'][0]['checks'][0]['kind'] = 'revision_preserved'
    assert not expectation_validator.is_valid(bad_expect)
    invalid_cases.append('unimplemented expectation predicate rejected')
    bad_expect = deepcopy(spec); bad_expect['source']['sha256'] += '\n'
    assert not expectation_validator.is_valid(bad_expect)
    invalid_cases.append('expectation hash with trailing newline rejected')
    bad_expect = deepcopy(spec); bad_expect['requirements'][0]['key'] += '\n'
    assert not expectation_validator.is_valid(bad_expect)
    invalid_cases.append('requirement key with trailing newline rejected')
    assert hashlib.sha256(source_path.read_bytes()).hexdigest() == digest
    print(json.dumps({'status': 'passed',
        'scope': 'Design-schema validation and in-memory four-paragraph text model ONLY; no proposed CLI execution.',
        'source_sha256': digest, 'schema_meta_validation': ['edit-plan-v2-core', 'expectations-v1'],
        'examples': examples, 'semantic_model_cases': semantic_cases,
        'schema_negative_cases': invalid_cases,
        'limitations': ['No actual DOCX mutation, revision settlement, tracking, comment-range construction, '
                       'package preservation, publication failure handling, rendering or runtime conformance was tested. '
                       'Attached comments are counted as declarations only. Working-basis drift is a string-model test.']},
        ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
