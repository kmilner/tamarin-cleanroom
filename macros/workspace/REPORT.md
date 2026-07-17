# Unit E — `.spthy` macro expansion — clean-room report

Clean-room reimplementation of tamarin-rs macro expansion, derived solely from
black-box oracle observation (`oracle/hs_oracle.sh`), the provided example
`.spthy` inputs, and the interface AST (`../wellformedness/interface/ast_types.rs`).
No tamarin-rs source or tamarin-prover internals were read; every behavioural
claim traces to a logged oracle interaction (`QUERIES.log`, tags `[Qn]`).

## Deliverables (in `workspace/`)
- `macro-clean/` — Rust crate. `pub fn expand(theory: &Theory) -> Theory`
  (`src/lib.rs`), AST surface (`src/ast.rs`), tests (`src/tests.rs`).
- `BEHAVIOR.md` — the characterised semantics (definition syntax, call-site
  expansion, name capture/collision, edge cases, visibility).
- `QUERIES.log` — 31 oracle interactions with purpose + observed result.
- `byteparity.sh` / `run_tests.sh` — acceptance harness.
- `fixtures/` — macro theories paired with hand-inlined equivalents.
- `captures/` — raw oracle outputs kept as observation evidence.

## What `expand` does
Replaces every macro call at every use site with its transitively-substituted
body and removes the `macros:` declarations, yielding an equivalent macro-free
theory. It performs *only* macro substitution — let-inlining, diff-projection,
AC-variant computation and Sapic translation are separate passes and are left
intact with their inner macro calls expanded.

## Characterised semantics (see BEHAVIOR.md for the full table)
- **Definition** `macros: name(f1..fk) = body, ...`; formals untyped; duplicate
  formal names rejected [Q22]; a body may reference builtins/functions and
  **strictly-earlier macros only** — forward/self/mutual/undeclared references
  are parse errors [Q8,Q19,Q20,Q24]. Macros therefore form a DAG and expansion
  always terminates. Arity 0 allowed; a body may contain free (non-formal)
  variables that are captured by the surrounding scope [Q10].
- **Name collisions** rejected at definition time: duplicate macro, or macro
  name equal to an actively-declared function/builtin -> `Conflicting name for
  macro <n>` [Q13,Q14,Q21]; no conflict when the builtin is absent [Q25].
- **Call site** `name(a1..ak)`: bind `fi := ai` and substitute
  **simultaneously** (capture-avoiding) [Q7]; a formal matches a body variable
  by **full identity including sort** — `~x`/`$x` do not match an untagged
  formal `x` and remain free, and an argument whose formal never occurs is
  dropped [Q27,Q28]. Expansion is transitive [Q9,Q18].
- **Arity** is enforced by the parser: arity >= 2 requires exact argument count
  [Q12,Q16,Q17]; for arity 1 extra comma-args are packed into a pair before the
  AST [Q11,Q15]. Consequently at the AST level a call's arg-count already equals
  the macro arity; `expand` assumes this and no-ops defensively on a mismatch.
- **Visibility**: `--parse-only` performs no expansion [Q2]; on close the
  `macros:` block and the primary `(modulo E)` rule / lemma / restriction keep
  the *call*, while the expansion appears in the `(modulo AC)` variant,
  `expanded formula`, `guarded formula`, diff projections, and translated
  process rules [Q3,Q4,Q26,Q29]. Macros are not added to the function signature.

## Acceptance evidence
- **Direct expansion checks** — 14 Rust tests assert `expand()` produces the
  exact expected macro-free AST for: simultaneous substitution, nesting, pair
  macros, 3-level chains, AST-level pair-packing, sort-sensitive matching,
  nullary/free-var bodies, exp+pub literals (both `BinOp` and `AlgApp`),
  expansion inside `diff()` and Sapic processes, macro-drop, non-macro-App
  preservation, and the defensive arity-mismatch path. All pass.
- **Byte-parity fixtures** — for 6 pairs (`capture`, `chain3`, `over_u3`,
  `sortmatch`, `issue777`, `MacroInLemmasAndRestrictions`) the oracle output on
  the macro theory equals the oracle output on a hand-inlined equivalent once
  the three known cosmetic scaffolding categories are removed (preserved
  `macros:` block; primary-line call vs expanded term; explicit `(modulo AC)`
  block vs trivial-variant note). `byteparity.sh` confirms the remaining
  expanded content is byte-identical [Q31]. `run_tests.sh` runs the whole suite:
  ALL PASS.

