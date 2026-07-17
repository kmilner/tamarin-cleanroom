export const meta = {
  name: 'closure-wave-round5',
  description: 'Second closure wave: clean rounds A5/B6/E5 + audits, dirty G/D swaps, then E/B and A integrators that delete ported GPL code',
  phases: [
    { title: 'Implement', detail: 'clean rounds A5, B6, E5 in parallel' },
    { title: 'Audit', detail: 'per-unit both-sides similarity audits (delta vs 8901219)' },
    { title: 'DirtySwaps', detail: 'G adapter+corpus-gate swap, then D coupled re-integration' },
    { title: 'Integrate', detail: 'integrator 1: E+B swaps; integrator 2: A full Server adoption' },
  ],
}

const COMMON = `You are a CLEAN-ROOM implementer under the campaign protocol at /home/kamilner/tamarin-cleanroom/PROTOCOL.md — read it first and comply strictly.
ACCESS RULES: You must NOT read any file under /home/kamilner/tamarin-rs/crates/, and NOT read any Haskell source (*.hs) of tamarin-prover anywhere on disk (notably /home/kamilner/tamarin-rs/tamarin-prover/{lib,src}/ and /home/kamilner/tamarin-prover-testing/{lib,src}/). No web access of any kind. You MAY: run the reference binary/servers via your cluster's oracle scripts (export PATH="$PATH:/home/linuxbrew/.linuxbrew/bin" if invoking directly — maude/dot live there), read .spthy example files, and read/write freely inside your own cluster directory under /home/kamilner/tamarin-cleanroom/. Do not modify anything outside your cluster directory. Do not run git commits.
DISCIPLINE: log every oracle interaction by appending to your cluster's workspace QUERIES.log; record every derived behavioral fact in BEHAVIOR.md with provenance; append a round section to a REPORT file in the workspace. Code comments must describe current behavior only. Every behavioral claim must trace to a logged probe or a captured fixture — never to model-memory of tamarin internals. Return a PLAIN-TEXT summary (no structured output).`

