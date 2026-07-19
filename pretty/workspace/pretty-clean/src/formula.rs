//! R3 — trace-formula rendering (shared by restrictions and lemma statements).
//!
//! Every glyph and layout decision is oracle-pinned (see workspace/BEHAVIOR.md
//! "Formula rendering" for probe provenance). Shape summary:
//!
//! * glyphs: `∀ ∃ ⇒ ∧ ∨ ¬ ⇔ ⊤ ⊥`, atoms `@ < = ⊏ last(..)`;
//! * EVERY operand of a binary connective and every `¬` argument is
//!   parenthesized; quantifier bodies and the top level are bare; chains
//!   keep their source association (probe:q_p2);
//! * binary connective = `sep [lhs <+> glyph, rhs]` — the glyph attaches to
//!   the lhs' last line, the rhs drops to the group origin;
//! * quantifier = `sep [`∀ `<>fsep binders<>`.`, nest 1 body]` — binders
//!   fill-wrap aligned after the glyph-plus-space, the body indents one
//!   column past the glyph (probe:q_l2);
//! * atoms reuse the R2 fact and R1 term docs: `Fact( … ) @ tp` is an hsep
//!   (the `@ tp` never drops alone — probe:q_l5), `l = r` / `l < r` /
//!   `l ⊏ r` are `sep [lhs <+> glyph, rhs]` (probe:q_l4), `last(tp)` has no
//!   interior spaces (probe:q_at1).

use crate::ast::{Atom, Formula, VarSpec};
use crate::doc::{beside_op, beside_space, fsep, hsep, nest, render_with, sep, Doc};
use crate::rule::fact_doc;
use crate::term::{self, RIBBON, WIDTH};
use crate::web::{hl_op_char, hl_op_text, w_char as char, w_text as text};

/// Render one formula at the echo's layout parameters, starting at column 0
/// (top-level position — no enclosing parentheses).
pub fn render(f: &Formula) -> String {
    render_with(WIDTH, RIBBON, &doc(f))
}

/// The formula's `Doc` in BARE position (top level or quantifier body).
pub(crate) fn doc(f: &Formula) -> Doc {
    // Every connective / quantifier / atom glyph is `hl_operator`-spanned in
    // web mode; the parenthesized operands, `¬` and the quantifier `.` too
    // (BEHAVIOR.md "Web mode"). Identity in batch. `⊤` (True) is spanned by
    // pattern (UNOBSERVED in the web corpus — flagged).
    match f {
        Formula::True => hl_op_text("\u{22a4}"),
        Formula::False => hl_op_text("\u{22a5}"),
        Formula::Atom(a) => atom_doc(a),
        Formula::Not(g) => beside_op(hl_op_text("\u{ac}"), parens(doc(g))),
        Formula::And(l, r) => connective_doc(l, "\u{2227}", r),
        Formula::Or(l, r) => connective_doc(l, "\u{2228}", r),
        Formula::Implies(l, r) => connective_doc(l, "\u{21d2}", r),
        Formula::Iff(l, r) => connective_doc(l, "\u{21d4}", r),
        Formula::Forall(vs, b) => quantifier_doc("\u{2200}", vs, b),
        Formula::Exists(vs, b) => quantifier_doc("\u{2203}", vs, b),
    }
}

fn parens(d: Doc) -> Doc {
    beside_op(beside_op(hl_op_char('('), d), hl_op_char(')'))
}

/// `(lhs) <glyph> (rhs)` — both operands parenthesized whatever they are
/// (probe:q_p2); on overflow the glyph stays on the lhs' last line and the
/// rhs drops to the group origin (targets NSLPK3/Cronto).
fn connective_doc(l: &Formula, glyph: &str, r: &Formula) -> Doc {
    sep(vec![
        beside_space(parens(doc(l)), hl_op_text(glyph)),
        parens(doc(r)),
    ])
}

