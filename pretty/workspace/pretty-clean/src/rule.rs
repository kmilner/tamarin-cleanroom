//! R2 — rule rendering: `rule (modulo E) Name: [prems] --[acts]-> [concs]`.
//!
//! Facts render as `!Name( a, b )` (persistent `!`), arguments via
//! `crate::term::render`. Observe the premise/action/conclusion bracket
//! layout and wrapping, rule attributes, the `variants (modulo AC)` block
//! (substitutions supplied pre-computed — you render their text), the
//! `/* has exactly the trivial AC variant */` comment, and loop-breaker lines.

use crate::ast::{AcVariants, Fact, Rule};

/// One fact: `Name( arg, … )`, persistent prefixed with `!`.
pub fn render_fact(_f: &Fact) -> String {
    unimplemented!("R2: fact rendering")
}

/// One multiset-rewrite rule, with its AC-variant block if present.
pub fn render(_rule: &Rule, _variants: Option<&AcVariants>) -> String {
    unimplemented!("R2: rule rendering")
}
