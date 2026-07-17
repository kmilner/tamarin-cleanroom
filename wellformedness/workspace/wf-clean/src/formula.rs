//! Unicode pretty-printer for trace formulas, calibrated from the oracle
//! (e.g. ks1 lemma L2 -> "∀ x #i #j. (Act( x ) @ #i) ⇒ (x = x)").
//!
//! `pp_formula` renders on a single line (byte-exact when the oracle keeps the
//! formula on one line). `pp_formula_wrapped` reproduces the oracle's multi-line
//! layout for wide formulas via a small HughesPJ-style document engine
//! (calibrated against probes r3_gw / r3_qm): a binary connective breaks after
//! its operator with the right operand hanging at the connective's starting
//! column, and a quantifier breaks after the `.` with its body hanging two
//! columns past the formula's base indent. See BEHAVIOR.md.

use crate::ast::*;
use crate::pretty::{pp_fact, pp_term};

pub fn pp_formula(f: &Formula) -> String {
    match f {
        Formula::False => "⊥".to_string(),
        Formula::True => "⊤".to_string(),
        Formula::Atom(a) => pp_atom(a),
        Formula::Not(g) => format!("¬({})", pp_formula(g)),
        Formula::And(a, b) => format!("({}) ∧ ({})", pp_formula(a), pp_formula(b)),
        Formula::Or(a, b) => format!("({}) ∨ ({})", pp_formula(a), pp_formula(b)),
        Formula::Implies(a, b) => format!("({}) ⇒ ({})", pp_formula(a), pp_formula(b)),
        Formula::Iff(a, b) => format!("({}) ⇔ ({})", pp_formula(a), pp_formula(b)),
        Formula::Forall(vs, g) => format!("∀ {}. {}", pp_bound_vars(vs), pp_formula(g)),
        Formula::Exists(vs, g) => format!("∃ {}. {}", pp_bound_vars(vs), pp_formula(g)),
    }
}

fn pp_bound_vars(vs: &[VarSpec]) -> String {
    vs.iter()
        .map(crate::pretty::pp_var)
        .collect::<Vec<_>>()
        .join(" ")
}

fn pp_atom(a: &Atom) -> String {
    match a {
        Atom::Eq(x, y) => format!("{} = {}", pp_term(x), pp_term(y)),
        Atom::Less(x, y) => format!("{} < {}", pp_term(x), pp_term(y)),
        Atom::LessMset(x, y) => format!("{} ⋖ {}", pp_term(x), pp_term(y)),
        Atom::Subterm(x, y) => format!("{} ⊏ {}", pp_term(x), pp_term(y)),
        Atom::Action(f, t) => format!("{} @ {}", pp_fact(f), pp_term(t)),
        Atom::Last(t) => format!("last({})", pp_term(t)),
        Atom::Pred(f) => pp_fact(f),
    }
}

// ---------------------------------------------------------------------------
// Multi-line layout (HughesPJ-style) for wide formulas
// ---------------------------------------------------------------------------

/// Effective page width for the formula printer (measured: a wide formula's
/// single line fits at total column 72 and breaks by 74 - probes r3_qm).
pub const FORMULA_WIDTH: usize = 72;

/// A layout document. `Beside` glues children horizontally (each child's break
/// indent = the column where it starts, i.e. column-aligned continuation).
/// `Group` is an all-or-nothing break point: it is laid flat when it fits,
/// otherwise the first child stays on the current line and each following child
/// starts a new line indented `base + hang`.
enum Doc {
    Text(String),
    Beside(Vec<Doc>),
    Group(Vec<Doc>, usize),
}

fn flat(d: &Doc) -> String {
    match d {
        Doc::Text(s) => s.clone(),
        Doc::Beside(ds) => ds.iter().map(flat).collect(),
        Doc::Group(ds, _) => ds.iter().map(flat).collect::<Vec<_>>().join(" "),
    }
}

/// Lay `d` beginning at column `col`; `base` is the reference indent for hanging
/// breaks. Returns the rendered text (with baked-in leading spaces on
/// continuation lines) and the end column of the last line.
fn lay(d: &Doc, col: usize, base: usize, width: usize) -> (String, usize) {
    match d {
        Doc::Text(s) => (s.clone(), col + s.chars().count()),
        Doc::Beside(ds) => {
            let mut out = String::new();
            let mut c = col;
            for x in ds {
                // Column-aligned: a child's own hanging breaks start at `c`.
                let (s, e) = lay(x, c, c, width);
                out.push_str(&s);
                c = e;
            }
            (out, c)
        }
        Doc::Group(ds, hang) => {
            let fw = flat(d).chars().count();
            if col + fw <= width {
                // Flat: children joined by single spaces, no internal breaks.
                let mut out = String::new();
                let mut c = col;
                for (i, x) in ds.iter().enumerate() {
                    if i > 0 {
                        out.push(' ');
                        c += 1;
                    }
                    let fs = flat(x);
                    c += fs.chars().count();
                    out.push_str(&fs);
                }
                (out, c)
            } else {
                let indent = base + hang;
                let (s0, _) = lay(&ds[0], col, base, width);
                let mut out = s0;
                for x in &ds[1..] {
                    out.push('\n');
                    out.push_str(&" ".repeat(indent));
                    let (s, _) = lay(x, indent, indent, width);
                    out.push_str(&s);
                }
                let end = out.rsplit('\n').next().unwrap().chars().count();
                (out, end)
            }
        }
    }
}

/// `(doc)` - the operand of a connective is always parenthesised.
fn parens(inner: Doc) -> Doc {
    Doc::Beside(vec![Doc::Text("(".into()), inner, Doc::Text(")".into())])
}

fn binop_doc(a: &Formula, op: &str, b: &Formula) -> Doc {
    // sep [ (a) <op>, (b) ] : breaks after the operator, right operand hanging
    // at the connective's start column (hang 0).
    Doc::Group(
        vec![
            Doc::Beside(vec![parens(formula_doc(a)), Doc::Text(format!(" {}", op))]),
            parens(formula_doc(b)),
        ],
        0,
    )
}

fn formula_doc(f: &Formula) -> Doc {
    match f {
        Formula::False => Doc::Text("⊥".into()),
        Formula::True => Doc::Text("⊤".into()),
        Formula::Atom(a) => Doc::Text(pp_atom(a)),
        Formula::Not(g) => Doc::Beside(vec![Doc::Text("¬".into()), parens(formula_doc(g))]),
        Formula::And(a, b) => binop_doc(a, "∧", b),
        Formula::Or(a, b) => binop_doc(a, "∨", b),
        Formula::Implies(a, b) => binop_doc(a, "⇒", b),
        Formula::Iff(a, b) => binop_doc(a, "⇔", b),
        Formula::Forall(vs, g) => quantifier_doc("∀", vs, g),
        Formula::Exists(vs, g) => quantifier_doc("∃", vs, g),
    }
}

fn quantifier_doc(sym: &str, vs: &[VarSpec], g: &Formula) -> Doc {
    // sep [ "Q v1 .. vn.", body ] : breaks after the dot, body hanging two
    // columns past the formula base indent (hang 2).
    let head = format!("{} {}.", sym, pp_bound_vars(vs));
    Doc::Group(vec![Doc::Text(head), formula_doc(g)], 2)
}

/// Render a formula the way the oracle embeds it inside `"..."` in the
/// guardedness report: starting at column `col`, with `base` the indentation of
/// the enclosing quote. Continuation lines carry their absolute indentation.
pub fn pp_formula_wrapped(f: &Formula, col: usize, base: usize) -> String {
    lay(&formula_doc(f), col, base, FORMULA_WIDTH).0
}
