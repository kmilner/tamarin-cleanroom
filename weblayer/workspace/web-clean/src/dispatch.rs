//! The interactive-UI **state machine**: route dispatch decisions, theory-version
//! management, and response assembly across the **whole** request surface.
//!
//! This owns the web layer's *behaviour* — what each route returns and how the
//! version state evolves — over a [`ProverOps`] callback trait supplied at
//! integration by the ported prover. This module never inspects a theory; it
//! only decides which callback to invoke, how the version state changes, and how
//! to wrap the result into an HTTP response.
//!
//! Everything here was derived from black-box observation (`BEHAVIOR.md` §§13–17,
//! `QUERIES.log`); no prover source was read.
//!
//! ## Surface owned here
//! * **Top level.** `/` (index GET / theory-upload POST), `/robots.txt`,
//!   `/favicon.ico` (a `303` to the static icon), `/kill` (cancel a running
//!   search — needs a `path` query arg), and `/static/**` (filesystem assets:
//!   content type by extension, `File not found` for a miss).
//! * **Theory scope** `/thy/<kind>/<index>/…` for both `trace` and `equiv`
//!   (observational-equivalence / diff) kinds: the read views, proof operations,
//!   structural edits, navigation, graph routes, plus `reload`, `download`,
//!   `get_and_append`, and the diff analogues `diffProof` / `diffMethod` /
//!   `diffrules` / `autoproveDiff` / `autoproveAll`.
//!
//! ## Version model (one global index space)
//! Every theory-version — an originally loaded theory, an uploaded theory, or a
//! version produced by a proof operation — occupies a distinct index in one
//! monotonically growing namespace. **Proof operations** (`method`, `diffMethod`,
//! `autoprove*`) allocate a **fresh** index (`= max ever + 1`) and leave earlier
//! versions resolvable. **Upload** likewise allocates a fresh index off the same
//! counter. **Structural edits** (`edit`/`add`/`delete`) and **`reload`** mutate
//! the theory **in place** at its index (no new index; the counter is untouched).
//! Navigation and read views never change the version set.
//!
//! ## Concurrency: snapshot → compute → commit (`BEHAVIOR.md` §17)
//! [`dispatch`](Server::dispatch) takes **`&self`**, so one [`Server`] is shared
//! across concurrent requests. The reference web UI serves a long-running proof
//! operation *without* freezing unrelated (or related) requests ([R71]); a slow
//! [`ProverOps`] call therefore must NOT hold any exclusive lock. Each request runs
//! as three phases:
//!
//! 1. **get-snapshot** — resolve the requested index and take a cheap owned
//!    snapshot of that version through [`StateOps::snapshot`], releasing the
//!    backend lock immediately.
//! 2. **compute** — run the [`ProverOps`] call (including the possibly-slow
//!    `autoprove*` / `apply_method` / `del_*` / `reload` / `load_theory` / edits)
//!    on the snapshot **with no state lock held**. Concurrent requests take their
//!    own snapshots and run in parallel.
//! 3. **commit** — apply the result with a separate, atomic [`StateOps`] call:
//!    [`insert_new`](StateOps::insert_new) for a proof op / upload / `del/path`
//!    (the fresh monotonic index is allocated **now**, at commit — matching the
//!    probed completion-order allocation), or [`replace`](StateOps::replace) for an
//!    in-place `reload`/structural edit. Reads commit nothing.
//!
//! The only serialized sections are the microsecond-scale atomic `StateOps` calls;
//! all compute is lock-free. Version state lives behind the [`StateOps`] backend's
//! interior mutability ([`InMemoryState`] is the reference in-memory implementation),
//! so a consumer can supply an asynchronous, internally-caching backend that remains
//! the single owner of theory state. The observed lifecycle (`BEHAVIOR.md` §17) —
//! commit-time monotonic allocation, atomicity under races, retention, in-place
//! mutation, snapshot isolation — is the documented **contract** that backend must
//! satisfy.

use std::collections::BTreeMap;
use std::sync::Mutex;

use crate::envelope::METHOD_FAILED_ALERT;
use crate::page::{Flash, Origin, PageParams, RootRow, ShellKind};
use crate::route::{
    Autoprove, AutoproveAll, AutoproveDiff, EditVerb, Handler, Index, Main, Nav, NavDir,
    OverviewView, Route, ThyPath, Toplevel,
};
use crate::{assets, envelope, errors, forms, intdot, page};

/// `application/json; charset=utf-8`.
pub const CT_JSON: &str = "application/json; charset=utf-8";
/// `text/html; charset=utf-8`.
pub const CT_HTML: &str = "text/html; charset=utf-8";
/// `text/plain; charset=utf-8` (text bodies, `next`/`prev`, DOT, `robots`, `kill`).
pub const CT_TEXT: &str = "text/plain; charset=utf-8";
/// `application/octet-stream` (the `download` route).
pub const CT_OCTET: &str = "application/octet-stream";

/// `Cache-Control` value on `303` redirects (and the favicon redirect).
pub const CACHE_CONTROL_NOCACHE: &str = "no-cache, must-revalidate";
/// `Expires` value paired with [`CACHE_CONTROL_NOCACHE`] (a fixed past instant).
pub const EXPIRES_PAST: &str = "Thu, 01 Jan 1970 05:05:05 GMT";

/// HTTP request method (only the two the UI uses are modelled).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
}

impl HttpMethod {
    fn as_str(self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
        }
    }
}

