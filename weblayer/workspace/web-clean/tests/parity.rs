//! Byte-parity tests: render functions vs. captured oracle response bodies.
//!
//! Fixtures under `tests/fixtures/` are captured program OUTPUT (allowed
//! observation material per the clean-room protocol), extracted verbatim from
//! the crawl manifests. Each test feeds the observed input slots into a render
//! function and asserts the produced bytes equal the captured response.

use serde::Deserialize;
use serde_json::Value;
use web_clean::envelope::{render_content, render_redirect};
use web_clean::forms::{add_form, delete_form, edit_form};
use web_clean::intdot::{render_intdot, EMPTY_GRAPH_DOT};
use web_clean::page::{render_page, PageParams};
use web_clean::proofscript::{render_proof_script, Item, Lemma, Overview, Proof, ProofLine};
use web_clean::text::{main_proof_path, nav_target, source_body};

/// The five standard theory-item links shared by both Chaum captures (the
/// labels/counts/descriptions are prover fragments taken verbatim from the
/// observed west pane).
fn chaum_items() -> Vec<Item<'static>> {
    vec![
        Item::Message,
        Item::Rules {
            label: "Multiset rewriting rules and restrictions",
            count: 7,
        },
        Item::Tactic,
        Item::RawSources {
            desc: "12 cases, deconstructions complete",
        },
        Item::RefinedSources {
            desc: "12 cases, deconstructions complete",
        },
    ]
}

// ---------------------------------------------------------------------------
// JSON envelopes: reproduce every captured json body from its parsed slots.
// ---------------------------------------------------------------------------

fn check_all_envelopes(manifest_json: &str) -> (usize, usize) {
    let entries: Vec<Value> = serde_json::from_str(manifest_json).unwrap();
    let (mut content, mut redirect) = (0usize, 0usize);
    for e in &entries {
        let body = e["body"].as_str().unwrap();
        let parsed: Value = serde_json::from_str(body).unwrap();
        let obj = parsed.as_object().unwrap();
        if let Some(r) = obj.get("redirect") {
            let got = render_redirect(r.as_str().unwrap());
            assert_eq!(got, body, "redirect envelope mismatch for {}", e["route"]);
            redirect += 1;
        } else {
            let html = obj["html"].as_str().unwrap();
            let title = obj["title"].as_str().unwrap();
            let got = render_content(html, title);
            assert_eq!(got, body, "content envelope mismatch for {}", e["route"]);
            content += 1;
        }
    }
    (content, redirect)
}

#[test]
fn json_envelopes_chaum_byte_identical() {
    let (c, r) = check_all_envelopes(include_str!("fixtures/envelopes_chaum.json"));
    assert!(c > 0 && r > 0, "expected both shapes, got {c} content / {r} redirect");
}

#[test]
fn json_envelopes_issue_byte_identical() {
    let (c, _r) = check_all_envelopes(include_str!("fixtures/envelopes_issue.json"));
    assert!(c > 0);
}

/// Generalization: 2450 distinct json bodies deduplicated across all 81
/// manifests (varied unicode, escaping, redirect targets). Reproducing every
/// one byte-for-byte demonstrates the envelope serializer matches the server
/// across the whole json surface (~18k routes), not just one theory.
#[test]
fn json_envelopes_corpus_byte_identical() {
    let (c, r) = check_all_envelopes(include_str!("fixtures/envelopes_corpus.json"));
    assert!(c >= 1500 && r >= 800, "coverage regressed: {c} content / {r} redirect");
}

// ---------------------------------------------------------------------------
// Full theory-view page shell (overview/*).
// ---------------------------------------------------------------------------

#[test]
fn page_shell_chaum_byte_identical() {
    let west = include_str!("fixtures/west_chaum.html");
    let center = include_str!("fixtures/center_chaum.html");
    let expected = include_str!("fixtures/page_overview_help_chaum.html");
    let params = PageParams {
        theory_name: "Chaum_Unforgeability",
        index: 1,
        version: "1.13.0",
        filename: "Chaum_Unforgeability.spthy",
    };
    assert_eq!(render_page(&params, west, center), expected);
}

