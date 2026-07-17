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
```
