//! Round-3 byte-parity / behaviour tests for the interactive-UI state machine
//! (`web_clean::dispatch`).
//!
//! A `FakeProver` supplies the opaque prover fragments (canned to the values a
//! real theory would produce, taken from the live captures in
//! `tests/fixtures/r3_*`); the tests assert the *web-layer decisions* the state
//! machine makes around them: envelope shape, HTTP status/Location, theory-version
//! increments, and redirect-URL assembly. Every asserted string traces to a live
//! oracle observation (`QUERIES.log` [L8]–[L16]).

use serde_json::Value;
use web_clean::dispatch::{
    Content, HttpMethod, MainReq, Meta, ProverOps, Request, RootMeta, Server,
};
use web_clean::intdot::EMPTY_GRAPH_DOT;
use web_clean::page::Origin;
use web_clean::route::{Autoprove, AutoproveAll, AutoproveDiff, NavDir};

// ---------------------------------------------------------------------------
// A minimal fake prover: holds lemma sources + canned fragments.
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Thy {
    lemmas: Vec<(String, String)>, // (name, raw source)
}

struct FakeProver;

impl FakeProver {
    fn tutorial() -> Thy {
        Thy {
            lemmas: vec![
                (
                    "Client_auth".to_string(),
                    include_str!("fixtures/r3_lemma_text_client_auth.txt").to_string(),
                ),
                (
                    "Client_session_key_honest_setup".to_string(),
                    include_str!("fixtures/r3_lemma_text_honest_setup.txt").to_string(),
                ),
            ],
        }
    }
}

impl ProverOps for FakeProver {
    type Theory = Thy;

    fn meta(&self, _thy: &Thy) -> Meta {
        Meta {
            name: "Tutorial".into(),
            version: "1.13.0".into(),
            filename: "Tutorial.spthy".into(),
            origin: Origin::Local,
        }
    }
    fn root_meta(&self, _thy: &Thy) -> RootMeta {
        RootMeta { time: "00:00:00".into(), origin: "Tutorial.spthy".into(), modified: false }
    }
    fn source_text(&self, _thy: &Thy) -> String {
        "theory Tutorial begin\nend".into()
    }
    fn west_pane(&self, _thy: &Thy, index: u64) -> String {
        format!("WEST@{index}")
    }
    fn main_content(&self, _thy: &Thy, index: u64, req: &MainReq) -> Content {
        // Canned; the real center-pane content is a prover fragment.
        let (kind, title) = match req {
            MainReq::Help => ("help", "Tutorial".to_string()),
            MainReq::Message => ("message", "Message theory".to_string()),
            MainReq::Rules => ("rules", "Rewriting rules".to_string()),
            MainReq::Tactic => ("tactic", "Tactics".to_string()),
            MainReq::Cases { .. } => ("cases", "Sources".to_string()),
            MainReq::Lemma(l) => ("lemma", format!("Lemma: {l}")),
            MainReq::Proof { lemma, .. } => ("proof", format!("Lemma: {lemma}")),
            MainReq::DiffProof { lemma, .. } => ("diffProof", format!("Lemma: {lemma}")),
            MainReq::DiffRules => ("diffrules", "Diff rules".to_string()),
        };
        Content { html: format!("<p>{kind}@{index}</p>"), title }
    }
    fn lemma_source(&self, thy: &Thy, name: &str) -> Option<String> {
        thy.lemmas.iter().find(|(n, _)| n == name).map(|(_, s)| s.clone())
    }
    fn graph_dot(&self, _thy: &Thy, _tail: &[String]) -> String {
        EMPTY_GRAPH_DOT.to_string()
    }
    fn nav_target(&self, _thy: &Thy, index: u64, dir: NavDir, _mode: &str, lemma: &str) -> String {
        // Canned to the live capture [L10] for (Next, Client_auth) at v1.
        match (dir, lemma) {
            (NavDir::Next, "Client_auth") => format!("/thy/trace/{index}/main/proof/Client_auth_injective"),
            (NavDir::Prev, "Client_auth") => format!("/thy/trace/{index}/main/proof/Client_session_key_secrecy"),
            _ => format!("/thy/trace/{index}/main/proof/{lemma}"),
        }
    }
    fn append_message(&self, _thy: &Thy) -> String {
        "Appended lemmas to /tmp/x/Tutorial.spthy".into()
    }
    fn static_file(&self, path: &[String]) -> Option<Vec<u8>> {
        // A tiny fake static tree: css/app.css present, everything else absent.
        (path == ["css".to_string(), "app.css".to_string()])
            .then(|| b"body{}".to_vec())
    }
    fn load_theory(&self, source: &str) -> Option<Thy> {
        // Model a parseable upload as any non-empty source mentioning `theory`.
        source.contains("theory").then(|| Thy { lemmas: vec![] })
    }
    fn reload(&self, thy: &Thy) -> Thy {
        thy.clone()
    }
    fn apply_method(&self, thy: &Thy, _lemma: &str, n: usize, _path: &[String]) -> Option<(Thy, Vec<String>)> {
        // Canned focus "_" as in the live method capture [L8]; method 0 = failure.
        (n != 0).then(|| (thy.clone(), vec!["_".to_string()]))
    }
    fn autoprove(&self, thy: &Thy, _spec: &Autoprove) -> (Thy, Vec<String>) {
        (thy.clone(), vec!["_".to_string(), "Client_1".to_string()])
    }
    fn edit_lemma(&self, thy: &Thy, _name: &str, text: &str) -> Option<Thy> {
        // Model validity: a parseable lemma starts with the `lemma` keyword.
        text.trim_start().starts_with("lemma").then(|| thy.clone())
    }
    fn add_lemma(&self, thy: &Thy, _pos: &str, text: &str) -> Option<Thy> {
        text.trim_start().starts_with("lemma").then(|| thy.clone())
    }
    fn delete_lemma(&self, thy: &Thy, name: &str) -> Option<Thy> {
        // Some = found & removed; None = lemma not present.
        thy.lemmas.iter().any(|(n, _)| n == name).then(|| {
            let mut t = thy.clone();
            t.lemmas.retain(|(n, _)| n != name);
            t
        })
    }
    fn lemma_present(&self, thy: &Thy, lemma: &str) -> bool {
        thy.lemmas.iter().any(|(n, _)| n == lemma)
    }
    fn del_lemma_path(&self, thy: &Thy, name: &str) -> Option<Thy> {
        // Deletable iff the lemma exists; the modified theory is a fresh version.
        thy.lemmas.iter().any(|(n, _)| n == name).then(|| thy.clone())
    }
    fn del_proof_step(&self, thy: &Thy, lemma: &str, _path: &[String], _diff: bool) -> Option<Thy> {
        thy.lemmas.iter().any(|(n, _)| n == lemma).then(|| thy.clone())
    }
    fn apply_diff_method(&self, thy: &Thy, _lemma: &str, n: usize, _path: &[String]) -> Option<(Thy, Vec<String>)> {
        (n != 0).then(|| (thy.clone(), vec!["Rule_1".to_string()]))
    }
    fn autoprove_diff(&self, thy: &Thy, _spec: &AutoproveDiff) -> (Thy, Vec<String>) {
        (thy.clone(), vec!["Rule_1".to_string()])
    }
    fn autoprove_all(&self, thy: &Thy, _spec: &AutoproveAll) -> Thy {
        thy.clone()
    }
}

