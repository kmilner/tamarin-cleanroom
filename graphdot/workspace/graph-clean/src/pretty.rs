//! A faithful Rust port of the Hughes/Peyton-Jones pretty-printing library
//! `Text.PrettyPrint.HughesPJ` (Haskell `pretty` 1.1.3.6, BSD-licensed), the
//! layout engine the reference toolchain (GHC boot library) uses to render
//! record-cell term text.
//!
//! This module implements the `Doc` document model and the combinators, the
//! best/fits layout selection, and the `PageMode` renderer, following the
//! sanctioned BSD source (`sanctioned/pretty-1.1.3.6/src/Text/PrettyPrint/
//! Annotated/HughesPJ.hs`) semantics precisely: `<>`/`<+>`, `$$`/`$+$`,
//! `hcat`/`hsep`/`vcat`, `sep`/`cat`, `fsep`/`fcat`, `nest`/`hang`, and the
//! `best`/`nicest`/`fits` fitting logic with `lineLength` and `ribbonsPerLine`.
//!
//! Nothing in this file is tamarin-specific: it is the general layout algebra.
//! WHICH combinators tamarin applies to a record cell, and with what line width
//! / ribbon / indent, is derived from black-box probes and lives in
//! [`crate::doclayout`]; this module is the exact engine those parameters feed.
//!
//! The annotation machinery of the original is elided (this port renders to a
//! plain `String`), so `TextBeside` carries the text and its column width
//! directly instead of an `AnnotDetails`.
//!
//! **Evaluation strategy.** The Haskell original relies on lazy evaluation: the
//! combinators (`sep`/`fill`/`beside`/`aboveNest`) describe exponentially large
//! sets of layouts as union trees, but `best` only ever forces the branches it
//! inspects, and `fits` only forces the *first line* of a candidate. A naive
//! strict port materializes those whole trees (exponential in the number of
//! fill elements). This port mirrors the laziness explicitly: recursive
//! combinator continuations are built as [`Doc::Lazy`] thunks (forced on
//! demand, memoized), and the `nicest` union choice uses [`fits_ahead`], which
//! decides `fits (min w r - sl) (best' ...)` by walking the *unresolved*
//! branch's first line only. Both are pure evaluation-order mirrors of the
//! sanctioned semantics: the resolved layout is byte-identical.

use std::cell::RefCell;
use std::rc::Rc;

/// The abstract document type. A `Doc` denotes a *set* of layouts; a `Doc` with
/// no `Union`/`NoDoc` denotes a single layout. Mirrors the Haskell `Doc a`
/// constructors (annotations elided).
#[derive(Clone, Debug)]
pub enum Doc {
    /// An empty span (`empty`): no height, no width.
    Empty,
    /// `text "" $$ x` — a newline above `x`. Its argument is never `Empty`.
    NilAbove(Rc<Doc>),
    /// `text s <> x` — literal text (with its column width) beside `x`. The
    /// argument is never a `Nest`.
    TextBeside(Rc<str>, usize, Rc<Doc>),
    /// `nest k x` — indent `x` by `k` columns (`k` may be negative).
    Nest(isize, Rc<Doc>),
    /// `ul \`union\` ur` — a choice between two layouts that flatten equal.
    Union(Rc<Doc>, Rc<Doc>),
    /// The empty *set* of documents.
    NoDoc,
    /// `Beside l sep r` — `sep` = true means a space is inserted between.
    Beside(Rc<Doc>, bool, Rc<Doc>),
    /// `Above u never_overlap l` — `never_overlap` = true forbids overlap.
    Above(Rc<Doc>, bool, Rc<Doc>),
    /// A suspended sub-document (the strict mirror of a Haskell thunk):
    /// computed on first [`force`], then memoized. Only ever wraps an already
    /// *reduced* document (an RDoc — no `Beside`/`Above`).
    Lazy(Rc<LazyDoc>),
}

/// The shared state of a [`Doc::Lazy`] thunk.
pub struct LazyDoc(RefCell<LazyState>);

enum LazyState {
    Pending(Option<Box<dyn FnOnce() -> Doc>>),
    Forced(Doc),
}

impl std::fmt::Debug for LazyDoc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self.0.borrow() {
            LazyState::Pending(_) => write!(f, "Lazy(<pending>)"),
            LazyState::Forced(d) => write!(f, "Lazy({:?})", d),
        }
    }
}

use Doc::*;

fn rc(d: Doc) -> Rc<Doc> {
    Rc::new(d)
}

