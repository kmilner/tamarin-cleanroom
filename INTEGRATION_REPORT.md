# Dirty-room integration report — graph/dot + web clusters

Date: 2026-07-17. Integrator: dirty-room (adapters only; no logic transplanted
from replaced files into clean code). Repo: `/home/kamilner/tamarin-rs`.

Precedent followed: the wellformedness cluster (`crates/tamarin-parser/src/wf/`)
— clean sources vendored as an in-crate module with mechanical path fixes plus
a small workspace-authored adapter; clean files carry no license header, kept
ported files keep theirs.

--------------------------------------------------------------------------------
## 0. Vendoring (both clusters) — DONE

Clean deliverables copied verbatim into the server crate as in-crate modules,
with the single mechanical fix `crate::` -> `super::` (module path re-rooting)
and — in `graph_clean/mod.rs` only — the module-doc doctest fence changed
```` ``` ```` -> ```` ```ignore ```` (it referenced `graph_clean::…` as an
extern crate, which does not exist once vendored; the doctest is the clean
crate's own, not a workspace test). No license headers added — these are the
relicensable clean sources.

* `crates/tamarin-server/src/graph_clean/` <- `graph-clean/src/`
  (`mod.rs` <- `lib.rs`, `model.rs`, `term.rs`, `abbrev.rs`, `dot.rs`) — 837 LOC.
* `crates/tamarin-server/src/web_clean/` <- `web-clean/src/`
  (`mod.rs` <- `lib.rs`, `envelope`, `errors`, `escape`, `forms`, `intdot`,
  `page`, `proofscript`, `route`, `text`, `notfound_template`,
  `shell_template`) — 1005 LOC.
* Wired `pub mod graph_clean; pub mod web_clean;` into `lib.rs` (headered file;
  only the two `pub mod` lines + an explanatory comment were added — the kept
  file's own header was not touched).

Fidelity check: `sed 's/super::/crate::/' <vendored>` diffs byte-identical to
each original clean source (and `mod.rs` modulo the doctest fence). The clean
crates' own suites still pass at their workspace locations: `graph-clean` 16
round-trip + 7 abbrev + 1 doctest; `web-clean` parity suite. The vendored
`web_clean` inline unit tests (19) run and pass inside `tamarin-server`.

--------------------------------------------------------------------------------
## 1. WEB cluster — REWIRED (partial, byte-safe)

Adapters route prover-generated content as opaque pane/body strings into the
clean templates. All routed paths are byte-identical to the ported output for
the corpus/fixture theories, or a strictly-more-HS-faithful byte change that
the (structural) parity suite accepts.

REPLACED (ported render logic removed, now sourced from the clean layer):

* `handlers/theory.rs::intdot` — the ported `intdotLayout` `format!` template
  deleted; now `web_clean::intdot::render_intdot(&html_escape(name), &dotsrc)`.
  Byte-identical (the `<title>` prefix `Theory: ` has no escapable chars, so
  escaping `Theory: NAME` vs escaping `NAME` coincide).
* `handlers/theory_html.rs::overview_page` — for **local-origin** theories the
  whole page shell now comes from `web_clean::page::render_page`, with the
  west (proof-script state) and center (main-view HTML) panes passed as opaque
  strings (`"{pane} "` — the trailing space the ported template emitted). Byte-
  identical to the ported local output (verified head/nav/pane/tail against
  `shell_template`).
* `handlers/theory_html.rs` add/delete forms — `add_lemma_html` and
  `delete_lemma_html` deleted; `path_html` now calls
  `web_clean::forms::add_form` / `delete_form`. Byte change: the clean forms
  reproduce HS's stray `</span>` in the `<noscript>` block (byte-exact HS),
  which the ported code had normalized away. Structural suite unaffected.

REMAINS PORTED (kept, headers untouched) — with the precise blocker:

* `overview_page` **non-local** branch: `web_clean::page`'s shell bakes in the
  local-origin header (Reload-file / Append-modified-lemmas actions); a
  non-local (uploaded) origin gates those off, which the clean shell does not
  model. Routing it would add the two actions — a behavior change — so the
  ported template is kept for that branch only.
* `edit_lemma_html`: `web_clean::forms::edit_form` hard-codes the textarea
  `rows="8"`; HS sizes it dynamically (`textHeight = 2 + #newlines` in the
  lemma plaintext). Routing Edit through the clean form would drop that. Gap in
  `web_clean::forms` (a fixed `rows` slot); kept ported.
* `proof_state` (west pane) and the `path_html` main-view bodies (help /
  message / rules / sources / proof / tactic) stay in the workspace: these ARE
  the opaque prover content the clean shell consumes — the prover pretty-prints
  (via `tamarin_theory`) are data the adapter supplies, not scaffolding.
  `theory_html.rs` is therefore NOT deleted; it is now the pane/body supplier +
  the two kept branches above.
* JSON envelopes (`handlers/json_resp.rs`) and the 404 page: `web_clean::
  envelope` (`{html,title}` / `{redirect}`) and `web_clean::errors::
  render_not_found` are validated and byte-compatible on keys, BUT the ported
  `json_resp` returns `axum::Json<Value>` across ~40 call sites (and carries a
  third `{alert}` variant `web_clean` does not model), and `not_found_response`
  is a pervasive minimal-body stub with no request-path in scope. Rewiring
  either is pure return-type/plumbing churn with no behavior win under the
  structural gate; left for a follow-up (noted, not forced).

--------------------------------------------------------------------------------
## 2. GRAPH/DOT cluster — VENDORED; serializer rewire BLOCKED, reported

`graph_clean` is vendored, builds, and passes its round-trip corpus. The DOT
**pipeline rewire** (`handlers/dot.rs` `system_to_dot` / `system_to_dot_with`
/ `render_svg_or_dot_with` -> repr/simplify -> adapter -> graph_clean model ->
graph_clean abbrev -> graph_clean serialization) is **not** performed: it cannot
be a thin adapter without a behavior change, blocked by three concrete gaps
between `graph_clean`'s model and the payload the ported serializer produces.
Per protocol ("where the clean crate lacks a behavior, leave the old path in
place and report the gap; never patch clean code with old-code logic"), the
ported `handlers/dot.rs` and `graph/abbreviation.rs` are kept intact (headers
untouched). `graph/abbreviation.rs` is therefore NOT deleted.

Blockers (each independently fatal to a byte-faithful, thin-adapter rewire):

1. **Term rendering / abbreviation are one coupled behavior the clean crate
   deliberately omits.** `graph_clean` treats record-cell text as pre-rendered
   (BEHAVIOR.md §3a `[GAP]`: full solver term pretty-printer, incl. the
   `renderRow`/`renderBalanced` width-proportional record-row wrapping, is out
   of scope), and its `abbrev` names/substitutes over its OWN independent
   `term::Term` model. The ported cell text is rendered from `LNTerm` with the
   HS-faithful balanced wrapping AND abbreviations substituted in-place
   (`apply_abbreviations_fact`, `LNTerm`-keyed). `graph_clean::abbrev` cannot
   substitute a chosen name into `LNTerm`-rendered text; using it for selection
   would force routing all term rendering through `graph_clean::Term`, losing
   the balanced wrapping — exactly the behavior the protocol says to keep. So
   both the cell renderer and `graph/abbreviation.rs` must stay ported.

2. **No representation for missing-node shapes.** `graph_clean::model::NodeKind`
   is `Record | Ellipse | Plain`, and `Ellipse` emits `shape="ellipse"` only.
   HS "missing" nodes are `trapezium` / `invtrapezium` (`dotConcC`/`dotPremC`).
   A constraint system whose edges reference an absent node cannot be modeled;
   extending the model would be patching clean code (forbidden).

3. **HS node/port-id allocation is not modeled.** `graph_clean` takes node ids
   and record ports verbatim (its round-trip builds them from captured DOT);
   it does not generate HS's graph-global `<n0>,<n1>,…` port scheme (`Text.Dot`
   `cacheState`/`dsPrems`/`dsConcs`). Reproducing byte-exact HS ids requires
   that allocation logic, which lives in the ported serializer being "deleted"
   — writing it into the adapter would transplant replaced logic.

Consequence: a rewire that kept the ported cell renderer + abbreviation +
missing-node fallback + id scheme would GROW `handlers/dot.rs` (adapter on top
of a still-live serializer) rather than thin it, and would not be byte-faithful
— the opposite of the task's goal. Recommended: a `graph_clean` round-2 that
(a) adds arbitrary `shape=` support (trapezium), (b) exposes an HS port/node-id
allocator, and (c) either a `LNTerm`->`Term` bridge or an accepted term-render
GAP; then the thin-adapter rewire becomes possible. One isolated sub-component
IS cleanly routable today — the legend table via `graph_clean::abbrev::
legend_html` (byte-exact, incl. the 65-space hang indent the ported legend
omits) — but wiring only it into the otherwise-kept ported serializer mixes two
escaping/whitespace regimes for no net structural gain, so it is left for the
round-2 rewire.

--------------------------------------------------------------------------------
## 3. LOC delta

* Added (vendored clean sources, headerless): `graph_clean` 837 + `web_clean`
  1005 = **1842 LOC**.
* Adapter glue authored in kept files (headerless additions within headered
  files): ~35 LOC (theory.rs intdot call, overview_page local branch,
  path_html arms, lib.rs module decls).
* Deleted (GPL-headered ported render logic): `add_lemma_html` (~22) +
  `delete_lemma_html` (~21) + ported `intdot` `format!` template (~15) ≈ **58
  LOC**. Tracked-file diff: +82 / -70 across `theory.rs`, `theory_html.rs`,
  `lib.rs`.
* Not deleted (blocked/kept, reported above): `handlers/dot.rs` (2266),
  `graph/abbreviation.rs` (whole), `theory_html.rs` non-local + edit + pane
  suppliers.

--------------------------------------------------------------------------------
## 4. Validation (all green)

* `cargo build --workspace` — 0 errors.
* `cargo test -p tamarin-server` — lib 80 (incl. 19 vendored `web_clean`),
  routes_autoprove 6, routes_basic 19, routes_graph 4, routes_proof_step 3,
  routes_static 3, routes_stubs 15, routes_upload 3; doctest 1 ignored. 0
  failures. The captured-HS-response parity fixtures (routes_basic JSON-key +
  structural, routes_stubs) stay green.
* `cargo test -p tamarin-parser` — all green (67 + 2).
* Graph fixture tests (`graph-clean` crate): 16 round-trip + 7 abbrev + 1
  doctest, 0 failures. Vendored copy verified byte-identical modulo path fixes.
