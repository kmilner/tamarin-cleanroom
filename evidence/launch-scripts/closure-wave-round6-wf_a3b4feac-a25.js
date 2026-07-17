export const meta = {
  name: 'closure-wave-round6',
  description: 'Third closure wave: clean rounds E6 (nullary guard), B7 (proportional budget), A6 (origin shell + state trait), D5 (summary content + streaming API); audits; then E+D and B+A integrators',
  phases: [
    { title: 'Implement', detail: 'clean rounds E6, B7, A6, D5 in parallel' },
    { title: 'Audit', detail: 'per-unit both-sides similarity audits (delta vs 75807c0)' },
    { title: 'Integrate', detail: 'integrator 1: E+D swaps; integrator 2: B+A swaps' },
  ],
}

const COMMON = `You are a CLEAN-ROOM implementer under the campaign protocol at /home/kamilner/tamarin-cleanroom/PROTOCOL.md — read it first and comply strictly.
ACCESS RULES: You must NOT read any file under /home/kamilner/tamarin-rs/crates/, and NOT read any Haskell source (*.hs) of tamarin-prover anywhere on disk (notably /home/kamilner/tamarin-rs/tamarin-prover/{lib,src}/ and /home/kamilner/tamarin-prover-testing/{lib,src}/). Do NOT read /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md (dirty-room document). No web access. You MAY: run the reference binary/servers via your cluster's oracle scripts (export PATH="$PATH:/home/linuxbrew/.linuxbrew/bin" if invoking directly), read .spthy example files, and read/write freely inside your own cluster directory under /home/kamilner/tamarin-cleanroom/. Do not modify anything outside your cluster directory. No git commits.
DISCIPLINE: log every oracle interaction in your workspace QUERIES.log; record derived behavioral facts in BEHAVIOR.md with provenance; append a round section to a REPORT file. Code comments describe current behavior only. Every behavioral claim must trace to a logged probe or captured fixture — never to model-memory of tamarin internals. Return a PLAIN-TEXT summary (no structured output).`