## Scope boundaries / notes
- The xor example (`MacroWithRestrictionCRxor`) expands structurally to
  `h(x XOR y XOR z)` but the oracle then AC-abstracts it into a variant table
  (`h(z)`, `z=(...)`); it is therefore excluded from the strict byte-parity set
  and only used as a structural observation [Q30].
- Macros in `equations:` were not observed in the corpus; `expand` still rewrites
  equation terms uniformly (a no-op when none are present) for robustness.
- `src/ast.rs` vendors the provided interface type surface so the crate compiles
  standalone; at integration the dirty room replaces it with the real type
  re-export.

## Reproduce
```
bash workspace/run_tests.sh          # cargo test + all byte-parity fixtures
cd workspace/macro-clean && cargo test
bash workspace/byteparity.sh <macro.spthy> <hand-inlined.spthy>
bash workspace/formula_parity.sh <macro.spthy> <hand-inlined.spthy>  # lemma/acc/case-test
```

---

## Round 4 — three expansion-semantics gaps + declaration preservation

Closed three gaps and reconciled the declaration-preservation interop contract.
Every claim traces to a round-4 oracle probe ([Q32]–[Q40], `probes/`, `captures/`).

**GAP 1 — bare nullary macro uses.** A 0-ary macro used as a plain name (no
parentheses) is a use of that macro, treated identically to the parenthesised
form: `konst()=h('k')`, bare `konst` → `h('k')` [Q32]. Characterised the exact
resolution conditions: it fires in every term position and transitively through
bodies [Q32,Q33]; **untagged sort only** (`~konst`/`$konst` stay ordinary vars,
and a macro-name use can't be sort-annotated) [Q34,Q32]; **nullary only** (a bare
name for an arity ≥ 1 macro stays an ordinary variable) [Q35]; and a nullary
macro **reserves its name against a same-named formal** [Q36]. Per the interface
contract such a use reaches `expand` as `Term::Var("konst")`; `expand_term` now
resolves an untagged `Var` whose name is a nullary macro to that macro's body —
and the existing "expand body first, then bind formals" order reproduces the
name-reservation result [Q36] by construction.

**GAP 2 — accountability-lemma formulas** and **GAP 3 — case-test formulas.**
The reference's macro stage DOES expand macros used in `... accounts for "..."`
acc-lemma formulas and `test <name>: "..."` case-test formulas. Stage behaviour:
`--parse-only` preserves the call; on close the acc lemma is translated into
generated lemmas (+`predicate:`) whose primary renderings keep the call and whose
`/* guarded formula ... */` blocks show the expansion — the exact
primary-keeps-call / guarded-shows-expansion pattern of ordinary lemmas [Q4].
Confirmed with identity and non-identity (`h(x)`) macros [Q38,Q39]. `expand`
already rewrites `AccLemma.formula` and `CaseTest.formula`; added parity fixtures
prove it.

**Declaration preservation.** Probed that the reference retains the `macros:`
block (with original, un-expanded bodies) in its pretty output after processing
[Q37]. Per the interop contract, `expand` now **preserves the `Macros` items in
place** (previously it dropped them); only use sites are expanded.

**New harness — `formula_parity.sh`.** `byteparity.sh`'s env-stripping approach
cannot exercise accountability theories (its reduced env lacks a UTF-8 locale, so
the reference aborts on ∀/∃ glyphs before generating lemmas). `formula_parity.sh`
runs the oracle with a UTF-8 locale and compares only the guarded/expanded-formula
blocks — the harness of record for the lemma/acc-lemma/case-test gaps.

**Acceptance (round 4).** `run_tests.sh`: **21** cargo direct-expansion tests
(14 prior — one renamed `drops_→preserves_macro_declarations` and one fixture
updated for preservation — plus 7 new: 5 bare-nullary + acc-lemma + case-test),
7 byteparity rule fixtures (incl. `bare_nullary`), and 4 formula_parity fixtures
(`lemmas`, `casetest`, `acclemma`, `acc_both`). **ALL PASS** [Q40].

Fixtures added: `fixtures/{bare_nullary,casetest,acclemma,acc_both}_{macro,expanded}.spthy`;
captures in `captures/{bare_nullary_*,acc_both_*,bare_sorts,bare_nonnullary,formal_vs_nullary}.out`.
