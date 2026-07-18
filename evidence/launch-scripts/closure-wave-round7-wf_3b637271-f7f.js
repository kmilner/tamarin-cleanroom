export const meta = {
  name: 'closure-wave-round7',
  description: 'Wave 4: B8 HughesPJ-based fill engine, A7 concurrency redesign, D6 verdict taxonomy; audits; then D driver swap, B serialization swap, A full adoption integrators',
  phases: [
    { title: 'Implement', detail: 'clean rounds B8, A7, D6 in parallel' },
    { title: 'Audit', detail: 'per-unit both-sides similarity audits (delta vs e76455a)' },
    { title: 'Integrate', detail: 'sequential: D driver swap, B serialization swap, A full adoption' },
  ],
}

const COMMON = `You are a CLEAN-ROOM implementer under the campaign protocol at /home/kamilner/tamarin-cleanroom/PROTOCOL.md — read it first and comply strictly.
ACCESS RULES: You must NOT read any file under /home/kamilner/tamarin-rs/crates/, and NOT read any Haskell source (*.hs) of tamarin-prover anywhere on disk (notably /home/kamilner/tamarin-rs/tamarin-prover/{lib,src}/ and /home/kamilner/tamarin-rs/tamarin-prover-testing/{lib,src}/). Do NOT read /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md. No web access. You MAY: run the reference binary/servers via your cluster's oracle scripts (export PATH="$PATH:/home/linuxbrew/.linuxbrew/bin" if invoking directly), read .spthy example files, and read/write freely inside your own cluster directory under /home/kamilner/tamarin-cleanroom/. Do not modify anything outside your cluster directory. No git commits.
DISCIPLINE: log every oracle interaction in your workspace QUERIES.log; record derived behavioral facts in BEHAVIOR.md with provenance; append a round section to a REPORT file (if .md creation is blocked, fold into BEHAVIOR.md). Code comments describe current behavior only. Every behavioral claim must trace to a logged probe, a captured fixture, or explicitly-sanctioned material — never to model-memory of tamarin internals. Return a PLAIN-TEXT summary (no structured output).`

