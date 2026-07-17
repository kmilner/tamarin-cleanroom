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

---

## Round 5 — the consumer's STAGED-MODE contract (`expand_staged`)

Added a second entry point so `expand` is adoptable by the consumer's pipeline,
which invokes expansion at a stage *earlier* than full close. Design: a private
`Mode` enum (`FullClose` / `Staged`) threaded through `expand_item` and
`expand_rule` only (the two functions that branch); the whole term / formula /
process / bare-nullary traversal is shared, so the two modes cannot drift.

**Two entry points.**
- `expand(theory)` — full close. **Unchanged.** Every existing test and every
  byte-parity / formula-parity fixture stays green (they observe the oracle on
  the source theories, and the round-4 direct-expansion tests still call `expand`
  and assert full expansion, incl. acc-lemma/case-test formulas and derived rule
  forms).
- `expand_staged(theory)` — the entry the consumer designates. Two carve-outs
  dictated by the consumer's staging, each reconciled with a fresh parse-stage
  probe:
  - **(a)** acc-lemma (`... accounts for "..."`) and case-test (`test <n>: "..."`)
    formulas are left byte-identical — a later consumer stage owns them. Reconciled
    by `--parse-only` on `acc_both_macro.spthy`: both calls preserved verbatim
    [Q41] (consistent with [Q2,Q38,Q39]).
  - **(b)** only the primary rule form is rewritten; derived `variants` /
    `left_right` fields pass through verbatim. Reconciled by `--parse-only` on
    `issue777.spthy` + `MacroInLemmasAndRestrictions.spthy`: each rule exists only
    as `rule (modulo E) <name>:` with no `(modulo AC)` variant / no diff
    projection present at parse stage [Q42].

  Everything else — ordinary lemmas / restrictions / predicates / processes /
  equations / the primary rule form / bare-nullary resolution — expands exactly
  as in full close; the `macros:` block is preserved in place [Q37].

**Distinction from the reference's `--parse-only`.** That rendering is fully lazy
(expands nothing, incl. ordinary lemmas' guarded forms [Q2]); the consumer's
staged contract is *not* that — it eagerly expands the ordinary items and carves
out only acc-lemma/case-test formulas [Q41] and derived rule forms [Q42].
`expand_staged` implements the consumer's contract, not the parse-only rendering.

**Also.** Fixed a stale comment in `expand_item` (it claimed `Macros` items are
filtered out before the function — false since round 4; they pass through in place
and are returned via `it.clone()`), flagged as a non-blocking nit in the round-4
audit. Comments now describe current behavior only.

**Acceptance (round 5).** `run_tests.sh`: **26** cargo tests (21 prior + 5 new
staged-mode tests: acc/case-test untouched, `macros:` preserved, ordinary
lemma/restriction/rule expanded, bare-nullary still resolves, and derived
variant/left-right forms not recursed into — with a full-close contrast), 7
byteparity rule fixtures, 4 formula_parity fixtures. **ALL PASS** [Q41,Q42].
New probes logged: `QUERIES.log` [Q41,Q42].

---

## Round 6 — bare-nullary resolution was firing too broadly (indexed/typed names)

One precise gap: the `expand_term` `Var` arm resolved a nullary macro's bare name
on the **sort** decoration alone (`sort == Untagged`), ignoring the var node's
other decorations — the numeric **index** and the optional **type**. Round 4
[Q34] had covered sorts (`~konst`/`$konst` stay variables) but never an INDEXED
bare name.

**Probe — full decoration matrix** (`konst() = h('k')`), rule-term [Q43] and
formula [Q44] positions, both cross-checked against a NON-macro baseline:

| decoration        | surface              | reference behavior                         |
|-------------------|----------------------|--------------------------------------------|
| none (plain)      | `konst`              | resolves to `h('k')` (rule + guarded formula) |
| fresh / pub sort  | `~konst` / `$konst`  | stays a variable [Q34]                     |
| explicit index    | `konst.1/.2/.0`      | **parse error** — indexed macro name is not a use |
| sort + index      | `~konst.1` / `$konst.1` | stays a variable                        |
| type annotation   | `konst:msg`          | **parse error** (reconfirms [Q32])         |

The baselines pin that the decorations are legal on an *ordinary* variable
(`notmac.1` parses as a plain indexed variable in both positions [Q43,Q44]); the
parse errors are specific to a nullary-*macro* name. So the reference treats an
indexed/typed nullary-macro name as **never a macro use**. In the consumer's AST
(where a var node carries `(name, idx, sort, typ)` and macro resolution is
deferred to `expand`) such a decorated `Var` must therefore stay a variable — the
old sort-only guard resolved it, firing too broadly.

**Fix.** The `Var` arm now resolves only the fully-undecorated plain name:
`formals.is_empty() && sort == Untagged && idx == 0 && typ.is_none()`. The arm is
shared by `expand` (full close) and `expand_staged`, so the tightened guard holds
in **both modes**. `konst.0` is indistinguishable from plain `konst` in the AST
(both `idx==0`), so `idx==0` is the faithful resolving predicate — the reference
rejects the surface `.0` at parse regardless.

**Acceptance (round 6).** `run_tests.sh`: **30** cargo tests (26 prior + 4 new:
indexed-not-resolved with plain-still-resolves contrast; typed-not-resolved;
sort+index-not-resolved; and the index guard in a formula position under BOTH
`expand` and `expand_staged`), 7 byteparity rule fixtures, 4 formula_parity
fixtures. **ALL PASS**. Every prior test/fixture stays green (the plain-name
uses they exercise have `idx==0`/`typ==None`, so they still resolve). New probes
`QUERIES.log` [Q43,Q44]; captures in
`captures/{bare_indexed,bare_indexed_zero,bare_indexed_sorted,bare_indexed_baseline,bare_typed,bare_formula_plain,bare_indexed_formula,bare_formula_indexed_baseline}.out`.
