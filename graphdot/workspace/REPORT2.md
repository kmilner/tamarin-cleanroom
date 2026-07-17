# REPORT2 — the abbreviation SELECTION rule (BEHAVIOR.md §5c gap)

Clean-room continuation of `graphdot`. Everything below traces to observed
oracle output: the captured DOT corpus (`oracle/dot_corpus/`, 12 022 unique
payloads), the captured crawl manifests (`oracle/captured_responses/`, 81
theories; the `main/proof` json giving the full sequent paired 1:1 with each
`interactive-graph-def` DOT), and controlled probes of the black-box server
(`hs_server` binary, `--port=` form). No tamarin-prover source was read.

## Result in one line

> A sub-term `t` of a constraint system is abbreviated in the graph **iff**
> (1) its rendered length >= **10**, (2) it occurs >= **2** times in the system,
> and (3) it is not a tuple `<...>`. Naming is bottom-up (nested eligible
> sub-terms are named first).

The prior named puzzle -- why `sign(...)` at 3 occurrences is abbreviated while
`KDF(z)` at 4 is not -- is resolved: the gate is rendered **string length**, not
structural size or occurrence count. `sign(<...>,...)` is >=10 chars; `KDF(z)`
is 6. The earlier investigator measured *size* and *occurrence* and so could not
separate them.

## Evidence

### (1) Length >= 10 -- necessary, corpus-exact
Over **all 97 538** legend expansions in **11 564** graphs (every legend in the
corpus), the minimum fully-expanded rendered length is **exactly 10**; **zero**
are shorter. Histogram floor: len10=7397, len11=3071, len12=3776, ... nothing at
5-9. Length is measured on tamarin's own rendering *including* surface
decorations -- quotes, `~`/`$` sigils (a bare `pk(PL1)` is only 7 chars but is
abbreviated because it *expands* to >=10; the decision is on the full expansion).

Controlled probe (`LenTest.spthy`): a rule whose one node emits
`Keep(~x, '12345678', '12345678', '1234567', '1234567')`. Observed graph:
`Keep( ~x, 2, 2, '1234567', '1234567' )` with legend `2 = '12345678'`.
`'12345678'` = 10 chars -> abbreviated; `'1234567'` = 9 chars, **same occurrence
count** -> not. The boundary is exactly 10.

### (2) Occurrence >= 2 -- necessary, hard gate independent of length
Structural per-graph analysis of 5 725 graphs (156 602 distinct candidate
sub-terms): among terms drawn >=2 times, **every** abbreviated term satisfies
len>=10 AND not-tuple -- **0** counterexamples. The 7 800 abbreviated terms the
graph-only occurrence "misses" all have graph-occurrence 1: they occur once in
the *drawn* graph but >=2 in the full system (the sequent's goals/formulas). This
is why terms such as `SI3 = sign(~y, k)` appear in a legend while being drawn
**nowhere** in the graph -- the abbreviation universe is the whole constraint
system, not the drawn nodes. Pairing each DOT with its `main/proof` json sequent
recovers those second occurrences.

Controlled probe (`OccTest.spthy`): `'ABCDEFGH'` (10 chars, x1) and
`'onceonly12'` (12 chars, x1) are **not** abbreviated; `'twicelong1'` (12, x2)
**is** (`TW1`). Probe (`LongOnce.spthy`): a **42-char** constant appearing once
is **not** abbreviated, while a 12-char constant appearing twice is. So there is
no "very long => abbreviate at 1 occurrence" branch -- occ >= 2 is a hard gate.

### (3) Tuples are never abbreviated -- necessary, corpus-exact
**0 of 97 538** legend expansions is a top-level tuple. Only function
applications, exponentiations, AC-operator terms (`* (+) ++`), and atoms (long
constants/variables) are abbreviated. Controlled probe (`TupTest.spthy`):
`<'aa','bb','cc'>` (18 chars, x2) stays inline, while the sibling
`h('longarg123')` (x2) is abbreviated to `H1` with `LO1 = 'longarg123'`,
`H1 = h(LO1)` -- confirming the tuple exclusion and **bottom-up nesting** (inner
`'longarg123'` named before the enclosing `h(...)`), and the prefix rule
(`lo`->LO, `h`->H).

## Accuracy and the residual

The three gates are **necessary** with 0 corpus counterexamples (length, tuple)
/ 0 counterexamples among drawn->=2 terms (occurrence). For **sufficiency** --
is every (len>=10 AND occ>=2 AND not-tuple) term abbreviated? -- the controlled
probes say yes for ordinary terms, and the corpus agrees except for one
characterized residual:

* **93.3%** of apparent sufficiency-exceptions occur in graphs using an **AC/DH
  operator** (`^ * (+) ++ inv em hp pmult`). Example: in state `56eeb42d`,
  `em(hp($A), hp($B))` (18 chars, drawn 7x) is *not* abbreviated, yet in a later
  state `54eb2e2c` the *same* term *is* (`EM1`). The difference is how DH
  exponentiation groups the term: tamarin counts occurrences over the
  **normalised** (AC-flattened, DH-normal-form) term, which a surface count does
  not model. This is solver-normalisation-dependent and cannot be derived from
  the rendering alone.

* The remaining **~7%** are terms nested inside a *shared* abbreviated term
  (e.g. `pk(~ska.3)` inside one `sign(...)` threaded through an edge and drawn as
  `SI` in two facts). These are **measurement artifacts** of validating by
  *expanding* abbreviations before counting: expansion turns one edge-shared
  occurrence into several. The occurrence gate counts occurrences in the actual
  constraint system (an edge-shared message counts once), so these are not true
  rule violations -- a direct search finds **no** graph where such a term appears
  literally >=2x and is left un-abbreviated.

Net: outside AC/DH-normalised sub-terms the rule is exact (confirmed by
controlled probe and by the AC-free portion of the corpus); the one honest gap
is the AC/DH occurrence-over-normal-form count.

## What was implemented (`graph-clean`)

`abbrev::select(roots) -> Vec<Term>` implements the rule: enumerate every
non-tuple sub-term of `roots` with `render_len() >= 10`, keep those occurring
>= 2 times, return them shortest-first (bottom-up). Thresholds are exposed as
`MIN_ABBREV_LEN=10`, `MIN_ABBREV_OCC=2` and via `select_with(roots,min_len,
min_occ)` for probing. `roots` must be the full multiset of constraint-system
terms (graph node-fact arguments **and** sequent terms) because occurrence is a
whole-system count. Feeding the result through the existing `Abbreviator`
reproduces the legend end-to-end.

`term::Term` gained `is_tuple()` and `render_len()` (Unicode-scalar count of the
full rendering, matching the observed 10-char boundary).

Tests (`tests/abbrev_tests.rs`, all green) reproduce each controlled probe:
`select_length_boundary_is_ten`,
`select_requires_two_occurrences_even_for_long_terms`,
`select_never_abbreviates_a_tuple`, `select_then_name_reproduces_probe_legend`,
`select_counts_nested_occurrences`, `select_with_custom_thresholds`. Full suite
is 24 tests incl. the 12 022-payload byte-exact roundtrip.

## Minor open detail
A numeric-only constant (`'12345678'`, no alphabetic chars) gets an **empty
prefix**, so its abbreviation name is a bare number -- observed as `2` for the
first such entry. The starting index for empty-prefix names was not pinned down
(single observation); it does not affect the length/occurrence/tuple rule.
