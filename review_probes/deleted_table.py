"""A wholly deleted nested table makes settling lose a revised paragraph break."""
from probe import *
if __name__=='__main__':
    d=ROOT/'deleted-table';d.mkdir(exist_ok=True)
    nested=tbl(tr(tc(p('Inner fixed.')),tc(p('Inner.'))),cols=2)
    aa=[p('Alpha beta gamma delta.'),nested,p('Two.')]
    bb=[p('Epsilon zeta eta theta.'),p('One.')]
    for side,blocks in [('a',aa),('b',bb)]:write(d/(side+'.docx'),body(blocks))
    for mode in ['', 'powertools']:
        suffix='-pt' if mode else ''
        out=d/('out'+suffix+'.docx')
        run('compare',d/'a.docx',d/'b.docx','-o',out,'--force','--datetime','2026-10-23T00:00:00Z',*(['--mode',mode] if mode else []))
        for op in ['accept','reject']:run(op,out,'-o',d/(op+suffix+'.docx'),'--force')
    for fn in ['a.docx','b.docx','out.docx','accept.docx','reject.docx','accept-pt.docx']:
        result=subprocess.run([BIN,'validate',str(d/fn),'--json'],capture_output=True,text=True)
        print(fn,'validate',result.returncode,result.stdout.strip())
        if fn!='out.docx':print(fn,[t for t,pr,r in snap(d/fn)])
