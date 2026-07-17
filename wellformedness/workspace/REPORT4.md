# Round 4 report — wellformedness clean-room (Unit C)

Scope: close the two gaps that kept `check_theory` from fully replacing
post-elaboration wellformedness checking. All behavior re-derived by probing
`oracle/wf_oracle.sh`; no tamarin-rs / Haskell source read. Round-4 probes are in
`scratchpad/probes4`, logged in `QUERIES.log`, spec'd in `BEHAVIOR.md`.

## GAP 1 — sort-aware quantifier binding

The wrong-form-terms check bound quantifiers by NAME only (round-2), which
over-binds. The reference distinguishes variable SORTS: a temporal binder `#x`
does not bind a message use `x` (`All #x. (A(x) @ #x) ==> (#x = #x)` ->
`Formula terms: `Free x'`), and a message binder `x` does not bind a node use
`#x` (-> `Free #x`). Fix: the de Bruijn binder stack now carries `(name, sort
class)` and matches on both. Sort classes collapse `{Untagged, Msg}` (which is
what the round-2 false positive actually needed) but keep Pub/Fresh/Node/Nat
distinct. De Bruijn indexing still counts every quantified var, so
`All x #x. ... h(x)` -> `h(Bound 1)` (probe g1_dbsort).

Probing the sort-annotation interaction surfaced a previously-unimplemented
topic, `Quantifier sorts`: quantifying over a public or fresh variable is
rejected (`Lemma `L' uses quantifiers with wrong sort: ("x",LSortPub)`); msg,
temporal and nat quantifiers are fine. It applies to lemmas and restrictions,
lists offending vars as Haskell `(name,LSort)` tuples in binding order with the
same width-69 fillSep wrap as Formula terms, and is emitted before Formula terms
within an item.

## GAP 2 — semantic guardedness

Replaced the round-3 heuristic (which recursed the guard through Or/Not/
quantifiers) with a decision procedure over the guarded fragment. Guard set of a
(sub)formula = the `(name, class)` keys of variables in ACTION atoms reachable
through conjunctions only; disjunction, negation, implication, `=`/`<`/`Mset`/
subterm/`last` atoms and nested quantifiers contribute no guards (probes
gx_disj_ant, gx_neg_ant, gx_less, gx_last, gx_quant_g, gx_eq_ant). A universal is
guarded only as `guard ==> rest`; any other body is "universal quantifier without
toplevel implication". An existential needs its variables covered by conjunctive
action guards, else "unguarded variable(s)". The first failing quantifier is
reported (antecedent before consequent, left before right — gr_both, gr_sib);
restrictions are lemma-only here (an unguarded restriction is a fatal error).

## Assembly and printer

- Per-item bundle. Quantifier sorts / Formula terms / Formula guardedness are
  emitted item-by-item (all lemmas in source order, then all restrictions), in
  that sub-order per item; consecutive same-topic entries merge under one header,
  non-consecutive ones become separate blocks (ord_qs_ft_qs -> QS,FT,QS as three
  blocks). `Lemma annotations` stays a separate global check after the bundle.
  New entry point `checks::formula_reports`.

- Col-relative formula printer. Round-3's `base+hang` was off by one at the top
  level (gt_and_two_all). The layout engine now hangs at `col + hang` (quantifier
  body `col+1`, binary/relational operands `col+0`), and relational atoms break
  at their operator with unparenthesised operands (gnest). Narrow formulas are
  byte-identical to the single-line printer, so all round-1..3 fixtures are
  unchanged.

## Tests

73 tests pass (was 53). 20 new byte-parity fixtures captured live from the oracle
(`tests/fixtures/g4_*.txt`) with hand-built ASTs: sort-aware binding (g1_core /
g1_msgnode / g1_dbsort), Quantifier sorts (pub, multi-var wrap, two lemmas,
restriction), per-item ordering (ord_qs_ft_qs, ord_la_ft), guardedness decision
(universal/existential disjunction, disjunction antecedent, ordering-atom
non-guard, partial coverage, recursion order, existential implication, mixed
conjunction), and the printer (top-level conjunction break, sibling nesting,
relational break + nested hang).

## Residual (documented in BEHAVIOR.md)

Formula-printer capture-avoidance (`$x.1` when a bound name collides with a free
name) is not reproduced; fixtures avoid collisions. Quantifier-head wrapping and
fact-argument wrapping are unprobed. Predicate atoms are expanded by the oracle
before the check, so they are not expected at the checker; guard extraction
treats them as non-guarding defensively.
