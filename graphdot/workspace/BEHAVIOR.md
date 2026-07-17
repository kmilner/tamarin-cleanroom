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

**Graph options** [GAP]: query params on the graph URL
(`?simplification=`, `?abbreviate=`, `?level=`, `?compress=`, `?simplify=`, …)
produced byte-identical output; there is no per-request simplification-level
knob, and no CLI simplification-level flag. See §5.

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
  cluster; node `fillcolor` is the saturated per-role color. Exact hash **[GAP]**.

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
## 6. What the crate reproduces vs. gaps

Reproduced & byte-tested against captured payloads:
- DOT document assembly: both headers, block/whitespace rule, node lines
  (record/ellipse/plain), edge lines with ports & attrs, cluster subgraphs,
  legend sink-block + invis edges, empty graph.
  VALIDATION: a parser for this dialect ingests each captured payload into the
  crate's model; re-serializing reproduces the original bytes for **all 14705
  DOT payloads in the corpus (14705/14705 byte-exact)**, incl. every clustered
  and every largest graph. See `graph-clean/tests/roundtrip.rs`.
- Abbreviation naming (prefix derivation incl. operator map), per-prefix
  numbering over a supplied order, and legend-table HTML (incl. the 65-space
  hang indent and HTML escaping).
- Abbreviation SELECTION rule (§5c, REPORT2.md): renderLen ≥ 10 ∧ occ ≥ 2 ∧
  ¬tuple, bottom-up. Three necessary gates corpus-exact over 97 538 legend
  entries (0 counterexamples) and each confirmed by a controlled live probe;
  implemented as `abbrev::select` with tests. Exact for non-AC/DH terms.
- Cluster/compact trigger rule (role ≠ Undefined).

Documented gaps (need the GPL solver or an unavailable backend):
- JSON graph backend format (unavailable / not in corpus).
- Abbreviation of AC/DH sub-terms (§5c residual): occurrence is counted over the
  solver's normalised (AC-flattened, DH-normal-form) term, not the surface
  rendering — not derivable from output alone (~93% of §5c exceptions).
- Canonical per-prefix numbering tie-break (§5b); empty-prefix (numeric-constant)
  start index.
- Term line-wrapping inside record labels and the color hashes (§3a, §4).
- The constraint-system → graph compression content (§4).
