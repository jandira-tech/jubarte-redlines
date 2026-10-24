"""Both inputs retain two controls; Accept All adds an extra paragraph."""
from probe import *
if __name__=='__main__':
    d=ROOT/'sdt-boundary';d.mkdir(exist_ok=True)
    aa=[sdt(p('Alpha gamma.'),p('Alpha gamma.')),sdt(p('Two.'))]
    bb=[sdt(p('Alpha gamma.')),sdt(p('Three.'))]
    for side,blocks in [('a',aa),('b',bb)]:write(d/(side+'.docx'),body(blocks))
    run('compare',d/'a.docx',d/'b.docx','-o',d/'out.docx','--force','--datetime','2026-10-23T00:00:00Z')
    for op in ['accept','reject']:run(op,d/'out.docx','-o',d/(op+'.docx'),'--force')
    for fn in ['a.docx','b.docx','out.docx','accept.docx','reject.docx']:
        result=subprocess.run([BIN,'validate',str(d/fn),'--json'],capture_output=True,text=True)
        print(fn,'validate',result.returncode,result.stdout.strip())
        if fn!='out.docx':print(fn,[t for t,pr,r in snap(d/fn)])
