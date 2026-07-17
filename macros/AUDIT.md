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

## Round 4 audit

Reviewer: similarity auditor (both-sides). Scope: THIS round's delta only, from
clean-room HEAD `63ed8a9`. Restricted `git diff`/untracked set under `macros/`:
`workspace/macro-clean/src/{lib.rs,tests.rs}`, `BEHAVIOR.md`, `QUERIES.log`,
`REPORT.md`, `run_tests.sh`, plus new `formula_parity.sh`, `probes/`, `fixtures/`,
`captures/`. Compared against upstream Haskell: `lib/term/src/Term/Macro.hs`;
the parser's nullary/signature machinery
(`Theory/Text/Parser/{Term.hs (`nullaryApp`, l.143), Token.hs
(`addMacrosToSignature`, `macroToFunSym`, l.197)}`); the item-level appliers
(`applyMacroIn{Rule,Fact,Formula,Restriction,Lemma}` and their call sites in
`ClosedTheory.hs`, `Prover.hs`, `Parser.hs`, `Wellformedness.hs`,
`Model/Formula.hs`).

Delta content: (a) bare untagged nullary-macro name resolution [Q32–Q36];
(b) `Macros` items now preserved in place instead of dropped [Q37]; (c) new
probes/fixtures for acc-lemma [Q38] & case-test [Q39] formula expansion (the
`expand_item` arms for `AccLemma`/`CaseTest` were already present in round 1 —
this round only adds evidence, no new code path); (d) `formula_parity.sh`.

**Findings: none (0).** No copied protectable expression in the delta.

Abstraction-filtration-comparison of the delta:

- Bare-nullary resolution is a genuinely DIFFERENT mechanism, not a mirror. The
  reference resolves a parenthesis-free nullary use at PARSE time: macro names
  are injected into the Maude signature as arity-0 `NoEq`/Private/Destructor
  funsyms (`addMacroSym` via `macroToFunSym`), and `nullaryApp` (Term.hs:143)
  matches a bare token against `funSyms ∪ macroNames`, emitting an `FApp fs []`;
  `applyMacros` then expands it as an ordinary call. The clean unit has no
  signature and no parser; it resolves at expansion time a `Term::Var` whose
  `sort == Untagged` and whose name is a **nullary** table entry (lib.rs:97-102).
  Sort-gating (`~x`/`$x` excluded) and nullary-gating are behavior-dictated
  (observed: `bare_sorts.out` keeps `~konst`/`$konst`; `bare_nonnullary.out`
  keeps `A( m )` as an unbound var) — the load-bearing choice (where and how the
  bare name becomes a use) differs from the reference.
- Name-reservation over a same-named formal [Q36] is likewise reached by a
  different route. The reference gets it for free from the parser: inside a body
  a name equal to an earlier nullary macro parses as `FApp base []`, never as a
  bound var, so the formal can't match. The clean unit gets the same observable
  result from its "expand each body against strictly-earlier macros BEFORE
  binding formals" order (`build_table` lib.rs:73-84 + the Var branch): a
  documented round-1 structural choice, not copied. `formal_vs_nullary.out`
  dictates the result (`A( <h('k'), h('k')> )`, formal+arg dropped).
- Preserve-in-place [Q37] is behavior-dictated: every full-close capture
  reprints `macros: name( args ) = body` with the original body
  (`bare_nullary_macro.out`, `formal_vs_nullary.out`, `acc_both_*.out`). The
  delta merely deletes the round-1 filter (expand.rs `expand`) — removing code,
  not importing any.
- Acc-lemma/case-test expansion [Q38,Q39]: the reference has NO per-`AccLemma`
  or per-`CaseTest` macro applier; it expands via the generated lemmas + derived
  `predicate:` produced by accountability translation (whence the observed lemma
  names `acc_blamed_{suff,verif_nonempty,min,uniq,inj,single}`, `acc_verif_empty`
  in `acc_both_*.out`). The clean unit instead rewrites `AccLemma.formula` /
  `CaseTest.formula` directly on the source AST. Different structural placement;
  no shared organization.
- Identifiers/comments/constants: the delta introduces no reference identifier
  (`applyMacros`, `macroToFunSym`, `findMatchingMacro`, `lnMacrosToBNMacros`,
  `BNMacro`, `addMacrosToSignature` all absent); its names (`MacroTable`,
  `expand_term`, `substitute_term`, `build_table`) are clean-unit-local. The
  accountability lemma-name suffixes appear only in `BEHAVIOR.md` prose and are
  boundary-observable in `captures/acc_both_*.out` — not lifted into code, not
  protectable. No reference comment phrasing is echoed. No magic constant that
  is not observable at the oracle boundary was introduced.

Behavioral-claim ↔ probe cross-check (all trace to logged probes/fixtures):
Q32 → `probes/{bare_nullary,paren_nullary,bare_prem,bare_inarg,bare_msgsort}`;
Q33 → `probes/bare_chain` + test `bare_nullary_transitive_in_body`;
Q34 → `probes/bare_sorts` + `captures/bare_sorts.out`;
Q35 → `probes/bare_nonnullary` + `captures/bare_nonnullary.out`;
Q36 → `probes/formal_vs_nullary` + `captures/formal_vs_nullary.out`;
Q37 → every full-close capture; Q38/Q39 → acc/case-test fixtures verified by
`formula_parity.sh` + `captures/acc_both_*.out`. `cargo test` = 21/21 pass,
matching the claimed count.

Non-blocking observations (not similarity violations; no redo):
- Stale comment. `lib.rs:426-428` still reads "Macros items are filtered out
  before this function is called" — false since the round removed that filter
  (`Macros(_)` now flows through `expand_item` and is returned via `it.clone()`).
  Current-state-accuracy nit only.
- Evidence completeness. The `konst:msg` parse-error sub-claim (Q32) and the
  transitive-body claim (Q33) have logged probe files but no saved `.out`
  capture; Q33's AST behavior is independently covered by a cargo test. Does not
  affect the similarity verdict.
- Grounding note. The BEHAVIOR §2.5 "consumer interface contract" asserts a bare
  nullary use reaches `expand` as `Term::Var("konst")` (not an `App`). This is a
  model of the clean unit's OWN downstream parser, not a reference observation;
  the reference in fact produces an `App`. The code is robust either way (the
  `App` arm resolves an arity-0 call identically), so it is harmless — and the
  divergence from the reference's representation is evidence of independent
  derivation, not a breach.

Verdict: pass.
