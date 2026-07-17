# Round 2 report — wellformedness clean-room

Scope: close the five (six) warning topics the oracle emits that `check_theory()`
did not, plus the reported `exists_trace_reuse` false positive. All behavior was
re-derived by probing `oracle/wf_oracle.sh` (diff cases via `--diff`); no
tamarin-rs source was read.

## Topics closed (byte-parity fixtures added)

| Topic | Mode | Trigger | Report position |
|-------|------|---------|-----------------|
| `Fresh public constants` | both | `~'foo'` literal in a rule | #2 (after Unbound, before public-names) |
| `Reserved prefixes` | diff only | fact name starts `DiffIntr`/`DiffProto` | after `Reserved names` |
| `Left rule` / `Right rule` | diff only | explicit `left`/`right` != parent projection | after lhs-not-rhs, before Formula terms |
| `Lemma annotations` | both | `[reuse]` + `exists-trace` lemma | after Formula guardedness |
| `Multiplication restriction of rules` | both | `*` term in a rule's conclusions | after Lemma annotations, before Nat Sorts |

Left rule and Right rule are one check (`diff_left_right`): the left projection is
tested first and, if inconsistent, the right is not reported for that rule
(observed via `diff_both`). All three diff topics are gated on `theory.is_diff`
(silent otherwise — confirmed by running the diff fixtures without `--diff`).

## False positive fixed

`round2/exists_trace_reuse` made the old code emit `Formula terms` +
` Formula guardedness`. Root cause: formula free-variable / guardedness matching
keyed on `(name, sort)`, so a quantified message var and its occurrence tagged
with different-but-compatible sorts (Msg vs the Untagged default; `@ i` vs `#i`)
looked distinct. Fix: bind and compare formula variables by NAME only. The
oracle emits only `Lemma annotations` for this input; `check_theory` now matches.
Prior fixtures (p05, p21, f_subterm) use consistent sorts and are unaffected.

## New tests (all green: 37 + 1)

Six byte-parity fixtures captured live from the oracle
(`tests/fixtures/r2_*.txt`) with matching hand-built ASTs:
`lemma_annotations_exists_trace_reuse`, `fresh_public_constant_literal`,
`multiplication_restriction_in_conclusion`, `diff_left_rule_inconsistent`,
`diff_right_rule_inconsistent`, `reserved_prefix_diff_only`. Plus
`diff_left_right_consistent_is_silent`, two ordering tests
(`fresh_public_constants_before_public_names`,
`lemma_annotations_between_guardedness_and_nat`), a non-diff silence assertion,
and a topics-set regression guard on the false positive. All six fixtures were
re-verified against the live oracle byte-for-byte.

## New infrastructure

`pretty::pp_rule` / `pp_fact_list` (rule printer: `rule (modulo E) N:\n   [..] <arrow> [..]`);
`checks::fill_words` (Wadler `fillSep` at measured width 69 for the Reserved
prefixes header); `indent_block`; diff term/fact projection.

## Remaining gaps (documented in BEHAVIOR.md)

- Rule-printer line wrapping for long rules (arrow drops to its own line) — fixtures use short rules.
- Multiplication: the "After replacing reducible symbols in lhs" rule is rendered
  equal to the original (correct only when the LHS has no reducible symbols); the
  alternate `Variables that occur only in rhs` failure mode is not implemented;
  the co-emitted Maude `Message Derivation Checks` block stays out of scope.
- Reserved prefixes: header word-wrap is reproduced (width 69), but the
  `(ProtoFact ...)` line is a raw Haskell tuple Show and multi-rule joining is
  unprobed; fresh-public-constant list wrapping (4-space continuation) is not
  reproduced — fixtures keep lists short.
- Diff mode merges `Fact arity`/`Fact multiplicity` into a single `Fact usage`
  topic (observed); `check_theory` still emits the two non-diff topics in diff
  mode. Out of the requested scope; noted for a future round.
