//! The interactive-UI **state machine**: route dispatch decisions, theory-version
//! management, and response-envelope assembly.
//!
//! This is the web layer's *behaviour* — what each route returns — as opposed to
//! the round-1/round-2 template modules (which produce the bytes of individual
//! bodies). It is parameterised over a [`ProverOps`] callback trait supplied at
//! integration by the ported prover; this module never inspects the theory, it
//! only decides which callback to invoke, how the version state changes, and how
//! to wrap the resulting fragment into an HTTP response.
//!
//! Everything here was derived from black-box observation (see `BEHAVIOR.md` §13
//! and `QUERIES.log` [Q029]–[L16]); no prover source was read.
//!
//! ## Observed semantics reproduced here
//! * **Version model.** Version 1 is the originally loaded theory; every *proof*
//!   operation (`main/method`, `autoprove`) allocates a fresh monotonically
//!   increasing index (`= previous max + 1`) and leaves earlier versions
//!   resolvable, whereas every *structural* edit (`edit`/`add`/`delete` lemma)
//!   mutates the theory **in place** at the requested index (no new index). Pure
//!   navigation and views never change the version set.
//! * **Envelopes.** Proof operations answer `200` with a JSON `{"redirect":URL}`
//!   pointing at `overview/proof/{lemma}/{focus}` of the new version. Structural
//!   POSTs answer `303 See Other` with a `Location` header (POST-redirect-GET) and
//!   an empty body; a *failed* structural edit instead answers `200` re-rendering
//!   the full-page form. `next`/`prev` answer `text/plain` with a bare URL.
//! * **Paths.** A proof path is the raw `/`-join of case-name segments (root `_`);
//!   segments are prover identifiers (`[A-Za-z0-9_]`), never percent-encoded. The
//!   focus path in a proof-op redirect is the prover's next open goal.

use std::collections::BTreeMap;

use crate::route::{
    Autoprove, EditVerb, Handler, Index, Main, Nav, NavDir, OverviewView, Route,
};
use crate::{envelope, errors, forms, intdot, page};

/// `application/json; charset=utf-8`.
pub const CT_JSON: &str = "application/json; charset=utf-8";
/// `text/html; charset=utf-8`.
pub const CT_HTML: &str = "text/html; charset=utf-8";
/// `text/plain; charset=utf-8` (used for text bodies, `next`/`prev`, and DOT).
pub const CT_TEXT: &str = "text/plain; charset=utf-8";

/// HTTP request method (only the two the UI uses are modelled).
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum HttpMethod {
    Get,
    Post,
}

/// An incoming request.
pub struct Request<'a> {
    pub method: HttpMethod,
    /// URL path, e.g. `/thy/trace/1/main/method/exec/1/_`.
    pub path: &'a str,
    /// Decoded POST form fields (e.g. `("lemma-text", "lemma foo: …")`).
    pub form: &'a [(String, String)],
}

impl<'a> Request<'a> {
    pub fn get(path: &'a str) -> Request<'a> {
        Request { method: HttpMethod::Get, path, form: &[] }
    }
    pub fn post(path: &'a str, form: &'a [(String, String)]) -> Request<'a> {
        Request { method: HttpMethod::Post, path, form }
    }
    fn field(&self, key: &str) -> &str {
        self.form
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
            .unwrap_or("")
    }
}

/// An outgoing response. `location` is set only for `303` redirects.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Response {
    pub status: u16,
    pub content_type: &'static str,
    pub body: String,
    pub location: Option<String>,
}

impl Response {
    fn json(body: String) -> Response {
        Response { status: 200, content_type: CT_JSON, body, location: None }
    }
    fn html(body: String) -> Response {
        Response { status: 200, content_type: CT_HTML, body, location: None }
    }
    fn text(body: String) -> Response {
        Response { status: 200, content_type: CT_TEXT, body, location: None }
    }
    fn see_other(location: String) -> Response {
        Response { status: 303, content_type: CT_TEXT, body: String::new(), location: Some(location) }
    }
    fn not_found(echoed_path: &str) -> Response {
        Response { status: 404, content_type: CT_HTML, body: errors::render_not_found(echoed_path), location: None }
    }
}

