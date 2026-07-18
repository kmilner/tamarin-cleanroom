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

  - **RESOLVED (Session 9) — the group WRAP TRIGGER is shape-corrected flat-sum,
    exact on every controlled probe.** Three live probe batteries (157 crafted
    2-/3-cell rows: cross-row sweeps, order swaps, 1-column sib steps, equal
    pairs/triples, mixed breakable/unbreakable pairs — QUERIES.log Session 9)
    plus the r8 grid pin the wrap decision to:
    > Cell *j* occupies `C_j = flat_j + Σ_{top-level tuple args}(2·elems − 4)`.
    > Cell *i*'s trigger budget is `max(87 [+4 if cell i has a ≥3-elem tuple arg
    > and the row has ≥2 cells] − Σ_{j≠i} C_j, 20)`; it wraps iff its effective
    > width exceeds the budget, where a single-quoted-atom fact above the floor
    > measures `flat − 2` and everything else `flat`. A lone cell's budget is
    > exactly 87.
    This scores **343/343** probe cells (flat-sum: 13 errors) and, on the corpus,
    1.051 % cell error vs flat-sum's 1.450 %. The corrections read as the
    reference deciding wraps on an *internal* term rendering (tuples as
    right-nested pairs — surplus `2n − 4`; quoted constants without quotes),
    which also explains the previously-unexplained probe anomalies (a sib
    wrapping at row total 78; nothing wrapping at total 90).
  - **REVISED (Session 9) — the FILL share is proportional over display flats
    with a 5/6 discount for single-quoted-atom siblings**:
    `b_i = clamp(round(87·flat_i / (flat_i + Σ_{j≠i} w_j·flat_j)), 20, flat_i−1)`,
    `w_j = 5/6` for single-quoted-atom siblings else 1. Probed: the
    [Big 87, atom-sib s] fill follows `87²/(87 + 5s/6)` across s = 12…120 with
    NO saturation (fill-band hit 96.9 % of probe wrap cells; corpus 90.94 % of
    banded wrap cells). Trigger and fill are genuinely separate layers: the
    trigger residual (87 − ΣC) does not reproduce the fill bands.
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

- **Record-cell group WRAP** (§3f, Sessions 8–9): a **faithful HughesPJ port**
  (`pretty.rs`, from the sanctioned BSD `pretty` library, with the Haskell
  laziness mirrored via `Doc::Lazy` thunks + first-line-only `fits` — pure
  evaluation-order change, byte-identical, kills an exponential blowup on
  many-element fills) laid out at **ribbonsPerLine = 1.5** so the paragraph fill
  is RAGGED, plus the Session-9 **two-layer allocation**
  (`generate::group_widths`): shape-corrected flat-sum TRIGGER (343/343 probe
  cells; corpus trigger error 1.05 % vs flat-sum 1.45 %) and proportional FILL
  with 5/6-discounted quoted-atom siblings. Corpus census: all-cells
  **95.57 %**, wrapping cells **81.59 %** (single-cell 94.75 %, multi-cell
  **80.45 %**), false-flat predictions 1000 (was 1843). Reproduces the live
  `Wide` record and the ragged `St_1_gNB` fill byte-exact.

Documented gaps (need the GPL solver or an unavailable backend):
- JSON graph backend format (unavailable / not in corpus).
- **Record-cell wrap residuals** (§3f, Session 9) — the remaining ~18 % of wrapping
  cells, characterized by the corpus band census (bands3):
  (a) **pre-abbreviation widths** — 84 % of the fill misses are cells whose group
      contains abbreviation names (`KD19`, `EX1`, …): the reference decides both
      trigger and fill on the UN-abbreviated internal term widths, which a crate
      consuming post-abbreviation cell text cannot see. (The Session-9 occupancy
      model makes this concrete: `C_j` is the *internal* width; abbreviated
      siblings have a larger internal width than their display.) A caller that
      passes unabbreviated widths could close this family — the crate model is
      parameterized by cell texts and could accept explicit occupancies.
  (b) **cell-doc ceiling** — 14 095 wrapping cells (9.9 %) are reproducible at NO
      budget (band-NONE): `++`-union and deeply-nested function-application cells
      whose internal breaks the fact/tuple cell grammar does not model, plus
      abbreviation-expansion layouts.
  (c) the residual ±1 boundary flips on clean cells (clean-cell fill hit 93.6 %).
- **Shape-occupancy corrections for further shapes** (§3f Session 9): the probed
  corrections cover tuple args, quoted-atom facts, and arg-facts; function-node
  (`senc(…)`) and multi-quote corrections are visible in corpus false-negatives
  but not yet pinned by controlled probes (the Session-9 P-series shapes were too
  narrow to force transitions).
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