#[test]
fn page_shell_issue_byte_identical() {
    let west = include_str!("fixtures/west_issue.html");
    let center = include_str!("fixtures/center_issue.html");
    let expected = include_str!("fixtures/page_overview_help_issue.html");
    let params = PageParams {
        theory_name: "issue515",
        index: 1,
        version: "1.13.0",
        filename: "issue515.spthy",
    };
    assert_eq!(render_page(&params, west, center), expected);
}

// ---------------------------------------------------------------------------
// Proof-script (west) pane, generated from the theory-overview model.
// ---------------------------------------------------------------------------

#[test]
fn proof_script_no_lemma_byte_identical() {
    let o = Overview {
        theory_name: "issue515",
        index: 1,
        items: vec![
            Item::Message,
            Item::Rules { label: "Multiset rewriting rules", count: 7 },
            Item::Tactic,
            Item::RawSources { desc: "4 cases, deconstructions complete" },
            Item::RefinedSources { desc: "4 cases, deconstructions complete" },
        ],
        lemmas: vec![],
    };
    let got = web_clean::proofscript::render_proof_script(&o);
    assert_eq!(got, include_str!("fixtures/west_issue.html"));
}

#[test]
fn proof_script_two_lemmas_byte_identical() {
    let exec_decl = include_str!("fixtures/exec_decl.html");
    let unforg_decl = include_str!("fixtures/unforg_decl.html");
    let o = Overview {
        theory_name: "Chaum_Unforgeability",
        index: 1,
        items: chaum_items(),
        lemmas: vec![
            Lemma { name: "exec", decl_html: exec_decl, proof: Proof::Sorry },
            Lemma { name: "unforgeability", decl_html: unforg_decl, proof: Proof::Sorry },
        ],
    };
    assert_eq!(render_proof_script(&o), include_str!("fixtures/west_chaum.html"));
}

// ---------------------------------------------------------------------------
// Solved proof tree (west pane after autoprove), reproduced from the observed
// proof-line model. The lemma declarations, proof-method HTML and case names
// are prover fragments (loaded from fixtures); every wrapper — the header
// status span, indentation, anchors, keyword spans, by-prefix, remove-step
// anchor, case/next/qed markup, href construction and blank-line spacing — is
// generated by the crate and asserted byte-for-byte against the capture.
// ---------------------------------------------------------------------------

/// One proof-tree line as captured (my own observation model, in JSON).
#[derive(Deserialize)]
struct LineRec {
    kind: String,
    depth: usize,
    status: String,
    #[serde(default)]
    by: bool,
    #[serde(default)]
    method_html: String,
    #[serde(default)]
    annotation: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    path: Vec<String>,
}

/// A lemma's observed proof: the header status and its proof-tree lines.
#[derive(Deserialize)]
struct LemmaProof {
    lemma: String,
    decl_status: String,
    lines: Vec<LineRec>,
}

/// Turn the observed line records into `ProofLine` values, building each step's
/// href with the crate's own URL builder so URL construction is under test too.
fn to_proof_lines<'a>(lp: &'a LemmaProof, idx: u64) -> Vec<ProofLine<'a>> {
    lp.lines
        .iter()
        .map(|r| match r.kind.as_str() {
            "step" => {
                let segs: Vec<&str> = r.path.iter().map(String::as_str).collect();
                ProofLine::Step {
                    depth: r.depth,
                    status: &r.status,
                    href: main_proof_path(idx, &lp.lemma, &segs),
                    method_html: &r.method_html,
                    annotation: &r.annotation,
                    by: r.by,
                }
            }
            "case" => ProofLine::Case { depth: r.depth, status: &r.status, name: &r.name },
            "next" => ProofLine::Next { depth: r.depth, status: &r.status },
            "qed" => ProofLine::Qed { depth: r.depth, status: &r.status },
            other => panic!("unknown proof-line kind {other:?}"),
        })
        .collect()
}

