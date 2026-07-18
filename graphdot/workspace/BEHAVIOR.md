# BEHAVIOR.md — observed spec of the tamarin web-UI constraint-system graph payload

All statements below are derived from OBSERVED oracle output: the 81 captured
crawl manifests under `oracle/captured_responses/` (14705 graph payloads) and
targeted live probing of the black-box HS binary (see QUERIES.log). No
tamarin-prover source was read. Where a behavior could not be pinned down
black-box it is marked **[GAP]**.

--------------------------------------------------------------------------------
## 1. Transport / payload family

A crawl manifest is JSON: `{base, lemmas, log, capped, manifest}` where
`manifest` maps a URL to `{kind, status, body}` (kind ∈ html|text|json|dot).

Graph-bearing URL families (theory index `N` replaces `#` in captures):

| URL family | kind | payload |
|---|---|---|
| `/thy/trace/N/interactive-graph-def/proof/<lemma>[/<step>...]` | `dot` | **the graph** — graphviz DOT text (`text/plain`) |
| `/thy/trace/N/intdot/proof/<lemma>[/<step>...]` | `html` | wrapper page embedding `<dot-graph-viz dotsrc="…/interactive-graph-def/…">` |
| `/thy/trace/N/main/cases/{raw,refined}/i/j` | `json` | `{html,title}`; html contains `<static-graph graphSrc="…/intdot/cases/raw/i/j">` |

The DOT text at `interactive-graph-def` is the whole deliverable. `intdot` is a
fixed HTML shell (only the `<title>` and the `dotsrc` path vary). The graph is
rendered client-side from the DOT by `/static/js/intdot-graph.es.js`.

**JSON graph payload** [GAP]: the binary exposes a *separate* `--with-json`
rendering backend (`tamarin-prover interactive --help`). No JSON rendering tool
is installed here and no `interactive-graph-def` body in the corpus is JSON —
the entire corpus is the DOT backend. JSON graph format is therefore unobserved.

**Graph options** [RESOLVED §8]: the graph URL *does* take query parameters, but
they are cookie-driven and added by the UI JS — `simplification=N` (level, default
2, always sent), plus the flags `uncompact=`/`uncompress=` (sent at level 0),
`unabbreviate=`, `no-auto-sources=`, `clustering=true`. The earlier
"byte-identical" observation was because varying only `simplification=1/2/3` does
not change output on any probed graph (see §8). The real transforms are triggered
by the boolean flags. See §8.

--------------------------------------------------------------------------------
## 2. DOT document structure

Two headers occur, selected by a single rule (§4):

### 2a. Simple header (non-clustered)
```
digraph "G" {
nodesep="0.3";
ranksep="0.3";
node[fontsize="8",fontname="Helvetica",width="0.3",height="0.2"];
edge[fontsize="8",fontname="Helvetica"];
<statements…>

}
```

### 2b. Compact header (clustered)
```
digraph "G" {
nodesep="0.8";
ranksep="0.8";
sep="4";
splines="true";
overlap="false";
pack="true";
packmode="cluster";
concentrate="true";
compound="true";
remincross="true";
mclimit="10";
nslimit="20";
nslimit1="20";
ordering="out";
rankdir="TB";
showboxes="false";
clusterrank="local";
node[fontsize="8",fontname="Helvetica",width="0.3",height="0.2",margin="0.05,0.05",shape="ellipse"];
edge[fontsize="8",fontname="Helvetica",penwidth="1.5",arrowsize="0.5",color="black",style="solid",weight="8"];
<statements…>

}
```

### Block / whitespace rule (byte-exact)
Every block (`digraph … {`, `subgraph … {`, anonymous `{`) is emitted as:
`OPEN "\n"` then each inner statement as `STMT "\n"`, then one blank line, then
`}`. i.e. the body always ends `…\n\n}`. The top-level digraph is followed by a
final `\n`. Subgraph/anonymous blocks are themselves statements of their parent
(so the parent supplies the `\n` after their closing `}`). There are **no** blank
lines between sections (header attrs / nodes / edges / subgraphs run together);
the only blank line in each block is the one before its own `}`.
An empty graph is header attrs + blank + `}` (154 bytes).

--------------------------------------------------------------------------------
## 3. Node & edge syntax

Only three node shapes appear in proof graphs: `record`, `ellipse`, `plain`.

### 3a. Record node (a rule / graph-node instance)
```
n<ID>[shape="record",label="{{<p> cell|<p> cell}|{<p> cell}|{<p> cell|<p> cell}}",fillcolor="#RRGGBB",style="filled",fontcolor="black|white",role="<Role>"];
```
- Attribute order is fixed: `shape,label,fillcolor,style,fontcolor,role`.
- Label is a graphviz record: three top groups `{prem}|{info}|{concl}` (a group
  may be absent/empty). Each group = cells joined by `|`. Each cell =
  `<portid> text` (port id like `n0`, a space, then the fact text). Ports are
  numbered per-graph; a record's own node id is allocated AFTER its port ids.
- Record-label text escaping: `<`→`\<`, `>`→`\>`, literal `{ } |`→`\{ \} \|`;
  long facts are wrapped with `\l` (left-justified break) and indented with
  `&nbsp;` runs. (The exact wrap width / term pretty-printing is **[GAP]** — it
  is graphviz-layout / solver-term-renderer dependent; the model treats cell
  text as pre-rendered.)
- `fillcolor` is a deterministic per-rule color (same rule name ⇒ same color
  within a graph). `role="Undefined"` for plain multiset-rewriting rules;
  otherwise the process/agent role. `fontcolor` is `black` on the light MSR
  palette, `white` on the saturated per-role cluster palette. Exact color hash
  is **[GAP]**; the model carries colors explicitly.

### 3b. Ellipse node (atomic / knowledge / action / temporal node)
```
n<ID>[label="<text>",shape="ellipse"];
n<ID>[label="<text>",shape="ellipse",color="<c>"];
```
- Attribute order: `label,shape[,color]`. Optional `color` (`gray` for `!KU`
  intruder-knowledge nodes, `darkblue` for action/event nodes, `gray30`, …).
- Ellipse text is NOT record-escaped (`<`,`>` appear literally, e.g.
  `Session( 'I', <pk(~ltkA), pkB>, … )`).

### 3c. Edge
```
n<S>[:<sport>] -> n<D>[:<dport>][<attrs>];
```
Endpoints optionally carry a `:port` (record port). `<attrs>` is a
comma-separated attribute list, in observed key order. Frequent styles:

| attrs | meaning (inferred from context) |
|---|---|
| `color="red",style="dashed"` | intruder-deduction / `!KU` edge |
| `style="bold",weight="10.0",color="gray50"` | structural premise/conclusion edge |
| `style="bold",weight="10.0"` | structural edge (uncolored) |
| `color="gray30"` | message/std edge |
| `color="blue3",style="dashed"` | temporal-order edge |
| `color="orangered2"` | (deduction variant) |
| `color="black",style="dashed"` | before/less-than temporal edge |
| `style="invis"` | ranking edge to the legend node |
| `color="green",style="dotted"` / `purple`/`darkorange3` dashed | other goal edges |

### 3d. Node shapes (§ Round-3 item 2)
Corpus census (all 12 022 dot + 81 manifests): only `record`, `ellipse`, `plain`.
**No trapezium appears in the crawl** — because the crawl captured only *proof*
graphs (solved states). Live probing (NAXOS_eCK, NSLPK3 case-distinction graphs)
adds one more shape:
- **`invtrapezium`** — a `(#var, idx)` placeholder for an **unresolved graph node**
  referenced by a still-open premise, e.g. `n9[label="(#i, 0)",shape="invtrapezium"]`.
  `#var` is the (temporal) node variable, `idx` the premise index; the node is fed
  by a structural edge from the conclusion that must satisfy that premise
  (`n5:n2 -> n9[style="bold",weight="10.0",color="gray50"]`). Appears in both the
  compressed default and the uncompressed variant, in every source-case graph
  tested. Attr order = ellipse's (`label,shape`).
- **`trapezium`** (spec-named dual: an unresolved *source* node feeding a present
  premise) was **NOT observed** in any probe (Session-3 NAXOS/NSLPK3 case graphs,
  Session-5 re-scan of NSLPK3 `cases/{raw,refined}/i/j` for i≤12,j≤8) nor anywhere
  in the corpus/manifests. The invtrapezium is the *conclusion→absent-node* case
  (its edge points **into** it: `n5:n2 -> n6`), i.e. the "conclusion feeding a node
  absent from the graph"; the un-seen dual would be the mirror (`absent -> premise`).
  Recorded unobserved; `GraphNode::Shaped{label,shape,color}` (generic) can emit any
  shape a caller supplies.

### 3g. Bare-timepoint ellipse + the `#last` designated timepoint (§ Round-5 item 3)
An uncolored ellipse whose label is a bare timepoint variable `#<var>`:
`n<ID>[label="#<var>",shape="ellipse"];`. Corpus census of ellipse labels found,
besides the `!KU(..)@#t` / `#t:rule[..]` / `Action(..)@#t` forms, bare-timepoint
nodes `#i` (1328), `#decrypt` (790), `#t0…#t6`, `#j`, `#j.1`, and the **designated
last timepoint `#last`** (107, e.g. `24a119958f784d43.dot`). `#last` arises when a
constraint system carries a last timepoint (induction / trace-property proofs); it
is the target of a `color="black",style="dashed"` before-edge. All render as the
plain uncolored ellipse above. Modelled as `GraphNode::Temporal{var}` (with
`GraphNode::last()` for `#last`).

### 3e. Node-id / port allocation (§ Round-3 item 3) — byte-verified over 12 022
One global monotonic counter, ids handed out in **emission order** so file order ==
id order. For each node in order:
- a **record** takes one id per cell (in cell order: premises, then the info cell,
  then conclusions) and **then** one id for the node itself — so a `k`-cell record
  occupies `n<p>…n<p+k-1>` (ports) and `n<p+k>` (node);
- every other kind (ellipse / plain / invtrapezium) takes exactly one id.

Example: a 4-prem/1-info/3-concl record takes ports `n0…n6`, node `n7`; the next
ellipse is `n8`. Verified 12022/12022 in `mine_ids.py` and again in Rust
(`tests/alloc_corpus.rs`, via `NodeIdAllocator`).

