# Clean-room task: theory PRETTY-PRINTER

Build a standalone Rust crate `pretty-clean` in `workspace/` that reimplements,
from BLACK-BOX BEHAVIOR ONLY, the tamarin-prover binary's **theory echo** — the
`theory <name> begin … end` text it prints at theory-load. Given a
parsed/closed theory (data model in `interface/ast_types.rs`), produce
byte-identical render text and the API in `interface/required_api.md`.

## The observable boundary

`oracle/pretty_oracle.sh <file.spthy>` runs the Haskell binary **WITHOUT
`--prove`** and prints the EXTRACTED THEORY ECHO — exactly the bytes you must
reproduce. Running without `--prove` is deliberate and load-bearing:

* the echo prints at theory-load (~0.25 s/file), so the gate is fast; and
* **no proof search runs**, so every lemma renders `by sorry` and NO proof
  tree is emitted. The proof/constraint-system renderer is solver output, not
  pretty-printer surface, and is explicitly OUT OF SCOPE (see "Stays ported").

The extracted block is `theory <name> begin … end` MINUS two trailing
formal-comment blocks the tool appends inside that span and everything after
`end`:

* the wellformedness report (`/* All wellformedness checks were successful. */`
  or the multi-line `/*\nWARNING …\n*/`) — a **separate slice** owned by the
  wellformedness cluster; do not reimplement it here;
* the volatile `/*\nGenerated from: …\n*/` build stamp (version / git rev /
  compile time); and
* the `====… summary of summaries …` after `end` (carries processing time).

Interior comments — per-rule `variants (modulo AC)` blocks, `has exactly the
trivial AC variant`, and `guarded formula characterizing all counter-examples`
— are KEPT: they are your output. The extraction awk is embedded in the oracle
and in the acceptance gate identically, so
`diff <(pretty_oracle.sh f) <(pretty-clean-render f)` == the gate verdict.

## Acceptance gate (full corpus, primary oracle AND integration check)

`/home/kamilner/tamarin-rs/scripts/pretty_gate.sh` runs the binary at `RS_PATH`
(point it at your build) no-prove over the whole 419-file corpus, extracts the
theory echo, and diffs it against a no-prove Haskell reference cache
(`scripts/.hs_pretty_cache`, auto-filled on first run). It reuses the batch
gate's `ckey`/`flags`/`strip_env` machinery so per-file canonical flags stay
identical. Output TSV: `relpath  MATCH|DIFF|SKIP_*  diffcount`. Warm-cache full
corpus ≈ 72 s at JOBS=6. Run it every iteration; `ALLOWLIST` subsets it while
you work a single sub-target. **Acceptance is the FULL-corpus gate green, not a
fixture subset** (the wf-regression lesson).

Measured baseline of the CURRENT ported tree against this gate: **403 MATCH /
16 DIFF / 0 SKIP**. All 16 DIFFs are the single `features/auto-sources/spore/*`
family and are NOT pretty-printer divergences (see "Stays ported"). The
pure-render surface is at 100% today, so your target is: keep every one of the
403 green while the ported renderer is deleted underneath you.

## Sub-target decomposition (each independently gate-checkable via ALLOWLIST)

Ranked by (individual-author yield ÷ isolation) — a good sealed round takes ONE
cleanly-observable sub-component at a time. Corpus counts are over the 419-file
parity corpus.

| # | sub-target | exercised by | ported LOC | isolation | authors it dents |
|---|------------|-------------|-----------|-----------|------------------|
| **R1** | **term core + signature block** — `Term`→text (operators `^ * ⊕ ++ %+`, pairs `<..>`, `exp`, `diff`, sorts `~ $ # %`, constants `'x'`, `.idx`) AND the `builtins:`/`functions:`/`equations:` block | builtins 319, functions 229, equations 137; `^`251 `*`(mult) `⊕`38 `++`/mset 89 nat 2 | term `crates/tamarin-term/src/pretty.rs` ~410; sig fns in `pretty_theory.rs` (`render_signature`/`render_fun_syms`/`render_equations`) ~130 | HIGHEST — term is a pure leaf (`Term`→`String`, no solver), the foundation every other sub-target reuses; signature is the first block, minimal wrapping | term.rs: meiersi, beschmi, jdreier, PhilipLukertWork, rsasse, charlie-j, BTom-GH, rkunnema |
| **R2** | **rule rendering** — facts + premises/actions/conclusions `[..] --[..]-> [..]`, rule attributes, `variants (modulo AC)` text (substitutions supplied), loop-breakers | rules 332 | `render_rule`/`render_rule_body`/`render_ac_variants_block`/`rule_attributes_doc` in `pretty_theory.rs` ~600 | MODERATE — reuses R1 term/fact; variant SUBSTITUTIONS come pre-computed from the solver, you render them | bulk of `pretty_theory.rs` (25-author header) |
| **R3** | **restriction / lemma FORMULA rendering** — `∀ ∃ ⇒ ∧ ∨ ¬ ⊤ ⊥ @ < = ⊏ last(..)`, quantifier binder allocation, temporal vars `#i`, the guarded-formula comment | lemmas 384, restrictions 179, exists-trace 296 | `pretty_formula.rs` ~2800 (formula + guarded); lemma/restriction wrappers in `pretty_theory.rs` ~300 | MODERATE — reuses R1 for atom term args; formula layout is its own file | `pretty_formula.rs`: meiersi, beschmi, jdreier, PhilipLukertWork, rkunnema, rsasse, Hong-Thai, BTom-GH, charlie-j, arcz |
| **R4** | **macros / predicates blocks** | macros 9, predicates 12 | `render_parsed_macros`/`render_predicate` ~120 | VERY HIGH — tiny, self-contained | mop-up; no unique authors |
| — | **proof / heuristic / constraint-system annotations** | (only under `--prove` / web) | `pretty_system.rs` 702, proof fns in `pretty_theory.rs`, `pretty_hpj` proof paths | — | **STAYS PORTED** (solver-entangled — see below) |

