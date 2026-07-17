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
- **`trapezium`** (spec-named dual) was **NOT observed** in any probe (all case
  graphs of two theories, compressed + uncompressed) nor the corpus. Recorded as
  unobserved; the serializer accepts any shape string so it can be emitted if a
  caller supplies one.

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
  - **Delimiter peel.** The fact's closing `)` peels onto its own physical line at
    the **functor column** (col 0) whenever the fact breaks; a tuple's `>` peels to
    the **`<` column** when the last element fills the line
    (`Out( <'aa…(74)', 'y'` │ `>` │ `)`); an unbreakable atom wider than the budget
    overflows and only the trailing delimiters wrap.
  - **KNOWN RESIDUAL** (`fsep` lookahead): the underlying combinator's one-element
    lookahead lets a *continuation* line hold **one more** element than the first
    line at the same start column (first line 11, continuation 12 for the 5-col
    `'aNN'` elements). The width (87), the top-level fit, the first-line packing and
    the peel columns are byte-verified; reproducing the ±1 continuation lookahead
    and the boundary peel byte-for-byte would require the exact `fsep`/`nest` doc
    tree. `graph-clean::render` implements `FILL_WIDTH=87`, `fits_one_line`, and
    `paragraph_fill` (first-line-exact), tested against these captured probes.

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
  group-drop rule, record escaping, the wrap FORMAT (alignment indent + `\l`,
  reproducing an observed wrapped cell byte-exact), and the wrap **DECISION** —
  fixed width `FILL_WIDTH=87` with `fits_one_line` (top-level fit, boundary
  byte-verified) and `paragraph_fill` (first-line packing) in `src/render.rs`.
- **System → graph GENERATION** (§6): `generate` over the independent `System`
  model; reproduces a live source-case graph byte-exact (`tests/generate_tests.rs`).
- **Simplification/abbreviation options** (§7): `Options` query-string model; the
  **L1/L2/L3 level number is proven inert** (§7a) — channel-verified negative.
- Abbreviation naming, numbering, legend HTML (65-space indent), and the SELECTION
  rule (§5c, REPORT2.md), plus the cluster/compact trigger (§4).

Documented gaps (need the GPL solver or an unavailable backend):
- JSON graph backend format (unavailable / not in corpus).
- **Record-cell wrap residual** (§3f): the `fsep` one-element lookahead (±1 element
  on continuation lines) and the exact boundary delimiter-peel; the width (87),
  top-level fit, first-line packing and peel columns are pinned.
- **compress/compact content** (§4, §6): which nodes/edges a raw constraint system
  yields — a solver transform. (The L1/L2/L3 level distinction is no longer a gap:
  it is proven non-existent, §7a.)
- Per-rule/per-cluster **color hashes** (§3a, §4); the `trapezium` dual (§3d, unobserved).
- Abbreviation of AC/DH sub-terms (§5c residual, normalised-form occurrence).
- Canonical per-prefix numbering tie-break (§5b); empty-prefix start index.
