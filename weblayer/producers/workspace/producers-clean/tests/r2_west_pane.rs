//! R2+R3 acceptance — the proof-script WEST pane assembly, proof trees
//! included.
//!
//! For every overview capture across the 81 manifests (478 bodies: 396 proof
//! views + 82 help views, materialized by tools/extract_r2_panes.py into
//! workspace/r2_panes/), this test:
//!   1. reads the pane inner HTML (the proof-script container's content);
//!   2. INVERTS the producer's skin — un-postprocessing into logical lines and
//!      slicing the opaque content (item labels/annotations, lemma names,
//!      attribute text, formula lines, per-proof-step METHOD text) out of the
//!      strict observed skeleton, with every frame byte asserted on the way.
//!      Proof displays are parsed into structured [`ProofTree`]s (indent,
//!      case/next/qed framing, status classes, step + remove-step links all
//!      asserted — only the method text stays opaque);
//!   3. feeds the slices back through `render_proof_script` and asserts the
//!      re-rendered pane equals the captured bytes EXACTLY.
//!
//! A second test replays live-captured panes from theories NOT in the corpus
//! (workspace/r2_live/, QUERIES.log [L15]) including a proved state, pinning
//! the inline-layout width boundary at exactly 69/70 ([L14]).
//!
//! The inversion is intentionally strict (panics on any unexpected shape) so
//! a capture deviating from the modeled skeleton fails loudly.

use std::fs;
use std::path::PathBuf;

use producers_clean::model::{
    Content, LemmaEntry, NavItem, ProofDisplay, ProofScriptPane, ThyPath,
};
use producers_clean::render_proof_script;

mod common;
use common::{replay, slice_pane};

fn workspace_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

// ---------------------------------------------------------------------------
// The sweeps
// ---------------------------------------------------------------------------

/// All 478 overview panes (82 help + 396 proof views) across the 81
/// manifests: slice, re-render, byte-compare the pane.
#[test]
fn corpus_sweep_all_overview_panes() {
    let dir = workspace_path("../r2_panes");
    let mut count = 0;
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .expect("workspace/r2_panes materialized (tools/extract_r2_panes.py)")
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
    assert_eq!(count, 478, "82 help + 396 proof views");
}

/// Live-probe replays (workspace/r2_live/): panes captured from the oracle on
/// theories NOT in the crawl corpus — the own-authored PathProbe (fresh) and
/// WProbe (the 35-lemma inline/vertical width bisection theory, straddling
/// the 69/70 boundary on four formula families), plus PathProbe at version 2
/// after a live autoprove (a proved hl_good tree) [L14][L15].
#[test]
fn live_probe_pane_replays() {
    let dir = workspace_path("../r2_live");
    let mut count = 0;
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .expect("workspace/r2_live materialized (QUERIES.log [L15])")
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
    assert_eq!(count, 3, "PathProbe v1 + WProbe v1 + proved PathProbe v2");
}

/// Constructed-input fixture: a two-lemma pane with one proved lemma, pinning
/// the frame bytes against hand-checked observed shapes (the AttestedComputation
/// west pane's element order, [S16]).
#[test]
fn fixture_minimal_pane() {
    let pane = ProofScriptPane {
        theory_name: "T".into(),
        index: 3,
        items: vec![NavItem {
            target: ThyPath::Message,
            label: "Message theory".into(),
            annotation: "".into(),
        }],
        lemmas: vec![LemmaEntry {
            name: "foo".into(),
            attributes: " [reuse]".into(),
            quantifier: "all-traces".into(),
            formula: Content { lines: vec!["&quot;F&quot;".into()] },
            proof: ProofDisplay::Unproven,
        }],
    };
    let got = render_proof_script(&pane);
    let want = "<span class=\"hl_keyword\">theory</span> <a class=\"internal-link help\" href=\"/thy/trace/3/main/help\">T</a> <span class=\"hl_keyword\">begin</span><br/>\n\
        <br/>\n\
        <a class=\"internal-link\" href=\"/thy/trace/3/main/message\"><strong>Message theory</strong> </a><br/>\n\
        <br/>\n\
        <a class=\"internal-link add\" href=\"/thy/trace/3/main/add/%3Cfirst%3E\">add lemma</a><br/>\n\
        <br/>\n\
        <span class=\"hl_keyword\">lemma</span> foo [reuse]:<br/>\n\
        &nbsp;&nbsp;all-traces &quot;F&quot;<br/>\n\
        <a class=\"internal-link edit\" href=\"/thy/trace/3/main/edit/foo\">edit lemma</a>  or  <a class=\"internal-link delete\" href=\"/thy/trace/3/main/delete/foo\">delete lemma</a><br/>\n\
        <span class=\"hl_keyword\">by</span> <a class=\"internal-link proof-step sorry-step\" href=\"/thy/trace/3/main/proof/foo\"><span class=\"hl_keyword\">sorry</span></a><br/>\n\
        <br/>\n\
        <a class=\"internal-link add\" href=\"/thy/trace/3/main/add/foo\">add lemma</a><br/>\n\
        <br/>\n\
        <span class=\"hl_keyword\">end</span><br/>\n ";
    assert_eq!(got, want);
    // Round-trip through the inverter too.
    assert_eq!(replay_unchecked(&got), got);
}

/// Zero-lemma pane: two blank lines between the add-first link and `end`
/// (observed in the two lemma-less corpus panes, [S16]).
#[test]
fn fixture_zero_lemma_spacing() {
    let pane = ProofScriptPane {
        theory_name: "E".into(),
        index: 1,
        items: vec![],
        lemmas: vec![],
    };
    let got = render_proof_script(&pane);
    assert!(got.ends_with(
        "add lemma</a><br/>\n<br/>\n<br/>\n<span class=\"hl_keyword\">end</span><br/>\n "
    ));
}

/// Replay without the five-item structural assertion (fixtures use fewer).
fn replay_unchecked(inner: &str) -> String {
    let (_idx, pane) = slice_pane(inner);
    render_proof_script(&pane)
}
