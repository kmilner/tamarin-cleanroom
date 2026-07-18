//! Round-7 tests: the snapshot → compute → commit dispatch is concurrency-safe.
//!
//! These assert the concurrency CONTRACT probed live against the reference server
//! (`BEHAVIOR.md` §17, `QUERIES.log` [R71]–[R75]): with a long proof operation in
//! flight, unrelated and related requests are served concurrently, the fresh
//! version index is allocated at COMMIT (completion) — so an op that *starts* first
//! but *commits* last gets the HIGHER index — the not-yet-committed version is
//! invisible until it commits, and concurrent allocations never collide or skip.
//!
//! A `GatedProver` makes the slow op (`autoprove`) block on a controllable gate:
//! it signals when it has entered (its snapshot already taken, no lock held) and
//! then parks until the test releases it. This deterministically reproduces the
//! probed interleaving — slow-start → fast-commit → slow-commit — with no sleeps.
//! The fast op (`apply_method`) and every read never touch the gate.

use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use web_clean::dispatch::{
    Content, MainReq, Meta, ProverOps, Request, RootMeta, Server,
};
use web_clean::page::Origin;
use web_clean::route::{Autoprove, AutoproveAll, AutoproveDiff, NavDir};

// ---------------------------------------------------------------------------
// A controllable gate: the slow op enters (announcing it is mid-compute) and
// parks until the test releases it. One mutex guards both flags; the Condvar
// wait releases the mutex while parked, so the test can observe/act meanwhile.
// ---------------------------------------------------------------------------

#[derive(Default)]
struct GateState {
    started: bool,
    released: bool,
}

struct Gate {
    m: Mutex<GateState>,
    started_cv: Condvar,
    release_cv: Condvar,
}

impl Gate {
    fn new() -> Gate {
        Gate { m: Mutex::new(GateState::default()), started_cv: Condvar::new(), release_cv: Condvar::new() }
    }
    /// Called by the slow op: announce entry, then park until released.
    fn enter_and_park(&self) {
        let mut g = self.m.lock().unwrap();
        g.started = true;
        self.started_cv.notify_all();
        while !g.released {
            g = self.release_cv.wait(g).unwrap();
        }
    }
    /// Called by the test: block until the slow op has entered its compute.
    fn wait_started(&self) {
        let mut g = self.m.lock().unwrap();
        while !g.started {
            g = self.started_cv.wait(g).unwrap();
        }
    }
    /// Called by the test: let the parked slow op proceed to commit.
    fn release(&self) {
        let mut g = self.m.lock().unwrap();
        g.released = true;
        self.release_cv.notify_all();
    }
}

// ---------------------------------------------------------------------------
// A minimal prover whose `autoprove` parks on the gate (the "slow op"); every
// other callback is fast and gate-free.
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct Thy {
    lemmas: Vec<String>,
}

struct GatedProver {
    gate: Arc<Gate>,
}

fn base() -> Thy {
    Thy { lemmas: vec!["L".into()] }
}

impl ProverOps for GatedProver {
    type Theory = Thy;

    fn meta(&self, _thy: &Thy) -> Meta {
        Meta { name: "T".into(), version: "1.13.0".into(), filename: "T.spthy".into(), origin: Origin::Local }
    }
    fn root_meta(&self, _thy: &Thy) -> RootMeta {
        RootMeta { time: "00:00:00".into(), origin: "T.spthy".into(), modified: false }
    }
    fn source_text(&self, _thy: &Thy) -> String {
        "theory T begin\nend".into()
    }
    fn west_pane(&self, _thy: &Thy, index: u64) -> String {
        format!("WEST@{index}")
    }
    fn main_content(&self, _thy: &Thy, index: u64, _req: &MainReq) -> Content {
        Content { html: format!("<p>help@{index}</p>"), title: "T".into() }
    }
    fn lemma_source(&self, _thy: &Thy, name: &str) -> Option<String> {
        Some(format!("lemma {name}: exists-trace \"x\""))
    }
    fn graph_dot(&self, _thy: &Thy, _tail: &[String]) -> String {
        web_clean::intdot::EMPTY_GRAPH_DOT.to_string()
    }
    fn nav_target(&self, _thy: &Thy, index: u64, _dir: NavDir, _mode: &str, lemma: &str) -> String {
        format!("/thy/trace/{index}/main/proof/{lemma}")
    }
    fn append_message(&self, _thy: &Thy) -> String {
        "Appended".into()
    }
    fn static_file(&self, _path: &[String]) -> Option<Vec<u8>> {
        None
    }
    fn load_theory(&self, source: &str) -> Option<Thy> {
        source.contains("theory").then(|| Thy { lemmas: vec![] })
    }
    fn reload(&self, thy: &Thy) -> Thy {
        thy.clone()
    }
    /// The SLOW op: park on the gate (after the caller has already taken the
    /// snapshot and released the state lock), then produce the new version.
    fn autoprove(&self, thy: &Thy, _spec: &Autoprove) -> (Thy, Vec<String>) {
        self.gate.enter_and_park();
        (thy.clone(), vec!["_".into()])
    }
    /// The FAST op: no gate, returns immediately.
    fn apply_method(&self, thy: &Thy, _l: &str, n: usize, _p: &[String]) -> Option<(Thy, Vec<String>)> {
        (n != 0).then(|| (thy.clone(), vec!["_".into()]))
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
        (n != 0).then(|| (thy.clone(), vec!["R".into()]))
    }
    fn autoprove_diff(&self, thy: &Thy, _s: &AutoproveDiff) -> (Thy, Vec<String>) {
        (thy.clone(), vec!["R".into()])
    }
    fn autoprove_all(&self, thy: &Thy, _s: &AutoproveAll) -> Thy {
        thy.clone()
    }
}