/// Theory metadata the page shell needs.
pub struct Meta {
    pub name: String,
    pub version: String,
    pub filename: String,
}

/// A rendered content pane (`html` + pane `title`) — the payload of a JSON
/// `{"html","title"}` envelope and of an overview page's center pane.
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
}

/// The prover-supplied callbacks. The web layer treats every returned string as an
/// opaque fragment; the *decisions* around these calls (version allocation, route
/// dispatch, envelope shape) live in [`Server::dispatch`].
pub trait ProverOps {
    /// Opaque per-version theory handle held by the [`Server`].
    type Theory;

    // ---- pretty-printers / fragment producers ----
    /// Shell metadata (theory name, Tamarin version string, source filename).
    fn meta(&self, thy: &Self::Theory) -> Meta;
    /// The `source`/`message` body: the pretty-printed theory source, verbatim.
    fn source_text(&self, thy: &Self::Theory) -> String;
    /// Inner HTML of the west proof-script pane at `index`.
    fn west_pane(&self, thy: &Self::Theory, index: u64) -> String;
    /// Center-pane content (html + title) for a `main/*` view at `index`.
    fn main_content(&self, thy: &Self::Theory, index: u64, req: &MainReq) -> Content;
    /// Raw (unescaped) current source of a lemma, for the edit form.
    fn lemma_source(&self, thy: &Self::Theory, name: &str) -> Option<String>;
    /// Graphviz DOT for an `interactive-graph-def` node.
    fn graph_dot(&self, thy: &Self::Theory, tail: &[String]) -> String;
    /// The `next`/`prev` target URL (bare path), computed by the prover's proof-tree
    /// traversal for `mode`/`lemma` at `index`.
    fn nav_target(&self, thy: &Self::Theory, index: u64, dir: NavDir, mode: &str, lemma: &str) -> String;

    // ---- mutations ----
    /// Apply proof method `n` at `(lemma, path)`. Returns the new theory and the
    /// resulting focus path (the prover's next open goal, incl. the root `_`).
    fn apply_method(&self, thy: &Self::Theory, lemma: &str, n: usize, path: &[String]) -> (Self::Theory, Vec<String>);
    /// Autoprove per `spec`. Returns the new theory and resulting focus path.
    fn autoprove(&self, thy: &Self::Theory, spec: &Autoprove) -> (Self::Theory, Vec<String>);
    /// Edit lemma `name` in place. `Some` = the modified theory; `None` = parse/wf
    /// failure (the web layer then re-renders the form with the theory unchanged).
    fn edit_lemma(&self, thy: &Self::Theory, name: &str, text: &str) -> Option<Self::Theory>;
    /// Add a lemma at position `pos`. `Some`/`None` as for [`Self::edit_lemma`].
    fn add_lemma(&self, thy: &Self::Theory, pos: &str, text: &str) -> Option<Self::Theory>;
    /// Delete lemma `name` in place, returning the modified theory.
    fn delete_lemma(&self, thy: &Self::Theory, name: &str) -> Self::Theory;
}

