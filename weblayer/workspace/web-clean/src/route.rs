//! Route model for the interactive web UI.
//!
//! Every observed request path has the shape
//! `/thy/<theory-kind>/<index>/<handler>/<args…>` where:
//! * `<theory-kind>` is the analysis kind — only `trace` appears in the corpus
//!   (a `diff` kind is plausible but unobserved).
//! * `<index>` is either `#` (the "current" theory version) or a decimal number.
//! * `<handler>` selects the response family. Observed handlers and their
//!   response `kind`:
//!     - `main/…`                    -> JSON envelope (`{html,title}` / `{redirect}`)
//!     - `overview/…`                -> full-page HTML
//!     - `intdot/…`                  -> HTML mini-page
//!     - `interactive-graph-def/…`   -> DOT
//!     - `next/…`, `prev/…`          -> text (a navigation URL)
//!     - `autoprove/…`               -> JSON (`{redirect}`), or text on timeout
//!     - `source`, `message`         -> text (theory source)
//!
//! Under `main`, the sub-handlers are: `help`, `message`, `rules`, `tactic`,
//! `cases/{raw|refined}/{level}/{n}`, `lemma/{name}`, `add/{pos}`,
//! `edit/{name}`, `delete/{name}`, `method/{lemma}/{n}`, and
//! `proof/{lemma}/{path…}` (the proof path is a sequence of case-name segments,
//! the root being `_`).
//!
//! This module parses a path into a structured value; it is descriptive (the
//! route grammar as observed), not a dispatcher.

/// Theory version selector.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Index {
    /// `#` — the server's "current" theory version.
    Current,
    /// An explicit decimal version index.
    Num(u64),
}

impl Index {
    fn parse(s: &str) -> Option<Index> {
        if s == "#" {
            Some(Index::Current)
        } else {
            s.parse::<u64>().ok().map(Index::Num)
        }
    }
}

/// A `main/*` sub-handler.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Main {
    Help,
    Message,
    Rules,
    Tactic,
    Cases { refined: bool, level: usize, n: usize },
    Lemma(String),
    Add(String),
    Edit(String),
    Delete(String),
    /// Apply proof method `n` at the node identified by `path` (a `/`-joined
    /// sequence of case-name segments, root `_`). The method number precedes the
    /// path in the URL: `method/{lemma}/{n}[/path…]` (observed live [L8] and in
    /// 67988 body links [Q032]).
    Method { lemma: String, n: usize, path: Vec<String> },
    Proof { lemma: String, path: Vec<String> },
    /// Observational-equivalence proof view (`main/diffProof/{lemma}[/path…]`),
    /// the diff-mode analogue of `Proof`.
    DiffProof { lemma: String, path: Vec<String> },
    /// Apply a diff proof method (`main/diffMethod/{lemma}/{n}[/path…]`), the
    /// diff-mode analogue of `Method`.
    DiffMethod { lemma: String, n: usize, path: Vec<String> },
    /// The diff construction/deconstruction rules view (`main/diffrules`).
    DiffRules,
    /// Any unrecognized `main/*` tail.
    Other(Vec<String>),
}

/// The selected handler and its arguments.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Handler {
    Source,
    Message,
    Main(Main),
    Overview(Vec<String>),
    Intdot(Vec<String>),
    InteractiveGraphDef(Vec<String>),
    Next(Vec<String>),
    Prev(Vec<String>),
    Autoprove(Vec<String>),
    /// Autoprove a single diff lemma (`autoproveDiff/{strategy}/{bound}/diffProof/…`).
    AutoproveDiff(Vec<String>),
    /// Autoprove all lemmas (`autoproveAll/{strategy}/{bound}`).
    AutoproveAll(Vec<String>),
    /// Re-read the theory from disk in place (`reload`; POST).
    Reload,
    /// Download the theory source (`download/{file}`; GET). The filename is a
    /// decorative URL segment — the body is always the current theory's source.
    Download(String),
    /// Append modified lemmas to the on-disk file (`get_and_append/{file}`; POST).
    GetAndAppend(String),
    /// A structural-edit form target (`edit/{verb}/{name}`; POST). The tail is the
    /// `{verb}/{name}` remainder.
    Edit(Vec<String>),
    /// Delete a theory path (`del/path/{theory-path…}`; GET). The tail is the
    /// theory-path segments after the fixed `path` literal. It is parsed into a
    /// [`ThyPath`] by the dispatcher (mode-aware): an unparseable tail is a `404`,
    /// a parseable one with a non-GET method is a `405`.
    DelPath(Vec<String>),
    /// Verify a theory path (`verify/{theory-path…}`; GET; trace theories only).
    /// The tail is the theory-path segments, parsed into a [`ThyPath`] by the
    /// dispatcher.
    Verify(Vec<String>),
    /// Unrecognized handler with its raw tail.
    Other { name: String, tail: Vec<String> },
}