/// An incoming request. `path` is the URL path with any `?query` split off into
/// `query` (decoded key/value pairs).
pub struct Request<'a> {
    pub method: HttpMethod,
    pub path: &'a str,
    /// Decoded query-string pairs (e.g. `("path", "solve(...)")` for `/kill`).
    pub query: &'a [(String, String)],
    /// Decoded POST form fields (e.g. `("lemma-text", "lemma foo: …")`, or
    /// `("uploadedTheory", <source>)` for an upload).
    pub form: &'a [(String, String)],
}

impl<'a> Request<'a> {
    pub fn get(path: &'a str) -> Request<'a> {
        Request { method: HttpMethod::Get, path, query: &[], form: &[] }
    }
    pub fn get_query(path: &'a str, query: &'a [(String, String)]) -> Request<'a> {
        Request { method: HttpMethod::Get, path, query, form: &[] }
    }
    pub fn post(path: &'a str, form: &'a [(String, String)]) -> Request<'a> {
        Request { method: HttpMethod::Post, path, query: &[], form }
    }
    fn field(&self, key: &str) -> &str {
        Self::lookup(self.form, key)
    }
    fn query_param(&self, key: &str) -> Option<&str> {
        self.query.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
    }
    fn lookup<'b>(pairs: &'b [(String, String)], key: &str) -> &'b str {
        pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str()).unwrap_or("")
    }
}

/// An outgoing response. `location` is set for `3xx` redirects; `no_cache`
/// requests the `Cache-Control`/`Expires` pair Yesod attaches to redirects.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub body: String,
    pub location: Option<String>,
    pub no_cache: bool,
}

impl Response {
    fn base(status: u16, content_type: &'static str, body: String) -> Response {
        Response { status, content_type, body, location: None, no_cache: false }
    }
    fn json(body: String) -> Response {
        Response::base(200, CT_JSON, body)
    }
    fn html(body: String) -> Response {
        Response::base(200, CT_HTML, body)
    }
    fn text(body: String) -> Response {
        Response::base(200, CT_TEXT, body)
    }
    fn octet(body: String) -> Response {
        Response::base(200, CT_OCTET, body)
    }
    fn see_other(location: String) -> Response {
        Response { status: 303, content_type: CT_TEXT, body: String::new(), location: Some(location), no_cache: true }
    }
    fn not_found(echoed_path: &str) -> Response {
        Response::base(404, CT_HTML, errors::render_not_found(echoed_path))
    }
    fn invalid_args(messages: &[&str]) -> Response {
        Response::base(400, CT_HTML, errors::render_invalid_args(messages))
    }
    fn bad_method(method: HttpMethod) -> Response {
        Response::base(405, CT_HTML, errors::render_bad_method(method.as_str()))
    }
    /// The static-handler `404` for a missing file (plain text, distinct from the
    /// full-HTML `404` of the dynamic router).
    fn static_missing() -> Response {
        Response::base(404, CT_TEXT, assets::STATIC_NOT_FOUND.to_string())
    }
}

/// Theory metadata the page shell needs.
pub struct Meta {
    pub name: String,
    pub version: String,
    pub filename: String,
    /// Where this version was loaded from (Local file vs `POST /` upload); gates
    /// the north-bar "Reload file"/"Append modified lemmas" items. Proof-derived
    /// versions inherit the base theory's origin.
    pub origin: Origin,
}

/// Per-version index-page row data (the non-deterministic parts of a table row).
pub struct RootMeta {
    /// Load time, e.g. `"17:44:37"` (non-deterministic).
    pub time: String,
    /// Source origin, e.g. a temp path or the uploaded filename (non-deterministic).
    pub origin: String,
    /// Whether the theory has been modified from its loaded state.
    pub modified: bool,
}

/// A rendered content pane (`html` + pane `title`).
pub struct Content {
    pub html: String,
    pub title: String,
}

/// A request for a `main/*` content pane, resolved to the prover.
pub enum MainReq<'a> {
    Help,
    Message,
    Rules,
    Tactic,
    Cases { refined: bool, level: usize, n: usize },
    Lemma(&'a str),
    Proof { lemma: &'a str, path: &'a [String] },
    /// Diff-mode proof view.
    DiffProof { lemma: &'a str, path: &'a [String] },
    /// Diff construction/deconstruction rules view.
    DiffRules,
}

/// The prover-supplied callbacks. The web layer treats every returned string as an
/// opaque fragment; the *decisions* around these calls (version allocation, route
/// dispatch, envelope shape, content types, redirects) live in [`Server`].
///
/// Callbacks receive a `&Self::Theory` **snapshot** (owned by the caller for the
/// duration of the call), never a live handle into the state backend; a possibly
/// slow call (`autoprove*`, `apply_method`/`apply_diff_method`, `del_*`, `reload`,
/// `load_theory`, edits) therefore runs with no state lock held (`BEHAVIOR.md` §17).
pub trait ProverOps {
    /// Opaque per-version theory handle held by the [`Server`].
    type Theory;

