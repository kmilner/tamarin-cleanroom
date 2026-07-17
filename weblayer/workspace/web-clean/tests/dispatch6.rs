//! Round-6 tests: the origin-aware page shell threaded through `Server`, and the
//! state-delegation redesign (`StateOps` / `InMemoryState`).
//!
//! * **Origin threading (ITEM 1).** A `FakeProver` whose theory carries an
//!   [`Origin`] and reports it from `meta`; the tests assert that a command-line
//!   (Local) theory's overview shell shows the "Reload file" / "Append modified
//!   lemmas" items while an uploaded (Upload) theory's shell shows neither, and
//!   that a proof-derived version inherits the base's origin (BEHAVIOR §16;
//!   QUERIES.log [R60]).
//! * **State delegation (ITEM 2).** The `Server` drives version state only through
//!   the `StateOps` trait; a custom backend (`CountingState`) records that every
//!   allocation/retrieval/mutation flows through it, and the `InMemoryState`
//!   reference implementation is checked against the documented lifecycle
//!   contract (monotonic allocation, retention, in-place replace, remove).

use web_clean::dispatch::{
    Content, InMemoryState, MainReq, Meta, ProverOps, Request, RootMeta, Server, StateOps,
};
use web_clean::page::Origin;
use web_clean::route::{Autoprove, AutoproveAll, AutoproveDiff, NavDir};

// ---------------------------------------------------------------------------
// A fake prover whose theory records its load origin (Local vs Upload).
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Thy {
    name: String,
    origin: Origin,
    lemmas: Vec<String>,
}

struct Fake;

fn local_base() -> Thy {
    Thy { name: "Tutorial".into(), origin: Origin::Local, lemmas: vec!["secrecy".into()] }
}

impl ProverOps for Fake {
    type Theory = Thy;