/// Suspend a document computation (a thunk). The closure runs at most once.
fn lazy<F: FnOnce() -> Doc + 'static>(f: F) -> Doc {
    Lazy(Rc::new(LazyDoc(RefCell::new(LazyState::Pending(Some(
        Box::new(f),
    ))))))
}

/// Force a document to its outermost non-`Lazy` constructor (shallow clone —
/// children stay shared via `Rc`). Thunks are computed once and memoized;
/// chains of thunks collapse to the final value.
fn force(d: &Doc) -> Doc {
    match d {
        Lazy(l) => {
            if let LazyState::Forced(v) = &*l.0.borrow() {
                return v.clone();
            }
            let f = match &mut *l.0.borrow_mut() {
                LazyState::Pending(f) => f.take().expect("re-entrant force"),
                LazyState::Forced(v) => return v.clone(),
            };
            let v = force(&f());
            *l.0.borrow_mut() = LazyState::Forced(v.clone());
            v
        }
        _ => d.clone(),
    }
}

// ---------------------------------------------------------------------------
// Constructors (§ "Values and Predicates")

/// A one-character document.
pub fn char(c: char) -> Doc {
    let mut buf = [0u8; 4];
    let s: &str = c.encode_utf8(&mut buf);
    TextBeside(Rc::from(s), 1, rc(Empty))
}

/// A one-line literal string document; its width is its Unicode-scalar count.
pub fn text(s: &str) -> Doc {
    let len = s.chars().count();
    TextBeside(Rc::from(s), len, rc(Empty))
}

/// Text with an explicit column width (`sizedText l s`); use width 0 for
/// non-printing text.
pub fn sized_text(len: usize, s: &str) -> Doc {
    TextBeside(Rc::from(s), len, rc(Empty))
}

/// The empty document (identity for `<>`, `$$`, and list combinators).
pub fn empty() -> Doc {
    Empty
}

/// Is this the empty document?
pub fn is_empty(d: &Doc) -> bool {
    matches!(force(d), Empty)
}


// ---------------------------------------------------------------------------
// Structural smart-constructors

fn nil_above_(d: Rc<Doc>) -> Doc {
    NilAbove(d)
}

fn text_beside_(s: Rc<str>, l: usize, d: Rc<Doc>) -> Doc {
    TextBeside(s, l, d)
}

fn nest_(k: isize, d: Rc<Doc>) -> Doc {
    Nest(k, d)
}

fn union_(a: Rc<Doc>, b: Rc<Doc>) -> Doc {
    Union(a, b)
}

/// `reduceDoc`: push `Beside`/`Above` down into reduced form (RDoc).
pub fn reduce_doc(d: &Doc) -> Doc {
    match d {
        Beside(p, g, q) => beside(p, *g, &reduce_doc(q)),
        Above(p, g, q) => above(p, *g, &reduce_doc(q)),
        other => other.clone(),
    }
}

// ---------------------------------------------------------------------------
// nest / mkNest / mkUnion

/// `nest k p` — indent by `k` (may be negative).
pub fn nest(k: isize, p: &Doc) -> Doc {
    mk_nest(k, &reduce_doc(p))
}

fn mk_nest(k: isize, d: &Doc) -> Doc {
    let df = force(d);
    match &df {
        Nest(k1, p) => mk_nest(k + k1, p),
        NoDoc => NoDoc,
        Empty => Empty,
        _ if k == 0 => df,
        _ => nest_(k, rc(df)),
    }
}

fn mk_union(p: Doc, q: Doc) -> Doc {
    match force(&p) {
        Empty => Empty,
        pf => union_(rc(pf), rc(q)),
    }
}

// ---------------------------------------------------------------------------
// Horizontal composition <> and <+>

/// `p <> q` — beside.
pub fn beside_op(p: Doc, q: Doc) -> Doc {
    beside_(p, false, q)
}

/// `p <+> q` — beside, separated by a space unless an argument is empty.
pub fn beside_space(p: Doc, q: Doc) -> Doc {
    beside_(p, true, q)
}

fn beside_(p: Doc, g: bool, q: Doc) -> Doc {
    match (&p, &q) {
        (_, Empty) => p,
        (Empty, _) => q,
        _ => Beside(rc(p), g, rc(q)),
    }
}

