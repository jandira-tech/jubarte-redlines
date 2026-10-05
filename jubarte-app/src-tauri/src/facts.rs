//! What the app tells about itself, from `data/facts.jsonl` compiled in: the
//! About window's company, contacts and links, and each section of the Terms of
//! Use and the Privacy Policy, the same records jubarte.pro renders.
//!
//! The log is append-only (`scripts/facts.py` writes it): a key's value is its
//! latest record, and uuidv7 ids sort by time; a null value retires the key.
//! A section's `{{key}}` placeholders are facts too.

use serde::Serialize;
use serde_json::{Map, Value};
use std::sync::LazyLock;

const LOG: &str = include_str!("../../data/facts.jsonl");

static FACTS: LazyLock<Map<String, Value>> = LazyLock::new(|| fold(LOG));

/// Each live key's latest value.
fn fold(log: &str) -> Map<String, Value> {
    let mut records: Vec<Value> = log
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("data/facts.jsonl holds a line that is not JSON"))
        .collect();
    // Lowercase uuidv7 strings sort as their timestamps do.
    records.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
    let mut out = Map::new();
    for mut record in records {
        let Some(key) = record["key"].as_str().map(str::to_owned) else {
            continue;
        };
        match record["value"].take() {
            Value::Null => out.remove(&key),
            value => out.insert(key, value),
        };
    }
    out
}

/// A fact as text; a missing one is a build that shipped without it.
pub fn text(key: &str) -> String {
    match FACTS.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(other) => other.to_string(),
        None => panic!("data/facts.jsonl has no {key}"),
    }
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// `html` with each `{{key}}` replaced by that fact, escaped.
fn fill(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find("{{") {
        let Some(len) = rest[start..].find("}}") else {
            break;
        };
        out.push_str(&rest[..start]);
        out.push_str(&escape(&text(&rest[start + 2..start + len])));
        rest = &rest[start + len + 2..];
    }
    out.push_str(rest);
    out
}

/// "2026-10-02" as "October 2, 2026", the way the site dates its pages.
fn long_date(iso: &str) -> String {
    const MONTHS: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    let mut parts = iso.splitn(3, '-').map(|p| p.parse::<usize>().ok());
    match (
        parts.next().flatten(),
        parts.next().flatten(),
        parts.next().flatten(),
    ) {
        (Some(y), Some(m @ 1..=12), Some(d)) => format!("{} {d}, {y}", MONTHS[m - 1]),
        _ => iso.to_owned(),
    }
}

/// A section of a legal document: its heading (none for the opening) and HTML.
#[derive(Serialize, Debug)]
pub struct Section {
    heading: Option<String>,
    html: String,
}

/// The Terms of Use or the Privacy Policy, as jubarte.pro prints it.
#[derive(Serialize, Debug)]
pub struct LegalDocument {
    title: &'static str,
    updated: String,
    url: String,
    sections: Vec<Section>,
}

fn legal(doc: &str) -> Option<LegalDocument> {
    let title = match doc {
        "terms" => "Terms of Use",
        "privacy" => "Privacy Policy",
        _ => return None,
    };
    let prefix = format!("legal.{doc}.");
    let mut found: Vec<(u64, Section)> = FACTS
        .iter()
        .filter(|(key, _)| key.starts_with(&prefix))
        .map(|(_, v)| {
            let section = Section {
                heading: v["heading"].as_str().map(str::to_owned),
                html: fill(v["html"].as_str().unwrap_or_default()),
            };
            (v["order"].as_u64().unwrap_or(u64::MAX), section)
        })
        .collect();
    found.sort_by_key(|(order, _)| *order);
    let mut sections: Vec<Section> = found.into_iter().map(|(_, s)| s).collect();
    let mail = escape(&text("contact.support_email"));
    sections.push(Section {
        heading: Some("Contact".to_owned()),
        html: format!(
            r#"<p>{} · <a href="mailto:{mail}">{mail}</a></p>"#,
            escape(&text("company.name"))
        ),
    });
    Some(LegalDocument {
        title,
        updated: long_date(&text("legal.updated")),
        url: format!("{}/{doc}", text("site.url")),
        sections,
    })
}

/// The Terms of Use (`terms`) or the Privacy Policy (`privacy`).
#[tauri::command]
pub fn legal_document(doc: String) -> Result<LegalDocument, String> {
    legal(&doc).ok_or_else(|| format!("no legal document {doc:?}"))
}

/// What the About window shows.
#[derive(Serialize, Debug)]
pub struct About {
    version: String,
    engine: &'static str,
    company: String,
    support_email: String,
    site: String,
    copyright: String,
    engine_repo: String,
}

