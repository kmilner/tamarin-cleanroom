export const meta = {
  name: 'closure-wave-round4',
  description: 'Clean-room gap-closure for units A,B,C,D,E,G + per-unit similarity audits + sequential theory/server integrators that swap and delete ported GPL code',
  phases: [
    { title: 'Implement', detail: 'six parallel clean-room gap-closure agents' },
    { title: 'Audit', detail: 'per-unit both-sides similarity audits (git delta vs 63ed8a9)' },
    { title: 'Integrate', detail: 'theory (C/D/E/G) then server (B/A) integrators, serialized' },
  ],
}

const COMMON = `You are a CLEAN-ROOM implementer under the campaign protocol at /home/kamilner/tamarin-cleanroom/PROTOCOL.md — read it first and comply strictly.
ACCESS RULES: You must NOT read any file under /home/kamilner/tamarin-rs/crates/, and NOT read any Haskell source (*.hs) of tamarin-prover anywhere on disk (notably /home/kamilner/tamarin-rs/tamarin-prover/{lib,src}/ and /home/kamilner/tamarin-prover-testing/{lib,src}/). No web access of any kind. You MAY: run the reference binary/servers via your cluster's oracle scripts (export PATH="$PATH:/home/linuxbrew/.linuxbrew/bin" if invoking directly — maude/dot live there), read .spthy example files, and read/write freely inside your own cluster directory under /home/kamilner/tamarin-cleanroom/. Do not modify anything outside your cluster directory. Do not run git commits.
DISCIPLINE: log every oracle interaction (command + salient output) by appending to your cluster's workspace QUERIES.log; record every derived behavioral fact in BEHAVIOR.md with provenance (which probe established it); write your round summary into a REPORT file in the workspace (append a new round section). Code comments must describe current behavior only — no narration of changes or process. If your model's training data tempts you to "recall" how tamarin's internals work, do not use it: every behavioral claim must trace to a logged probe or a captured fixture. Return a PLAIN-TEXT summary at the end (no structured output).`

