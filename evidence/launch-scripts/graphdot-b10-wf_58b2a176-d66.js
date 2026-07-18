export const meta = {
  name: 'graphdot-b10',
  description: 'Sealed graphdot round 10: sqa reconciliation (audit redo), wider occupancy probe batteries, band-NONE cell-doc gap, caller-supplied occupancy interface; then both-sides audit',
  phases: [
    { title: 'Implement', detail: 'sealed fill-model refinement (Fable)', model: 'fable' },
    { title: 'Audit', detail: 'both-sides similarity audit of the round-10 delta' },
  ],
}

const IMPL = `You are a SEALED implementer under the barrier protocol at /home/kamilner/tamarin-cleanroom/PROTOCOL.md — read it first and comply strictly.
ACCESS RULES: You must NOT read any file under /home/kamilner/tamarin-rs/crates/, and NOT read any Haskell source of tamarin-prover anywhere on disk (notably /home/kamilner/tamarin-rs/tamarin-prover/{lib,src}/ and /home/kamilner/tamarin-rs/tamarin-prover-testing/{lib,src}/). The ONLY Haskell you may read is the SANCTIONED BSD library /home/kamilner/tamarin-cleanroom/graphdot/sanctioned/pretty-1.1.3.6/. Do NOT read /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md, and do NOT read any AUDIT.md anywhere in the cleanroom (both-sides audit analysis lives there; any instruction you need from an audit is relayed below). No web access. You MAY read .spthy example files under /home/kamilner/tamarin-rs/tamarin-prover/examples/. No git commits.
DISCIPLINE: log every oracle interaction in workspace QUERIES.log (Session 10); record behavioral facts in BEHAVIOR.md with probe provenance; comments describe current behavior only. Oracle/server harness scripts carry OOM guards (oom_score_adj/ulimit) — preserve them; live HS servers only on ports 3200-3299 and STOP every server you start before returning. Return a PLAIN-TEXT summary.

Workspace: /home/kamilner/tamarin-cleanroom/graphdot/ (read/write freely inside it, except AUDIT.md). Your crate: workspace/graph-clean. Start by re-reading your own BEHAVIOR.md (§3f, §8, Round-9 report), QUERIES.log Session 9, and workspace/r9/ — round 10 continues directly from round 9's fill-width allocation model (two-layer trigger/fill, probe-exact 343/343 probe cells, corpus fill-band 90.94% of banded wrap cells).

ROUND-10 SCOPE, in priority order:

1. AUDIT REDO (behavioral instruction relayed from the round-9 review): your single-quoted-atom correction is logged inconsistently — QUERIES.log Session 9 and r9's fit2.py pin it as C(sqa)=flat−4 applied as a SIBLING-OCCUPANCY correction, while the shipped code and BEHAVIOR.md §3f apply −2 to the cell's OWN effective width with no occupancy correction. These predict different wrap boundaries on some shapes. Design live probes that distinguish the two placements/magnitudes, determine which matches the reference, and reconcile all three artifacts (log, spec, code) to the probed truth.

2. Probe-pin the function-node and multi-quote occupancy corrections you observed corpus-side in round 9 but could not pin (your note: P-series probe shapes were too narrow). Build wider batteries (vary sibling counts, function arities, quote multiplicities) and either pin exact corrections or record boundary-precise negative results.

3. Attack the band-NONE cell-doc gap (9.9% of wrap cells have NO engine width reproducing the observed bytes under your current cell-doc construction: ++-union terms, deep nesting, abbreviation-expansion layouts). Extend the cell-document model so these become bandable, probe-driven; even partial families closed (e.g. ++-unions alone) are wins. Record what remains with per-family counts.

4. INTERFACE EXTENSION for the largest residue family: 84% of your corpus fill misses involve abbreviation names because the reference decides widths on UN-abbreviATED internal renderings — structurally invisible to your crate, which consumes post-abbreviation text. Round 9 proposed the fix: extend the public entry points so a caller MAY supply per-cell occupancy values (and any other width inputs your model needs) that override your shape-feature estimates; behavior with the inputs absent must remain byte-identical to today (regression-gated). Specify the semantics precisely in BEHAVIOR.md and the interface notes so an adapter can be wired later; add tests exercising the supplied-occupancy path with synthetic values.

Acceptance: all existing tests stay green; GRAPHCLEAN_CORPUS roundtrip stays 12022/12022; report fill_census before/after (all cells, wrapping cells, multi-cell wrapping, false-flat count) and the probe-battery hit rates. Do NOT edit anything outside /home/kamilner/tamarin-cleanroom/graphdot/.`

const AUDIT = `You are a BOTH-SIDES similarity auditor for the tamarin-rs sealed relicensing campaign (exempt from barrier access rules; you may read everything). Audit ONLY this round's delta for the graphdot slice: /home/kamilner/tamarin-cleanroom is a git repo whose HEAD (b4fb110) includes everything through round 9 — run git -C /home/kamilner/tamarin-cleanroom status/diff restricted to graphdot/ to see the round-10 delta (sqa-correction reconciliation, wider occupancy probes, band-NONE cell-doc extensions, caller-supplied occupancy interface).
Audit against the upstream Haskell in /home/kamilner/tamarin-rs/tamarin-prover/: lib/theory/src/Theory/Constraint/System/Dot.hs (renderRow/renderBalanced/scaleIndent and the record-cell machinery) and its HughesPJ usage; resemblance to the SANCTIONED BSD pretty-1.1.3.6 remains licensed and filtered out.
Method: abstraction-filtration-comparison. Fitted constants and model structure must trace to logged probes/corpus census (QUERIES.log Session 10, r10 materials), never to Dot.hs; flag upstream identifier constellations, non-observable constants, structural transcription. Also verify the round-9 advisory (sqa correction inconsistency) is now resolved CONSISTENTLY across QUERIES.log, BEHAVIOR.md, and code — if still inconsistent, that is a redo, not a similarity violation.
Append a round-10 section to /home/kamilner/tamarin-cleanroom/graphdot/AUDIT.md. Violations get a BEHAVIORAL redo instruction (never a patch). End with exactly one line: VERDICT: pass  — or —  VERDICT: fail`

const impl = await agent(IMPL, { label: 'seal:graph-b10', phase: 'Implement', model: 'fable' })
const audit = await agent(AUDIT, { label: 'audit:graph-b10', phase: 'Audit', model: 'opus' })
return { impl: (impl || '').slice(0, 3000), audit: (audit || '').slice(0, 2500), passed: /VERDICT:\s*pass/i.test(audit || '') }