#[test]
fn proof_script_solved_tree_byte_identical() {
    let exec_lp: LemmaProof =
        serde_json::from_str(include_str!("fixtures/proof_lines_exec.json")).unwrap();
    let unforg_lp: LemmaProof =
        serde_json::from_str(include_str!("fixtures/proof_lines_unforgeability.json")).unwrap();
    let exec_lines = to_proof_lines(&exec_lp, 3);
    let unforg_lines = to_proof_lines(&unforg_lp, 3);
    let o = Overview {
        theory_name: "Chaum_Unforgeability",
        index: 3,
        items: chaum_items(),
        lemmas: vec![
            Lemma {
                name: "exec",
                decl_html: include_str!("fixtures/exec_decl.html"),
                proof: Proof::Steps { status: &exec_lp.decl_status, lines: exec_lines },
            },
            Lemma {
                name: "unforgeability",
                decl_html: include_str!("fixtures/unforg_decl.html"),
                proof: Proof::Steps { status: &unforg_lp.decl_status, lines: unforg_lines },
            },
        ],
    };
    assert_eq!(
        render_proof_script(&o),
        include_str!("fixtures/west_chaum_proved.html")
    );
}

/// The full theory-view page for a solved proof node (`overview/proof/exec` at
/// version 3): a second page type and a second index, exercising the shell
/// around a proof-tree west pane and an "Applicable Proof Methods" center pane.
#[test]
fn page_shell_proof_view_byte_identical() {
    let west = include_str!("fixtures/west_chaum_proved.html");
    let center = include_str!("fixtures/center_proof_exec.html");
    let expected = include_str!("fixtures/page_overview_proof_exec.html");
    let params = PageParams {
        theory_name: "Chaum_Unforgeability",
        index: 3,
        version: "1.13.0",
        filename: "Chaum_Unforgeability.spthy",
    };
    assert_eq!(render_page(&params, west, center), expected);
}

// ---------------------------------------------------------------------------
// Lemma edit / delete / add form envelopes.
// ---------------------------------------------------------------------------

#[test]
fn edit_form_byte_identical() {
    let text = include_str!("fixtures/edit_exec_lemmatext.txt");
    let got = render_content(&edit_form("exec", text), "Edit Lemma: exec");
    assert_eq!(got, include_str!("fixtures/edit_exec.json"));
}

#[test]
fn delete_form_byte_identical() {
    let got = render_content(&delete_form("exec"), "Delete exec");
    assert_eq!(got, include_str!("fixtures/delete_exec.json"));
}

#[test]
fn add_form_named_byte_identical() {
    let got = render_content(&add_form("exec"), "Add new Lemma");
    assert_eq!(got, include_str!("fixtures/add_exec.json"));
}

#[test]
fn add_form_first_byte_identical() {
    let got = render_content(&add_form("<first>"), "Add new Lemma");
    assert_eq!(got, include_str!("fixtures/add_first.json"));
}

// ---------------------------------------------------------------------------
// intdot mini-page and empty-graph DOT.
// ---------------------------------------------------------------------------

#[test]
fn intdot_page_byte_identical() {
    let got = render_intdot(
        "Chaum_Unforgeability",
        "/thy/trace/3/interactive-graph-def/proof/exec",
    );
    assert_eq!(got, include_str!("fixtures/intdot_exec.html"));
}

#[test]
fn empty_graph_dot_byte_identical() {
    assert_eq!(EMPTY_GRAPH_DOT, include_str!("fixtures/igd_exec.dot"));
}

// ---------------------------------------------------------------------------
// Plain-text bodies (pass-through).
// ---------------------------------------------------------------------------

