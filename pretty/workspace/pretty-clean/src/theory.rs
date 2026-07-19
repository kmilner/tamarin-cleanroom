//! Top-level assembly (GAP 3): the `theory <name> begin … end` echo frame.
//!
//! The frame is a pure STRING assembly — every block renders through the
//! existing R1–R4 entry points (`signature::render`, `macros::render_macros` /
//! `render_predicates`, `rule::render`, `lemma::render_restriction` /
//! `render_lemma`); the frame contributes only the header/footer, the item
//! ORDER, and the inter-block blank-line rhythm.
//!
//! Observed shape (targets round1-3/*.hs.txt + fresh captures — see
//! workspace/BEHAVIOR.md "Theory frame"):
//!
//! ```text
//! theory NAME
//! <blank>
//! begin
//! <blank>
//! // Function signature and definition of the equational theory E
//! <blank>
//! functions: …            (the signature block)
//! equations: …
//! <blank>
//! <item 0>                (items in source order, blank-line separated)
//! <blank>
//! <item 1>
//! …
//! <blank>
//! <blank>
//! <blank>
//! end
//! ```
//!
//! * the header is always `theory NAME` then a blank then `begin` then a blank;
//! * the signature block is always first (right after `begin`), from the
//!   separate `Signature` — never a `TheoryItem`;
//! * successive items (rules, restrictions, lemmas, macros, predicates,
//!   heuristic, verbatim tactic/section blocks) are separated by ONE blank
//!   line; the item order is the echo's source order;
//! * before `end` there are THREE blank lines. These are the residue the gate's
//!   extraction leaves after dropping the two trailing comment blocks that are
//!   OUT of this crate's span (the wellformedness report and the
//!   `Generated from:` build stamp): each was one blank-separated slot, plus
//!   the blank before `end`, so `<last item>\n\n\n\nend` (RAW-tail verified —
//!   BEHAVIOR.md). Every observed capture ends with exactly this.

use crate::ast::{Signature, Theory, TheoryItem};
use crate::{lemma, macros, rule, signature};

/// Whole theory echo `theory NAME begin … end`, minus the trailing
/// wellformedness comment and `Generated from:` stamp (OUT of span — their
/// blank-line residue IS reproduced, see the module doc).
pub fn render(thy: &Theory, sig: &Signature) -> String {
    let mut parts: Vec<String> = Vec::with_capacity(thy.items.len() + 3);
    parts.push(format!("theory {}", thy.name));
    parts.push("begin".to_string());
    parts.push(signature::render(sig));
    for item in &thy.items {
        parts.push(render_item(item));
    }
    // Every part is joined by one blank line; the tail before `end` carries the
    // extra two blank slots the stripped wf/stamp comments leave behind.
    format!("{}\n\n\n\nend", parts.join("\n\n"))
}

/// Render one item through its R1–R4 entry point (or emit it verbatim).
fn render_item(item: &TheoryItem) -> String {
    match item {
        TheoryItem::Macros(ms) => macros::render_macros(ms),
        TheoryItem::Predicates(ps) => macros::render_predicates(ps),
        TheoryItem::Rule(r, v) => rule::render(r, v.as_ref()),
        TheoryItem::Restriction(r) => lemma::render_restriction(r),
        TheoryItem::Lemma(l, g) => lemma::render_lemma(l, g.as_ref()),
        TheoryItem::Heuristic(h) => format!("heuristic: {h}"),
        TheoryItem::Verbatim(s) => s.clone(),
    }
}
