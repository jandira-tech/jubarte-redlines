from probe import *
STAMP='w:author="Ann" w:date="2026-10-23T00:00:00Z"'
def mark(kind,id):return f'<w:{kind} w:id="{id}" {STAMP}/>'
def moved(kind,mid,tid,rid,text):
    t='t'  # Word uses w:t for moved-from as well as moved-to runs.
    return (f'<w:p><w:pPr><w:rPr>{mark(kind,mid)}</w:rPr></w:pPr>'
        f'<w:{kind}RangeStart w:id="{rid}" w:name="MoveA" {STAMP}/>'
        f'<w:{kind} w:id="{tid}" {STAMP}><w:r><w:{t}>{text}</w:{t}></w:r></w:{kind}>'
        f'<w:{kind}RangeEnd w:id="{rid}"/></w:p>')
if __name__=='__main__':
    d=ROOT/'selective-move';d.mkdir(exist_ok=True)
    inserted=f'<w:p><w:pPr><w:rPr>{mark("ins",1)}</w:rPr></w:pPr><w:ins w:id="2" {STAMP}><w:r><w:t>Inserted.</w:t></w:r></w:ins></w:p>'
    doc=moved('moveFrom',3,4,5,'Moved paragraph.')+tbl(tr(tc(p('Fixed.')),tc(inserted,moved('moveTo',7,8,6,'Moved paragraph.'))),cols=2)+p('After.')
    source=d/'input.docx';write(source,doc)
    for args in [('validate',source,'--json'),('changes',source,'--json')]:
        r=subprocess.run([BIN,*map(str,args)],capture_output=True,text=True);print(args,r.returncode,r.stdout,r.stderr)
    for op in ['accept','reject']:
        out=d/(op+'.docx');run(op,source,'--id','r8','-o',out,'--force')
        r=subprocess.run([BIN,'validate',str(out),'--json'],capture_output=True,text=True)
        print(op,'validate',r.returncode,r.stdout)
        print(E.tostring(xml(out),encoding='unicode'))