const UNITS = {
  A: {
    key: 'A', cluster: 'weblayer',
    impl: `${COMMON}

UNIT A (web dispatch), round 5 — close the last two missing route families: del/path and verify. Live-server ports: 3100-3199 only.
Cluster: /home/kamilner/tamarin-cleanroom/weblayer/ — read workspace BEHAVIOR.md, QUERIES.log, REPORT4 first. Crate: workspace/web-clean.

Your round-4 conclusion that del/path and verify are absent in the reference was a PROBE-SHAPE ARTIFACT. Read /home/kamilner/tamarin-cleanroom/weblayer/round5/ORACLE_NOTES.md — it stages four genuine captured reference responses with the requests that produced them: both routes take a further two-segment sub-path (del/path/lemma/<name>, del/path/rules, verify/lemma/<name>, verify/proof/<path>).
Probe both families live and exhaustively with those shapes: status codes, headers, envelope shapes, exact redirect targets, version-index allocation semantics (which operations allocate a fresh index vs mutate in place — reconcile with your round-4 state machine), behavior on undeletable/nonexistent sub-paths, method restrictions, diff-mode (equiv) variants, interaction with subsequent requests (does the deleted step stay deleted in the new version? does verify change proof state?). Extend Route::parse, Server, and (minimally) ProverOps to cover both families with byte-parity tests; correct any round-4 assumptions these probes overturn (e.g. the Handler::Other routing and your honest-negative note in BEHAVIOR.md — update it with the resolution). All existing tests stay green. Stop all servers when done.`,
    auditHint: 'src/Web/Handler.hs (DeleteStepR / TheoryVerifyR handlers), src/Web/Dispatch.hs, src/Web/Types.hs',
  },
  B: {
    key: 'B', cluster: 'graphdot',
    impl: `${COMMON}

UNIT B (graph generation), round 6 — close the record-cell wrap TRIGGER, the one gap blocking adoption. Live-server ports: 3200-3299 only.
Cluster: /home/kamilner/tamarin-cleanroom/graphdot/ — read workspace BEHAVIOR.md (your §3f documents this residual honestly), QUERIES.log Session 5, REPORTs. Crate: workspace/graph-clean.

Your wrap_cell triggers on a cell's OWN flat width > 87. That is wrong in general: the reference decides per-cell within the WHOLE record line — a cell of flat width 25 (e.g. Ack( ~n.4, <x1.4, x2.4> )) wraps when it sits deep on a wide record line, and corpus first-line widths span 19..277, so neither 87 nor any per-cell constant is the trigger. Characterize the true record-level rule with controlled live probes: fix a target cell and systematically vary the number and widths of PRECEDING cells on the same record line (and the rule name/header width), bisect the exact accumulated column at which the target flips from flat to wrapped; test whether the decision for one cell depends on FOLLOWING cells (lookahead) or only on what precedes; check interaction with the per-cell fill width you already pinned (a wrapped cell's continuation lines pack greedily — re-verify at record level). Mine the 12022-payload corpus (144k+ wide records) to validate the candidate rule at scale, then implement record-level layout in build_record (and the RawRule pre-rendered path) so wrap decisions are byte-exact end to end.
OPPORTUNISTIC (only if the trigger is closed with time left): cluster colors are currently caller inputs — mine corpus role-name→color pairs and probe novel role names live to test whether the color is derivable from the name alone; implement if derivable, else document the negative.
The full GRAPHCLEAN_CORPUS round-trip must stay 12022/12022. Stop all servers when done.`,
    auditHint: 'lib/theory/src/Theory/Constraint/System/Dot.hs (renderRow/renderBalanced/scaleIndent and the record assembly around mkNode/dotNodeCompact)',
  },
  E: {
    key: 'E', cluster: 'macros',
    impl: `${COMMON}

UNIT E (macros), round 5 — add the consumer's STAGED-MODE contract so expand() is adoptable.
Cluster: /home/kamilner/tamarin-cleanroom/macros/ — read workspace BEHAVIOR.md, QUERIES.log (Q32-Q40), REPORT. Crate: the macro-clean workspace crate.

INTEROP REQUIREMENT (the consumer's pipeline staging — take as interface contract, then reconcile with your own stage observations): the consumer invokes your expand() at a pipeline stage where
(a) accountability-lemma and case-test formulas must be left UNTOUCHED — a later consumer stage owns their expansion (your own Q-probes showed the reference's --parse-only output preserves the calls and the expansion only becomes visible at close; the consumer's staging matches the parse-stage view), and
(b) rules exist only in their primary form — your expand must touch only the primary rule form and must not assume or recurse into any derived/variant rule forms.
Provide this staged mode (as the default entry or a clearly-named variant — the consumer will call whichever you designate). Keep every existing byte-parity fixture green (full-close behavior stays available and unchanged); add tests pinning the staged mode: acc-lemma/case-test formulas byte-identical through it, macros: block preserved, ordinary lemmas/restrictions/rules still fully expanded, bare-nullary resolution still active.`,
    auditHint: 'lib/term/src/Term/Macro.hs and the applyMacroIn* call sites in the theory-loading pipeline',
  },
}

function auditPrompt(u) {
  return `You are a BOTH-SIDES similarity auditor for the tamarin-rs clean-room relicensing campaign (exempt from clean-room access rules; you may read everything). Audit ONLY this round's delta for unit ${u.key} (${u.cluster}).
The clean-room repo /home/kamilner/tamarin-cleanroom is a git repository whose HEAD (8901219) predates this round: run git -C /home/kamilner/tamarin-cleanroom status and diff, restricted to ${u.cluster}/, to see exactly what changed. Audit those changes (plus immediate context) against the upstream Haskell sources under /home/kamilner/tamarin-rs/tamarin-prover/ — start from ${u.auditHint} and follow the code.
Method: abstraction-filtration-comparison. Filter behavior-dictated content, merger, scenes-a-faire, and anything provably derived from logged probes (cross-check BEHAVIOR.md/QUERIES.log — every behavioral claim in the new code should trace to a logged probe or captured fixture; flag claims that do not). Flag copied protectable EXPRESSION: identifier constellations, structure beyond behavior, tamarin-internal names, non-boundary magic constants, comment lineage.
Append a round section to /home/kamilner/tamarin-cleanroom/${u.cluster}/AUDIT.md. If a violation requires redo, give a BEHAVIORAL redo instruction (never a patch). End with exactly one line: VERDICT: pass  — or —  VERDICT: fail`
}