/// A parsed route.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Route {
    pub theory_kind: String,
    pub index: Index,
    pub handler: Handler,
}

fn owned(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

fn parse_main(tail: &[&str]) -> Main {
    match tail {
        ["help"] => Main::Help,
        ["message"] => Main::Message,
        ["rules"] => Main::Rules,
        ["diffrules"] => Main::DiffRules,
        ["tactic"] => Main::Tactic,
        ["cases", kind, level, n]
            if (*kind == "raw" || *kind == "refined")
                && level.parse::<usize>().is_ok()
                && n.parse::<usize>().is_ok() =>
        {
            Main::Cases {
                refined: *kind == "refined",
                level: level.parse().unwrap(),
                n: n.parse().unwrap(),
            }
        }
        ["lemma", name] => Main::Lemma((*name).to_string()),
        ["add", pos] => Main::Add((*pos).to_string()),
        ["edit", name] => Main::Edit((*name).to_string()),
        ["delete", name] => Main::Delete((*name).to_string()),
        ["method", lemma, n, rest @ ..] if n.parse::<usize>().is_ok() => Main::Method {
            lemma: (*lemma).to_string(),
            n: n.parse().unwrap(),
            path: owned(rest),
        },
        ["diffMethod", lemma, n, rest @ ..] if n.parse::<usize>().is_ok() => Main::DiffMethod {
            lemma: (*lemma).to_string(),
            n: n.parse().unwrap(),
            path: owned(rest),
        },
        ["diffProof", lemma, rest @ ..] => Main::DiffProof {
            lemma: (*lemma).to_string(),
            path: owned(rest),
        },
        [proof, lemma, rest @ ..] if *proof == "proof" => Main::Proof {
            lemma: (*lemma).to_string(),
            path: owned(rest),
        },
        _ => Main::Other(owned(tail)),
    }
}

impl Route {
    /// Parse a request path such as `/thy/trace/#/main/proof/exec/_/B_2`.
    /// Returns `None` if the path is not under `/thy/<kind>/<index>/…`.
    pub fn parse(path: &str) -> Option<Route> {
        let trimmed = path.strip_prefix('/').unwrap_or(path);
        let segs: Vec<&str> = trimmed.split('/').collect();
        // Need at least: thy / kind / index / handler
        if segs.len() < 4 || segs[0] != "thy" {
            return None;
        }
        let theory_kind = segs[1].to_string();
        let index = Index::parse(segs[2])?;
        let handler_name = segs[3];
        let tail = &segs[4..];
        let handler = match handler_name {
            "source" => Handler::Source,
            "message" => Handler::Message,
            "main" => Handler::Main(parse_main(tail)),
            "overview" => Handler::Overview(owned(tail)),
            "intdot" => Handler::Intdot(owned(tail)),
            "interactive-graph-def" => Handler::InteractiveGraphDef(owned(tail)),
            "next" => Handler::Next(owned(tail)),
            "prev" => Handler::Prev(owned(tail)),
            "autoprove" => Handler::Autoprove(owned(tail)),
            "autoproveDiff" => Handler::AutoproveDiff(owned(tail)),
            "autoproveAll" => Handler::AutoproveAll(owned(tail)),
            "reload" => Handler::Reload,
            "download" => match tail {
                [file] => Handler::Download((*file).to_string()),
                _ => Handler::Other { name: "download".to_string(), tail: owned(tail) },
            },
            "get_and_append" => match tail {
                [file] => Handler::GetAndAppend((*file).to_string()),
                _ => Handler::Other { name: "get_and_append".to_string(), tail: owned(tail) },
            },
            "edit" => Handler::Edit(owned(tail)),
            // `del/path/<theory-path…>` — the fixed `path` literal must follow.
            "del" => match tail {
                ["path", rest @ ..] => Handler::DelPath(owned(rest)),
                _ => Handler::Other { name: "del".to_string(), tail: owned(tail) },
            },
            "verify" => Handler::Verify(owned(tail)),
            other => Handler::Other {
                name: other.to_string(),
                tail: owned(tail),
            },
        };
        Some(Route {
            theory_kind,
            index,
            handler,
        })
    }
}

/// The full request surface: the top-level (non-`/thy`) routes plus the
/// theory-scoped `/thy/<kind>/<index>/…` routes.
///
/// `Toplevel::parse` takes the URL **path** only (any `?query` must be split off
/// by the caller and passed separately to the dispatcher).
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Toplevel {
    /// `/` — the index page (GET) / theory upload (POST).
    Root,
    /// `/robots.txt`.
    Robots,
    /// `/favicon.ico` — a `303` redirect to the static icon.
    Favicon,
    /// `/kill` — cancel a running proof search (requires a `path` query arg).
    Kill,
    /// `/static/<path…>` — a filesystem-backed asset.
    Static(Vec<String>),
    /// A theory-scoped route.
    Thy(Route),
    /// Anything else (→ 404).
    Other(String),
}

impl Toplevel {
    pub fn parse(path: &str) -> Toplevel {
        let trimmed = path.strip_prefix('/').unwrap_or(path);
        if trimmed.is_empty() {
            return Toplevel::Root;
        }
        let segs: Vec<&str> = trimmed.split('/').collect();
        match segs[0] {
            "robots.txt" if segs.len() == 1 => Toplevel::Robots,
            "favicon.ico" if segs.len() == 1 => Toplevel::Favicon,
            "kill" if segs.len() == 1 => Toplevel::Kill,
            "static" => Toplevel::Static(owned(&segs[1..])),
            "thy" => match Route::parse(path) {
                Some(r) => Toplevel::Thy(r),
                None => Toplevel::Other(path.to_string()),
            },
            _ => Toplevel::Other(path.to_string()),
        }
    }
}

/// A parsed `autoprove/{strategy}/{bound}/{allSol}/proof/{lemma}[/path…]` request.
///
/// Observed variant matrix (corpus [Q030], keyboard help [Q031]): `strategy` is
/// `idfs` (solve/prove) or `characterize` (characterization, e.g. exists-trace);
/// `bound` is a depth bound (`0` = unbounded — the `a`/`A`/`s`/`S` shortcuts; `5`
/// = bounded — the `b`/`B` shortcuts); `all_solutions` is the `True`/`False` flag
/// (`False` = stop after the first solution, the lowercase shortcuts; `True` =
/// search for all solutions, the uppercase shortcuts).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Autoprove {
    pub strategy: String,
    pub bound: u64,
    pub all_solutions: bool,
    pub lemma: String,
    pub path: Vec<String>,
}

