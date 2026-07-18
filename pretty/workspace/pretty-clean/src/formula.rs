//! R3 — trace-formula rendering (shared by restrictions and lemma statements).
//!
//! Glyphs: `∀ ∃ ⇒ ∧ ∨ ¬ ⊤ ⊤ ⊥ @ < = ⊏ last(..)`. Observe from the oracle:
//! quantifier binder rendering (`∀ x #i.`), the exact parenthesization of
//! nested connectives, temporal-variable form `#i`, atom rendering
//! (`Fact( .. ) @ #i`, `#i < #j`, `x = y`, `x ⊏ y`), and the multi-line
//! wrapping of long formulas at the theory width. Atom term args route through
//! `crate::term::render`.

use crate::ast::Formula;

/// Port of `prettyLNFormula`. Byte-identical to the oracle.
pub fn render(_f: &Formula) -> String {
    unimplemented!("R3: formula rendering")
}