const UNITS = {
  E: {
    key: 'E', cluster: 'macros',
    impl: `${COMMON}

UNIT E (macros), round 6 — one precise gap: bare-nullary resolution fires too broadly.
Cluster: /home/kamilner/tamarin-cleanroom/macros/ — read workspace BEHAVIOR.md, QUERIES.log (Q32-Q42), REPORT. Crate: workspace/macro-clean.

INTEROP AST FACT: in the consumer's AST a variable node carries (name, numeric index, optional sort/type decoration) — your round-4 probes covered sort decorations (~konst/$konst stay variables) but never an INDEXED bare name. Probe the reference: given macro konst() = h('k'), does a use written konst.1 (indexed variable form) resolve to the macro or stay a variable? Probe the full decoration matrix (plain konst; konst.1; konst.2; indexed+sorted combinations if expressible) in both rule terms and formula positions. Pin exactly which decorations block resolution and add the corresponding guard to your expand_term Var arm (both modes). Add fixtures/tests for each decoration case. All existing tests and fixtures stay green.`,
    auditHint: 'the parser nullary/signature machinery (Theory/Text/Parser/Term.hs nullaryApp, Token.hs addMacrosToSignature) and lib/term/src/Term/Macro.hs',
  },
  B: {
    key: 'B', cluster: 'graphdot',
    impl: `${COMMON}

UNIT B (graph generation), round 7 — refine the group wrap budget: your flat-sum rule is close but not exact. Live-server ports: 3200-3299 only.
Cluster: /home/kamilner/tamarin-cleanroom/graphdot/ — read workspace BEHAVIOR.md (Session 6 group-budget findings), QUERIES.log, REPORTs. Crate: workspace/graph-clean.

NEW OBSERVED DATUM (from a live reference capture you should reproduce first): in a three-cell conclusion group [flat 68, flat 25, flat 14] rendered by the reference, the 68-flat cell wraps as if its effective budget were between 55 and 57 — NOT max(87 - 39, 20) = 48 nor max(87-36,20)=51 as the flat-sum rule predicts — and it packs one MORE element per line than your current fill produces. Your Session-6 rule already matches 98.3% of corpus records; the residual 1.7% apparently follows a different, size-DEPENDENT budget: the effective budget seems to depend on the cells' RELATIVE flat widths, not only the sibling sum.
Task: characterize the exact budget function budget_i = f(flats of all cells in the group). Method: 2D/3D bisection sweeps — controlled live probes over pair and triple groups with systematically varied flats (hold siblings, bisect the target's flip point; repeat across sibling configurations, including very small, equal, and very large siblings; check whether cell ORDER in the group matters; re-verify your floor-20 and single-cell-87 findings as special cases). Fit and cross-validate the rule against the corpus: extract the residual records your Session-6 rule mispredicts, compute which budget reproduces each observed wrap, and require the fitted rule to explain them. Also re-verify the continuation-line fill width under the fitted budget (does a wrapped cell's continuation use the same effective budget or the full width?). Implement in render (thread through wrap_cell_budget/group_cells); the corpus round-trip must stay 12022/12022 and your cell-accuracy metric should reach ~100% on the wrap-prediction census — report the exact residual honestly if any remains. Stop all servers when done.`,
    auditHint: 'lib/theory/src/Theory/Constraint/System/Dot.hs (renderRow/renderBalanced/scaleIndent) and its HughesPJ usage',
  },
  A: {
    key: 'A', cluster: 'weblayer',
    impl: `${COMMON}

UNIT A (web dispatch), round 6 — two adoption-critical items. Live-server ports: 3100-3199 only.
Cluster: /home/kamilner/tamarin-cleanroom/weblayer/ — read workspace BEHAVIOR.md, QUERIES.log, REPORTs. Crate: workspace/web-clean.

ITEM 1 — origin-aware page shell (a round-3 honest negative that is NOW observable): round 4 gave you working theory upload (POST /). Load one theory from the server's local command line AND upload another via POST; byte-compare their overview/main page shells and headers. Pin every difference (menu items present/absent, header content, any other variance) and thread an origin distinction through your page rendering (PageParams/shell) so both variants are byte-exact. Fixtures for both.

ITEM 2 — state-delegation redesign (INTEROP REQUIREMENT): the consumer's state backend is asynchronous, internally caching, and must remain the single owner of theory/version state — your Server cannot own the BTreeMap. Refactor: extract every state operation Server performs (fresh-version allocation from the monotonic counter, retrieval by index, in-place mutation for structural edits/reload, deletion, enumeration for the root page) into a trait boundary (a StateOps trait alongside ProverOps, or fold into ProverOps — your choice, document it). Server keeps ALL dispatch/transport/envelope logic and drives state only through the trait; the lifecycle semantics you probed (which operations allocate vs mutate in place, counter monotonicity, retention of prior versions) become the documented CONTRACT the backend must satisfy. Provide an in-memory reference implementation so all your existing tests keep running byte-identical. All tests green; stop all servers when done.`,
    auditHint: 'src/Web/Handler.hs, src/Web/Hamlet.hs (headerTpl/isLocalOrigin region), src/Web/Types.hs',
  },
  D: {
    key: 'D', cluster: 'console',
    impl: `${COMMON}

UNIT D (console/CLI), round 5 — close the remaining batch-summary content and add a streaming emission contract.
Cluster: /home/kamilner/tamarin-cleanroom/console/ — read workspace BEHAVIOR.md (§10-§11), QUERIES.log, captures/. Oracle: console/oracle/hs_oracle.sh (use split streams throughout). Crate: workspace/cli-clean.

GAP 1 — summary content your model does not yet cover:
(a) Probe prove-mode runs whose analysis is incomplete or bounded (e.g. --prove with --bound=N on theories whose proofs exceed the bound; partial/failed analyses) and pin any additional summary lines the reference emits in those cases (wording, stream, position) that your render_summary does not produce.
(b) Probe theories that produce BOTH wellformedness warnings AND lemma results: pin the exact separator/joining bytes between the warning block and the lemma lines inside the summary (and between multiple theories' blocks).
Byte-exact captures + tests for each.

GAP 2 — streaming emission (INTEROP REQUIREMENT): the consumer emits output INCREMENTALLY while the prover runs (progress markers and payload lines appear as produced, not after completion). Your framing assembles complete Streams at the end. Provide an incremental API — an emitter/sink the consumer can drive line-by-line (or event-by-event) whose total per-stream byte output is identical to the assembled model (prove equivalence with a test that runs both paths on the same inputs). Design it so the consumer chooses flush timing; you define only content and ordering. All existing tests and fixtures stay green.`,
    auditHint: 'src/Main/Mode/Batch.hs (summary assembly, ppRep), src/Main/Utils.hs',
  },
}

