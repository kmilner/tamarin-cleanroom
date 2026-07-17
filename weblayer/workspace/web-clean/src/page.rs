//! The full-page theory-view HTML shell (`overview/*`) and the index page (`/`).
//!
//! Observed structure (constant across theories after substituting the
//! parameters below): a fixed `<head>` of stylesheet/script links, a north header
//! bar, then four layout panes — west "Proof scripts", east "Debug information"
//! (always empty), and center "Visualization display". The west pane embeds the
//! proof-script markup (see [`crate::proofscript`]); the center pane embeds the
//! currently-selected main content HTML.
//!
//! The shell is shared between the ordinary trace view and the observational-
//! equivalence (diff) view: the only differences are the `<title>` prefix
//! (`Theory:` vs `DiffTheory:`), the `/thy/<kind>/` segment in every internal
//! link (`trace` vs `equiv`), and the presence of the Actions-menu "Append
//! modified lemmas" item (trace only). These are captured by [`ShellKind`].
//!
//! Scaffolding constants in `shell_template` are byte-exact copies of oracle
//! output. The body has no trailing newline (ends `</html>`).

use crate::escape::html_escape;
use crate::shell_template::{
    APPEND_ITEM, PAGE_MID, PAGE_PREFIX, PAGE_TAIL, RELOAD_ITEM, ROOT_TEMPLATE,
};

/// Where a theory version was loaded from, as it affects the page shell.
///
/// Observed live (round 6): a theory loaded from an on-disk file (the server
/// command line, or a version derived from one) renders the north-bar "Reload
/// file" item and — for trace theories — the Actions-menu "Append modified
/// lemmas to file" item; a theory uploaded through `POST /` (which has no on-disk
/// file) omits both. The distinction is inherited: a proof-derived version of an
/// uploaded theory is still [`Origin::Uploaded`].
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Origin {
    /// Loaded from an on-disk source file: "Reload file" is shown, and (trace
    /// only) "Append modified lemmas to file".
    Local,
    /// Uploaded via `POST /` — no on-disk file, so both "Reload file" and "Append
    /// modified lemmas to file" are omitted regardless of theory kind.
    Uploaded,
}

/// Parameters that vary between rendered theory-view pages.
pub struct PageParams<'a> {
    /// Theory name, shown in `<title>…Theory: NAME</title>`.
    pub theory_name: &'a str,
    /// Resolved numeric theory index used in every internal URL.
    pub index: u64,
    /// Tamarin version string shown in the header (e.g. `"1.13.0"`).
    pub version: &'a str,
    /// Source filename used in the download / append links (e.g. `"foo.spthy"`).
    pub filename: &'a str,
    /// Where this version was loaded from (gates the "Reload file"/"Append"
    /// north-bar items).
    pub origin: Origin,
}

/// Which theory-view shell variant to render.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ShellKind {
    /// Ordinary trace analysis: `/thy/trace/`, `Theory:` title. The "Append
    /// modified lemmas" item is present for a Local-origin trace theory.
    Trace,
    /// Observational-equivalence (diff) analysis: `/thy/equiv/`, `DiffTheory:`
    /// title, no append item (for any origin).
    Equiv,
}

impl ShellKind {
    /// The `/thy/<kind>/` path segment.
    pub fn path(self) -> &'static str {
        match self {
            ShellKind::Trace => "trace",
            ShellKind::Equiv => "equiv",
        }
    }
    fn title_diff(self) -> &'static str {
        match self {
            ShellKind::Trace => "",
            ShellKind::Equiv => "Diff",
        }
    }
}

/// The `§RELOAD§` slot content: the "Reload file" item, present iff the theory has
/// an on-disk source file (Local origin).
fn reload_item(origin: Origin) -> &'static str {
    match origin {
        Origin::Local => RELOAD_ITEM,
        Origin::Uploaded => "",
    }
}

/// The `§APPEND§` slot content: the "Append modified lemmas to file" item, present
/// iff the theory is a trace theory (not diff) of Local origin.
fn append_item(kind: ShellKind, origin: Origin) -> &'static str {
    match (kind, origin) {
        (ShellKind::Trace, Origin::Local) => APPEND_ITEM,
        _ => "",
    }
}

/// Render the trace theory-view page (the common case).
pub fn render_page(p: &PageParams, west_inner: &str, center_inner: &str) -> String {
    render_page_kind(ShellKind::Trace, p, west_inner, center_inner)
}

/// Render the theory-view page for a given shell variant.
pub fn render_page_kind(
    kind: ShellKind,
    p: &PageParams,
    west_inner: &str,
    center_inner: &str,
) -> String {
    let idx = p.index.to_string();
    // Fill the reload/append slots first so their own KIND/IDX/FILENAME slots are
    // then resolved by the scalar substitutions below.
    let prefix = PAGE_PREFIX
        .replace("§RELOAD§", reload_item(p.origin))
        .replace("§APPEND§", append_item(kind, p.origin))
        .replace("§DIFF§", kind.title_diff())
        .replace("§NAME§", &html_escape(p.theory_name))
        .replace("§KIND§", kind.path())
        .replace("§IDX§", &idx)
        .replace("§VERSION§", p.version)
        .replace("§FILENAME§", p.filename);
    let mut out = String::with_capacity(
        prefix.len() + west_inner.len() + PAGE_MID.len() + center_inner.len() + PAGE_TAIL.len(),
    );
    out.push_str(&prefix);
    out.push_str(west_inner);
    out.push_str(PAGE_MID);
    out.push_str(center_inner);
    out.push_str(PAGE_TAIL);
    out
}

/// A flash banner shown once at the top of the index page.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Flash {
    /// No banner (a plain `GET /`).
    None,
    /// `POST /` succeeded: a new theory was loaded.
    Loaded,
    /// `POST /` failed (e.g. no file / unparseable upload).
    PostFailed,
}

impl Flash {
    fn html(self) -> &'static str {
        match self {
            Flash::None => "",
            Flash::Loaded => "<p class=\"message\">Loaded new theory!</p>",
            Flash::PostFailed => "<p class=\"message\">Post request failed.</p>",
        }
    }
}

/// One row of the index page's theory table. `time` (load time) and `origin`
/// (source path) are non-deterministic and supplied verbatim by the caller.
pub struct RootRow<'a> {
    pub index: u64,
    pub name: &'a str,
    pub time: &'a str,
    /// `true` once the theory has been modified from its loaded state (rendered
    /// as an italicised `Modified`); `false` renders a plain `Original`.
    pub modified: bool,
    pub origin: &'a str,
}

/// Render a single index-page theory row. The `Version` cell is a plain
/// `Original` or an (unclosed, as observed) `<em>Modified`.
pub fn render_root_row(r: &RootRow) -> String {
    let version_cell = if r.modified { "<em>Modified" } else { "Original" };
    format!(
        "<tr><td><a href=\"/thy/trace/{}/overview/help\">{}</a></td><td>{}</td><td>{}</td><td>{}</td></tr>",
        r.index,
        html_escape(r.name),
        r.time,
        version_cell,
        r.origin,
    )
}

/// Render the index page (`GET /` / the `POST /` result page).
pub fn render_root(flash: Flash, version: &str, rows: &[RootRow]) -> String {
    let rows_html: String = rows.iter().map(render_root_row).collect();
    ROOT_TEMPLATE
        .replace("§FLASH§", flash.html())
        .replace("§VERSION§", version)
        .replace("§ROWS§", &rows_html)
}
