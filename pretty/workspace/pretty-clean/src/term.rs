//! R1 — TERM rendering (the deep core; every other sub-target reuses this).
//!
//! `Term` → surface text as Tamarin's UI shows it. Learn from the oracle:
//!   * infix AC ops render with their glyphs (`*`, `⊕`=U+2295, `++`, `%+`) and
//!     their own precedence/parenthesization;
//!   * `exp(a,b)` → `a^b`; `diff(a,b)` stays `diff(a, b)`;
//!   * `pair`-trees flatten into `<a, b, c>`;
//!   * constants → `'name'`; the `%1` / `tone` constant → `%1`;
//!   * variables → `~k` `$pk` `#i` `%n` … with `.idx` suffix when idx > 0.
//! Nail spacing and parenthesization against `oracle/pretty_oracle.sh` — the
//! signature `equations:` block and every fact argument surface term text.

use crate::ast::Term;

/// Port of `prettyLNTerm`/`prettyTerm`. Byte-identical to the oracle.
pub fn render(_t: &Term) -> String {
    // TODO(sealed): R1. Derive glyphs/precedence/spacing from the oracle.
    unimplemented!("R1: term rendering")
}