function auditPrompt(u) {
  return `You are a BOTH-SIDES similarity auditor for the tamarin-rs clean-room relicensing campaign (exempt from clean-room access rules). Audit ONLY this round's delta for unit ${u.key} (${u.cluster}).
The clean-room repo /home/kamilner/tamarin-cleanroom is a git repository whose HEAD (75807c0) predates this round: run git -C /home/kamilner/tamarin-cleanroom status and diff, restricted to ${u.cluster}/, to see exactly what changed. Audit those changes against the upstream Haskell sources under /home/kamilner/tamarin-rs/tamarin-prover/ — start from ${u.auditHint} and follow the code.
Method: abstraction-filtration-comparison; filter behavior-dictated/merger/scenes-a-faire and probe-derived content (cross-check BEHAVIOR.md/QUERIES.log; flag claims with no logged provenance). Flag copied protectable EXPRESSION: identifier constellations, structure beyond behavior, internal names, non-boundary magic constants, comment lineage.
Append a round section to /home/kamilner/tamarin-cleanroom/${u.cluster}/AUDIT.md. Violations get BEHAVIORAL redo instructions (never patches). End with exactly one line: VERDICT: pass  — or —  VERDICT: fail`
}

const DIRTY_COMMON = `You are a DIRTY-ROOM integrator for the tamarin-rs relicensing campaign, working in /home/kamilner/tamarin-rs. Read /home/kamilner/tamarin-cleanroom/PROTOCOL.md and the relevant INTEGRATION_REPORT.md sections (the wave-2 sections document each blocker this round aimed to close).
RULES: adapters and extraction only — NEVER transplant ported logic into clean (headerless) files; clean vendored sources get mechanical fixes only. A clean file acquiring a GPL header from scripts/gen_license_headers.py is a provenance violation: stop and report. If a gap still blocks, KEEP the ported path + report precisely. Comments describe current code only. No git commits. Append a dated section to INTEGRATION_REPORT.md; return a plain-text summary.
FINAL VALIDATION: cargo build --workspace; cargo test -p tamarin-parser -p tamarin-theory -p tamarin-prover -p tamarin-server; wellformedness fixture suite; scripts/gen_license_headers.py then --check (0 stale). Report the header-count delta and author citations that disappeared. If you must stop partway, leave the tree BUILDING AND GREEN and report the precise remaining steps.`

