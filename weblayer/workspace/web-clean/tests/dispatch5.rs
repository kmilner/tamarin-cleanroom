//! Round-5 byte-parity / behaviour tests for the `del/path/…` and `verify/…`
//! route families (BEHAVIOR.md §15; QUERIES.log [R50]–[R57]).
//!
//! Every asserted string traces to a live oracle observation of theory
//! `RevealingSignatures` (regression/trace/issue193.spthy, one exists-trace lemma
//! `debug`) plus the four staged captures in `round5/` (`del_path.json`,
//! `del_path_bad.json`, `verify.json`, `verify_proof.json`). A `FakeProver` supplies
//! the opaque prover fragments (lemma-existence model + a canned help pane); the
//! tests assert the WEB-LAYER decisions around them: route match (404) vs method
//! (405), envelope shape, redirect target, version allocation, and alert selection.

use web_clean::dispatch::{Content, MainReq, Meta, ProverOps, Request, RootMeta, Server};
use web_clean::page::Origin;
use web_clean::route::{Autoprove, AutoproveAll, AutoproveDiff, NavDir, Route, ThyPath};

// ---------------------------------------------------------------------------
// A fake prover that models lemma existence (the one datum del/path + verify need)
// and returns a canned help pane. `del`/`verify` never inspect proof internals.
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Thy {
    name: String,
    lemmas: Vec<String>,
}

struct Fake;

fn trace_base() -> Thy {
    Thy { name: "RevealingSignatures".into(), lemmas: vec!["debug".into()] }
}
fn equiv_base() -> Thy {
    Thy { name: "KCL07_UK1_attack_manual".into(), lemmas: vec!["Observational_equivalence".into()] }
}

// The help content the FakeProver returns for MainReq::Help — the exact fragment
// is a prover concern; verify must return *this same* envelope for every
// non-redirecting path (that is the web-layer claim under test).
const HELP_HTML: &str = "<p>Theory: RevealingSignatures</p><div id=\"help\">…</div>";

impl ProverOps for Fake {
    type Theory = Thy;

    fn meta(&self, thy: &Thy) -> Meta {
        Meta {
            name: thy.name.clone(),
            version: "1.13.0".into(),
            filename: format!("{}.spthy", thy.name),
            origin: Origin::Local,
        }
    }
    fn root_meta(&self, thy: &Thy) -> RootMeta {
        RootMeta { time: "00:00:00".into(), origin: format!("{}.spthy", thy.name), modified: false }
    }
    fn source_text(&self, thy: &Thy) -> String {
        format!("theory {} begin\nend", thy.name)
    }
    fn west_pane(&self, _thy: &Thy, index: u64) -> String {
        format!("WEST@{index}")
    }
    fn main_content(&self, thy: &Thy, _index: u64, req: &MainReq) -> Content {
        match req {
            // verify's non-redirect branch renders the help pane; assert its exact
            // envelope round-trips through the web layer unchanged.
            MainReq::Help => Content { html: HELP_HTML.to_string(), title: format!("Theory: {}", thy.name) },
            _ => Content { html: "<p>other</p>".to_string(), title: "other".to_string() },
        }
    }
    fn lemma_source(&self, _thy: &Thy, _name: &str) -> Option<String> {
        Some("lemma debug: exists-trace \"T\"".into())
    }
    fn graph_dot(&self, _thy: &Thy, _tail: &[String]) -> String {
        web_clean::intdot::EMPTY_GRAPH_DOT.to_string()
    }
    fn nav_target(&self, _thy: &Thy, index: u64, _dir: NavDir, _mode: &str, lemma: &str) -> String {
        format!("/thy/trace/{index}/main/proof/{lemma}")
    }
    fn append_message(&self, _thy: &Thy) -> String {
        "Appended lemmas to /tmp/x/issue193.spthy".into()
    }
    fn static_file(&self, _path: &[String]) -> Option<Vec<u8>> {
        None
    }
    fn load_theory(&self, source: &str) -> Option<Thy> {
        source.contains("theory").then(|| Thy { name: "NSLPK3".into(), lemmas: vec![] })
    }
    fn reload(&self, thy: &Thy) -> Thy {
        thy.clone()
    }
    fn apply_method(&self, thy: &Thy, _l: &str, n: usize, _p: &[String]) -> Option<(Thy, Vec<String>)> {
        (n != 0).then(|| (thy.clone(), vec!["_".to_string()]))
    }
    fn autoprove(&self, thy: &Thy, _s: &Autoprove) -> (Thy, Vec<String>) {
        (thy.clone(), vec!["_".to_string()])
    }
    fn edit_lemma(&self, thy: &Thy, _n: &str, _t: &str) -> Option<Thy> {
        Some(thy.clone())
    }
    fn add_lemma(&self, thy: &Thy, _p: &str, _t: &str) -> Option<Thy> {
        Some(thy.clone())
    }
    fn delete_lemma(&self, thy: &Thy, name: &str) -> Option<Thy> {
        thy.lemmas.iter().any(|l| l == name).then(|| thy.clone())
    }
    fn apply_diff_method(&self, thy: &Thy, _l: &str, n: usize, _p: &[String]) -> Option<(Thy, Vec<String>)> {
        (n != 0).then(|| (thy.clone(), vec!["Rule".to_string()]))
    }
    fn autoprove_diff(&self, thy: &Thy, _s: &AutoproveDiff) -> (Thy, Vec<String>) {
        (thy.clone(), vec!["Rule".to_string()])
    }
    fn autoprove_all(&self, thy: &Thy, _s: &AutoproveAll) -> Thy {
        thy.clone()
    }
    // ---- round-5 additions: del/path + verify ----
    fn lemma_present(&self, thy: &Thy, lemma: &str) -> bool {
        thy.lemmas.iter().any(|l| l == lemma)
    }
    fn del_lemma_path(&self, thy: &Thy, name: &str) -> Option<Thy> {
        thy.lemmas.iter().any(|l| l == name).then(|| thy.clone())
    }
    fn del_proof_step(&self, thy: &Thy, lemma: &str, _path: &[String], _diff: bool) -> Option<Thy> {
        thy.lemmas.iter().any(|l| l == lemma).then(|| thy.clone())
    }
}

