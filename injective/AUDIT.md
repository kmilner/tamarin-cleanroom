# Similarity audit — injective / injfacts-clean

## Round 1 audit

Reviewer: similarity auditor (both-sides). Compared
`workspace/injfacts-clean/src/lib.rs` against HASKELL
`lib/theory/src/Theory/Tools/InjectiveFactInstances.hs`
(`simpleInjectiveFactInstances`). Key question from the brief: does the Haskell
mechanism match only in behavior, or also in expression? Cross-checked against
`workspace/QUERIES.log` (35 probes, batches 1–6) and `workspace/BEHAVIOR.md`.

**Findings: none (0). The match is behavioral only.**

- Decision rule. Both compute the same injectivity semantics (compatibility):
  (I) a linear tag occurring in both premises and conclusions of one rule, and
  (II) every net-new conclusion first-argument is a freshly `Fr`-generated name
  or carried from a premise. This behavioral agreement is REQUIRED (it is the
  observable `/main/rules` "Fact Symbols with Injective Instances" output) and is
  filtered out. The clean rule was derived from live probes (evidence grid in
  BEHAVIOR.md: p04/p05/b05/b06/d04 for (II), e02/e03/g01/g02 for the multiset
  counting), not from source.
- Expression/decomposition differs materially:
  - Condition (II) is formalized as a multiset difference `concKeys ⊖ premKeys`
    that must be a sub-multiset of `freshVars` (lib.rs:77–124, multiset
    consumption via `remove`). Haskell instead uses an explicit
    `duplicateFirstTerms` set plus a per-conclusion (a)-Fr-membership-then-(b)
    `getPrem` exactly-one check (InjectiveFactInstances.hs:180–228). Different
    accounting; the clean checks carried-premise before fresh, Haskell the reverse.
  - The clean unit returns only `BTreeSet<FactTag>` (name+arity). It does NOT
    reproduce the Haskell `MonotonicBehaviour` shape machinery
    (`combineShapes`/`combine`/`getBehaviour`/`elemNotBelowReducible`/tuple
    right-flattening, lines 100–223) — correct, because the web UI only surfaces
    the tag set and the per-position glyphs are cosmetic (BEHAVIOR.md §"Rendering").
    Omitting a large unobservable internal computation is strong evidence of
    behavioral, not expression, derivation.
  - Different data model (clean `Fact`/`Rule`/`Term`/`VarSpec` vs. `LNFact`/
    `factTag`/`viewTerm2`), different candidate construction (nested loops +
    `BTreeSet` vs. list-monad + `M.fromListWith`).

Verdict: pass.
