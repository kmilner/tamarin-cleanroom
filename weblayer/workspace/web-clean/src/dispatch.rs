//! The interactive-UI **state machine**: route dispatch decisions, theory-version
//! management, and response assembly across the **whole** request surface.
//!
//! This owns the web layer's *behaviour* — what each route returns and how the
//! version state evolves — over a [`ProverOps`] callback trait supplied at
//! integration by the ported prover. This module never inspects a theory; it
//! only decides which callback to invoke, how the version state changes, and how
//! to wrap the result into an HTTP response.
//!
//! Everything here was derived from black-box observation (`BEHAVIOR.md` §§13–14,
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

use std::collections::BTreeMap;

use crate::envelope::METHOD_FAILED_ALERT;
use crate::page::{Flash, PageParams, RootRow, ShellKind};
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
pub trait ProverOps {
    /// Opaque per-version theory handle held by the [`Server`].
    type Theory;

    // ---- pretty-printers / fragment producers ----
    /// Shell metadata (theory name, Tamarin version string, source filename).
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

/// The interactive server's state: the version map plus the monotonic next-index
/// counter. One global index namespace covers every loaded/derived theory-version.
pub struct Server<T: ProverOps> {
    ops: T,
    versions: BTreeMap<u64, T::Theory>,
    next_index: u64,
}

impl<T: ProverOps> Server<T> {
    /// Create a server holding `base` as version 1.
    pub fn new(ops: T, base: T::Theory) -> Server<T> {
        let mut versions = BTreeMap::new();
        versions.insert(1, base);
        Server { ops, versions, next_index: 2 }
    }

    /// Version indices currently allocated (ascending). All are resolvable.
    pub fn versions(&self) -> Vec<u64> {
        self.versions.keys().copied().collect()
    }

    fn resolve(&self, index: &Index) -> Option<u64> {
        match index {
            Index::Num(v) if self.versions.contains_key(v) => Some(*v),
            _ => None,
        }
    }

    fn commit_new_version(&mut self, thy: T::Theory) -> u64 {
        let new_index = self.next_index;
        self.next_index += 1;
        self.versions.insert(new_index, thy);
        new_index
    }

