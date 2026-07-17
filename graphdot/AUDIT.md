# Similarity audit — graphdot cluster (clean room vs GPL Haskell)

Auditor role: SIMILARITY AUDITOR (may read both sides). Discipline:
abstraction–filtration–comparison. A match is a VIOLATION only if it is expression
that could not plausibly come from observing program OUTPUT: mirrored internal helper
decomposition where alternatives exist, matching internal (non-API, non-output) names,
comments echoing Haskell comments, algorithmic expression matching where the observed
behavior admits materially different implementations, or any content present in the
Haskell source but in NO observable output.

## Sides compared

CLEAN (reimplementation):
- `workspace/graph-clean/src/model.rs` — syntactic DOT graph model
- `workspace/graph-clean/src/dot.rs` — byte-exact DOT serializer
- `workspace/graph-clean/src/term.rs` — small term model for legend expansions
- `workspace/graph-clean/src/abbrev.rs` — prefix derivation, numbering, legend HTML
- `workspace/graph-clean/src/lib.rs` — module glue

HASKELL (originals, GPL):
- `lib/theory/src/Theory/Constraint/System/Dot.hs`
- `lib/theory/src/Theory/Constraint/System/Graph/Abbreviation.hs`
- `lib/theory/src/Theory/Constraint/System/Graph/GraphRepr.hs`
- `lib/theory/src/Theory/Constraint/System/Graph/Simplification.hs`
- (secondary dirty-room Rust ports: `crates/tamarin-server/src/graph/{repr,abbreviation,simplify,options}.rs`,
  `handlers/dot.rs` — translations of the Haskell; used only to understand the mapping)

## Output-observability check (evidence the compatibility content is genuinely observed)

Grepped the captured DOT payloads at `oracle/captured_responses`
(→ `tamarin-rs/scripts/.web_hs_cache`, 81 payloads). Confirmed present in OUTPUT:
- compact header incl. `packmode="cluster"` (8 payloads); simple header `nodesep="0.3"`.
- legend `<TABLE … CELLPADDING="1">` (73 payloads); `role=` on record nodes (79 payloads).
- abbrev names as PREFIX+counter: `AE1 AM1 … FS… EX…`; the exact 65-space hang indent
  before continuation `<TR>`; HTML escaping `&amp;/&lt;/&gt;`; exp rendering `'g'^…`.
- `FS` (from `F_status`) and `AM` (from `AMF_UE_NGAP_ID`) both appear as OUTPUT names.

Consequence: every string constant and every naming/rendering behavior the clean side
reproduces is present in observed output, so it is compatibility content, not expression
copied from source.

Grepped the clean side for distinctive Haskell internal identifiers
(`judgeTerm`, `getTermPrefix`, `abbreviateTerm`, `makeRecursive`, `prefixMap`, `allNames`,
`relativeOccs`, `termWeight`, `topoSort`, `roleColor`, `simpleHash`, `generateValue`,
`renderBalanced`, `scaleIndent`, `dotNodeCompact`, `mergeLessEdges`, `tryHideNode`,
`compressSystem`, `transitiveReduction`, …): NONE present.

## VIOLATIONS

None. No finding survives filtration as expression that could not have come from
observation. No redo instructions are issued.

## Per-module analysis

### model.rs vs GraphRepr.hs — CLEARED (different abstraction level)
The clean model is a *syntactic* DOT model: `Header{Simple,Compact}`,
`Stmt{Node,Edge,Cluster,RankBlock}`, `NodeKind{Record,Ellipse,Plain}`,
`Record{columns,fillcolor,fontcolor,role}`, `Cell{port,text}`, `Ellipse`, `EndPoint`,
`Cluster{label,color,body}`, `RankBlock{rank,body}`. The Haskell `GraphRepr` is a
*semantic* model: `Node{nNodeId,nNodeType}`, `NodeType{SystemNode ru, UnsolvedActionNode,
LastActionAtom, MissingNode}`, `Edge{SystemEdge,LessEdge,UnsolvedChain}`,
`Cluster{cName,cNodes,cEdges}`. The two carry different data (records/ellipses/attrs vs
rule instances / premise-conclusion indices). No shared internal names, no mirrored
decomposition. Fields present are dictated by what the DOT output contains. Not a violation.

