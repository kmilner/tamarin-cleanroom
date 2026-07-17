export const meta = {
  name: 'similarity-audit',
  description: 'Both-sides similarity audit of the three clean-room crates against the GPL Haskell originals',
  phases: [{ title: 'Audit' }],
}

const VERDICT = { type: 'object', properties: {
  verdict: { type: 'string', enum: ['pass', 'redo'] },
  findings_count: { type: 'integer' },
  top_findings: { type: 'array', items: { type: 'string' }, maxItems: 6 },
  audit_file: { type: 'string' } },
  required: ['verdict','findings_count','top_findings','audit_file'] }

const BRIEF = `You are the SIMILARITY AUDITOR in a clean-room process (see /home/kamilner/tamarin-cleanroom/PROTOCOL.md).
You MAY read both sides — that is your function. A clean-room agent implemented a crate purely from
black-box oracles; you now compare it against the GPL Haskell originals to catch expression that could
not plausibly have come from observation, including inadvertent reproduction from model training memory.

Apply abstraction-filtration-comparison discipline. NOT violations (filter these out): identical output
strings (warning texts, DOT/HTML/JSON syntax, templates) — these are compatibility content the clean
side copied from OBSERVED OUTPUT; identical behavior/algorithms at the idea level; API names given in
the provided interface docs; structure forced by the data model or the output format.
VIOLATIONS to flag: internal helper decomposition mirroring the Haskell where alternatives exist;
internal (non-API, non-output) names matching Haskell internals; comments echoing Haskell comments;
algorithmic EXPRESSION matching where the observed behavior admits materially different implementations;
any content that appears in the Haskell source but NOT in any observable output.

Write a detailed audit to the specified AUDIT.md (per-module notes, each finding with clean-side
file:line, Haskell file:line, and why it survives filtration), and return ONLY the small summary object.
For each finding include a REDO INSTRUCTION phrased behaviorally (what to reimplement differently),
never a code patch.`

phase('Audit')

const [wf, graph, web] = await parallel([
  () => agent(`${BRIEF}
CLEAN SIDE: /home/kamilner/tamarin-cleanroom/wellformedness/workspace/wf-clean/src/ (pretty.rs, report.rs, checks.rs, formula.rs, lib.rs — ignore ast.rs, it is the provided interface header).
HASKELL ORIGINALS: /home/kamilner/tamarin-rs/tamarin-prover/lib/theory/src/Theory/Tools/Wellformedness.hs (primary), plus the pretty-printing it uses (lib/theory/src/Theory/Text/Pretty.hs, lib/term/src/Term/Term.hs pretty parts) as needed.
Also secondary check: the replaced Rust module /home/kamilner/tamarin-rs/crates/tamarin-parser/src/wf.rs — flag only if the clean code matches ITS structure in ways the Haskell and the behavior do not explain.
Write audit to /home/kamilner/tamarin-cleanroom/wellformedness/AUDIT.md`,
    { label: 'audit:wf', phase: 'Audit', model: 'opus', schema: VERDICT }),

  () => agent(`${BRIEF}
CLEAN SIDE: /home/kamilner/tamarin-cleanroom/graphdot/workspace/graph-clean/src/ (model.rs, dot.rs, term.rs, abbrev.rs, lib.rs).
HASKELL ORIGINALS: /home/kamilner/tamarin-rs/tamarin-prover/lib/theory/src/Theory/Constraint/System/{Dot.hs,GraphRepr.hs,Graph/Abbreviation.hs,Graph/Simplification.hs} — locate exact paths with git ls-files in the submodule (Abbreviation/Simplification may live under Theory/Constraint/System/Graph/).
Secondary: /home/kamilner/tamarin-rs/crates/tamarin-server/src/graph/*.rs and handlers/dot.rs.
Write audit to /home/kamilner/tamarin-cleanroom/graphdot/AUDIT.md`,
    { label: 'audit:graph', phase: 'Audit', model: 'opus', schema: VERDICT }),

  () => agent(`${BRIEF}
CLEAN SIDE: /home/kamilner/tamarin-cleanroom/weblayer/workspace/web-clean/src/ (12 modules).
HASKELL ORIGINALS: /home/kamilner/tamarin-rs/tamarin-prover/src/Web/{Theory.hs,Handler.hs,Types.hs,Hamlet.hs,Dispatch.hs}.
Secondary: /home/kamilner/tamarin-rs/crates/tamarin-server/src/handlers/*.rs.
Note: the clean crate's shell_template.rs/notfound_template.rs contain byte-copies of OBSERVED page HTML — that is compatibility content, not a violation; check instead whether any template fragment exists in Hamlet source but never appears in served output.
Write audit to /home/kamilner/tamarin-cleanroom/weblayer/AUDIT.md`,
    { label: 'audit:web', phase: 'Audit', model: 'opus', schema: VERDICT }),
])

return { wf, graph, web }