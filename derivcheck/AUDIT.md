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

## Round 4 audit (gap-closing delta)

Reviewer: both-sides similarity auditor (exempt from clean-room access). Scope:
ONLY this round's working-tree delta against clean-room HEAD `63ed8a9`, restricted
to `derivcheck/` (`git status`/`diff`): `workspace/BEHAVIOR.md` (+§3a,§5a,§9,§10;
rewritten §5/§6), `workspace/QUERIES.log` (+ROUND 4 block), `workspace/REPORT.md`
(+Round 4 section), `workspace/derivcheck-clean/src/lib.rs` (752-line delta), plus
untracked `workspace/probes4/*.spthy` (26 probe inputs) and
`workspace/derivcheck-clean/examples/dump_two_rule.rs`. Compared against Haskell
`lib/theory/src/Theory/Tools/MessageDerivationChecks.hs`,
`Theory/Tools/Wellformedness.hs` (`prettyWfErrorReport`, `underlineTopic`),
`Term/LTerm.hs` (`Ord LVar`, `Show LVar`, `data LSort`), `Term/Macro.hs`
(`applyMacros`), `Theory/Model/Rule.hs` / `ClosedTheory.hs` (`applyMacroInRule`,
`applyMacroInProtoRule`). The five gaps (GAP1..5): output contract, variable
ordering, candidate scope, macro/let pre-expansion, batched solver interface.

**Findings: none (0). No copied protectable expression; every behavioral claim in
the delta traces to a logged probe or captured fixture.** Evidence verified:
`cargo test` → 28 passed; `examples/dump_two_rule` output is byte-identical to
`probes4/fixture_two_rule.txt` (297 bytes, `cmp`, trailing NL aside); the g2/g3/g4
probe `.spthy` files exist and match their QUERIES.log ROUND 4 entries.

