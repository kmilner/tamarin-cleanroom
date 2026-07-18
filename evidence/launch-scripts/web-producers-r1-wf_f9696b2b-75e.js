export const meta = {
  name: 'web-producers-r1',
  description: 'Web producer surface cluster: open-side scaffold (SPEC/oracle/interface/targets), sealed round-1 implementation (Fable), both-sides audit. Integration deferred to the serialized queue.',
  phases: [
    { title: 'Scaffold', detail: 'open-side: observable boundary, sub-target decomposition, oracle harness, byte targets' },
    { title: 'Implement', detail: 'sealed round-1 producer fragments (Fable)', model: 'fable' },
    { title: 'Audit', detail: 'both-sides similarity audit + scaffold barrier-hygiene check' },
  ],
}

const SCAFFOLD = `You are an OPEN-SIDE scaffolding agent for the tamarin-rs sealed relicensing campaign (full read access everywhere; you are NOT sealed). Your job: build the sealed-round scaffold for a NEW cluster — the WEB PRODUCER SURFACE — at /home/kamilner/tamarin-cleanroom/weblayer/producers/. Model it on the existing high-quality scaffold at /home/kamilner/tamarin-cleanroom/pretty/ (read its SPEC.md/README.md/interface/ first as the template).

MISSION CONTEXT (read these first): /home/kamilner/tamarin-cleanroom/PROTOCOL.md (barrier rules + "Full-corpus gates"), the web/A-unit sections of /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md and /home/kamilner/tamarin-cleanroom/TAIL_ELIMINATION_SCOPE.md, and the existing weblayer cluster (weblayer/SPEC*.md, workspace/web-clean). The campaign's A-phase-1 (dispatch/state machine, StateOps/ProverOps, page shells) is DONE in web-clean; the citation-topology finding showed the remaining GPL hold is the PRODUCER surface: the open files in /home/kamilner/tamarin-rs/crates/tamarin-server/src/ that render HTML/JSON fragment CONTENT consumed by the dispatch layer — the proof-script pane (west pane), proof-method bodies, proof-tree HTML nodes, root/overview table rows, theory-view fragments. These carry the Kanakanajm ("Jackie") / YannColomb / Esslingen-Schoop headers plus Web/*.hs citations (check the current headers with grep -l "Kanakanajm\\|YannColomb" over crates/tamarin-server/src/ and read those files' header citation lists to fix the exact target set). OUT OF SCOPE: the three routes with no clean home (proof-step progressive UI = Rust-only design, graph = dot -Tsvg passthrough, unload), and all solver-COMPUTED content (constraint systems, proof trees as data structures, dot payloads) — the producers RENDER pre-computed values handed to them.

OBSERVABLE BOUNDARY: HTTP responses of the reference Haskell webserver. Oracles available: (a) the live HS server harness weblayer/oracle/hs_server.sh (ports 3100-3199 — verify it still starts against the HS binary and document usage; it must carry OOM guards: oom_score_adj/ulimit and port cleanup — add them if missing); (b) the captured corpus /home/kamilner/tamarin-rs/scripts/.web_hs_cache (81 manifests: base/lemmas/log/capped/manifest families, ~15k HTML bodies + 2450 JSON envelopes). The sealed room reproduces fragment bytes from these observations only.

DELIVERABLES (mirror the pretty scaffold):
1. producers/SPEC.md — the observable boundary; a sub-target decomposition R1..Rn ranked by (author-yield ÷ isolation) with a recommended round-1 target (prefer the most isolated, most reused fragment family — likely the proof-tree node / proof-script pane HTML, but decide from the captures); the pure-render vs solver-entangled split stated precisely; method requirements (QUERIES.log/BEHAVIOR.md discipline, fixture tests, byte targets); the acceptance ladder (fragment fixtures -> capture-corpus sweep -> integration gate scripts/web_parity.sh full-crawl).
2. producers/README.md — layout + "first move" steps for the sealed implementer.
3. producers/oracle/ — the harness the sealed room runs (wrap/reuse hs_server.sh; plus a capture-extraction helper that slices the relevant fragment families out of .web_hs_cache so the implementer can materialize byte targets without crawling); example inputs.
4. producers/interface/ — input-type surface (expression-stripped Rust type declarations for the pre-computed values the producers render) + required_api.md (one entry point per sub-target). CRITICAL BARRIER HYGIENE, the past failure mode you must not repeat: these files are READ BY THE SEALED ROOM. They must convey BEHAVIOR only — no .hs citations, no upstream identifier names, no tree-structure seams (a previous round's required_api.md leaked splice-anchor names and the similarity audit flagged it). Byte targets are captured OUTPUT only.
5. producers/workspace/producers-clean/ — a compiling std-only stub crate with per-sub-target module stubs and a test scaffold (ignored round-1 tests), same shape as pretty/workspace/pretty-clean.
6. producers/round1/ — curated byte targets for the recommended round-1 sub-target: families.tsv (what each exercises), fetch/extraction script, targets/ materialized.
Verify everything compiles/runs (cargo test on the stub; oracle harness smoke test — start, probe one page, stop, ports freed). Do not launch any implementer. Return a PLAIN-TEXT summary: target-set files + authors, decomposition table, round-1 recommendation, oracle status, and anything surprising.`

