# Similarity audit — macros / macro-clean

## Round 1 audit

Reviewer: similarity auditor (both-sides). Compared
`workspace/macro-clean/src/{lib.rs,ast.rs}` against HASKELL
`lib/term/src/Term/Macro.hs`, the macro handling threaded through
`lib/theory/src/TheoryObject.hs` and the per-item `applyMacroIn*` functions
(`Theory/Model/{Rule,Fact,Formula,Restriction}.hs`, `Lemma.hs`,
`ClosedTheory.hs`), and `lib/theory/src/Theory/Text/Parser/Macro.hs`.

**Findings: none (0).**

The core algorithm is materially different, not a mirror:

- Architecture. Haskell has NO single whole-theory expansion pass; macro
  application is scattered and lazy — `applyMacroInRule`, `applyMacroInFormula`
  (via generic `mapAtoms`), `applyMacroInLemma`, `applyMacroInFact`, etc., each
  invoked at different pipeline stages (rule closing, proving, wellformedness,
  export) over a flat unexpanded `[Macro]` list. The clean unit implements a
  single eager `expand(theory) -> theory` (lib.rs:44) that walks the clean AST
  once and drops the declarations — the design the SPEC mandates. The traversal
  (`expand_item`/`expand_rule`/`expand_formula`/…) is dictated by the vendored
  clean AST (ast.rs), so visiting every term-bearing field is forced, not copied.
- Term expansion. Haskell `applyMacros` (Macro.hs:40) keeps bodies unexpanded,
  matches a funsym by name+arity (`macroToFunSym`), and RE-RUNS `applyMacros` on
  the substituted body to expand nested calls. The clean unit instead
  pre-expands every body into a DAG-ordered `MacroTable` (`build_table`,
  lib.rs:65) keyed by name, then does a single `substitute_term` with NO re-run
  (lib.rs:82,141). Different accounting (memoized table + separate substitution
  vs. flat-list lazy re-expansion).

Shared shape that is filtered out: "expand args, then substitute formals→args
into the body" is the only correct eager, capture-avoiding, parallel-substitution
order, dictated by the observed macro semantics ([Q7]); the load-bearing
structural choice (pre-expand vs. re-run) differs.

Verdict: pass.