const UNITS = {
  C: {
    key: 'C', cluster: 'wellformedness',
    impl: `${COMMON}

UNIT C (wellformedness), round 4 — close the two gaps that keep the crate from fully replacing post-elaboration checking.
Cluster: /home/kamilner/tamarin-cleanroom/wellformedness/ — read the workspace BEHAVIOR.md, QUERIES.log, SPEC*.md and prior REPORTs first; the oracle wrapper script and its usage are documented there (it accepts extra args such as --diff). Crate: workspace/wf-clean.

GAP 1 — sort-aware quantifier binding in the wrong-form-terms check. Captured reference behavior you currently fail to reproduce: on a theory containing
    lemma L: "All #x. (K(x) @ #x) ==> F"
the reference prints:  Lemma \`L' uses terms of the wrong form: \`Free x'
That is: the temporal binder #x does NOT bind the message-position use x. Your current name-only de-Bruijn binding wrongly binds it and reports nothing (a silent false negative). Systematically probe how quantifier binding interacts with variable sort annotations: #x vs plain x, x:pub, x:fresh, x:nat, same-name binders of different sorts in both directions, nested and shadowed quantifiers, uses in temporal position vs message position. Rework the binding model so every probe matches the oracle byte-exactly. Add fixtures for each probe family.

GAP 2 — guardedness must be decided semantically. Your current guardedness classification is a documented heuristic over-approximation. Probe the reference's guardedness warning exhaustively: existential failure sub-modes, exotic universal bodies, nested/mixed quantifiers, which atom kinds may appear as guards, negation placement, conjunction/disjunction/implication in bodies, quantified variables not covered by the guard atoms, multiple guard atoms, vacuous quantifiers. Also probe which theory item kinds can produce this warning at all (lemmas only? others?). Replace the heuristic with a decision procedure matching the oracle on the full probe set, with byte-exact report text (including the formula rendering your round-3 printer produces). Add fixtures.

All existing tests and fixtures must stay green; run the full fixture suite at the end.`,
    auditHint: 'lib/theory/src/Theory/Tools/Wellformedness.hs and the guardedness conversion (grep formulaToGuarded; Guarded.hs under lib/theory/src/Theory/Constraint/System/)',
  },
  D: {
    key: 'D', cluster: 'console',
    impl: `${COMMON}

UNIT D (console/CLI), round 4 — close the two gaps that keep the argument parser and batch framing from being adoptable.
Cluster: /home/kamilner/tamarin-cleanroom/console/ — read workspace BEHAVIOR.md, QUERIES.log, captures/ and SPEC.md first. Oracle: console/oracle/hs_oracle.sh. Crate: workspace/cli-clean.

GAP 1 — typed argument parsing with value validation. Your parse currently returns untyped (name,value) pairs. Probe the reference's value-validation error taxonomy: non-integer or malformed values to numeric flags (e.g. --bound=x), garbage to enum-like flags (e.g. stop-on-trace variants), empty values, missing values, repeated flags, = vs space-separated forms. Reproduce each error byte-exactly. Then provide a TYPED parse API — the consuming binary requires roughly: parse(args) -> Result<Args, ParseError> where Args carries typed fields (integers parsed, defaults applied) for every flag your tables model and ParseError renders the probed error texts.
INTEROP REQUIREMENT (the consumer's own extensions — not present in the reference, so not probeable): additionally accept --processors <positive integer>, --maude-processes <positive integer>, and --data-dir <path> as typed flags. Do NOT list them in help output — help stays byte-exact to your reference captures.

GAP 2 — stream-aware batch framing. Your framing module models a single merged capture. Re-probe batch mode capturing stdout and stderr SEPARATELY (1>out.txt 2>err.txt) across: single and multiple input files, --output/-o variants (and none), parse-only vs prove, error cases. Determine exactly which lines (tool preamble, per-theory progress markers, per-lemma results, the summary-of-summaries block, output-file path lines) go to which stream, and the exact format/alignment of the output-path column in summaries when an output file is written. Rework framing into a stream-aware model reproducing BOTH streams byte-exactly. Add split-stream captures and tests.`,
    auditHint: 'src/Main/Console.hs, src/Main/Environment.hs, src/Main/Mode/Batch.hs, src/Main/TheoryLoader.hs, src/Main.hs',
  },
  E: {
    key: 'E', cluster: 'macros',
    impl: `${COMMON}

UNIT E (macros), round 4 — close three expansion-semantics gaps.
Cluster: /home/kamilner/tamarin-cleanroom/macros/ — read workspace BEHAVIOR.md, SPEC.md, QUERIES.log and the example theories first; oracle recipes are documented there (reference binary output of parsed/processed theories). Crate: the macro-clean workspace crate.

GAP 1 — bare nullary macro uses. Probe how the reference treats a use of a 0-ary macro written WITHOUT parentheses (e.g. macro konst() = h('k'), then a rule using plain konst in a term position). INTEROP FACT about your consumer (not probeable, take as interface contract): in the consuming parser's AST such a use reaches your expander as a plain variable node Var("konst"), NOT as an application — your expand must resolve and expand such names if (and exactly when) the reference does.

GAP 2 — accountability lemmas. Construct theories where macros are used inside accountability-lemma formulas; probe whether the reference's macro stage expands them (inspect the stage at which expansion becomes visible in reference output — parse-only vs later processing — and pin the parse/wf-stage behavior your consumer needs).

GAP 3 — case tests. Same probe for case-test formulas.

Also verify: the consuming pipeline requires the macros: declaration block be PRESERVED in your expand output (probe whether the reference's pretty output retains the block after processing; regardless, preserving declarations in place is an interop requirement). Match the oracle everywhere; add parity fixtures for each gap; existing tests stay green.`,
    auditHint: 'the macro machinery (grep -r applyMacros lib/ src/; Macro.hs; TheoryObject/item-level expansion call sites)',
  },
  G: {
    key: 'G', cluster: 'derivcheck',
    impl: `${COMMON}

UNIT G (derivation checks), round 4 — close five gaps so your report can replace the consumer's.
Cluster: /home/kamilner/tamarin-cleanroom/derivcheck/ — read workspace BEHAVIOR.md, QUERIES.log, SPEC.md first; the oracle is the reference wf output's "Message Derivation Checks" topic (recipes in your cluster). Crate: workspace/derivcheck-clean.

GAP 1 — output contract (INTEROP): the consumer's report renderer adds NO per-topic header; your message_derivation_checks must return the COMPLETE topic block byte-exact as the reference prints it — INCLUDING the underlined "Message Derivation Checks" heading — as one single error value, not fragments.

GAP 2 — multi-variable ordering: construct rules where two or more variables are flagged non-derivable, with mixed sorts (public, fresh, plain message, temporal, natural-number if expressible), colliding names, differing declaration/first-use positions. Pin the reference's exact emission order. Your current (sort_rank, name, idx) key is suspect — derive the true ordering black-box from discriminating probes (your existing fixtures do not discriminate; build ones that do).

GAP 3 — candidate scope: probe which variable classes can EVER be flagged: public-sort variables, temporal/node variables, names colliding with user-declared nullary functions, variables appearing only in actions vs only in conclusions vs premises, variables introduced via a rule's let-block. Match the reference on both which warnings appear and which never can.

GAP 4 — pre-expansion: probe rules where macros:/let{} constructs affect derivability (e.g. a macro or let body wrapping a value in a fresh or public constructor). Determine whether the reference decides derivability before or after that expansion, and match.

GAP 5 — solver interface shape (INTEROP REQUIREMENT): the consumer's solver saturates once per RULE and answers all that rule's variables from one saturation; per-variable trait calls force wasteful re-saturation. Provide a batch API — one call per rule carrying all candidate variables, returning per-variable verdicts (keep the current per-variable trait as a thin wrapper if you wish). Update tests/fixtures.`,
    auditHint: 'derivation-check logic (grep -ri "derivation" lib/theory/src/Theory/Tools/; Wellformedness.hs; the probe-theory construction near src/ Prover modules)',
  },
  B: {
    key: 'B', cluster: 'graphdot',
    impl: `${COMMON}

UNIT B (graph generation), round 5 — complete generate() so it can serve as the consumer's graph producer. Live-server ports: use 3200-3299 only.
Cluster: /home/kamilner/tamarin-cleanroom/graphdot/ — read workspace BEHAVIOR.md (esp. the cluster section and the round-4 wrap section), QUERIES.log, REPORTs first; oracle: oracle/dot_corpus (12022 payloads), the captured web manifests, and the live reference server recipe in your cluster. Crate: workspace/graph-clean.

1 — role CLUSTER subgraphs: generate() currently emits a flat node list and never the subgraph "cluster_..." blocks your own BEHAVIOR.md documents for role-annotated theories. Wire cluster emission into generate(): probe live role-annotated theories, validate byte-exact against corpus payloads containing clusters.

2 — record-cell WRAP: your render module (fill width, paragraph fill, fits-one-line) exists but cell construction renders flat. Wire wrapping into record-cell construction. Also close the two characterized residuals byte-level now — the one-element-lookahead behavior on continuation lines and the delimiter-peel at the boundary — with targeted live probes across the fill-width boundary.

3 — missing-node kinds: your model has an open-PREMISE target node. Probe the DUAL — a conclusion feeding a node absent from the graph — and any additional node kind appearing in induction-style proofs (probe lemmas annotated [use_induction] and constraint systems carrying a designated last timepoint). Extend model + serializer to whatever the reference emits, byte-exact.

4 — pre-rendered-cell entry (INTEROP REQUIREMENT): the consumer renders fact/term cell text with its own printer; add a generate entry that accepts PRE-RENDERED cell strings (your wrap/escape/abbreviation pipeline still applies to them as observed) so the consumer needs no term-model bridge. Keep the existing Term-based path too.

Validate: the full GRAPHCLEAN_CORPUS round-trip must stay green; add fixtures for each closed gap. Stop all servers when done.`,
    auditHint: 'the dot/graph rendering sources (lib/theory/src/Theory/Constraint/System/Dot.hs, Simplification-related modules, src/Web graph emission)',
  },
  A: {
    key: 'A', cluster: 'weblayer',
    impl: `${COMMON}

UNIT A (web dispatch), round 4 — extend Route::parse and Server<P: ProverOps> to the FULL route surface so the dispatcher can own the consumer's entire request path. Live-server ports: use 3100-3199 only.
Cluster: /home/kamilner/tamarin-cleanroom/weblayer/ — read workspace BEHAVIOR.md, QUERIES.log, the route census and captured envelope manifests first; live reference server recipe in your cluster. Crate: workspace/web-clean.

Your dispatch currently covers main/*, overview/*, autoprove, next/prev, source/message, intdot, interactive-graph-def, and the edit POST. Extend to the REST of the observed surface: the root page (GET /) and theory-upload POST, /static/* asset serving, download, reload, kill, the equiv overview variant, del/path, verify, and robots.txt. For each: probe live and/or from your captured manifests — methods, status codes, headers (content-type, disposition, caching), redirect behavior, and body bytes; byte-parity tests per route family.

State ownership: Server must own ALL version/theory state for the full surface — one state machine. Extend your version-map/counter semantics per observed behavior across upload, reload, kill, delete sequences (probe the index/version lifecycles live: what appears on the root page after each operation, which indices are reused or retired, what 404s after kill/delete).

Grow the ProverOps trait minimally for what the new routes need — pure data/fragment producers only; keep all transport concerns in Server. All existing tests stay green. Stop all servers when done.`,
    auditHint: 'src/Web/Handler.hs, src/Web/Dispatch.hs, src/Web/Types.hs, src/Web/Settings.hs',
  },
}