- `Role::UNDEFINED = "Undefined"` (model.rs:29) mirrors Haskell's
  `fromMaybe "Undefined" (getNodeRole node)` (Dot.hs:243) only because `role="Undefined"`
  is emitted into the DOT output (verified in 79 payloads). Sentinel derived from output.
- `infer_header` = "compact iff any node has a non-Undefined role" (model.rs:56-64).
  Haskell's actual trigger is `null grClusters` (Dot.hs:503). These are different internal
  predicates that happen to coincide behaviorally (clusters are built only from role-bearing
  nodes). The clean side chose a *different* expression of the same observable correlation.
  Not a violation.

### dot.rs vs Dot.hs — CLEARED (output strings + independent structure)
- `header_lines` Simple/Compact attribute lists (dot.rs:38-69) are identical in content
  and order to `setDefaultAttributes` / `setDefaultAttributesIfCluster` (Dot.hs:130-161).
  These are exact bytes emitted to output (verified) and their order is output-determined.
  Compatibility content. Critically, the Haskell per-attribute explanatory comments
  ("Slight increase to space out the graph", "Combine parallel edges", etc.) are NOT
  reproduced. Not a violation.
- Record/edge/endpoint/cluster/rankblock serialization (dot.rs:80-175) is plain Rust
  string formatting. Haskell builds via a monadic `Text.Dot` DSL (`D.record`, `D.node`,
  `D.edge`, StateT/ReaderT). Entirely different implementation strategy; the emitted bytes
  match because bytes are the observed contract. The `render_block` "blank line before `}`"
  rule (dot.rs:26-36) is reverse-engineered from output formatting. Not a violation.
- `subgraph "cluster_…"` prefix (dot.rs:165) is DOT output syntax present in payloads.

### term.rs vs Term.LTerm view (used in Abbreviation.hs) — CLEARED (renderings are output)
- `Term{Fresh,Pub,Msg,Const,Pair,Exp,Ac,App}` and its renderings `~n / $n / n / 'n' /
  <a, b, c> / base^e / (a op b) / f(a,b)` (term.rs:144-165) all reproduce surface syntax
  visible in node labels and legend expansions (verified `'g'^…`). The Ac/Exp-vs-App split
  is forced by observed output (AC operators and exp render infix; ordinary functions render
  prefix). Not a violation.
- `Ac` operator table `mult→* , union→++ , xor→⊕` and `root_symbol_name` mapping
  `mult/union/xor/exp/pair` (term.rs:57-83) use lowercase logical names, whereas Haskell
  derives the prefix from `show o` = `Mult/Union/Xor` and `BC.unpack s` = `exp`
  (Abbreviation.hs:109-116). Different source expression; identical observable prefixes
  (MU/UN/XO/EX seen in output). Not a violation.
- `for_each_subterm` is fully recursive pre-order (term.rs:104-118); Haskell `getSubTerms`
  is one level only (`t : ts`, Abbreviation.hs:213-215). Divergent, not mirrored.

### abbrev.rs vs Abbreviation.hs — CLEARED (closest calls documented below)
- Legend HTML (`TABLE_OPEN`, `legend_row`, `legend_html`, `escape_expansion`,
  abbrev.rs:19-151) reproduces exact output bytes: table/cell tags, `<FONT COLOR="#000000">`,
  the `=` cell, `&amp;/&lt;/&gt;` escaping, and the 65-space hang indent equal to
  `len(TABLE_OPEN)`. All verified in captured payloads; the indent was measured from output,
  not taken from the Haskell (whose layout comes from the `Text.Dot` pretty-printer, no such
  constant in source). Compatibility content. Not a violation.
- `Abbreviator` numbering: per-prefix `1,2,3,…` with dedup by full rendering
  (abbrev.rs:63-75). Names ARE the output (PREFIX+counter, verified). The clean side does
  NOT reproduce Haskell's collision-avoidance against `allNames`
  (Abbreviation.hs:135-137, 220-225), soft-limit / always-abbrev-weight
  (Abbreviation.hs:178), weight formula `judgeTerm` (Abbreviation.hs:83-101), or the
  legend `topoSortAbbrevs` ordering (Dot.hs:456-474) — all source-only machinery is absent.
  Not a violation.