    // ---- pretty-printers / fragment producers ----
    /// Shell metadata (theory name, Tamarin version string, source filename, and
    /// the load [`Origin`]). The prover is the authority on a version's origin: a
    /// proof-derived version reports the same origin as the base it came from.
    fn meta(&self, thy: &Self::Theory) -> Meta;
    /// Index-page row data (load time, origin, modified flag) for `thy`.
    fn root_meta(&self, thy: &Self::Theory) -> RootMeta;
    /// The `source`/`message`/`download` body: the theory source, verbatim.
    fn source_text(&self, thy: &Self::Theory) -> String;
    /// Inner HTML of the west proof-script pane at `index`.
    fn west_pane(&self, thy: &Self::Theory, index: u64) -> String;
    /// Center-pane content (html + title) for a `main/*` view at `index`.
    fn main_content(&self, thy: &Self::Theory, index: u64, req: &MainReq) -> Content;
    /// Raw (unescaped) current source of a lemma, for the edit form.
    fn lemma_source(&self, thy: &Self::Theory, name: &str) -> Option<String>;
    /// Graphviz DOT for an `interactive-graph-def` node.
    fn graph_dot(&self, thy: &Self::Theory, tail: &[String]) -> String;
    /// The `next`/`prev` target URL (bare path), from the prover's traversal.
    fn nav_target(&self, thy: &Self::Theory, index: u64, dir: NavDir, mode: &str, lemma: &str) -> String;
    /// The `get_and_append` alert message (names the file appended to).
    fn append_message(&self, thy: &Self::Theory) -> String;
    /// Raw bytes of a `/static/<path>` asset, or `None` if the file is absent.
    fn static_file(&self, path: &[String]) -> Option<Vec<u8>>;

    // ---- mutations / loads ----
    /// Parse an uploaded theory source into a theory, or `None` on failure.
    fn load_theory(&self, source: &str) -> Option<Self::Theory>;
    /// Re-read the theory from its source in place (the `reload` route).
    fn reload(&self, thy: &Self::Theory) -> Self::Theory;
    /// Apply proof method `n` at `(lemma, path)`. `Some((theory, focus))` on
    /// success (`focus` = the prover's next open goal, incl. root `_`); `None`
    /// if the method application failed (the web layer answers a JSON alert).
    fn apply_method(&self, thy: &Self::Theory, lemma: &str, n: usize, path: &[String]) -> Option<(Self::Theory, Vec<String>)>;
    /// Autoprove per `spec`. Returns the new theory and resulting focus path.
    fn autoprove(&self, thy: &Self::Theory, spec: &Autoprove) -> (Self::Theory, Vec<String>);
    /// Edit lemma `name` in place. `Some` = the modified theory; `None` = parse/wf
    /// failure or unknown lemma (the web layer then re-renders the edit form).
    fn edit_lemma(&self, thy: &Self::Theory, name: &str, text: &str) -> Option<Self::Theory>;
    /// Add a lemma at position `pos`. `Some`/`None` as for [`Self::edit_lemma`].
    fn add_lemma(&self, thy: &Self::Theory, pos: &str, text: &str) -> Option<Self::Theory>;
    /// Delete lemma `name` in place. `Some` = the modified theory (redirect to the
    /// help view); `None` = lemma not found (redirect to the delete view).
    fn delete_lemma(&self, thy: &Self::Theory, name: &str) -> Option<Self::Theory>;

    // ---- del/path + verify (theory-path operations) ----
    /// Whether `lemma` is a lemma present in `thy`. Drives the `verify/proof/…`
    /// redirect-vs-content choice: a `verify/proof/{lemma}[/path]` redirects to the
    /// proof view iff the lemma is present, otherwise the help pane is returned.
    fn lemma_present(&self, thy: &Self::Theory, lemma: &str) -> bool;
    /// Delete the proof at the lemma node `name` (`del/path/lemma/{name}`),
    /// producing a fresh theory version. `None` if the lemma cannot be removed
    /// (absent) — the web layer then answers a "removing the selected lemma failed"
    /// alert.
    fn del_lemma_path(&self, thy: &Self::Theory, name: &str) -> Option<Self::Theory>;
    /// Remove the proof step at a proof node (`del/path/proof/…`, or the diff
    /// `del/path/diffProof/…` when `diff`), producing a fresh theory version.
    /// `None` if the node cannot be removed (nonexistent lemma or unremovable node)
    /// — the web layer then answers a "removing the selected proof step failed"
    /// alert.
    fn del_proof_step(&self, thy: &Self::Theory, lemma: &str, path: &[String], diff: bool) -> Option<Self::Theory>;

    // ---- diff-mode (observational equivalence) ----
    /// Apply a diff proof method; `Some`/`None` as for [`Self::apply_method`].
    fn apply_diff_method(&self, thy: &Self::Theory, lemma: &str, n: usize, path: &[String]) -> Option<(Self::Theory, Vec<String>)>;
    /// Autoprove a single diff lemma per `spec`; returns theory + focus path.
    fn autoprove_diff(&self, thy: &Self::Theory, spec: &AutoproveDiff) -> (Self::Theory, Vec<String>);
    /// Autoprove all lemmas per `spec`; returns the resulting theory.
    fn autoprove_all(&self, thy: &Self::Theory, spec: &AutoproveAll) -> Self::Theory;
}