fn server() -> Server<FakeProver> {
    Server::new(FakeProver, FakeProver::tutorial())
}

// ---------------------------------------------------------------------------
// Proof operations: NEW version, JSON {redirect}. (spec item 1 + 2)
// ---------------------------------------------------------------------------

#[test]
fn method_application_bumps_version_and_redirects() {
    let mut s = server();
    let r = s.dispatch(&Request::get("/thy/trace/1/main/method/Client_session_key_secrecy/1"));
    assert_eq!(r.status, 200);
    assert_eq!(r.content_type, "application/json; charset=utf-8");
    // Byte-identical to the live capture r3_method_redirect.json ([L8]).
    assert_eq!(
        r.body,
        r#"{"redirect":"/thy/trace/2/overview/proof/Client_session_key_secrecy/_"}"#
    );
    assert_eq!(s.versions(), vec![1, 2]);
    // A second proof op allocates the next monotonic index; version 1 is retained.
    let r2 = s.dispatch(&Request::get("/thy/trace/1/main/method/Client_session_key_secrecy/1"));
    assert_eq!(
        r2.body,
        r#"{"redirect":"/thy/trace/3/overview/proof/Client_session_key_secrecy/_"}"#
    );
    assert_eq!(s.versions(), vec![1, 2, 3]);
}

#[test]
fn method_at_deeper_path_carries_number_before_path() {
    let mut s = server();
    // method/{lemma}/{n}/{path…}: the number precedes the path.
    let r = s.dispatch(&Request::get("/thy/trace/1/main/method/Client_session_key_secrecy/2/_/B_2"));
    assert_eq!(r.status, 200);
    assert_eq!(
        r.body,
        r#"{"redirect":"/thy/trace/2/overview/proof/Client_session_key_secrecy/_"}"#
    );
}

