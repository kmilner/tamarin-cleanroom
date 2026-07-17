export const meta = {
  name: 'cleanroom-round1',
  description: 'Three isolated clean-room agents reimplement wellformedness, graph/dot, and web layer from black-box oracles only',
  phases: [{ title: 'CleanRoom' }],
}

const REPORT = { type: 'object', properties: {
  components_done: { type: 'array', items: { type: 'string' } },
  loc: { type: 'integer' },
  tests_written: { type: 'integer' },
  tests_passing: { type: 'integer' },
  oracle_queries: { type: 'integer' },
  behavior_spec_summary: { type: 'string' },
  gaps: { type: 'array', items: { type: 'string' } },
  notes: { type: 'string' } },
  required: ['components_done','loc','tests_written','tests_passing','oracle_queries','behavior_spec_summary','gaps','notes'] }

const RULES = `You are a CLEAN-ROOM implementer. Your work will be legally relied upon as an
independent creation, and your complete tool-call transcript is retained as evidence. STRICT RULES:
1. Work ONLY inside your cluster directory. First action: read PROTOCOL.md at
   /home/kamilner/tamarin-cleanroom/PROTOCOL.md, then your SPEC.md and interface/ files.
2. NEVER read any file under /home/kamilner/tamarin-rs/ (source, git, anything). The ONLY
   exceptions: EXECUTING the oracle scripts in your oracle/ dir, and READING files under your
   oracle/captured_responses/ or oracle/captures/ or oracle/examples/ (captured program output
   and observation inputs). Do not use git anywhere. Do not fetch tamarin-prover source or
   internals documentation from the web.
3. Do NOT reproduce tamarin-prover implementation internals from your training memory. Every
   behavior claim must trace to an oracle observation, a capture, or the SPEC. Exact output
   strings must be copied from OBSERVED oracle output (compatibility content). If you notice
   yourself recalling how tamarin-prover's source does something, discard that and observe instead.
4. Log every oracle interaction's purpose in workspace/QUERIES.log. Maintain workspace/BEHAVIOR.md
   as you learn. Design your own internal structure — do not try to guess the original's.
Your final structured report summarizes components, tests, coverage, and gaps.`

phase('CleanRoom')

const [wf, graph, web] = await parallel([
  () => agent(`${RULES}

Your cluster directory: /home/kamilner/tamarin-cleanroom/wellformedness/
Task per SPEC.md: build the wf-clean crate in workspace/ implementing the wellformedness
checker from oracle behavior. Aim for COMPLETE coverage of every check the oracle performs:
enumerate checks by mutating the example inputs and minimal hand-written theories until new
warning topics stop appearing (also mine oracle/captures/ for topics you haven't triggered).
Implement the API in interface/required_api.md over the interface/ast_types.rs model, with
tests per check, cargo test passing. Work as long as needed; report gaps honestly.`,
    { label: 'cleanroom:wellformedness', phase: 'CleanRoom', model: 'opus', schema: REPORT }),

  () => agent(`${RULES}

Your cluster directory: /home/kamilner/tamarin-cleanroom/graphdot/
Task per SPEC.md: reverse-engineer the graph payloads (DOT/JSON, node abbreviation,
simplification/clustering) from the captured web-UI responses and live server probing, write
BEHAVIOR.md as a precise spec, and build the graph-clean crate implementing the observed
transforms over your own data model with tests against captured payloads. Prioritize:
(1) payload-format spec, (2) abbreviation rules, (3) simplification levels. Report gaps honestly.`,
    { label: 'cleanroom:graphdot', phase: 'CleanRoom', model: 'opus', schema: REPORT }),

  () => agent(`${RULES}

Your cluster directory: /home/kamilner/tamarin-cleanroom/weblayer/
CONTINUATION: a prior clean-room session already produced workspace/BEHAVIOR.md, workspace/QUERIES.log,
and the web-clean crate (12 src modules + tests/parity.rs + fixtures) but ended before finishing.
Read what exists in workspace/ FIRST (it is clean-room material you may build on), then continue per
SPEC.md: finish the implementation, make cargo test pass with real parity assertions against captures
(the current test run reports 0 tests executed — wire the fixtures into asserted tests), extend
BEHAVIOR.md coverage (route map, envelopes, main theory-view + proof-tree HTML), and keep logging
oracle queries. NOTE: hs_server.sh has a --port parsing bug; if you use the live server, invoke the
underlying binary with --port=<n> syntax via your own wrapper (black-box execution only, log it).
FINAL REPORT: write workspace/REPORT.md (components done, LOC, tests written/passing, oracle query
count, gaps, notes) and return as your final text a 10-line plain summary of it.`,
    { label: 'cleanroom:weblayer-cont', phase: 'CleanRoom', model: 'opus' })
])

return { wf, graph, web }