fn about_for(version: String) -> About {
    let updated = text("legal.updated");
    let year = updated.split('-').next().unwrap_or_default();
    About {
        version,
        engine: env!("JUBARTE_ENGINE_VERSION"),
        company: text("company.name"),
        support_email: text("contact.support_email"),
        site: text("site.url"),
        copyright: format!("© {year} {}", text("company.name")),
        engine_repo: text("engine.repo"),
    }
}

#[tauri::command]
pub fn about<R: tauri::Runtime>(app: tauri::AppHandle<R>) -> About {
    about_for(app.package_info().version.to_string())
}

/// A whole-number fact.
fn number(key: &str) -> u64 {
    FACTS
        .get(key)
        .and_then(Value::as_u64)
        .unwrap_or_else(|| panic!("data/facts.jsonl has no whole number {key}"))
}

/// Documents under this many bytes each preview as soon as they are chosen,
/// unless Settings turns that off (src/settings.js). jubarte.pro's Demo and
/// App pages use the same fact.
#[tauri::command]
pub fn instant_preview_limit() -> u64 {
    number("app.instant_preview_max_bytes")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documents_under_a_megabyte_as_finder_counts_preview_at_once() {
        assert_eq!(instant_preview_limit(), 1_000_000);
    }

    #[test]
    fn the_latest_record_wins_whatever_the_line_order_and_null_retires_a_key() {
        let log = [
            r#"{"id":"01a0fe27-0000-7000-8000-000000000002","ts":"","key":"engine.version","value":"0.10.2","source":"t"}"#,
            r#"{"id":"01a0fe27-0000-7000-8000-000000000001","ts":"","key":"engine.version","value":"0.10.1","source":"t"}"#,
            r#"{"id":"01a0fe27-0000-7000-8000-000000000001","ts":"","key":"site.gone","value":1,"source":"t"}"#,
            r#"{"id":"01a0fe27-0000-7000-8000-000000000003","ts":"","key":"site.gone","value":null,"source":"t"}"#,
        ]
        .join("\n");
        let facts = fold(&log);
        assert_eq!(facts.len(), 1);
        assert_eq!(facts["engine.version"], "0.10.2");
    }

    #[test]
    fn both_documents_are_the_sites_sections_with_every_placeholder_filled() {
        for doc in ["terms", "privacy"] {
            let d = legal(doc).unwrap();
            assert!(d.sections.len() > 5, "{doc}");
            assert!(
                d.sections[0].heading.is_none(),
                "{doc} opens without a heading"
            );
            assert_eq!(
                d.sections.last().unwrap().heading.as_deref(),
                Some("Contact")
            );
            for s in &d.sections {
                assert!(!s.html.contains("{{"), "{doc}: {}", s.html);
            }
            assert_eq!(d.url, format!("https://jubarte.pro/{doc}"));
        }
        let terms = legal("terms").unwrap();
        let all: String = terms.sections.iter().map(|s| s.html.as_str()).collect();
        assert!(all.contains(&format!(
            "more than {} page requests",
            number("site.limit_per_minute")
        )));
        assert!(all.contains("Jandira Technologies, LLC shall not be liable"));
        assert!(legal("cookies").is_none());
    }

    #[test]
    fn dates_read_as_the_site_prints_them() {
        assert_eq!(long_date("2026-10-02"), "October 2, 2026");
        assert_eq!(long_date("2026-13-02"), "2026-13-02");
        assert_eq!(long_date(&text("legal.updated")).split(' ').count(), 3);
    }

    #[test]
    fn about_names_the_app_the_engine_and_the_company() {
        let a = about_for("0.10.1".to_owned());
        assert_eq!(a.company, "Jandira Technologies, LLC");
        assert!(a.copyright.starts_with("© 20"));
        assert!(a.copyright.ends_with("Jandira Technologies, LLC"));
        assert!(a.engine.split('.').count() == 3, "{}", a.engine);
        assert_eq!(a.site, "https://jubarte.pro");
    }

    #[test]
    fn the_free_uses_are_the_ones_the_site_and_terms_promise() {
        assert_eq!(u64::from(crate::quota::FREE_LIMIT), number("app.free_uses"));
    }

    #[test]
    fn a_placeholder_is_escaped_and_plain_text_stays() {
        assert_eq!(fill("no placeholders"), "no placeholders");
        assert_eq!(fill("© {{company.name}}"), "© Jandira Technologies, LLC");
        assert_eq!(escape(r#"<a & "b">"#), "&lt;a &amp; &quot;b&quot;&gt;");
    }
}
