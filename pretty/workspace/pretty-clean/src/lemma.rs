//! R3 — restriction & lemma wrappers around `crate::formula::doc`.
//!
//! Byte shapes (probe provenance in workspace/BEHAVIOR.md "Restriction
//! blocks" / "Lemma blocks"):
//!
//! ```text
//! restriction Name:
//!   "formula"
//!   // safety formula          (iff the formula classifies safety)
//! <blank>
//!   /*
//!   expanded formula:
//!   "formula"
//!   */
//! ```
//!
//! ```text
//! lemma Name [attr1, attr2]:
//!   all-traces|exists-trace "formula"     (two lines when it overflows)
//! /*
//! guarded formula characterizing all counter-examples:   (| satisfying traces)
//! "<guarded block — opaque input>"
//! */
//! by sorry                                (| the embedded proof, verbatim)
//! ```

use crate::ast::{Guarded, Lemma, LemmaAttr, Restriction, TraceQuantifier};
use crate::doc::{
    above_op, above_plus, beside_op, fsep, nest, punctuate, render_with, sep, text, vcat, Doc,
};
use crate::formula;
use crate::term::{RIBBON, WIDTH};
use crate::web::{hl_comment, hl_kw, hl_wrap};

/// One whole restriction block (also the echo of legacy `axiom` items —
/// probe:q_ax1).
pub fn render_restriction(r: &Restriction) -> String {
    render_with(WIDTH, RIBBON, &restriction_doc(r))
}

/// One whole lemma block: header, quantifier/statement, guarded-formula
/// comment (skipped when `guarded` is `None`), and the proof tail
/// (`by sorry` when the lemma carries no embedded proof).
pub fn render_lemma(l: &Lemma, guarded: Option<&Guarded>) -> String {
    render_with(WIDTH, RIBBON, &lemma_doc(l, guarded))
}

/// `"formula"` — the quotes attach directly around the formula doc; interior
/// breaks land one column past the opening quote's nesting (probe:q_l2).
fn quoted_formula(f: &crate::ast::Formula) -> Doc {
    beside_op(
        beside_op(crate::doc::char('"'), formula::doc(f)),
        crate::doc::char('"'),
    )
}

/// A verbatim multi-line input block (guarded content, embedded proofs):
/// one `text` line per input line, stacked at the current nesting.
fn verbatim_doc(s: &str) -> Doc {
    vcat(s.lines().map(text).collect())
}

// ── restrictions ────────────────────────────────────────────────────────────

pub(crate) fn restriction_doc(r: &Restriction) -> Doc {
    // `restriction` is `hl_keyword`-spanned in web mode (name/colon plain); the
    // `"…"` quotes stay plain with the formula's operator spans nested inside;
    // identity in batch.
    let mut d = above_op(
        beside_op(hl_kw("restriction"), text(&format!(" {}:", r.name))),
        nest(2, &quoted_formula(&r.formula)),
    );
    if formula::is_safety(&r.formula) {
        d = above_op(d, nest(2, &hl_comment("// safety formula")));
    }
    // One blank line, then the expanded-formula comment at col 2. The STATEMENT
    // renders `r.formula` (macros unexpanded); the comment renders
    // `r.expanded` (macro/predicate-expanded — target:MacroInLemmasAndRestrictions
    // shows the two differing, e.g. statement `A( m(m3(x)) )` vs expanded
    // `A( x )`). Without macros the two are equal, so this reproduces the
    // earlier same-formula-twice observation (probes q_w1/q_pred1).
    d = above_plus(d, text(""));
    // The whole `/* expanded formula: "…" */` block is ONE `hl_comment` span in
    // web mode, with the expanded formula's operator spans nested inside
    // (`expanded formula:` plain); identity in batch.
    let comment = above_op(
        above_op(
            above_op(text("/*"), text("expanded formula:")),
            quoted_formula(&r.expanded),
        ),
        text("*/"),
    );
    above_plus(d, nest(2, &hl_wrap("hl_comment", comment)))
}

// ── lemmas ──────────────────────────────────────────────────────────────────

fn lemma_doc(l: &Lemma, guarded: Option<&Guarded>) -> Doc {
    let mut d = above_op(header_doc(l), nest(2, &statement_doc(l)));
    if let Some(g) = guarded {
        d = above_op(d, guarded_comment_doc(l.trace_quantifier, g));
    }
    let tail = match &l.proof {
        None => text("by sorry"),
        Some(p) => verbatim_doc(p),
    };
    above_op(d, tail)
}

/// `lemma Name:` or `lemma Name [attrs]:` — a space before the `[`, the
/// attribute list fill-wrapped aligned after it with `]:` attached to the
/// last item (targets 5G_AKA sqn_ue_nodecrease / sqn_ue_unique); attributes
/// keep source order and duplicates (probe:q_la1).
fn header_doc(l: &Lemma) -> Doc {
    if l.attributes.is_empty() {
        return text(&format!("lemma {}:", l.name));
    }
    let items: Vec<Doc> = l.attributes.iter().map(|a| text(&attr_str(a))).collect();
    beside_op(
        beside_op(
            text(&format!("lemma {} [", l.name)),
            fsep(punctuate(&crate::doc::char(','), items)),
        ),
        text("]:"),
    )
}

fn attr_str(a: &LemmaAttr) -> String {
    match a {
        LemmaAttr::Sources => "sources".into(),
        LemmaAttr::Reuse => "reuse".into(),
        LemmaAttr::UseInduction => "use_induction".into(),
        LemmaAttr::HideLemma(n) => format!("hide_lemma={n}"),
        LemmaAttr::Heuristic(v) => format!("heuristic={v}"),
    }
}

/// `all-traces "…"` / `exists-trace "…"` at nest 2 — one line when the whole
/// statement fits, otherwise the keyword alone with the quoted formula below
/// at the same column (probe:q_w1; target:Minimal_Loop one-line cases).
fn statement_doc(l: &Lemma) -> Doc {
    let kw = match l.trace_quantifier {
        TraceQuantifier::AllTraces => "all-traces",
        TraceQuantifier::ExistsTrace => "exists-trace",
    };
    sep(vec![text(kw), quoted_formula(&l.formula)])
}

/// The guarded-formula comment at column 0. Header text keyed by the trace
/// quantifier; content is opaque pre-computed input (probe:q_w1). Failed
/// conversions get the alternate header with the error text indented 2
/// (probe:q_r1 vs. the transform's raw error spelling).
fn guarded_comment_doc(tq: TraceQuantifier, g: &Guarded) -> Doc {
    let body = match g {
        Guarded::Formula(content) => {
            let header = match tq {
                TraceQuantifier::AllTraces => {
                    "guarded formula characterizing all counter-examples:"
                }
                TraceQuantifier::ExistsTrace => {
                    "guarded formula characterizing all satisfying traces:"
                }
            };
            above_op(text(header), verbatim_doc(content))
        }
        Guarded::Failed(err) => {
            let indented: Vec<Doc> = err
                .lines()
                .map(|ln| {
                    if ln.is_empty() {
                        text("")
                    } else {
                        text(&format!("  {ln}"))
                    }
                })
                .collect();
            above_op(text("conversion to guarded formula failed:"), vcat(indented))
        }
    };
    above_op(above_op(text("/*"), body), text("*/"))
}
