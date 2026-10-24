import sys, subprocess, json, zipfile, pathlib, xml.etree.ElementTree as E
from mkdocx import *
ROOT=pathlib.Path(__file__).resolve().parent
BIN='/Users/arthrod/temp/T/fix013w/bin/jubarte-D'
NSM={'w':W}
def run(*args):
    r=subprocess.run([BIN,*map(str,args)],capture_output=True,text=True)
    if r.returncode: raise RuntimeError((args,r.returncode,r.stderr[-1500:]))
    return r.stdout
def xml(path,part='word/document.xml'):
    with zipfile.ZipFile(path) as z:return E.fromstring(z.read(part))
def local(e):return e.tag.split('}')[-1]
def canon(e):
    if e is None:return None
    kids=tuple(canon(c) for c in e)
    return (local(e),tuple(sorted((k.split('}')[-1],v) for k,v in e.attrib.items() if 'rsid' not in k)),e.text.strip() if e.text and e.text.strip() else '',kids)
def props(p):
    pr=p.find('w:pPr',NSM)
    if pr is None:return ()
    return tuple(canon(c) for c in pr if not (local(c)=='rPr' and len(c)==0))
def snap(path,part='word/document.xml'):
    return [(''.join(p.itertext()),props(p),[(r.find('w:t',NSM).text,canon(r.find('w:rPr',NSM))) for r in p.findall('w:r',NSM) if r.find('w:t',NSM) is not None]) for p in xml(path,part).iter('{'+W+'}p')]
def semantic(path,part='word/document.xml'):
    out=[]
    for text,pr,runs in snap(path,part):
        pr=tuple(x for x in pr if x != ('jc',(('val','left'),),'',()))
        chars=[]
        for t,rpr in runs:
            if rpr==('rPr',(),' ',()) or rpr==('rPr',(),' ',()):rpr=None
            if rpr==('rPr',(),'',()):rpr=None
            chars.extend((c,rpr) for c in t or '')
        out.append((text,pr,chars))
    return out
def body(blocks):return p('Before.')+tbl(tr(tc(p('Fixed.')),tc(*blocks)),cols=2)+p('After.')
def pair(tag,aa,bb,mode='',**kw):
    d=ROOT/tag;d.mkdir(exist_ok=True)
    a=d/'a.docx';b=d/'b.docx';o=d/'out.docx'
    write(a,body(aa),**kw);write(b,body(bb),**kw)
    run('compare',a,b,'-o',o,'--force',*(['--mode',mode] if mode else []))
    v=run('validate',o,'--json')
    (d/'validate.jsonl').write_text(v)
    for op,wanted in [('accept',b),('reject',a)]:
        out=d/(op+'.docx');run(op,o,'-o',out,'--force')
        got=snap(out);want=snap(wanted)
        if got!=want:
            print(tag,op,json.dumps({'got':got,'want':want}),flush=True)
    return d
def pp(text,jc='',bold=False):
    pr=('<w:jc w:val="'+jc+'"/>' if jc else '')+('<w:rPr><w:b/></w:rPr>' if bold else '')
    return p(text,'<w:pPr>'+pr+'</w:pPr>' if pr else '')
if __name__=='__main__':
    cases=[
      ([pp('Alpha.','left')],[pp('Alpha.','left'),pp('Beta.','right',True)]),
      ([pp('Alpha.','left'),pp('Beta.','right',True)],[pp('Alpha.','left')]),
      ([pp('Alpha.','left')],[pp('Alpha.','center'),pp('Beta.','right',True)]),
      ([pp('Alpha.','center'),pp('Beta.','right',True)],[pp('Alpha.','left')]),
      ([pp('Alpha.','left',True),pp('Beta.','right')],[pp('Gamma.','center')]),
      ([pp('Gamma.','center')],[pp('Alpha.','left',True),pp('Beta.','right')]),
      ([pp('Alpha.','left',True),pp('Beta.','right')],[pp('Alpha.','center'),pp('Gamma.','right',True)]),
    ]
    for i,(a,b) in enumerate(cases):
        for mode in ['', 'powertools']:
            pair('fmt'+str(i)+('-pt' if mode else ''),a,b,mode)