const DIRTY_COMMON = `You are a DIRTY-ROOM integrator for the tamarin-rs relicensing campaign, working in /home/kamilner/tamarin-rs. Read /home/kamilner/tamarin-cleanroom/PROTOCOL.md (integration rules) and the relevant INTEGRATION_REPORT.md sections first.
RULES: adapters and extraction only — NEVER transplant ported logic into clean (headerless) files; clean vendored sources get mechanical fixes only. A clean file acquiring a GPL header from scripts/gen_license_headers.py is a provenance violation: stop and report. If a gap still blocks, KEEP the ported path + report precisely — never force a swap. Comments describe current code only. No git commits. Append a dated section to INTEGRATION_REPORT.md; return a plain-text summary.
FINAL VALIDATION: cargo build --workspace; cargo test -p tamarin-parser -p tamarin-theory -p tamarin-prover -p tamarin-server; the wellformedness fixture suite; scripts/gen_license_headers.py then --check (expect 0 stale). Report the header-count delta and any author citations that disappeared.`

const G_PROMPT = `${DIRTY_COMMON}

TASK: complete the unit-G swap (derivation checks). The round-4 clean crate (vendored at crates/tamarin-theory/src/deriv_check_clean/, audited pass) is behaviorally complete; the prior integrator identified two remaining items, both dirty-room work:
1. ADAPTER input normalization: the clean candidate collector treats declared nullary function symbols as App(name,[]) but the parser AST leaves them as Var(name). Write a thin, behavior-neutral pre-normalization in the ADAPTER (rewrite Var(n) -> App(n,[]) for names in the theory's declared nullary function set) before feeding the clean check. Do not modify the clean crate.
2. Implement the clean crate's batched DerivabilitySolver (check_rule) over the ported probe-theory solver (prove_probe/synthesise_probe_theory — the solver stays ported and headered): one saturation per rule answering all its variables.
Then BUILD A CORPUS GATE before swapping: run the current (ported) pipeline and the reference oracle (the wellformedness cluster's oracle script drives the v1.13.0 binary; PATH needs /home/linuxbrew/.linuxbrew/bin) over a corpus of example theories that exercise Message Derivation Checks — include theories with macros, let-blocks, multi-variable rules, nat/fresh/pub sorts — and byte-compare the MDC topic. Then route through deriv_check_clean (via the adapter) and require the SAME byte output on the whole gate corpus plus all existing captured tests. NOTE: the previous integrator relocated a byte-neutral show_lvar helper INTO deriv_check.rs — rehome it before deleting. When green, DELETE the ported crates/tamarin-theory/src/deriv_check.rs and route all call sites through the clean orchestration. If any corpus theory diverges, keep-and-report with the exact input and byte diff.`

const D_PROMPT = `${DIRTY_COMMON}

TASK: complete the unit-D swap (console/CLI) — the full coupled re-integration the prior integrator scoped: the round-4 clean modules (console cluster workspace/cli-clean: version/errors/framing/modes/parse + new args/stream) changed APIs together, so re-sync them as a SET into crates/tamarin-prover/src/cli/ and redo the integration in one pass:
1. Re-sync all clean cli modules (mechanical fixes only, headerless) + fixtures.
2. Rewrite cli/adapt.rs against the new APIs (help/version routing must stay byte-identical — the round-4 clean version model fixed a stream bug: banner to stdout, maude preamble to stderr).
3. Route argument parsing through the clean typed parse/args (thin mapping to the existing Args consumer struct; the clean args now carry typed values, validation errors, and the three Rust-only flags --processors/--maude-processes/--data-dir).
4. Route batch output framing in run.rs through the clean stream-aware model: stdout/stderr split byte-exact (theory payloads + summary + help/version/validation envelopes to stdout; maude preamble, progress markers, bare cmdargs one-liners, runtime errors, saturation lines to stderr), including the output-path column alignment.
GATES: cli_e2e suite; the console cluster's split-stream captures (console/workspace/captures/split_*) byte-compared; live spot-checks against the reference via console/oracle/hs_oracle.sh with separated streams; --help and --version byte-identical to the vendored fixtures. When green, DELETE the ported parse_args/Args-validation bodies and the ported framing in run.rs (keep the Rust-only semantics working through the clean typed args). Keep-and-report anything that still blocks.`

