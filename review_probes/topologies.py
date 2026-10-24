from probe import *
from fuzz import para, check, rng

def nested():return tbl(tr(tc(p('Inner fixed.')),tc(p('Inner.'))),cols=2)
for n in range(60):
    def blocks():
        paras=[para(rng.choice(['One.','Two.','Three.','Alpha beta.','Alpha gamma.']),rng.randrange(16)) for _ in range(rng.randrange(1,5))]
        at=rng.randrange(len(paras))
        return paras[:at]+[nested()]+paras[at:]
    aa=blocks();bb=blocks()
    for mode in ['', 'powertools']:check(200+n,aa,bb,mode)
for n in range(60):
    def blocks():
        return [sdt(*[para(rng.choice(['One.','Two.','Three.','Alpha beta.','Alpha gamma.']),rng.randrange(16)) for _ in range(rng.randrange(1,4))]) for _ in range(2)]
    aa=blocks();bb=blocks()
    for mode in ['', 'powertools']:check(300+n,aa,bb,mode)