const UNITS = {
  B: {
    key: 'B', cluster: 'graphdot',
    impl: `${COMMON}

UNIT B (graph generation), round 8 — replace the closed-form fill with a faithful layout engine. Live-server ports: 3200-3299 only.
Cluster: /home/kamilner/tamarin-cleanroom/graphdot/ — read workspace BEHAVIOR.md (Sessions 5-7: trigger rule, fill census, the 44% multi-line-fill ceiling), QUERIES.log, REPORTs. Crate: workspace/graph-clean.

NEW SANCTIONED MATERIAL: /home/kamilner/tamarin-cleanroom/graphdot/sanctioned/pretty-1.1.3.6/ contains the BSD-licensed Haskell pretty-printing library (Text.PrettyPrint.HughesPJ) that the reference's toolchain ships — read its README.md first. This library is NOT tamarin source; reading it and implementing a faithful Rust equivalent of its layout algorithms (Doc model, sep/fsep/fillSep semantics, best/fits fitting, ribbon handling, nesting) is PERMITTED. Everything tamarin-specific (which combinators are applied to record cells, with what widths/ribbons/indents per cell/row/group) is NOT in the library and must still come from black-box probes.

TASK: (1) Implement a faithful Doc/layout engine in your crate from the sanctioned source (port semantics precisely — fillSep's fitting behavior is the known weak spot of your closed-form model). (2) Reconstruct how record-cell content is fed through it: using your Session 5-7 probe data plus new targeted live probes, determine the per-cell/per-row layout parameters (line width, ribbon fraction if any, indent behavior, how a group's cells share the row) by bisection — now that the engine is exact, the parameter space is small. Key adequacy tests: the Wide-rule probe must come out byte-exact for ALL cells (your round-7 model got Big right but squeezed Ack to the floor where the reference keeps it on one line); your wrapping-cell census byte-exactness should rise from ~44% toward 100% — report the exact number and characterize any residue honestly. (3) Wire it through wrap/build_record/RawRule. The GRAPHCLEAN_CORPUS round-trip must stay 12022/12022. Stop all servers when done.`,
    auditHint: 'lib/theory/src/Theory/Constraint/System/Dot.hs (renderRow/renderBalanced/scaleIndent) — note the clean side is EXPECTED to resemble Text.PrettyPrint.HughesPJ (BSD, sanctioned); flag only tamarin-specific expression beyond the library and beyond probed parameters',
  },
  A: {
    key: 'A', cluster: 'weblayer',
    impl: `${COMMON}

UNIT A (web dispatch), round 7 — concurrency-safe dispatch design. Live-server ports: 3100-3199 only.
Cluster: /home/kamilner/tamarin-cleanroom/weblayer/ — read workspace BEHAVIOR.md, QUERIES.log, REPORTs. Crate: workspace/web-clean.

CONTEXT (interop): the consumer runs your dispatcher inside an asynchronous server where long-running proof operations (autoprove can take seconds-to-minutes) must NOT block unrelated requests. Your current Server::dispatch(&mut self) requires exclusive ownership for every request — behind one lock, a long autoprove would freeze the whole server. That adoption-blocking property is yours to fix by design.

STEP 1 — probe the reference's concurrency semantics live (this defines the behavioral contract): start a long autoprove (pick a slow theory/deep bound; your cluster recipes have candidates) and, while it runs, concurrently issue requests — GETs on OTHER theories, GETs on the SAME theory's other versions, a second proof op on the same theory, an upload, a reload. Record which are served immediately, which block, which interleave; verify version-counter behavior under concurrent allocation (two proof ops racing — do indexes ever collide or skip?); check whether a version becomes visible before its proof op completes.

STEP 2 — redesign dispatch to honor those semantics without exclusive long-held state: restructure so each request runs as get-snapshot -> compute -> commit: state reads take a snapshot/clone through StateOps; long computations (the ProverOps calls that can be slow) run on the snapshot WITHOUT any state borrow held; mutations commit as separate atomic StateOps calls afterwards (allocation of the result version happens at commit, matching the probed counter semantics). Make dispatch callable per-request (&self or split into cheap-route/slow-route paths — your design; document it). The probed concurrency semantics become part of the documented CONTRACT. Update the in-memory reference impl + all tests; add a test simulating an in-flight slow op with interleaved requests (using a controllable fake ProverOps) asserting the observable interleaving matches what you probed. All existing byte-parity tests stay green. Stop all servers when done.`,
    auditHint: 'src/Web/Handler.hs, src/Web/Types.hs (the threading/queue machinery), src/Web/Settings.hs',
  },
  D: {
    key: 'D', cluster: 'console',
    impl: `${COMMON}

UNIT D (console/CLI), round 6 — complete the lemma-verdict taxonomy in batch summaries.
Cluster: /home/kamilner/tamarin-cleanroom/console/ — read workspace BEHAVIOR.md, QUERIES.log, captures/. Oracle: console/oracle/hs_oracle.sh (split streams). Crate: workspace/cli-clean.

Your summary model distinguishes 3 lemma verdicts; the reference's per-lemma summary lines have MORE distinct forms. Probe systematically until you have exhausted the taxonomy — construct theories/invocations driving each lemma into every reachable terminal state, e.g.: proofs completed (verified/falsified, trace found/not, all-traces vs exists-trace), analysis stopped by --bound (incomplete/unfinished states), --stop-on-trace variants, lemmas that error (ill-formed within an otherwise-running theory, solver errors), diff-mode lemmas in each state, lemmas skipped/filtered (--prove pattern not matching), [sources] lemmas with/without partial deconstructions, mirrored/unmirrored diff outcomes. For each: capture the exact per-lemma summary line (wording, counts, stream, position) and any associated advisory lines. Extend your LemmaOutcome/summary rendering to cover the full probed taxonomy with byte-parity fixtures per verdict. Also re-verify the interleaving/order of per-lemma lines vs warning blocks in multi-lemma, multi-theory runs. All existing tests/fixtures stay green.`,
    auditHint: 'src/Main/Mode/Batch.hs and the proof-status rendering (grep showProofStatus in lib/theory/src/)',
  },
}

