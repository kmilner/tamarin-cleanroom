export const meta = {
  name: 'graph-b11-integration',
  description: 'Open-side graph round-11 integration measurement: re-sync, GRAPHCLEAN_CORPUS record-level numbers, adoption decision, residual re-ranking for B12',
  phases: [{ title: 'Integrate', detail: 're-sync + corpus measurement + adoption-or-rank (Opus)' }],
}

const GATE = `You are the OPEN-SIDE integrator for the tamarin-rs sealed relicensing campaign, working in /home/kamilner/tamarin-rs. Read first: the graphdot sections of /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md (especially the round-10 occupancy-adapter report), /home/kamilner/tamarin-cleanroom/graphdot/workspace/INTERFACE.md (REWRITTEN in round 11 — display-text estimates ARE the probed reference behavior; new trigger_width self-width override), and PROTOCOL.md ("Full-corpus gates").
CONTEXT: sealed round 11 landed five new allocation laws + a relief second pass + the tuple-opener hang; sealed census now: all cells 98.794%, wrapping cells 95.60%, single-cell wrapping 100%, band-NONE rows 6612→854. The round-10 belief that occupancy overrides help was refuted — do NOT wire internal-width occupancies; the display-text default path is the reference behavior.
RULES: mechanical re-sync only (documented recipe, crate::->super::, headerless, reverse-transform byte-identity). No git commits. PATH needs /home/linuxbrew/.linuxbrew/bin. Live HS servers only on ports 3200-3299; stop everything you start. Preserve OOM guards.
STEPS:
1. Re-sync the round-11 graph-clean sources from /home/kamilner/tamarin-cleanroom/graphdot/workspace/graph-clean/src into crates/tamarin-server/src/graph_clean/ (same recipe as the round-10 re-sync section documents). Build + full tamarin-server test suite.
2. Measure on the 12,022-payload GRAPHCLEAN_CORPUS gate (default path, NO width overrides): report (a) cell-level byte-exactness, (b) RECORD-level all-cells-exact, and (c) whole-PAYLOAD byte-exactness counts — before (round-10 numbers are in the round-10 report) vs after.
3. Characterize every remaining whole-payload divergence into ranked families with counts and a first-diverging-byte witness each (these are the B12 targets). Distinguish fill/wrap families from any NON-fill families the census cannot see (id/port allocation, clustering, legend, edge ordering, header/dialect).
4. ADOPTION decision: if whole-payload byte-exactness reaches 100% on any precisely-characterizable subset that covers the live web routes' actual usage, attempt routing the web DOT serialization through the clean module and gate (build, suites, live spot probes on 3 theories ports 3200-3299, scripts/web_parity.sh, gen_license_headers.py --check). Otherwise keep-and-report — do NOT partially adopt with known byte regressions.
5. Append a dated graph round-11-integration section to INTEGRATION_REPORT.md. Return a plain-text summary: the three measurement numbers before/after, the ranked residual families, and the adoption outcome.`

const gate = await agent(GATE, { label: 'integrate:graph-b11', phase: 'Integrate', model: 'opus' })
return { gate: (gate || '').slice(0, 4000) }