/// The theory-version **state backend**: the single owner of the version set and
/// the monotonic version counter. [`Server`] holds no version map of its own and
/// drives all version state exclusively through this trait.
///
/// The trait is an **interior-mutability** façade (every method takes `&self`) so a
/// single [`Server`] can be **shared** across concurrent requests
/// ([`dispatch`](Server::dispatch) is `&self`). Reads take an owned **snapshot**
/// ([`snapshot`](StateOps::snapshot)) so the caller can run a possibly-slow
/// computation on it with no lock held; mutations
/// ([`insert_new`](StateOps::insert_new)/[`replace`](StateOps::replace)) are
/// separate atomic commits. A consumer can supply an asynchronous, internally-caching
/// backend that remains the sole owner of theory state ([`InMemoryState`] is the
/// reference in-memory implementation).
///
/// The lifecycle semantics below are the **contract** a backend must satisfy for the
/// observed web-UI behaviour to hold (probed live; `BEHAVIOR.md` §§13.1/14.3/17).
/// One global index namespace covers every loaded, uploaded, or proof-derived
/// version:
///
/// * **Commit-time monotonic allocation.** [`insert_new`](StateOps::insert_new)
///   stores the theory at `= (max index ever allocated) + 1`, never reusing an index
///   and independent of any base index. Allocation happens **at the call** (i.e. when
///   a proof op *completes*), not when the request begins, and is **atomic**: two
///   concurrent `insert_new` calls receive distinct, consecutive indices — never a
///   collision, never a skip (`BEHAVIOR.md` §17.2/§17.3). The first allocation on a
///   fresh backend is index `1` (the originally loaded theory).
/// * **Retention.** Every allocated index stays resolvable by
///   [`snapshot`](StateOps::snapshot) for the backend's lifetime; the web layer never
///   asks for a version to be dropped (the index-page window is a display cap only).
/// * **Snapshot isolation.** [`snapshot`](StateOps::snapshot) returns a cheap owned
///   copy that is unaffected by later mutations of the same index; a long compute run
///   on a snapshot is not corrupted by a concurrent [`replace`](StateOps::replace) of
///   its base (`BEHAVIOR.md` §17.4).
/// * **In-place mutation.** [`replace`](StateOps::replace) overwrites the theory at an
///   existing index (structural edits + `reload`) without allocating, and leaves the
///   counter and every other version untouched. The web layer only calls it on an
///   index it has already resolved.
/// * **Enumeration.** [`indices`](StateOps::indices) yields all allocated indices in
///   ascending order (drives the index page and [`Server::versions`]).
/// * **Deletion.** [`remove`](StateOps::remove) drops a version. Under the observed
///   retention contract the web layer never invokes it; it is part of the backend's
///   ownership surface (e.g. cache eviction) — see the honesty note in `REPORT.md`.
pub trait StateOps {
    /// The per-version theory handle, shared with [`ProverOps::Theory`].
    type Theory;
    /// Take a cheap **owned snapshot** of the theory at `index` (`None` if no such
    /// version exists), releasing any internal lock before returning so the caller
    /// can compute on it lock-free.
    fn snapshot(&self, index: u64) -> Option<Self::Theory>;
    /// Atomically allocate a fresh monotonic index, store `theory` there, and return
    /// the index (never reused; `1` on a fresh backend). Prior versions are retained.
    /// Concurrent calls never collide or skip.
    fn insert_new(&self, theory: Self::Theory) -> u64;
    /// Overwrite the theory at an existing `index` in place (no allocation; the
    /// counter and other versions are untouched).
    fn replace(&self, index: u64, theory: Self::Theory);
    /// Remove the version at `index` (returning it if present). Not exercised by the
    /// current web-layer surface — see the trait-level note.
    fn remove(&self, index: u64) -> Option<Self::Theory>;
    /// All allocated indices in ascending order.
    fn indices(&self) -> Vec<u64>;
}

/// In-memory reference implementation of [`StateOps`]: a `BTreeMap` keyed by index
/// plus a monotonic counter, behind a [`Mutex`] for interior mutability. Satisfies
/// the full lifecycle contract and is the default backend used by [`Server::new`].
/// `snapshot`/`insert_new`/`replace`/`remove` each hold the lock only for the
/// map/counter operation itself, never across a [`ProverOps`] computation.
pub struct InMemoryState<T> {
    inner: Mutex<StateInner<T>>,
}

struct StateInner<T> {
    versions: BTreeMap<u64, T>,
    next_index: u64,
}

impl<T> InMemoryState<T> {
    /// An empty backend whose first [`StateOps::insert_new`] allocates index `1`.
    pub fn new() -> InMemoryState<T> {
        InMemoryState { inner: Mutex::new(StateInner { versions: BTreeMap::new(), next_index: 1 }) }
    }
}

impl<T: Clone> InMemoryState<T> {
    /// A backend pre-seeded with `base` at index `1` (the originally loaded theory).
    pub fn seeded(base: T) -> InMemoryState<T> {
        let s = InMemoryState::new();
        let i = s.insert_new(base);
        debug_assert_eq!(i, 1, "the first allocated version index is 1");
        s
    }
}

impl<T> Default for InMemoryState<T> {
    fn default() -> Self {
        InMemoryState::new()
    }
}

impl<T: Clone> StateOps for InMemoryState<T> {
    type Theory = T;

    fn snapshot(&self, index: u64) -> Option<T> {
        self.inner.lock().unwrap().versions.get(&index).cloned()
    }
    fn insert_new(&self, theory: T) -> u64 {
        let mut g = self.inner.lock().unwrap();
        let index = g.next_index;
        g.next_index += 1;
        g.versions.insert(index, theory);
        index
    }
    fn replace(&self, index: u64, theory: T) {
        self.inner.lock().unwrap().versions.insert(index, theory);
    }
    fn remove(&self, index: u64) -> Option<T> {
        self.inner.lock().unwrap().versions.remove(&index)
    }
    fn indices(&self) -> Vec<u64> {
        self.inner.lock().unwrap().versions.keys().copied().collect()
    }
}

/// The interactive server. It owns all route dispatch, transport, and response
/// assembly, but **not** the version state: that lives behind a [`StateOps`]
/// backend `S` (defaulting to the in-memory [`InMemoryState`]).
///
/// [`dispatch`](Server::dispatch) takes `&self`, so one `Server` is shared across
/// concurrent requests; every version read/allocation/mutation flows through `S`'s
/// interior mutability as a snapshot → compute → commit pipeline (see the module
/// docs and `BEHAVIOR.md` §17).
pub struct Server<P: ProverOps, S: StateOps<Theory = P::Theory> = InMemoryState<<P as ProverOps>::Theory>> {
    ops: P,
    state: S,
}