Recommended order: **R1 → R2 → R3 → R4**. R1 unblocks R2 and R3 (both consume
term rendering). Do R1 first: it is the deepest, most-reused, most-isolated
core, and it is what makes `pretty_theory.rs` (the 25-author file carrying the
pseudonymous web authors) deletable once R2/R3 also land.

## Layout engine (NOT part of the erasure surface)

Every renderer builds a HughesPJ `Doc` and renders it at width 110 / ribbon 73.
That layout algebra is BSD-licensed (Haskell `pretty-1.1.3.6`) and has ALREADY
been clean-roomed by the graphdot cluster
(`../graphdot/workspace/graph-clean/src/pretty.rs`, from
`../graphdot/sanctioned/pretty-1.1.3.6`). REUSE it — do not re-derive the
fitting logic from the GPL side. Your job is the tamarin-specific part: WHICH
combinators, in what nesting, with what literal strings and glyphs.

## Solver-entangled inputs — pure render vs. "stays ported"

The renderer is a `Theory → text` function ONLY for the surface below. Some
displayed data is COMPUTED by the ported closure/solver; the clean crate
RENDERS it but is handed it pre-computed, and the computation stays ported.

CLEANLY REIMPLEMENTABLE (pure render — R1–R4):
* term rendering; signature block; facts; rules; macros/predicates;
  formula & restriction & lemma-statement rendering; the guarded-formula
  comment TEXT (given the guarded formula); the `variants (modulo AC)` block
  TEXT (given the substitutions).

STAYS PORTED (solver/closure output, NOT pure `Theory→text`):
* **Proof trees / proof methods / constraint-system pane** (`pretty_system.rs`,
  the `pp_proof`/`solve_goal_to_doc`/`pp_contradiction` paths in
  `pretty_theory.rs`). Only rendered under `--prove` or in the web UI, so they
  are OUTSIDE the no-prove echo the gate observes. Leave them ported.
* **The DATA feeding R2/R3's comments**: the AC-variant substitutions come from
  Maude (`tools/rule_variants`), the guarded-formula negation from the guarded
  transform, the closed signature (merged/sorted symbols) from theory closure.
  The clean crate consumes these; the transforms stay ported.
* **`--auto-sources` injection** (the 16 gate DIFFs): with `--auto-sources` the
  closer injects `AUTO_IN_TERM_*`/`AUTO_OUT_TERM_*` action facts and an
  `AUTO_typing [sources]` lemma into the CLOSED theory. The RS no-prove path
  does not perform this injection, so its echo lacks them vs the HS reference —
  a **closure/preprocessing** gap, upstream of the renderer, not a pretty-print
  bug. Exclude this family via ALLOWLIST while iterating (see
  `round1/exclude_auto_sources.txt`); it is not the pretty slice's to fix.

## Method requirements (per PROTOCOL.md)

* Derive every glyph, operator precedence, wrap column, spacing and ordering by
  experimenting against `oracle/pretty_oracle.sh`; take exact output strings
  from observed oracle output (compatibility content, never memory).
* Log every oracle query's purpose in `workspace/QUERIES.log` (one line each).
* Maintain `workspace/BEHAVIOR.md`: the growing behavioral spec you infer
  (operator → glyph, precedence/parenthesization, wrap width, section order).
  A deliverable equal in weight to the code.
* Tests: per sub-target, a fixture (AST value as Rust constructors) + expected
  render snippet, asserted against your impl AND spot-checked against the
  oracle. Integration truth is the full-corpus gate.
* Dependencies: std only (plus the reused BSD Doc engine). You may NOT parse
  `.spthy` yourself; your input is the `ast_types.rs` model.
