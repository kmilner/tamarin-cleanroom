//! R1 — signature block: `builtins:` / `functions:` / `equations:`.
//!
//! Renders the CLOSED signature (merged builtin + user symbols, canonically
//! sorted by the ported closure). Observe from the oracle: the exact ordering,
//! the `/N` arity suffix, the `[private]`/`[destructor]`/`[constructor]`
//! attribute spelling, comma/line wrapping, and the leading
//! `// Function signature and definition of the equational theory E` comment.
//! Equation LHS/RHS route through `crate::term::render`.

use crate::ast::Signature;

/// Full signature section (builtins, functions, equations) as one block.
pub fn render(_sig: &Signature) -> String {
    // TODO(sealed): R1. Reuse term::render for equation sides.
    unimplemented!("R1: signature block")
}