/// The interactive server's per-theory state: the version map plus the monotonic
/// next-index counter. Generic over the prover callbacks.
pub struct Server<T: ProverOps> {
    ops: T,
    /// index -> theory version. Version 1 is the originally loaded theory.
    versions: BTreeMap<u64, T::Theory>,
    /// Next free version index (monotonic; `= max ever allocated + 1`).
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
        // Real requests always carry a numeric index; `#` is a crawl placeholder
        // that never reaches a live server (a literal `#` is a URL fragment, and
        // `%23` 404s — live [L2]), so it resolves to nothing here.
        match index {
            Index::Num(v) if self.versions.contains_key(v) => Some(*v),
            _ => None,
        }
    }

    /// Dispatch a request to a response, mutating version state as observed.
    pub fn dispatch(&mut self, req: &Request) -> Response {
        let route = match Route::parse(req.path) {
            Some(r) => r,
            None => return Response::not_found(req.path),
        };
        let index = match self.resolve(&route.index) {
            Some(v) => v,
            None => return Response::not_found(req.path),
        };
        match (req.method, &route.handler) {
            // `main/method` is delivered under `main` but is a proof operation
            // (allocates a new version); intercept it before the read-only views.
            (HttpMethod::Get, Handler::Main(Main::Method { lemma, n, path })) => {
                self.get_method(index, &lemma.clone(), *n, &path.clone())
            }
            (HttpMethod::Get, Handler::Main(m)) => self.get_main(index, m),
            (HttpMethod::Get, Handler::Overview(tail)) => self.get_overview(index, &OverviewView::parse(tail)),
            (HttpMethod::Get, Handler::Autoprove(tail)) => match Autoprove::parse(tail) {
                Some(ap) => self.proof_op_autoprove(index, &ap),
                None => Response::not_found(req.path),
            },
            (HttpMethod::Get, Handler::Next(tail)) => self.get_nav(index, NavDir::Next, tail, req.path),
            (HttpMethod::Get, Handler::Prev(tail)) => self.get_nav(index, NavDir::Prev, tail, req.path),
            (HttpMethod::Get, Handler::Source) | (HttpMethod::Get, Handler::Message) => {
                Response::text(self.ops.source_text(&self.versions[&index]))
            }
            (HttpMethod::Get, Handler::Intdot(tail)) => self.get_intdot(index, tail),
            (HttpMethod::Get, Handler::InteractiveGraphDef(tail)) => {
                Response::text(self.ops.graph_dot(&self.versions[&index], tail))
            }
            // Structural-edit POST form targets: /thy/trace/V/edit/{verb}/{name}
            (HttpMethod::Post, Handler::Other { name, tail }) if name == "edit" => {
                self.post_edit(index, tail, req)
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
            Main::Tactic => self.ops.main_content(thy, index, &MainReq::Tactic),
            Main::Cases { refined, level, n } => {
                self.ops.main_content(thy, index, &MainReq::Cases { refined: *refined, level: *level, n: *n })
            }
            Main::Lemma(name) => self.ops.main_content(thy, index, &MainReq::Lemma(name)),
            Main::Proof { lemma, path } => {
                self.ops.main_content(thy, index, &MainReq::Proof { lemma, path })
            }
            // Forms are web-layer templates; only the edit form needs prover input.
            Main::Edit(name) => {
                let src = self.ops.lemma_source(thy, name).unwrap_or_default();
                Content { html: forms::edit_form(name, &src), title: format!("Edit Lemma: {name}") }
            }
            Main::Add(pos) => Content { html: forms::add_form(pos), title: "Add new Lemma".to_string() },
            Main::Delete(name) => Content { html: forms::delete_form(name), title: format!("Delete {name}") },
            // `Method` is a proof op intercepted in `dispatch`; unreachable here.
            Main::Method { .. } => return Response::not_found("method"),
            Main::Other(_) => return Response::not_found("main"),
        };
        Response::json(envelope::render_content(&content.html, &content.title))
    }

    // ---- GET overview/* : full-page HTML ----
    fn get_overview(&self, index: u64, view: &OverviewView) -> Response {
        let thy = &self.versions[&index];
        let meta = self.ops.meta(thy);
        let west = self.ops.west_pane(thy, index);
        let center = self.center_for(thy, index, view);
        let params = page::PageParams {
            theory_name: &meta.name,
            index,
            version: &meta.version,
            filename: &meta.filename,
        };
        Response::html(page::render_page(&params, &west, &center))
    }

    /// The center-pane inner HTML for an overview view. Observed (BEHAVIOR §12):
    /// the center pane is the corresponding `main/*` html **plus one trailing
    /// space**.
    fn center_for(&self, thy: &T::Theory, index: u64, view: &OverviewView) -> String {
        let inner = match view {
            OverviewView::Help => self.ops.main_content(thy, index, &MainReq::Help).html,
            OverviewView::Proof { lemma, path } => {
                self.ops.main_content(thy, index, &MainReq::Proof { lemma, path }).html
            }
            OverviewView::Edit(name) => {
                let src = self.ops.lemma_source(thy, name).unwrap_or_default();
                forms::edit_form(name, &src)
            }
            OverviewView::Add(pos) => forms::add_form(pos),
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

    // ---- GET intdot : html mini page (handler swapped intdot -> i-g-def, same tail) ----
    fn get_intdot(&self, index: u64, tail: &[String]) -> Response {
        let meta = self.ops.meta(&self.versions[&index]);
        let trailing = tail.join("/");
        Response::html(intdot::render_intdot(&meta.name, &intdot::dotsrc_path(index, &trailing)))
    }

    // ---- proof operations: allocate a NEW version, answer JSON {redirect} ----
    fn get_method(&mut self, index: u64, lemma: &str, n: usize, path: &[String]) -> Response {
        let (new_thy, focus) = self.ops.apply_method(&self.versions[&index], lemma, n, path);
        let new_index = self.commit_new_version(new_thy);
        Response::json(envelope::render_redirect(&overview_proof_path(new_index, lemma, &focus)))
    }

    fn proof_op_autoprove(&mut self, index: u64, ap: &Autoprove) -> Response {
        let (new_thy, focus) = self.ops.autoprove(&self.versions[&index], ap);
        let new_index = self.commit_new_version(new_thy);
        Response::json(envelope::render_redirect(&overview_proof_path(new_index, &ap.lemma, &focus)))
    }

    fn commit_new_version(&mut self, thy: T::Theory) -> u64 {
        let new_index = self.next_index;
        self.next_index += 1;
        self.versions.insert(new_index, thy);
        new_index
    }

    // ---- structural POST: mutate IN PLACE, answer 303 (or 200 form on failure) ----
    fn post_edit(&mut self, index: u64, tail: &[String], req: &Request) -> Response {
        let (verb, name) = match tail {
            [v, n] => match EditVerb::parse(v) {
                Some(verb) => (verb, n.clone()),
                None => return Response::not_found(req.path),
            },
            _ => return Response::not_found(req.path),
        };
        match verb {
            EditVerb::Delete => {
                let new_thy = self.ops.delete_lemma(&self.versions[&index], &name);
                self.versions.insert(index, new_thy);
                Response::see_other(format!("/thy/trace/{index}/overview/help"))
            }
            EditVerb::Edit => {
                let text = req.field("lemma-text");
                match self.ops.edit_lemma(&self.versions[&index], &name, text) {
                    Some(new_thy) => {
                        self.versions.insert(index, new_thy);
                        Response::see_other(format!("/thy/trace/{index}/overview/edit/{name}"))
                    }
                    // Failure: theory unchanged, re-render the full-page edit form.
                    None => self.get_overview(index, &OverviewView::Edit(name)),
                }
            }
            EditVerb::Add => {
                let text = req.field("lemma-text");
                match self.ops.add_lemma(&self.versions[&index], &name, text) {
                    Some(new_thy) => {
                        self.versions.insert(index, new_thy);
                        Response::see_other(format!("/thy/trace/{index}/overview/add/{name}"))
                    }
                    None => self.get_overview(index, &OverviewView::Add(name)),
                }
            }
        }
    }

    /// Apply a proof method that arrived as a parsed `Main::Method` (proof op).
    /// Exposed so a caller that pre-parses `main/method` can drive the version
    /// bump directly; the normal path routes GET `main/method` through here.
    pub fn apply_main_method(&mut self, index: u64, lemma: &str, n: usize, path: &[String]) -> Response {
        self.get_method(index, lemma, n, path)
    }
}

/// Build an `overview/proof/{lemma}[/focus…]` URL for version `index`.
fn overview_proof_path(index: u64, lemma: &str, focus: &[String]) -> String {
    let mut s = format!("/thy/trace/{index}/overview/proof/{lemma}");
    for seg in focus {
        s.push('/');
        s.push_str(seg);
    }
    s
}