function auditPrompt(u) {
  return `You are a BOTH-SIDES similarity auditor for the tamarin-rs clean-room relicensing campaign (you are exempt from the clean-room access rules; you may read everything). Audit ONLY this round's delta for unit ${u.key} (${u.cluster}).
The clean-room repo /home/kamilner/tamarin-cleanroom is a git repository whose HEAD (63ed8a9) predates this round: run git -C /home/kamilner/tamarin-cleanroom status and diff, restricted to the ${u.cluster}/ directory, to see exactly what the round changed. Audit those changes (plus immediate context) against the corresponding upstream Haskell sources under /home/kamilner/tamarin-rs/tamarin-prover/ — start from ${u.auditHint} and follow the code.
Method: abstraction-filtration-comparison. Filter out behavior-dictated content, merger, scenes-a-faire, and anything provably derived from logged probes (cross-check the delta's BEHAVIOR.md/QUERIES.log entries — every behavioral claim in the new code should trace to a logged probe or captured fixture; flag claims that do not). Flag copied protectable EXPRESSION: identifier constellations, structural organization beyond what behavior dictates, tamarin-internal names, magic constants not observable at the boundary, comment lineage.
Append a new round section to /home/kamilner/tamarin-cleanroom/${u.cluster}/AUDIT.md with your findings. If any violation requires redo, state a precise BEHAVIORAL redo instruction (never a patch, never code). End your reply with exactly one line: VERDICT: pass  — or —  VERDICT: fail`
}

