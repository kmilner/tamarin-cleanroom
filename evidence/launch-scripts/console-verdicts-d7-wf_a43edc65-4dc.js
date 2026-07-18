export const meta = {
  name: 'console-verdicts-d7',
  description: 'Overlapped clean round D7: the non-diff lemma-verdict family (unfinishable/undetermined/invalidated/error) + audit — cleanroom only, runs alongside wave-4 integrators',
  phases: [
    { title: 'Implement', detail: 'D7 non-diff verdict taxonomy' },
    { title: 'Audit', detail: 'both-sides delta audit (baseline d1ada4b)' },
  ],
}

const IMPL = `You are a CLEAN-ROOM implementer under the campaign protocol at /home/kamilner/tamarin-cleanroom/PROTOCOL.md — read it first and comply strictly.
ACCESS RULES: You must NOT read any file under /home/kamilner/tamarin-rs/crates/, and NOT read any Haskell source (*.hs) of tamarin-prover anywhere on disk. Do NOT read /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md. No web access. You MAY run the reference binary via console/oracle/hs_oracle.sh (or directly with PATH="$PATH:/home/linuxbrew/.linuxbrew/bin"; ALWAYS under timeout, and prefer bounded --prove runs — long unbounded proof searches are forbidden for resource safety), read .spthy example files, and read/write freely inside /home/kamilner/tamarin-cleanroom/console/. No git commits.
DISCIPLINE: log every oracle interaction in workspace QUERIES.log; record behavioral facts in BEHAVIOR.md with provenance; append a round section to a report file (fold into BEHAVIOR.md if .md creation is blocked). Comments describe current behavior only. Return a PLAIN-TEXT summary.

UNIT D (console/CLI), round 7 — the NON-diff lemma-verdict family. Round 6 closed the --diff taxonomy (LemmaSide/verdict_phrase); this round targets the remaining per-lemma summary phrases in PLAIN batch mode that your LemmaResult still cannot express.
KNOWN CAPTURED FACT to reproduce first: running plain batch --prove on the example theory /home/kamilner/tamarin-rs/tamarin-prover/examples/csf23-subterms/YellowTest.spthy, the reference's summary lines for its lemmas read:
    <name> ... analysis cannot be finished (reducible operators in subterms) (4 steps)
— a verdict phrase distinct from verified/falsified that your model cannot render.
Then hunt the rest of the family systematically: bounded proofs (--bound=N exhaustion — how does the summary phrase an unfinished-by-bound analysis, and does it differ from the reducible-operators case?), --stop-on-trace variants, lemmas that error during analysis (e.g. induction on non-inductible lemmas, solver failures), [sources] lemmas with remaining open/partial deconstructions (check the summary's sources line phrasing states), anything --prove=<pattern> non-matching produces, and any advisory lines accompanying these states (e.g. a might-be-wrong warning — pin its exact wording, stream, and position). Sweep example theories for further distinct phrases if helpful (grep your own captured outputs, not sources).
Extend LemmaResult/verdict rendering to cover every phrase found, with byte-parity fixtures per verdict (split streams). All existing tests and fixtures stay green.`

const AUDIT = `You are a BOTH-SIDES similarity auditor for the tamarin-rs clean-room relicensing campaign (exempt from clean-room access rules). Audit ONLY this round's delta for unit D (console).
The clean-room repo /home/kamilner/tamarin-cleanroom is a git repository whose HEAD (d1ada4b) predates this round: run git -C /home/kamilner/tamarin-cleanroom status and diff, restricted to console/, to see exactly what changed. Audit against the upstream Haskell sources under /home/kamilner/tamarin-rs/tamarin-prover/ — start from src/Main/Mode/Batch.hs and the proof-status rendering (grep showProofStatus / prettyProofStatus in lib/theory/src/).
Method: abstraction-filtration-comparison; filter behavior-dictated/merger/scenes-a-faire and probe-derived content (cross-check BEHAVIOR.md/QUERIES.log; flag claims with no logged provenance). Flag copied protectable EXPRESSION.
Append a round section to /home/kamilner/tamarin-cleanroom/console/AUDIT.md. Violations get BEHAVIORAL redo instructions (never patches). End with exactly one line: VERDICT: pass  — or —  VERDICT: fail`

const impl = await agent(IMPL, { label: 'impl:D7', phase: 'Implement', model: 'opus' })
const audit = await agent(AUDIT, { label: 'audit:D7', phase: 'Audit', model: 'opus' })
return { impl: (impl || '').slice(0, 3000), audit: (audit || '').slice(0, 3000), passed: /VERDICT:\s*pass/i.test(audit || '') }