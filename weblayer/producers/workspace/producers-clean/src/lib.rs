//! producers-clean: a clean-room reimplementation of the tamarin-prover
//! interactive web UI's fragment PRODUCERS — the code that renders pre-computed
//! prover values into the HTML/JSON response-body CONTENT that the already
//! clean-roomed dispatch layer serves. Derived purely from black-box
//! observation of captured responses + the live oracle. See ../../SPEC.md for
//! the observable boundary and the R1–R5 sub-target decomposition, and
//! workspace/BEHAVIOR.md for the behavioral spec you build up as you probe.
//!
//! Status: R1 (center section fragments + the shared HTML skin) IMPLEMENTED —
//! `html` (escape / postprocess / envelopes) and `section` (`render_pane`,
//! `render_help_pane`), gated by tests/round1_center_fragments.rs (fixtures
//! pinned to observed bytes) and tests/corpus_sweep.rs (reassembly byte-parity
//! over all 81 capture manifests). R2–R5 remain UNIMPLEMENTED stubs.
//!
//! Order: R1 (done) → R2 (proof-script west pane) → R3 (proof-tree / method
//! HTML) → R4 (welcome / housekeeping) → R5 (theory-path grammar).
//!
//! In scope: pure-render producers — given pre-computed prover values, emit the
//! fragment bytes (tags, links, headings, escaping, line breaks, envelope).
//! OUT of scope (opaque input / other clusters): the pretty-printed content
//! text itself (formula / rule / signature / method text), the constraint
//! system and applicable-proof-methods panes, graph DOT/SVG, and the Rust-only
//! progressive-UI route.

pub mod model;

pub mod html; // R1 — shared skin: escaping, line postprocess, JSON envelope
pub mod section; // R1 — theory-view center section fragments (round-1 target)
pub mod proofscript; // R2 — proof-script west pane (theory index)
pub mod prooftree; // R3 — proof-tree / proof-method HTML
pub mod welcome; // R4 — welcome/index page + housekeeping
pub mod path; // R5 — theory-path grammar (URL <-> structured)

pub use model::*;

/// Round-1 entry point: render a theory-view center-section content pane
/// (`main/message` / `main/rules` / `main/tactic`) to its response-body bytes.
/// Delegates to [`section::render_pane`].
pub fn render_content_pane(pane: &model::ContentPane) -> String {
    section::render_pane(pane)
}

/// Round-1 entry point: render the `main/help` pane (env line + static help
/// block) to its response-body bytes. Delegates to
/// [`section::render_help_pane`].
pub fn render_help_pane(help: &model::HelpPane) -> String {
    section::render_help_pane(help)
}
