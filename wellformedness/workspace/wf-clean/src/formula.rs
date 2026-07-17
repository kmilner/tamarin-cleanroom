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
// Multi-line layout for wide formulas
// ---------------------------------------------------------------------------

/// Effective page width for the formula printer (measured: a wide formula's
/// single line fits at total column 72 and breaks by 74 - probes r3_qm).
pub const FORMULA_WIDTH: usize = 72;

/// A layout document. `Beside` glues children horizontally. `Group` is an
/// all-or-nothing break point: it is laid flat when it fits on the current line,
/// otherwise the first child stays on the current line and each following child
/// starts a new line indented to the group's own start column plus `hang`.
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

/// Lay `d` beginning at column `col`. Continuation lines break relative to the
/// column where the enclosing group started (`col + hang`). Returns the
/// rendered text (with baked-in leading spaces on continuation lines) and the
/// end column of the last line.
fn lay(d: &Doc, col: usize, width: usize) -> (String, usize) {
    match d {
        Doc::Text(s) => (s.clone(), col + s.chars().count()),
        Doc::Beside(ds) => {
            let mut out = String::new();
            let mut c = col;
            for x in ds {
                let (s, e) = lay(x, c, width);
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
                // Broken: the first child stays on the current line; every
                // following child hangs at `col + hang` (the group's start
                // column plus its hang offset).
                let indent = col + hang;
                let (s0, _) = lay(&ds[0], col, width);
                let mut out = s0;
                for x in &ds[1..] {
                    out.push('\n');
                    out.push_str(&" ".repeat(indent));
                    let (s, _) = lay(x, indent, width);
                    out.push_str(&s);
                }
                let end = out.rsplit('\n').next().unwrap().chars().count();
                (out, end)
            }
        }
    }
}

/// `(doc)` - the operand of a logical connective is always parenthesised.
fn parens(inner: Doc) -> Doc {
    Doc::Beside(vec![Doc::Text("(".into()), inner, Doc::Text(")".into())])
}

fn binop_doc(a: &Formula, op: &str, b: &Formula) -> Doc {
    // Breaks after the operator; the right operand hangs at the connective's
    // start column (hang 0). Both operands are parenthesised.
    Doc::Group(
        vec![
            Doc::Beside(vec![parens(formula_doc(a)), Doc::Text(format!(" {}", op))]),
            parens(formula_doc(b)),
        ],
        0,
    )
}

/// A relational atom `a OP b` (=, <, ⋖, ⊏): breaks after the operator with the
/// right term hanging at the atom's start column (hang 0). The term operands are
/// NOT parenthesised (only the whole atom is, by its enclosing connective).
fn relation_doc(a: &Term, op: &str, b: &Term) -> Doc {
    Doc::Group(
        vec![
            Doc::Text(format!("{} {}", pp_term(a), op)),
            Doc::Text(pp_term(b)),
        ],
        0,
    )
}

fn atom_doc(a: &Atom) -> Doc {
    match a {
        Atom::Eq(x, y) => relation_doc(x, "=", y),
        Atom::Less(x, y) => relation_doc(x, "<", y),
        Atom::LessMset(x, y) => relation_doc(x, "⋖", y),
        Atom::Subterm(x, y) => relation_doc(x, "⊏", y),
        // Action, Last and Pred atoms are laid on a single line.
        Atom::Action(_, _) | Atom::Last(_) | Atom::Pred(_) => Doc::Text(pp_atom(a)),
    }
}

fn formula_doc(f: &Formula) -> Doc {
    match f {
        Formula::False => Doc::Text("⊥".into()),
        Formula::True => Doc::Text("⊤".into()),
        Formula::Atom(a) => atom_doc(a),
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
    // Breaks after the dot; the body hangs one column past the quantifier's
    // start column (hang 1).
    let head = format!("{} {}.", sym, pp_bound_vars(vs));
    Doc::Group(vec![Doc::Text(head), formula_doc(g)], 1)
}

/// Render a formula the way the oracle embeds it inside `"..."` in the
/// guardedness report: the formula's first character sits at column `col`
/// (the oracle uses column 7, i.e. after `      "`). Continuation lines carry
/// their absolute indentation. When the whole formula fits on one line the
/// output is byte-identical to [`pp_formula`].
pub fn pp_formula_wrapped(f: &Formula, col: usize) -> String {
    lay(&formula_doc(f), col, FORMULA_WIDTH).0
}