/// `beside p g q` (spec: `p <g> q`) over reduced docs. Union branches and the
/// text-tail continuation are suspended ([`lazy`]), mirroring the sanctioned
/// source's (non-`$!`) laziness.
fn beside(p: &Doc, g: bool, q: &Doc) -> Doc {
    let pf = force(p);
    match &pf {
        NoDoc => NoDoc,
        Union(p1, p2) => {
            let (p1, p2, qa, qb) = (p1.clone(), p2.clone(), q.clone(), q.clone());
            union_(
                rc(lazy(move || beside(&p1, g, &qa))),
                rc(lazy(move || beside(&p2, g, &qb))),
            )
        }
        Empty => q.clone(),
        Nest(k, p1) => nest_(*k, rc(beside(p1, g, q))),
        Beside(p1, g1, q1) => {
            if *g1 == g {
                beside(p1, *g1, &beside(q1, g, q))
            } else {
                beside(&reduce_doc(&pf), g, q)
            }
        }
        Above(..) => beside(&reduce_doc(&pf), g, q),
        NilAbove(p1) => nil_above_(rc(beside(p1, g, q))),
        TextBeside(s, l, p1) => {
            let (p1, q1) = (p1.clone(), q.clone());
            let rest = lazy(move || match force(&p1) {
                Empty => nil_beside(g, &q1),
                _ => beside(&p1, g, &q1),
            });
            text_beside_(s.clone(), *l, rc(rest))
        }
        Lazy(_) => unreachable!("beside: forced doc"),
    }
}

