# Required public API (interoperability surface) — pretty-clean

The clean crate must expose these entry points so the open side can wire it in
behind thin adapters that translate the live `tamarin_parser::ast` /
closed-theory values into the crate's own AST subset. Every function returns
text that is byte-identical to the oracle's no-prove theory echo for the
corresponding fragment. Names are indicative; the boundary that matters is
"one entry point per observable sub-target, each independently gate-checkable".

```rust
// ── (top) whole theory echo: `theory <name> begin … end` (minus the wf
//    comment and the Generated-from stamp, which are appended by callers). ──
pub fn render_theory(thy: &Theory, sig: &Signature) -> String;

// ── (a) signature block: builtins / functions / equations declarations ──
pub fn render_signature_block(sig: &Signature) -> String;

// ── (b) macros / predicates blocks ──
pub fn render_macros(macros: &[Macro]) -> String;        // "macros: …"
pub fn render_predicates(preds: &[Predicate]) -> String; // "predicates: …"

// ── (c) rule rendering: one multiset-rewrite rule, plus fact & TERM render ──
pub fn render_rule(rule: &Rule, variants: Option<&AcVariants>) -> String;
pub fn render_fact(fact: &Fact) -> String;               // `Foo( a, b )`
pub fn render_term(term: &Term) -> String;               // THE deep core

// ── (d) restriction / lemma FORMULA rendering ──
pub fn render_restriction(r: &Restriction) -> String;
pub fn render_lemma(l: &Lemma, guarded: Option<&Guarded>) -> String;
pub fn render_formula(f: &Formula) -> String;            // ∀ ∃ ⇒ ∧ ∨ ¬ @ ⊏ …

// Layout primitive (BSD-provenance, NOT part of the GPL-erasure surface):
// the HughesPJ `Doc` engine + combinators. Reuse the graphdot cluster's
// clean-room engine (workspace/graph-clean/src/pretty.rs, derived from
// sanctioned/pretty-1.1.3.6). Width = 110, ribbon = 73 for the theory echo.
```

`Theory`, `Term`, `Fact`, `Formula`, `Atom`, `Rule`, `Lemma`, `Restriction`,
`Macro`, `Predicate`, `VarSpec`, `SortHint`, `BinOp`: see `ast_types.rs`.

`Signature`, `AcVariants`, `Guarded` are pre-computed inputs supplied by the
ported closure/solver (SPEC.md "Solver-entangled inputs"); the clean crate
only RENDERS them, and their observable text is learned from the oracle.
