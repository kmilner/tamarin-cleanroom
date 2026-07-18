//! R3 — restriction & lemma wrappers around `crate::formula::render`.
//!
//! `restriction Name: "<formula>"` and
//! `lemma Name [attrs]:\n  all-traces|exists-trace\n  "<formula>"` followed by
//! the `/* guarded formula characterizing all counter-examples: … */` comment
//! (guarded formula supplied pre-computed by the ported transform — you render
//! its text) and the no-prove proof placeholder `by sorry`. Observe lemma
//! attribute spelling (`[sources]`, `[reuse]`, `[use_induction]`, …) and the
//! quantifier keyword line from the oracle.

use crate::ast::{Guarded, Lemma, Restriction};

pub fn render_restriction(_r: &Restriction) -> String {
    unimplemented!("R3: restriction rendering")
}

pub fn render_lemma(_l: &Lemma, _guarded: Option<&Guarded>) -> String {
    unimplemented!("R3: lemma rendering")
}
