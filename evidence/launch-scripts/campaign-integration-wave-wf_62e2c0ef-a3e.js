export const meta = {
  name: 'campaign-integration-wave',
  description: 'B simplification re-probe + audit, then A+B server integration; parallel C/D/E/F/G integration',
  phases: [{ title: 'Reprobe' }, { title: 'Integrate' }],
}

const CLEAN_RULES = `You are a CLEAN-ROOM implementer (evidence transcript retained). Read
/home/kamilner/tamarin-cleanroom/PROTOCOL.md. Never read anything under /home/kamilner/tamarin-rs/;
only execute your cluster's oracle scripts (extra CLI flags allowed) and read your cluster's
oracle/ data. No git, no web, no recall of tamarin-prover internals — observe instead. Log oracle
interactions in workspace/QUERIES.log; update workspace/BEHAVIOR.md. Return a 10-line plain-text
summary; fold report content into BEHAVIOR.md. No structured output.`

const DIRTY_RULES = `You are the DIRTY-ROOM INTEGRATOR (may read everything; must NOT transplant
logic from replaced files — write ADAPTERS: type conversion and data plumbing routing existing
entry points through the audited clean-room code). Where clean code lacks a behavior, KEEP the old
path (with its header) and report the gap precisely; never patch clean code with old logic, never
restore old logic outside kept files. Files you author carry NO license header. After finishing,
run: python3 scripts/gen_license_headers.py (from the repo root) so headers track the new
derivation surface, and validate with cargo build --workspace plus the test suites named below —
captured-HS parity fixtures MUST stay green; if a parity test fails, fix your ADAPTER only.
NAMING POLICY: integrated clean modules take natural suffix-free names; when you delete a ported
module, the clean replacement takes its name (precedent: crates/tamarin-parser/src/wf/).
Repo: /home/kamilner/tamarin-rs. Clean sources under /home/kamilner/tamarin-cleanroom/<cluster>/workspace/.
Append what you did to /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md. Return a 12-line
plain-text summary. No structured output.`

phase('Reprobe')
const serverBranch = async () => {
  const reprobe = await agent(`${CLEAN_RULES}
Cluster: /home/kamilner/tamarin-cleanroom/graphdot/ (continue workspace/graph-clean; ports 3200-3299).
GAP TO CLOSE: simplification levels 1/2/3 produced byte-identical graphs in all your round-3 probes.
METHODOLOGICAL LEAD (re-examine your probing method): your URL-parameter probes may be overridden by
server-side per-session state — inspect Set-Cookie headers on responses and the served JS's cookie
handling; drive the level through whatever channel the responses actually honor, and verify a change
took effect before diffing. Also probe structurally RICHER graphs (deep source-case trees, long
chains, repeated rule instances — pick heavyweight corpus theories) where levels are likelier to
diverge. If levels genuinely diverge, characterize and implement each level's transform with tests.
If they truly do not diverge, document the negative rigorously (channel verified + rich graphs).
Secondly, with the same session-state lens, re-attempt the record-cell wrap DECISION rule.`,
    { label: 'B:simplification-reprobe', phase: 'Reprobe', model: 'opus' })

  const audit = await agent(`You are the SIMILARITY AUDITOR (both-sides). Read
/home/kamilner/tamarin-cleanroom/PROTOCOL.md. Audit ONLY the just-added simplification/wrap delta in
/home/kamilner/tamarin-cleanroom/graphdot/workspace/graph-clean/ (identify it from BEHAVIOR.md's
newest session and git-less file mtimes / QUERIES.log tail) against the Haskell
lib/theory/src/Theory/Constraint/System/Graph/Simplification.hs and the Dot.hs wrap machinery in
/home/kamilner/tamarin-rs/tamarin-prover/. Abstraction-filtration as before; append to
graphdot/AUDIT.md under "## Simplification re-probe audit". Return "graphdot-delta: pass|redo" +
one paragraph + finding count. No structured output.`,
    { label: 'audit:B-delta', phase: 'Reprobe', model: 'opus' })

  return await agent(`${DIRTY_RULES}
TASK: complete the tamarin-server integration for units A (web handlers) and B (graph generation).
Clean sources: weblayer/workspace/web-clean (dispatch.rs Server<ProverOps> state machine, routes,
forms incl. dynamic textarea rows, templates) and graphdot/workspace/graph-clean (generation:
alloc/render/generate/options + serializer + abbreviation). Latest re-probe summary (may have
closed the simplification-level and wrap gaps — read graphdot BEHAVIOR.md newest section first):
${'${'}JSON.stringify((await Promise.resolve('')))${'}'} — ignore this placeholder; read the files.
1. Refresh the vendored copies in crates/tamarin-server/src/ from the clean workspaces (mechanical
   path fixes only). Rename per policy: after deleting the ported graph/ modules you replace,
   graph_clean -> graph; web_clean -> web.
2. GRAPH: route system_to_dot/system_to_dot_with/render_svg_or_dot_with through clean generation
   (adapter: ported System -> clean model). Delete handlers/dot.rs ported serialization,
   graph/abbreviation.rs, graph/repr.rs, graph/simplify.rs where the clean side now covers them
   (generation round-3 characterized content mapping; re-probe may cover simplification). Keep
   ported code ONLY for verified remaining gaps (e.g. wrap decision if still open) — report each.
3. WEB: implement the clean ProverOps trait as an adapter over the ported prover
   (parse/apply-method/autoprove/pretty fragments), route requests through web::dispatch, replace
   remaining theory_html.rs rendering and the non-local-origin/edit-form kept paths (textarea rows
   now solved; origin stays a parameterized hook — wire it to the real peer-address check).
   Delete theory_html.rs and ported handler logic that dispatch now covers.
4. Validate: cargo build --workspace; cargo test -p tamarin-server (all suites, parity fixtures
   green); cargo test -p tamarin-parser; the graph-clean opt-in corpus gates
   (GRAPHCLEAN_CORPUS=/home/kamilner/tamarin-cleanroom/graphdot/oracle/dot_corpus). Then
   gen_license_headers.py and report the header-count delta.`,
    { label: 'integrate:server-A+B', phase: 'Integrate', model: 'opus' })
}