function integ1Prompt(passed) {
  return `${DIRTY_COMMON}

Integrate units E (macros) and D (console). Audits passed this round for: ${passed.join(', ') || '(none)'} — only integrate passed units.
- E (if passed): re-sync crates/tamarin-theory/src/macros.rs from the round-6 workspace (the bare-nullary Var arm now guards on decoration — the exact gap that failed bare_nullary_macro_name_expands last round). Route macro_expand::expand_theory_macros / macro_expanded_clone through macros::expand_staged. Gate: ALL nine macro_expand captured tests pass UNMODIFIED (especially bare_nullary_macro_name_expands, acc_lemma/case_test non-expansion), theory suite green, wf pipeline fixtures green (macro_expanded_clone feeds the wf checks). When green, DELETE the ported expand_theory_macros/expand_items/expand_rule from macro_expand.rs (keep whatever thin staging glue remains necessary, headered as-is).
- D (if passed): finish the coupled swap the wave-2 D agent scoped (its INTEGRATION_REPORT.md section lists the four blockers with live evidence). Re-sync the round-5 clean modules (summary additions + incremental emission API). Then make the run-driver contract changes in the PORTED files (allowed — they stay headered): defer value-validation forcing until after the maude preamble exactly as the reference does; route structural-error one-liners bare to stderr and validation envelopes (error + full help) to stdout via the clean errors module; recover the stop-on-trace absent/present distinction via an options re-parse; stream batch output through the clean incremental emitter (progress as produced). Gates: cli_e2e; tests/console_split_parity.rs; live split-stream comparisons via the console cluster oracle on: --bound=x (preamble-then-error), unknown flag (23-byte stderr, empty stdout), no-input envelope, multi-theory batch with wf warnings + lemma results, bounded --prove incomplete-analysis case. When green, DELETE the ported parse_args/validation bodies and ported framing/summary assembly in run.rs; the three Rust-only flags keep working through clean typed args.`
}

function integ2Prompt(passed) {
  return `${DIRTY_COMMON}

Integrate units B (graph) and A (web) in crates/tamarin-server. Audits passed this round for: ${passed.join(', ') || '(none)'} — only integrate passed units. An E/D integrator ran before you; rebase on the CURRENT tree.
- B (if passed): re-sync graph_clean from the round-7 workspace (budget function refined beyond flat-sum). Rebuild the live byte gate from the wave-2 round-5/6 INTEGRATION_REPORT sections (reference server, PATH /home/linuxbrew/.linuxbrew/bin, ports 3200-3299, equals-form --port=; the Wide-rule probe at interactive-graph-def/cases/raw/1/1 whose Big-cell divergence was the last failure) + fresh captures across clustered/wide/induction/compressed variants; drive the actual RawRule seam (ported repr/simplify content + ported term printer cells into clean generate). Require byte-equality. When green: route system_to_dot/_with/render_svg_or_dot_with through clean generate, update routes_graph::dot_output_for_a_simple_system to the reference dialect, DELETE the ported serialization in handlers/dot.rs and graph/abbreviation.rs if the clean abbreviation engine now serves the route end-to-end; keep repr.rs/simplify.rs ported; keep-and-report residue.
- A (if passed): re-sync web_clean from the round-6 workspace — it now has (1) origin-aware page shells and (2) the state-delegation trait (Server no longer owns the version map; it drives state through a trait). This removes wave-2 blockers A2 and A3 as clean-side gaps. Implement the state trait over the ported state::TheoryStore (async bridge adapter: the trait impl may block_on/spawn_blocking internally — extraction and bridging of ported code is allowed and stays headered), wire ProverOps over the already-pure ported producers (theory_html::{overview_page,path_html,proof_html}, render_theory_source, title_for, next/prev_theory_path, root::render_index), and route ALL routes through web_clean dispatch with axum as transport glue. Gates: every route test + captured-HS parity fixture green (routes_stubs del_path/verify included). When the ported routing/dispatch surfaces (routes.rs route table, handler dispatch shells, state.rs version-allocation logic now expressed by the clean contract) are no longer referenced, DELETE them and rename web_clean -> web. Expect a header drop; report exactly which files/authors. If the async bridge proves behaviorally unsafe (deadlock/serialization concerns under concurrent proof search), stop, keep ported, and report the precise design conflict instead of forcing.`
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
const integ1 = await agent(integ1Prompt(clean.passed), { label: 'integrate:E+D', phase: 'Integrate', model: 'opus' })
const integ2 = await agent(integ2Prompt(clean.passed), { label: 'integrate:B+A', phase: 'Integrate', model: 'opus' })
return { clean, integ1, integ2 }