- `select` (abbrev.rs:158-176) is an explicitly-labeled best-effort heuristic (count
  occurrences, size threshold, sort ascending by size). It is materially different from the
  Haskell greedy max-weight algorithm with subterm-occurrence decrementing
  (`computeAbbreviations`, Abbreviation.hs:166-188), and even sorts in the opposite
  direction. Not a violation.

#### Closest call (considered, CLEARED): prefix derivation
- Clean `prefix_for_symbol` (abbrev.rs:33-39):
  `chars.filter(is_ascii_alphabetic).take(2).flat_map(to_uppercase)`.
- Haskell `getTermPrefix` (Abbreviation.hs:107):
  `map toUpper . take (aoPrefixLength) . filter isAlpha`.
- The pipeline is the same three operations in the same order (filter-alpha → take-2 →
  uppercase). FILTRATION: the observed output forces exactly this semantics and even pins
  the operation ORDER. `F_status → FS` (present in output) can only arise from
  filter-then-take (take-then-filter would yield `F`); `h2 → H`, `KDF → KD`,
  `AMF_UE_NGAP_ID → AM`, `senc → SE` fix "first two alphabetic characters, uppercased,
  non-letters skipped, length 2." There is no materially different implementation of that
  forced behavior. Per protocol this is "identical behavior at the idea level" / "structure
  forced by the output format," not expression. CLEARED — no redo. (If a future reviewer
  wants extra distance anyway, the behavioral redo would be: derive the two-letter uppercase
  tag by scanning characters and collecting letters directly, rather than as a
  filter/take/map chain — but this is not required, as the current code is output-forced.)

### Simplification.hs / GraphRepr.hs clustering — NO CLEAN COUNTERPART
The clean cluster has no analog to `compressSystem`, `dropEntailedOrdConstraints`,
`transitiveReduction`, `tryHideNodeId` (Simplification.hs) or `addCluster`,
`groupNodesByRole`, `findConnectedComponents`, `extractBaseName`, `roleColor`/`simpleHash`
color hashing (GraphRepr.hs / Dot.hs:525-544). The clean side takes cluster label and color
as pre-computed inputs (model.rs:202-210). The one piece of source-only, non-observable
expression that would be a red flag if copied — the `simpleHash`/`generateValue`/`roleColor`
constants (31, 7, HSV 0.75/0.85, alpha 0.3) — is entirely ABSENT from the clean side.
Strong positive signal for non-access.

## Conclusion
The clean-room modules are an independent reimplementation. Every similarity to the Haskell
traces to observable OUTPUT (exact DOT/HTML strings, abbrev names, surface-syntax renderings)
or to behavior that the output forces. No mirrored internal decomposition, no shared internal
names, no echoed comments, and none of the source-only machinery (weight/greedy abbreviation,
collision avoidance, topo-sort ordering, color hashing, system simplification/clustering)
appears on the clean side. VERDICT: PASS. No redo instructions.

## Round 2 incremental audit

Scope: the abbreviation SELECTION policy added this round —
`abbrev::select` / `select_with` / `MIN_ABBREV_LEN=10` / `MIN_ABBREV_OCC=2` and the
tuple-exclusion + bottom-up-ordering it uses (`abbrev.rs:157-210`), the supporting
`term.rs` additions (`render_len`, `is_tuple`, `for_each_subterm`, `size`,
`term.rs:91-134`), and the `select_*` tests (`tests/abbrev_tests.rs:238-320`).
Haskell original: the term-judging / selection machinery in
`Graph/Abbreviation.hs` — `judgeTerm` (83-101), `computeAbbreviations`/`go`
(166-231), `getFactTerms`/`getSubTerms` (209-215), `defaultAbbreviationOptions`
(66-72). Key question posed: does the clean rule reproduce the Haskell EXPRESSION
(constants, weight formula, decomposition) or only observable BEHAVIOR?

### Side-by-side (abstraction–filtration)