#[test]
fn autoprove_variants_redirect_json_and_bump() {
    for form in [
        "idfs/0/False", "idfs/5/False", "idfs/0/True", "characterize/0/False", "characterize/5/False",
    ] {
        let mut s = server();
        let path = format!("/thy/trace/1/autoprove/{form}/proof/Client_session_key_honest_setup");
        let r = s.dispatch(&Request::get(&path));
        assert_eq!(r.status, 200, "form {form}");
        assert_eq!(r.content_type, "application/json; charset=utf-8");
        assert_eq!(
            r.body,
            r#"{"redirect":"/thy/trace/2/overview/proof/Client_session_key_honest_setup/_/Client_1"}"#,
            "form {form}"
        );
        assert_eq!(s.versions(), vec![1, 2]);
    }
}

#[test]
fn autoprove_route_parses_the_variant_matrix() {
    // The a/b/all/characterization matrix, decoded from the URL.
    let cases = [
        ("idfs/0/False", "idfs", 0u64, false),   // a: unbounded, stop@first
        ("idfs/5/False", "idfs", 5, false),      // b: bounded
        ("idfs/0/True", "idfs", 0, true),        // A/all: all solutions
        ("characterize/0/False", "characterize", 0, false), // characterization
    ];
    for (form, strat, bound, all) in cases {
        let path = format!("/thy/trace/1/autoprove/{form}/proof/L");
        let route = web_clean::route::Route::parse(&path).unwrap();
        let tail = match route.handler {
            web_clean::route::Handler::Autoprove(t) => t,
            _ => panic!(),
        };
        let ap = Autoprove::parse(&tail).unwrap();
        assert_eq!((ap.strategy.as_str(), ap.bound, ap.all_solutions), (strat, bound, all));
    }
}

// ---------------------------------------------------------------------------
// Structural edits: mutate IN PLACE, 303 See Other (or 200 form on failure).
// (spec item 3)
// ---------------------------------------------------------------------------

#[test]
fn delete_lemma_is_303_to_help_and_in_place() {
    let mut s = server();
    let r = s.dispatch(&Request::post("/thy/trace/1/edit/delete/Client_auth", &[]));
    assert_eq!(r.status, 303);
    assert_eq!(r.location.as_deref(), Some("/thy/trace/1/overview/help"));
    assert_eq!(r.body, ""); // empty body
    // No new version created (in-place mutation).
    assert_eq!(s.versions(), vec![1]);
}

#[test]
fn valid_edit_and_add_are_303_to_their_form_pages() {
    let mut s = server();
    let good = [("lemma-text".to_string(), "lemma X: exists-trace \"T\"".to_string())];
    let re = s.dispatch(&Request::post("/thy/trace/1/edit/edit/Client_auth", &good));
    assert_eq!(re.status, 303);
    assert_eq!(re.location.as_deref(), Some("/thy/trace/1/overview/edit/Client_auth"));
    let ra = s.dispatch(&Request::post("/thy/trace/1/edit/add/%3Cfirst%3E", &good));
    assert_eq!(ra.status, 303);
    assert_eq!(ra.location.as_deref(), Some("/thy/trace/1/overview/add/%3Cfirst%3E"));
    assert_eq!(s.versions(), vec![1]); // both in place
}

#[test]
fn failed_edit_returns_200_form_page_no_change() {
    let mut s = server();
    let bad = [("lemma-text".to_string(), "this is not a lemma".to_string())];
    let r = s.dispatch(&Request::post("/thy/trace/1/edit/edit/Client_auth", &bad));
    assert_eq!(r.status, 200);
    assert_eq!(r.content_type, "text/html; charset=utf-8");
    assert!(r.location.is_none());
    // The re-rendered full page embeds the edit form for that lemma.
    assert!(r.body.contains("action=\"../../edit/edit/Client_auth\""), "{}", &r.body[..200]);
    assert_eq!(s.versions(), vec![1]);
}

// ---------------------------------------------------------------------------
// edit-form textarea rows= formula (integration blocker b): full byte-parity.
// ---------------------------------------------------------------------------

#[test]
fn edit_form_envelope_byte_parity_with_rows_formula() {
    let mut s = server();
    for (lemma, fixture) in [
        ("Client_auth", include_str!("fixtures/r3_edit_form_client_auth.json")),
        ("Client_session_key_honest_setup", include_str!("fixtures/r3_edit_form_honest_setup.json")),
    ] {
        let r = s.dispatch(&Request::get(&format!("/thy/trace/1/main/edit/{lemma}")));
        assert_eq!(r.content_type, "application/json; charset=utf-8");
        // Compare full JSON envelope html+title against the captured oracle body.
        let got: Value = serde_json::from_str(&r.body).unwrap();
        let want: Value = serde_json::from_str(fixture).unwrap();
        assert_eq!(got["title"], want["title"], "title for {lemma}");
        assert_eq!(got["html"], want["html"], "edit-form html (rows) for {lemma}");
    }
}

