// SPDX-FileCopyrightText: 2026 Jandira Technologies, LLC
//
// SPDX-License-Identifier: AGPL-3.0-only

//! NAME-01c — additional accept/RP W/PT XNames are cached and equal XName::get.

use jubarte::namespaces::{PT, W};
use jubarte::xmllinq::XName;

#[test]
fn name01c_accept_pipeline_names_match_get() {
    assert_eq!(
        W::move_from_range_end(),
        XName::get("moveFromRangeEnd", W::URI)
    );
    assert_eq!(W::move_to_range_end(), XName::get("moveToRangeEnd", W::URI));
    assert_eq!(W::sdt(), XName::get("sdt", W::URI));
    assert_eq!(W::sdt_content(), XName::get("sdtContent", W::URI));
    assert_eq!(W::sdt_pr(), XName::get("sdtPr", W::URI));
    assert_eq!(W::fld_char(), XName::get("fldChar", W::URI));
    assert_eq!(W::instr_text(), XName::get("instrText", W::URI));
    assert_eq!(W::del_instr_text(), XName::get("delInstrText", W::URI));
    assert_eq!(W::num_pr(), XName::get("numPr", W::URI));
    assert_eq!(W::cell_del(), XName::get("cellDel", W::URI));
    assert_eq!(W::cell_ins(), XName::get("cellIns", W::URI));
    assert_eq!(W::cell_merge(), XName::get("cellMerge", W::URI));
    assert_eq!(W::hyperlink(), XName::get("hyperlink", W::URI));
    assert_eq!(W::smart_tag(), XName::get("smartTag", W::URI));
    assert_eq!(W::r_pr_change(), XName::get("rPrChange", W::URI));
    assert_eq!(W::p_pr_change(), XName::get("pPrChange", W::URI));
    assert_eq!(W::numbering_change(), XName::get("numberingChange", W::URI));
    assert_eq!(W::v_merge(), XName::get("vMerge", W::URI));
    assert_eq!(W::hdr(), XName::get("hdr", W::URI));
    assert_eq!(W::ftr(), XName::get("ftr", W::URI));
    assert_eq!(W::fld_simple(), XName::get("fldSimple", W::URI));
}

#[test]
fn name01c_pt_unique_and_run_ids_match_get() {
    assert_eq!(PT::unique_id(), XName::get("UniqueId", PT::URI));
    assert_eq!(PT::run_ids(), XName::get("RunIds", PT::URI));
}

#[test]
fn name01c_cached_stable_across_calls() {
    assert_eq!(W::sdt(), W::sdt());
    assert_eq!(W::cell_del(), W::cell_del());
    assert_eq!(PT::unique_id(), PT::unique_id());
}

#[test]
fn name01c_distinct_from_prior_cache() {
    assert_ne!(W::sdt(), W::p());
    assert_ne!(W::move_from_range_end(), W::move_from_range_start());
    assert_ne!(PT::unique_id(), PT::unid());
}