| aspect | Haskell (Abbreviation.hs) | Clean (`select`) | observable? | disposition |
|--------|---------------------------|------------------|-------------|-------------|
| length gate ≥10 | `judgeTerm … \| termWeight < 10 = -1` where `termWeight = length $ render $ prettyLNTerm replacedTerm` | `render_len() >= MIN_ABBREV_LEN` (10) | YES — corpus floor is exactly 10 over 97 538 legends, and probe `'12345678'`(10)→abbrev vs `'1234567'`(9)→not | FILTERED (output-forced boundary) |
| occurrence gate ≥2 | `\| relativeOccs <= 1 = -1` | count over `roots` `>= MIN_ABBREV_OCC` (2) | YES — probe: 42-char×1 not abbreviated, 12-char×2 abbreviated | FILTERED |
| tuple/pair exclusion | `getFactTerms = filter (not . isPair) …` | skip `t.is_tuple()` | YES — 0/97 538 legends is a top-level tuple; probe left an 18-char tuple×2 inline | FILTERED |
| length measured on the **rendered string** | `length $ render $ prettyLNTerm` (of the abbrev-**applied** term) | `render_full().chars().count()` (fully **expanded**, no abbrev applied) | YES (measurable per legend) — and the two measures *differ* in the iterative case | FILTERED / DIVERGENT |
| product weight `relativeOccs * termWeight` | present (the ranking value) | **absent** | NO — a non-observable internal ordering weight | NOT reproduced — positive signal |
| greedy max-weight pick + per-subterm occurrence decrement loop (`go`) | present | **absent** (flat filter, no iteration) | NO | NOT reproduced |
| soft-limit `aoAbbrevsSoftLimit=10` / `aoAlwaysAbbrevWeight=30` cutoff | present (178) | **absent** | NO — source-only constants | NOT reproduced — positive signal |
| `relativeOccs` legend-only special case (`occs==1 && legendOccs==[1] → 0`) | present (97-101) | **absent** | NO | NOT reproduced |
| subterm enumeration | one level: `getSubTerms t = t : ts` (215) | **fully recursive** pre-order `for_each_subterm` | — | DIVERGENT (opposite depth), not mirrored |
| ordering of survivors | decreasing weight (`sortOn Down`, greedy-first) | **ascending** `render_len` (shortest/inner first) | numbering only partly observable | OPPOSITE direction — not mirrored |
| occurrence universe | graph `grNodes`/cluster nodes (192-205) | whole constraint system (`roots` = node-fact args **and** sequent terms), derived because legends carry terms drawn nowhere | observation-derived | DIVERGENT, not mirrored |

### Assessment of the key question
The clean rule reproduces the observable BEHAVIOR, not the Haskell EXPRESSION.
The three gates it keeps (len≥10, occ≥2, not-tuple) are each independently
witnessed by corpus statistics and by controlled boundary probes
(QUERIES.log / BEHAVIOR §5c), so reproducing them — including the two literal
constants 10 and 2, which the corpus pins directly — is compatibility content.
The predicate "len≥10 AND occ≥2 AND not-tuple" is the minimal expression of that
observed selection behavior and admits no materially different implementation of
the predicate itself; it is output-forced, not copied.

Decisively, the entire non-observable apparatus that surrounds those gates in the
source is ABSENT on the clean side: the product weight formula, the greedy
max-weight/decrement iteration, the soft-limit + always-abbrev-weight cutoff
(constants 10/30), the `relativeOccs` legend special-case, and the one-level
`getSubTerms`. Where the clean side had latitude it chose *differently* from the
source — full-recursive enumeration vs one-level, ascending-length ordering vs
greedy decreasing-weight, fully-expanded length vs abbrev-applied length, a
whole-system occurrence universe vs `grNodes`. These divergences are affirmative
evidence of derivation from observation rather than from the source.

No shared internal names (clean `select`/`select_with`/`render_len`/`is_tuple`/
`for_each_subterm`/`size` vs HS `judgeTerm`/`computeAbbreviations`/`termWeight`/
`relativeOccs`/`getSubTerms`/`getFactTerms`/`aoAbbrevsSoftLimit`). No echoed
comments — the clean comments cite the clean room's own corpus/probe evidence,
whereas the HS comments describe the weight design and its `cleandot.py` origin.

### VIOLATIONS (Round 2)
None. No finding survives filtration as non-observable expression. No redo
instructions. VERDICT: PASS.

## Round 3 incremental audit