const SEAL_RULES = `You are a SEALED implementer under the barrier protocol at /home/kamilner/tamarin-cleanroom/PROTOCOL.md — read it first and comply strictly.
ACCESS RULES: You must NOT read any file under /home/kamilner/tamarin-rs/crates/, and NOT read any Haskell source of tamarin-prover anywhere on disk (notably /home/kamilner/tamarin-rs/tamarin-prover/{lib,src}/ and /home/kamilner/tamarin-rs/tamarin-prover-testing/{lib,src}/). Do NOT read /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md or /home/kamilner/tamarin-cleanroom/TAIL_ELIMINATION_SCOPE.md, and do NOT read any AUDIT.md anywhere in the cleanroom. No web access. You MAY read .spthy example files under /home/kamilner/tamarin-rs/tamarin-prover/examples/, run the sanctioned oracle harnesses, and read the captured-output corpus /home/kamilner/tamarin-rs/scripts/.web_hs_cache. No git commits.
DISCIPLINE: log every oracle interaction in your workspace QUERIES.log; record behavioral facts in BEHAVIOR.md with provenance; comments describe current behavior only. Oracle/server scripts carry OOM guards — preserve them; live HS servers only on ports 3100-3199; STOP every server you start before returning. Return a PLAIN-TEXT summary.`

const IMPL = `${SEAL_RULES}

Workspace: /home/kamilner/tamarin-cleanroom/weblayer/ (read/write freely inside it, EXCEPT AUDIT.md which is off-limits). A new sub-cluster has been scaffolded for you at weblayer/producers/ — READ FIRST: producers/SPEC.md, producers/README.md, producers/interface/{the input types, required_api.md}, then follow README's "first move" exactly. You may also read the rest of the weblayer cluster's sealed materials (web-clean workspace, its BEHAVIOR.md/QUERIES.log, ORACLE_NOTES) — they are prior sealed-side work.

THE TASK — round 1 of the producer surface: implement the SPEC's recommended round-1 sub-target in the stub crate producers/workspace/producers-clean, from black-box observation only: the byte targets in producers/round1/targets/, the capture corpus, and live probes via the producers/oracle harness. Acceptance: byte-parity on the round-1 target families (assert in tests against materialized targets), all cargo tests green, zero warnings, BEHAVIOR.md and QUERIES.log complete enough that a stranger could re-derive every rendering rule from your logged probes. If a fragment's rendering cannot be forced through any observable channel, record it as UNOBSERVABLE with the probes that failed, rather than guessing. Do NOT edit anything outside /home/kamilner/tamarin-cleanroom/weblayer/.`

const AUDIT = `You are a BOTH-SIDES similarity auditor for the tamarin-rs sealed relicensing campaign (exempt from barrier access rules; you may read everything). Audit this round's delta: the NEW weblayer/producers/ cluster under /home/kamilner/tamarin-cleanroom (repo HEAD b4fb110 predates it — git status/diff restricted to weblayer/ shows both the open-side scaffold and the sealed round-1 delta; the scaffold (SPEC/README/oracle/interface/round1) was OPEN-side-authored, the workspace crate + BEHAVIOR/QUERIES are SEALED-side).
TWO audits, both required:
1. BARRIER HYGIENE of the open-authored scaffold: do SPEC.md/interface/required_api.md/round1 leak upstream identifiers, .hs citations, or port seams into sealed-readable material? (Precedent finding: a required_api.md once leaked splice-anchor names.) Leaks are findings with a rewrite instruction for the scaffold file.
2. SIMILARITY of the sealed crate: abstraction-filtration-comparison against the upstream Haskell web layer in /home/kamilner/tamarin-rs/tamarin-prover/src/ (Web/Theory.hs, Web/Handler.hs, Web/Types.hs and related — locate the fragment-producing functions). Byte-parity-forced HTML/JSON shapes are merger/compatibility; flag copied protectable EXPRESSION: identifier constellations, helper decomposition not forced by the boundary, non-observable constants, comment lineage. Cross-check BEHAVIOR.md/QUERIES.log provenance for the hardest-to-guess behaviors.
Append a producers round-1 section to /home/kamilner/tamarin-cleanroom/weblayer/AUDIT.md. Violations get BEHAVIORAL redo instructions (never a patch). End with exactly one line: VERDICT: pass  — or —  VERDICT: fail`

const scaffold = await agent(SCAFFOLD, { label: 'scaffold:producers', phase: 'Scaffold', model: 'opus' })
if (!scaffold) return { scaffold: 'scaffold agent died — round not started', impl: null, audit: null, passed: false }
const impl = await agent(IMPL, { label: 'seal:producers-r1', phase: 'Implement', model: 'fable' })
const audit = await agent(AUDIT, { label: 'audit:producers-r1', phase: 'Audit', model: 'opus' })
return {
  scaffold: (scaffold || '').slice(0, 2000),
  impl: (impl || '').slice(0, 3000),
  audit: (audit || '').slice(0, 2500),
  passed: /VERDICT:\s*pass/i.test(audit || ''),
}