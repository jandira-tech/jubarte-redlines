# SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC

# SPDX-License-Identifier: AGPL-3.0-only

"""Reproduce release diagnostics in a new directory using a supplied binary.

Usage: python probe_release.py --binary /path/to/jubarte --out-dir /new/probe-dir
No repository files are modified. Do not point this script at a production document.
The binary is not downloaded or installed by this script.
"""
from pathlib import Path
import argparse
import hashlib
import json
import subprocess
import tempfile
import zipfile
from xml.sax.saxutils import escape

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--binary', type=Path, required=True, help='Native Jubarte 0.12.0 executable')
parser.add_argument('--out-dir', type=Path, required=True, help='A new directory for probe results')
args = parser.parse_args()
BIN = args.binary.resolve(strict=True)
version = subprocess.run([str(BIN), '--version'], text=True, capture_output=True, timeout=30)
if version.returncode != 0 or '0.12.0' not in version.stdout:
    parser.error('The supplied executable must report Jubarte 0.12.0; record build/runtime provenance separately.')
ROOT = args.out_dir.resolve()
ROOT.mkdir(parents=True, exist_ok=False)
RUN = Path(tempfile.mkdtemp(prefix='release-probes-', dir=ROOT))
paragraphs = [
    'Payment is due within 30 days.',
    'The fee is due; the fee is payable.',
    'One  space.',
    'Notice is due within 30\u00a0days.',
]
source = RUN / 'source.docx'
parts = {
    '[Content_Types].xml': '<?xml version="1.0" encoding="UTF-8"?><Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types"><Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/><Default Extension="xml" ContentType="application/xml"/><Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/></Types>',
    '_rels/.rels': '<?xml version="1.0" encoding="UTF-8"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/></Relationships>',
    'word/document.xml': '<?xml version="1.0" encoding="UTF-8"?><w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"><w:body>' + ''.join('<w:p><w:r><w:t xml:space="preserve">'+escape(p)+'</w:t></w:r></w:p>' for p in paragraphs) + '<w:sectPr><w:pgSz w:w="12240" w:h="15840"/><w:pgMar w:top="1440" w:right="1440" w:bottom="1440" w:left="1440"/></w:sectPr></w:body></w:document>',
}
with zipfile.ZipFile(source, 'w', compression=zipfile.ZIP_STORED) as z:
    for name, text in parts.items():
        entry = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
        z.writestr(entry, text.encode())
digest = hashlib.sha256(source.read_bytes()).hexdigest()
base = {'schema_version': 1, 'source_sha256': digest, 'author': 'Audit', 'date': '2026-10-17T07:00:00Z', 'operations': []}
(RUN/'empty-plan.json').write_text(json.dumps(base))
disabled = dict(base, operations=[{'kind':'rewrite','paragraph':'p0','text':paragraphs[0],'run_operation':False}])
(RUN/'disabled-plan.json').write_text(json.dumps(disabled))
for name, args in [('root-help',['--help']),('edit-help',['edit','--help']),('read-help',['read','--help']),('capabilities',['capabilities','--json'])]:
    p = subprocess.run([str(BIN),*args],cwd=RUN,text=True,capture_output=True,timeout=30)
    (ROOT/f'release-{name}.txt').write_text(p.stdout+p.stderr)
cases = [
 ('read-agent',['read','source.docx','--no-page-markers']),
 ('read-json',['read','source.docx','--json','--no-page-markers']),
 ('empty-changes-json',['changes','source.docx','--json']),
 ('edit-json',['edit','source.docx','-p','p0','--anchor','30 days','--content','45 days','--json','--dry-run']),
 ('edit-exact-dry',['edit','source.docx','-p','p0','--anchor','30 days','--content','45 days','--dry-run']),
 ('missing-anchor',['edit','source.docx','-p','p0','--anchor','31 days','--content','45 days','--dry-run']),
 ('ambiguous-anchor',['edit','source.docx','-p','p1','--anchor','fee','--content','charge','--dry-run']),
 ('implicit-rewrite',['edit','source.docx','-p','p0','--content','45 days','--dry-run']),
 ('whitespace-only-rewrite',['edit','source.docx','-p','p2','--content','One space.','--dry-run']),
 ('normalized-anchor',['edit','source.docx','-p','p3','--anchor','30 days','--content','45 days','--dry-run']),
 ('unknown-enabled-key',['edit','source.docx','--plan','disabled-plan.json','--dry-run']),
 ('empty-plan-dry',['edit','source.docx','--plan','empty-plan.json','--dry-run']),
 ('plan-plus-flags',['edit','source.docx','--plan','empty-plan.json','-p','p0','--content','45 days','--dry-run']),
 ('default-write',['edit','source.docx','-p','p0','--anchor','30 days','--content','45 days']),
]
results=[]
for name,args in cases:
    before={str(p.relative_to(RUN)):hashlib.sha256(p.read_bytes()).hexdigest() for p in RUN.rglob('*') if p.is_file()}
    p=subprocess.run([str(BIN),*args],cwd=RUN,text=True,capture_output=True,timeout=30)
    after={str(p.relative_to(RUN)):hashlib.sha256(p.read_bytes()).hexdigest() for p in RUN.rglob('*') if p.is_file()}
    try:
        value=json.loads(p.stdout)
        shape=type(value).__name__
    except Exception:
        shape='empty' if not p.stdout else 'not-one-JSON-value'
    row={'case':name,'argv':['jubarte',*args],'exit_code':p.returncode,'stdout_shape':shape,'stdout':p.stdout,'stderr':p.stderr,'created':[x for x in after if x not in before],'modified':[x for x in before if after.get(x)!=before[x]]}
    results.append(row)
    print(json.dumps({k:row[k] for k in ['case','exit_code','stdout_shape','created','modified']}))
assert hashlib.sha256(source.read_bytes()).hexdigest()==digest
(ROOT/'release-probes.json').write_text(json.dumps({'binary_version':'0.12.0','release_tag':'v0.12.0','binary_sha256':hashlib.sha256(BIN.read_bytes()).hexdigest(),'source_sha256':digest,'warning':'Caller-supplied binary; version 0.12.0 alone does not prove the same release build as the original audit.','cases':results},indent=2,ensure_ascii=False)+'\n')
