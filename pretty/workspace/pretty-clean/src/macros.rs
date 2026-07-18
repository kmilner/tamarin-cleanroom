//! `macros:` block (rule-adjacent surface — appears in the echo between the
//! signature and the rules; target:issue777, probe:p_mac1) and the
//! `predicates:` block (R4, still unimplemented).
//!
//! ```text
//! macros: m1( x, y ) =  h(<x, y>),
//!         m2( ) =  h('c')
//! ```
//!
//! * block = `text "macros: " <> sep (punctuate ',' items)` — all-or-nothing:
//!   one line when everything fits, otherwise EVERY macro on its own line
//!   aligned after `macros: ` (probe:p_mac1 — `m2( )` gets its own line even
//!   though it would fit beside `m1`);
//! * item = `hsep [name( <+> fsep params <+> ")", "= ", body]` — the head is
//!   fact-style (`m2( )` when nullary; params fill-wrap aligned after the
//!   paren) but its `)` stays ATTACHED to the last param line, and the `= `
//!   token plus hsep spacing yields the observed `) =  body` (two spaces
//!   after `=`); the body always sits beside, wrapping internally.

use crate::ast::{Macro, Predicate};
use crate::doc::{
    beside_op, beside_space, char, fsep, hsep, punctuate, render_with, sep,
    text, Doc,
};
use crate::term::{self, RIBBON, WIDTH};

pub fn render_macros(macros: &[Macro]) -> String {
    render_with(WIDTH, RIBBON, &macros_doc(macros))
}

fn macros_doc(macros: &[Macro]) -> Doc {
    beside_op(
        text("macros: "),
        sep(punctuate(
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

pub fn render_predicates(_preds: &[Predicate]) -> String {
    unimplemented!("R4: predicates block")
}
