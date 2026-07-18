//! pretty-clean: a clean-room reimplementation of the tamarin-prover theory
//! pretty-printer (the `theory <name> begin … end` echo), derived purely from
//! black-box oracle behavior. See ../../SPEC.md for the observable boundary and
//! the R1–R4 sub-target decomposition, and workspace/BEHAVIOR.md for the
//! inferred behavioral spec you build up as you probe the oracle.
//!
//! SCAFFOLD ONLY. Every render entry point below is an UNIMPLEMENTED stub so
//! the crate compiles and the test harness runs (and fails meaningfully) from
//! day one. Fill in bodies one sub-target at a time, gating each against
//! `scripts/pretty_gate.sh` with an ALLOWLIST (see round1/).
//!
//! Recommended order: R1 (term + signature) → R2 (rules) → R3 (formula /
//! lemma / restriction) → R4 (macros / predicates).

pub mod ast;
pub mod doc; // BSD HughesPJ Doc engine — reuse graphdot's clean-room port.

pub mod formula; // R3
pub mod lemma; // R3
pub mod macros; // R4
pub mod rule; // R2
pub mod signature; // R1
pub mod term; // R1 (deep core)
pub mod theory; // top-level assembly

pub use ast::*;

/// Whole theory echo: `theory <name> begin … end`, minus the wf comment and
/// the Generated-from stamp (both appended by the caller / stripped by the
/// gate). Top-level assembly of the R1–R4 sub-targets. See `theory::render`.
pub fn render_theory(thy: &ast::Theory, sig: &ast::Signature) -> String {
    theory::render(thy, sig)
}