function auditPrompt(u) {
  return `You are a BOTH-SIDES similarity auditor for the tamarin-rs clean-room relicensing campaign (exempt from clean-room access rules). Audit ONLY this round's delta for unit ${u.key} (${u.cluster}).
The clean-room repo /home/kamilner/tamarin-cleanroom is a git repository whose HEAD (e76455a) predates this round: run git -C /home/kamilner/tamarin-cleanroom status and diff, restricted to ${u.cluster}/, to see exactly what changed. Audit against the upstream Haskell sources under /home/kamilner/tamarin-rs/tamarin-prover/ — start from ${u.auditHint}.
Method: abstraction-filtration-comparison; filter behavior-dictated/merger/scenes-a-faire, probe-derived content (cross-check BEHAVIOR.md/QUERIES.log), and content derived from explicitly SANCTIONED material (the graphdot cluster sanctions the BSD pretty-1.1.3.6 library — resemblance to IT is fine; resemblance to tamarin's own GPL expression beyond it is not). Flag copied protectable EXPRESSION: identifier constellations, structure beyond behavior, tamarin-internal names, non-boundary magic constants, comment lineage.
Append a round section to /home/kamilner/tamarin-cleanroom/${u.cluster}/AUDIT.md. Violations get BEHAVIORAL redo instructions (never patches). End with exactly one line: VERDICT: pass  — or —  VERDICT: fail`
}

const DIRTY_COMMON = `You are a DIRTY-ROOM integrator for the tamarin-rs relicensing campaign, working in /home/kamilner/tamarin-rs. Read /home/kamilner/tamarin-cleanroom/PROTOCOL.md and the relevant INTEGRATION_REPORT.md sections (waves 2-3 document each blocker this round aimed to close).
RULES: adapters and extraction only — NEVER transplant ported logic into clean (headerless) files; clean vendored sources get mechanical fixes only. A clean file acquiring a GPL header from scripts/gen_license_headers.py is a provenance violation: stop and report. If a gap still blocks, KEEP the ported path + report precisely. Comments describe current code only. No git commits. Append a dated section to INTEGRATION_REPORT.md; return a plain-text summary.
FINAL VALIDATION: cargo build --workspace; cargo test -p tamarin-parser -p tamarin-theory -p tamarin-prover -p tamarin-server; wellformedness fixture suite; scripts/gen_license_headers.py then --check (0 stale). Report the header-count delta and author citations that disappeared. If you must stop partway, leave the tree BUILDING AND GREEN and report the precise remaining steps.`

function integDPrompt(passed) {
  return `${DIRTY_COMMON}

Unit D — the run-driver swap (waves 2-3 scoped it; round-6 closed the verdict taxonomy${passed.includes('D') ? ', audit PASSED' : ' but its audit DID NOT PASS — in that case only re-sync nothing and report'}). Steps:
1. Re-sync the round-6 clean cli modules (verdict taxonomy in framing/emitter) into crates/tamarin-prover/src/cli/ (mechanical, headerless).
2. Make the run-driver contract changes in the PORTED files (allowed, they stay headered): defer value-validation forcing until after the maude preamble exactly as the reference does; route structural-error one-liners bare to stderr and validation envelopes (error + full help) to stdout via the clean errors module; recover the stop-on-trace absent/present distinction via an options re-parse; render batch summaries through the clean framing (now carrying the full verdict taxonomy) and stream via the clean incremental emitter.
3. Route argument tokenization/typing through the clean typed args (thin mapping into the existing ~40-field Args consumer struct is fine and may stay ported/headered as a struct definition; the goal is deleting the ported TOKENIZER/VALIDATOR bodies).
4. BUILD THE CORPUS GATE before deleting: a curated theory set exercising every verdict class the round-6 taxonomy pinned (use the console cluster's round-6 probe inputs), plus multi-theory/wf-warning/bounded-prove cases; run reference vs RS with split streams; require byte-equality (modulo build metadata). Plus cli_e2e + console_split_parity + live spot-checks.
5. When green, DELETE the ported parse_args/validation bodies and the ported summary/framing assembly in run.rs. The three Rust-only flags keep working through clean typed args. Keep-and-report anything still blocked.`
}