Scope (the round-3 GENERATION delta per SPEC_ROUND3 / REPORT3): `src/alloc.rs`
(the `NodeIdAllocator`), `src/render.rs` (record-cell rendering + wrap FORMAT),
`src/generate.rs` (system→graph mapping + edge-style vocabulary), `src/options.rs`
(simplification/abbreviation flags + query string), and the `invtrapezium` support
in `src/model.rs` (`Shaped`) / `src/dot.rs`. Haskell originals (both sides):
`Constraint/System/Dot.hs`, `Graph/{Graph,GraphRepr,Abbreviation,Simplification}.hs`.
Key questions from the task: is the `NodeIdAllocator` / record-cell renderer's Haskell
EXPRESSION materially different or output-forced, given the corpus-derived
(12022/12022) provenance? Grepped the round-3 clean files for Haskell internal names
(`dotNodeCompact`, `mkNode`, `renderRow`, `renderBalanced`, `scaleIndent`,
`setDefaultAttributesIfCluster`, `mergeLessEdges`, `dotGenEdge`, `missingNode`,
`systemToGraph`, `computeBasicGraphRepr`, `goSimplificationLevel`, `goShowAutoSource`,
`prettyNodePrem`, `BoringNodeStyle`): NONE present.

### alloc.rs (`NodeIdAllocator`) vs Dot.hs record/node id allocation — CLEARED

- Clean allocator (alloc.rs:19–55): one `usize` counter; `record(n_cells)` takes
  `n_cells` port ids in cell order then one node id; `node()` takes one. Haskell has
  no explicit allocator — ids are produced monadically inside the `Text.Dot` DSL: a
  record's ports and node id fall out of `D.record attrs $ D.vcat . map D.hcat .
  map (map (uncurry D.portField))` (Dot.hs:310–312) over `filter (not . null)
  [ps,as,cs]`, and simple nodes from `D.node`/`D.nextId`. Materially different
  EXPRESSION (a plain incrementing counter vs `StateT`-threaded `nextId` in a
  pretty-printer monad). The ORDER it reproduces — global counter, emission order,
  one id per cell (premises→info→conclusions) then the node id, one id for other kinds
  — is output-forced: it is `id order == file order`, verified 12022/12022 in the
  corpus [mine_ids.py] and re-verified in Rust (tests/alloc_corpus.rs). Output-forced
  scheme + divergent expression → not a violation.

### render.rs (record-cell renderer + wrap FORMAT) vs Dot.hs `renderLNFact`/`renderRow`/`renderBalanced` — CLEARED

- Fact spacing `Name( a, b )` / `Name( )` (`pad`, render.rs:77–83), metachar escaping
  `< > { } |` (`escape_record`, 30–42), and the info-cell shape `#t : Rule[…]`
  (`render_info`, 88–95) reproduce surface bytes mined across the corpus
  (fact-vs-function spacing, escaping, info-cell census — QUERIES.log session 3). The
  Haskell produces those bytes via `prettyLNFact` (Dot.hs:225–233) plus the record
  `ruleLabelM` (330–339) and graphviz record-label escaping — an entirely different
  code path (a `Document` pretty-printer, not string concatenation). Compatibility
  content; different expression.
- The wrap FORMAT (`fill`/`join_wrapped`, render.rs:105–157): `\l`-separated physical
  lines with a `&nbsp;`-run continuation indent equal to the broken group's
  first-element column. This is the exact observed format [mine_indent.py/mine_wrap.py,
  188 192 cells]. Crucially the clean side documents the BREAK DECISION (the line-width
  budget) as a GAP and does not model it, whereas Haskell decides breaks with
  `renderBalanced 100 (max 30 . round . (* 1.3))` + `scaleIndent` (×1.5 leading
  spaces) (Dot.hs:357–379) — proportional column-width budgeting with magic factors.
  None of that non-observable width machinery appears on the clean side; `fill` is a
  generic greedy word-fill parameterized by an externally supplied width. Divergent,
  not mirrored.

### generate.rs (system→graph mapping + edge styles) vs Dot.hs `dotNodeCompact`/`dotEdge`/`dotLessEdge` — CLEARED

