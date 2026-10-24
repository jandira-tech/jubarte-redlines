"""Reproduce an unsettled final mark in a relationship-named story."""
from probe import *

def rename_part(path,old,new):
    with zipfile.ZipFile(path) as z: parts={n:z.read(n) for n in z.namelist()}
    parts[new]=parts.pop(old)
    parts['[Content_Types].xml']=parts['[Content_Types].xml'].replace(('/'+old).encode(),('/'+new).encode())
    parts['word/_rels/document.xml.rels']=parts['word/_rels/document.xml.rels'].replace(old.removeprefix('word/').encode(),new.removeprefix('word/').encode())
    with zipfile.ZipFile(path,'w',zipfile.ZIP_DEFLATED) as z:
        for n,x in parts.items():z.writestr(n,x)

if __name__=='__main__':
    for story,old,new in [('header','word/header1.xml','word/chrome.xml'),('footer','word/footer1.xml','word/bottom.xml'),('footnotes','word/footnotes.xml','word/notes.xml'),('endnotes','word/endnotes.xml','word/tailnotes.xml')]:
        for renamed in [False,True]:
            d=ROOT/('story-'+story+('-renamed' if renamed else '-standard'));d.mkdir(exist_ok=True)
            for side,blocks in [('a',[p('Old.')]),('b',[p('Old.'),p('New.')])]:
                text=p('Body.')
                if story in ['footnotes','endnotes']:
                    tag='footnote' if story=='footnotes' else 'endnote'
                    text='<w:p><w:r><w:t>Body.</w:t></w:r><w:r><w:'+tag+'Reference w:id="1"/></w:r></w:p>'
                write(d/(side+'.docx'),text,**{story:body(blocks)})
                if renamed:rename_part(d/(side+'.docx'),old,new)
            for mode in ['', 'powertools']:
                out=d/('out'+('-pt' if mode else '')+'.docx')
                run('compare',d/'a.docx',d/'b.docx','-o',out,'--force',*(['--mode',mode] if mode else []))
                v=subprocess.run([BIN,'validate',str(out),'--json'],capture_output=True,text=True)
                out.with_suffix('.jsonl').write_text(v.stdout)
                print(story,renamed,mode,'exit',v.returncode,v.stdout,flush=True)
