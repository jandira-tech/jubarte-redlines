from probe import *
from fuzz import check,para,rng
for n in range(80):
    def blocks(insert):
        words=['One.','Two.','Three.','Alpha beta gamma delta.','Epsilon zeta eta theta.']
        paras=[para(rng.choice(words),rng.randrange(16)) for _ in range(rng.randrange(1,5))]
        if not insert:return paras
        at=rng.randrange(len(paras))
        return paras[:at]+[tbl(tr(tc(p('Inner fixed.')),tc(p('Inner.'))),cols=2)]+paras[at:]
    aa=blocks(n%2==0);bb=blocks(n%2!=0)
    for mode in ['', 'powertools']:check(500+n,aa,bb,mode)
