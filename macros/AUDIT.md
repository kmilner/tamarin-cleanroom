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

## Round 5 audit

Reviewer: similarity auditor (both-sides). Scope: THIS round's delta only, from
clean-room HEAD `8901219`. Restricted `git diff` under `macros/`:
`workspace/macro-clean/src/{lib.rs,tests.rs}`, `BEHAVIOR.md`, `QUERIES.log`,
`REPORT.md`. (Untracked `weblayer/round5/` is out of unit-E scope, ignored.)
Compared against upstream Haskell: `lib/term/src/Term/Macro.hs` and the
per-stage macro appliers + their call sites — `Theory/Text/Parser.hs`
(`parseLemmaWithMacros`, l.96–105), `Prover.hs` (l.89, l.197–199),
`ClosedTheory.hs` (`applyMacroInProtoRule`/`applyMacroInDiffProtoRule`,
l.318–323), `Rule.hs` (`closeProtoRule`, l.95–99),
`Theory/Model/Rule.hs` (`applyMacroInRule`, l.1042), and
`Theory/Text/Parser/Accountability.hs` (`caseTest`/`lemmaAcc`).

Delta content: a private `Mode` enum (`FullClose`/`Staged`) threaded ONLY through
`expand_item` and `expand_rule`; a second public entry `expand_staged` (and
`expand_with`) alongside the unchanged `expand`; two `Staged` carve-outs —
(a) `AccLemma`/`CaseTest` formulas left untouched [Q41], (b) only the primary
rule form rewritten, `variants`/`left_right` carried verbatim [Q42]; 5 new tests;
and the round-4 stale-comment nit in `expand_item` fixed.

**Findings: none (0).** No copied protectable expression in the delta.

Abstraction-filtration-comparison of the delta:

- Two-entry-point / `Mode` structure is the clean unit's OWN abstraction, not a
  mirror. Upstream has no expansion "mode" flag: it has a family of per-type
  appliers (`applyMacroIn{Rule,Fact,Formula,Lemma,Restriction,ProtoRule,
  DiffProtoRule}`) invoked at physically different pipeline stages (parser vs.
  `closeTheory`/`closeDiffTheory` vs. wellformedness vs. export). The clean unit
  unifies its two call-sites into one traversal parameterised by a local enum.
  Different organisation; the enum names (`FullClose`/`Staged`), `expand_staged`,
  `expand_with` are clean-unit-local — none appear upstream.
- Carve-out (b) is behavior-dictated merger, not copied structure. It is true
  that upstream `applyMacroInProtoRule`/`applyMacroInDiffProtoRule`
  (ClosedTheory.hs:322–323) also expand only the primary `ruE` and pass
  `variants`/`sides` through untouched, and `applyMacroInRule` (Rule.hs:1042)
  has no variants field at all — so the *observable* Staged behavior coincides
  with an upstream behavior. But that coincidence is forced, not lifted: I
  reproduced `[Q42]` verbatim against the live oracle (`issue777.spthy` and
  `MacroInLemmasAndRestrictions.spthy` under `--parse-only`) — at parse stage a
  rule prints ONLY as `rule (modulo E) <name>:` with no `(modulo AC)` block and
  no diff projection, so the derived fields are empty and "expand primary, carry
  the derived fields verbatim" is the single expression the consumer AST admits.
  The mechanism differs regardless: upstream RE-DERIVES AC variants from the
  expanded `ruE` via Maude (`variantsProtoRule hnd (applyMacroInRule …)`),
  whereas the clean unit's *FullClose* reproduces the close-stage output by
  recursing into the already-present variant/`left_right` forms — a strategy with
  NO upstream analog (upstream never macro-recurses into a variant). FullClose is
  unchanged this round and stays gated by the round-4 byte-parity fixtures.
- Carve-out (a) is behavior-dictated. I reproduced `[Q41]` verbatim against the
  live oracle (`acc_both_macro.spthy --parse-only`): the case-test prints
  `test blamed: "∃ sid #i. Blame( mtest(sid), x ) @ #i"` and the acc-lemma
  `blamed accounts for "¬(∃ sid #i. Unequal( mfor(sid) ) @ #i)"` — both macro
  CALLS un-expanded at parse stage; the close-stage guarded forms are expanded
  ([Q38,Q39]). "A later stage owns them" is a legitimate black-box inference
  from parse-vs-close, both logged. Upstream corroborates but is not the source:
  the parser (`Accountability.hs`) only builds `AccLemma`/`CaseTest` with the raw
  formula; expansion is downstream in accountability translation. The clean unit
  rewrites `.formula` directly on the source AST — different placement.