const theoryBranch = async () =>
  await agent(`${DIRTY_RULES}
TASK: integrate units C, D, E, F, G (sequential; keep the build green between units).
C (wf round-3): re-sync clean sources from wellformedness/workspace/wf-clean/src into
  crates/tamarin-parser/src/wf/ (same mechanical path-fix recipe as the existing files; PRESERVE
  the workspace-authored wf/order.rs and the 'pub mod order; pub use order::*;' lines in wf/mod.rs).
  Then replace the Wellformedness.hs-derived residue: crates/tamarin-theory/src/check_terms.rs's
  formula-terms checking becomes an adapter computing the reducible-symbol set from the MaudeSig
  and calling wf::formula_terms_reducible / check_theory_with_reducible; the guardedness call path
  (elaborate::check_guarded_wf call sites in run.rs/theory_io.rs) routes through the clean two-mode
  guardedness. Delete check_terms.rs if fully covered; keep-and-report otherwise. Validate with
  cargo test -p tamarin-parser AND the oracle harness:
  PATH=/home/linuxbrew/.linuxbrew/bin:$PATH cargo run -p tamarin-parser --example wellformedness_fixtures.
D (console): vendor console/workspace/cli-clean into crates/tamarin-prover/src/cli/ (suffix-free),
  route arg parsing, help/version rendering, error taxonomy, and batch framing through it; delete
  the ported equivalents in cli.rs and the console slices of run.rs that it covers (the deep
  theory-loading driver logic in run.rs stays ported). Validate: cargo test -p tamarin-prover and
  spot-check binary output: target/debug/tamarin-prover --help / --version byte-vs the HS captures
  in console/workspace/cli-clean/tests.
E (macros): vendor macros/workspace/macro-clean as the macro-expansion module where expansion
  currently lives (find the ported expansion sites via headers citing Macro.hs/TheoryObject.hs);
  adapter maps the real AST to the clean expand(); delete replaced expansion code. Validate:
  cargo test -p tamarin-parser -p tamarin-theory + run the oracle on a macro theory via the corpus
  scripts if quick.
F (injective): vendor injective/workspace/injfacts-clean; replace
  crates/tamarin-theory/src/tools/injective_fact_instances.rs with an adapter over the clean
  decision (rules -> tag set). Validate: cargo test -p tamarin-theory.
G (derivcheck): vendor derivcheck/workspace/derivcheck-clean; implement its DerivabilitySolver
  trait as an adapter over the ported solver machinery that deriv_check.rs currently drives;
  replace deriv_check.rs's orchestration/report-text with the clean crate; the solver itself stays
  ported. Validate: cargo test -p tamarin-theory and one oracle comparison on a private-function
  theory (derivcheck/oracle/examples/) via the HS binary if quick.
FINALLY: cargo build --workspace; cargo test -p tamarin-server (regression); gen_license_headers.py;
report per-unit: files deleted, files kept-with-reason, header-count delta.`,
    { label: 'integrate:C-G', phase: 'Integrate', model: 'opus' })

const [server, theory] = await parallel([serverBranch, theoryBranch])
return { server, theory }