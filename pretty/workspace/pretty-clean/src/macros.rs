//! `macros:` block (rule-adjacent surface — appears in the echo between the
//! signature and the rules; target:issue777, probe:p_mac1, probe:r5_mac2) and
//! the `predicate:` blocks (probe:features_predicates_minimal / timepoints).
//!
//! ```text
//! macros: m1( x, y ) =  h(<x, y>),
//!         m2( ) =  h('c')
//! ```
//!
//! * block = `text "macros: " <> vcat (punctuate ',' items)` — the block
//!   ALWAYS breaks: the first macro sits beside `macros: ` and every
//!   subsequent macro is on its own line aligned after `macros: ` (col 8),
//!   REGARDLESS of fit (probe:r5_mac2 — two short macros that would fit one
//!   line still break; probe:p_mac1; target:issue777 — a lone macro is
//!   trivially one line). Commas attach to the preceding macro; the last
//!   macro carries none;
//! * item = `hsep [name( <+> fsep params <+> ")", "= ", body]` — the head is
//!   fact-style (`m2( )` when nullary; params fill-wrap aligned after the
//!   paren) but its `)` stays ATTACHED to the last param line, and the `= `
//!   token plus hsep spacing yields the observed `) =  body` (two spaces
//!   after `=`); the body always sits beside, wrapping internally.
//!
//! Predicates render one `predicate: <fact><=><formula>` per predicate, the
//! group stacked with a blank line between successive predicates
//! (target:features_predicates_minimal, target:timepoints, target:dmn-basic —
//! no spaces around `<=>`; the fact uses R2 fact spacing, the body the R3
//! formula printer). The body formula is rendered at ABSOLUTE margin 0 and the
//! header is textually prepended to its first line: a wrapping body wraps at
//! column 1 (the formula's own nesting from the margin) INDEPENDENT of the
//! header width — pinned by dmn-basic, where `Sender_duplicate` and
//! `Mixer_duplicate` bodies both wrap at column 1 despite different name
//! lengths, and a 66-column body row fits ribbon 73 measured from column 1, not
//! from the `<=>` column. A `<>`-beside composition would instead indent the
//! body under `<=>` (the sanctioned HughesPJ display threads the line width
//! into continuations), so the header is spliced textually here.

use crate::ast::{Fact, Macro, Predicate, Term};
use crate::doc::{beside_op, beside_space, fsep, hsep, punctuate, render_with, vcat, Doc};
use crate::rule::render_fact;
use crate::term::{self, RIBBON, WIDTH};
use crate::web::{w_char as char, w_text as text};

pub fn render_macros(macros: &[Macro]) -> String {
    render_with(WIDTH, RIBBON, &macros_doc(macros))
}

pub(crate) fn macros_doc(macros: &[Macro]) -> Doc {
    beside_op(
        text("macros: "),
        vcat(punctuate(
            &char(','),
            macros.iter().map(macro_doc).collect(),
        )),
    )
}

fn macro_doc(m: &Macro) -> Doc {
    let head = beside_space(
        beside_space(
            text(&format!("{}(", m.name)),
            fsep(punctuate(
                &char(','),
                m.params.iter().map(term::doc).collect(),
            )),
        ),
        char(')'),
    );
    hsep(vec![head, text("= "), term::doc(&m.body)])
}

/// `predicate: <fact><=>formula` per predicate, the group stacked with a blank
/// line between successive predicates — the frame treats the group as one item
/// (BEHAVIOR.md "Predicates block").
pub fn render_predicates(preds: &[Predicate]) -> String {
    preds
        .iter()
        .map(render_one_predicate)
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// `predicate: Name( args )<=>formula` — the `predicate: ` prefix, the R2 fact
/// head built from the predicate's name + parameter variables, the bare `<=>`
/// glyph (no surrounding spaces), then the R3 formula body rendered at margin 0
/// and spliced after the header (its first line follows `<=>`; a wrapping body
/// keeps its margin-relative indentation — see the module note).
fn render_one_predicate(p: &Predicate) -> String {
    let fact = Fact {
        persistent: false,
        name: p.name.clone(),
        args: p.params.iter().map(|v| Term::Var(v.clone())).collect(),
        annotations: vec![],
    };
    format!(
        "predicate: {}<=>{}",
        render_fact(&fact),
        crate::formula::render(&p.body)
    )
}