const INTEG_COMMON = `You are the DIRTY-ROOM integrator for the tamarin-rs relicensing campaign, working in /home/kamilner/tamarin-rs. Read /home/kamilner/tamarin-cleanroom/PROTOCOL.md (integration rules) and the relevant sections of /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md first — this round's clean-room work aimed to close exactly the blockers documented there.
RULES: adapters and extraction only — NEVER transplant ported logic into clean files (a clean file acquiring a GPL header from scripts/gen_license_headers.py is a provenance violation: stop and report it). Vendored clean sources get mechanical fixes only (crate:: path adjustments, include paths); they stay headerless. If a gap still blocks a swap, KEEP the ported path with its header and report the gap precisely — never force it. Code comments describe current code only. Do not run git commits. Return a plain-text summary; append a new dated section to INTEGRATION_REPORT.md with per-unit deleted/kept/header-delta detail.
VALIDATION (at the end): cargo build --workspace; cargo test -p tamarin-parser -p tamarin-theory -p tamarin-prover -p tamarin-server; the wellformedness fixture suite; scripts/gen_license_headers.py followed by --check (expect 0 stale).`

function theoryIntegratorPrompt(passed, failed) {
  return `${INTEG_COMMON}

Integrate the round-4 clean-room closures for units C, D, E, G. Similarity audits PASSED for: ${passed.join(', ') || '(none)'}. FAILED (do NOT integrate these — keep-and-report): ${failed.join(', ') || '(none)'}.
For each passed unit, re-sync the clean sources from the cluster workspace into the repo (vendoring recipes are in INTEGRATION_REPORT.md; the wf/ re-sync MUST preserve crates/tamarin-parser/src/wf/order.rs and its two mod-lines), then perform the swap the prior blockers prevented:
- C: route the formula-terms check through the clean sort-aware binding, and the run.rs/theory_io.rs guardedness call sites through the clean semantic guardedness. Gate: parser suite + wellformedness fixture suite + the captured node-binder Free-x case, all byte-green. When green, DELETE the ported check_terms module and elaborate::check_guarded_wf (locations in INTEGRATION_REPORT.md section C.2) plus any dead code they leave.
- D: route argument parsing through the clean typed parse (a thin mapping to the existing Args struct is fine; the clean parser now accepts the Rust-only flags) and batch output framing through the clean stream-aware framing. Gate: cli_e2e + captured CLI fixtures byte-green including the stdout/stderr split and the output-path column. When green, DELETE the ported parse_args/validation bodies and the ported framing in run.rs.
- E: re-attempt the expand rewire. The three previously-failing captured tests (bare nullary macro; AccLemma; CaseTest non-expansion) must pass UNMODIFIED; the positional-zip macros:-preservation adapter from last round is allowed. When green, DELETE the ported expand_theory_macros/expand_items/expand_rule.
- G: implement the clean crate's batch DerivabilitySolver interface over the ported probe-theory solver (adapter; the solver itself stays ported and headered), and replace the ported deriv_check orchestration + report with deriv_check_clean. Gate: theory suite green + live oracle spot-probes of the Message Derivation Checks wf topic byte-exact (the wellformedness cluster's oracle script can drive the reference binary). When green, DELETE the ported deriv_check.rs.
Where a clean module replaces the last ported holder of a natural name, take the suffix-free name.`
}