- Node kinds (generate.rs:85–99, 206–245): the clean side emits records, gray `!KU(
  m ) @ #t` ellipses, darkblue `Fact @ #t` ellipses, uncolored `#t : rule` compressed
  ellipses, and `invtrapezium` open targets. Haskell `dotNodeCompact` (Dot.hs:236–275)
  covers the same via `SystemNode`→record, `UnsolvedActionNode`→ellipse colored gray
  iff `any isKUFact` else darkblue, the compact boring-node `show v ++ " : " ++
  showDotRuleCaseName` branch (301), and `MissingNode`→`invtrapezium`/`trapezium`. The
  clean split into `Knowledge`/`Action` (by observed gray-vs-darkblue) and `Compressed`
  is a re-derivation from OUTPUT colors/labels, and the clean side OMITS what the corpus
  never showed — `LastActionAtom` (a bare `#t` ellipse) and `trapezium` (only
  `invtrapezium` was observed live; [L]/BEHAVIOR §3d). Under-modeling to the observed
  subset is affirmative independence evidence, not similarity.
- `build_record` drops empty premise/conclusion groups and always keeps info
  (generate.rs:281–297), reproducing `filter (not . null) [ps,as,cs]` (Dot.hs:312) with
  info never null. The group set is output-forced (mine_groups.py: 100% carry info,
  empty prem/concl dropped); the clean expression (explicit conditionals) differs from
  the Haskell `filter`. Not a violation.
- Edge-style vocabulary (`EdgeStyle`, generate.rs:114–151): a flat enum whose eight
  variants emit the exact observed attribute lists. Haskell instead computes attrs by
  PREDICATE — `check isProtoFact`/`isPersistentFact`/`isKFact` in `dotEdge`
  (Dot.hs:390–397) and `Reason`→color in `dotLessEdge`/`toColor` (599–606). The clean
  side even RE-CATEGORIZES by appearance (`KnowledgeDeduction = red,dashed` — which in
  the source is the `Adversary` *less-edge* reason, not a `SystemEdge`), i.e. it
  grouped by observed bytes, not by the Haskell's source-structural taxonomy. The 11
  fixed styles are the observed census (mine_content.py). Output-forced strings +
  divergent classification → not a violation.

### options.rs vs Graph.hs `GraphOptions` / `SimplificationLevel` — CLEARED

- Clean `Options {level, abbreviate, compact, compress, clustering}` (options.rs:40–52,
  default level 2 / abbreviate / compact / compress / no-cluster) vs Haskell
  `GraphOptions {_goSimplificationLevel(SL2), _goShowAutoSource(False), _goClustering,
  _goAbbreviate(True), _goCompress(True)}` (Graph.hs:56–73). The overlapping fields and
  their defaults are the observed UI/query semantics: `simplification=N` (default 2),
  `unabbreviate`, `uncompress`, `uncompact`, `clustering=true` were read out of the
  SERVED `tamarin-prover-ui.js` and confirmed by live level-0 diffs (QUERIES.log session
  3). Note the field sets DIVERGE — the clean struct carries `compact` (Haskell's
  compaction lives in `DotOptions.BoringNodeStyle`, Dot.hs:70, not in `GraphOptions`)
  and OMITS `showAutoSource`; and `query_string` (options.rs:73–87) reconstructs the JS
  parameter order (`uncompact=&uncompress=&…&simplification=N`), a served-artifact byte
  shape with no Haskell counterpart. Different field names (`level/abbreviate/compact`
  vs `_goSimplificationLevel/_goAbbreviate/_goCompress`), observation-derived. Not a
  violation.

### invtrapezium support (model.rs `Shaped`, dot.rs) vs Dot.hs `missingNode`/`dotPremC` — CLEARED

- `Shaped::invtrapezium` (model.rs:201–203) builds label `(#var, idx)` with
  `shape="invtrapezium"`; dot.rs:99–105 serializes `label="…",shape="…"[,color]`.
  Haskell `dotPremC` (Dot.hs:281) is `missingNode "invtrapezium" (prettyNodePrem prem)`
  and `missingNode shape label = D.node [("label", render label),("shape",shape)]`
  (280). The `(#i, N)` label text, the `invtrapezium` shape string, and the
  `label,shape` attribute order are all observed live output ([L] fixtures
  nsl_invtrap.dot / invtrap_compressed.dot; BEHAVIOR §3d). Compatibility content; the
  clean side additionally notes the `trapezium` dual as spec-named-but-unobserved
  rather than emitting it — matching only what the probe showed. Not a violation.

