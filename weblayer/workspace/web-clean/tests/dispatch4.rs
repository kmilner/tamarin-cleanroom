//! Round-4 byte-parity / behaviour tests for the full request surface: the
//! top-level routes (`/`, `/robots.txt`, `/favicon.ico`, `/kill`, `/static/**`),
//! the theory-scoped `reload` / `download` / `get_and_append`, the diff (equiv)
//! shell and proof ops, and the refined version lifecycle (upload / reload /
//! delete-not-found / method-failure).
//!
//! Every asserted string traces to a live oracle observation captured under
//! `tests/fixtures/r4_*` (see `QUERIES.log` round 4). A `FakeProver` supplies the
//! opaque prover fragments; the tests assert the web-layer decisions around them.

use web_clean::dispatch::{
    Content, HttpMethod, MainReq, Meta, ProverOps, Request, RootMeta, Server,
};
use web_clean::page::{render_page_kind, render_root, Flash, PageParams, RootRow, ShellKind};
use web_clean::route::{Autoprove, AutoproveAll, AutoproveDiff, NavDir, Toplevel};

// ---------------------------------------------------------------------------
// A fake prover that models just enough for the transport decisions under test.
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Thy {
    name: String,
    lemmas: Vec<String>,
}

struct Fake;

fn base() -> Thy {
    Thy { name: "Tutorial".into(), lemmas: vec!["types".into(), "secrecy".into()] }
}

impl ProverOps for Fake {
    type Theory = Thy;