impl Autoprove {
    /// Parse the tail of an `autoprove/…` route (everything after `autoprove/`).
    pub fn parse(tail: &[String]) -> Option<Autoprove> {
        match tail {
            [strategy, bound, flag, proof, lemma, rest @ ..] if proof == "proof" => {
                Some(Autoprove {
                    strategy: strategy.clone(),
                    bound: bound.parse().ok()?,
                    all_solutions: parse_bool(flag)?,
                    lemma: lemma.clone(),
                    path: rest.to_vec(),
                })
            }
            _ => None,
        }
    }
}

/// A parsed `autoproveDiff/{strategy}/{bound}/diffProof/{lemma}[/side…]` request
/// (observational-equivalence autoprove). Unlike [`Autoprove`], the diff form
/// carries **no** all-solutions flag (observed in the diff-proof body links).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AutoproveDiff {
    pub strategy: String,
    pub bound: u64,
    pub lemma: String,
    pub path: Vec<String>,
}

impl AutoproveDiff {
    /// Parse the tail of an `autoproveDiff/…` route.
    pub fn parse(tail: &[String]) -> Option<AutoproveDiff> {
        match tail {
            [strategy, bound, marker, lemma, rest @ ..] if marker == "diffProof" => {
                Some(AutoproveDiff {
                    strategy: strategy.clone(),
                    bound: bound.parse().ok()?,
                    lemma: lemma.clone(),
                    path: rest.to_vec(),
                })
            }
            _ => None,
        }
    }
}