/// `nilBeside g p` (spec: `text "" <g> p`).
fn nil_beside(g: bool, p: &Doc) -> Doc {
    let pf = force(p);
    match &pf {
        Empty => Empty,
        Nest(_, p1) => nil_beside(g, p1),
        _ => {
            if g {
                TextBeside(Rc::from(" "), 1, rc(pf))
            } else {
                pf
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Vertical composition $$ and $+$

/// `p $$ q` — above, with overlap allowed.
pub fn above_op(p: Doc, q: Doc) -> Doc {
    above_(p, false, q)
}

/// `p $+$ q` — above, no overlap.
pub fn above_plus(p: Doc, q: Doc) -> Doc {
    above_(p, true, q)
}

fn above_(p: Doc, g: bool, q: Doc) -> Doc {
    match (&p, &q) {
        (_, Empty) => p,
        (Empty, _) => q,
        _ => Above(rc(p), g, rc(q)),
    }
}

fn above(p: &Doc, g: bool, q: &Doc) -> Doc {
    let pf = force(p);
    match &pf {
        Above(p1, g1, q1) => above(p1, *g1, &above(q1, g, q)),
        Beside(..) => above_nest(&reduce_doc(&pf), g, 0, &reduce_doc(q)),
        _ => above_nest(&pf, g, 0, &reduce_doc(q)),
    }
}

/// `aboveNest p g k q` (spec: `p $g$ (nest k q)`). Union branches and the
/// recursive continuations are suspended, mirroring the sanctioned laziness.
fn above_nest(p: &Doc, g: bool, k: isize, q: &Doc) -> Doc {
    let pf = force(p);
    match &pf {
        NoDoc => NoDoc,
        Union(p1, p2) => {
            let (p1, p2, qa, qb) = (p1.clone(), p2.clone(), q.clone(), q.clone());
            union_(
                rc(lazy(move || above_nest(&p1, g, k, &qa))),
                rc(lazy(move || above_nest(&p2, g, k, &qb))),
            )
        }
        Empty => mk_nest(k, q),
        Nest(k1, p1) => {
            let (k1, p1, q1) = (*k1, p1.clone(), q.clone());
            nest_(k1, rc(lazy(move || above_nest(&p1, g, k - k1, &q1))))
        }
        NilAbove(p1) => {
            let (p1, q1) = (p1.clone(), q.clone());
            nil_above_(rc(lazy(move || above_nest(&p1, g, k, &q1))))
        }
        TextBeside(s, l, p1) => {
            let k1 = k - (*l as isize);
            let (p1, q1) = (p1.clone(), q.clone());
            let rest = lazy(move || match force(&p1) {
                Empty => nil_above_nest(g, k1, &q1),
                _ => above_nest(&p1, g, k1, &q1),
            });
            text_beside_(s.clone(), *l, rc(rest))
        }
        Above(..) => panic!("aboveNest Above"),
        Beside(..) => panic!("aboveNest Beside"),
        Lazy(_) => unreachable!("aboveNest: forced doc"),
    }
}

/// `nilAboveNest g k q` (spec: `text s <> (text "" $g$ nest k q)`).
fn nil_above_nest(g: bool, k: isize, q: &Doc) -> Doc {
    let qf = force(q);
    match &qf {
        Empty => Empty,
        Nest(k1, q1) => nil_above_nest(g, k + k1, q1),
        _ => {
            if !g && k > 0 {
                let ind = indent(k);
                let len = ind.chars().count();
                text_beside_(Rc::from(ind.as_str()), len, rc(qf))
            } else {
                nil_above_(rc(mk_nest(k, &qf)))
            }
        }
    }
}

// ---------------------------------------------------------------------------
// List versions: hcat / hsep / vcat  (via reduceHoriz/reduceVert)

#[derive(Clone, Copy)]
enum IsEmpty {
    // `Empty` mirrors the Haskell `reduceHoriz`/`reduceVert` return type; this
    // strict port only ever yields `NotEmpty`, but the variant is kept to match
    // the sanctioned source's shape.
    #[allow(dead_code)]
    Empty,
    NotEmpty,
}

fn reduce_horiz(d: &Doc) -> (IsEmpty, Doc) {
    match d {
        Beside(p, g, q) => {
            let (_, pr) = reduce_horiz(p);
            eliminate_empty(true, pr, *g, reduce_horiz(q))
        }
        _ => (IsEmpty::NotEmpty, d.clone()),
    }
}

fn reduce_vert(d: &Doc) -> (IsEmpty, Doc) {
    match d {
        Above(p, g, q) => {
            let (_, pr) = reduce_vert(p);
            eliminate_empty(false, pr, *g, reduce_vert(q))
        }
        _ => (IsEmpty::NotEmpty, d.clone()),
    }
}

/// `beside_cons` = true builds `Beside`, false builds `Above`.
fn eliminate_empty(beside_cons: bool, p: Doc, g: bool, q: (IsEmpty, Doc)) -> (IsEmpty, Doc) {
    match p {
        Empty => q,
        _ => {
            let out = match q {
                (IsEmpty::NotEmpty, q1) => {
                    if beside_cons {
                        Beside(rc(p), g, rc(q1))
                    } else {
                        Above(rc(p), g, rc(q1))
                    }
                }
                (IsEmpty::Empty, _) => p,
            };
            (IsEmpty::NotEmpty, out)
        }
    }
}

/// `hcat` — list version of `<>`.
pub fn hcat(ds: Vec<Doc>) -> Doc {
    let folded = ds
        .into_iter()
        .rev()
        .fold(Empty, |q, p| Beside(rc(p), false, rc(q)));
    reduce_horiz(&folded).1
}

/// `hsep` — list version of `<+>`.
pub fn hsep(ds: Vec<Doc>) -> Doc {
    let folded = ds
        .into_iter()
        .rev()
        .fold(Empty, |q, p| Beside(rc(p), true, rc(q)));
    reduce_horiz(&folded).1
}

/// `vcat` — list version of `$$`.
pub fn vcat(ds: Vec<Doc>) -> Doc {
    let folded = ds
        .into_iter()
        .rev()
        .fold(Empty, |q, p| Above(rc(p), false, rc(q)));
    reduce_vert(&folded).1
}

/// `hang d1 n d2 = sep [d1, nest n d2]`.
pub fn hang(d1: Doc, n: isize, d2: Doc) -> Doc {
    sep(vec![d1, nest(n, &d2)])
}

/// `punctuate p [d1,…,dn] = [d1<>p, …, d(n-1)<>p, dn]`.
pub fn punctuate(p: &Doc, ds: Vec<Doc>) -> Vec<Doc> {
    let n = ds.len();
    ds.into_iter()
        .enumerate()
        .map(|(i, d)| {
            if i + 1 < n {
                beside_op(d, p.clone())
            } else {
                d
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// sep / cat

/// `sep` — either `hsep` (one line) or `vcat`.
pub fn sep(ds: Vec<Doc>) -> Doc {
    sep_x(true, ds)
}

/// `cat` — either `hcat` (one line) or `vcat`.
pub fn cat(ds: Vec<Doc>) -> Doc {
    sep_x(false, ds)
}

fn sep_x(x: bool, ds: Vec<Doc>) -> Doc {
    if ds.is_empty() {
        return Empty;
    }
    let mut it = ds.into_iter();
    let p = it.next().unwrap();
    let ys: Vec<Doc> = it.collect();
    sep1(x, &reduce_doc(&p), 0, &ys)
}

fn sep1(g: bool, p: &Doc, k: isize, ys: &[Doc]) -> Doc {
    let pf = force(p);
    match &pf {
        NoDoc => NoDoc,
        Union(p1, q) => {
            let (p1, q) = (p1.clone(), q.clone());
            let (ya, yb) = (ys.to_vec(), ys.to_vec());
            union_(
                rc(lazy(move || sep1(g, &p1, k, &ya))),
                rc(lazy(move || {
                    above_nest(&q, false, k, &reduce_doc(&vcat(yb)))
                })),
            )
        }
        Empty => mk_nest(k, &sep_x(g, ys.to_vec())),
        Nest(n, p1) => {
            let (n, p1, ys) = (*n, p1.clone(), ys.to_vec());
            nest_(n, rc(lazy(move || sep1(g, &p1, k - n, &ys))))
        }
        NilAbove(p1) => {
            let (p1, ys) = (p1.clone(), ys.to_vec());
            nil_above_(rc(lazy(move || {
                above_nest(&p1, false, k, &reduce_doc(&vcat(ys)))
            })))
        }
        TextBeside(s, l, p1) => {
            let li = *l as isize;
            let (p1, ys) = (p1.clone(), ys.to_vec());
            text_beside_(
                s.clone(),
                *l,
                rc(lazy(move || sep_nb(g, &p1, k - li, &ys))),
            )
        }
        Above(..) => panic!("sep1 Above"),
        Beside(..) => panic!("sep1 Beside"),
        Lazy(_) => unreachable!("sep1: forced doc"),
    }
}

fn sep_nb(g: bool, p: &Doc, k: isize, ys: &[Doc]) -> Doc {
    let pf = force(p);
    match &pf {
        Nest(_, p1) => sep_nb(g, p1, k, ys),
        Empty => {
            let rest = if g {
                hsep(ys.to_vec())
            } else {
                hcat(ys.to_vec())
            };
            let left = one_liner(&nil_beside(g, &reduce_doc(&rest)));
            let (k1, ys1) = (k, ys.to_vec());
            let right = lazy(move || nil_above_nest(false, k1, &reduce_doc(&vcat(ys1))));
            mk_union(left, right)
        }
        _ => sep1(g, &pf, k, ys),
    }
}

// ---------------------------------------------------------------------------
// fill (fcat / fsep)

/// `fcat` — paragraph-fill version of `cat` (no inter-element space).
pub fn fcat(ds: Vec<Doc>) -> Doc {
    fill(false, ds)
}

/// `fsep` — paragraph-fill version of `sep` (space between elements).
pub fn fsep(ds: Vec<Doc>) -> Doc {
    fill(true, ds)
}

fn fill(g: bool, ds: Vec<Doc>) -> Doc {
    if ds.is_empty() {
        return Empty;
    }
    let mut it = ds.into_iter();
    let p = it.next().unwrap();
    let ys: Vec<Doc> = it.collect();
    fill1(g, &reduce_doc(&p), 0, &ys)
}

fn fill1(g: bool, p: &Doc, k: isize, ys: &[Doc]) -> Doc {
    let pf = force(p);
    match &pf {
        NoDoc => NoDoc,
        Union(p1, q) => {
            let (p1, q) = (p1.clone(), q.clone());
            let (ya, yb) = (ys.to_vec(), ys.to_vec());
            union_(
                rc(lazy(move || fill1(g, &p1, k, &ya))),
                rc(lazy(move || above_nest(&q, false, k, &fill(g, yb)))),
            )
        }
        Empty => mk_nest(k, &fill(g, ys.to_vec())),
        Nest(n, p1) => {
            let (n, p1, ys) = (*n, p1.clone(), ys.to_vec());
            nest_(n, rc(lazy(move || fill1(g, &p1, k - n, &ys))))
        }
        NilAbove(p1) => {
            let (p1, ys) = (p1.clone(), ys.to_vec());
            nil_above_(rc(lazy(move || {
                above_nest(&p1, false, k, &fill(g, ys))
            })))
        }
        TextBeside(s, l, p1) => {
            let li = *l as isize;
            let (p1, ys) = (p1.clone(), ys.to_vec());
            text_beside_(
                s.clone(),
                *l,
                rc(lazy(move || fill_nb(g, &p1, k - li, &ys))),
            )
        }
        Above(..) => panic!("fill1 Above"),
        Beside(..) => panic!("fill1 Beside"),
        Lazy(_) => unreachable!("fill1: forced doc"),
    }
}

fn fill_nb(g: bool, p: &Doc, k: isize, ys: &[Doc]) -> Doc {
    let pf = force(p);
    match &pf {
        Nest(_, p1) => fill_nb(g, p1, k, ys),
        Empty => {
            if ys.is_empty() {
                Empty
            } else if matches!(force(&ys[0]), Empty) {
                fill_nb(g, &Empty, k, &ys[1..])
            } else {
                fill_nbe(g, k, &ys[0], &ys[1..])
            }
        }
        _ => fill1(g, &pf, k, ys),
    }
}

fn fill_nbe(g: bool, k: isize, y: &Doc, ys: &[Doc]) -> Doc {
    let k1 = if g { k - 1 } else { k };
    let y_one = elide_nest(&one_liner(&reduce_doc(y)));
    let left = nil_beside(g, &fill1(g, &y_one, k1, ys));
    let (y2, ys2) = (y.clone(), ys.to_vec());
    let right = lazy(move || {
        let mut rest = vec![y2];
        rest.extend(ys2);
        nil_above_nest(false, k, &fill(g, rest))
    });
    mk_union(left, right)
}

fn elide_nest(d: &Doc) -> Doc {
    let df = force(d);
    match &df {
        Nest(_, d1) => d1.as_ref().clone(),
        _ => df,
    }
}

// ---------------------------------------------------------------------------
// Best layout: best / nicest / fits / oneLiner

fn one_liner(d: &Doc) -> Doc {
    let df = force(d);
    match &df {
        NoDoc => NoDoc,
        Empty => Empty,
        NilAbove(_) => NoDoc,
        TextBeside(s, l, p) => {
            let p = p.clone();
            text_beside_(s.clone(), *l, rc(lazy(move || one_liner(&p))))
        }
        Nest(k, p) => {
            let (k, p) = (*k, p.clone());
            nest_(k, rc(lazy(move || one_liner(&p))))
        }
        Union(p, _) => one_liner(p),
        Above(..) => panic!("oneLiner Above"),
        Beside(..) => panic!("oneLiner Beside"),
        Lazy(_) => unreachable!("oneLiner: forced doc"),
    }
}

/// `best w r doc` — resolve unions to a single layout for line width `w`,
/// ribbon width `r`. Mirrors the Haskell `best`/`nicest1` including its
/// laziness: the resolved document's tails are thunks, so `fits` (which only
/// inspects a candidate's *first line*) never forces a branch beyond that
/// line, and the right branch of a union is only resolved when the left's
/// first line does not fit.
fn best(w: isize, r: isize, d: &Doc) -> Doc {
    get(w, r, d)
}

fn get(w: isize, r: isize, d: &Doc) -> Doc {
    let df = force(d);
    match &df {
        Empty => Empty,
        NoDoc => NoDoc,
        NilAbove(p) => {
            let p = p.clone();
            nil_above_(rc(lazy(move || get(w, r, &p))))
        }
        TextBeside(s, l, p) => {
            let (li, p) = (*l as isize, p.clone());
            text_beside_(s.clone(), *l, rc(lazy(move || get1(w, r, li, &p))))
        }
        Nest(k, p) => {
            let (k, p) = (*k, p.clone());
            nest_(k, rc(lazy(move || get(w - k, r, &p))))
        }
        Union(p, q) => nicest(w, r, p, q),
        Above(..) => panic!("best get Above"),
        Beside(..) => panic!("best get Beside"),
        Lazy(_) => unreachable!("get: forced doc"),
    }
}

fn get1(w: isize, r: isize, sl: isize, d: &Doc) -> Doc {
    let df = force(d);
    match &df {
        Empty => Empty,
        NoDoc => NoDoc,
        NilAbove(p) => {
            let p = p.clone();
            nil_above_(rc(lazy(move || get(w - sl, r, &p))))
        }
        TextBeside(s, l, p) => {
            let (li, p) = (*l as isize, p.clone());
            text_beside_(
                s.clone(),
                *l,
                rc(lazy(move || get1(w, r, sl + li, &p))),
            )
        }
        Nest(_, p) => get1(w, r, sl, p),
        Union(p, q) => nicest1(w, r, sl, p, q),
        Above(..) => panic!("best get1 Above"),
        Beside(..) => panic!("best get1 Beside"),
        Lazy(_) => unreachable!("get1: forced doc"),
    }
}

fn nicest(w: isize, r: isize, p: &Doc, q: &Doc) -> Doc {
    nicest1(w, r, 0, p, q)
}

fn nicest1(w: isize, r: isize, sl: isize, p: &Doc, q: &Doc) -> Doc {
    // Resolve the left branch lazily; keep it iff its first line fits (`fits`
    // forces only that first line). Resolve the right branch only otherwise.
    let (pc, qc) = (p.clone(), q.clone());
    let lp = lazy(move || get1(w, r, sl, &pc));
    if fits(w.min(r) - sl, &lp) {
        lp
    } else {
        lazy(move || get1(w, r, sl, &qc))
    }
}

/// True iff the *first line* of `d` fits in `n` columns. Forces only up to the
/// first line break of an already-`best`-resolved (thunked) document.
fn fits(n: isize, d: &Doc) -> bool {
    if n < 0 {
        return false;
    }
    match &force(d) {
        NoDoc => false,
        Empty => true,
        NilAbove(_) => true,
        TextBeside(_, l, p) => fits(n - *l as isize, p),
        Above(..) => panic!("fits Above"),
        Beside(..) => panic!("fits Beside"),
        Union(..) => panic!("fits Union"),
        Nest(..) => panic!("fits Nest"),
        Lazy(_) => unreachable!("fits: forced doc"),
    }
}

// ---------------------------------------------------------------------------
// Rendering (PageMode only)

fn indent(n: isize) -> String {
    if n <= 0 {
        String::new()
    } else {
        " ".repeat(n as usize)
    }
}

/// Banker's-rounding of `line_len / ribbons` to match Haskell `round`.
fn round_ribbon(line_len: isize, ribbons: f64) -> isize {
    let v = line_len as f64 / ribbons;
    let floor = v.floor();
    let diff = v - floor;
    let rounded = if (diff - 0.5).abs() < 1e-9 {
        // round half to even
        let f = floor as i64;
        if f % 2 == 0 {
            f
        } else {
            f + 1
        }
    } else {
        v.round() as i64
    };
    rounded as isize
}

/// Render a document in `PageMode` at line length `line_len` and
/// `ribbons_per_line`, producing the plain string (newline-separated lines,
/// each continuation prefixed by its indentation spaces).
pub fn render_page(line_len: isize, ribbons_per_line: f64, doc: &Doc) -> String {
    let ribbon_len = round_ribbon(line_len, ribbons_per_line);
    let reduced = reduce_doc(doc);
    let best_doc = best(line_len, ribbon_len, &reduced);
    display_page(line_len, ribbon_len, &best_doc)
}

fn display_page(page_width: isize, ribbon_width: isize, doc: &Doc) -> String {
    let _gap = page_width - ribbon_width;
    let mut out = String::new();
    lay(&mut out, 0, doc);
    out
}

fn lay(out: &mut String, k: isize, d: &Doc) {
    match &force(d) {
        Nest(k1, p) => lay(out, k + k1, p),
        Empty => {}
        NilAbove(p) => {
            out.push('\n');
            lay(out, k, p);
        }
        TextBeside(s, l, p) => lay1(out, k, s, *l, p),
        Above(..) => panic!("display lay Above"),
        Beside(..) => panic!("display lay Beside"),
        NoDoc => panic!("display lay NoDoc"),
        Union(..) => panic!("display lay Union"),
        Lazy(_) => unreachable!("lay: forced doc"),
    }
}

fn lay1(out: &mut String, k: isize, s: &str, l: usize, p: &Doc) {
    out.push_str(&indent(k));
    out.push_str(s);
    lay2(out, k + l as isize, p);
}

fn lay2(out: &mut String, k: isize, d: &Doc) {
    match &force(d) {
        NilAbove(p) => {
            out.push('\n');
            lay(out, k, p);
        }
        TextBeside(s, l, p) => {
            out.push_str(s);
            lay2(out, k + *l as isize, p);
        }
        Nest(_, p) => lay2(out, k, p),
        Empty => {}
        Above(..) => panic!("display lay2 Above"),
        Beside(..) => panic!("display lay2 Beside"),
        NoDoc => panic!("display lay2 NoDoc"),
        Union(..) => panic!("display lay2 Union"),
        Lazy(_) => unreachable!("lay2: forced doc"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render87(d: &Doc) -> String {
        render_page(87, 1.0, d)
    }

    #[test]
    fn text_and_beside() {
        let d = beside_op(text("hello"), text("world"));
        assert_eq!(render87(&d), "helloworld");
        let d = beside_space(text("hello"), text("world"));
        assert_eq!(render87(&d), "hello world");
    }

    #[test]
    fn fsep_wraps_at_width() {
        // Three words, width forces a break.
        let d = fsep(vec![text("aaaa"), text("bbbb"), text("cccc")]);
        assert_eq!(render_page(100, 1.0, &d), "aaaa bbbb cccc");
        // Narrow: 6 columns -> each on its own line.
        assert_eq!(render_page(6, 1.0, &d), "aaaa\nbbbb\ncccc");
    }

    #[test]
    fn fsep_continuation_aligns_to_origin() {
        // "AB" then fsep of two words; when the second wraps it aligns to the
        // column where the fsep began (after "AB").
        let d = beside_op(text("AB"), fsep(vec![text("cd"), text("ef")]));
        assert_eq!(render_page(6, 1.0, &d), "ABcd\n  ef");
    }

    #[test]
    fn vcat_stacks() {
        let d = vcat(vec![text("one"), text("two")]);
        assert_eq!(render87(&d), "one\ntwo");
    }

    #[test]
    fn sep_all_or_nothing() {
        let d = sep(vec![text("aa"), text("bb")]);
        assert_eq!(render_page(10, 1.0, &d), "aa bb");
        assert_eq!(render_page(3, 1.0, &d), "aa\nbb");
    }

    // ---- Exploration: candidate tuple/fact constructions vs fixtures --------
    fn punct_tokens(sep_s: &str, elems: &[&str]) -> Vec<Doc> {
        let n = elems.len();
        elems
            .iter()
            .enumerate()
            .map(|(i, e)| {
                if i + 1 < n {
                    text(&format!("{}{}", e, sep_s))
                } else {
                    text(e)
                }
            })
            .collect()
    }

    // fact = sep [ text "Name( " <> args, text ")" ]
    fn fact(name: &str, args: Doc) -> Doc {
        sep(vec![
            beside_op(text(&format!("{}( ", name)), args),
            text(")"),
        ])
    }

    #[test]
    #[ignore]
    fn explore_tuple_constructions() {
        let elems12: Vec<&str> = vec![
            "'a01'", "'a02'", "'a03'", "'a04'", "'a05'", "'a06'", "'a07'", "'a08'",
            "'a09'", "'a10'", "'a11'", "'a12'",
        ];
        let w74a = format!("'{}'", "a".repeat(74));
        let elemsW: Vec<&str> = vec![&w74a, "'y'"];

        for (name, elems) in [("E12", &elems12), ("W74", &elemsW)] {
            println!("===== {name} =====");
            // Candidate B: < <> fcat(punct ", ") <> >
            let tb = beside_op(
                beside_op(char('<'), fcat(punct_tokens(", ", elems))),
                char('>'),
            );
            println!("[B <>fcat] fact:\n{}", render_page(87, 1.0, &fact(name, tb)));
            // Candidate D: cat[ < <> fcat, > ]
            let td = cat(vec![
                beside_op(char('<'), fcat(punct_tokens(", ", elems))),
                char('>'),
            ]);
            println!("[D cat] fact:\n{}", render_page(87, 1.0, &fact(name, td)));
            // Candidate F: fcat[ < <> fcat_inner, > ]
            let tf = fcat(vec![
                beside_op(char('<'), fcat(punct_tokens(", ", elems))),
                char('>'),
            ]);
            println!("[F fcat-outer] fact:\n{}", render_page(87, 1.0, &fact(name, tf)));
            // Candidate G: < <> fcat(inner ++ [>])
            let mut toks = punct_tokens(", ", elems);
            toks.push(char('>'));
            let tg = beside_op(char('<'), fcat(toks));
            println!("[G <>fcat+>] fact:\n{}", render_page(87, 1.0, &fact(name, tg)));
            // Candidate H: < <> fcat(inner ++ [nest -1 >])
            let mut toks2 = punct_tokens(", ", elems);
            toks2.push(nest(-1, &char('>')));
            let th = beside_op(char('<'), fcat(toks2));
            println!("[H <>fcat+nest-1>] fact:\n{}", render_page(87, 1.0, &fact(name, th)));
            // Candidate I: fsep instead of fcat with nest -1 close
            let mut toks3 = punct_tokens(",", elems); // comma only, fsep adds space
            toks3.push(nest(-1, &char('>')));
            let ti = beside_op(char('<'), fsep(toks3));
            println!("[I <>fsep+nest-1>] fact:\n{}", render_page(87, 1.0, &fact(name, ti)));
        }
    }
}