### VIOLATIONS (Round 3)
None. The `NodeIdAllocator` and the record-cell renderer reproduce corpus-forced byte
schemes (id order == file order 12022/12022; fact spacing / escaping / wrap indent
mined at scale) through expressions materially different from the Haskell `Text.Dot`
monadic DSL and the `renderBalanced`/`scaleIndent` width machinery (which is a
documented GAP on the clean side, absent entirely). The generation mapping, edge
vocabulary, options, and invtrapezium are observed output or served-JS/live-probe
behavior, and where the clean side had latitude it diverged (under-modeled to the
observed subset, re-categorized edges by appearance, dropped `trapezium`/`showAutoSource`/
`LastActionAtom`). No mirrored decomposition, no shared internal names, no echoed
comments, none of the source-only apparatus (color hashing, `renderBalanced` budget,
system simplification/clustering). Findings that survive filtration: 0. No redo
instructions issued. VERDICT: PASS.

## Simplification re-probe audit

Scope (the newest session's delta — Session 4 in `workspace/BEHAVIOR.md`; identified
by git-less mtime: `src/render.rs` at 16:48, the sole code file touched, ~2 h newer
than every other src/test file, all Round-3 era ≤14:57; `options.rs` UNCHANGED at
14:53). The delta is the record-cell wrap **DECISION** now pinned in
`src/render.rs`: the constant `FILL_WIDTH = 87` (37–42), `fits_one_line` (44–52),
`paragraph_fill` (160–207), and their inline Round-4 tests (305–352). The
"simplification" half is a *re-probe*, not new code: Session 4 re-affirmed that the
`simplification=N` level number is inert (BEHAVIOR §7a) and left `options.rs`
untouched. Haskell originals audited (both sides): the wrap machinery in
`Constraint/System/Dot.hs` (`renderRow`/`renderBalanced`/`scaleIndent`,
Dot.hs:357–379, and `fsep`-based `renderLNFact`/`prettyLNFact`, 225–233/268) and the
whole of `Graph/Simplification.hs` (`simplifySystem`/`compressSystem`/
`transitiveReduction`/`dropEntailedOrdConstraints`/`tryHideNodeId`).

Grepped the clean side for Haskell wrap/simplification internal names
(`renderBalanced`, `scaleIndent`, `renderRow`, `renderStyle`, `OneLineMode`,
`lineLength`, `usedWidths`, `widthRender`, `simplifySystem`, `compressSystem`,
`transitiveReduction`, `transRed`, `reachableSet`, `dropEntailed`, `tryHideNode`,
`LessAtom`, `rawLessRel`, `Dag`) and for the source magic constants
(`100`, `1.3`, `30`, `1.5`): **NONE present** in `src/` or `tests/`.

### Wrap DECISION — `FILL_WIDTH=87` / `fits_one_line` / `paragraph_fill` (Dot.hs `renderBalanced`/`scaleIndent`/`fsep`) — CLEARED

| aspect | Haskell (Dot.hs) | Clean (`render.rs`) | observable? | disposition |
|--------|------------------|---------------------|-------------|-------------|
| line-width budget | `renderBalanced 100 (max 30 . round . (*1.3))` — a *proportional* per-row balance: total 100 cols split by each cell's one-line width, ×1.3, floored at 30 (357–374) | flat absolute `FILL_WIDTH = 87` from column 0 | YES — a live one-column width sweep across functor lengths 2/3/6/10 pinned the single-line→wrap boundary at flat 87 fits / 88 breaks, functor-invariant (QUERIES.log S4) | FILTERED (output-forced boundary); constant materially different (87 vs 100/1.3/30) |
| the break itself | HughesPJ `renderStyle{lineLength=w}` with `fsep`-fill inside `prettyLNFact` | `fits_one_line` = `chars().count() <= 87`; `paragraph_fill` = hand-rolled greedy loop, separator trails, first-line-exact | YES (boundary + first-line packing byte-verified vs captured probes) | FILTERED / DIVERGENT expression (char-count + plain loop vs pretty-printer monad) |
| leading-indent scaling | `scaleIndent` multiplies leading-space runs ×1.5 (375–379) | **absent** — indent is literal `&nbsp;`×`open_col` from the FORMAT observation (Round-3), no scale factor | NO (a source-only transform on spaces) | NOT reproduced — positive signal |
| proportional row balance (`ratio`, `usedWidths`, total 100) | present | **absent** | NO | NOT reproduced — positive signal |
| `fsep` one-element continuation lookahead + closing-delimiter peel | present (emergent from `fsep`/HughesPJ) | **deliberately NOT modelled** — documented as KNOWN RESIDUAL / GAP (render.rs:29–33, 170–175); `paragraph_fill` packs the first line exactly and a continuation one element short | partially (the ±1 shows in probes) | DIVERGENT under-model — affirmative independence |