#[test]
fn source_text_passthrough() {
    let src = include_str!("fixtures/source_chaum.txt");
    assert_eq!(source_body(src), src);
}

#[test]
fn nav_target_passthrough() {
    let n = include_str!("fixtures/next_exec.txt");
    assert_eq!(nav_target(n), n);
}

// ---------------------------------------------------------------------------
// 404 Not Found page (captured by live probing).
// ---------------------------------------------------------------------------

#[test]
fn not_found_page_byte_identical() {
    let got = web_clean::errors::render_not_found("/thy/trace/1/main/nope");
    assert_eq!(got, include_str!("fixtures/notfound_nope.html"));
}

// ---------------------------------------------------------------------------
// Multi-theory HTML page generality (a committed sample of the corpus-wide
// harness in `examples/corpus_html.rs`). Reproduces both html families —
// `intdot` mini-pages and `overview/*` full shells — across six distinct
// theories and four theory-version indices, byte-for-byte. The intdot bodies
// are rendered fully from the model (theory name + request index + URL tail);
// the overview shells are reproduced with the west/center pane inner HTML
// treated as opaque prover/proof fragments, exactly as the single-page shell
// tests above. See workspace/REPORT2.md for the full-corpus percentages.
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct HtmlRec {
    name: String,
    ver: String,
    file: String,
    u: String,
    b: String,
}

fn index_after(body: &str, needle: &str) -> Option<u64> {
    let start = body.find(needle)? + needle.len();
    body[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit())
        .collect::<String>()
        .parse()
        .ok()
}

fn between<'a>(body: &'a str, open: &str, close: &str) -> Option<&'a str> {
    let a = body.find(open)? + open.len();
    let b = body[a..].find(close)? + a;
    Some(&body[a..b])
}

#[test]
fn html_page_generality_sample_byte_identical() {
    use web_clean::intdot::{dotsrc_path, render_intdot};

    const WEST_OPEN: &str = r#"<div class="monospace" id="proof">"#;
    const WEST_CLOSE: &str = r#"</div></div></div><div class="ui-layout-east">"#;
    const CENTER_OPEN: &str = r#"<div id="ui-main-display">"#;
    const CENTER_CLOSE: &str = r#"</div></div></div><div id="dialog">"#;

    let sample = include_str!("fixtures/html_sample.ndjson");
    let (mut intdot, mut overview) = (0usize, 0usize);
    for line in sample.lines().filter(|l| !l.trim().is_empty()) {
        let r: HtmlRec = serde_json::from_str(line).unwrap();
        let rest = r.u.strip_prefix("/thy/trace/#/").unwrap();
        let (handler, tail) = rest.split_once('/').unwrap_or((rest, ""));
        match handler {
            "intdot" => {
                // The only value read from the target is the erased request
                // index; name (sibling page) and tail (URL) are independent.
                let idx = index_after(&r.b, "dotsrc=\"/thy/trace/").unwrap();
                let got = render_intdot(&r.name, &dotsrc_path(idx, tail));
                assert_eq!(got, r.b, "intdot mismatch for {}", r.u);
                intdot += 1;
            }
            "overview" => {
                let idx = index_after(&r.b, "action=\"/thy/trace/").unwrap();
                let west = between(&r.b, WEST_OPEN, WEST_CLOSE).unwrap();
                let center = between(&r.b, CENTER_OPEN, CENTER_CLOSE).unwrap();
                let params = PageParams {
                    theory_name: &r.name,
                    index: idx,
                    version: &r.ver,
                    filename: &r.file,
                };
                assert_eq!(render_page(&params, west, center), r.b, "overview mismatch for {}", r.u);
                overview += 1;
            }
            other => panic!("unexpected html handler {other:?}"),
        }
    }
    assert!(intdot >= 8 && overview >= 10, "sample coverage regressed: {intdot} intdot / {overview} overview");
}
