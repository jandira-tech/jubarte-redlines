//! Regression — accepting revisions must not panic when a block-level content
//! control's runs were ALL deleted revisions.
//!
//! Upstream C# (`RevisionProcessor.AddBlockLevelContentControls`,
//! RevisionProcessor.cs:1985) looked each annotated run up in the transformed
//! document with `.First()`; when the whole `w:sdt` was a deleted revision
//! (e.g. a Table of Contents deleted by WmlComparer) the runs no longer exist
//! after acceptance and it threw `InvalidOperationException`. The port
//! faithfully reproduced that as an `.expect()` panic. Trigger shape (reduced
//! from `ole-object_ooxml-style-link` Word-compare redline): a trailing sdt
//! whose only paragraph has a deleted paragraph mark and fully-deleted runs.

use jubarte::namespaces::W;
use jubarte::revision_processor::{accept_revisions_document, element_has_tracked_revisions};
use jubarte::xmllinq::Dom;

const FULLY_DELETED_TRAILING_SDT: &str = concat!(
    r#"<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">"#,
    r#"<w:body>"#,
    r#"<w:p><w:r><w:t>Hello survivor</w:t></w:r></w:p>"#,
    r#"<w:sdt><w:sdtPr><w:id w:val="787555665"/>"#,
    r#"<w:docPartObj><w:docPartGallery w:val="Table of Contents"/><w:docPartUnique/></w:docPartObj>"#,
    r#"</w:sdtPr><w:sdtContent>"#,
    r#"<w:p><w:pPr><w:rPr><w:del w:id="1" w:author="a" w:date="2026-01-01T00:00:00Z"/></w:rPr></w:pPr>"#,
    r#"<w:del w:id="2" w:author="a" w:date="2026-01-01T00:00:00Z">"#,
    r#"<w:r><w:delText>Contents</w:delText></w:r></w:del>"#,
    r#"</w:p>"#,
    r#"</w:sdtContent></w:sdt>"#,
    r#"</w:body></w:document>"#,
);

#[test]
fn accept_survives_fully_deleted_trailing_content_control() {
    let mut dom = Dom::new();
    let doc = dom.parse_xdocument(FULLY_DELETED_TRAILING_SDT);
    let root = dom.root(doc).unwrap();

    // Must not panic (the regression), and must fully resolve the revisions.
    let accepted = accept_revisions_document(&mut dom, root);

    assert!(
        !element_has_tracked_revisions(&dom, accepted),
        "accepted document still carries tracked revisions"
    );
    assert!(
        dom.value(accepted).contains("Hello survivor"),
        "untouched paragraph text must survive acceptance"
    );
    assert!(
        dom.descendants(accepted, Some(&W::name("sdt"))).is_empty(),
        "the fully-deleted content control must be dropped, not re-wrapped"
    );
    assert!(
        !dom.value(accepted).contains("Contents"),
        "deleted content must be gone after acceptance"
    );
}