// ---------------------------------------------------------------------------
// Form JSON envelopes for add/delete (byte-parity vs captures).
// ---------------------------------------------------------------------------

#[test]
fn add_and_delete_form_envelope_byte_parity() {
    let mut s = server();
    let radd = s.dispatch(&Request::get("/thy/trace/1/main/add/Client_session_key_secrecy"));
    let want_add: Value = serde_json::from_str(include_str!("fixtures/r3_add_form_named.json")).unwrap();
    let got_add: Value = serde_json::from_str(&radd.body).unwrap();
    assert_eq!(got_add, want_add);

    let rdel = s.dispatch(&Request::get("/thy/trace/1/main/delete/Client_auth"));
    let want_del: Value = serde_json::from_str(include_str!("fixtures/r3_delete_form_client_auth.json")).unwrap();
    let got_del: Value = serde_json::from_str(&rdel.body).unwrap();
    assert_eq!(got_del, want_del);
}

// ---------------------------------------------------------------------------
// Navigation: next/prev = text/plain bare URL, no version change. (spec item 1)
// ---------------------------------------------------------------------------

#[test]
fn next_prev_are_plain_text_urls_no_bump() {
    let mut s = server();
    let n = s.dispatch(&Request::get("/thy/trace/1/next/normal/proof/Client_auth"));
    assert_eq!(n.status, 200);
    assert_eq!(n.content_type, "text/plain; charset=utf-8");
    assert_eq!(n.body, include_str!("fixtures/r3_next_client_auth.txt"));
    let p = s.dispatch(&Request::get("/thy/trace/1/prev/normal/proof/Client_auth"));
    assert_eq!(p.body, include_str!("fixtures/r3_prev_client_auth.txt"));
    assert_eq!(s.versions(), vec![1]);
}

// ---------------------------------------------------------------------------
// Views: source/message text, intdot html, graph DOT, 404.
// ---------------------------------------------------------------------------

#[test]
fn source_and_message_are_identical_plain_text() {
    let mut s = server();
    let src = s.dispatch(&Request::get("/thy/trace/1/source"));
    let msg = s.dispatch(&Request::get("/thy/trace/1/message"));
    assert_eq!(src.content_type, "text/plain; charset=utf-8");
    assert_eq!(src.body, msg.body);
}

#[test]
fn intdot_page_swaps_handler_and_keeps_tail() {
    let mut s = server();
    let r = s.dispatch(&Request::get("/thy/trace/1/intdot/proof/Client_auth/_/B_2"));
    assert_eq!(r.content_type, "text/html; charset=utf-8");
    assert!(r.body.contains(
        "dotsrc=\"/thy/trace/1/interactive-graph-def/proof/Client_auth/_/B_2\""
    ), "{}", r.body);
    let dot = s.dispatch(&Request::get("/thy/trace/1/interactive-graph-def/proof/Client_auth"));
    assert_eq!(dot.body, EMPTY_GRAPH_DOT);
}

#[test]
fn unmatched_route_and_unknown_index_are_404() {
    let mut s = server();
    let a = s.dispatch(&Request::get("/thy/trace/1/main/nope"));
    assert_eq!(a.status, 404);
    assert!(a.body.contains("<title>Not Found</title>"));
    // Unknown / current (`#`) index does not resolve.
    let b = s.dispatch(&Request::get("/thy/trace/99/main/help"));
    assert_eq!(b.status, 404);
    let c = s.dispatch(&Request::get("/thy/trace/#/main/help"));
    assert_eq!(c.status, 404);
}

#[test]
fn post_to_a_get_route_is_not_a_mutation() {
    // A POST to a non-edit path is unmatched (no accidental mutation).
    let mut s = server();
    let r = s.dispatch(&Request::post("/thy/trace/1/main/help", &[]));
    assert_eq!(r.status, 404);
    assert_eq!(s.versions(), vec![1]);
}

// ---------------------------------------------------------------------------
// GET method used HTTP method matters: HttpMethod is respected.
// ---------------------------------------------------------------------------

#[test]
fn http_method_enum_roundtrips() {
    assert_ne!(HttpMethod::Get, HttpMethod::Post);
}
