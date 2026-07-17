# Similarity audit — derivcheck / derivcheck-clean

## Round 1 audit

Reviewer: similarity auditor (both-sides). Compared
`workspace/derivcheck-clean/src/lib.rs` against HASKELL
`lib/theory/src/Theory/Tools/MessageDerivationChecks.hs`
(`checkVariableDeducability` and helpers). Key question from the brief: the
callback-trait architecture must NOT mirror the Haskell probe-theory
construction; probe payloads must be behaviorally motivated. Cross-checked
against `workspace/QUERIES.log` and `workspace/BEHAVIOR.md`.

**Findings: none (0). The callback trait does not mirror the probe-theory
construction.**

- Probe architecture. Haskell decides derivability by CONSTRUCTING an auxiliary
  provable theory per rule: strip rules/lemmas/restrictions, make funs public
  (`deleteRulesAndLemmasAndRestrictionsFromTheory`, `makeFunsPublic`), turn
  premises into `Out` facts (`premisesToOut`), synthesize a rule
  (`generateRule`) and one exists-trace `KU` lemma per free var
  (`generateSeparatedLemmas`), close with maude and prove
  (MessageDerivationChecks.hs:36–47, 181–221). The clean unit reproduces NONE of
  this: it defines a `DerivabilitySolver` trait and hands the solver a
  `DerivProbe { rule_name, rule, variable, premises, timeout_secs }` (lib.rs:56–79)
  — the direct restatement of the observable check ("variables … not derivable
  from their premises", BEHAVIOR.md §1/§5). The payload carries the abstract
  question, not the Haskell's Out-fact/KU-lemma encoding; the derivability oracle
  is explicitly delegated to the caller and not reimplemented. Behaviorally
  motivated → survives the mirror charge as a genuinely different design.
- Report text and ordering. The identical strings (topic `Message Derivation
  Checks`, 25-`=` underline, 2-space intro paragraph, `"Rule <name>: \n"` with
  trailing space, `"Failed to derive Variable(s): "`) are verbatim observed
  output (BEHAVIOR.md §3 cat -A captures; QUERIES.log entry 3) — compatibility
  content, filtered out. The sort key `(sort_rank Fresh<Msg, name lexicographic,
  idx numeric)` was probed (o1/o2/s1/s2) and the clean re-sorts explicitly, where
  Haskell relies on `frees` order — different mechanism.
- Report model. Clean emits multiple `WfError` sharing a topic, joined at render
  (lib.rs:180–186, 332–346); Haskell emits a single concatenated-body tuple
  (reportVars:122–138). Different decomposition.
- Timeout policy (`TimeoutPolicy`, `timeout_secs==0` deactivation) is the clean's
  own design over an unobserved residual (BEHAVIOR.md §7), not a Haskell mirror.

Verdict: pass.
