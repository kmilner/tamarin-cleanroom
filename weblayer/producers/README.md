# Clean-room cluster: web UI fragment PRODUCERS

Sealed reimplementation of the tamarin-prover interactive web UI's **fragment
producers** — the code that renders pre-computed prover values into the HTML/JSON
response-body CONTENT the (already clean-roomed) dispatch layer serves: the
proof-script west pane, the theory-view center fragments, the proof-tree HTML,
the index/overview rows.

This is the residual GPL blocker for retiring the pseudonymous web authors
(Kanakanajm/"Jackie", YannColomb, Esslingen-Security-Privacy/Schoop) plus the
named web team: their citations sit on the dispatch shells
(already clean-covered by `web_clean`) AND on these producers, so the producers
must be reimplemented before the shells can delete. See ../../PROTOCOL.md for the
room rules and SPEC.md "Author topology".

## Layout
- `SPEC.md` — observable boundary, the **semantic** (not byte) acceptance gate,
  the R1–R5 sub-target decomposition + ranking, the pure-render vs.
  solver-entangled split, author topology, method requirements, acceptance
  ladder.
- `oracle/hs_server.sh` — live black-box oracle: serve one `.spthy` in the HS
  interactive UI and curl any route. `start|stop|probe|smoke`; ports 3100-3199;
  OOM guards + readiness poll + port cleanup. `oracle/examples/` = input theories.
- `oracle/extract_fragments.py` — slice any fragment family out of the captured
  corpus (`list` / `families` / `extract <family[,…]> <outdir> [--only …]`);
  captures-only, no source tree.
- `oracle/captured_responses/` — the 81 captured crawl manifests (the sanctioned
  channel; captured OUTPUT).
- `interface/` — `fragment_inputs.rs` (the behavioral input-type surface — the
  shape of the pre-computed values the producers render) and `required_api.md`
  (one entry point per sub-target). BEHAVIOR only: no source citations, no
  upstream identifiers.
- `workspace/producers-clean/` — the clean Rust crate (std-only; per-sub-target
  module stubs). `workspace/{QUERIES.log,BEHAVIOR.md}` are the required probe log
  + growing behavioral spec.
- `round1/` — first-round byte targets (R1 = center section fragments):
  `families.tsv`, `fetch_targets.sh` → `targets/*.html` + `*.title`.

## Primary + acceptance gate
Iterate against the **capture-corpus sweep** (SPEC.md ladder rung 2): slice the
opaque sub-parts out of each captured fragment, feed them back through the
producer, assert the reassembly matches the capture — no prover needed.
Integration acceptance is `/home/kamilner/tamarin-rs/scripts/web_parity.sh`
(RS_PATH → your build, ALLOWLIST → a subset): full crawl, HS vs RS, **semantic**
diff. Acceptance = full-corpus green, not a fixture subset.

## First move
1. `cargo test` in `workspace/producers-clean/` (baseline: scaffold green,
   `model_constructs` passes, R1 tests `#[ignore]`d).
2. Read `round1/targets/*_message.html`, `*_rules.html`, `*_tactic.html`,
   `*_help.html` — the exact response bodies R1 reproduces (and `.title` = the
   envelope title). `oracle/extract_fragments.py families` to see the full map.
3. Implement R1 in `src/html.rs` (skin: escape + postprocess + envelope) then
   `src/section.rs` (`render_pane`): un-ignore the round-1 tests, confirming each
   expected string against the oracle (`oracle/hs_server.sh start
   oracle/examples/issue515.spthy 3131`).
4. Sweep the corpus (ladder rung 2) with `extract_fragments.py`; then hand off to
   the integrator for `web_parity.sh`.