- Identifiers/comments/constants: the delta reproduces no upstream source name
  (`applyMacroInProtoRule`, `applyMacroInDiffProtoRule`, `OpenProtoRule`,
  `DiffProtoRule`, `ruE`, `sides`, `variantsProtoRule`, `closeProtoRule`,
  `theoryMacros`, `parseLemmaWithMacros` all absent). The tamarin surface tokens
  the comments quote — `(modulo E)`, `(modulo AC)`, `rule (modulo E) <name>:`,
  `test <name>: "..."`, `... accounts for "..."` — all appear verbatim in the
  parse-only / close-stage oracle OUTPUT I captured, i.e. boundary-observable
  scenes-a-faire, not lifted internal expression. Upstream's own nearby comments
  (e.g. Rule.hs:96 on new-var overwrite in diff mode) are NOT echoed. No
  non-boundary magic constant introduced.

Behavioral-claim ↔ probe cross-check: both new claims trace to logged probes in
`QUERIES.log`, and I INDEPENDENTLY reproduced each verbatim against the live HS
oracle this audit:
Q41 → `fixtures/acc_both_macro.spthy --parse-only` (acc/case-test calls preserved);
Q42 → `examples/regression/trace/issue777.spthy` +
`examples/features/macros/MacroInLemmasAndRestrictions.spthy --parse-only`
(primary-only rule form, no derived block). `cargo test` = 26/26 pass, matching
the claimed count (21 prior + 5 new staged-mode tests, incl. a full-close
contrast that pins the mode difference).

Non-blocking observations (not similarity violations; no redo):
- Capture trail. Q41/Q42 quote parse-only output in `QUERIES.log` but no matching
  `.out` capture was committed this round (prior rounds committed a `captures/*.out`
  per probe; the existing `acc_both_macro.out`/`example_issue777.out`/
  `example_MacroInLemmasAndRestrictions.out` are CLOSE-stage runs, not the cited
  `--parse-only`). I reran both under `--parse-only` and they match the logged
  quotes byte-for-byte, so the claims are sound; committing the two parse-only
  captures would close the evidence trail. Does not affect the verdict.
- Contract framing. `expand_staged`'s end-to-end behavior (eagerly expand
  ordinary lemmas/restrictions/primary-rule + carve out acc/case-test & derived
  rule forms) corresponds to NO single reference invocation — the reference's own
  `--parse-only` is fully lazy (expands nothing, incl. ordinary lemmas' guarded
  forms [Q2], which I also confirmed: `A( m(x) )` stands un-expanded in the M
  guarded form). BEHAVIOR.md/REPORT.md state this plainly and frame Staged as the
  consumer's synthetic staging contract. Each behavioral PIECE traces to a probe
  (ordinary-item expansion = round-4 byte-parity; (a) = Q41; (b) = Q42), so the
  assembly is an interface requirement, not smuggled upstream structure —
  acceptable for a unit whose deliverable is the consumer's API.
- The round-4 stale-comment nit (`expand_item` claiming `Macros` items are
  "filtered out before this function") is FIXED this round; the comment now
  describes current pass-through-in-place behavior [Q37].

Verdict: pass.

## Round 6 audit

Reviewer: similarity auditor (both-sides). Scope: THIS round's delta only, from
clean-room HEAD `75807c0`. Restricted `git diff` under `macros/`:
`workspace/macro-clean/src/{lib.rs,tests.rs}`, `BEHAVIOR.md`, `QUERIES.log`,
`REPORT.md`, plus untracked `probes/bare_{indexed,indexed_zero,indexed_sorted,
indexed_baseline,typed,formula_plain,indexed_formula,formula_indexed_baseline}.spthy`
and their `captures/*.out`. Compared against upstream Haskell:
`lib/term/src/Term/Macro.hs` (`applyMacros`/`applyMacro`/`macroToFunSym`/
`findMatchingMacro`), the parser's nullary/signature machinery —
`Theory/Text/Parser/Term.hs` (`nullaryApp`, l.143–148) and
`Theory/Text/Parser/Token.hs` (`addMacrosToSignature`/`addMacroSym`, l.198–204) —
and, for the decoration behavior, the reference's variable/index/sort grammar
that `nullaryApp` does NOT participate in.

Delta content: one guard-tightening. The `expand_term` `Var` arm previously
resolved a bare nullary-macro name on the sort decoration alone
(`sort == Untagged`); it now requires the fully-undecorated name
(`formals.is_empty() && sort == Untagged && idx == 0 && typ.is_none()`), so an
indexed (`konst.1`) or type-annotated (`konst:msg`) `Var` stays an ordinary
variable [Q43,Q44]. Supporting docs (BEHAVIOR §2.5 decoration matrix, REPORT
round-6 entry, QUERIES [Q43,Q44]); 4 new tests (+2 helpers `ivar`/`tvar`).

