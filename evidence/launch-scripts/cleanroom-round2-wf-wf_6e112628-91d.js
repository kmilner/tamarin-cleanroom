export const meta = {
  name: 'cleanroom-round2-wf',
  description: 'Clean-room wf round 2: close the six missing-topic gaps found by fixture testing',
  phases: [{ title: 'Redo' }],
}
phase('Redo')
const out = await agent(`You are a CLEAN-ROOM implementer continuing prior clean-room work. Read
/home/kamilner/tamarin-cleanroom/PROTOCOL.md and /home/kamilner/tamarin-cleanroom/wellformedness/SPEC.md
first and obey the access rules absolutely: never read anything under /home/kamilner/tamarin-rs/;
only execute oracle/wf_oracle.sh and read files under oracle/ and workspace/. No web access.
Do not reproduce tamarin-prover internals from training memory — observe instead.

Your prior implementation lives in workspace/wf-clean (BEHAVIOR.md and QUERIES.log document what you
learned). BEHAVIORAL BUG REPORT from integration testing — the oracle produces warning topics your
check_theory() does not:
1. "Left rule" and "Right rule" — appear for diff-mode theories. wf_oracle.sh now accepts extra
   arguments: run \`oracle/wf_oracle.sh <file> --diff\` for these. New observation inputs are in
   oracle/examples/round2/ (diff_left_right_mismatch.spthy, diff_right_rule_mismatch.spthy, plus
   diff corpus examples).
2. "Reserved prefixes" — see round2/diff_reserved_prefix.spthy (also diff mode).
3. "Lemma annotations" — round2/exists_trace_reuse.spthy triggers it. NOTE: your current
   implementation wrongly emits " Formula guardedness" and "Formula terms" for this input where the
   oracle emits neither — fix that false positive too (observe the oracle's actual output).
4. "Fresh public constants" — round2/fresh_public_constant.spthy.
5. "Multiplication restriction of rules" — round2/multiplication_in_rule_lhs.spthy.

For each: probe the oracle (mutate the inputs to map trigger conditions and exact message bytes),
determine where each topic sits in the canonical report order, extend the AST-model handling as
needed (the interface/ast_types.rs model includes diff/annotation fields — study it), implement the
checks, add byte-parity fixtures to your test suite, update BEHAVIOR.md and QUERIES.log, keep
cargo test green. Write workspace/REPORT2.md (topics closed, new tests, remaining gaps) and return
a 10-line plain-text summary. Do NOT use structured output.`,
  { label: 'cleanroom:wf-round2', phase: 'Redo', model: 'opus' })
return { out }