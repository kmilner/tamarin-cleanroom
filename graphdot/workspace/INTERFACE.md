# graph-clean — adapter-facing width-input interface (round 12)

Audience: the OPEN-side integrator wiring an adapter later. Everything here is
implemented and test-gated in `workspace/graph-clean`; semantics trace to
BEHAVIOR.md §3f (Sessions 10–12).

## Round-11 status (unchanged): internal widths are NOT how the reference decides

Round-11 live probes (battery J, QUERIES.log Session 11) REFUTED the round-9/10
belief that the reference wraps cells on their UN-abbreviated internal widths:
abbreviated cells with internal widths 96–150 render FLAT when their display
text fits, and a sibling's budget follows the abbreviated display occupancy.
The reference lays out the POST-abbreviation display text everywhere probed.
Consequently an adapter should normally pass **no overrides at all** — the
display-text estimates are the probed reference behavior. The override surface
remains for callers that know better in exceptional cases, and every field
falls back per-field to the estimate, byte-identically.

## Surface

```rust
pub struct generate::CellWidths {
    pub occupancy:     Option<i64>, // row occupancy C (columns this cell counts
                                    // for in siblings' budgets + fill shares)
    pub bonus:         Option<i64>, // the cell's own pass-1 trigger SLACK
    pub fill_width:    Option<i64>, // fill numerator once the cell wraps
    pub trigger_width: Option<i64>, // effective self width in BOTH trigger
                                    // passes (incl. lone cells vs 87)
}
pub fn generate::group_widths_with(cells: &[String],
                                   overrides: &[Option<CellWidths>]) -> Vec<usize>;
// RawRule builders (one Option<CellWidths> per premise / conclusion cell):
RawRule::premise_widths(Vec<Option<CellWidths>>)
RawRule::conclusion_widths(Vec<Option<CellWidths>>)
```

## Semantics (precise, round-12 laws)

Per record group (premises, or conclusions; the info cell is its own group):

- **occupancy**: `C_i = flat_i + rec_sur_i + ftup_i` where `rec_sur` is the
  recursive tuple/union surcharge — each tuple/union node contributes
  `elems + 1`, except nodes directly inside another tuple/union which
  contribute `elems − 1` — and `ftup` counts function-application nodes
  INSIDE a tuple/union subtree (top-level funcs are uncharged);
- **trigger pass 1**: cell *i* wraps iff
  `eff_i > max(87 + slack_i − Σ_{j≠i} C_j, 20)` (lone cell: budget exactly
  87), `eff_i` = display flat (or `trigger_width`), `slack_i` = the largest
  `⌈elems/2⌉ − 1` over top-level tuple/union args in ANY position (mid-list
  included), capped at 4 — round-12 battery L; the round-10/11 last-gated
  `⌊elems/2⌋ + 2` bonus is refuted;
- **fill** (only if wrapping): ribbon
  `clamp(hd(87·N_i / (N_i + Σ_{j≠i} w_j·C_j)), 20, flat_i − 1)` rounded
  half-DOWN, `N_i = flat_i + rec_sur7_i + ftup_i` (per-node tuple surcharges
  capped at 7; NO top-level-func term, NO quote discount), `w_j = 5/6` for a
  single-quoted-atom sibling when cell *i* has a top-level tuple/union arg,
  else 1; the cell is laid out at lineLength `⌊3·ribbon/2⌋`, ribbonsPerLine
  1.5 (ragged HughesPJ fill; the tuple opener `<` may hang);
- **trigger pass 2 (relief)**: a pass-1-wrapping cell renders FLAT iff
  `eff_i ≤ max(87 − Σ_{j≠i} charge_j, 20)` where a pass-1-wrapping sibling
  charges `min(hd(q_j + bump_i), C_j)` on its UNROUNDED fill quotient `q_j`
  — `bump_i = 1/3`, dropped (0) when cell *i* carries a top-level
  tuple/union arg of ≥ 4 elements — and every flat sibling charges `C_j`;
  no slack in this comparison.

Overrides substitute per FIELD (`C_i` ← `occupancy`, `slack_i` ← `bonus`,
`N_i` ← `fill_width`, `eff_i` ← `trigger_width`); each absent field (or absent
entry, or a short/empty override slice) falls back to the display-text
estimate for that field. With every input absent the output is byte-identical
to `group_widths` — regression-gated by
`tests/generate_tests.rs::supplied_cell_widths_override_estimates`,
`::supplied_trigger_width_overrides_self_width`,
`::raw_rule_supplied_widths_reach_cells`, plus the corpus census.

## Known terminal residue (do not chase in an adapter)

Rows whose occupancies sum to exactly 88 (budget margin ±1..2) are decided by
the reference's coupled per-row `fits`; round-12 battery O proves no function
of the cell widths reproduces them all ([45,43] keeps the 43 flat, [46,42]
wraps the 42, etc.). The shipped model takes the best discrete tie-break;
~1,477 corpus cells (0.24 %) sit in this zone.
