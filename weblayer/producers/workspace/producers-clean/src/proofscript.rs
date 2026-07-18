//! R2 — the proof-script WEST pane (the theory index left of every page).
//!
//! A flat line sequence: the `theory NAME begin` header, one link line per top
//! item (message / rules / tactic / sources), an `add lemma` link, then per
//! lemma a declaration (name + attributes + quantifier + the opaque formula
//! body) followed by its proof display (a single `by sorry` step, or the R3
//! proof tree) and a trailing `add lemma`, then `end`. Reuses R1's postprocess
//! and R5's link construction; the formula/method text is opaque input.
//!
//! Observe the west pane inside the `overview` targets (present in every
//! overview capture; the lemma declaration is proof-invariant, so the fresh
//! no-prove state exercises the whole line grammar without proof-tree content).

use crate::model::ProofScriptPane;

/// Render the inner HTML of the west proof-script pane.
pub fn render_index(_thy: &ProofScriptPane) -> String {
    // TODO(sealed): R2. Emit the item links, per-lemma declaration + proof
    // display, add/edit/delete links; postprocess once.
    unimplemented!("R2: proof-script west pane")
}
