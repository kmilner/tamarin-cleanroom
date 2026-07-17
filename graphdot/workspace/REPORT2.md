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

================================================================================

# REPORT — Round 6: the record-cell wrap TRIGGER (BEHAVIOR.md §3f gap)

Clean-room continuation of `graphdot`. Everything below traces to observed oracle
output: the captured DOT corpus (`oracle/dot_corpus/`, 12 022 payloads / 160 409
records) and controlled probes of the black-box interactive server (autoprove →
`interactive-graph-def`). No tamarin-prover source was read.

## Result in one line

> The old per-cell rule ("a cell wraps iff its own flat width > 87") is wrong for
> multi-cell record groups. Each record group (premises, info, conclusions) is laid
> out **independently**, and within a group the cells **share** the fill width:
> a cell wraps **iff** `group_total_flat > 87` **AND** `cell_flat > 20`
> — equivalently `cell_flat > max(87 − Σ(other cell flats in the group), 20)`.
> The `#t : Rule[…]` info cell is its own single-cell group (budget 87) with an
> extra rule: an info list of **≥ 2 action facts always goes vertical**, whatever
> its width.

The prior puzzle — first-line widths spanning 19..277 with "no fixed page width" —
is resolved: 87 is a *per-group* budget shared across cells, so a cell deep in a
wide group wraps far below 87 while a lone cell tolerates 87. The In-vs-Big pair in
`ref_raw_1_1.dot` is the crisp witness: `In(<x1.4..x10.4>)` flat 67 (premise, alone
with Fr) does NOT wrap, while the byte-identical `Big(<x1.4..x10.4>)` flat 68
(conclusion, sharing with Ack+Out) DOES.

## Evidence

### (1) Groups are independent — controlled probes
- Growing premises (In flat 69/103/198) or the rule name (len 40) leaves a lone
  conclusion `Out` (flat 87) flat; adding ONE preceding conclusion (flat 12) makes
  it wrap. ⇒ a conclusion's budget counts only other conclusions.

### (2) Shared budget = 87 − Σ(others), floor 20 — step-1 sweeps
- 2-cell group [Fa(p), Fb(q)]: with q fixed, Fa's fit/wrap boundary is exactly
  `flat = 87 − q_flat` (q=11 → 76/77, q=28 → 58/61, q=48 → 39/40, q=68 → 19/22).
- 3-cell [p,28,28] flips at `flat = 87 − 56 = 31` ⇒ Σ is over all other cells.
- Floor: a sibling forced far past budget still leaves the target fitting at flat
  ≤ 20, wrapping at 21 ⇒ per-cell minimum budget 20.
- Algebra: `flat > max(87−Σothers, 20)` ⟺ `(Σall > 87) ∧ (flat > 20)`.

### (3) Info cell — separate probes
- A single-action info wraps by width at flat > 87 (independent of prem/concl).
- An info with ≥ 2 actions ALWAYS wraps vertically (TwoShort flat 34, AB flat 22).
  Corpus census: **0** non-wrapped info cells carry a top-level action-list comma.

### (4) Corpus-exactness
- The rule matches actual `\l` on **99.635 %** of 776 259 cells and **98.324 %** of
  160 409 records (info cells 99.968 %). Hand-verified on the Wide record: In (77,
  fits), Big (budget 48, wraps), Ack (budget 20, wraps), Out(h) (14, fits) — all
  four correct; and on the 2-action info cells of `b5a8773`/`001113` (both wrap).

## Accuracy and the residual (honest gaps)

Every one of the 2 831 residual cells sits within ~1–2 columns of its budget
(mismatch group-totals cluster entirely at 79 ≤ T ≤ 95). At that exact boundary the
outcome is a **±1 rounding artifact of tamarin's HughesPJ `fits`**: a live split
sweep at group-total = `budget + 1` flips fit/wrap with atom parity, while at
`budget + 5` it wraps cleanly. A second residual: the exact greedy fill **width**
when a *multi-cell* cell wraps is not a clean function of the budget (probeB: a W
tuple keeps a 9-element first line for two different Pad widths / budgets), so the
per-line element count of a multi-cell wrap is not byte-exact. Both need the GPL
pretty-printer's internal `fits`; the **trigger** (does a cell wrap) is byte-exact
away from the ±1 boundary.

## What was implemented (`graph-clean`)

- `render::cell_budget(flats, i) = max(87 − Σ_{j≠i} flats, 20)` and
  `render::count_info_actions`; `MIN_CELL_BUDGET = 20`.
- `render::wrap_cell_budget(flat, budget)` threads the group budget through the
  fill/peel machinery (the old `wrap_cell` = `wrap_cell_budget(flat, 87)`, so all
  single-cell fixtures are unchanged). Info cells force the vertical action `sep`
  when ≥ 2 actions. A **fact-argument** break now ends the line at the bare `,`
  (byte-exact vs `Init_1(...)`/Ack), while a **tuple-element** break keeps the `, `.
- `generate::build_record` computes per-group budgets (premises together,
  conclusions together, info alone) and wraps each cell with its budget.
- Tests: `cell_budget_shares_the_group_width`, `group_trigger_matches_wide_record`,
  `multi_arg_fact_break_drops_the_comma_space`, `count_info_actions_counts_top_level`,
  `info_two_actions_always_vertical_even_when_short`. Full suite green; the
  GRAPHCLEAN_CORPUS round-trip stays **12 022/12 022** byte-exact.

## Opportunistic: role → color (characterized, not implemented)

Corpus mining (15 non-`Undefined` roles): the record `fillcolor` is a **deterministic
function of the role name** (role `A` → `#804046` in all 3321 instances, `B` →
`#628040`, `Tag` → `#40807c`, `Process` → `#ffffff`, …), and the cluster ARGB is a
deterministic function of the role too (`A_Session_k` → `#D836744C` for every k,
`B` → `#3671D84C`, `Initiator` → `#4936D84C`) — the session index does not affect
color. So the color IS name-determined (a necessary condition for derivability), and
the values look like a name-hash → hue → HSV rendering; but the node hue and cluster
hue differ per role (two distinct derivations), so the exact hash is unresolved.
Reverse-engineering it needs a dedicated live campaign of crafted novel role names
(like §5b's per-prefix numbering) — left as a documented open item; the crate keeps
colors as caller inputs.