impl<P: ProverOps> Server<P, InMemoryState<P::Theory>>
where
    P::Theory: Clone,
{
    /// Create a server holding `base` as version 1 in a fresh in-memory backend.
    pub fn new(ops: P, base: P::Theory) -> Server<P, InMemoryState<P::Theory>> {
        Server::with_state(ops, InMemoryState::seeded(base))
    }
}

impl<P: ProverOps, S: StateOps<Theory = P::Theory>> Server<P, S> {
    /// Create a server over a caller-supplied state backend. The backend must
    /// already hold the base theory at index 1 (see the [`StateOps`] contract);
    /// this is the seam a consumer uses to plug in its own theory-state owner.
    pub fn with_state(ops: P, state: S) -> Server<P, S> {
        Server { ops, state }
    }

    /// Version indices currently allocated (ascending). All are resolvable.
    pub fn versions(&self) -> Vec<u64> {
        self.state.indices()
    }

    /// Dispatch a request across the whole surface. Takes `&self`: safe to call
    /// concurrently on a shared server (each call snapshots, computes lock-free, and
    /// commits atomically — see the module docs).
    pub fn dispatch(&self, req: &Request) -> Response {
        match Toplevel::parse(req.path) {
            Toplevel::Root => self.root(req),
            Toplevel::Robots => self.get_only(req, || Response::text(assets::ROBOTS_TXT.to_string())),
            Toplevel::Favicon => self.get_only(req, || Response::see_other(assets::FAVICON_TARGET.to_string())),
            Toplevel::Kill => self.kill(req),
            Toplevel::Static(segs) => self.static_asset(req, &segs),
            Toplevel::Thy(route) => self.thy(req, &route),
            Toplevel::Other(_) => Response::not_found(req.path),
        }
    }

    /// Run `f` for a GET; a non-GET on a GET-only route answers `405`.
    fn get_only(&self, req: &Request, f: impl FnOnce() -> Response) -> Response {
        match req.method {
            HttpMethod::Get => f(),
            other => Response::bad_method(other),
        }
    }

    // ---- top-level routes ------------------------------------------------

    fn root(&self, req: &Request) -> Response {
        match req.method {
            HttpMethod::Get => Response::html(self.render_index(Flash::None)),
            HttpMethod::Post => {
                let uploaded = req.field("uploadedTheory");
                // compute: parse the uploaded source (no state lock held).
                let loaded = if uploaded.is_empty() {
                    None
                } else {
                    self.ops.load_theory(uploaded)
                };
                match loaded {
                    Some(thy) => {
                        // commit: allocate a fresh index for the upload.
                        self.state.insert_new(thy);
                        Response::html(self.render_index(Flash::Loaded))
                    }
                    None => Response::html(self.render_index(Flash::PostFailed)),
                }
            }
        }
    }

    /// Render the index page for the current version set. Each version is snapshotted
    /// (releasing the lock) and its row metadata produced lock-free; rows are emitted
    /// in ascending index order. The reference server's exact row *ordering* and the
    /// per-row time/origin are non-deterministic (prover/environment concerns).
    fn render_index(&self, flash: Flash) -> String {
        let mut version = String::new();
        // Owned strings must outlive the borrowed RootRow slice.
        let mut owned: Vec<(u64, String, String, bool, String)> = Vec::new();
        for idx in self.state.indices() {
            let thy = match self.state.snapshot(idx) {
                Some(t) => t,
                None => continue, // dropped between the index list and the snapshot
            };
            let m = self.ops.meta(&thy);
            let rm = self.ops.root_meta(&thy);
            if version.is_empty() {
                version = m.version.clone();
            }
            owned.push((idx, m.name, rm.time, rm.modified, rm.origin));
        }
        let rows: Vec<RootRow> = owned
            .iter()
            .map(|(idx, name, time, modified, origin)| RootRow {
                index: *idx,
                name,
                time,
                modified: *modified,
                origin,
            })
            .collect();
        page::render_root(flash, &version, &rows)
    }

    fn kill(&self, req: &Request) -> Response {
        match req.method {
            HttpMethod::Get => match req.query_param("path") {
                Some(_) => Response::text(assets::KILL_CANCELED.to_string()),
                None => Response::invalid_args(&[assets::KILL_NO_PATH_MSG]),
            },
            other => Response::bad_method(other),
        }
    }

    fn static_asset(&self, req: &Request, segs: &[String]) -> Response {
        match req.method {
            HttpMethod::Get => match self.ops.static_file(segs) {
                Some(bytes) => {
                    let joined = segs.join("/");
                    // Asset bytes are opaque environment data; a UTF-8 asset is
                    // reproduced exactly, a binary asset is streamed at integration.
                    let body = String::from_utf8_lossy(&bytes).into_owned();
                    Response::base(200, assets::static_content_type(&joined), body)
                }
                None => Response::static_missing(),
            },
            other => Response::bad_method(other),
        }
    }

    // ---- theory-scoped routes -------------------------------------------

