//! Compile-time check that the public API matches interface/required_api.md.
use std::collections::BTreeSet;
use wf_clean::ast::Theory;
use wf_clean::*;

#[test]
fn required_signatures_exist() {
    let _new: fn(String, String) -> WfError = |t, m| WfError::new(t, m);
    let _report_type: fn() -> WfReport = Vec::new;
    let _check: fn(&Theory) -> WfReport = check_theory;
    let _topics: fn(&WfReport) -> BTreeSet<String> = topics;
    let _underline: fn(&str) -> String = underline_topic;
    let _render: fn(&WfReport) -> String = render_report;
    let _lemmas: fn(&[String], &Theory) -> WfReport = check_if_lemmas_in_theory;
    let _lhs: fn(&Theory) -> WfReport = fact_lhs_occur_no_rhs;
    let _pn: fn(&Theory) -> WfReport = public_names_report;
    let _pnp: fn(Vec<(String, String)>) -> WfReport = public_names_report_from_pairs;
    let _ins: fn(&mut Vec<WfError>, Vec<WfError>, &[&str]) = insert_wf_before;
    let _after: fn() -> Vec<&'static str> = after_public_names_topics;
    // touch the WfError fields
    let e = WfError::new("t", "m");
    assert_eq!(e.topic, "t");
    assert_eq!(e.message, "m");
}
