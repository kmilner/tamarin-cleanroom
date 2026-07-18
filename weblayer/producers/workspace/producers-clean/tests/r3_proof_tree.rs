//! R3 acceptance — structured proof-tree rendering.
//!
//! The corpus-wide gate lives in tests/r2_west_pane.rs (every proof display
//! in all 478 overview panes is parsed into a [`ProofTree`] and re-rendered
//! byte-identically). This file adds:
//!   * live-probe replays of panes from own-authored theories NOT in the
//!     corpus (workspace/r3_live/, QUERIES.log [L16]–[L18]), forcing
//!     never-captured shapes: an autoproved multi-case/branching state, a
//!     bounded (incomplete) autoprove, a replayed-script leftover
//!     (hl_superfluous) after a doctored invalid step, and a mixed
//!     good/bad tree from a characterize autoprove;
//!   * constructed-input fixtures pinning the exact observed bytes of each
//!     line form (method step, `by ` closing step, terminal marker, sorry
//!     slot, case/next/qed framing, unnamed continuation, replayed step).

use std::fs;
use std::path::PathBuf;

use producers_clean::html::postprocess_lines;
use producers_clean::model::{Highlight, ProofTree};
use producers_clean::render_proof_tree;

mod common;
use common::replay;

fn workspace_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

/// Live-probe replays (workspace/r3_live/): whole west panes captured from
/// the oracle on own-authored theories, sliced and re-rendered byte-exactly.
///   * treeprobe_v5_proved — TreeProbe after autoproveAll: a good tree with
///     case/qed framing, a two-case next-separated good tree, and a bad
///     (attack) tree with SOLVED under a case [L16];
///   * treeprobe_v6_bounded — a bound-1 autoprove: unwrapped header,
///     sorry-step interior WITH remove-step, bare-`by` sorry leaf with a
///     comment and NO remove-step [L17];
///   * scriptprobe_v1_superfluous — a doctored embedded proof script: the
///     invalid step replaced by `sorry /* invalid proof step encountered */`
///     carrying the leftover subtree as its unnamed continuation, every
///     leftover node span-wrapped hl_superfluous with remove-step links
///     through the leftover [L18];
///   * treeprobe2_v2_mixed — characterize on a false lemma: an hl_bad parent
///     whose case list mixes a bad (SOLVED) and a good (`by` zero-case
///     solve) subtree — pinning next/qed to the PARENT status where parent
///     and sibling statuses differ [L18].
#[test]
fn live_probe_tree_replays() {
    let dir = workspace_path("../r3_live");
    let mut count = 0;
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .expect("workspace/r3_live materialized (QUERIES.log [L16]-[L18])")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "pane"))
        .collect();
    entries.sort();
    for path in entries {
        let stem = path.file_stem().unwrap().to_str().unwrap().to_string();
        let inner = fs::read_to_string(&path).unwrap();
        assert_eq!(replay(&inner), inner, "byte mismatch replaying {stem}");
        count += 1;
    }
    assert_eq!(count, 4, "proved + bounded + superfluous + mixed");
}

fn kw(k: &str) -> String {
    format!("<span class=\"hl_keyword\">{k}</span>")
}

fn good(
    method_text: &str,
    terminal_marker: bool,
    cases: Vec<(String, ProofTree)>,
) -> ProofTree {
    ProofTree {
        method_text: method_text.to_string(),
        status: Highlight::Good,
        live: true,
        terminal_marker,
        cases,
    }
}