function integBPrompt(passed) {
  return `${DIRTY_COMMON}

Unit B — the graph serialization swap (blocked in waves 2-3 on the fill engine; round-8 replaced the closed-form fill with a HughesPJ-faithful layout engine${passed.includes('B') ? ', audit PASSED' : ' but its audit DID NOT PASS — in that case only re-sync and report'}). A D integrator ran before you; rebase on the CURRENT tree.
1. Re-sync graph_clean from the round-8 workspace (mechanical, headerless).
2. Rebuild the live byte gate (waves 2-3 INTEGRATION_REPORT sections document it: reference server, PATH /home/linuxbrew/.linuxbrew/bin, --port=3211 equals-form, the Wide-rule probe at interactive-graph-def/cases/raw/1/1 that failed at the Ack cell last round) + fresh captures across clustered/wide/induction/compressed variants; drive the actual RawRule seam (ported repr/simplify content + ported term printer cells -> clean generate -> to_dot). Require byte-equality on all probes.
3. When green: route system_to_dot/_with/render_svg_or_dot_with through clean generate; update routes_graph::dot_output_for_a_simple_system to the reference dialect; DELETE the ported serialization in handlers/dot.rs and graph/abbreviation.rs if the clean abbreviation engine serves the route end-to-end; keep repr.rs/simplify.rs/options.rs ported (solver-side content); keep-and-report residue precisely. GRAPHCLEAN_CORPUS stays green.`
}

function integAPrompt(passed) {
  return `${DIRTY_COMMON}

Unit A — full adoption of web_clean::dispatch as the server's request path (blocked in wave 3 on the sync-dispatch serialization conflict; round-7 redesigned dispatch to snapshot->compute->commit with per-request concurrency semantics probed from the reference${passed.includes('A') ? ', audit PASSED' : ' but its audit DID NOT PASS — in that case only re-sync and report'}). D and B integrators ran before you; rebase on the CURRENT tree. This is the campaign's largest single win (deleting routes.rs/state.rs dispatch erases the last pseudonymous web authors) — proceed deliberately.
1. Re-sync web_clean from the round-7 workspace (mechanical, headerless).
2. Implement StateOps over the ported state::TheoryStore honoring the snapshot/commit contract (clone-on-read maps the ported mutex-cloning get; commit-allocates maps the counter semantics; NO lock held across ProverOps compute — the ported spawn_blocking offload pattern slots into the compute step).
3. Implement ProverOps over the ported pure producers (theory_html::{overview_page,path_html,proof_html}, render_theory_source, title_for, next/prev_theory_path, root::render_index, the proof-step/autoprove logic in handlers/theory.rs — extraction of ported code into pure functions is allowed and stays headered).
4. Route ALL routes through the clean dispatch with axum as transport glue; wire the origin-aware shell (capture an uploaded-overview parity fixture from the live reference first so the non-local branch is byte-gated — wave-3 report flagged this exact missing fixture).
5. Gates: every route test + captured-HS parity fixture green (routes_stubs del_path/verify included); concurrent-behavior sanity (a slow autoprove must not block unrelated GETs — add/adapt a test).
6. When the ported routing/dispatch surfaces (routes.rs route table, handler dispatch shells, state.rs version-allocation logic now expressed through the clean contract) are no longer referenced, DELETE them and rename web_clean -> web. Expect a header drop; report exactly which files and author citations disappeared. If the concurrency contract still cannot be honored, stop, keep ported, report the precise conflict.`
}

async function runUnits() {
  const units = Object.values(UNITS)
  const results = await pipeline(
    units,
    u => agent(u.impl, { label: `impl:${u.key}`, phase: 'Implement', model: 'opus' })
      .then(r => ({ key: u.key, impl: r })),
    (prev, u) => prev && agent(auditPrompt(u), { label: `audit:${u.key}`, phase: 'Audit', model: 'opus' })
      .then(a => ({ ...prev, audit: a }))
  )
  const done = results.filter(Boolean)
  const passed = done.filter(r => /VERDICT:\s*pass/i.test(r.audit || '')).map(r => r.key)
  log(`audits passed=${passed.join(',') || 'none'}`)
  return {
    passed,
    implSummaries: Object.fromEntries(done.map(r => [r.key, (r.impl || '').slice(0, 2500)])),
    auditSummaries: Object.fromEntries(done.map(r => [r.key, (r.audit || '').slice(0, 2500)])),
  }
}

const clean = await runUnits()
phase('Integrate')
const integD = await agent(integDPrompt(clean.passed), { label: 'integrate:D', phase: 'Integrate', model: 'opus' })
const integB = await agent(integBPrompt(clean.passed), { label: 'integrate:B', phase: 'Integrate', model: 'opus' })
const integA = await agent(integAPrompt(clean.passed), { label: 'integrate:A', phase: 'Integrate', model: 'opus' })
return { clean, integD, integB, integA }