function integ1Prompt(passed) {
  return `${DIRTY_COMMON}

Integrate the round-5 clean closures for units E (macros) and B (graph wrap trigger). Similarity audits passed for: ${passed.join(', ') || '(none)'} — only integrate passed units; keep-and-report the rest. Two dirty-room agents (G swap, D re-integration) ran before you — rebase on the CURRENT tree and re-run validation yourself.
- E (if passed): re-sync crates/tamarin-theory/src/macros.rs from the round-5 workspace (it now has the consumer staged mode: acc-lemma/case-test formulas untouched, primary-rule-form only). Route macro_expand::expand_theory_macros / macro_expanded_clone through the clean staged entry. Gate: the three captured tests (bare-nullary; AccLemma and CaseTest non-expansion) pass UNMODIFIED, plus the whole theory suite and the wf pipeline fixtures (macro_expanded_clone feeds the wf checks — verify no regression). When green, DELETE the ported expand_theory_macros/expand_items/expand_rule in macro_expand.rs.
- B (if passed): re-sync crates/tamarin-server/src/graph_clean/ from the round-6 workspace (record-level wrap trigger closed). Wire the RawRule pre-rendered-cell seam: ported repr/simplify produce content, ported term printer renders cells, clean generate assembles + wraps + serializes. Rebuild the live byte gate the round-5 integrator used (INTEGRATION_REPORT.md round-5 section documents it: reference server, PATH /home/linuxbrew/.linuxbrew/bin, ports 3200-3299, wide-record probe via interactive-graph-def/cases/raw — including the Ack( ~n.4, <x1.4, x2.4> ) wide-line case that previously failed) plus fresh captures across clustered/wide/induction/compressed variants; require Rust route output byte-equal to the reference. Update routes_graph::dot_output_for_a_simple_system to the reference dialect. When green, DELETE the ported serialization in handlers/dot.rs (and anything else the clean path now fully covers — abbreviation.rs only if the clean abbreviation engine serves the route end-to-end); keep repr.rs/simplify.rs (solver-side content) ported; keep-and-report residue precisely.`
}

function integ2Prompt(passed) {
  return `${DIRTY_COMMON}

FINAL TASK of this wave: unit A — adopt web_clean::dispatch::Server as the single request path of crates/tamarin-server. Precondition: the round-5 clean web audit ${passed.includes('A') ? 'PASSED' : 'DID NOT PASS — in that case only re-sync web_clean and report; do NOT swap'}. An E/B integrator ran just before you — rebase on the CURRENT tree.
1. Re-sync web_clean/ from the round-5 workspace (Route::parse now covers del/path and verify; mechanical fixes only, headerless).
2. Implement ProverOps by EXTRACTING the fragment/data producers from handlers/theory.rs, theory_html.rs, handlers/proof_tree.rs, handlers/root.rs into pure functions (extraction/refactor of ported code is allowed; extracted code stays in ported headered files; the clean Server owns all transport/dispatch/state).
3. Route ALL routes through Server with axum reduced to transport glue; migrate version/theory state into Server's state machine (one namespace — its counter semantics were probed to match the reference).
4. Gates: every route test + captured-HS parity fixture green (including routes_stubs del/path + verify fixtures — the clean route coverage was extended for exactly these; update axum glue tests as needed, never weaken byte assertions).
5. When the ported dispatch/state files (routes.rs routing, state.rs version logic, handler dispatch shells) are no longer referenced, DELETE them; rename web_clean -> web (suffix-free). Expect a header-count DROP; report exactly which files and author citations disappeared.
This is a large refactor — if you must stop partway, leave the tree BUILDING AND GREEN (partial extraction with the ported router still live is acceptable; a broken tree is not) and report the precise remaining steps.`
}

async function cleanBranch() {
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
  log(`clean branch: audits passed=${passed.join(',') || 'none'}`)
  return {
    passed,
    implSummaries: Object.fromEntries(done.map(r => [r.key, (r.impl || '').slice(0, 2500)])),
    auditSummaries: Object.fromEntries(done.map(r => [r.key, (r.audit || '').slice(0, 2500)])),
  }
}

async function dirtyBranch() {
  const g = await agent(G_PROMPT, { label: 'swap:G', phase: 'DirtySwaps', model: 'opus' })
  const d = await agent(D_PROMPT, { label: 'swap:D', phase: 'DirtySwaps', model: 'opus' })
  return { g: (g || '').slice(0, 4000), d: (d || '').slice(0, 4000) }
}

const cleanP = cleanBranch()
const dirtyP = dirtyBranch()
const clean = await cleanP
const dirty = await dirtyP

phase('Integrate')
const integ1 = await agent(integ1Prompt(clean.passed), { label: 'integrate:E+B', phase: 'Integrate', model: 'opus' })
const integ2 = await agent(integ2Prompt(clean.passed), { label: 'integrate:A', phase: 'Integrate', model: 'opus' })

return { clean, dirty, integ1, integ2 }