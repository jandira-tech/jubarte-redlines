from probe import *
def comment(path,id):
    with zipfile.ZipFile(path) as z:parts={n:z.read(n) for n in z.namelist()}
    parts['[Content_Types].xml']=parts['[Content_Types].xml'].replace(b'</Types>',f'<Override PartName="/word/comments.xml" ContentType="{CT}comments+xml"/></Types>'.encode())
    parts['word/_rels/document.xml.rels']=parts['word/_rels/document.xml.rels'].replace(b'</Relationships>',f'<Relationship Id="rIdC" Type="{REL}comments" Target="comments.xml"/></Relationships>'.encode())
    parts['word/comments.xml']=f'<w:comments {NS}><w:comment w:id="{id}" w:author="Ann" w:date="2026-10-23T00:00:00Z">{p("Comment.")}</w:comment></w:comments>'.encode()
    with zipfile.ZipFile(path,'w',zipfile.ZIP_DEFLATED) as z:
        for n,x in parts.items():z.writestr(n,x)
if __name__=='__main__':
    for id in [2147483647]:
        d=ROOT/('id-comment-'+str(id));d.mkdir(exist_ok=True)
        anchor=f'<w:p><w:commentRangeStart w:id="{id}"/><w:r><w:t>Anchor.</w:t></w:r><w:commentRangeEnd w:id="{id}"/><w:r><w:commentReference w:id="{id}"/></w:r></w:p>'
        for side in ['a','b']:
            cells=[tc(p('Fixed.'))]
            for i in range(2):cells.append(tc(pp('Old'+str(i)+'.','center',True),*([pp('New'+str(i)+'.','right')] if side=='b' else [])))
            write(d/(side+'.docx'),anchor+tbl(tr(*cells),cols=3)+p('After.'));comment(d/(side+'.docx'),id)
        run('compare',d/'a.docx',d/'b.docx','-o',d/'out.docx','--force','--datetime','2026-10-23T00:00:00Z')
        for fn in ['a.docx','b.docx','out.docx']:
            v=subprocess.run([BIN,'validate',str(d/fn),'--json'],capture_output=True,text=True)
            print(id,fn,v.returncode,v.stdout,flush=True)
        print(run('changes',d/'out.docx','--json'),flush=True)
