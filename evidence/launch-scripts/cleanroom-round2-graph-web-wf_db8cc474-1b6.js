export const meta = {
  name: 'cleanroom-round2-graph-web',
  description: 'Graph round 2: abbreviation-selection policy from paired captures; Web round 2: full-page generality',
  phases: [{ title: 'Redo' }],
}
const RULES = `You are a CLEAN-ROOM implementer continuing prior clean-room work. Read
/home/kamilner/tamarin-cleanroom/PROTOCOL.md and your cluster SPEC.md first; obey access rules
absolutely: never read anything under /home/kamilner/tamarin-rs/ except through the sanctioned
oracle wrappers and the captured_responses/dot_corpus data; no web access; never reproduce
tamarin-prover internals from training memory — every claim must trace to observed output.
Log oracle interactions in workspace/QUERIES.log; update workspace/BEHAVIOR.md.
Write your final report to a file and return a 10-line plain summary. Do NOT use structured output.`

phase('Redo')
const [graph, web] = await parallel([
  () => agent(`${RULES}
Cluster: /home/kamilner/tamarin-cleanroom/graphdot/ — continuation of workspace/graph-clean.
STATUS: your DOT serializer now round-trips 12022/12022 unique captured payloads byte-exact
(corpus extracted to oracle/dot_corpus/). The remaining named gap is the abbreviation SELECTION
policy: WHICH terms get abbreviated when a graph is generated (your earlier note: neither pure
size nor pure occurrence threshold explains sign(...) at 3 occurrences abbreviated while KDF(z)
at similar counts is not).
NEW LEVER: the captured manifests (oracle/captured_responses/*.hs.json — fields {base, lemmas,
log, capped, manifest}, manifest maps url->{kind,status,body}) contain for the same proof states
both 'json' responses (often carrying full/unabbreviated term structure) and 'dot' responses
(abbreviated + legend). Mine these pairs at scale: for each abbreviated legend entry you know the
full term it stands for; for each full term NOT abbreviated you have a negative example. Build a
labeled dataset (thousands of examples), then search systematically for the decision rule
(term size measures, occurrence counts, operator weights, interaction of both, per-graph budget,
ordering effects). Validate candidate rules against the whole dataset and report accuracy; if no
rule reaches 100%, characterize the residual precisely. You may also probe the live oracle
(hs_server.sh — remember the --port= workaround) with crafted theories to isolate boundaries.
Implement the best-validated rule in graph-clean with tests. Report to workspace/REPORT2.md.`,
    { label: 'cleanroom:graph-round2', phase: 'Redo', model: 'opus' }),

  () => agent(`${RULES}
Cluster: /home/kamilner/tamarin-cleanroom/weblayer/ — continuation of workspace/web-clean.
STATUS: 37/37 tests; byte-parity on 2450 JSON envelope bodies and 3 full HTML pages. GOAL: page
GENERALITY — make your renderer reproduce every captured 'html' page body (~15k across the 81
manifests in oracle/captured_responses/). Method: write a bulk harness (like your parity tests but
corpus-wide) that renders from your model parsed out of each captured page's observable inputs, or
where that inverts the problem, classify pages by template family, verify your templates cover
each family, and count byte-exact reproduction per family. Extend templates/models for uncovered
families (proof-method pages, source views, diff views, whatever the corpus shows). Report
per-family byte-parity percentages honestly in workspace/REPORT2.md; 100% is not required this
round — a precise coverage map with the biggest families closed is.`,
    { label: 'cleanroom:web-round2', phase: 'Redo', model: 'opus' }),
])
return { graph, web }