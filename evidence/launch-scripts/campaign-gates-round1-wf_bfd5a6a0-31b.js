export const meta = {
  name: 'campaign-gates-round1',
  description: 'Similarity audits for units A-G deltas + unit F corpus-validation completion',
  phases: [{ title: 'Gates' }],
}
const AUD = `You are the SIMILARITY AUDITOR (both-sides reviewer) in the clean-room process — read
/home/kamilner/tamarin-cleanroom/PROTOCOL.md. Apply abstraction-filtration-comparison: identical
output strings / observed behavior / interface-doc names are NOT violations (compatibility content);
FLAG internal decomposition mirroring the Haskell where alternatives exist, non-observable internal
names matching Haskell internals, echoed comments, or algorithmic expression matching where the
observed behavior admits materially different implementations. Cross-check suspicious identical
strings against the cluster's QUERIES.log/BEHAVIOR.md — observed-output provenance filters them.
Append findings to the cluster's AUDIT.md under the specified heading (per finding: clean file:line,
Haskell file:line, why it survives filtration, a BEHAVIORAL redo instruction). Return a plain-text
verdict per cluster ("<name>: pass|redo") with one-paragraph justification and finding counts.
No structured output. Do not modify any other files.`

phase('Gates')
const [c, ab, defg, fval] = await parallel([
  () => agent(`${AUD}
CLUSTER: wellformedness round-3 delta ONLY (see workspace/BEHAVIOR.md "Round 3 closed and residual"):
the formula_terms term-based rewrite + formula_terms_reducible, the pp_formula_wrapped multi-line
formula printer, fact_capitalization, the two-mode guardedness detection, lemma-sourced fact
arity gather, fillSep-69 wrapping — in /home/kamilner/tamarin-cleanroom/wellformedness/workspace/wf-clean/src/.
HASKELL: /home/kamilner/tamarin-rs/tamarin-prover/lib/theory/src/Theory/Tools/Wellformedness.hs and
the formula/document printers it uses (Theory/Text/Pretty.hs, Theory/Model/Formula.hs pretty parts,
lib/utils HughesPJ-adjacent code). Heading: "## Round 3 incremental audit".`,
    { label: 'audit:C', phase: 'Gates', model: 'opus' }),

  () => agent(`${AUD}
CLUSTER 1: weblayer round-3 delta (src/dispatch.rs Server<ProverOps>, route.rs/forms.rs extensions,
textarea-rows rule) in /home/kamilner/tamarin-cleanroom/weblayer/workspace/web-clean/ vs HASKELL
/home/kamilner/tamarin-rs/tamarin-prover/src/Web/{Handler.hs,Theory.hs,Dispatch.hs,Settings.hs}.
Key check: the version-map/state-machine semantics are behavior (live-probed, see QUERIES.log
[L8]-[L16]); flag only expression-level mirroring of the Haskell handler internals.
CLUSTER 2: graphdot round-3 delta (src/{alloc,render,generate,options}.rs, invtrapezium support)
in /home/kamilner/tamarin-cleanroom/graphdot/workspace/graph-clean/ vs HASKELL
lib/theory/src/Theory/Constraint/System/{Dot.hs,GraphRepr.hs} and Graph/{Abbreviation,Simplification}.hs
(locate exact paths). Key check: the NodeIdAllocator and record-cell renderer were corpus-derived
(12022/12022 validations) — verify the Haskell's allocation/rendering EXPRESSION differs or is
output-forced. Headings: "## Round 3 incremental audit" in each cluster's AUDIT.md.`,
    { label: 'audit:A+B', phase: 'Gates', model: 'opus' }),

  () => agent(`${AUD}
Four NEW clusters, first audits (create AUDIT.md in each cluster dir, heading "## Round 1 audit"):
1. /home/kamilner/tamarin-cleanroom/console/workspace/cli-clean/ vs HASKELL src/Main/{Console.hs,
   Environment.hs} and src/Main.hs + the modes under src/Main/Mode/ (arg tables, help text assembly).
2. /home/kamilner/tamarin-cleanroom/macros/workspace/macro-clean/ vs HASKELL lib/term/src/Term/Macro.hs,
   lib/theory macro handling in TheoryObject.hs, and Theory/Text/Parser/Macro.hs.
3. /home/kamilner/tamarin-cleanroom/injective/workspace/injfacts-clean/ vs HASKELL
   lib/theory/src/Theory/Tools/InjectiveFactInstances.hs. Key: the clean decision rule was derived
   from live probes (QUERIES.log batches 1-6) — check whether the Haskell mechanism matches only in
   behavior or also in expression.
4. /home/kamilner/tamarin-cleanroom/derivcheck/workspace/derivcheck-clean/ vs HASKELL
   lib/theory/src/Theory/Tools/MessageDerivationChecks.hs. Key: the callback-trait architecture must
   NOT mirror the Haskell's probe-theory construction expression; verify probe payloads are
   behaviorally motivated.`,
    { label: 'audit:D-G', phase: 'Gates', model: 'opus' }),

  () => agent(`You are a CLEAN-ROOM implementer finishing unit F validation. Rules per
/home/kamilner/tamarin-cleanroom/PROTOCOL.md: never read anything under /home/kamilner/tamarin-rs/
except executing your oracle scripts and reading your cluster's oracle/ data; no web; log oracle
interactions in workspace/QUERIES.log. Cluster: /home/kamilner/tamarin-cleanroom/injective/.
Your prior session implemented injfacts-clean (32 tests green) and started a 49-theory corpus
validation (QUERIES.log entries 017-018) whose harness lived in a temp dir and is gone. Rebuild the
validation INSIDE workspace/validation/ this time (scripts + results persist): for each theory in
oracle/examples/, obtain the oracle's injective-facts line (--precompute-only or the method your
QUERIES.log describes), run your crate's decision on the theory's rules, and record per-theory
oracle-vs-clean agreement in workspace/validation/corpus_results.tsv. Investigate and document any
mismatch (fix the crate only if the oracle shows your rule wrong — log the probe). Finish with a
complete tally appended to workspace/BEHAVIOR.md and return a 10-line summary. No structured output.`,
    { label: 'F:corpus-validation', phase: 'Gates', model: 'opus' }),
])
return { c, ab, defg, fval }