    fn meta(&self, thy: &Thy) -> Meta {
        Meta { name: thy.name.clone(), version: "1.13.0".into(), filename: format!("{}.spthy", thy.name) }
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
    fn main_content(&self, _thy: &Thy, index: u64, req: &MainReq) -> Content {
        let (k, t) = match req {
            MainReq::Help => ("help", "Help".to_string()),
            MainReq::Message => ("message", "Message".to_string()),
            MainReq::Rules => ("rules", "Rules".to_string()),
            MainReq::Tactic => ("tactic", "Tactic".to_string()),
            MainReq::Cases { .. } => ("cases", "Cases".to_string()),
            MainReq::Lemma(l) => ("lemma", format!("Lemma: {l}")),
            MainReq::Proof { lemma, .. } => ("proof", format!("Lemma: {lemma}")),
            MainReq::DiffProof { lemma, .. } => ("diffProof", format!("Lemma: {lemma}")),
            MainReq::DiffRules => ("diffrules", "Diff rules".to_string()),
        };
        Content { html: format!("<p>{k}@{index}</p>"), title: t }
    }
    fn lemma_source(&self, _thy: &Thy, name: &str) -> Option<String> {
        Some(format!("lemma {name}: exists-trace \"T\""))
    }
    fn graph_dot(&self, _thy: &Thy, _tail: &[String]) -> String {
        web_clean::intdot::EMPTY_GRAPH_DOT.to_string()
    }
    fn nav_target(&self, _thy: &Thy, index: u64, _dir: NavDir, _mode: &str, lemma: &str) -> String {
        format!("/thy/trace/{index}/main/proof/{lemma}")
    }
    fn append_message(&self, _thy: &Thy) -> String {
        "Appended lemmas to /tmp/x/Tutorial.spthy".into()
    }
    fn static_file(&self, path: &[String]) -> Option<Vec<u8>> {
        (path == ["css".to_string(), "app.css".to_string()]).then(|| b"body{}".to_vec())
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
    fn edit_lemma(&self, thy: &Thy, _n: &str, text: &str) -> Option<Thy> {
        text.trim_start().starts_with("lemma").then(|| thy.clone())
    }
    fn add_lemma(&self, thy: &Thy, _p: &str, text: &str) -> Option<Thy> {
        text.trim_start().starts_with("lemma").then(|| thy.clone())
    }
    fn delete_lemma(&self, thy: &Thy, name: &str) -> Option<Thy> {
        thy.lemmas.iter().any(|l| l == name).then(|| {
            let mut t = thy.clone();
            t.lemmas.retain(|l| l != name);
            t
        })
    }
    fn lemma_present(&self, thy: &Thy, lemma: &str) -> bool {
        thy.lemmas.iter().any(|l| l == lemma)
    }
    fn del_lemma_path(&self, thy: &Thy, name: &str) -> Option<Thy> {
        thy.lemmas.iter().any(|l| l == name).then(|| thy.clone())
    }
    fn del_proof_step(&self, thy: &Thy, lemma: &str, _path: &[String], _diff: bool) -> Option<Thy> {
        thy.lemmas.iter().any(|l| l == lemma).then(|| thy.clone())
    }
    fn apply_diff_method(&self, thy: &Thy, _l: &str, n: usize, _p: &[String]) -> Option<(Thy, Vec<String>)> {
        (n != 0).then(|| (thy.clone(), vec!["Rule_Destrd_0_fst".to_string()]))
    }
    fn autoprove_diff(&self, thy: &Thy, _s: &AutoproveDiff) -> (Thy, Vec<String>) {
        (thy.clone(), vec!["Rule_Destrd_0_fst".to_string()])
    }
    fn autoprove_all(&self, thy: &Thy, _s: &AutoproveAll) -> Thy {
        thy.clone()
    }
}

fn server() -> Server<Fake> {
    Server::new(Fake, base())
}

// ---------------------------------------------------------------------------
// Top-level routes: robots / favicon / kill / static.
// ---------------------------------------------------------------------------

#[test]
fn robots_txt_byte_identical() {
    let mut s = server();
    let r = s.dispatch(&Request::get("/robots.txt"));
    assert_eq!(r.status, 200);
    assert_eq!(r.content_type, "text/plain; charset=utf-8");
    assert_eq!(r.body, include_str!("fixtures/r4_robots.txt"));
    // POST /robots.txt -> 405 Method Not Supported.
    let p = s.dispatch(&Request::post("/robots.txt", &[]));
    assert_eq!(p.status, 405);
    assert_eq!(p.body, include_str!("fixtures/r4_badmethod_post_kill.html"));
}

#[test]
fn favicon_is_303_to_static_icon_with_no_cache() {
    let mut s = server();
    let r = s.dispatch(&Request::get("/favicon.ico"));
    assert_eq!(r.status, 303);
    assert_eq!(r.location.as_deref(), Some("/static/img/favicon.ico"));
    assert_eq!(r.body, "");
    assert!(r.no_cache);
}

#[test]
fn kill_requires_path_query() {
    let mut s = server();
    // No path arg -> 400 Invalid Arguments (byte-identical to the live page).
    let bad = s.dispatch(&Request::get("/kill"));
    assert_eq!(bad.status, 400);
    assert_eq!(bad.content_type, "text/html; charset=utf-8");
    assert_eq!(bad.body, include_str!("fixtures/r4_invalidargs_kill.html"));
    // With a path arg -> 200 text/plain "Canceled request!".
    let q = vec![("path".to_string(), "solve(x)".to_string())];
    let ok = s.dispatch(&Request::get_query("/kill", &q));
    assert_eq!(ok.status, 200);
    assert_eq!(ok.content_type, "text/plain; charset=utf-8");
    assert_eq!(ok.body, include_str!("fixtures/r4_kill_canceled.txt"));
    // POST /kill -> 405 (byte-identical to the live Method-Not-Supported page).
    let post = s.dispatch(&Request::post("/kill", &[]));
    assert_eq!(post.status, 405);
    assert_eq!(post.body, include_str!("fixtures/r4_badmethod_post_kill.html"));
}

#[test]
fn static_serves_by_extension_and_404s_missing() {
    let mut s = server();
    let hit = s.dispatch(&Request::get("/static/css/app.css"));
    assert_eq!(hit.status, 200);
    assert_eq!(hit.content_type, "text/css");
    assert_eq!(hit.body, "body{}");
    let miss = s.dispatch(&Request::get("/static/css/nope.css"));
    assert_eq!(miss.status, 404);
    assert_eq!(miss.content_type, "text/plain; charset=utf-8");
    assert_eq!(miss.body, "File not found");
    // POST /static/... -> 405.
    let post = s.dispatch(&Request::post("/static/css/app.css", &[]));
    assert_eq!(post.status, 405);
}

// ---------------------------------------------------------------------------
// download / reload / get_and_append.
// ---------------------------------------------------------------------------

#[test]
fn download_is_octet_stream_equal_to_source() {
    let mut s = server();
    let dl = s.dispatch(&Request::get("/thy/trace/1/download/Tutorial.spthy"));
    assert_eq!(dl.status, 200);
    assert_eq!(dl.content_type, "application/octet-stream");
    assert!(dl.location.is_none());
    let src = s.dispatch(&Request::get("/thy/trace/1/source"));
    // Same bytes as `source`, only the content type differs.
    assert_eq!(dl.body, src.body);
    // Wrong method -> 405.
    assert_eq!(s.dispatch(&Request::post("/thy/trace/1/download/x", &[])).status, 405);
}

#[test]
fn reload_is_json_redirect_in_place() {
    let mut s = server();
    let r = s.dispatch(&Request::post("/thy/trace/1/reload", &[]));
    assert_eq!(r.status, 200);
    assert_eq!(r.content_type, "application/json; charset=utf-8");
    assert_eq!(r.body, r#"{"redirect":"/thy/trace/1/overview/help"}"#);
    assert_eq!(s.versions(), vec![1]); // no new version
    // GET /reload -> 405.
    assert_eq!(s.dispatch(&Request::get("/thy/trace/1/reload")).status, 405);
}

#[test]
fn get_and_append_is_alert_envelope() {
    let mut s = server();
    let r = s.dispatch(&Request::post("/thy/trace/1/get_and_append/Tutorial.spthy", &[]));
    assert_eq!(r.status, 200);
    assert_eq!(r.content_type, "application/json; charset=utf-8");
    assert_eq!(r.body, r#"{"alert":"Appended lemmas to /tmp/x/Tutorial.spthy"}"#);
    assert_eq!(s.dispatch(&Request::get("/thy/trace/1/get_and_append/x")).status, 405);
}

// ---------------------------------------------------------------------------
// Version lifecycle: upload / delete-not-found / method-failure.
// ---------------------------------------------------------------------------

#[test]
fn upload_allocates_fresh_index_from_global_counter() {
    let mut s = server();
    // First bump the counter via a proof op so an upload cannot collide with it.
    s.dispatch(&Request::get("/thy/trace/1/main/method/types/1"));
    assert_eq!(s.versions(), vec![1, 2]);
    // Upload a new theory: new index = max ever + 1 = 3; index page + Loaded flash.
    let form = vec![("uploadedTheory".to_string(), "theory NSLPK3 begin end".to_string())];
    let up = s.dispatch(&Request::post("/", &form));
    assert_eq!(up.status, 200);
    assert_eq!(up.content_type, "text/html; charset=utf-8");
    assert!(up.body.contains("<p class=\"message\">Loaded new theory!</p>"));
    assert_eq!(s.versions(), vec![1, 2, 3]);
    // A failed upload keeps the version set and shows the failure flash.
    let bad = vec![("uploadedTheory".to_string(), "garbage input".to_string())];
    let fail = s.dispatch(&Request::post("/", &bad));
    assert!(fail.body.contains("<p class=\"message\">Post request failed.</p>"));
    assert_eq!(s.versions(), vec![1, 2, 3]);
    // Plain GET / has no flash paragraph.
    let get = s.dispatch(&Request::get("/"));
    assert!(!get.body.contains("class=\"message\""));
}

#[test]
fn delete_missing_lemma_redirects_to_delete_view() {
    let mut s = server();
    // Existing lemma -> 303 to help, removed in place.
    let ok = s.dispatch(&Request::post("/thy/trace/1/edit/delete/types", &[]));
    assert_eq!(ok.status, 303);
    assert_eq!(ok.location.as_deref(), Some("/thy/trace/1/overview/help"));
    assert!(ok.no_cache);
    // Deleting it again (now absent) -> 303 to the delete view, theory unchanged.
    let miss = s.dispatch(&Request::post("/thy/trace/1/edit/delete/types", &[]));
    assert_eq!(miss.status, 303);
    assert_eq!(miss.location.as_deref(), Some("/thy/trace/1/overview/delete/types"));
    assert_eq!(s.versions(), vec![1]);
}

#[test]
fn failed_method_is_alert_no_version_bump() {
    let mut s = server();
    // Method 0 is the failure case in the fake: a JSON {alert}, no new version.
    let r = s.dispatch(&Request::get("/thy/trace/1/main/method/types/0"));
    assert_eq!(r.status, 200);
    assert_eq!(r.content_type, "application/json; charset=utf-8");
    assert_eq!(r.body, r#"{"alert":"Sorry, but the prover failed on the selected method!"}"#);
    assert_eq!(s.versions(), vec![1]);
}

// ---------------------------------------------------------------------------
// Index page (root) byte-parity via decomposition (canned non-det row values).
// ---------------------------------------------------------------------------

#[test]
fn root_index_page_byte_identical() {
    // Feed the exact captured row (time/origin are non-deterministic inputs).
    let row = RootRow {
        index: 1,
        name: "NSLPK3",
        time: "17:44:37",
        modified: false,
        origin: "/tmp/tmp.RTTW9GQKpy/NSLPK3.spthy",
    };
    let got = render_root(Flash::None, "1.13.0", &[row]);
    assert_eq!(got, include_str!("fixtures/r4_root_single.html"));
}

// ---------------------------------------------------------------------------
// Equiv (diff) overview shell byte-parity via decomposition.
// ---------------------------------------------------------------------------

#[test]
fn equiv_overview_shell_byte_identical() {
    let fixture = include_str!("fixtures/r4_equiv_overview_kcl.html");
    const WEST_OPEN: &str = "<div class=\"monospace\" id=\"proof\">";
    const EAST_OPEN: &str = "</div></div></div><div class=\"ui-layout-east\">";
    const MAIN_OPEN: &str = "<div id=\"ui-main-display\">";
    const TAIL_OPEN: &str = "</div></div></div><div id=\"dialog\"></div>";
    let west_start = fixture.find(WEST_OPEN).unwrap() + WEST_OPEN.len();
    let east_start = fixture.find(EAST_OPEN).unwrap();
    let main_start = fixture.find(MAIN_OPEN).unwrap() + MAIN_OPEN.len();
    let tail_start = fixture.find(TAIL_OPEN).unwrap();
    let west = &fixture[west_start..east_start];
    let center = &fixture[main_start..tail_start];
    let params = PageParams {
        theory_name: "KCL07_UK1_attack_manual",
        index: 1,
        version: "1.13.0",
        filename: "KCL07_UK1_attack_manual.spthy",
    };
    let got = render_page_kind(ShellKind::Equiv, &params, west, center);
    assert_eq!(got, fixture);
}

#[test]
fn diff_method_redirects_to_diffproof_new_version() {
    let mut s = server();
    let r = s.dispatch(&Request::get("/thy/equiv/1/main/diffMethod/Observational_equivalence/1"));
    assert_eq!(r.status, 200);
    assert_eq!(r.content_type, "application/json; charset=utf-8");
    assert_eq!(
        r.body,
        r#"{"redirect":"/thy/equiv/2/overview/diffProof/Observational_equivalence/Rule_Destrd_0_fst"}"#
    );
    assert_eq!(s.versions(), vec![1, 2]);
    // A failed diff method -> alert, no bump.
    let f = s.dispatch(&Request::get("/thy/equiv/1/main/diffMethod/Observational_equivalence/0"));
    assert_eq!(f.body, r#"{"alert":"Sorry, but the prover failed on the selected method!"}"#);
    assert_eq!(s.versions(), vec![1, 2]);
}

#[test]
fn equiv_intdot_uses_equiv_kind_in_dotsrc() {
    let mut s = server();
    let r = s.dispatch(&Request::get("/thy/equiv/1/intdot/graph/diffProof/Observational_equivalence/Rule_Send"));
    assert_eq!(r.content_type, "text/html; charset=utf-8");
    assert!(r.body.contains(
        "dotsrc=\"/thy/equiv/1/interactive-graph-def/graph/diffProof/Observational_equivalence/Rule_Send\""
    ), "{}", r.body);
}

// ---------------------------------------------------------------------------
// Toplevel route grammar.
// ---------------------------------------------------------------------------

#[test]
fn toplevel_route_grammar() {
    assert_eq!(Toplevel::parse("/"), Toplevel::Root);
    assert_eq!(Toplevel::parse("/robots.txt"), Toplevel::Robots);
    assert_eq!(Toplevel::parse("/favicon.ico"), Toplevel::Favicon);
    assert_eq!(Toplevel::parse("/kill"), Toplevel::Kill);
    assert_eq!(
        Toplevel::parse("/static/css/x.css"),
        Toplevel::Static(vec!["css".into(), "x.css".into()])
    );
    assert!(matches!(Toplevel::parse("/thy/trace/1/overview/help"), Toplevel::Thy(_)));
    assert!(matches!(Toplevel::parse("/thy/equiv/1/main/diffrules"), Toplevel::Thy(_)));
    assert!(matches!(Toplevel::parse("/nonsense"), Toplevel::Other(_)));
    // `robots.txt`/`kill` only match at their bare paths, not as prefixes.
    assert!(matches!(Toplevel::parse("/kill/extra"), Toplevel::Other(_)));
}

#[test]
fn unknown_index_and_current_are_404() {
    let mut s = server();
    assert_eq!(s.dispatch(&Request::get("/thy/trace/99/overview/help")).status, 404);
    assert_eq!(s.dispatch(&Request::get("/thy/trace/#/overview/help")).status, 404);
}

// ---------------------------------------------------------------------------
// HttpMethod is a plain enum used by the harness.
// ---------------------------------------------------------------------------

#[test]
fn method_enum() {
    assert_ne!(HttpMethod::Get, HttpMethod::Post);
}