/// The TreeProbe v5 `origin` display: simplify, an unnamed solve
/// continuation, two contradiction cases separated by `next`, closed by
/// `qed` — expected bytes verbatim from the live capture [L16].
#[test]
fn fixture_two_case_good_tree() {
    let contra = format!(
        "{} <span class=\"hl_comment\">/* from formulas */</span>",
        kw("contradiction")
    );
    let solve = format!("{} Fin( x ) ▶₀ #i {}", kw("solve("), kw(")"));
    let tree = good(
        &kw("simplify"),
        false,
        vec![(
            String::new(),
            good(
                &solve,
                false,
                vec![
                    ("HopOn_case_1".to_string(), good(&contra, false, vec![])),
                    ("HopOn_case_2".to_string(), good(&contra, false, vec![])),
                ],
            ),
        )],
    );
    let want = "<a class=\"internal-link proof-step hl_good\" href=\"/thy/trace/5/main/proof/origin\"><span class=\"hl_keyword\">simplify</span></a><a class=\"internal-link remove-step\" href=\"/thy/trace/5/main/proof/origin\"></a><br/>\n\
        <a class=\"internal-link proof-step hl_good\" href=\"/thy/trace/5/main/proof/origin/_\"><span class=\"hl_keyword\">solve(</span> Fin( x ) ▶₀ #i <span class=\"hl_keyword\">)</span></a><a class=\"internal-link remove-step\" href=\"/thy/trace/5/main/proof/origin/_\"></a><br/>\n\
        &nbsp;&nbsp;<span class=\"hl_good\"><span class=\"hl_keyword\">case</span> HopOn_case_1</span><br/>\n\
        &nbsp;&nbsp;<span class=\"hl_good\"><span class=\"hl_keyword\">by</span> </span><a class=\"internal-link proof-step hl_good\" href=\"/thy/trace/5/main/proof/origin/_/HopOn_case_1\"><span class=\"hl_keyword\">contradiction</span> <span class=\"hl_comment\">/* from formulas */</span></a><a class=\"internal-link remove-step\" href=\"/thy/trace/5/main/proof/origin/_/HopOn_case_1\"></a><br/>\n\
        <span class=\"hl_good\"><span class=\"hl_keyword\">next</span></span><br/>\n\
        &nbsp;&nbsp;<span class=\"hl_good\"><span class=\"hl_keyword\">case</span> HopOn_case_2</span><br/>\n\
        &nbsp;&nbsp;<span class=\"hl_good\"><span class=\"hl_keyword\">by</span> </span><a class=\"internal-link proof-step hl_good\" href=\"/thy/trace/5/main/proof/origin/_/HopOn_case_2\"><span class=\"hl_keyword\">contradiction</span> <span class=\"hl_comment\">/* from formulas */</span></a><a class=\"internal-link remove-step\" href=\"/thy/trace/5/main/proof/origin/_/HopOn_case_2\"></a><br/>\n\
        <span class=\"hl_good\"><span class=\"hl_keyword\">qed</span></span><br/>\n";
    assert_eq!(
        postprocess_lines(&render_proof_tree(5, "origin", &tree)),
        want
    );
}

/// A terminal marker (`SOLVED // trace found`) never takes the `by ` prefix
/// a closing method step carries ([S19], TreeProbe v5 `reach` [L16]).
#[test]
fn fixture_terminal_marker_no_by() {
    let solved = format!(
        "{} <span class=\"hl_comment\">// trace found</span>",
        kw("SOLVED")
    );
    let tree = good(&solved, true, vec![]);
    assert_eq!(
        render_proof_tree(9, "reach", &tree),
        "<a class=\"internal-link proof-step hl_good\" href=\"/thy/trace/9/main/proof/reach\"><span class=\"hl_keyword\">SOLVED</span> <span class=\"hl_comment\">// trace found</span></a><a class=\"internal-link remove-step\" href=\"/thy/trace/9/main/proof/reach\"></a>"
    );
}

/// The bounded-autoprove incomplete display (TreeProbe v6 [L17]): the
/// status-less interior keeps its remove-step; the sorry leaf renders with a
/// bare `by ` and no remove-step.
#[test]
fn fixture_incomplete_sorry_leaf() {
    let sorry = format!(
        "{} <span class=\"hl_comment\">/* bound 1 hit */</span>",
        kw("sorry")
    );
    let tree = ProofTree {
        method_text: kw("simplify"),
        status: Highlight::None,
        live: true,
        terminal_marker: false,
        cases: vec![(
            String::new(),
            ProofTree {
                method_text: sorry,
                status: Highlight::None,
                live: false,
                terminal_marker: false,
                cases: vec![],
            },
        )],
    };
    assert_eq!(
        render_proof_tree(6, "origin", &tree),
        "<a class=\"internal-link proof-step sorry-step\" href=\"/thy/trace/6/main/proof/origin\"><span class=\"hl_keyword\">simplify</span></a><a class=\"internal-link remove-step\" href=\"/thy/trace/6/main/proof/origin\"></a>\n\
         <span class=\"hl_keyword\">by</span> <a class=\"internal-link proof-step sorry-step\" href=\"/thy/trace/6/main/proof/origin/_\"><span class=\"hl_keyword\">sorry</span> <span class=\"hl_comment\">/* bound 1 hit */</span></a>"
    );
}

