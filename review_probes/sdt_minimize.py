from probe import *
import itertools

def trial(tag,aa,bb):
    d=ROOT/tag;d.mkdir(exist_ok=True)
    for side,blocks in [('a',aa),('b',bb)]:write(d/(side+'.docx'),body([sdt(*map(p,x)) for x in blocks]))
    run('compare',d/'a.docx',d/'b.docx','-o',d/'out.docx','--force')
    run('accept',d/'out.docx','-o',d/'accept.docx','--force')
    want=[t for t,pr,r in snap(d/'b.docx')];got=[t for t,pr,r in snap(d/'accept.docx')]
    if want!=got:print(tag,json.dumps(aa),json.dumps(bb),json.dumps(got),flush=True)
    return want!=got

if __name__=='__main__':
    for n,(a,b) in enumerate([
        ([['Alpha gamma.'],['Two.']],[['Alpha beta.','Two.','Alpha gamma.'],['Three.']]),
        ([['Alpha gamma.','Alpha gamma.'],['Two.']],[['Two.','Alpha gamma.'],['Three.']]),
        ([['Alpha gamma.','Alpha gamma.'],['Two.']],[['Alpha beta.','Alpha gamma.'],['Three.']]),
        ([['Alpha gamma.','Alpha gamma.'],['Two.']],[['Alpha beta.','Two.'],['Three.']]),
        ([['Alpha gamma.','Alpha gamma.'],['Two.']],[['Alpha gamma.'],['Three.']]),
        ([['Alpha gamma.','Alpha gamma.','Two.']],[['Alpha beta.','Two.','Alpha gamma.','Three.']]),
        ([['Alpha gamma.','Two.']],[['Alpha beta.','Alpha gamma.','Three.']]),
        ([['Alpha gamma.','Two.']],[['Alpha gamma.','Three.']]),
    ]):trial('sdt-delta'+str(n),a,b)
