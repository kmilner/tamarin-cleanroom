//! Top-level assembly: `theory <name>\n\nbegin\n … \nend`.
//!
//! Emits the header (`theory <name>` / `begin`), then each section in the
//! oracle's order (signature, macros, predicates, rules, restrictions,
//! lemmas), then `end`. The trailing wellformedness comment and the
//! `/* Generated from: … */` stamp are NOT emitted here — they are appended by
//! the caller (wf is a separate slice; the stamp is volatile) and the
//! acceptance gate strips both from each side. Section spacing/blank-line
//! discipline is observed from the oracle.

use crate::ast::{Signature, Theory};

pub fn render(_thy: &Theory, _sig: &Signature) -> String {
    // TODO(sealed): assemble sections once R1–R4 land. Reuse
    // signature::render, macros::*, rule::render, lemma::*.
    unimplemented!("top-level theory assembly")
}