fn trace_server() -> Server<Fake> {
    Server::new(Fake, trace_base())
}
fn equiv_server() -> Server<Fake> {
    Server::new(Fake, equiv_base())
}

fn body(s: &Server<Fake>, path: &str) -> (u16, &'static str, String) {
    let r = s.dispatch(&Request::get(path));
    (r.status, r.content_type, r.body)
}

// ---------------------------------------------------------------------------
// verify — the four staged captures + the redirect-vs-content rule.
// ---------------------------------------------------------------------------

#[test]
fn verify_lemma_returns_help_pane_no_version_bump() {
    // round5/verify.json: verify/lemma/<name> -> the help content envelope.
    let s = trace_server();
    let (status, ct, b) = body(&s, "/thy/trace/1/verify/lemma/debug");
    assert_eq!(status, 200);
    assert_eq!(ct, "application/json; charset=utf-8");
    assert_eq!(
        b,
        r#"{"html":"<p>Theory: RevealingSignatures</p><div id=\"help\">…</div>","title":"Theory: RevealingSignatures"}"#
    );
    // verify is a read: no version allocated.
    assert_eq!(s.versions(), vec![1]);
}

#[test]
fn verify_proof_of_existing_lemma_redirects_same_version() {
    // round5/verify_proof.json: verify/proof/<lemma> -> {redirect} at the SAME idx.
    let s = trace_server();
    let (status, ct, b) = body(&s, "/thy/trace/1/verify/proof/debug");
    assert_eq!(status, 200);
    assert_eq!(ct, "application/json; charset=utf-8");
    assert_eq!(b, r#"{"redirect":"/thy/trace/1/overview/proof/debug"}"#);
    assert_eq!(s.versions(), vec![1]); // no bump
    // The redirect target is overview/ + the verbatim proof path.
    let deep = body(&s, "/thy/trace/1/verify/proof/debug/_/ONE/ONE").2;
    assert_eq!(deep, r#"{"redirect":"/thy/trace/1/overview/proof/debug/_/ONE/ONE"}"#);
    // A bogus sub-node of a REAL lemma still redirects (predicate = lemma existence).
    let bogus = body(&s, "/thy/trace/1/verify/proof/debug/BOGUS").2;
    assert_eq!(bogus, r#"{"redirect":"/thy/trace/1/overview/proof/debug/BOGUS"}"#);
}

#[test]
fn verify_proof_of_absent_lemma_falls_back_to_help() {
    let s = trace_server();
    // proof/<nonexistent-lemma> -> the help pane, not a redirect.
    let b = body(&s, "/thy/trace/1/verify/proof/nope").2;
    assert!(b.starts_with(r#"{"html":"#), "{b}");
    assert!(b.contains(r#""title":"Theory: RevealingSignatures""#), "{b}");
}

#[test]
fn verify_nonproof_paths_all_return_the_same_help_envelope() {
    // Every non-proof theory path returns the identical help envelope (== main/help).
    let s = trace_server();
    let help = body(&s, "/thy/trace/1/verify/lemma/debug").2;
    for p in ["help", "message", "rules", "tactic", "cases/raw/0/0", "method/debug/1", "add/debug", "edit/debug", "delete/debug"] {
        let b = body(&s, &format!("/thy/trace/1/verify/{p}")).2;
        assert_eq!(b, help, "verify/{p} should equal the help envelope");
    }
}

#[test]
fn verify_method_and_parse_ordering() {
    let s = trace_server();
    // GET-only: a parseable path with POST -> 405; an unparseable tail -> 404 (any method).
    assert_eq!(s.dispatch(&Request::post("/thy/trace/1/verify/proof/debug", &[])).status, 405);
    assert_eq!(s.dispatch(&Request::post("/thy/trace/1/verify/sources", &[])).status, 404);
    // Unparseable / bare tails -> 404.
    assert_eq!(s.dispatch(&Request::get("/thy/trace/1/verify")).status, 404);
    assert_eq!(s.dispatch(&Request::get("/thy/trace/1/verify/sources")).status, 404);
    assert_eq!(s.dispatch(&Request::get("/thy/trace/1/verify/x")).status, 404);
}

#[test]
fn verify_is_absent_in_equiv_mode() {
    // The verify route is not registered for equiv theories: 404 for any method/path.
    let s = equiv_server();
    assert_eq!(s.dispatch(&Request::get("/thy/equiv/1/verify/diffProof/Observational_equivalence")).status, 404);
    assert_eq!(s.dispatch(&Request::get("/thy/equiv/1/verify/help")).status, 404);
    assert_eq!(s.dispatch(&Request::post("/thy/equiv/1/verify/diffProof/Observational_equivalence", &[])).status, 404);
}

// ---------------------------------------------------------------------------
// del/path — the two staged captures + deletability / alert selection.
// ---------------------------------------------------------------------------

#[test]
fn del_path_lemma_redirects_to_overview_lemma_new_version() {
    // round5/del_path.json: del/path/lemma/<name> -> {redirect} to overview/lemma/<name>
    // at a FRESH version (base retained). Fresh server -> new index 2.
    let s = trace_server();
    let (status, ct, b) = body(&s, "/thy/trace/1/del/path/lemma/debug");
    assert_eq!(status, 200);
    assert_eq!(ct, "application/json; charset=utf-8");
    assert_eq!(b, r#"{"redirect":"/thy/trace/2/overview/lemma/debug"}"#);
    assert_eq!(s.versions(), vec![1, 2]); // new version; base kept
}

#[test]
fn del_path_undeletable_path_alerts_no_bump() {
    // round5/del_path_bad.json: del/path/rules -> the "Can't delete" alert, no bump.
    let s = trace_server();
    let b = body(&s, "/thy/trace/1/del/path/rules").2;
    assert_eq!(b, r#"{"alert":"Can't delete the given theory path!"}"#);
    // Every non-lemma / non-proof theory path gets the same alert.
    for p in ["help", "message", "tactic", "cases/raw/0/0", "method/debug/1", "add/debug", "edit/debug", "delete/debug"] {
        assert_eq!(body(&s, &format!("/thy/trace/1/del/path/{p}")).2, b, "del/path/{p}");
    }
    assert_eq!(s.versions(), vec![1]); // none of these mutate
}

#[test]
fn del_path_proof_redirect_and_verbatim_target() {
    let s = trace_server();
    let b = body(&s, "/thy/trace/1/del/path/proof/debug").2;
    assert_eq!(b, r#"{"redirect":"/thy/trace/2/overview/proof/debug"}"#);
    assert_eq!(s.versions(), vec![1, 2]);
    // A deeper proof path redirects to overview/ + the verbatim path at a new version.
    let deep = body(&s, "/thy/trace/1/del/path/proof/debug/_/ONE").2;
    assert_eq!(deep, r#"{"redirect":"/thy/trace/3/overview/proof/debug/_/ONE"}"#);
    assert_eq!(s.versions(), vec![1, 2, 3]);
}

#[test]
fn del_path_failure_alerts_selected_by_path_type() {
    let s = trace_server();
    // lemma/<absent> -> the lemma-removal alert; no bump.
    assert_eq!(
        body(&s, "/thy/trace/1/del/path/lemma/nope").2,
        r#"{"alert":"Sorry, but removing the selected lemma failed!"}"#
    );
    // proof/<absent lemma> -> the proof-step-removal alert; no bump.
    assert_eq!(
        body(&s, "/thy/trace/1/del/path/proof/nope").2,
        r#"{"alert":"Sorry, but removing the selected proof step failed!"}"#
    );
    assert_eq!(s.versions(), vec![1]);
}

#[test]
fn del_path_method_and_parse_ordering() {
    let s = trace_server();
    // Parseable path + POST -> 405 (registration proof, overturns round-4 [R47]).
    assert_eq!(s.dispatch(&Request::post("/thy/trace/1/del/path/lemma/debug", &[])).status, 405);
    // Unparseable tail + POST -> 404 (route miss precedes method dispatch).
    assert_eq!(s.dispatch(&Request::post("/thy/trace/1/del/path/sources", &[])).status, 404);
    // GET on unparseable / malformed tails -> 404, echoing the full request path.
    let nf = s.dispatch(&Request::get("/thy/trace/1/del/path/x"));
    assert_eq!(nf.status, 404);
    assert!(nf.body.contains("<p>/thy/trace/1/del/path/x</p>"), "{}", nf.body);
    assert_eq!(s.dispatch(&Request::get("/thy/trace/1/del/path")).status, 404);
    // `del` without the fixed `path` literal is not a del/path route.
    assert_eq!(s.dispatch(&Request::get("/thy/trace/1/del/lemma/debug")).status, 404);
    // No mutation occurred on any 404/405.
    assert_eq!(s.versions(), vec![1]);
}

// ---------------------------------------------------------------------------
// del/path in equiv mode — diffProof grammar; proof/rules do not parse.
// ---------------------------------------------------------------------------

#[test]
fn del_path_equiv_uses_diffproof_grammar() {
    let s = equiv_server();
    // Deletable diff proof node -> redirect to overview/diffProof/<lemma> new version.
    let b = body(&s, "/thy/equiv/1/del/path/diffProof/Observational_equivalence").2;
    assert_eq!(b, r#"{"redirect":"/thy/equiv/2/overview/diffProof/Observational_equivalence"}"#);
    assert_eq!(s.versions(), vec![1, 2]);
    // diffProof/<absent> -> the proof-step alert; diffrules -> the "Can't delete" alert.
    assert_eq!(
        body(&s, "/thy/equiv/1/del/path/diffProof/nope").2,
        r#"{"alert":"Sorry, but removing the selected proof step failed!"}"#
    );
    assert_eq!(
        body(&s, "/thy/equiv/1/del/path/diffrules").2,
        r#"{"alert":"Can't delete the given theory path!"}"#
    );
    // Trace-grammar heads do not parse in equiv -> 404 (route miss for any method).
    for p in ["rules", "message", "proof/Observational_equivalence", "lemma/Observational_equivalence", "cases/raw/0/0"] {
        assert_eq!(s.dispatch(&Request::get(&format!("/thy/equiv/1/del/path/{p}"))).status, 404, "equiv del/path/{p}");
        assert_eq!(s.dispatch(&Request::post(&format!("/thy/equiv/1/del/path/{p}"), &[])).status, 404, "equiv POST del/path/{p}");
    }
}

// ---------------------------------------------------------------------------
// Version model reconciliation: del/path (deletable) is a PROOF OP off the same
// global monotonic counter as method/autoprove, not an in-place structural edit.
// ---------------------------------------------------------------------------

#[test]
fn del_path_allocates_off_the_global_counter_like_proof_ops() {
    let s = trace_server();
    // A proof method bumps 1 -> 2.
    s.dispatch(&Request::get("/thy/trace/1/main/method/debug/1"));
    assert_eq!(s.versions(), vec![1, 2]);
    // del/path on v2 allocates 3 off the same counter, base retained.
    let b = body(&s, "/thy/trace/2/del/path/lemma/debug").2;
    assert_eq!(b, r#"{"redirect":"/thy/trace/3/overview/lemma/debug"}"#);
    assert_eq!(s.versions(), vec![1, 2, 3]);
    // Every earlier version stays resolvable.
    assert_eq!(s.dispatch(&Request::get("/thy/trace/1/overview/help")).status, 200);
    assert_eq!(s.dispatch(&Request::get("/thy/trace/2/overview/help")).status, 200);
}

// ---------------------------------------------------------------------------
// Route grammar unit coverage for the new handlers via the public parser.
// ---------------------------------------------------------------------------

#[test]
fn route_and_thypath_public_api() {
    use web_clean::route::Handler;
    let r = Route::parse("/thy/trace/1/del/path/proof/debug/_").unwrap();
    assert_eq!(r.handler, Handler::DelPath(vec!["proof".into(), "debug".into(), "_".into()]));
    // ThyPath parse is reachable and mode-aware.
    let segs: Vec<String> = vec!["proof".into(), "debug".into()];
    assert_eq!(
        ThyPath::parse(&segs, false),
        Some(ThyPath::Proof { lemma: "debug".into(), path: vec![] })
    );
    assert_eq!(ThyPath::parse(&segs, true), None); // `proof` is not an equiv path
}
