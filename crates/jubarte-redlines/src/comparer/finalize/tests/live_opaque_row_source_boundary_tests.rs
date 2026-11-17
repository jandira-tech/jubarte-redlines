// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

use super::*;

// Nontext source owners are real content, even when their cached result
// is empty. No row lifetime can be inferred from its revised peer cell.
#[test]
fn unchanged_opaque_cell_owners_prevent_row_lifetime_synthesis_without_text_caches() {
    let payloads = [
        (
            "object",
            "<w:r><w:object w:dxaOrig='240' w:dyaOrig='240'><v:shape id='OwnedObject' style='width:12pt;height:12pt'/></w:object></w:r>",
        ),
        (
            "picture",
            "<w:r><w:pict><v:rect id='OwnedRectangle' style='width:12pt;height:12pt' fillcolor='#123456'/></w:pict></w:r>",
        ),
        (
            "alternate",
            "<mc:AlternateContent><mc:Choice Requires='v'><w:r><w:pict><v:rect id='OwnedChoice' style='width:12pt;height:12pt'/></w:pict></w:r></mc:Choice><mc:Fallback><w:r><w:pict><v:rect id='OwnedFallback' style='width:12pt;height:12pt'/></w:pict></w:r></mc:Fallback></mc:AlternateContent>",
        ),
        (
            "uncached-field",
            "<w:fldSimple w:instr=' PAGE ' w:dirty='1'/>",
        ),
    ];
    for (kind, payload) in payloads {
        for direction in ["ins", "del"] {
            for history in [false, true] {
                for control in [false, true] {
                    let old = if history {
                        "<w:trPrChange w:id='10' w:author='Prior row editor' w:date='2025-01-01T00:00:00Z'><w:trPr><w:trHeight w:val='220' w:hRule='atLeast'/></w:trPr></w:trPrChange>"
                    } else {
                        ""
                    };
                    let text = if direction == "ins" { "t" } else { "delText" };
                    let table = format!(
                        "<w:tbl><w:tblPr><w:tblW w:w='2400' w:type='dxa'/></w:tblPr><w:tblGrid><w:gridCol w:w='1200'/><w:gridCol w:w='1200'/></w:tblGrid><w:tr><w:trPr><w:trHeight w:val='260' w:hRule='atLeast'/>{old}</w:trPr><w:tc><w:tcPr><w:tcW w:w='1200' w:type='dxa'/></w:tcPr><w:p><w:pPr><w:pStyle w:val='BodyText'/><w:spacing w:before='120' w:after='80'/><w:jc w:val='center'/></w:pPr>{payload}</w:p></w:tc><w:tc><w:tcPr><w:tcW w:w='1200' w:type='dxa'/></w:tcPr><w:p><w:pPr><w:rPr><w:{direction} w:id='11' w:author='Source change owner' w:date='2026-01-01T00:00:00Z'/></w:rPr></w:pPr><w:{direction} w:id='12' w:author='Source change owner' w:date='2026-01-01T00:00:00Z'><w:r><w:rPr><w:b/><w:color w:val='654321'/></w:rPr><w:{text}>Independently revised peer cell</w:{text}></w:r></w:{direction}></w:p></w:tc></w:tr></w:tbl>"
                    );
                    let body = if control {
                        format!(
                            "<w:sdt><w:sdtPr><w:tag w:val='Row source control'/><w:id w:val='42'/><w:richText/></w:sdtPr><w:sdtContent>{table}</w:sdtContent></w:sdt>"
                        )
                    } else {
                        table
                    };
                    let mut dom = Dom::new();
                    let doc=dom.parse_xdocument(&format!("<w:document xmlns:w='{}' xmlns:v='urn:schemas-microsoft-com:vml' xmlns:mc='http://schemas.openxmlformats.org/markup-compatibility/2006'><w:body>{body}<w:p/><w:sectPr><w:pgSz w:w='12240' w:h='15840'/></w:sectPr></w:body></w:document>",W::URI));
                    let root = dom.root(doc).expect("valid tracked row source");
                    let before = dom.serialize_element(root);
                    let nodes = dom.node_count();
                    let mut next_id = 100;
                    mark_fully_revised_rows(
                        &mut dom,
                        root,
                        &WmlComparerSettings::default(),
                        &mut next_id,
                    );
                    assert_eq!(
                        dom.serialize_element(root),
                        before,
                        "{kind}/{direction}/history={history}/control={control}: complete source owners and property history"
                    );
                    assert_eq!(dom.node_count(), nodes);
                    assert_eq!(next_id, 100, "no synthesized history or lifetime");
                    let row = dom.descendants(root, Some(&W::tr()))[0];
                    let row_properties = dom
                        .element(row, &W::tr_pr())
                        .expect("authored row properties");
                    assert!(dom.element(row_properties, &W::ins()).is_none());
                    assert!(dom.element(row_properties, &W::del()).is_none());
                    mark_fully_revised_rows(
                        &mut dom,
                        root,
                        &WmlComparerSettings::default(),
                        &mut next_id,
                    );
                    assert_eq!(dom.serialize_element(root), before);
                    assert_eq!(dom.node_count(), nodes);
                    assert_eq!(next_id, 100);
                }
            }
        }
    }
}