- **GAP2 ordering — inevitable convergence, NOT a copy.** The corrected key
  `(idx, sort_rank, name)` in `var_order_key` coincides exactly with Haskell
  `instance Ord LVar` (LTerm.hs:522-524: `compare x3 y3 <> compare x2 y2 <>
  compare x1 y1` = idx·sort·name, carrying the source comment "prefers the
  lvarIdx over the lvarName"). This is FILTERED: the variable order is directly
  observable in the warning text, so byte-compatibility forces this exact order
  (merger). The clean unit re-derived it from discriminating black-box probes —
  `g2_freshidx` (`In(h(~a.2)),In(h(b))` → `b, ~a.2`, idx outranks sort),
  `g2_strong`, `g2_case` (ASCII, `Z`<`a`), `g2_nodot`, `g2_mixed`, `g2_cross` —
  each a real file matching QUERIES.log. Positive independence signals: the
  Haskell comment is NOT reproduced; `sort_rank` places `Pub` LAST (rank 3)
  whereas Haskell `LSort` derives Ord with `LSortPub` FIRST; the clean `SortHint`
  enum order (Msg,Pub,Fresh,Node,Nat) differs from `LSort`
  (Pub,Fresh,Msg,Node,Nat) and `sort_rank` is written explicitly rather than
  leaning on a discriminant. `Fresh<Msg<Nat` and the `~`/`%`/`$` prefixes
  (matching `Show LVar` `sortPrefix`) are observable (`g3_nat*`, render tests).

- **GAP4 macro expansion — standard algorithm, wholly different names.**
  `expand_term` follows the textbook call-by-value macro rule (expand args →
  substitute into body → re-expand result), structurally parallel to
  `Term.Macro.applyMacros` (Macro.hs:40-54) and matching by name+arity as
  `macroToFunSym` does. This is merger / scenes-a-faire (there is essentially one
  correct way to expand nested macros); identifiers share nothing with the
  Haskell (`expand_term`/`subst_vars`/`param_map`/`macro_table` vs
  `applyMacros`/`apply`/`substFromList`/`findMatchingMacro`), and no
  tamarin-internal name leaks. The clean unit additionally expands `let` bindings
  (`let_substitution`,`bind_pattern`) — which the Haskell does NOT do in this
  file (tamarin pre-expands `let` at parse; `applyMacroInRule` touches only
  macros). That is the clean parser's own design (its AST retains `let_block`),
  behaviorally motivated by `g4_let`/`g4_let_rename`/`g4_macro`/`g4_macro_rename`/
  `g4_let_twice`. Different mechanism, probe-traced.

- **GAP3 candidate scope — same observable set, different exclusion mechanism.**
  Free vars of premises+actions+conclusions equals Haskell `freesInThyRules`
  (`frees` over the whole `oprRuleE`, minus `LSortNode`). The clean unit excludes
  temporal/node vars by observing they are a PARSE error in message-term position
  (`g3_temporal`) rather than post-filtering `LSortNode`; public is not
  pre-filtered but resolved by the solver (`g3_pub`); nullary functions parse to
  `App(name,[])` contributing no var (`g3_nullary` vs `g3_nullary_ctrl`);
  action-/conclusion-only vars are candidates (`g3_act`,`g3_concl`). All
  probe-traced; the reimplemented decision logic is the clean's, the derivability
  oracle stays delegated.

- **GAP5 batched solver — SPEC-sanctioned interface, oracle still delegated.**
  `DerivabilitySolver::check_rule(&RuleProbe) -> Vec<Derivability>` (one call per
  rule, one verdict per candidate) mirrors the Haskell decomposition granularity
  (one modified theory closed/proved per rule; one separated KU lemma per free
  var → `[ProofStatus]` per rule). Reviewed and cleared: this is an
  efficiency-merger interface shape, documented as an interop fact (BEHAVIOR §10)
  within the SPEC's explicit latitude ("design a small trait … with whatever
  probe payload your observed semantics need"). Critically, NONE of the
  probe-theory construction is reproduced — no `makeFunsPublic`, `premisesToOut`,
  `generateRule`, `generateSeparatedLemmas`, KU-lemma synthesis, or
  `deleteRulesAndLemmasAndRestrictionsFromTheory`; the Haskell has no callback at
  all, so no expression is copied. The `PerVariable`/`PerVariableSolver` adapter
  is the clean's own convenience.

- **GAP1 output contract — fixture-traced bytes, different decomposition.**
  Emitting the whole topic as one `WfError` whose `message` bakes in the heading +
  25-`=` underline + 2-space-indented intro + rule blocks reproduces the Haskell
  render quirk (Wellformedness.hs:118-125 `prettyWfErrorReport` = `vcat .
  intersperse (text "") . map ppTopic . groupOn fst`, `ppTopic = text topic $-$
  nest 2 (…)`; the `nest 2` indents only the intro's first line because the body
  is one `text` with embedded `\n`). This is captured byte-exact in
  `probes4/fixture_two_rule.txt` and reproduced verbatim (verified 297-byte
  identity). The heading-in-`message` bundling is the clean's own inter-unit
  contract (its renderer adds no heading and just joins blocks with one blank
  line) — the OPPOSITE decomposition from Haskell (which keeps the topic Doc
  separate and applies the nest at render). Whether that contract holds globally
  is the wellformedness/integration unit's concern, not derivcheck's.

- **Fixture reconstruction.** `examples/dump_two_rule.rs` rebuilds two POIDC_CMB
  rules (`sign`/`raenc`; vars `sk1,r1,m,r2,sk2,x,rndA,pkA`) purely to emit the
  byte-exact block. Those names are probe-STIMULUS data from
  `oracle/examples/thesis-SvenHammann-POIDC/POIDC_CMB.spthy` (a probe input the
  clean room legitimately holds); the flagged subset (`m,r2,sk2,pkA`) and rule
  names are the observable output. No Haskell source expression involved.

Redo required: none.

Verdict: pass.