function serverIntegratorPrompt(passed, failed) {
  return `${INTEG_COMMON}

Integrate the round-5 clean-room closures for units B (graph) and A (web) in crates/tamarin-server. Similarity audits PASSED for: ${passed.join(', ') || '(none)'}. FAILED (do NOT integrate these — keep-and-report): ${failed.join(', ') || '(none)'}. NOTE: a theory-side integrator (units C/D/E/G) ran before you and is finished; rebase your expectations on the CURRENT tree, and re-run the full validation yourself.
- B: re-sync graph_clean from /home/kamilner/tamarin-cleanroom/graphdot/workspace/graph-clean (mechanical fixes only, headerless). Route system_to_dot / system_to_dot_with / render_svg_or_dot_with through graph_clean generate — the new pre-rendered-cell entry with the ported term printer as cell provider is the intended adapter seam (the ported pretty-printer belongs to the theory layer and stays ported+headered; that is fine). BYTE GATE: the in-repo captured HS graph fixtures are ISE pages, so build a LIVE gate: use the graphdot cluster's reference-server recipes (PATH needs /home/linuxbrew/.linuxbrew/bin; ports 3200-3299) to capture fresh reference DOT for a probe set (clustered/role-annotated, wide-cell, induction, compressed and uncompressed variants) and require the Rust route output byte-equal; keep the GRAPHCLEAN_CORPUS round-trip green. Update routes_graph::dot_output_for_a_simple_system from the ported dialect to the reference dialect. When green, DELETE handlers/dot.rs's serialization, graph/abbreviation.rs, graph/repr.rs, graph/simplify.rs, graph/options.rs — whatever the clean modules now cover; keep-and-report any residue precisely. Then rename graph_clean -> graph (update paths/imports; suffix-free).
- A: adopt web_clean::dispatch::Server as the single request path. Implement ProverOps by EXTRACTING the fragment/data producers from handlers/theory.rs, theory_html.rs, handlers/proof_tree.rs into pure functions — extraction/refactor of ported code is allowed; extracted code stays in ported (headered) files. Route ALL routes through Server (its Route::parse now covers the full surface) with axum reduced to transport glue; migrate version/theory state into Server's state machine. Gate: ALL route tests + captured-HS parity fixtures green (update axum glue tests as needed, never weaken byte assertions). When the ported dispatch/state files (routes.rs routing, state.rs version logic, handler dispatch shells) are no longer referenced, DELETE them. Then rename web_clean -> web (suffix-free).
Stop any live servers you start. Expect the header count to DROP this round; report the exact delta and which upstream authors' citations disappeared.`
}

async function runBranch(keys) {
  const units = keys.map(k => UNITS[k])
  const results = await pipeline(
    units,
    u => agent(u.impl, { label: `impl:${u.key}`, phase: 'Implement', model: 'opus' })
      .then(r => ({ key: u.key, impl: r })),
    (prev, u) => prev && agent(auditPrompt(u), { label: `audit:${u.key}`, phase: 'Audit', model: 'opus' })
      .then(a => ({ ...prev, audit: a }))
  )
  const done = results.filter(Boolean)
  const passed = done.filter(r => /VERDICT:\s*pass/i.test(r.audit || '')).map(r => r.key)
  const failed = keys.filter(k => !passed.includes(k))
  log(`branch [${keys.join(',')}]: audits passed=${passed.join(',') || 'none'} failed=${failed.join(',') || 'none'}`)
  return {
    passed, failed,
    implSummaries: Object.fromEntries(done.map(r => [r.key, (r.impl || '').slice(0, 3000)])),
    auditSummaries: Object.fromEntries(done.map(r => [r.key, (r.audit || '').slice(0, 3000)])),
  }
}

const theoryP = runBranch(['C', 'D', 'E', 'G'])
const serverP = runBranch(['B', 'A'])

const theory = await theoryP
phase('Integrate')
const theoryInteg = await agent(
  theoryIntegratorPrompt(theory.passed, theory.failed),
  { label: 'integrate:theory-CDEG', phase: 'Integrate', model: 'opus' })

const server = await serverP
const serverInteg = await agent(
  serverIntegratorPrompt(server.passed, server.failed),
  { label: 'integrate:server-BA', phase: 'Integrate', model: 'opus' })

return {
  theory: { ...theory, integration: theoryInteg },
  server: { ...server, integration: serverInteg },
}