    fn thy(&self, req: &Request, route: &Route) -> Response {
        // get-snapshot: resolve the index and take one owned snapshot of the version.
        // A `#` (current) index never resolves; an unknown index is a 404. This is the
        // only state read for the whole request; the compute below runs on the snapshot
        // with no lock held.
        let index = match route.index {
            Index::Num(v) => v,
            Index::Current => return Response::not_found(req.path),
        };
        let thy = match self.state.snapshot(index) {
            Some(t) => t,
            None => return Response::not_found(req.path),
        };
        let kind = shell_kind(&route.theory_kind);
        match (req.method, &route.handler) {
            // Proof operations (allocate a new version) — intercept before reads.
            (HttpMethod::Get, Handler::Main(Main::Method { lemma, n, path })) => {
                self.proof_method(kind, &thy, lemma, *n, path, false)
            }
            (HttpMethod::Get, Handler::Main(Main::DiffMethod { lemma, n, path })) => {
                self.proof_method(kind, &thy, lemma, *n, path, true)
            }
            (HttpMethod::Get, Handler::Main(m)) => self.get_main(index, &thy, m),
            (HttpMethod::Get, Handler::Overview(tail)) => {
                self.get_overview(kind, index, &thy, &OverviewView::parse(tail))
            }
            (HttpMethod::Get, Handler::Autoprove(tail)) => match Autoprove::parse(tail) {
                Some(ap) => {
                    // compute (possibly slow) on the snapshot, then commit a new version.
                    let (new_thy, focus) = self.ops.autoprove(&thy, &ap);
                    let new = self.state.insert_new(new_thy);
                    Response::json(envelope::render_redirect(&overview_proof_path(kind, new, &ap.lemma, &focus, false)))
                }
                None => Response::not_found(req.path),
            },
            (HttpMethod::Get, Handler::AutoproveDiff(tail)) => match AutoproveDiff::parse(tail) {
                Some(ap) => {
                    let (new_thy, focus) = self.ops.autoprove_diff(&thy, &ap);
                    let new = self.state.insert_new(new_thy);
                    Response::json(envelope::render_redirect(&overview_proof_path(kind, new, &ap.lemma, &focus, true)))
                }
                None => Response::not_found(req.path),
            },
            (HttpMethod::Get, Handler::AutoproveAll(tail)) => match AutoproveAll::parse(tail) {
                Some(ap) => {
                    let new_thy = self.ops.autoprove_all(&thy, &ap);
                    let new = self.state.insert_new(new_thy);
                    Response::json(envelope::render_redirect(&format!("/thy/{}/{}/overview/help", kind.path(), new)))
                }
                None => Response::not_found(req.path),
            },
            (HttpMethod::Get, Handler::Next(tail)) => self.get_nav(index, &thy, NavDir::Next, tail, req.path),
            (HttpMethod::Get, Handler::Prev(tail)) => self.get_nav(index, &thy, NavDir::Prev, tail, req.path),
            (HttpMethod::Get, Handler::Source) | (HttpMethod::Get, Handler::Message) => {
                Response::text(self.ops.source_text(&thy))
            }
            (HttpMethod::Get, Handler::Download(_)) => {
                Response::octet(self.ops.source_text(&thy))
            }
            (HttpMethod::Get, Handler::Intdot(tail)) => self.get_intdot(kind, index, &thy, tail),
            (HttpMethod::Get, Handler::InteractiveGraphDef(tail)) => {
                Response::text(self.ops.graph_dot(&thy, tail))
            }
            // Wrong method on GET-only theory routes.
            (HttpMethod::Post, Handler::Source)
            | (HttpMethod::Post, Handler::Message)
            | (HttpMethod::Post, Handler::Download(_)) => Response::bad_method(HttpMethod::Post),

            // In-place mutating POSTs.
            (HttpMethod::Post, Handler::Reload) => {
                // compute: re-parse (possibly slow) on the snapshot, then commit in place.
                let new_thy = self.ops.reload(&thy);
                self.state.replace(index, new_thy);
                Response::json(envelope::render_redirect(&format!("/thy/{}/{}/overview/help", kind.path(), index)))
            }
            (HttpMethod::Get, Handler::Reload) => Response::bad_method(HttpMethod::Get),
            (HttpMethod::Post, Handler::GetAndAppend(_)) => {
                Response::json(envelope::render_alert(&self.ops.append_message(&thy)))
            }
            (HttpMethod::Get, Handler::GetAndAppend(_)) => Response::bad_method(HttpMethod::Get),
            (HttpMethod::Post, Handler::Edit(tail)) => self.post_edit(kind, index, &thy, tail, req),

            // del/path (both kinds) and verify (trace only). The theory-path parse
            // decides route-match (unparseable -> 404 for any method); a parseable
            // path with a non-GET method is a 405.
            (_, Handler::DelPath(segs)) => self.del_path(req.method, kind, &thy, segs, req.path),
            (_, Handler::Verify(segs)) => {
                if kind == ShellKind::Trace {
                    self.verify(req.method, index, &thy, segs, req.path)
                } else {
                    // `verify` is not registered for equiv theories: a miss for any method.
                    Response::not_found(req.path)
                }
            }
            _ => Response::not_found(req.path),
        }
    }

