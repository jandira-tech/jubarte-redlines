from probe import *
import random
rng=random.Random(1680)
BASE='/Users/arthrod/temp/T/fix013w/bin/jubarte-base'
def para(t,fmt):
    jc=['center','right','both','left'][fmt%4]
    mark=['','<w:b/>','<w:i/>','<w:sz w:val="32"/>'][fmt//4%4]
    return p(t,f'<w:pPr><w:jc w:val="{jc}"/>'+('<w:rPr>'+mark+'</w:rPr>' if mark else '')+'</w:pPr>')
def check(n,aa,bb,mode):
    tag=f'fuzz{n:03}'+('-pt' if mode else '')
    d=ROOT/tag;d.mkdir(exist_ok=True)
    a=d/'a.docx';b=d/'b.docx';o=d/'out.docx'
    write(a,body(aa));write(b,body(bb))
    run('compare',a,b,'-o',o,'--force',*(['--mode',mode] if mode else []))
    failures=[]
    for op,wanted in [('accept',b),('reject',a)]:
        dst=d/(op+'.docx');run(op,o,'-o',dst,'--force')
        if semantic(dst)!=semantic(wanted):failures.append(op)
    if failures:
        old=d/'base.docx'
        subprocess.run([BASE,'compare',str(a),str(b),'-o',str(old),'--force',*(['--mode',mode] if mode else [])],capture_output=True,check=True)
        new=[]
        for op in failures:
            dst=d/('base-'+op+'.docx');run(op,old,'-o',dst,'--force')
            wanted=b if op=='accept' else a
            if semantic(dst)!=semantic(d/(op+'.docx')):
                new.append({'op':op,'base_ok':semantic(dst)==semantic(wanted),'base':semantic(dst)[2:-1],'new':semantic(d/(op+'.docx'))[2:-1],'want':semantic(wanted)[2:-1]})
        if new:print(tag,json.dumps(new),flush=True)
    if n%10==0:print('progress',n,mode,flush=True)
if __name__=='__main__':
    for n in range(180):
        aa=[para(rng.choice(['One.','Two.','Three.','Four.','Alpha beta.','Alpha gamma.']),rng.randrange(16)) for _ in range(rng.randrange(1,5))]
        bb=[para(rng.choice(['One.','Two.','Three.','Four.','Alpha beta.','Alpha gamma.']),rng.randrange(16)) for _ in range(rng.randrange(1,5))]
        for mode in ['', 'powertools']:
            check(n,aa,bb,mode)