/// A replayed-script leftover (ScriptProbe [L18]): the invalid-step sorry
/// keeps its proof-step link but takes no `by ` (it has a continuation) and
/// no remove-step; the leftover child renders span-wrapped hl_superfluous
/// with a remove-step link, its `by ` prefix wrapped the same way.
#[test]
fn fixture_replayed_leftover() {
    let leftover = ProofTree {
        method_text: format!("{} Missing( x ) ▶₀ #i {}", kw("solve("), kw(")")),
        status: Highlight::Replayed,
        live: true,
        terminal_marker: false,
        cases: vec![],
    };
    let tree = ProofTree {
        method_text: format!(
            "{} <span class=\"hl_comment\">/* invalid proof step encountered */</span>",
            kw("sorry")
        ),
        status: Highlight::None,
        live: false,
        terminal_marker: false,
        cases: vec![(String::new(), leftover)],
    };
    assert_eq!(
        render_proof_tree(1, "origin", &tree),
        "<a class=\"internal-link proof-step sorry-step\" href=\"/thy/trace/1/main/proof/origin\"><span class=\"hl_keyword\">sorry</span> <span class=\"hl_comment\">/* invalid proof step encountered */</span></a>\n\
         <span class=\"hl_superfluous\"><span class=\"hl_keyword\">by</span> </span><span class=\"hl_superfluous\"><span class=\"hl_keyword\">solve(</span> Missing( x ) ▶₀ #i <span class=\"hl_keyword\">)</span></span><a class=\"internal-link remove-step\" href=\"/thy/trace/1/main/proof/origin/_\"></a>"
    );
}

/// Mixed statuses (TreeProbe2 characterize [L18]): `next`/`qed` carry the
/// PARENT's (bad) status even when the following case is good; the case
/// line carries its child's own status.
#[test]
fn fixture_mixed_status_framing() {
    let bad_leaf = ProofTree {
        method_text: format!(
            "{} <span class=\"hl_comment\">// trace found</span>",
            kw("SOLVED")
        ),
        status: Highlight::Bad,
        live: true,
        terminal_marker: true,
        cases: vec![],
    };
    let good_leaf = ProofTree {
        method_text: format!("{} St( ~x ) ▶₀ #j {}", kw("solve("), kw(")")),
        status: Highlight::Good,
        live: true,
        terminal_marker: false,
        cases: vec![],
    };
    let tree = ProofTree {
        method_text: format!("{} Hop( x ) ▶₀ #i {}", kw("solve("), kw(")")),
        status: Highlight::Bad,
        live: true,
        terminal_marker: false,
        cases: vec![
            ("Fork1".to_string(), bad_leaf),
            ("Fork2".to_string(), good_leaf),
        ],
    };
    let got = render_proof_tree(2, "mixed", &tree);
    let lines: Vec<&str> = got.split('\n').collect();
    assert_eq!(
        lines[1],
        "  <span class=\"hl_bad\"><span class=\"hl_keyword\">case</span> Fork1</span>"
    );
    assert_eq!(
        lines[3],
        "<span class=\"hl_bad\"><span class=\"hl_keyword\">next</span></span>",
        "next carries the parent's status"
    );
    assert_eq!(
        lines[4],
        "  <span class=\"hl_good\"><span class=\"hl_keyword\">case</span> Fork2</span>",
        "case carries the child's status"
    );
    assert!(lines[5].starts_with(
        "  <span class=\"hl_good\"><span class=\"hl_keyword\">by</span> </span><a class=\"internal-link proof-step hl_good\""
    ));
    assert_eq!(
        lines[6],
        "<span class=\"hl_bad\"><span class=\"hl_keyword\">qed</span></span>",
        "qed carries the parent's status"
    );
}