### 3f. Record-cell term rendering + line wrapping (§ Round-3 item 1)
**Group structure** (byte-verified over 160 409 records): the label is
`{grp}|{grp}|{grp}` where the groups are the *non-empty* ones among
`[premises, info, conclusions]`, in that order; the **info** group is always
present (100 %). An empty premise or conclusion group is dropped (a source rule
renders `{info}|{concl}`).
- **Info cell**: `#<temporal> : <RuleName>` and, if the rule has action facts,
  `[<action>, …]` (e.g. `#i : I_Complete[Complete( … ), …]`, or bare `#vf.5 : Fresh`).
- **Premise/conclusion cell**: a single fact.
- **Fact spacing**: a fact pads — `Name( a, b )`, and `Name( )` with no args;
  a **function application** does not — `f(a, b)`; a **tuple** is `<a, b>`. Verified
  against the corpus.
- **Escaping**: inside a record label, `< > { } |` each get a leading backslash;
  everything else (quotes, `~ $ ^ * ⊕`, spaces) is literal.
- **Line wrapping FORMAT** (byte-exact). When a group breaks, physical lines are
  separated by `\l` (with a trailing `\l`), and each continuation line is indented
  with a `&nbsp;` run **equal to the column of the broken group's first element**
  (just after `( ` for a fact, after `<` for a tuple, after `[` for an action
  list). Verified across 188 192 wrapped cells.