**Findings: none (0).** No copied protectable expression in the delta.

Abstraction-filtration-comparison of the delta:

- The tightened guard is a genuinely DIFFERENT mechanism, not a mirror, and the
  divergence is structural. The reference never represents a "decorated Var that
  might be a macro": macro names are injected into the Maude signature as arity-0
  funsyms (`addMacrosToSignature`/`macroToFunSym`), and `nullaryApp`
  (Term.hs:143–148) matches a *bare token* against `funSyms ∪ macroNames` and
  emits `fApp fs []` (an **App**). A trailing `.` or `:` is simply not consumed by
  `nullaryApp` — it is left for the surrounding grammar to reject, whence the
  observed parse errors. So decoration-blocking is an emergent property of the
  reference grammar, resolved at PARSE, with no index/type/sort predicate anywhere
  near the macro path. The clean unit has no signature and no parser; it resolves
  at expansion time on a `Term::Var`'s own AST fields. The 4-way conjunction is
  dictated by (i) the vendored clean AST's four decoration-bearing fields
  (`VarSpec{name,idx,sort,typ}`, ast.rs:351–355, a round-1 structure) and (ii) the
  boundary-observed fact that each decoration blocks resolution — merger, the only
  faithful predicate given the observed behavior and the clean unit's own AST.
- `idx == 0` is not a smuggled magic constant. It is the AST encoding of "no
  explicit index"; the surface `.0` rejection is itself boundary-observable
  (`captures/bare_indexed_zero.out`: parse error "unexpected '.'"), and the note
  that `konst.0` is indistinguishable from plain `konst` in the AST (both idx 0)
  is the clean unit reasoning about ITS OWN representation, not the reference's.
- Identifiers/comments/constants: the delta reproduces no upstream source name —
  `applyMacros`, `applyMacro`, `macroToFunSym`, `findMatchingMacro`, `nullaryApp`,
  `addMacrosToSignature`, `addMacroSym`, `macroNames`, `NoEq`, `Private`,
  `Destructor`, `BNMacro`, `lnMacrosToBNMacros` are all absent (grepped the diff:
  zero hits). The tamarin surface tokens the new comments quote — `konst`,
  `~konst`/`$konst`, `konst.1`, `konst:msg` — all appear verbatim in the probe
  `.spthy` inputs and the captured parse-error `.out` output, i.e.
  boundary-observable scenes-a-faire, not lifted internal expression. Upstream's
  own comments (Macro.hs "Apply and substitute all the macros"; Term.hs's
  "FIXME: This try should not be necessary") are NOT echoed. No non-boundary
  magic constant introduced.

Behavioral-claim ↔ probe cross-check (all trace to logged probes/captures, which
I read this audit and which corroborate every sub-claim):
Q43 (rule-term matrix) → `probes/bare_indexed{,_zero,_sorted,_baseline}.spthy` +
`probes/bare_typed.spthy` with matching captures — `konst.1`/`.2`/`.0` parse-error
"unexpected '.'"; `~konst.1`/`$konst.1` parse OK, stay variables (unbound + sort
warnings); `notmac.1` parse OK, ordinary indexed var; `konst:msg` parse-error
"unexpected ':'". Q44 (formula matrix) → `probes/bare_formula_plain.spthy` (plain
`konst` resolves, guarded form `A( h('k') )`), `probes/bare_indexed_formula.spthy`
(`konst.1` parse-error), `probes/bare_formula_indexed_baseline.spthy` (`notmac.1`
parse OK, formula-terms warning). `#[test]` count in tests.rs = 30, matching the
REPORT's claim (26 prior + 4 new); +2 test helpers.

Non-blocking observations (not similarity violations; no redo):
- The round-4 `bare_typed`/[Q32] parse-error sub-claim, previously logged without a
  saved capture, now has one (`captures/bare_typed.out`) — evidence trail for the
  type-annotation row is closed this round.
- The formula-position `konst.1` rejection surfaces as a different parser message
  ("unexpected '(' expecting letter or digit, '.' or '='",
  `captures/bare_indexed_formula.out`) than the rule-term case ("unexpected '.'").
  QUERIES [Q44] states "PARSE ERROR (formula parser rejects the indexed macro
  name)" without quoting a message, so the claim is accurate; the mechanism (the
  formula grammar consumes the `.1` as a node-index continuation and then chokes on
  the `(`) differs from the rule-term path but reaches the same "not a macro use"
  observable. Does not affect the verdict.

Verdict: pass.