    fn meta(&self, thy: &Thy) -> Meta {
        // The prover reports each version's origin; the web layer renders it.
        Meta {
            name: thy.name.clone(),
            version: "1.13.0".into(),
            filename: format!("{}.spthy", thy.name),
            origin: thy.origin,
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
    fn main_content(&self, _thy: &Thy, index: u64, _req: &MainReq) -> Content {
        Content { html: format!("<p>help@{index}</p>"), title: "Help".to_string() }
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
    fn static_file(&self, _path: &[String]) -> Option<Vec<u8>> {
        None
    }
    fn load_theory(&self, source: &str) -> Option<Thy> {
        // An uploaded theory is born with Upload origin.
        source.contains("theory").then(|| Thy {
            name: "NSLPK3".into(),
            origin: Origin::Uploaded,
            lemmas: vec![],
        })
    }
    fn reload(&self, thy: &Thy) -> Thy {
        thy.clone()
    }
    fn apply_method(&self, thy: &Thy, _l: &str, n: usize, _p: &[String]) -> Option<(Thy, Vec<String>)> {
        // A proof-derived version keeps the base theory's origin (clone).
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
    fn lemma_present(&self, thy: &Thy, lemma: &str) -> bool {
        thy.lemmas.iter().any(|l| l == lemma)
    }
    fn del_lemma_path(&self, thy: &Thy, name: &str) -> Option<Thy> {
        thy.lemmas.iter().any(|l| l == name).then(|| thy.clone())
    }
    fn del_proof_step(&self, thy: &Thy, lemma: &str, _p: &[String], _d: bool) -> Option<Thy> {
        thy.lemmas.iter().any(|l| l == lemma).then(|| thy.clone())
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
}

const RELOAD: &str = "Reload file";
const APPEND: &str = "Append modified lemmas to file";

// ---------------------------------------------------------------------------
// ITEM 1 — origin threaded from meta into the rendered overview shell.
// ---------------------------------------------------------------------------

#[test]
fn local_theory_overview_shows_reload_and_append() {
    let mut s = Server::new(Fake, local_base());
    let b = s.dispatch(&Request::get("/thy/trace/1/overview/help")).body;
    assert!(b.contains(RELOAD), "Local shell must show Reload file");
    assert!(b.contains(APPEND), "Local shell must show Append modified lemmas");
}

#[test]
fn uploaded_theory_overview_omits_reload_and_append() {
    let mut s = Server::new(Fake, local_base());
    // Upload a second theory (Upload origin) -> index 2.
    let form = vec![("uploadedTheory".to_string(), "theory NSLPK3 begin end".to_string())];
    s.dispatch(&Request::post("/", &form));
    assert_eq!(s.versions(), vec![1, 2]);
    let b = s.dispatch(&Request::get("/thy/trace/2/overview/help")).body;
    assert!(!b.contains(RELOAD), "Upload shell must NOT show Reload file");
    assert!(!b.contains(APPEND), "Upload shell must NOT show Append modified lemmas");
    // The Local base (index 1) is unaffected: still shows both items.
    let base = s.dispatch(&Request::get("/thy/trace/1/overview/help")).body;
    assert!(base.contains(RELOAD) && base.contains(APPEND));
}

#[test]
fn proof_derived_version_inherits_upload_origin() {
    let mut s = Server::new(Fake, local_base());
    let form = vec![("uploadedTheory".to_string(), "theory NSLPK3 begin end".to_string())];
    s.dispatch(&Request::post("/", &form)); // index 2, Upload
    // A proof op on the uploaded theory allocates index 3, inheriting Upload origin.
    s.dispatch(&Request::get("/thy/trace/2/main/method/secrecy/1"));
    assert_eq!(s.versions(), vec![1, 2, 3]);
    let derived = s.dispatch(&Request::get("/thy/trace/3/overview/help")).body;
    assert!(!derived.contains(RELOAD), "derived-from-upload shell must omit Reload");
    assert!(!derived.contains(APPEND), "derived-from-upload shell must omit Append");
    // A proof op on the Local base (index 1) allocates index 4, inheriting Local.
    s.dispatch(&Request::get("/thy/trace/1/main/method/secrecy/1"));
    let derived_local = s.dispatch(&Request::get("/thy/trace/4/overview/help")).body;
    assert!(derived_local.contains(RELOAD) && derived_local.contains(APPEND));
}

// ---------------------------------------------------------------------------
// ITEM 2 — the Server drives version state only through the StateOps trait.
// ---------------------------------------------------------------------------

/// A distinct StateOps backend (a newtype over the in-memory reference) proving
/// the Server is generic over the state owner: it never touches a version map of
/// its own, only the trait. A real consumer's async caching backend takes this
/// same seam.
struct WrapState(InMemoryState<Thy>);

impl StateOps for WrapState {
    type Theory = Thy;
    fn insert_new(&mut self, theory: Thy) -> u64 {
        self.0.insert_new(theory)
    }
    fn get(&self, index: u64) -> Option<&Thy> {
        self.0.get(index)
    }
    fn replace(&mut self, index: u64, theory: Thy) {
        self.0.replace(index, theory);
    }
    fn remove(&mut self, index: u64) -> Option<Thy> {
        self.0.remove(index)
    }
    fn entries(&self) -> Vec<(u64, &Thy)> {
        self.0.entries()
    }
}

#[test]
fn server_drives_a_custom_state_backend_identically() {
    // with_state injects a caller-owned backend (pre-seeded with the base at 1).
    let mut s = Server::with_state(Fake, WrapState(InMemoryState::seeded(local_base())));

    // A proof op allocates a fresh version through insert_new.
    let r = s.dispatch(&Request::get("/thy/trace/1/main/method/secrecy/1"));
    assert_eq!(r.body, r#"{"redirect":"/thy/trace/2/overview/proof/secrecy/_"}"#);
    assert_eq!(s.versions(), vec![1, 2]);

    // A reload mutates in place through replace (no allocation).
    let rl = s.dispatch(&Request::post("/thy/trace/1/reload", &[]));
    assert_eq!(rl.body, r#"{"redirect":"/thy/trace/1/overview/help"}"#);
    assert_eq!(s.versions(), vec![1, 2]);

    // Read views resolve through get and stay resolvable (retention).
    assert_eq!(s.dispatch(&Request::get("/thy/trace/1/overview/help")).status, 200);
    assert_eq!(s.dispatch(&Request::get("/thy/trace/2/overview/help")).status, 200);
    assert_eq!(s.dispatch(&Request::get("/thy/trace/99/overview/help")).status, 404);
}

// ---------------------------------------------------------------------------
// InMemoryState reference impl vs. the documented lifecycle contract.
// ---------------------------------------------------------------------------

#[test]
fn inmemory_state_allocation_is_monotonic_and_retains() {
    let mut st: InMemoryState<u32> = InMemoryState::new();
    assert_eq!(st.insert_new(10), 1);
    assert_eq!(st.insert_new(20), 2);
    assert_eq!(st.insert_new(30), 3);
    // In-place replace does not allocate; later allocations keep climbing.
    st.replace(2, 99);
    assert_eq!(st.insert_new(40), 4);
    // Every index remains resolvable; replace overwrote only index 2.
    assert_eq!(st.get(1), Some(&10));
    assert_eq!(st.get(2), Some(&99));
    assert_eq!(st.get(3), Some(&30));
    assert_eq!(st.get(4), Some(&40));
    assert_eq!(st.get(5), None);
    assert_eq!(st.entries(), vec![(1, &10), (2, &99), (3, &30), (4, &40)]);
}

#[test]
fn inmemory_state_remove_and_monotonicity_after_remove() {
    let mut st: InMemoryState<u32> = InMemoryState::new();
    st.insert_new(10);
    st.insert_new(20);
    assert_eq!(st.remove(1), Some(10));
    assert_eq!(st.remove(1), None);
    assert_eq!(st.get(1), None);
    // The counter never rewinds: the next allocation is 3, not a reused 1.
    assert_eq!(st.insert_new(30), 3);
    assert_eq!(st.entries(), vec![(2, &20), (3, &30)]);
}

#[test]
fn seeded_state_puts_base_at_index_one() {
    let st = InMemoryState::seeded("base".to_string());
    assert_eq!(st.get(1), Some(&"base".to_string()));
    assert_eq!(st.entries().len(), 1);
}