- **Line wrapping DECISION — RESOLVED (Session 4): a fixed-width paragraph fill,
  width = 87 columns, measured per fact from column 0.** The Round-3 claim ("first
  line 5..87, not a flat width, not derivable") is **refuted**: it *is* a flat
  width. Established by driving crafted single-node theories
  (`Out(<'a01', …, 'aN'>)`, `Out(<'aa…a', 'y'>)`) through the live server and
  sweeping the term width one column at a time:
  - **W = 87 absolute, functor-invariant.** A fact stays on one line iff its flat
    rendering is ≤ 87 columns; at 88 it breaks. Tested with functor names of length
    2, 3, 6, 10 (`Ba`,`Out`,`Fact12`,`Factabcdef`): the boundary was **always** at
    flat width 87 fits / 88 breaks, so the budget is counted from the functor's own
    column 0, *including* the functor name (a longer name simply fits fewer args).
    Corpus-consistent: 99.2 % of 190 044 physical lines have fact-content ≤ 87; the
    residual are single unbreakable atoms wider than 87 (which overflow verbatim).
  - **Greedy paragraph fill.** When a group overflows, elements pack left-to-right;
    the separator `, ` trails the element it follows and stays on the line, and the
    next element wraps when it would pass column 87. Continuation lines use the
    §3f indent. Example `Out(<'a01'…'a12'>)` (flat 91): line 0 =
    `Out( <'a01', … 'a11', ` (eleven elements + trailing `, `, ending col 83),
    `'a12'>` on the next line, then the fact's `)` on its own line.
  - **Delimiter peel — BYTE-IMPLEMENTED (Session 5).** Re-probed the boundary with
    fresh single-node sweeps (`E10..E14`, `W69..W74`, ports 3210): a **tuple's `>`**
    stays with the last element iff it fits (`Out( <'aaa(72)', 'y'>` │ `)`), else it
    peels onto its own line at the tuple's **`<` column** (`W74`: `…'y'` │ `<5sp>>`
    │ `)`); a **fact's `)` ALWAYS** peels onto its own line at **col 0** once the
    fact wraps — even when it would fit (`E12`: `'a12'>` │ `)`) — because the padded
    ` )` space is the break. An **info action-list `]`** stays attached to its last
    action. An unbreakable atom wider than the budget overflows verbatim; only the
    trailing delimiters wrap.
  - **Continuation lookahead — RESOLVED (subsumed).** The old "first line 11,
    continuation 12" claim is a mis-read: the continuation packs **greedily to the
    same width 87** as the first line (`E13`→2, `E14`→3 elements on the second line,
    each ending well under 87). There is no ±1 combinator lookahead to reproduce.
  - **Info action-list = vertical `sep`, not fill.** An overflowing info cell puts
    **one action per line** (corpus `6738eb64…`: 4 actions → 4 lines, short actions
    included), `]` on the last; whereas a tuple / fact-arg list uses the greedy
    fill. Two distinct combinators, both byte-implemented.
  - **`graph-clean::render::wrap_cell`** implements all of the above (`layout_fact`
    fill+`)`-peel, `layout_tuple` fill+`>`-peel, `layout_info` vertical `sep`,
    `run_layout` greedy fill, `split_top_commas`), wired into
    `generate::build_record`; byte-verified against the 7 captured probe fixtures
    (`wrap_E11..E14`, `wrap_W71/W72/W74`).
  - **The wrap TRIGGER is a per-GROUP shared budget — RESOLVED (Session 6).** The
    old per-cell `flat > 87` decision is wrong for multi-cell groups. Each record
    group (`premises`, `info`, `conclusions`) is laid out **independently** (a
    conclusion cell's wrap ignores premise/info widths, and vice versa — live
    probes PM/R: a lone conclusion `Out` of flat 87 stays flat with premises of
    flat 69/103/198 and rule names of length 40). Within a group, the cells share
    a budget:
    > For a premise/conclusion group, let `T = Σ(flat width of every cell in the
    > group)`. Cell *i* wraps **iff** `flat_i > budget_i`, where
    > `budget_i = max(87 − (T − flat_i), 20)`. Equivalently: **cell *i* wraps iff
    > `T > 87` AND `flat_i > 20`.**
    Established by controlled live 2-/3-cell sweeps (QUERIES §6): with a fixed
    second cell of flat *q*, the first cell's fit/wrap boundary is exactly
    `flat = 87 − q` (q=11 → 76/77, q=28 → 58/61, q=48 → 39/40, q=68 → 19/22); a
    third cell adds into the same Σ (3-cell [p,28,28] flips at `flat = 87−56 = 31`).
    The **floor 20** is a per-cell minimum budget (a cell of flat ≤ 20 never wraps;
    live: Fb flat 98 forces the sibling budget negative yet the sibling fits at
    flat ≤ 20, wraps at 21). Verified by hand on `ref_raw_1_1` (Wide rule): premise
    `In` flat 67 does **not** wrap (budget 87−10 = 77) while conclusion `Big` flat
    68 **does** (budget 87−(25+14) = 48); `Ack` flat 25 wraps (budget 20), `Out(h)`
    flat 14 fits — all four correct.
  - **The `#t : Rule[…]` INFO cell** is its own single-cell group (budget 87,
    independent of prem/concl), with an added rule: **an info cell with ≥ 2 action
    facts ALWAYS goes vertical** (one action per line), regardless of width — a
    ≤ 1-action info wraps only when its flat > 87. Corpus census: 0 non-wrapped
    info cells carry a top-level action-list comma (all ≥2-action infos wrap).
  - **Corpus validation at scale.** The rule matches actual `\l` on **99.635 %** of
    776 259 cells and **98.324 %** of 160 409 records (info cells 99.968 %). Every
    one of the 2 831 residual cells is within ~1–2 columns of its budget (mismatch
    `T`-distribution clusters entirely at 79 ≤ T ≤ 95). At that exact boundary the
    outcome is a **±1 HughesPJ `fits`/ribbon rounding artifact** (live: at a group
    total of exactly `budget+1` the fit/wrap flips with atom parity; at `budget+5`
    it wraps cleanly), depending on the cell's token structure, order, and the port
    marker widths — reproducing it byte-exactly needs the GPL pretty-printer's
    internal `fits`. Two residuals remain **[GAP]**: (a) this ±1 boundary flip, and
    (b) the exact greedy fill width when a *multi-cell* cell wraps (live probeB: the
    per-line element count is not a clean function of the budget). The budget
    **trigger** itself (does a cell wrap) is byte-exact away from the boundary.
  - **The FILL width once a cell wraps ≠ the trigger budget — REFINED (Session 7).**
    The wrap *trigger* is the flat-sum budget above (which cell breaks). The *fill*
    width (how a broken cell packs) is **wider**: a sibling that itself wraps
    occupies only the width it is ALLOCATED, not its flat width, so the remaining
    cell packs more elements per line. Live datum (`Wide` conclusion group
    `[Ack 25, Big 68, Out 11]`): the 68-flat `Big` cell packs **8** tuple elements
    on line 0, as if its budget were **56** — not the flat-sum `87−(25+11) = 51`
    (which packs 7). The fill budgets come from a **smallest-flat-first** greedy
    allocation: process cells by increasing flat; each cell's budget is
    `max(87 − Σ others' allocations, 20)` where a processed sibling contributes
    `min(flat, its budget)` and an un-processed (wider) one its full flat. For
    `Wide`: `Out`(11) fits; `Ack`, seen while `Big` is still at flat 68, is squeezed
    to budget 20 and wraps (breaking after `~n.4`); `Big`, placed last, sees `Ack`'s
    allocation 20 and gets `87 − 20 − 11 = 56` → 8 elements. This reproduces the
    whole `Wide` record byte-exact (fixture `wide_record.dot`, verified against the
    live `wide_group.dot` capture) and, applied to every wrapping prem/concl cell in
    the corpus, raises multi-line fill byte-exactness from **41.45 %** (flat-sum) to
    **44.11 %** (best of the models tried: flat-sum 41.45 %, sibling-occ 43.29 %,
    this greedy `min(flat,budget)` 44.11 %). The fill is nonetheless still largely
    the GPL `fillSep`'s `fits`, not a per-cell budget — 56 % of multi-line cells
    remain **[GAP]**: a single non-wrapping sibling of flat 10 barely reduces a wide
    cell's fill budget (live batchA: still 11 elements), whereas a wrapping sibling
    of the same flat reduces it a lot, so the sibling contribution is not a clean
    function of its width (batchA fitting-sibling vs Wide wrapping-sibling).
  - **The TRIGGER residual, decomposed — Session 7.** The 2 831 flat-sum
    mispredictions split, for prem/concl cells (2 779), into two mechanisms, each
    beyond a rendering-crate closed form:
    * **false-positive (1 320; group total 88–97):** flat-sum predicts wrap but the
      cell FITS, because a *wide* sibling wraps and frees room for a *small* cell
      (live hg68: `[Ta, Hg 68]` → `Ta` fits at flat 21 because `Hg` wraps and
      occupies 66, so `Ta`'s budget is `87 − 66 = 21`, not `87 − 68`). This
      occ-relief is the greedy printer's coupled `fits`: it saves a small cell but
      NOT a large one beside a comparable sibling (live `[49,58]` both wrap), and no
      closed rule captures it — feeding the ACTUAL occupied widths back universally
      is *worse* (0.82 % vs 0.45 %), so flat-sum stands.
    * **false-negative (1 459; group total 79–87):** flat-sum predicts fit but the
      cell WRAPS. **954 (65 %)** are explained by the wrap being decided on the
      **unabbreviated** term: e.g. `St_3_eNB( ~eNB_ID, KD19, ~MME_ID, EN2, ~gNB_ID )`
      displays at flat 48 but its abbreviations (`KD19`, `EN2`) expand well past 87,
      so it breaks; abbreviations are substituted into the already-broken layout.
      This is structurally outside a rendering crate that receives POST-abbreviation
      cell text — an honest **[GAP]**. The remaining ~505 are the ±1 `fits` boundary.
  - **Order-independence — re-confirmed (Session 7).** Cell ORDER within a group
    never affects a wrap boundary, even for comparable cells at the margin and even
    with wrapping siblings (live G: `[A,B]` ≡ `[B,A]`, and `[A,30,30]` gives the
    same `A` boundary in every position). So the fill allocation is order-free
    (sorted by flat, not by position).
  - **REVISED (Session 9) — the per-group share is a TWO-LAYER trigger + fill
    allocation with SHAPE-CORRECTED occupancies; the round-8 proportional rule is
    superseded (see the Session-9 items below and the Round-9 report).**
  - **RESOLVED (Session 8) — the fill is HughesPJ `fill` with a 1.5 RIBBON, and the
    per-group share is PROPORTIONAL.** With the sanctioned BSD `pretty` library
    (Text.PrettyPrint.HughesPJ) ported faithfully, two parameters close most of the
    round-7 fill GAP:
    * **Ribbon.** A record cell is laid out with **ribbonsPerLine = 1.5** (the
      HughesPJ default): the fit boundary is the *ribbon* (= 87 for a lone cell),
      and the line length is `1.5 ×` the ribbon. Because lineLength > ribbon,
      HughesPJ `fill` produces **ragged** paragraph fills — a physical line can be
      SHORTER than a later one. Corpus proof: `St_1_gNB( ~gNB_ID, KD8, KD1, '0',
      AM2, GN1 )` wraps as `~gNB_ID, KD8,` (2 args) │ `KD1, '0', AM2, GN1` (4 wider
      args, aligned under `(`) │ `)`. No greedy fill (lineLength == ribbon) can ever
      produce a shorter line0; the round-7 model always could not, which is why 56 %
      of multi-line cells were a GAP. Re-probed live (`layout_at` LINELEN/RIBBON):
      ribbon 24–26 (lineLength 36–39) reproduces `St_1_gNB` byte-exact; all prior
      fixtures (E11–E14, W71/W72/W74, Ack, Big, In) still reproduce under rpl=1.5.
    * **Per-group share = proportional.** Cell *i* of a prem/concl group gets fit
      budget (ribbon) `B_i = max(round(87 · flat_i / T), 20)`, `T = Σ flat_j`. For a
      lone cell `T = flat` ⇒ `B = 87` (recovers the boundary). `T ≤ 87` ⇒ every
      `B_i ≥ flat_i` ⇒ nothing wraps. Wide conclusions `[Ack 25, Big 68, Out 11]`,
      `T = 104`: `Ack 21` (wraps), `Big 57` (8 tuple elems on line 0), `Out 9→20`
      (fits) — byte-exact. The 20 floor is the §3f per-cell minimum.
    * **The engine at the right budget IS byte-exact** (the cell doc — fact `fsep` /
      tuple `fcat` / info `vcat` — is correct): single-cell wrapping cells match
      **94.7 %** (allocator-independent; residual = abbreviation-expansion + ±1).
    * **Corpus census (12 022 dot).** Wrapping-cell byte-exactness rose from ~44 %
      (round-7 greedy) to **81.1 %** overall (single-cell 94.7 %, multi-cell 80.0 %).
      Proportional is the best of the allocators tried (smallest-first 62.6 %,
      flat-sum 59.3 %, prop-ceil 79.9 %, reserve-small 77.4 %).
    * **Residual [GAP], honestly.** (a) ±1 `fits` boundary — proportional lands the
      budget within a few columns of the reference's own coupled `fits`; at a bucket
      edge (e.g. `In_S( 'D2', 'H2', spkDD )` where prop=24.2→24 but the reference
      breaks at ≤23) it flips. (b) Wrap decided on the UN-abbreviated width
      (abbreviations substituted into an already-broken layout) — structurally
      outside a crate that receives POST-abbreviation text. (c) `++`-union / deeply
      nested function-application cells whose internal breaks the fact/tuple grammar
      does not model. The width-model CEILING (any budget reproduces the cell) was
      78 % of multi-cell under greedy; the ragged fill raised it so proportional now
      reaches 80 %.

  - **RESOLVED (Session 9, SUPERSEDED by Session 10) — the group WRAP TRIGGER
    is size-corrected flat-sum with the `elems + 1` law.** Six round-10 live
    batteries (audit RA–RE; func FB–FE; quote QA–QC; tuple/union pins PA–PF /
    UEV / TN / TB / UN / UB — QUERIES.log Session 10) on top of the round-8/9
    grids pin the wrap decision to:
    > Cell *j* occupies `C_j = flat_j + Σ_{top-level tuple/union args}
    > (elems + 1)`. Cell *i*'s trigger budget in a multi-cell row is
    > `max(87 + bonus_i − Σ_{j≠i} C_j, 20)` with `bonus_i` = the largest
    > `⌊elems/2⌋ + 2` over its own tuple/union args (an arg with ≥ 9 elements
    > contributes 4; no such arg ⇒ 0); it wraps iff `flat_i` exceeds the
    > budget. A lone cell's budget is exactly 87.
    The `elems + 1` occupancy is boundary-exact at n = 2, 3, 4, 6 (tuples)
    and 3, 5, 8 (unions): the 45-argfact partner flips exactly one column
    after `C` crosses 42 in every sweep, refuting round-9's `2n − 4` (which
    coincides only at n = 5). Facts with function-application or quoted
    arguments carry NO correction (FB/QA/QB flips are at the plain flat
    crossing), quote position/count inside tuples is irrelevant (PA/PB/PC/PD
    ≡ PF byte-wise), and both round-9 single-quoted-atom corrections are
    refuted (the logged `C(sqa) = flat − 4` puts the partner flip at 47 vs
    the live 43; the shipped own `flat − 2` discount mispredicts 6 round-10
    cells). The bonus values read from the own-side flips are
    `{n=2:3, 3:3, 5:4–5, 6:5–6, 8:6–7}` (intervals from the relief ambiguity
    below) with the r8 16/20-element rows capping large-n at ≤ 4 —
    `⌊n/2⌋ + 2` (≤ 8 elements, else 4) is consistent with every row. Score:
    **722/731** probe cells; ALL 9 misses are ONE pattern — beside a 45-flat
    argfact partner a cell at exactly budget+1 stays flat (observed alike for
    quoted atoms, func facts, 2-quote facts, tuples, unions) while beside a
    46-flat partner or in triples it wraps — the known ±1 coupled-`fits`
    relief, not modelable in closed form. Corpus: 1.203 % cell error vs
    1.051 % for the round-9 model — but on abbreviation-FREE groups the new
    law is clearly better (1.98 % vs 2.48 %), and 90 % of the old/new
    disagreements lie in abbreviated groups where the reference's widths are
    unknowable from display text; the probe-pinned law stands.
  - **REVISED (Session 10) — the FILL share is proportional with an INTERNAL
    numerator and occupancy denominators**:
    `b_i = clamp(round(87·N_i / (N_i + Σ_{j≠i} w_j·C_j)), 20, flat_i−1)` where
    `N_i = flat_i + Σ_{union args}(elems + 1) + #function-nodes` (union
    separators are internally spaced; function applications ~1 column each —
    pinned by the UD/UG squeezed-union fills at 12/13/14 elements per line
    and the DN chain-tail widths), `C_j` = the trigger occupancy above, and
    `w_j = 5/6` for single-quoted-atom siblings of a tuple-fact receiver
    (round-9 Q/I series: the [Big 87, atom s] fill follows 87²/(87 + 5s/6)
    across s = 12…120), else 1. Tuple args do NOT enter the numerator — the
    live `Wide` record byte-pins the tuple receiver at the display-flat
    share. Probe-battery byte-exactness (fill_census over all ten probe dot
    sets): 525/547 wrapping cells (96.0 %).
  - **NEW (Session 10) — union and function-application cell documents.**
    A `++`-union displays PARENTHESIZED and unspaced — `(y1++y2)` — and lays
    out like a tuple: elements fill one past the `(`, each `++` trails its
    element, and the `)` stays beside the last element iff it fits, else
    peels onto its own line at the `(` column (UB_39 vs UB_40, byte-exact).
    A function application `f(a, b)` breaks INSIDE: its arguments fill after
    `name(` (continuations at that column; nested chains indent +2 per
    level), and its `)` stays ATTACHED to the last argument — it never peels
    alone, unlike the fact-level `)` (FD_88 peel-only at 88, FD_90 internal
    break at 90, FC_1/FC_3 fills/chains, all byte-exact). Both implemented in
    `doclayout::arg_doc` (recursive), closing most of the round-9 band-NONE
    family: corpus band-NONE wrap cells fell from 14 695 (10.3 %) to
    **8 305 (5.8 %)** — plain 1360→9, abbr-only 1502→14, func-only
    1430→646 (remaining: `name(<`-hang layouts), func+abbr 10403→7636
    (pre-abbreviation widths, structurally unbandable from display text).
  - **NEW (Session 10) — caller-supplied width inputs.** The reference
    decides row sharing on internal (UN-abbreviated) widths; a caller that
    knows them may supply per-cell overrides: `generate::CellWidths
    { occupancy, bonus, fill_width }` (each field optional, falling back
    per-field to the display-text estimate), accepted by
    `group_widths_with(cells, overrides)` and by `RawRule::premise_widths /
    conclusion_widths` (one `Option<CellWidths>` per cell). With every
    override absent the behavior is byte-identical to the estimate path
    (regression-gated by `supplied_cell_widths_override_estimates` /
    `raw_rule_supplied_widths_reach_cells` and the corpus census).
  - **REVISED (Session 11) — the row-share model is a TWO-PASS trigger with
    RECURSIVE occupancies, a last-arg-gated bonus, half-DOWN fill rounding, an
    internal numerator that includes tuples (capped), and a TUPLE-OPENER HANG
    in the cell document; and the round-9/10 "wrap decided on the
    UN-abbreviated width" belief is REFUTED by direct probe.** Round-11 live
    batteries (G/H/I 124 rules on :3200-3202; J 9 rules on :3203; K 65 rules
    on :3204 — QUERIES.log Session 11):
    * **Fill rounding = round-half-down** (exact .5 down, else nearest):
      equal both-wrap pairs [50,50]…[80,80] allocate 43/43, refuting half-up's
      44 (probe GB; archived r10 probe re-score 510/535 vs 503).
    * **Occupancy is RECURSIVE**: `C = flat + Σ tuple/union nodes`, each node
      `elems + 1` except directly-nested-in-a-tuple nodes at `elems − 1`
      (K1 pair-of-pairs: partner flips at 38 = +5 exactly; K2 tuple inside a
      FUNC arg counts FULL `elems + 1`, flip at 39; K6 pair-of-6-tuples pins
      nested-6 ≥ ~5).
    * **The self-budget bonus applies ONLY when the fact's LAST top-level arg
      is the tuple/union** (WIT battery: a mid-list 4-tuple fact flips at its
      bonus-free budget 78/79 beside `Fr( ~ni )`; TB4/TB4f single-tuple facts
      keep it — and much of the apparent bonus is really the RELIEF pass:
      TB4's 47/48 flip is reproduced by relief alone).
    * **Trigger pass 2 — RELIEF (family 3)**: a pass-1-wrapping cell renders
      flat iff it fits in the room its siblings actually occupy:
      `flat ≤ max(87 − Σ charge_j, 20)`, where a truly-broken wrapping
      sibling (fill < flat − 2, beyond the `)`-peel-only zone) charges its
      fill allocation, everything else its C; NO bonus in this comparison
      (IB: a tuple target beside a wrapping 90 fits only at the floor 20).
      Pins the corpus family-3 false-wraps (1,478 → ~1,150) and the
      beside-65 fits-at-23/wraps-at-24 boundary (the 23-case itself is the
      known ±1 coupled-`fits` residue, as is `[45,43]` saved vs `[46,42]`
      not).
    * **Fill numerator includes tuples**: `N = flat + rec_sur(cap 7) + nfunc`
      — a 6-tuple receiver fills at ribbon 38 beside a 60-argfact (= flat+7,
      K3), a pair receiver at 25/26 beside 70/75; per-node contributions cap
      at 7 (the r8 16/20-element grids refute the uncapped sum). Round-10's
      "tuple args do NOT enter the numerator" is REFUTED (the Wide fixture
      still reproduces byte-exact under the new law).
    * **Tuple-opener HANG**: when a tuple's first element does not fit beside
      the `<`, the `<` stays at the end of the current line and the elements
      start on the next line at the fill column (one past the `<`) — also
      inside function arguments (`w1(<` hang). FACT and FUNCTION openers do
      NOT hang: a too-wide first argument overflows verbatim (K4 battery;
      union first elements sort last under AC ordering, so a union hang is
      unobservable). Implemented as a zero-width leading fill item;
      byte-fixtures `tuple_opener_hang_byte_fixtures`. This closed the bulk
      of the corpus band-NONE family (14,695 → 8,305 in round 10 → ~850 rows
      round 11) INCLUDING almost all cells previously attributed to
      pre-abbreviation layout.
    * **Abbreviation/internal-width REFUTATION (battery J)**: crafted
      theories whose abbreviated terms have internal widths 96–150 while
      display fits (J1/J2/J7, with `?unabbreviate=` twins as ground truth)
      render FLAT — the reference lays out the POST-abbreviation display
      text; a sibling beside an abbr cell keeps the display-C budget (J6).
      The round-7 "954 cells explained by un-abbreviated width" and the
      round-9/10 family framing are superseded: those cells are now
      reproduced by the display-side laws above (single-cell wrapping cells:
      100 % byte-exact corpus-wide). The corpus family-4 witness
      (`St_I( …, SI4 )`, e.g. 01e0a9f6bf86b671.dot) is actually a 2-cell row
      `[St_I 77, Fr( ~ni.1 ) 9]` wrapping by `)`-peel at its bonus-free
      budget — a trigger-margin case, not an internal-width one.
  - **REVISED (Session 12) — the trigger slack is `⌈elems/2⌉ − 1` for a
    tuple/union arg in ANY position (the round-10/11 last-gated `⌊elems/2⌋+2`
    bonus is REFUTED — its probe readings were relief artifacts of wrapping
    45-siblings); occupancy and the fill numerator count FUNCTION NODES
    INSIDE tuples; the fill numerator does NOT carry top-level `nfunc`; and
    the relief charge follows the wrapping sibling's UNROUNDED fill quotient.**
    Round-12 batteries L/M/N/O (111 rules on :3200-3203 — QUERIES.log
    Session 12):
    * **Pass-1 slack** (battery L, beside a floor-protected flat-20 sibling,
      bonus-free budget 67): pair 0 (LA2_68 wraps at budget+1), 3-tuple 1
      (LA3_68 flat / 69 wraps), 4-tuple 1 — MID-LIST INCLUDED (LD4_68 FLAT at
      budget+1 / 69 wraps), 6-tuple 2 (LA6_70 wraps at +3), 3-union 1,
      single-arg 3-tuple 1 (LC3_69 wraps at +2, refuting the single-arg
      `⌊e/2⌋+2`); law `s = max over top-level tuple/union args of
      ⌈elems/2⌉ − 1`, capped at 4. WIT re-read: `Fr( ~ni )` is flat 9, so the
      78/79 flip = slack 1, consistent; the WIT_79 wrap itself is the ΣC=88
      zone (below).
    * **Occupancy & fill numerator count funcs-inside-tuples** (`ftup`): the
      corpus `[41w, 51 single-arg-nested-pair]` false-wrap witness resolves
      exactly when `C = flat + rec_sur + ftup` (its 3 in-tuple funcs) and the
      OD replica byte-confirms the charge arithmetic (49 saved / 50 wraps);
      top-level funcs stay uncharged (round-10 FB pins hold). The fill
      numerator drops the round-10 `+nfunc` term (corpus dq bias + FB_47
      in-band under `N = flat + rec_sur7 + ftup`).
    * **No quote discount** (battery NB): `N` unmodified by quoted constants
      (−1/−2 per quote predictions fall out of band at NB2_58_43/NB4_58_41);
      **numerator cap 7 holds at 8 elements** (NC8_62_40); **fill
      denominators use FULL recursive occupancy** (ND16: a 16-tuple sibling
      charged `flat + 17` is in-band, capped `+7` is not).
    * **Relief charge** (batteries M/MC, sibling gaps 3–8): a wrapping
      sibling charges `min(hd(q + 1/3), C)` where `q` is its UNROUNDED
      proportional quotient (byte-pinned: q 43.02→43, 43.5→44, 49.45→50,
      54.03→54, 53.47→54, 57.4→58, 59.48→60); MC pins the charge as q-based,
      NOT the occupancy C. EXCEPT: a saved cell whose top-level tuple/union
      arg has ≥ 4 elements drops the bump (TB4 47/48, TB6 48/49, UEV 47/48,
      UB8 49/50 re-reads — all four boundary pairs byte-consistent only
      bump-free). No slack enters relief.
    * **The ΣC = 88 coupled zone is TERMINAL** (battery O): at row totals of
      exactly 88, `[45,43]` keeps the 43 flat while `[46,42]` wraps the 42
      (sibling quotients 44.49 vs 45.48 — same frac, opposite outcomes);
      `[29,30,29]` keeps the 29s flat while `[30,30,28]` wraps all three; the
      OC replica (`[34 3-tuple, T, 25 pair]`) keeps T flat at budget+2. The
      required per-row charge roundings are mutually contradictory (mixed
      up/down for equal fracs), proving NO closed form over cell widths
      decides these rows — they are the reference's coupled per-row `fits`.
      Battery-M rows and plain 2/3-cell relief boundaries are DEGENERATE for
      plain targets (the proportional fill couples the relief boundary onto
      the pass-1 boundary: `f(W + 87 − f) = 87W` forces `f ∈ {W, 87}`), which
      is why this zone concentrates all remaining trigger error.
  - **REFUTED (Session 9): the group is NOT one horizontal HughesPJ document**
    (fcat/fsep/cat/sep of cell docs at any (lineLen, ribbon) tried fails almost
    every controlled case — cells that share a line would never wrap
    internally), and **premise-row width does NOT couple into conclusion-row
    budgets** (byte-identical conclusion rows across premise widths 17…127),
    and **cell order within a group is irrelevant** (order-swapped probes
    byte-identical modulo the swap).

--------------------------------------------------------------------------------
## 4. Clustering / simplification (§ priority 3)

**Trigger rule (byte-verified over all 14705 graphs, 0 violations):**
> The COMPACT clustered rendering (§2b, with `subgraph "cluster_…"` blocks) is
> used **iff the graph contains at least one node with `role` ≠ `"Undefined"`.**
> Otherwise the SIMPLE header (§2a) is used.

Non-Undefined roles arise from process/SAPIC agents (e.g. `Initiator`,
`Responder`, `server`, `Tag`, `v`). Pure multiset-rewriting theories emit the
simple header everywhere; role-annotated theories emit the compact header on
every non-empty graph (empty graphs have no role nodes ⇒ simple).

### Cluster block
```
subgraph "cluster_<Role>_Session_<k>" {
nodesep="0.6";
ranksep="0.6";
label="<Role>_Session_<k>";
style="filled";
color="#RRGGBBAA";
penwidth="2";
fillcolor="#RRGGBBAA";
overlap="false";
sep="4";
n<ID>[shape="record",label="…",fillcolor="#RRGGBB",style="filled",fontcolor="white",role="<Role>"];

}
```
- Cluster name = `cluster_<clusterlabel>`; `label` = the clusterlabel
  (`<Role>_Session_<k>`). Nodes grouped by (role, session instance).
- `color`/`fillcolor` are an 8-hex ARGB (alpha `4C`, ~30%) hashed from the
  cluster; node `fillcolor` is the saturated per-role color (live SAPIC probe
  `cluster_Process_Session_1` had node `fillcolor="#ffffff"`, `fontcolor="black"`
  — so the node fill is not always saturated). Exact hash **[GAP]**.

### Cluster EMISSION ORDER (§ Round-5 item 1) — byte-verified over 1457 corpus files
The top-level statement order of a clustered graph is **always**
`[free nodes] [clusters] [edges] [rankblock/legend] [invis edges]`: 0 files have a
top-level node after a cluster, 0 clusters after an edge. **Clusters contain only
records** (50 453 record cells, 0 ellipse/edge). A cluster's records occupy a
**contiguous** id range and clusters are ordered by increasing id (first-appearance
== id order). Since id order == emission order (§3e), the global id counter runs
over the free (non-role) nodes first — giving them the low ids — then over the
clustered records grouped by cluster. `graph-clean::generate` reproduces this:
Pass 1 allocates ids in `System.nodes` order, routing each role-record's statement
into its `cluster(label,color)` bucket and everything else to the free top level,
then emits free nodes, then cluster blocks in first-appearance order, then edges,
then the legend. Byte-exact on a live SAPIC single-cluster graph
(`cluster_process.dot`) and the multi-cluster corpus payload
`79c16911ad179d51.dot` (4 free ellipses + 1 cluster + 2 deduction edges).

### What "simplification" means here
There is no exposed simplification *level*; the corpus reflects one fixed
rendering pipeline. The observable "simplification" is the fixed compression the
server applies before rendering: intruder-knowledge (`!KU`) nodes are drawn as
gray ellipses; single-premise/conclusion structural chains are drawn with bold
weight-10 edges; role nodes are packed into per-session clusters (compact mode).
Reproducing the *content* of that compression from a raw constraint system
requires the solver and is **[GAP]** — out of scope for a rendering crate.

--------------------------------------------------------------------------------
## 5. Node abbreviation (§ priority 2)

When a graph is drawn, sufficiently complex sub-terms are replaced by short
NAMES and listed in a legend node. Observed from 113 924 legend rows.

### 5a. Legend node
A `plain` node with an HTML-like label, placed in a sink-rank block:
```
{
rank="sink";
n<ID>[shape="plain",label=<<TABLE BORDER="1" CELLBORDER="0" CELLSPACING="3" CELLPADDING="1"><TR>…row0…</TR>
<65 spaces><TR>…row1…</TR>
…
<65 spaces><TR>…rowN…</TR></TABLE>>];

}
```
- Each row:
  `<TR><TD ALIGN="LEFT" VALIGN="TOP"><FONT COLOR="#000000">NAME</FONT></TD> <TD ALIGN="LEFT" VALIGN="TOP">=</TD> <TD ALIGN="LEFT" VALIGN="TOP">EXPANSION</TD></TR>`
  (single spaces between `</TD>` and the next `<TD>`).
- The first `<TR>` is inline after the `<TABLE …>` tag; every continuation `<TR>`
  is on its own line indented by **65 spaces = the byte length of the
  `<TABLE BORDER="1" CELLBORDER="0" CELLSPACING="3" CELLPADDING="1">` opening
  tag** (graphviz hang-indent of the table's children; independent of node id).
- EXPANSION is HTML-escaped: `<`→`&lt;`, `>`→`&gt;`, `&`→`&amp;`. Single quotes
  and `* ^ ~ $ ⊕` are literal.
- The legend node is wired to the rest with `style="invis"` edges (from a couple
  of real nodes to the legend), emitted AFTER the sink block.

### 5b. Naming scheme (byte-verified rule)
`NAME = <PREFIX><n>`, n = 1,2,3,… per prefix.
`PREFIX` = the first **two alphabetic characters, uppercased**, of the ROOT
symbol's *name* (one letter if the name has only one). The root symbol name is:
- function application `f(…)`  ⇒ `f`      (`sign`→SI, `senc`→SE, `hash`→HA,
  `h`/`h1`/`h2`→H, `KDF`/`kdf1`→KD, `pk`→PK, `aenc`/`aead`→AE, `pmult`→PM, …)
- public constant `'c'`        ⇒ `c`      (`'uninitialized'`→UN, `'F_status'`→FS,
  `'ho_req_ack'`→HO, `'no_message_state'`→NO — non-letters skipped: F,s→FS)
- variable `~v` / `$v` / `v`    ⇒ `v`      (`~AMF_UE_NGAP_ID`→AM, `~cid_N26`→CI,
  `~gNB_UE_ID`→GN, `~mid…`→MI, `commitmsg`→CO, `StateChannel`→ST)
- infix operator, mapped to its function name then the letter rule:
  `^`→exp→**EX**, `*`→mult→**MU**, `++`→union→**UN**, `⊕`→xor→**XO**.

Abbreviations NEST: an inner abbreviated term is referenced by name inside an
outer expansion (`KD1='KDF(EX1)'`, `MA1='mac(EX1, SI1)'`, `EX1="'g'^MU1"`,
`H1="h(<…, EX2, EX3, EX1, …>)"`). So naming is applied bottom-up.

Per-prefix numbering is by tamarin's canonical term order, NOT by first
appearance in the drawing (observed: `In(EX2)` is the first occurrence yet gets
`EX2`, and larger/deeper terms get the lower numbers within a prefix). The exact
tie-break total order is **[GAP]** (needs the solver's term ordering); the crate
assigns per-prefix numbers over a defined order and matches whenever that order
is unambiguous.

### 5c. Selection — WHICH sub-terms get abbreviated  **[RESOLVED — see REPORT2.md]**

**Rule (three necessary gates; each confirmed by a controlled black-box probe):**
> A sub-term `t` is abbreviated **iff**
>  1. `renderLen(t) ≥ 10`  (Unicode-scalar length of the fully-expanded surface
>     rendering, *including* `'…'` quotes and `~`/`$` sigils), AND
>  2. `occ(t) ≥ 2`  (occurrences over the whole constraint system — graph node
>     facts **and** the sequent's goals/formulas — edge-shared messages count
>     once), AND
>  3. `t` is **not** a tuple `<…>`.
> Naming is bottom-up: nested eligible sub-terms are named first
> (`LO1='longarg123'`, then `H1='h(LO1)'`).

This is a rendered **length** gate, not the *size* or *occurrence* the earlier
note guessed at — which is why it looked context-free-yet-unexplained. It
cleanly settles the old counter-observations: `sign(<…,'g'^~ekR>,~ltkI)` is ≥10
chars (abbreviated); `KDF(z)` is 6 chars (never abbreviated, whatever its count);
`mac(z,SI1)` and `'g'^~lkR` are below the length/occurrence thresholds in their
graphs. `'g'^~lkR` vs `'g'^~lkR.1` differ because the `.1` copy reaches occ 2 in
the system while the other does not.

**Corpus-exactness (necessary direction):** across all **97 538** legend
expansions in **11 564** graphs, the minimum rendered length is exactly **10**
(0 below) and **0** entries are top-level tuples. Among terms drawn ≥2× in a
graph, **0** abbreviated terms violate len≥10 ∧ ¬tuple. Abbreviated terms with
graph-occurrence 1 (7 800) all reach ≥2 once the sequent is counted.

**Controlled probes (crafted theories, live server; see QUERIES.log):**
- `'12345678'` (10 chars, ×2) abbreviated; `'1234567'` (9, ×2) not  ⇒ boundary = 10.
- `'ABCDEFGH'`/`'onceonly12'` (×1) not abbreviated; `'twicelong1'` (×2) is;
  a 42-char constant (×1) not abbreviated  ⇒ occ ≥ 2 is a hard gate at any length.
- `<'aa','bb','cc'>` (18 chars, ×2) not abbreviated while sibling `h('longarg123')`
  (×2) is  ⇒ tuple exclusion + bottom-up nesting.

**Residual (sufficiency):** for **AC/DH-operator** sub-terms (`^ * ⊕ ++ inv em
hp pmult`) tamarin counts occurrences over the *normalised* (AC-flattened,
DH-normal-form) term, not the surface rendering, so the same term can be
abbreviated in one proof state and not another (`em(hp($A),hp($B))` inline in
state `56eeb42d`, `EM1` in `54eb2e2c`). 93 % of apparent exceptions are such
AC/DH cases; the rest are validation artifacts from expanding shared
abbreviations before counting. Outside AC/DH terms the rule is exact.

The crate `abbrev::select` implements this rule (`MIN_ABBREV_LEN=10`,
`MIN_ABBREV_OCC=2`, tuple-excluded, bottom-up); tests reproduce every probe.
Byte-parity of the selection set is claimed for non-AC/DH terms and gated by the
caller supplying the full-system term multiset.

--------------------------------------------------------------------------------
## 6. System → graph mapping (§ Round-3 item 4)

From paired `main/proof` sequents and `interactive-graph-def` graphs, the
structural mapping a proof state yields (each point traces to §3d/§3f and the
content census; the *content* — which nodes/edges exist — is a solver GAP):
- a **rule instance** `#t : Rule[actions]` → a `record` node with groups
  `{premises}|{info}|{conclusions}` (empty prem/concl dropped, info always kept);
- an **intruder-knowledge fact** → a gray `!KU( m ) @ #t` ellipse;
- a **protocol action/event** → a darkblue `Fact @ #t` ellipse;
- a **compressed intruder rule** → an uncolored `#t : rule` ellipse (§4 compression);
- an **unresolved node** referenced by an open premise → an `invtrapezium`
  `(#var, idx)` (§3d);
- **edges** connect conclusion-port → premise-port (structural bold w10 [gray50]),
  intruder deduction (red dashed), message (gray30), temporal order
  (blue3/black dashed), etc. — the finite §3c vocabulary;
- ids from §3e; header from §4 (role); legend from §5.

`graph-clean::generate` implements this over an independent input model
(`System`/`GraphNode`/`RuleInstance`/`SysEdge`), taking node/edge lists, colors,
and cell text as INPUTS. `tests/generate_tests.rs` reproduces a live source-case
graph (`nsl_invtrap.dot`) **byte-exact** end-to-end.

--------------------------------------------------------------------------------
## 7. Simplification / abbreviation options (§ Round-3 item 5)

The UI's "Graph simplification off/L1/L2/L3" menu and abbreviation toggle are
cookie-driven and appended by the JS as graph-URL **query params** (mined from
`/static/js/tamarin-prover-ui.js`, confirmed by live diffs — QUERIES.log):

| param              | UI sends when                    | observed effect                          |
|--------------------|----------------------------------|------------------------------------------|
| `simplification=N` | always (N=level, default 2)      | **inert** — the *number* changes nothing (see below) |
| `uncompact=`       | level 0 only                     | node COMPACTION off (lone intruder rule → full record vs ellipse) |
| `uncompress=`      | level 0 only                     | system COMPRESSION off (Fresh sources + intruder rules re-expand to records) |
| `unabbreviate=`    | abbreviate cookie off            | abbreviation off (terms inlined, legend block omitted) |
| `no-auto-sources=` | auto-sources cookie off          | disables auto-source precomputation      |
| `clustering=true`  | clustering cookie on             | forces role clustering                   |

### 7a. The L1/L2/L3 distinction — RESOLVED (channel-verified negative, Session 4)

The Round-3 GAP ("simplification 1/2/3 produced byte-identical graphs") is **not a
probing artifact and not a hidden session channel** — it is the genuine server
behavior: **there is no L1/L2/L3 distinction; the `simplification=N` *number* is
inert for every N.** Established under the session-state lens the follow-up
demanded:

- **Channel.** The served JS (`tamarin-prover-ui.js`, lines 78–100 / 476–500)
  reads a *client-side* `simplification` cookie (default 2) and converts it into
  the graph-URL query param; at level 0 it *additionally* appends `uncompact=` +
  `uncompress=`. Levels 1/2/3 send **only** `simplification=N`. The server emits
  **no `Set-Cookie` on any response** (root, case-graph, main/cases json, proof
  method) — there is *no server-side session state*; the param is the only channel.
- **Change verified before diffing.** Driving the *flags* through the URL param
  demonstrably changes the graph (e.g. rich NSLPK3 case `refined/6/5`: default
  5 668 B → `uncompact=&uncompress=` 7 792 B), and the result is byte-identical
  whether or not a matching cookie is also sent. So the URL param **is** honored.
- **The number is inert.** On that 17-node case graph and on a **39-node, 27 KB**
  DH/bilinear proof-state graph (`eurosp19-eccDAA` analysed,
  `proof/functional_correctness/_`), every probe of `simplification` ∈
  {0,1,2,3,4,5,10,99} — via URL param, via cookie, and via both together, and
  crossed with every flag combination — collapses to a **single** md5. `flags_L0`
  … `flags_L99` are byte-identical. (Second theory NAXOS_eCK case `refined/4/1`
  reproduces this.)
- **Mechanism.** The UI's four-item menu (`for i<4`: off / L1 / L2 / L3) dresses up
  a *binary*: "off" (level 0) = the UI adds the two flags; "L1/L2/L3" = no flags,
  and the server never consumes the level integer. The real transforms are the
  independent boolean flags, each with a distinct, measured effect on the 39-node
  graph: `uncompact=` 39 nodes→richer records (27 361→30 880 B); `uncompress=`
  39→**74** nodes (re-expands compressed chains, 31 407 B); `unabbreviate=` drops
  the legend node (39→38) and inlines terms (36 690 B, 0 legend rows);
  level 0 (`uncompact`+`uncompress`) = 74 nodes, byte-identical to the two flags
  sent explicitly.

`graph-clean::options::Options` models the knobs and reproduces the exact query
string; `simplification=N` is carried but is documented inert. The abbreviation
on/off transform is applied directly (drop legend / inline). Remaining **GAP**
here: the *content* of compress/compact (a solver transform — the caller supplies
the already-(un)compressed node set).

--------------------------------------------------------------------------------
## 8. What the crate reproduces vs. gaps

Reproduced & byte-tested against captured/live payloads:
- DOT document assembly: both headers, block/whitespace rule, node lines
  (record/ellipse/plain/**invtrapezium**), edge lines with ports & attrs, cluster
  subgraphs, legend sink-block + invis edges, empty graph. Round-trip reproduces
  **12 022/12 022** corpus payloads byte-exact (`tests/roundtrip.rs`).
- **Node-id/port allocation** (§3e): `NodeIdAllocator`; validated 12 022/12 022 via
  the crate model (`tests/alloc_corpus.rs`).
- **Record-cell rendering** (§3f): fact/function/tuple spacing, info-cell shape,
  group-drop rule, record escaping, and the wrap **DECISION + FORMAT + PEEL** —
  `wrap_cell` (`FILL_WIDTH=87`, greedy `fillSep` for fact-arg/tuple lists, vertical
  `sep` for info action lists, tuple-`>` peel to the `<` column, fact-`)` peel to
  col 0), wired into generation and byte-verified against 7 live boundary probes.
- **System → graph GENERATION** (§4, §6): `generate` over the independent `System`
  model — free/ellipse/record/invtrapezium nodes, **role clusters** (`subgraph
  cluster_…`, free-then-cluster id/emission order), the **`#last`/bare-timepoint**
  ellipse, the finite edge vocabulary, id/port allocation, and the wired record-cell
  wrap. Byte-exact on live + corpus fixtures (invtrap, single/multi cluster,
  `#last`, wrap sweep) in `tests/generate_tests.rs`.
- **Pre-rendered-cell interop** (§ round 5): `RawRule` / `GraphNode::RawRule` accept
  already-rendered cell strings and run them through the same wrap/escape pipeline,
  byte-identical to the Term-based path (`raw_rule_matches_term_path_byte_exact`).
- **Simplification/abbreviation options** (§7): `Options` query-string model; the
  **L1/L2/L3 level number is proven inert** (§7a) — channel-verified negative.
- Abbreviation naming, numbering, legend HTML (65-space indent), and the SELECTION
  rule (§5c, REPORT2.md), plus the cluster/compact trigger (§4).

- **Record-cell group WRAP** (§3f, Sessions 8–12): a **faithful HughesPJ port**
  (`pretty.rs`, from the sanctioned BSD `pretty` library, with the Haskell
  laziness mirrored via `Doc::Lazy` thunks + first-line-only `fits` — pure
  evaluation-order change, byte-identical, kills an exponential blowup on
  many-element fills) laid out at **ribbonsPerLine = 1.5** so the paragraph fill
  is RAGGED, plus the **two-PASS allocation** (`generate::group_widths`,
  Session 12): recursive-occupancy flat-sum TRIGGER (`C = flat + Σ per
  tuple/union node (elems+1), directly-nested-in-tuple nodes (elems−1), + 1
  per function node INSIDE a tuple/union`; slack `⌈elems/2⌉−1` capped at 4
  over top-level tuple/union args in ANY position), a RELIEF second pass
  (wrapping sibs charged `min(hd(q + 1/3), C)` on their UNROUNDED quotient,
  bump dropped for ≥ 4-element-tuple receivers), and proportional FILL
  rounded half-DOWN with the internal numerator
  `flat + rec_sur(cap 7) + ftup` over sibling occupancies, quoted-atom sibs
  at 5/6 for tuple-fact receivers. The cell grammar includes **union and
  function-application documents** (§3f Session 10) and the **tuple-opener
  hang** (§3f Session 11). Corpus census (round 12): all-cells **98.93 %**,
  wrapping cells **96.45 %** (single-cell **100.00 %**, multi-cell
  **96.14 %**), false-flat 172 (round 11: 324), false-wrap ~1,477 (1,150 —
  moved INTO the proven-non-closed-form ΣC=88 zone by the probe-forced
  tighter slack), fill misses 4,884 (5,954). Probe-battery wrap
  byte-exactness 979/1,023 across every battery ever captured (old set
  824/862; round-12 L 28/28, M 33/34, N 60/64, O 34/35). Callers may
  override per-cell widths (`CellWidths` incl. round-11 `trigger_width`;
  INTERFACE.md) — though the J-battery shows the reference itself computes
  on display text, so the estimates ARE the probed behavior.

Documented gaps (need the GPL solver or an unavailable backend):
- JSON graph backend format (unavailable / not in corpus).
- **Record-cell wrap residuals** (§3f, Session 12) — the remaining ~3.5 % of
  wrapping cells (fill 4,884 + false-flat 172 + false-wrap ~1,477):
  (a) the **ΣC = 88 coupled-`fits` zone** (now the DOMINANT residue, and
      PROVEN non-closed-form by battery O: `[45,43]` vs `[46,42]`,
      `[29,30,29]` vs `[30,30,28]`, OC budget+2 saves, WIT_79 vs LD4_68 —
      required charge roundings are mutually contradictory at equal
      fractional parts): cells at exactly budget±1..2 where the reference's
      per-row coupled `fits` decides;
  (b) **large-tuple numerators**: the cap-7 law approximates 8–20-element
      tuple receivers (r8 grids 78/86; K3 small-receiver-beside-huge-sib and
      K6 pair-of-6-tuples fills stay ±1-2 out);
  (c) a band-NONE residue (~1,640 cells): mostly abbreviation-expansion
      layouts inside deep func chains and multi-run band shapes — NOT
      explained by internal-width layout (refuted, §3f Session 11); likely
      further combinator subtleties.
- **compress/compact content** (§4, §6): which nodes/edges a raw constraint system
  yields — a solver transform. (The L1/L2/L3 level distinction is no longer a gap:
  it is proven non-existent, §7a.)
- Per-rule/per-cluster **color hashes** (§3a, §4); the `trapezium` dual (§3d, unobserved).
- Abbreviation of AC/DH sub-terms (§5c residual, normalised-form occurrence).
- Canonical per-prefix numbering tie-break (§5b); empty-prefix start index.

--------------------------------------------------------------------------------
## Round 8 report — faithful layout engine + record-cell feeding (folded here per protocol)

**Task.** Replace the closed-form fill with a faithful layout engine; reconstruct
how record-cell content is fed through it; wire through wrap/build_record/RawRule;
keep the GRAPHCLEAN_CORPUS round-trip at 12022/12022.

**Sanctioned material.** `sanctioned/pretty-1.1.3.6/` (BSD HughesPJ). The
Doc/best/fits/fill port lives in `graph-clean/src/pretty.rs` (audited faithful;
default Style confirmed `lineLength=100, ribbonsPerLine=1.5`). Everything
tamarin-specific (which combinators, widths, ribbon) came from black-box probes.

**What changed vs round 7.**
1. The fill is the sanctioned HughesPJ `fill`, not a bespoke greedy pass.
2. **Ribbon = 1.5** (was implicitly 1.0). `doclayout` renders each cell at
   lineLength = `⌊3·budget/2⌋`, ribbonsPerLine 1.5, so the fit boundary is the
   ribbon (= budget) but lineLength is 1.5× larger — the gap that makes `fill`
   RAGGED. This was THE missing piece: it reproduces `2-args-then-4-wider-args`
   cells (`St_1_gNB`) that no greedy fill can. Derived by live re-probe
   (`layout_at` LINELEN/RIBBON) after finding such cells in the corpus.
3. **Per-group budget = proportional** (`generate::group_widths`):
   `B_i = max(round(87·flat_i/T), 20)`, replacing round-7's smallest-flat-first.
   Best of every allocator tried on the full corpus census.

**Result (corpus census, `tests/fill_census.rs`, 12 022 dot, 142 540 wrapping
prem/concl cells).** Wrapping-cell byte-exactness **44 % → 81.1 %** (single-cell
94.7 %, multi-cell 80.0 %). The Wide-rule probe is byte-exact for ALL cells
(`wide_conclusion_group_fill_byte_exact`); `ragged_fill_line0_shorter_than_line1`
pins the ragged fill. GRAPHCLEAN_CORPUS round-trip stays **12022/12022**.

**Residual, characterized honestly (§3f, §8).** ~19 % of wrapping cells: (a) ±1
`fits` boundary where proportional is a column off the reference's coupled per-cell
`fits`; (b) cells wrapped on their UN-abbreviated width; (c) `++`-union / deep
function-application internal breaks not in the fact/tuple grammar. All need the
reference's exact per-cell `fits` over the pre-abbreviation document, structurally
outside a crate that consumes post-abbreviation cell text.

**Probes logged** in QUERIES.log Session 8 (probe.spthy 70 rules on :3200; all
servers stopped, ports 3200-3299 clear). No forbidden paths read.

--------------------------------------------------------------------------------
## Round 9 report — cracking the multi-cell allocation (folded here per protocol)

**Task.** Invert the corpus through the now-exact engine (round 8): extract
per-cell width BANDS (the set of engine widths reproducing each observed cell),
fit the allocation function against thousands of band constraints, split
remaining candidates with live probes, implement, and re-census.

**Tooling.** `band_dump` (src/bin): for every prem/concl group in a dot corpus,
per cell, the L-space band (ribbonsPerLine 1.5) at which the engine reproduces
the observed bytes, plus shape features (top-level tuple-arg elem counts, quoted
constants, single-quoted-atom flag, function nodes, abbreviation tokens). Runs
over the 12 022-file corpus; also over all probe captures.

**Engine fix en route.** The strict HughesPJ port materialized exponential union
trees (2.7 s / 1 GB for a 20-element tuple fill; probe extraction hung). Fixed by
mirroring the Haskell laziness: `Doc::Lazy` thunks (forced once, memoized) at
every union branch / continuation the sanctioned source leaves un-forced, and
`fits` forcing only a candidate's first line. Byte-equivalence verified: all
tests, fill-census numbers identical to round 8, corpus round-trip 12022/12022
(now 3 s). 2 ms for the 20-element fill.

**Method & findings.**
1. Corpus bands alone REFUTED every single-parameter proportional family (pairs
   of narrow bands are mutually unsatisfiable, e.g. [42,116] needs b0 ≤ 21 with
   b1 ≥ 60), and refuted `b_i = 87 − mw(sibling)` fixpoints.
2. Live battery #2 (probe2, 56 rules): premise-width coupling REFUTED (byte-
   identical conclusions across premise 17…127); order effects REFUTED;
   group-as-one-HughesPJ-doc REFUTED computationally (`groupdoc_lab`).
3. Live battery #3 (probe3, 56 rules; 1-column sib steps + equal pairs/triples +
   mixed pairs) exposed shape anomalies (a 31-flat atom-sib wrapping at row
   total 86; a 71-flat tuple-fact fitting at total 90) and pinned the trigger:
   occupancies C = flat + (2n−4 per top-level tuple arg), +4 self-budget bonus
   for ≥3-elem tuple-facts in multi-cell rows, quoted-atom −2 fit-check
   discount, floor 20, lone-cell budget 87 — **0 errors on 343 probe cells**,
   corpus trigger error 1.05 % (flat-sum: 1.45 %). Reads as the reference
   measuring INTERNAL term renderings (tuples = right-nested pairs, quotes
   dropped).
   **[CORRECTED, round 10: the quoted-atom terms are refuted. The 343-cell
   grid stepped by 2 through the critical windows, so both the −2 own-width
   discount (implemented) and the `C(sqa) = flat − 4` occupancy form (logged
   in QUERIES.log Session 9 / fit2.py) fit it; they are inconsistent with
   each other and BOTH fail the round-10 odd/even completion battery. The
   probed truth is NO quote correction anywhere; the tuple `2n − 4` term and
   the flat `+4` bonus were likewise grid artifacts — the round-10 size-law
   batteries pin `elems + 1` occupancy and the `⌊n/2⌋ + 2` bonus (§3f
   Session-10 bullet).]**
4. Battery #4 (probe4 Q-series) killed the fill-saturation hypothesis: the
   [Big 87, atom s] fill follows 87²/(87 + 5s/6) for s = 12…120. Fill =
   proportional over display flats with single-quoted-atom siblings at 5/6,
   half-up rounding, clamped to [20, flat−1]. Probe fill-band hit 96.9 %.
5. Implemented in `generate::group_widths` (cells passed as flat texts; shape
   parsing in-crate). Census (12 022 files): **all cells 95.572 %** (prop87
   95.451 %), wrapping cells **81.59 %** (81.09 %), multi-cell wrapping
   **80.45 %** (79.90 %), false-flat 1000 (1843). Round-trip 12022/12022; all
   55 tests green; `Wide`/`St_1_gNB`/E-W fixtures unchanged.

**Residual, characterized honestly.** The corpus fill-band ceiling is 90.94 %
of banded wrap cells (93.6 % on cells with no abbreviations/functions in the
group); 84 % of misses involve abbreviation names whose internal (unabbreviated)
widths drive the reference's decisions — structurally invisible post-
abbreviation; 9.9 % of wrap cells are band-NONE (cell-doc gap: ++-unions, deep
nesting, abbreviation-expansion layouts). These are the honest limits of a crate
consuming post-abbreviation cell text; the model would accept caller-supplied
internal occupancies to close family (a).

**Probes logged** in QUERIES.log Session 9 (probe2/3/4 on ports 3200-3202, all
servers stopped, ports verified clear). No forbidden paths read.

--------------------------------------------------------------------------------
## Round 10 report — audit redo, size laws, union/func documents, width interface (folded here per protocol)

**Task.** (1) AUDIT REDO: the round-9 single-quoted-atom correction was logged
inconsistently (QUERIES Session 9 / fit2.py: `C(sqa)=flat−4` sibling-occupancy;
shipped code / §3f: own `flat−2`); probe the truth and reconcile. (2) Pin the
function-node / multi-quote corrections round 9 could not. (3) Attack the
band-NONE cell-doc gap. (4) Extend the interface for caller-supplied widths.

**Batteries** (all on ports 3200–3205, serve wrapper with OOM guards, servers
stopped; ~214 rules total): R10-A audit (RA/RB/RC/RD/RE — odd/even completion
of the r9 grids), R10-B functions (FB occupancy sweep, FC/FD/FE lone + squeezed
func fills), R10-C quotes (QA 2-quote, QB quote+var, QC quote-in-tuple),
R10-D unions (UA/UB/UC/UD display + fills, DN chains), R10-E quote-position ×
tuple + pure-var controls + 5-union pins (PA–PF, UEV, UG, DN_2), R10-F size
laws (TN/TB tuples n = 2,4,6; UN/UB unions n = 3,8).

**Findings.**
1. AUDIT: BOTH round-9 sqa forms are wrong. The archived LA/LB rows already
   refute the logged −4 occupancy (partner flips at 43, not 47); the new
   RA_44/RC_44/RB_42/RB_43/RD_32/RD_33 rows refute the shipped −2 own
   discount (443/446 for no-correction vs 440/446); every sqa variant is
   corpus-identical. Resolution: NO sqa trigger correction. The lone
   surviving probe residue is the `[45-partner, budget+1]` relief, later
   re-observed for func/quote/tuple/union sibs alike (9 occurrences, one
   pattern).
2. SIZE LAWS: occupancy `C = flat + (elems+1)` per top-level tuple/union arg
   (boundary-exact at every probed n; refutes round-9's `2n−4`), self-budget
   bonus `⌊n/2⌋+2` (n ≤ 8; the r8 16/20-elem rows force ≤ 4 beyond), quote
   position/count in tuples irrelevant, func/multi-quote facts plain. The
   round-9 corrections were artifacts of step-2 grids (both laws coincide at
   n = 5).
3. CELL DOCS: funcs break internally after `name(` with an attached `)`;
   unions display `(a++b)` and break after `++` with a tuple-style `)` peel;
   both implemented recursively in `arg_doc` and byte-locked by probe
   fixtures. A dewrap bug (union-`)` peels mis-reconstructed) was also fixed
   in the census/band tooling.
4. FILL: numerator = internal width (`+ elems+1` per union arg, `+1` per
   function node; tuple receivers stay at display flat — `Wide` byte-pins
   it), denominator = sibling occupancies `C_j`, quoted-atom sibs 5/6 for
   tuple receivers. Probe fill byte-exactness 525/547 (96.0 %).
5. INTERFACE: `CellWidths { occupancy, bonus, fill_width }` +
   `group_widths_with` + `RawRule::{premise,conclusion}_widths`; absent
   inputs are byte-identical to the estimates (regression-gated).

**Result (acceptance gates).** All 73 tests green; corpus round-trip
**12022/12022** byte-exact; allocator 12022/12022. fill_census (615 850 cells,
142 540 wrapping): all cells 95.572 % → **96.580 %**, wrapping 81.59 % →
**86.26 %**, single-cell 94.75 % → **97.41 %**, multi-cell 80.45 % →
**85.29 %**, false-flat 1000 → **819**. Probe-battery wrap byte-exactness:
A 42/44, B 19/20, C 32/34, D 12/12, E 64/68, F 47/51, r9-p2 129/133,
r9-p3 86/89, r9-p4 9/10, r8 85/86 (total 525/547). Corpus band-NONE
14 695 → **8 305** (5.8 %): plain 9, abbr 14, func 646, func+abbr 7 636.

**Probes logged** in QUERIES.log Session 10 (probeA–F on ports 3200–3205, all
servers stopped, ports 3200-3299 verified clear). No forbidden paths read.

--------------------------------------------------------------------------------
## Round 11 report — multi-cell fill allocation, relief, tuple hang, abbreviation refutation (folded here per protocol)

**Task (open-side round-10 measurement relayed four residue families).**
(1) FAMILY 1 (dominant): multi-cell fill allocation mispredicts break
positions. (2) FAMILY 2: abbreviated cells apparently laid out on internal
text. (3) FAMILY 3: false-positive wraps beside a wrapping wide sibling.
(4) FAMILY 4: lone-abbreviated-cell false negatives; extend the interface.

**Method.** Corpus miss inventory from the captures (bands6-9 re-dumps with
recursive shape features; eval/pairfit/seqfit/variants offline fitting against
band constraints), then five live batteries (G/H/I/J/K, 198 rules, ports
3200-3204, serve wrapper OOM-guarded, all servers stopped) designed around the
miss-cluster centroids, then implementation + full gates.

**Findings → laws shipped** (each pinned in §3f Session-11 bullet):
1. Fill rounding is **half-down** (equal-pair probes).
2. Occupancy is **recursive** (`elems+1`; nested-in-tuple `elems−1`; full
   inside func args).
3. The self-budget bonus is **gated on the LAST argument** being the
   tuple/union (WIT); much of the round-10 "bonus" is really…
4. …the **relief pass** (family 3): wrapping cells re-checked against the room
   siblings actually occupy (truly-broken sibs charge their fill; peel-only
   sibs their C; no bonus).
5. The fill numerator **includes tuple surcharges capped at 7** (K3 pins
   pair +3 / 6-tuple +7; r8 16/20-elem grids force the cap; Wide fixture
   still byte-exact).
6. The **tuple opener hangs** (`<` left at line end, elements below at the
   fill column; also inside func args; fact/func openers never hang) —
   implemented as a zero-width leading fill item; this closed nearly the
   whole band-NONE family (6,612 → 854 corpus rows).
7. **Families 2 and 4 dissolved under probing**: battery J (abbreviation
   theories with `?unabbreviate=` twins) shows the reference lays out
   POST-abbreviation display text (internal widths 96–150 render flat;
   sibling budgets follow display C). The family-4 corpus witness is a
   2-cell row wrapping at its bonus-free budget by `)`-peel. The
   internal-text/substitution cell-document the relay requested was NOT
   built — it would model behavior the reference demonstrably does not
   have; instead `CellWidths.trigger_width` (self-width override, both
   trigger passes, incl. lone cells) was added for adapter flexibility,
   regression-gated.

**Result (acceptance gates).** 80 tests green (7 new round-11 fixtures);
GRAPHCLEAN_CORPUS roundtrip **12022/12022**; alloc_corpus 12022/12022.
fill_census (615,850 cells, 142,540 wrapping): all cells 96.580 % →
**98.794 %**, wrapping 86.26 % → **95.60 %**, single-cell 97.41 % →
**100.00 %**, multi-cell 85.29 % → **95.21 %**, false-flat 819 → **324**,
false-wrap 1,478 → **1,150**, fill misses 18,764 → **5,954**. Probe-battery
wrap byte-exactness (fill_census over every battery ever captured):
G 84/86, H 55/58, I 99/101, K 63/70, A 43/44, B 19/20, C 33/34, D 11/12,
E 66/68, F 50/51, p2 133/133, p3 81/89, p4 7/10, r8 78/86 → **822/862**
(95.4 %). Absent-override behavior byte-identical (regression-gated).

**Honest residue.** The ±1 coupled-`fits` relief boundary (IA_65_23,
`[45,43]`/`[46,42]`), large-tuple numerator approximation (cap 7), ~850
band-NONE rows (deep func-chain + abbreviation-expansion layouts — NOT an
internal-width effect). Old probe subset regressed slightly (525/547 →
521/547) while the corpus gained ~13,500 byte-exact cells; the p3/p4/r8
losses are all big-tuple fill ±1s.

**Probes logged** in QUERIES.log Session 11 (probeG-K on ports 3200-3204, all
servers stopped, ports 3200-3299 verified clear). No forbidden paths read.

--------------------------------------------------------------------------------
## Round 12 report — corpus-residue attack: slack law, relief charge, ftup occupancy (folded here per protocol)

**Task (open-side corpus measurement relayed three families).** Against the
12,022-payload corpus: FAMILY A (dominant, both-wrap fill breaks one element
apart), FAMILY B (trigger false-negatives — we flat, reference wraps, often a
`)`-peel), FAMILY C (residual false-wraps).

**Method.** Baseline gates re-run byte-identical to round-11 finals. Corpus
miss inventory over bands9 + all archived probe re-dumps (eval12/frac12/dq12/
variants12*): FM misses band-edge-hug (+1 dominant), needed-Δq correlates with
nfunc/nq (negative) and rec7=6/sqa-sib (positive); FF cells sit at bonus-free
margin exactly = their granted bonus; offline law competition narrowed the
axes; then four live batteries (L/M/N/O, 111 rules, ports 3200-3203, serve
wrapper OOM-guarded, servers stopped) around the cluster centroids; band_dump
extended to 22-field cells (`smax`, `ftup`); final grid selection; implement +
full gates.

**Findings → laws shipped** (each pinned in the §3f Session-12 bullet):
1. Pass-1 slack `⌈e/2⌉−1` (cap 4) over top-level tuple/union args, ANY
   position — replaces the last-gated `⌊e/2⌋+2` bonus (battery L; the old
   probe readings were relief artifacts). This alone dissolves family B's
   dominant [40w, 50 last-2-tuple] SndS class (bonus-free budget 47 < 50).
2. Occupancy AND fill numerator count function nodes INSIDE tuples/unions
   (`ftup`); top-level `nfunc` leaves the numerator (corpus [41w,51] witness
   + OD replica byte-pair; FB pins hold).
3. Relief charge = `min(hd(q_sib + 1/3), C_sib)` on the UNROUNDED quotient
   (battery M, gaps 3–8 byte-pinned; MC: q-based, not C), bump dropped for
   ≥ 4-element-tuple receivers (TB4/TB6/UEV/UB8 boundary pairs).
4. No quote discount (NB); numerator cap 7 holds at 8 elems (NC8); fill
   denominators use FULL recursive occupancy (ND16 refutes capped sibs).
5. The ΣC=88 budget-margin zone is TERMINAL: battery O proves closed-form
   impossibility (mutually contradictory roundings at equal fracs).

**Result (acceptance gates).** 82 tests green (2 new battery fixtures; the
WIT_79 assertion re-scoped to the ΣC=88 residue per LD4_68);
GRAPHCLEAN_CORPUS roundtrip **12022/12022**; alloc 12022/12022. fill_census
(615,850 cells, 142,540 wrapping): all cells 98.794 % → **98.933 %**,
wrapping 95.60 % → **96.45 %**, single-cell 100.00 % held, multi-cell
95.21 % → **96.14 %**, false-flat 324 → **172**, fill misses 5,954 →
**4,884**, false-wrap 1,150 → ~**1,477** (moved into the proven-terminal
zone; net divergent cells 7,428 → 6,533, −12 %). Probe-battery wrap
byte-exactness **979/1,023** (95.7 %) over every battery ever captured
(old-set subset 824/862 vs round-11's 822/862; B/C/E now 20/20, 34/34,
68/68; K 61/70 — K1/K3/K6 pre-existing + TB4f_48 ±1 collateral).
Absent-override behavior regression-gated (all four override fields).

**Honest residue, per relayed family.** FAMILY A (fill): 5,954 → 4,884
(−18 %); pinned: half-down + numerator/denominator terms above; still open:
large-tuple numerator approximation (K3/K6/r8), band-NONE ~1,640
(abbreviation-expansion deep-func layouts), ±1 band-edge roundings. FAMILY B
(false-flat): 324 → 172 (−47 %); pinned: the slack law; residue: ΣC=88-zone
cells whose reference wraps at budget+slack (e.g. [60 rec9, 26] margin-3
shapes). FAMILY C (false-wrap): 1,150 → ~1,477; the probe-forced tighter
slack EXPOSES the coupled zone (ref keeps flat at budget+1..+2 in about half
of the exact-88 rows — battery O proves no width-function decides them); this
regression is the honest price of probe-exact laws elsewhere and is bounded
by the zone's row population.

**Probes logged** in QUERIES.log Session 12 (probeL-O on ports 3200-3203, all
servers stopped, ports 3200-3299 verified clear). No forbidden paths read.