The clean width is an *opaque measured* constant (87), not a reverse-engineering of
the source formula: the whole 100 / 1.3 / 30 / 1.5 balance-and-scale apparatus is
absent, and the clean printer is a flat-width greedy fill rather than a
proportional per-row budget fed to a pretty-printer monad. Materially different
expression reproducing an output-forced boundary. Not a violation.

**Closest call (considered, CLEARED): the word `fsep` in two comments**
(render.rs:29, 170) names the HughesPJ combinator tamarin actually uses. FILTRATION:
(1) it appears only in prose describing the residual the clean side does **not**
implement — it is not program expression and changes no emitted byte; (2) `fsep` is
public `Text.PrettyPrint` library API, not a tamarin-internal identifier; (3) the
observed fingerprint it names (a continuation line holding one more element than the
first line at the same start column) *is* the defining behavior of a fill-with-
lookahead, so attributing it to an "`fsep`-style combinator" reads as behavioral
characterization / general Haskell knowledge, not recitation of hidden source. The
clean CODE diverges from `fsep` rather than matching it. CLEARED — no redo. (Optional
distancing, not required: describe the residual purely behaviorally — "a fill
combinator with one-element lookahead" — without naming the combinator. Flagged for
the transcript auditor's glance, but it carries no expressive similarity.)

### Simplification re-probe vs `Graph/Simplification.hs` — NO CLEAN COUNTERPART

The Session-4 re-probe concluded "the `simplification=N` number is inert; L1≡L2≡L3
byte-identical" (BEHAVIOR §7a) and added no code. The clean side has **zero** analog
to `simplifySystem`, `compressSystem`, `transitiveReduction`,
`dropEntailedOrdConstraints`, or `tryHideNodeId` — none of the DAG/`transRed`/
`reachableSet`/`LessAtom` ordering machinery, no shared names, no mirrored
decomposition; `options.rs::level` is carried but documented inert and the
compress/compact *content* is left a solver GAP (caller supplies the node set).
Notably the clean "level is inert" conclusion is behaviorally **wrong** against the
source — `simplifySystem` genuinely branches (`i==3` → full `transitiveReduction`,
`i==2` → partial keeping `Formula`/`Adversary` less-atoms, else no-op), differing
only on graphs carrying redundant ordering (`Less`) edges, which none of the probed
graphs had. An incorrect black-box inference that contradicts the source is
affirmative evidence of NON-access, not similarity. (The wrongness is a
behavioral-parity note for the acceptance team, outside the similarity remit.) Not a
violation.

### VIOLATIONS (Simplification re-probe)
None. The wrap DECISION reproduces an output-forced boundary (flat width 87,
live-swept and functor-invariant; first-line packing byte-verified) through an
expression materially different from the Haskell `renderBalanced`/`scaleIndent`
proportional-budget + `fsep`/HughesPJ printer — every source-only piece (the 100 /
1.3 / 30 / 1.5 constants, the proportional row balance, the ×1.5 indent scale, the
`fsep` lookahead and delimiter peel) is absent or explicitly left a GAP. The
simplification re-probe adds no counterpart to `Simplification.hs` and even reaches
a conclusion that contradicts it. No mirrored decomposition, no shared internal
names, no echoed comments. One closest-call (`fsep` in comments) considered and
cleared. Findings that survive filtration: 0. No redo instructions issued.
VERDICT: PASS.
