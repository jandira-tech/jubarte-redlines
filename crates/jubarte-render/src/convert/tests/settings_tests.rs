// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: Apache-2.0

use super::*;

const W_NS: &str = "http://schemas.openxmlformats.org/wordprocessingml/2006/main";

fn settings(body: &str) -> String {
    format!(r#"<w:settings xmlns:w="{W_NS}">{body}</w:settings>"#)
}

#[test]
fn on_off_flags_honour_every_false_spelling_and_quote_style() {
    for local in [
        "trackRevisions",
        "suppressSpBfAfterPgBrk",
        "evenAndOddHeaders",
        "mirrorMargins",
        "displayBackgroundShape",
    ] {
        assert!(!settings_flag_xml(&settings(""), local), "{local} absent");
        assert!(settings_flag_xml(
            &settings(&format!("<w:{local}/>")),
            local
        ));
        assert!(settings_flag_xml(
            &settings(&format!(r#"<w:{local} w:val="on"/>"#)),
            local
        ));
        for off in [
            r#"w:val="0""#,
            r#"w:val="false""#,
            r#"w:val="off""#,
            "w:val='false'",
            "w:val='off'",
        ] {
            assert!(
                !settings_flag_xml(&settings(&format!("<w:{local} {off}/>")), local),
                "{local} {off} is off"
            );
        }
    }
}

#[test]
fn compressed_punctuation_squeezes_unless_balanced_east_asian() {
    let csc = r#"<w:characterSpacingControl w:val="compressPunctuation"/>"#;
    let bal = "<w:compat><w:balanceSingleByteDoubleByteWidth/></w:compat>";
    let lang = |ea: &str| format!(r#"<w:themeFontLang w:val="en-US" w:eastAsia="{ea}"/>"#);
    assert!(settings_punct_squeeze_xml(&settings(csc)));
    assert!(settings_punct_squeeze_xml(&settings(&format!(
        "{csc}{bal}"
    ))));
    assert!(settings_punct_squeeze_xml(&settings(&format!(
        "{csc}{}",
        lang("zh-CN")
    ))));
    assert!(settings_punct_squeeze_xml(&settings(&format!(
        "{csc}{}{bal}",
        lang("en-US")
    ))));
    for ea in ["zh-CN", "zh-TW", "ja-JP", "ko-KR"] {
        let xml = settings(&format!("{csc}{}{bal}", lang(ea)));
        assert!(!settings_punct_squeeze_xml(&xml), "{ea}");
    }
    let fifteen = r#"<w:compat><w:compatSetting w:name="compatibilityMode" w:uri="http://schemas.microsoft.com/office/word" w:val="15"/></w:compat>"#;
    assert!(!settings_punct_squeeze_xml(&settings(&format!(
        "{csc}{fifteen}"
    ))));
    assert!(!settings_punct_squeeze_xml(&settings(
        r#"<w:characterSpacingControl w:val="doNotCompress"/>"#
    )));
}

#[test]
fn compat_flags_are_read_inside_w_compat() {
    for local in [
        "ulTrailSpace",
        "adjustLineHeightInTable",
        "doNotExpandShiftReturn",
        "balanceSingleByteDoubleByteWidth",
    ] {
        let on = settings(&format!("<w:compat><w:{local}/></w:compat>"));
        assert!(settings_flag_xml(&on, local), "{local}");
        let off = settings(&format!("<w:compat><w:{local} w:val='0'/></w:compat>"));
        assert!(!settings_flag_xml(&off, local), "{local} val='0'");
    }
}

#[test]
fn flags_ignore_look_alike_names_and_foreign_namespaces() {
    // Substring matching turned these on: a longer local name, and the
    // same local name in another vendor's namespace.
    let xml = settings(r#"<w:mirrorMarginsX/><x:evenAndOddHeaders xmlns:x="urn:not-word"/>"#);
    assert!(!settings_flag_xml(&xml, "mirrorMargins"));
    assert!(!settings_flag_xml(&xml, "evenAndOddHeaders"));
}

#[test]
fn compat_mode_requires_the_word_uri() {
    let word = settings(
        r#"<w:compat><w:compatSetting w:name="compatibilityMode" w:uri="http://schemas.microsoft.com/office/word" w:val="15"/></w:compat>"#,
    );
    assert_eq!(settings_compat_mode_xml(&word), 15);
    let foreign = settings(
        r#"<w:compat><w:compatSetting w:name="compatibilityMode" w:uri="urn:other-vendor" w:val="15"/></w:compat>"#,
    );
    assert_eq!(
        settings_compat_mode_xml(&foreign),
        12,
        "foreign URI is not Word's mode"
    );
    assert_eq!(settings_compat_mode_xml(&settings("")), 12);
}

#[test]
fn character_spacing_reads_the_attribute_not_the_text() {
    let kana =
        settings(r#"<w:characterSpacingControl w:val="compressPunctuationAndJapaneseKana"/>"#);
    assert!(matches!(
        settings_character_spacing_xml(&kana),
        CharacterSpacing::CompressPunctuationAndKana
    ));
    let punct = settings(r#"<w:characterSpacingControl w:val='compressPunctuation'/>"#);
    assert!(matches!(
        settings_character_spacing_xml(&punct),
        CharacterSpacing::CompressPunctuation
    ));
    // The token elsewhere (a doc variable) is not the setting.
    let decoy = settings(
        r#"<w:characterSpacingControl w:val="doNotCompress"/><w:docVars><w:docVar w:name="n" w:val="compressPunctuation"/></w:docVars>"#,
    );
    assert!(matches!(
        settings_character_spacing_xml(&decoy),
        CharacterSpacing::DoNotCompress
    ));
}