    /// Dispatch a request across the whole surface, mutating version state as
    /// observed.
    pub fn dispatch(&mut self, req: &Request) -> Response {
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

    fn root(&mut self, req: &Request) -> Response {
        match req.method {
            HttpMethod::Get => Response::html(self.render_index(Flash::None)),
            HttpMethod::Post => {
                let uploaded = req.field("uploadedTheory");
                let loaded = if uploaded.is_empty() {
                    None
                } else {
                    self.ops.load_theory(uploaded)
                };
                match loaded {
                    Some(thy) => {
                        self.commit_new_version(thy);
                        Response::html(self.render_index(Flash::Loaded))
                    }
                    None => Response::html(self.render_index(Flash::PostFailed)),
                }
            }
        }
    }

    /// Render the index page for the current version set. Rows are emitted in
    /// ascending index order; the reference server's exact row *ordering* and the
    /// per-row time/origin are non-deterministic (prover/environment concerns).
    fn render_index(&self, flash: Flash) -> String {
        let mut version = String::new();
        let mut rows: Vec<RootRow> = Vec::with_capacity(self.versions.len());
        // Owned strings must outlive the borrowed RootRow slice.
        let mut owned: Vec<(u64, String, String, bool, String)> = Vec::new();
        for (&idx, thy) in &self.versions {
            let m = self.ops.meta(thy);
            let rm = self.ops.root_meta(thy);
            if version.is_empty() {
                version = m.version.clone();
            }
            owned.push((idx, m.name, rm.time, rm.modified, rm.origin));
        }
        for (idx, name, time, modified, origin) in &owned {
            rows.push(RootRow { index: *idx, name, time, modified: *modified, origin });
        }
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

    fn thy(&mut self, req: &Request, route: &Route) -> Response {
        let index = match self.resolve(&route.index) {
            Some(v) => v,
            None => return Response::not_found(req.path),
        };
        let kind = shell_kind(&route.theory_kind);
        match (req.method, &route.handler) {
            // Proof operations (allocate a new version) — intercept before reads.
            (HttpMethod::Get, Handler::Main(Main::Method { lemma, n, path })) => {
                self.proof_method(kind, index, &lemma.clone(), *n, &path.clone(), false)
            }
            (HttpMethod::Get, Handler::Main(Main::DiffMethod { lemma, n, path })) => {
                self.proof_method(kind, index, &lemma.clone(), *n, &path.clone(), true)
            }
            (HttpMethod::Get, Handler::Main(m)) => self.get_main(index, m),
            (HttpMethod::Get, Handler::Overview(tail)) => {
                self.get_overview(kind, index, &OverviewView::parse(tail))
            }
            (HttpMethod::Get, Handler::Autoprove(tail)) => match Autoprove::parse(tail) {
                Some(ap) => {
                    let (thy, focus) = self.ops.autoprove(&self.versions[&index], &ap);
                    let new = self.commit_new_version(thy);
                    Response::json(envelope::render_redirect(&overview_proof_path(kind, new, &ap.lemma, &focus, false)))
                }
                None => Response::not_found(req.path),
            },
            (HttpMethod::Get, Handler::AutoproveDiff(tail)) => match AutoproveDiff::parse(tail) {
                Some(ap) => {
                    let (thy, focus) = self.ops.autoprove_diff(&self.versions[&index], &ap);
                    let new = self.commit_new_version(thy);
                    Response::json(envelope::render_redirect(&overview_proof_path(kind, new, &ap.lemma, &focus, true)))
                }
                None => Response::not_found(req.path),
            },
            (HttpMethod::Get, Handler::AutoproveAll(tail)) => match AutoproveAll::parse(tail) {
                Some(ap) => {
                    let thy = self.ops.autoprove_all(&self.versions[&index], &ap);
                    let new = self.commit_new_version(thy);
                    Response::json(envelope::render_redirect(&format!("/thy/{}/{}/overview/help", kind.path(), new)))
                }
                None => Response::not_found(req.path),
            },
            (HttpMethod::Get, Handler::Next(tail)) => self.get_nav(index, NavDir::Next, tail, req.path),
            (HttpMethod::Get, Handler::Prev(tail)) => self.get_nav(index, NavDir::Prev, tail, req.path),
            (HttpMethod::Get, Handler::Source) | (HttpMethod::Get, Handler::Message) => {
                Response::text(self.ops.source_text(&self.versions[&index]))
            }
            (HttpMethod::Get, Handler::Download(_)) => {
                Response::octet(self.ops.source_text(&self.versions[&index]))
            }
            (HttpMethod::Get, Handler::Intdot(tail)) => self.get_intdot(kind, index, tail),
            (HttpMethod::Get, Handler::InteractiveGraphDef(tail)) => {
                Response::text(self.ops.graph_dot(&self.versions[&index], tail))
            }
            // Wrong method on GET-only theory routes.
            (HttpMethod::Post, Handler::Source)
            | (HttpMethod::Post, Handler::Message)
            | (HttpMethod::Post, Handler::Download(_)) => Response::bad_method(HttpMethod::Post),

            // In-place mutating POSTs.
            (HttpMethod::Post, Handler::Reload) => {
                let new_thy = self.ops.reload(&self.versions[&index]);
                self.versions.insert(index, new_thy);
                Response::json(envelope::render_redirect(&format!("/thy/{}/{}/overview/help", kind.path(), index)))
            }
            (HttpMethod::Get, Handler::Reload) => Response::bad_method(HttpMethod::Get),
            (HttpMethod::Post, Handler::GetAndAppend(_)) => {
                Response::json(envelope::render_alert(&self.ops.append_message(&self.versions[&index])))
            }
            (HttpMethod::Get, Handler::GetAndAppend(_)) => Response::bad_method(HttpMethod::Get),
            (HttpMethod::Post, Handler::Edit(tail)) => self.post_edit(kind, index, tail, req),

            // del/path (both kinds) and verify (trace only). The theory-path parse
            // decides route-match (unparseable -> 404 for any method); a parseable
            // path with a non-GET method is a 405.
            (_, Handler::DelPath(segs)) => self.del_path(req.method, kind, index, segs, req.path),
            (_, Handler::Verify(segs)) => {
                if kind == ShellKind::Trace {
                    self.verify(req.method, index, segs, req.path)
                } else {
                    // `verify` is not registered for equiv theories: a miss for any method.
                    Response::not_found(req.path)
                }
            }
            _ => Response::not_found(req.path),
        }
    }

    // ---- GET main/* : JSON envelopes ----
    fn get_main(&self, index: u64, m: &Main) -> Response {
        let thy = &self.versions[&index];
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
    fn get_overview(&self, kind: ShellKind, index: u64, view: &OverviewView) -> Response {
        let thy = &self.versions[&index];
        let meta = self.ops.meta(thy);
        let west = self.ops.west_pane(thy, index);
        let center = self.center_for(thy, index, view);
        let params = PageParams {
            theory_name: &meta.name,
            index,
            version: &meta.version,
            filename: &meta.filename,
        };
        Response::html(page::render_page_kind(kind, &params, &west, &center))
    }

    /// The center-pane inner HTML for an overview view: the corresponding `main/*`
    /// html **plus one trailing space** (BEHAVIOR §12).
    fn center_for(&self, thy: &T::Theory, index: u64, view: &OverviewView) -> String {
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
    fn get_nav(&self, index: u64, dir: NavDir, tail: &[String], path: &str) -> Response {
        match Nav::parse(dir, tail) {
            Some(nav) => Response::text(self.ops.nav_target(&self.versions[&index], index, dir, &nav.mode, &nav.lemma)),
            None => Response::not_found(path),
        }
    }

    // ---- GET intdot : html mini page (handler swapped intdot -> i-g-def) ----
    fn get_intdot(&self, kind: ShellKind, index: u64, tail: &[String]) -> Response {
        let meta = self.ops.meta(&self.versions[&index]);
        let trailing = tail.join("/");
        let dotsrc = format!("/thy/{}/{}/interactive-graph-def/{}", kind.path(), index, trailing);
        Response::html(intdot::render_intdot(&meta.name, &dotsrc))
    }

    // ---- proof operations: new version + JSON {redirect}, or {alert} on failure ----
    fn proof_method(&mut self, kind: ShellKind, index: u64, lemma: &str, n: usize, path: &[String], diff: bool) -> Response {
        let applied = if diff {
            self.ops.apply_diff_method(&self.versions[&index], lemma, n, path)
        } else {
            self.ops.apply_method(&self.versions[&index], lemma, n, path)
        };
        match applied {
            Some((new_thy, focus)) => {
                let new = self.commit_new_version(new_thy);
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
    fn del_path(&mut self, method: HttpMethod, kind: ShellKind, index: u64, segs: &[String], path: &str) -> Response {
        let diff = kind == ShellKind::Equiv;
        let thy_path = match ThyPath::parse(segs, diff) {
            Some(tp) => tp,
            None => return Response::not_found(path),
        };
        if method != HttpMethod::Get {
            return Response::bad_method(method);
        }
        match thy_path {
            ThyPath::Lemma(name) => match self.ops.del_lemma_path(&self.versions[&index], &name) {
                Some(new_thy) => {
                    let new = self.commit_new_version(new_thy);
                    Response::json(envelope::render_redirect(&overview_lemma_path(kind, new, &name)))
                }
                None => Response::json(envelope::render_alert(envelope::DEL_LEMMA_FAILED_ALERT)),
            },
            ThyPath::Proof { lemma, path } => self.del_proof(kind, index, &lemma, &path, false),
            ThyPath::DiffProof { lemma, path } => self.del_proof(kind, index, &lemma, &path, true),
            ThyPath::Other => Response::json(envelope::render_alert(envelope::DEL_PATH_CANT_ALERT)),
        }
    }

    fn del_proof(&mut self, kind: ShellKind, index: u64, lemma: &str, path: &[String], diff: bool) -> Response {
        match self.ops.del_proof_step(&self.versions[&index], lemma, path, diff) {
            Some(new_thy) => {
                let new = self.commit_new_version(new_thy);
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
    fn verify(&self, method: HttpMethod, index: u64, segs: &[String], path: &str) -> Response {
        let thy_path = match ThyPath::parse(segs, false) {
            Some(tp) => tp,
            None => return Response::not_found(path),
        };
        if method != HttpMethod::Get {
            return Response::bad_method(method);
        }
        let thy = &self.versions[&index];
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
    fn post_edit(&mut self, kind: ShellKind, index: u64, tail: &[String], req: &Request) -> Response {
        let (verb, name) = match tail {
            [v, n] => match EditVerb::parse(v) {
                Some(verb) => (verb, n.clone()),
                None => return Response::not_found(req.path),
            },
            _ => return Response::not_found(req.path),
        };
        let k = kind.path();
        match verb {
            EditVerb::Delete => match self.ops.delete_lemma(&self.versions[&index], &name) {
                Some(new_thy) => {
                    self.versions.insert(index, new_thy);
                    Response::see_other(format!("/thy/{k}/{index}/overview/help"))
                }
                // Lemma not found: theory unchanged, redirect to the delete view.
                None => Response::see_other(format!("/thy/{k}/{index}/overview/delete/{name}")),
            },
            EditVerb::Edit => {
                let text = req.field("lemma-text");
                match self.ops.edit_lemma(&self.versions[&index], &name, text) {
                    Some(new_thy) => {
                        self.versions.insert(index, new_thy);
                        Response::see_other(format!("/thy/{k}/{index}/overview/edit/{name}"))
                    }
                    None => self.get_overview(kind, index, &OverviewView::Edit(name)),
                }
            }
            EditVerb::Add => {
                let text = req.field("lemma-text");
                match self.ops.add_lemma(&self.versions[&index], &name, text) {
                    Some(new_thy) => {
                        self.versions.insert(index, new_thy);
                        Response::see_other(format!("/thy/{k}/{index}/overview/add/{name}"))
                    }
                    None => self.get_overview(kind, index, &OverviewView::Add(name)),
                }
            }
        }
    }

    /// Apply a proof method that arrived as a parsed `Main::Method`. Exposed so a
    /// caller that pre-parses `main/method` can drive the version bump directly.
    pub fn apply_main_method(&mut self, index: u64, lemma: &str, n: usize, path: &[String]) -> Response {
        self.proof_method(ShellKind::Trace, index, lemma, n, path, false)
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