/// A parsed `autoproveAll/{strategy}/{bound}` request (autoprove every lemma).
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AutoproveAll {
    pub strategy: String,
    pub bound: u64,
}

impl AutoproveAll {
    /// Parse the tail of an `autoproveAll/…` route.
    pub fn parse(tail: &[String]) -> Option<AutoproveAll> {
        match tail {
            [strategy, bound] => Some(AutoproveAll {
                strategy: strategy.clone(),
                bound: bound.parse().ok()?,
            }),
            _ => None,
        }
    }
}

/// Navigation direction for `next`/`prev`.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum NavDir {
    Next,
    Prev,
}

/// A parsed `next|prev/{mode}/proof/{lemma}` request. `mode` (`normal` observed;
/// the server also accepts other tokens and passes them to the prover) selects the
/// prover's traversal function.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Nav {
    pub dir: NavDir,
    pub mode: String,
    pub lemma: String,
}

impl Nav {
    pub fn parse(dir: NavDir, tail: &[String]) -> Option<Nav> {
        match tail {
            [mode, proof, lemma] if proof == "proof" => Some(Nav {
                dir,
                mode: mode.clone(),
                lemma: lemma.clone(),
            }),
            _ => None,
        }
    }
}

/// The full-page `overview/*` views. `help` and `proof/…` appear in the crawl;
/// `edit/{name}` and `add/{pos}` appear only as POST-redirect (303) targets
/// (live [L12]).
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum OverviewView {
    Help,
    Proof { lemma: String, path: Vec<String> },
    /// The diff-mode proof view (`overview/diffProof/{lemma}[/path…]`), the target
    /// of a `diffMethod`/`autoproveDiff` redirect.
    DiffProof { lemma: String, path: Vec<String> },
    Edit(String),
    Add(String),
    /// The delete-confirmation view (`overview/delete/{name}`), the target of a
    /// failed (lemma-not-found) delete.
    Delete(String),
    Other(Vec<String>),
}

impl OverviewView {
    pub fn parse(tail: &[String]) -> OverviewView {
        match tail.iter().map(String::as_str).collect::<Vec<_>>().as_slice() {
            ["help"] => OverviewView::Help,
            ["proof", lemma, rest @ ..] => OverviewView::Proof {
                lemma: (*lemma).to_string(),
                path: rest.iter().map(|s| s.to_string()).collect(),
            },
            ["diffProof", lemma, rest @ ..] => OverviewView::DiffProof {
                lemma: (*lemma).to_string(),
                path: rest.iter().map(|s| s.to_string()).collect(),
            },
            ["edit", name] => OverviewView::Edit((*name).to_string()),
            ["add", pos] => OverviewView::Add((*pos).to_string()),
            ["delete", name] => OverviewView::Delete((*name).to_string()),
            _ => OverviewView::Other(tail.to_vec()),
        }
    }
}

/// The structural-edit verb of a POST `edit/{verb}/{name}` form target
/// (live [L12]/[L13]).
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EditVerb {
    Edit,
    Add,
    Delete,
}

impl EditVerb {
    pub fn parse(s: &str) -> Option<EditVerb> {
        match s {
            "edit" => Some(EditVerb::Edit),
            "add" => Some(EditVerb::Add),
            "delete" => Some(EditVerb::Delete),
            _ => None,
        }
    }
}

/// A theory navigation path, as accepted by the `del/path/…` and `verify/…`
/// routes. The accepted heads are **mode-dependent**: a `trace` theory accepts
/// `help` · `message` · `rules` · `tactic` · `cases/{raw|refined}/{level}/{n}` ·
/// `lemma/{name}` · `proof/{lemma}[/seg…]` · `method/{lemma}/{n}[/seg…]` ·
/// `add/{pos}` · `edit/{name}` · `delete/{name}`; an `equiv` (diff) theory accepts
/// `help` · `diffrules` · `diffProof/{lemma}[/side…]` · `diffMethod/{lemma}/{n}[/…]`.
/// A tail outside the grammar does not parse — the route then answers `404`.
///
/// Only the lemma and proof nodes carry data the dispatcher needs; every other
/// accepted head collapses to [`ThyPath::Other`].
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ThyPath {
    /// `lemma/{name}` (trace).
    Lemma(String),
    /// `proof/{lemma}[/seg…]` (trace).
    Proof { lemma: String, path: Vec<String> },
    /// `diffProof/{lemma}[/side…]` (equiv).
    DiffProof { lemma: String, path: Vec<String> },
    /// Any other accepted theory path (a navigable view that is neither a lemma
    /// nor a proof node).
    Other,
}

