//! R4 — `macros:` and `predicates:` blocks (small, self-contained mop-up).
//!
//! `macros: Name(a, b) = <body>, …` (bodies via `crate::term::render`) and
//! `predicates: Name(x, y) <=> <formula>, …` (via `crate::formula::render`).
//! Observe the exact separators, `=`/`<=>` spelling, and wrapping.

use crate::ast::{Macro, Predicate};

pub fn render_macros(_macros: &[Macro]) -> String {
    unimplemented!("R4: macros block")
}

pub fn render_predicates(_preds: &[Predicate]) -> String {
    unimplemented!("R4: predicates block")
}