    // ---- GET main/* : JSON envelopes ----
    fn get_main(&self, index: u64, thy: &P::Theory, m: &Main) -> Response {
        let content = match m {
            Main::Help => self.ops.main_content(thy, index, &MainReq::Help),
            Main::Message => self.ops.main_content(thy, index, &MainReq::Message),
            Main::Rules => self.ops.main_content(thy, index, &MainReq::Rules),
            Main::DiffRules => self.ops.main_content(thy, index, &MainReq::DiffRules),
            Main::Tactic => self.ops.main_content(thy, index, &MainReq::Tactic),
            Main::Cases { refined, level, n } => {
                self.ops.main_content(thy, index, &MainReq::Cases { refined: *refined, level: *level, n: *n })
            }
            Main::Lemma(name) => self.ops.main_content(thy, index, &MainReq::Lemma(name)),
            Main::Proof { lemma, path } => {
                self.ops.main_content(thy, index, &MainReq::Proof { lemma, path })
            }
            Main::DiffProof { lemma, path } => {
                self.ops.main_content(thy, index, &MainReq::DiffProof { lemma, path })
            }
            Main::Edit(name) => {
                let src = self.ops.lemma_source(thy, name).unwrap_or_default();
                Content { html: forms::edit_form(name, &src), title: format!("Edit Lemma: {name}") }
            }
            Main::Add(pos) => Content { html: forms::add_form(pos), title: "Add new Lemma".to_string() },
            Main::Delete(name) => Content { html: forms::delete_form(name), title: format!("Delete {name}") },
            // Proof ops are intercepted in `thy`; unreachable read-side.
            Main::Method { .. } | Main::DiffMethod { .. } => return Response::not_found("method"),
            Main::Other(_) => return Response::not_found("main"),
        };
        Response::json(envelope::render_content(&content.html, &content.title))
    }

    // ---- GET overview/* : full-page HTML ----
    fn get_overview(&self, kind: ShellKind, index: u64, thy: &P::Theory, view: &OverviewView) -> Response {
        let meta = self.ops.meta(thy);
        let west = self.ops.west_pane(thy, index);
        let center = self.center_for(thy, index, view);
        let params = PageParams {
            theory_name: &meta.name,
            index,
            version: &meta.version,
            filename: &meta.filename,
            origin: meta.origin,
        };
        Response::html(page::render_page_kind(kind, &params, &west, &center))
    }

    /// The center-pane inner HTML for an overview view: the corresponding `main/*`
    /// html **plus one trailing space** (BEHAVIOR §12).
    fn center_for(&self, thy: &P::Theory, index: u64, view: &OverviewView) -> String {
        let inner = match view {
            OverviewView::Help => self.ops.main_content(thy, index, &MainReq::Help).html,
            OverviewView::Proof { lemma, path } => {
                self.ops.main_content(thy, index, &MainReq::Proof { lemma, path }).html
            }
            OverviewView::DiffProof { lemma, path } => {
                self.ops.main_content(thy, index, &MainReq::DiffProof { lemma, path }).html
            }
            OverviewView::Edit(name) => {
                let src = self.ops.lemma_source(thy, name).unwrap_or_default();
                forms::edit_form(name, &src)
            }
            OverviewView::Add(pos) => forms::add_form(pos),
            OverviewView::Delete(name) => forms::delete_form(name),
            OverviewView::Other(_) => String::new(),
        };
        format!("{inner} ")
    }

    // ---- GET next/prev : text/plain bare URL ----
    fn get_nav(&self, index: u64, thy: &P::Theory, dir: NavDir, tail: &[String], path: &str) -> Response {
        match Nav::parse(dir, tail) {
            Some(nav) => Response::text(self.ops.nav_target(thy, index, dir, &nav.mode, &nav.lemma)),
            None => Response::not_found(path),
        }
    }

    // ---- GET intdot : html mini page (handler swapped intdot -> i-g-def) ----
    fn get_intdot(&self, kind: ShellKind, index: u64, thy: &P::Theory, tail: &[String]) -> Response {
        let meta = self.ops.meta(thy);
        let trailing = tail.join("/");
        let dotsrc = format!("/thy/{}/{}/interactive-graph-def/{}", kind.path(), index, trailing);
        Response::html(intdot::render_intdot(&meta.name, &dotsrc))
    }

    // ---- proof operations: new version + JSON {redirect}, or {alert} on failure ----
    fn proof_method(&self, kind: ShellKind, thy: &P::Theory, lemma: &str, n: usize, path: &[String], diff: bool) -> Response {
        // compute (possibly slow) on the snapshot with no state lock held.
        let applied = if diff {
            self.ops.apply_diff_method(thy, lemma, n, path)
        } else {
            self.ops.apply_method(thy, lemma, n, path)
        };
        match applied {
            Some((new_thy, focus)) => {
                // commit: allocate the fresh index at completion.
                let new = self.state.insert_new(new_thy);
                Response::json(envelope::render_redirect(&overview_proof_path(kind, new, lemma, &focus, diff)))
            }
            None => Response::json(envelope::render_alert(METHOD_FAILED_ALERT)),
        }
    }

    // ---- del/path : delete a theory path (proof op — new version) ----
    /// Dispatch a `del/path/…` request. The theory-path tail parses mode-aware
    /// (`kind`); an unparseable tail is a `404` (for any method), a parseable tail
    /// with a non-GET method is a `405`. A deletable path (a lemma or proof node)
    /// allocates a fresh version and redirects to `overview/<same-path>`; every
    /// other outcome is a JSON alert whose text is selected by the path type.
    fn del_path(&self, method: HttpMethod, kind: ShellKind, thy: &P::Theory, segs: &[String], path: &str) -> Response {
        let diff = kind == ShellKind::Equiv;
        let thy_path = match ThyPath::parse(segs, diff) {
            Some(tp) => tp,
            None => return Response::not_found(path),
        };
        if method != HttpMethod::Get {
            return Response::bad_method(method);
        }
        match thy_path {
            ThyPath::Lemma(name) => match self.ops.del_lemma_path(thy, &name) {
                Some(new_thy) => {
                    let new = self.state.insert_new(new_thy);
                    Response::json(envelope::render_redirect(&overview_lemma_path(kind, new, &name)))
                }
                None => Response::json(envelope::render_alert(envelope::DEL_LEMMA_FAILED_ALERT)),
            },
            ThyPath::Proof { lemma, path } => self.del_proof(kind, thy, &lemma, &path, false),
            ThyPath::DiffProof { lemma, path } => self.del_proof(kind, thy, &lemma, &path, true),
            ThyPath::Other => Response::json(envelope::render_alert(envelope::DEL_PATH_CANT_ALERT)),
        }
    }