impl ThyPath {
    /// Parse a theory-path tail. `diff` selects the equiv (diff-theory) grammar.
    /// Returns `None` for a tail outside the grammar (the caller then answers a
    /// `404`).
    pub fn parse(segs: &[String], diff: bool) -> Option<ThyPath> {
        let s: Vec<&str> = segs.iter().map(String::as_str).collect();
        if diff {
            match s.as_slice() {
                ["help"] | ["diffrules"] => Some(ThyPath::Other),
                ["diffProof", lemma, rest @ ..] => Some(ThyPath::DiffProof {
                    lemma: (*lemma).to_string(),
                    path: owned(rest),
                }),
                ["diffMethod", _lemma, n, ..] if n.parse::<usize>().is_ok() => Some(ThyPath::Other),
                _ => None,
            }
        } else {
            match s.as_slice() {
                ["help"] | ["message"] | ["rules"] | ["tactic"] => Some(ThyPath::Other),
                ["cases", kind, level, n]
                    if (*kind == "raw" || *kind == "refined")
                        && level.parse::<usize>().is_ok()
                        && n.parse::<usize>().is_ok() =>
                {
                    Some(ThyPath::Other)
                }
                ["lemma", name] => Some(ThyPath::Lemma((*name).to_string())),
                ["proof", lemma, rest @ ..] => Some(ThyPath::Proof {
                    lemma: (*lemma).to_string(),
                    path: owned(rest),
                }),
                ["method", _lemma, n, ..] if n.parse::<usize>().is_ok() => Some(ThyPath::Other),
                ["add", _] | ["edit", _] | ["delete", _] => Some(ThyPath::Other),
                _ => None,
            }
        }
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s {
        "True" => Some(true),
        "False" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_main_rules() {
        let r = Route::parse("/thy/trace/#/main/rules").unwrap();
        assert_eq!(r.theory_kind, "trace");
        assert_eq!(r.index, Index::Current);
        assert_eq!(r.handler, Handler::Main(Main::Rules));
    }

    #[test]
    fn parses_cases_with_numbers() {
        let r = Route::parse("/thy/trace/1/main/cases/refined/0/2").unwrap();
        assert_eq!(r.index, Index::Num(1));
        assert_eq!(
            r.handler,
            Handler::Main(Main::Cases { refined: true, level: 0, n: 2 })
        );
    }

    #[test]
    fn parses_proof_path() {
        let r = Route::parse("/thy/trace/3/main/proof/exec/_/B_2").unwrap();
        assert_eq!(
            r.handler,
            Handler::Main(Main::Proof {
                lemma: "exec".to_string(),
                path: vec!["_".to_string(), "B_2".to_string()],
            })
        );
    }

    #[test]
    fn parses_method_and_dot_and_text() {
        assert_eq!(
            Route::parse("/thy/trace/#/main/method/exec/1").unwrap().handler,
            Handler::Main(Main::Method { lemma: "exec".to_string(), n: 1, path: vec![] })
        );
        assert_eq!(
            Route::parse("/thy/trace/3/main/method/exec/2/_/B_2").unwrap().handler,
            Handler::Main(Main::Method {
                lemma: "exec".to_string(),
                n: 2,
                path: vec!["_".to_string(), "B_2".to_string()],
            })
        );
        assert!(matches!(
            Route::parse("/thy/trace/#/interactive-graph-def/proof/exec").unwrap().handler,
            Handler::InteractiveGraphDef(_)
        ));
        assert_eq!(
            Route::parse("/thy/trace/#/source").unwrap().handler,
            Handler::Source
        );
    }

    #[test]
    fn rejects_non_thy() {
        assert!(Route::parse("/static/css/x.css").is_none());
        assert!(Route::parse("/").is_none());
    }

    #[test]
    fn parses_autoprove_variants() {
        let r = Route::parse("/thy/trace/1/autoprove/idfs/0/False/proof/types").unwrap();
        let tail = match r.handler {
            Handler::Autoprove(t) => t,
            _ => panic!("not autoprove"),
        };
        assert_eq!(
            Autoprove::parse(&tail).unwrap(),
            Autoprove {
                strategy: "idfs".into(),
                bound: 0,
                all_solutions: false,
                lemma: "types".into(),
                path: vec![],
            }
        );
        // characterize, bounded, all-solutions, with a path.
        let r = Route::parse("/thy/trace/1/autoprove/characterize/5/True/proof/L/_/B_2").unwrap();
        let tail = match r.handler {
            Handler::Autoprove(t) => t,
            _ => panic!(),
        };
        let ap = Autoprove::parse(&tail).unwrap();
        assert_eq!(ap.strategy, "characterize");
        assert_eq!(ap.bound, 5);
        assert!(ap.all_solutions);
        assert_eq!(ap.path, vec!["_".to_string(), "B_2".to_string()]);
    }

    #[test]
    fn parses_del_path_and_verify_handlers() {
        // del/path strips the fixed `path` literal into the theory-path tail.
        assert_eq!(
            Route::parse("/thy/trace/1/del/path/lemma/debug").unwrap().handler,
            Handler::DelPath(vec!["lemma".into(), "debug".into()])
        );
        // `del` without the `path` literal is not a del/path route.
        assert!(matches!(
            Route::parse("/thy/trace/1/del/lemma/debug").unwrap().handler,
            Handler::Other { .. }
        ));
        assert_eq!(
            Route::parse("/thy/trace/1/verify/proof/debug/_/B_2").unwrap().handler,
            Handler::Verify(vec!["proof".into(), "debug".into(), "_".into(), "B_2".into()])
        );
    }

    #[test]
    fn thy_path_grammar_is_mode_dependent() {
        let sp = |s: &str| s.split('/').map(String::from).collect::<Vec<_>>();
        // Trace grammar.
        assert_eq!(ThyPath::parse(&sp("lemma/debug"), false), Some(ThyPath::Lemma("debug".into())));
        assert_eq!(
            ThyPath::parse(&sp("proof/debug/_/ONE"), false),
            Some(ThyPath::Proof { lemma: "debug".into(), path: vec!["_".into(), "ONE".into()] })
        );
        for ok in ["help", "message", "rules", "tactic", "cases/raw/0/0", "method/debug/1", "add/x", "edit/x", "delete/x"] {
            assert_eq!(ThyPath::parse(&sp(ok), false), Some(ThyPath::Other), "trace {ok}");
        }
        for bad in ["sources", "cases", "diffrules", "diffProof/debug", "x", "foo/bar"] {
            assert_eq!(ThyPath::parse(&sp(bad), false), None, "trace {bad} should 404");
        }
        // Equiv grammar: diff heads parse, trace heads do not.
        assert_eq!(
            ThyPath::parse(&sp("diffProof/L/RHS"), true),
            Some(ThyPath::DiffProof { lemma: "L".into(), path: vec!["RHS".into()] })
        );
        for ok in ["help", "diffrules", "diffMethod/L/1"] {
            assert_eq!(ThyPath::parse(&sp(ok), true), Some(ThyPath::Other), "equiv {ok}");
        }
        for bad in ["rules", "message", "tactic", "lemma/L", "proof/L", "cases/raw/0/0", "add/x"] {
            assert_eq!(ThyPath::parse(&sp(bad), true), None, "equiv {bad} should 404");
        }
    }

    #[test]
    fn parses_nav_and_overview_views() {
        let r = Route::parse("/thy/trace/1/next/normal/proof/Client_auth").unwrap();
        if let Handler::Next(t) = r.handler {
            assert_eq!(
                Nav::parse(NavDir::Next, &t).unwrap(),
                Nav { dir: NavDir::Next, mode: "normal".into(), lemma: "Client_auth".into() }
            );
        } else {
            panic!();
        }
        assert_eq!(OverviewView::parse(&owned(&["help"])), OverviewView::Help);
        assert_eq!(
            OverviewView::parse(&owned(&["edit", "L"])),
            OverviewView::Edit("L".into())
        );
        assert_eq!(
            OverviewView::parse(&owned(&["proof", "L", "_", "B_2"])),
            OverviewView::Proof { lemma: "L".into(), path: vec!["_".into(), "B_2".into()] }
        );
    }
}
