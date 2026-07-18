# graph-clean — adapter-facing width-input interface (round 10)

Audience: the OPEN-side integrator wiring an adapter later. Everything here is
implemented and test-gated in `workspace/graph-clean`; semantics trace to
BEHAVIOR.md §3f (Session 10).

## Why this exists

The reference decides record-row wrapping (which cells break, and how wide a
broken cell's fill is) on the *internal* term widths — notably the
UN-abbreviATED renderings — while this crate consumes post-abbreviation display
text. The crate's estimates reproduce 96.6 % of corpus cells; the dominant
residual is exactly this internal-width family. An adapter that sits on the
term representation can close it by supplying the values directly.

## Surface

```rust
pub struct generate::CellWidths {
    pub occupancy:  Option<i64>, // row occupancy C (columns this cell counts
                                 // for in its SIBLINGS' trigger budgets)
    pub bonus:      Option<i64>, // the cell's own trigger-budget bonus
    pub fill_width: Option<i64>, // fill numerator (internal width) once the
                                 // cell wraps
}
pub fn generate::group_widths_with(cells: &[String],
                                   overrides: &[Option<CellWidths>]) -> Vec<usize>;
// RawRule builders (one Option<CellWidths> per premise / conclusion cell):
RawRule::premise_widths(Vec<Option<CellWidths>>)
RawRule::conclusion_widths(Vec<Option<CellWidths>>)
```

## Semantics (precise)

Per record group (premises, or conclusions; the info cell is its own group):

- trigger: cell *i* wraps iff `flat_i > max(87 + bonus_i − Σ_{j≠i} C_j, 20)`
  (lone cell: budget exactly 87, no bonus);
- fill (only if wrapping): ribbon
  `clamp(round(87·N_i / (N_i + Σ_{j≠i} w_j·C_j)), 20, flat_i − 1)`,
  `w_j = 5/6` for a single-quoted-atom sibling when cell *i* has a top-level
  tuple argument, else 1; the cell is laid out at lineLength `⌊3·ribbon/2⌋`,
  ribbonsPerLine 1.5 (ragged HughesPJ fill).

Overrides substitute per FIELD: `C_i` ← `occupancy`, `bonus_i` ← `bonus`,
`N_i` ← `fill_width`; each absent field (or absent entry, or a short/empty
override slice) falls back to the display-text estimate for that field:

- `C_i` default: `flat_i + Σ_{top-level tuple/union args}(elems + 1)`;
- `bonus_i` default: max over those args of `⌊elems/2⌋ + 2` (an arg with ≥ 9
  elements contributes 4); 0 with no such arg;
- `N_i` default: `flat_i + Σ_{union args}(elems + 1) + #function-nodes`.

The display flat width `flat_i` always comes from the cell text (it IS the
rendered content). With every input absent the output is byte-identical to
`group_widths` — regression-gated by
`tests/generate_tests.rs::supplied_cell_widths_override_estimates` and
`::raw_rule_supplied_widths_reach_cells`, plus the corpus census.

## What an adapter should pass

For each premise/conclusion cell, compute on the UN-abbreviATED internal term:

- `occupancy`: the internal flat width (with tuple/union args measured in
  their internal form — the display-side law `+ (elems+1)` / spaced `++`
  separators approximates exactly this);
- `fill_width`: the same internal width used as the cell's fill numerator;
- `bonus`: only if the adapter can compute the reference's own-budget bonus;
  otherwise leave `None` (the display-text law is probe-exact on
  un-abbreviated shapes).

Passing only `occupancy` (the biggest lever — it fixes the sibling budgets) is
valid; fields are independent.