/// `∀ v1 v2 #i. body` — binders fill-wrap aligned after `∀ ` with the `.`
/// attached to the last one; the body sits beside on one line or drops to
/// (quantifier origin + 1) (probe:q_l2 bw1/bw2).
fn quantifier_doc(glyph: &str, vs: &[VarSpec], body: &Formula) -> Doc {
    let binders = fsep(vs.iter().map(|v| text(&term::var_str(v))).collect());
    // `∀ ` / `∃ ` (glyph + trailing space, one span) and the binder `.` are
    // `hl_operator`-spanned in web mode; the binders themselves plain.
    let head = beside_op(
        beside_op(hl_op_text(&format!("{glyph} ")), binders),
        hl_op_char('.'),
    );
    sep(vec![head, nest(1, &doc(body))])
}

fn atom_doc(a: &Atom) -> Doc {
    match a {
        Atom::Eq(l, r) => relation_doc(l, "=", r),
        Atom::Less(l, r) => relation_doc(l, "<", r),
        // UNOBSERVABLE placeholder (BEHAVIOR.md): rendered like Less; not
        // oracle-pinned.
        Atom::LessMset(l, r) => relation_doc(l, "<", r),
        Atom::Subterm(l, r) => relation_doc(l, "\u{228f}", r),
        Atom::Action(f, tp) => hsep(vec![fact_doc(f), hl_op_char('@'), term::doc(tp)]),
        // `last(…)` UNOBSERVED in the web corpus — left unspanned, flagged.
        Atom::Last(tp) => beside_op(beside_op(text("last("), term::doc(tp)), char(')')),
        // UNOBSERVABLE placeholder (BEHAVIOR.md): predicates are expanded
        // upstream of the echo; rendered as a bare fact, not oracle-pinned.
        Atom::Pred(f) => fact_doc(f),
    }
}

/// `lhs <glyph> rhs` for `=` / `<` / `⊏`: glyph attached to the lhs' last
/// line, rhs drops to the atom origin on overflow (probe:q_l4).
fn relation_doc(l: &crate::ast::Term, glyph: &str, r: &crate::ast::Term) -> Doc {
    // The relation glyph (`=` / `<` / `⊏`) is `hl_operator`-spanned in web mode
    // (`⊏` UNOBSERVED — spanned by pattern); the terms plain.
    sep(vec![
        beside_space(term::doc(l), hl_op_text(glyph)),
        term::doc(r),
    ])
}

/// Safety classification for the restriction wrapper's `// safety formula`
/// line: a formula is safety iff its negation-normal form contains NO
/// existential quantifier — msg-sort or temporal alike (probes q_s1/q_s2:
/// ∃ in positive polarity anywhere, including a `¬∃` antecedent that
/// double-negates back, defeats it; `¬∃` in conclusion and top-level `¬∃`
/// remain safety).
pub(crate) fn is_safety(f: &Formula) -> bool {
    !has_existential_in_nnf(f, false)
}

/// Does `f` contain a quantifier that becomes existential in NNF, given the
/// number of enclosing negations (`negated`)?
fn has_existential_in_nnf(f: &Formula, negated: bool) -> bool {
    match f {
        Formula::True | Formula::False | Formula::Atom(_) => false,
        Formula::Not(g) => has_existential_in_nnf(g, !negated),
        Formula::And(l, r) | Formula::Or(l, r) => {
            has_existential_in_nnf(l, negated) || has_existential_in_nnf(r, negated)
        }
        Formula::Implies(l, r) => {
            has_existential_in_nnf(l, !negated) || has_existential_in_nnf(r, negated)
        }
        // ⇔ expands to implications in both directions, so each side occurs
        // in both polarities. No safety-relevant ⇔ was observable (see
        // BEHAVIOR.md); this is the NNF-consistent reading.
        Formula::Iff(l, r) => {
            has_existential_in_nnf(l, false)
                || has_existential_in_nnf(l, true)
                || has_existential_in_nnf(r, false)
                || has_existential_in_nnf(r, true)
        }
        Formula::Forall(_, b) => negated || has_existential_in_nnf(b, negated),
        Formula::Exists(_, b) => !negated || has_existential_in_nnf(b, negated),
    }
}
