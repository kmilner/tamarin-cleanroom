//! producers-clean: a clean-room reimplementation of the tamarin-prover
//! interactive web UI's fragment PRODUCERS — the code that renders pre-computed
//! prover values into the HTML/JSON response-body CONTENT that the already
//! clean-roomed dispatch layer serves. Derived purely from black-box
//! observation of captured responses + the live oracle. See ../../SPEC.md for
//! the observable boundary and the R1–R5 sub-target decomposition, and
//! workspace/BEHAVIOR.md for the behavioral spec you build up as you probe.
//!
//! Status: R1 (center section fragments + the shared HTML skin), R5 (the
//! theory-path grammar), R2 (the proof-script west pane assembly), R3 (the
//! structured proof-tree lines the pane embeds) and R4 (the welcome/index
//! page + housekeeping bodies) IMPLEMENTED — `html` (escape / postprocess /
//! envelopes), `section` (`render_pane`, `render_help_pane`), `path`
//! (`parse` / `render`), `proofscript` (`render_index`, consuming R5 for
//! every link and R3 for every proof display), `prooftree` (`render_tree`),
//! and `welcome` (`render_welcome` + housekeeping bodies). Gated by
//! tests/round1_center_fragments.rs + tests/corpus_sweep.rs (R1: 324-fragment
//! byte parity), tests/r5_path_grammar.rs (R5: live-probe acceptance replay +
//! 40037-tail corpus byte round-trip), tests/r2_west_pane.rs (R2+R3: all 478
//! overview west panes sliced to opaque formula/method text and re-rendered
//! byte-identically + live-probe replays), tests/r3_proof_tree.rs (R3
//! fixtures + live autoprove replays), and tests/r4_welcome.rs (R4 live-probe
//! byte replays).
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

/// Round-2 entry point: parse a raw theory-path wildcard tail (the part after
/// `/thy/trace/<idx>/main/`, still percent-encoded) into a structured path.
/// Delegates to [`path::parse`].
pub fn parse_path(raw: &str) -> Option<model::ThyPath> {
    path::parse(raw)
}

/// Round-2 entry point: render a structured theory path to its href segments
/// (percent-encoded; join with `/`). Delegates to [`path::render`].
pub fn render_path(p: &model::ThyPath) -> Vec<String> {
    path::render(p)
}

/// Round-2 entry point: render the proof-script WEST pane (the theory index
/// shown left of every page) to its inner HTML. Delegates to
/// [`proofscript::render_index`].
pub fn render_proof_script(pane: &model::ProofScriptPane) -> String {
    proofscript::render_index(pane)
}

/// Round-3 entry point: render a lemma's proof tree as the pre-postprocess
/// document fragment the west pane embeds (logical lines joined with
/// newlines). Delegates to [`prooftree::render_tree`].
pub fn render_proof_tree(index: u64, lemma: &str, tree: &model::ProofTree) -> String {
    prooftree::render_tree(index, lemma, tree)
}

/// Round-3 entry point: render the welcome / index (`/`) page body.
/// Delegates to [`welcome::render_welcome`].
pub fn render_welcome(w: &model::Welcome) -> String {
    welcome::render_welcome(w)
}