/// Extract the version index `N` from a `{"redirect":"/thy/trace/N/…"}` body.
fn redirect_index(body: &str) -> u64 {
    let marker = "/thy/trace/";
    let start = body.find(marker).expect("redirect body") + marker.len();
    let rest = &body[start..];
    let end = rest.find('/').unwrap_or(rest.len());
    rest[..end].parse().expect("numeric index")
}

// ---------------------------------------------------------------------------
// The main interleaving test (BEHAVIOR §17.1–§17.4).
// ---------------------------------------------------------------------------

#[test]
fn slow_op_in_flight_is_non_blocking_and_commits_last() {
    let gate = Arc::new(Gate::new());
    let server = Arc::new(Server::new(GatedProver { gate: gate.clone() }, base()));

    // Launch the SLOW op (autoprove on idx1) on another thread. `dispatch(&self)`
    // lets the same server be shared; the op takes its snapshot, releases the state
    // lock, then parks inside `autoprove`.
    let slow_server = server.clone();
    let slow = thread::spawn(move || {
        slow_server
            .dispatch(&Request::get("/thy/trace/1/autoprove/idfs/0/False/proof/L"))
            .body
    });

    // Wait until the slow op is provably mid-compute (snapshot taken, lock released).
    gate.wait_started();

    // §17.1 NON-BLOCKING: an unrelated read is served immediately while the slow op
    // is parked — dispatch did not need exclusive ownership.
    let read = server.dispatch(&Request::get("/thy/trace/1/overview/help"));
    assert_eq!(read.status, 200, "a read must be served while a slow op is in flight");

    // §17.2 A second (fast) proof op COMMITS while the slow op is still parked, so it
    // takes the LOWER index 2 — even though the slow op STARTED first.
    let fast = server.dispatch(&Request::get("/thy/trace/1/main/method/L/1"));
    assert_eq!(fast.status, 200);
    assert_eq!(redirect_index(&fast.body), 2, "fast op commits first -> lower index");

    // §17.3 The slow op's version is NOT yet visible (allocation is at commit): only
    // 1 and 2 exist, and index 3 does not resolve yet.
    assert_eq!(server.versions(), vec![1, 2], "slow op's index must be invisible pre-commit");
    assert_eq!(
        server.dispatch(&Request::get("/thy/trace/3/overview/help")).status,
        404,
        "the not-yet-committed version must not resolve",
    );

    // Release the slow op; it now commits AFTER the fast op and takes the HIGHER
    // index 3 (§17.2: completion order, independent of start order).
    gate.release();
    let slow_body = slow.join().unwrap();
    assert_eq!(redirect_index(&slow_body), 3, "slow op commits last -> higher index");

    // §17.3/§17.4 Final set is contiguous with no collision or skip.
    assert_eq!(server.versions(), vec![1, 2, 3]);
    assert_eq!(server.dispatch(&Request::get("/thy/trace/3/overview/help")).status, 200);
}

// ---------------------------------------------------------------------------
// Counter race: many simultaneous proof ops (BEHAVIOR §17.3/§17.4, [R74]).
// ---------------------------------------------------------------------------

#[test]
fn concurrent_allocations_never_collide_or_skip() {
    // A gate that is pre-released so `autoprove` does not park: here every op is fast
    // and we stress the atomic allocation under a genuine thread race.
    let gate = Arc::new(Gate::new());
    gate.release();
    let server = Arc::new(Server::new(GatedProver { gate }, base()));

    const N: u64 = 16;
    let mut handles = Vec::new();
    for _ in 0..N {
        let s = server.clone();
        handles.push(thread::spawn(move || {
            // A fast proof op (method) -> a fresh version each.
            redirect_index(&s.dispatch(&Request::get("/thy/trace/1/main/method/L/1")).body)
        }));
    }
    let mut got: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    got.sort_unstable();

    // Exactly the contiguous block 2..=N+1: N distinct indices, no collision, no skip.
    let want: Vec<u64> = (2..=N + 1).collect();
    assert_eq!(got, want, "concurrent insert_new must be atomic, contiguous, unique");
    // The backend agrees: base + N new versions, all resolvable (retention).
    assert_eq!(server.versions(), (1..=N + 1).collect::<Vec<_>>());
}