    fn del_proof(&self, kind: ShellKind, thy: &P::Theory, lemma: &str, path: &[String], diff: bool) -> Response {
        match self.ops.del_proof_step(thy, lemma, path, diff) {
            Some(new_thy) => {
                let new = self.state.insert_new(new_thy);
                Response::json(envelope::render_redirect(&overview_proof_path(kind, new, lemma, path, diff)))
            }
            None => Response::json(envelope::render_alert(envelope::DEL_PROOF_STEP_FAILED_ALERT)),
        }
    }

    // ---- verify : re-render a theory path (trace only; never mutates) ----
    /// Dispatch a `verify/…` request (trace theories only — the caller answers a
    /// `404` in equiv mode). The theory-path tail parses in the trace grammar; an
    /// unparseable tail is a `404`, a parseable tail with a non-GET method a `405`.
    /// A `proof/{lemma}[/path]` whose lemma is present redirects to
    /// `overview/proof/{lemma}[/path]` at the **same** version; every other path
    /// (including a proof path to an absent lemma) returns the theory help pane.
    fn verify(&self, method: HttpMethod, index: u64, thy: &P::Theory, segs: &[String], path: &str) -> Response {
        let thy_path = match ThyPath::parse(segs, false) {
            Some(tp) => tp,
            None => return Response::not_found(path),
        };
        if method != HttpMethod::Get {
            return Response::bad_method(method);
        }
        match thy_path {
            ThyPath::Proof { lemma, path } if self.ops.lemma_present(thy, &lemma) => {
                Response::json(envelope::render_redirect(&overview_proof_path(
                    ShellKind::Trace, index, &lemma, &path, false,
                )))
            }
            _ => {
                let content = self.ops.main_content(thy, index, &MainReq::Help);
                Response::json(envelope::render_content(&content.html, &content.title))
            }
        }
    }

    // ---- structural POST: mutate in place; 303 (or 200 form on edit/add failure) ----
    fn post_edit(&self, kind: ShellKind, index: u64, thy: &P::Theory, tail: &[String], req: &Request) -> Response {
        let (verb, name) = match tail {
            [v, n] => match EditVerb::parse(v) {
                Some(verb) => (verb, n.clone()),
                None => return Response::not_found(req.path),
            },
            _ => return Response::not_found(req.path),
        };
        let k = kind.path();
        match verb {
            EditVerb::Delete => match self.ops.delete_lemma(thy, &name) {
                Some(new_thy) => {
                    self.state.replace(index, new_thy);
                    Response::see_other(format!("/thy/{k}/{index}/overview/help"))
                }
                // Lemma not found: theory unchanged, redirect to the delete view.
                None => Response::see_other(format!("/thy/{k}/{index}/overview/delete/{name}")),
            },
            EditVerb::Edit => {
                let text = req.field("lemma-text");
                match self.ops.edit_lemma(thy, &name, text) {
                    Some(new_thy) => {
                        self.state.replace(index, new_thy);
                        Response::see_other(format!("/thy/{k}/{index}/overview/edit/{name}"))
                    }
                    None => self.get_overview(kind, index, thy, &OverviewView::Edit(name)),
                }
            }
            EditVerb::Add => {
                let text = req.field("lemma-text");
                match self.ops.add_lemma(thy, &name, text) {
                    Some(new_thy) => {
                        self.state.replace(index, new_thy);
                        Response::see_other(format!("/thy/{k}/{index}/overview/add/{name}"))
                    }
                    None => self.get_overview(kind, index, thy, &OverviewView::Add(name)),
                }
            }
        }
    }

    /// Apply a proof method that arrived as a parsed `Main::Method`. Exposed so a
    /// caller that pre-parses `main/method` can drive the version bump directly.
    /// Snapshots `index` internally (a `404` body if the version is absent).
    pub fn apply_main_method(&self, index: u64, lemma: &str, n: usize, path: &[String]) -> Response {
        match self.state.snapshot(index) {
            Some(thy) => self.proof_method(ShellKind::Trace, &thy, lemma, n, path, false),
            None => Response::not_found("method"),
        }
    }
}

/// Map a URL theory-kind segment to a shell variant.
fn shell_kind(kind: &str) -> ShellKind {
    match kind {
        "equiv" => ShellKind::Equiv,
        _ => ShellKind::Trace,
    }
}

/// Build a `del/path/lemma/{name}` redirect target: the `overview/lemma/{name}`
/// URL at version `index`.
fn overview_lemma_path(kind: ShellKind, index: u64, name: &str) -> String {
    format!("/thy/{}/{}/overview/lemma/{}", kind.path(), index, name)
}

/// Build the redirect target for a proof operation: an `overview/proof/…` URL for
/// trace, or `overview/diffProof/…` for diff, at version `index`.
fn overview_proof_path(kind: ShellKind, index: u64, lemma: &str, focus: &[String], diff: bool) -> String {
    let handler = if diff { "diffProof" } else { "proof" };
    let mut s = format!("/thy/{}/{}/overview/{}/{}", kind.path(), index, handler, lemma);
    for seg in focus {
        s.push('/');
        s.push_str(seg);
    }
    s
}
