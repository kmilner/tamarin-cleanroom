# Clean-room cluster: theory PRETTY-PRINTER

Sealed reimplementation of the tamarin-prover **theory echo** — the
`theory <name> begin … end` text rendered at theory-load. This is the residual
GPL blocker for erasing the pseudonymous web authors (Kanakanajm/"Jackie",
YannColomb, Esslingen-Security-Privacy/Schoop) plus ~10 named individuals whose
only remaining hold is `pretty_theory.rs` / `pretty_formula.rs` / the term
printer. See ../PROTOCOL.md for the room rules.

## Layout
- `SPEC.md` — observable boundary, R1–R4 sub-target decomposition + ranking,
  the pure-render vs. solver-entangled split, method requirements.
- `oracle/pretty_oracle.sh` — black-box oracle: HS binary, **no `--prove`**,
  prints the extracted theory echo (the exact render target). `RAW=1` for full
  output. `oracle/examples/` for observation inputs.
- `interface/` — `ast_types.rs` (expression-stripped render-input type surface)
  and `required_api.md` (the crate's public entry points, one per sub-target).
- `workspace/pretty-clean/` — the clean Rust crate (std-only; scaffold with
  per-sub-target module stubs). `workspace/{QUERIES.log,BEHAVIOR.md}` are the
  required probe log + growing behavioral spec (create on first probe).
- `round1/` — first-round byte targets (sub-target R1 = term core + signature):
  `families.tsv`, `fetch_hs_targets.sh` → `targets/*.hs.txt`, plus
  `exclude_auto_sources.txt` / `corpus_minus_autosources.txt` (the 16-file
  `--auto-sources` family is a closure-preprocessing gap, not a render bug —
  exclude while iterating).

## Primary + acceptance gate
`/home/kamilner/tamarin-rs/scripts/pretty_gate.sh` (RS_PATH → your build,
ALLOWLIST → a subset). No-prove, full 419-file corpus, ~72 s warm at JOBS=6.
Current ported-tree baseline: **403 MATCH / 16 DIFF (all auto-sources) / 0
SKIP**. Acceptance = full-corpus green (not a fixture subset).

## First move
1. `cargo test` in `workspace/pretty-clean/` (baseline: scaffold green, R1
   tests `#[ignore]`d).
2. `round1/fetch_hs_targets.sh` to materialize R1 reference blocks.
3. Implement R1 (term + signature), gating with
   `ALLOWLIST=round1/target_files.txt scripts/pretty_gate.sh`.
4. Reuse the graphdot BSD Doc engine for layout (SPEC.md "Layout engine").
