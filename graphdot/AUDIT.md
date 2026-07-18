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

## Round 5 incremental audit — generate() completion (clusters, node kinds, wrap wiring, pre-rendered entry)

Scope (this round's delta ONLY; clean-room HEAD `63ed8a9` → working tree, restricted
to `graphdot/`). Code files touched: `src/generate.rs` (cluster routing + the
`ClusterRef` / `RawRule` / `RecordSpec` types, `GraphNode::{Temporal,Shaped,RawRule}`,
`build_record` over `RecordSpec` + `wrap_cell`), `src/render.rs` (the record-cell
wrap+peel implementation: `wrap_cell`, `run_layout`, `split_top_commas`, `is_tuple`,
`layout_tuple`, `layout_fact`, `layout_info`, `width`, `Unit`, `PLine`), `src/lib.rs`
(re-exports only), plus `tests/generate_tests.rs` / `tests/roundtrip.rs` and the new
fixtures. Provenance docs: `workspace/BEHAVIOR.md` §3g/§3f/§4 and `QUERIES.log`
Session 5. Haskell originals (both sides), followed from the dot/graph rendering
entry point: `Constraint/System/Dot.hs` (`dotGraphCompact`, `dotCluster`/`roleCluster`,
`dotNodeCompact` incl. `LastActionAtom`/`UnsolvedActionNode`/`MissingNode`, the
`mkNode`/`renderRow`/`renderBalanced`/`scaleIndent` wrap machinery, `ruleLabelM`) and
`Constraint/System/Graph/{GraphRepr,Graph,Simplification,Abbreviation}.hs`.

### Provenance cross-check (every new behavioral claim traces to a logged probe / captured fixture)

Walked each behavioral claim in the new code against BEHAVIOR §3f/§3g/§4 and
QUERIES Session 5, and validated the fixtures against the actual corpus:
- Cluster emission order `[free nodes][clusters][edges][legend]`, clusters contain
  only records, free ids < clustered ids, contiguous per-cluster id ranges,
  first-appearance == id order — `analyze_clusters.py` census over 1457 files;
  witnessed by fixtures `cluster_process.dot` (live SAPIC) and `cluster_multi.dot`.
- `cluster_multi.dot` (79c16911ad179d51) and `last_timepoint.dot` (24a119958f784d43)
  are **byte-identical to the actual `oracle/dot_corpus/` files** (verified by diff),
  i.e. real captured OUTPUT, not fabricated expectations.
- Cluster label `<Role>_Session_<k>` — 9594/9594 census (and directly in the DOT).
- Bare-timepoint / `#last` ellipse — node-kind census (`#last` 107, `#i` 1328,
  `#decrypt` 790, …); `#last` as target of a `color="black",style="dashed"` edge
  (fixture 24a). `GraphNode::Temporal`/`last()`.
- Wrap width 87 (P=71 fits / 72 breaks), fact `)`-always-peel (E12), tuple `>` stays
  iff fits else peels to the `<` column (W72/W74), continuation packs greedily not
  +1-lookahead (E13→2/E14→3), info action-list = one-per-line vertical (6738eb64,
  b5a8773) — all in QUERIES Session 5; 7 live fixtures `wrap_E11..E14`/`W71/W72/W74`
  (the captured tails match the `wrap_cell` unit-test expectations byte-for-byte).
- The accumulated-column wrap **TRIGGER** (Wadler `group`/`fits`) is documented a
  RESIDUAL/GAP and NOT implemented (`measure_break.py` line-0 width 19..277; the
  `Y_counter` probe). No unlogged behavioral claim found.

### Grep sweeps (this round's files)
- Haskell internal names — `renderBalanced`, `scaleIndent`, `renderRow`,
  `widthRender`, `usedWidths`, `oneLineRender`, `roleCluster`, `createSubGraph`,
  `createClusterNodeId`, `extractBaseName`, `dotCluster`, `dotNodeCompact`,
  `simpleHash`, `generateValue`, `roleColor`, `LastActionAtom`, `UnsolvedActionNode`,
  `SystemNode`, `MissingNode`, `mkNode`, `dotGraphCompact`, `systemToGraph`,
  `grClusters`, `cEdges`, `mergeLessEdges`, `prettyLNFact`, `ruleLabelM`,
  `groupNodesByRole`, `addCluster`: **NONE present** in `src/` or `tests/`.
- Source magic constants — `100`, `1.3`, `1.5`, `30`, and the color-hash constants
  `31`, `7`, `0.75`, `0.85`, `0.3`, `hsv`/`floor`: **NONE** (the only `0.3` hits are
  the observable DOT attribute strings `nodesep="0.3"` etc.).

### Role CLUSTERS — `generate` bucketing (Dot.hs `dotGraphCompact`/`dotCluster`/`roleCluster`, GraphRepr `addCluster`/`groupNodesByRole`) — CLEARED
The clean side reproduces only the **observable statement order**: allocate ids in
`System.nodes` order, route each role-record's statement into its `cluster(label,color)`
bucket and everything else to a free top-level list, then emit free nodes → cluster
blocks (first-appearance order) → edges → legend. That order and the id-allocation
consequence (free ids < clustered ids) are directly visible in the DOT and verified
1457/1457. The Haskell instead consumes a pre-computed `grClusters` produced by the
SOLVER's role machinery — `groupNodesByRole`, `addCluster`/`createSubClusters`,
`findConnectedComponents`/`expandCluster`, `extractBaseName`, `addClusterByRole`
(suffix `"_Session_"`) — and emits clusters through the `roleCluster`/`D.createSubGraph`
StateT dance with the `roleColor`(`simpleHash`/`generateValue`) hash. NONE of that
appears on the clean side: cluster membership, label, and the 8-hex ARGB color are
solver-supplied INPUTS (`ClusterRef{label,color}`, a documented GAP), and the color
hash (`simpleHash` 31/7, HSV 0.75/0.85, alpha 0.3) is absent. The clean side's own
first-appearance `HashMap` + `cluster_order` bucketing is a materially different
expression. The per-cluster attribute block (`nodesep="0.6"`, `penwidth="2"`, `sep="4"`,
…) it serializes is exact output bytes and lives in the pre-existing `Cluster` model,
not this delta. Output-forced order + divergent expression + absent hash → not a
violation.

### Node kinds `#last` / bare timepoint (Dot.hs `LastActionAtom`→`mkSimpleNode (show v)`) — CLEARED
`GraphNode::Temporal{var}` (with `last()` == `Temporal{var:"last"}`) emits an uncolored
`#<var>` ellipse. The Haskell equivalent is the `LastActionAtom -> mkSimpleNode (show v) []`
branch of `dotNodeCompact`. The `#last`/`#i`/`#decrypt` labels and the plain-ellipse
form are captured OUTPUT (census + fixture 24a). Different name (`Temporal` vs
`LastActionAtom`), and the clean side derives the node kind from the corpus label
census rather than from the `NodeType` sum. Round 3 had *omitted* this kind as
unobserved-live; adding it now strictly on corpus evidence is observation-driven.
Not a violation.

### `GraphNode::Shaped{label,shape,color}` (generic) vs Dot.hs `missingNode shape label` — CLEARED
An explicit escape-hatch for shapes beyond the observed set (named for the unobserved
`trapezium` dual). Haskell's `missingNode shape label = D.node [("label",…),("shape",shape)]`
is the same two-attribute output shape (observable). The clean side does **not** emit
`trapezium` from any mapping (Session-5 re-scan of NSLPK3 cases confirmed it absent in
corpus/probe); it only exposes a caller-supplied generic node. Under-modeling to the
observed subset → affirmative independence, not similarity.

### Record-cell WRAP + delimiter PEEL — `wrap_cell` / `run_layout` / `split_top_commas` / `layout_{fact,tuple,info}` (Dot.hs `mkNode`/`renderRow`/`renderBalanced`/`scaleIndent`, `ruleLabelM`) — CLEARED (closest call)
This is the round's substantive addition and the closest call. Abstraction–filtration:

| aspect | Haskell (Dot.hs) | Clean (`render.rs`) | observable? | disposition |
|--------|------------------|---------------------|-------------|-------------|
| line-width budget | `renderBalanced 100 (max 30 . round . (*1.3))` — proportional per-row balance across a row's cells, fed to HughesPJ `renderStyle{lineLength=w}` | flat absolute `FILL_WIDTH=87` measured from the cell's own column 0 | YES (live width sweep W69–W74, functor-invariant) | FILTERED; constant materially different (87 vs 100/1.3/30) |
| the break | HughesPJ `fsep`-fill inside `prettyLNFact`, driven by `renderStyle` | hand-rolled greedy `run_layout` over `Unit{sep,text,indent,hard}`, char-count width | YES (byte-verified) | FILTERED / DIVERGENT expression |
| fill vs vertical split | `prettyLNFact` fact-args via `fsep`; `ruleLabelM` action list via `brackets (vcat …)` | `layout_fact`/`layout_tuple` greedy fill; `layout_info` vertical, dispatched by the flat string's SHAPE (`#…[…]`) | YES (info→one-per-line even when short: 6738eb64; args→greedy fill) | FILTERED (observed two modes); dispatch differs (output-shape heuristic vs builder identity) |
| delimiter peel (`)` to col 0, `>` to `<`-col iff overflow) | emergent from `fsep`/HughesPJ, no explicit "peel" code | explicit reconstruction in `layout_fact`/`layout_tuple` | YES (E12/W72/W74 fixtures) | FILTERED; clean side reverse-engineered explicit logic the source does not contain |
| string-splitting of flat cells (`split_top_commas`, nesting/quote aware) | NONE — Haskell operates on the structured `Doc`/term tree | required because the clean side wraps flat rendered strings | — | clean-room-only expression, no HS analog |
| proportional row balance (`ratio`, `usedWidths`, total 100) | present | **absent** | NO | NOT reproduced — positive signal |
| `scaleIndent` ×1.5 leading-space scale | present (375–379) | **absent** — indent is literal `&nbsp;`×open-col | NO | NOT reproduced — positive signal |
| accumulated-column wrap TRIGGER (`group`/`fits`) | present (emergent) | **documented RESIDUAL/GAP, not implemented** (wraps on the cell's own flat width) | partially | DIVERGENT under-model — affirmative independence |

The clean printer is a flat-width greedy fill over an explicit `Unit` list with an
explicit delimiter-peel reconstruction and a nesting-aware string splitter — a
materially different implementation from the Haskell proportional-balance
(`renderBalanced`) + `scaleIndent` transform feeding a HughesPJ `Doc` built by
`prettyLNFact`/`ruleLabelM`. Every source-only piece (100 / 1.3 / 30 / 1.5, the row
balance, the ×1.5 indent scale, the accumulated-column trigger) is absent or an
explicit GAP. The width 87 is an opaque measured boundary, not the source formula.
Output-forced boundary + divergent expression → not a violation.

**Closest call (considered, CLEARED): combinator names in comments.** `render.rs`
comments name `fillSep`, `sep`, `fsep`, and attribute the accumulated-column effect to
"tamarin's Wadler `group`/`fits`". FILTRATION, consistent with the Round-4 `fsep`
disposition: (1) these are public pretty-printer *library / textbook algorithm*
vocabulary, not tamarin identifiers — and verified against the source, tamarin's
`Text.PrettyPrint.Class` exposes `fsep`/`sep`/`vcat`/`nest` but has **no** `fillSep`
and **no** `group`/`fits` (those are Leijen/Wadler names tamarin does not use), so the
comment cannot be reciting tamarin source; (2) tamarin's printer is HughesPJ (`fsep`),
so the "Wadler `group`/`fits`" attribution is in fact slightly *wrong* about the
mechanism — an inference from general knowledge, not a reading of the source;
(3) the words appear only in prose characterizing observed behavior and a
not-implemented residual, and change no emitted byte; the clean CODE diverges from
`fsep` rather than matching it. CLEARED — no redo. (Optional, not required: describe
the two modes purely behaviorally — "greedy fill" / "one element per line" — and the
residual as "a break decision that depends on the whole record line" without naming
`fillSep`/`group`/`fits`. Flagged for the transcript auditor's glance only.)

### `RawRule` / `RecordSpec` pre-rendered interop — CLEARED (no HS counterpart)
`RawRule` (accept already-rendered cell strings) and `RecordSpec` (shared flat-content
struct feeding `build_record`) are clean-room API/refactor affordances. The Haskell has
no pre-rendered-string entry (it always renders from structured `LNFact`s); `RecordSpec`
is a Rust code-sharing intermediate with no analog. The interop path runs through the
same `wrap_cell`/escape pipeline (byte-parity test). No similarity concern.

### Simplification — no new counterpart
This round adds no code against `Simplification.hs`; the compress/compact content and
the wrap trigger remain documented solver GAPs. Consistent with prior rounds.

### VIOLATIONS (Round 5)
None. The cluster routing reproduces an output-forced statement/id order with the
solver's role/color machinery (`groupNodesByRole`/`addCluster`/`roleColor` hash) absent
and cluster identity/color taken as inputs; the `#last`/bare-timepoint and generic
`Shaped` node kinds are corpus-observed labels under a divergent type vocabulary; the
record-cell wrap+peel is a flat-width greedy fill with an explicit delimiter-peel
reconstruction and a nesting-aware string splitter — materially different from the
`renderBalanced`/`scaleIndent` proportional-budget + HughesPJ printer, with the 100 /
1.3 / 30 / 1.5 constants, the row balance, the ×1.5 indent scale, and the
accumulated-column trigger all absent or explicit GAPs. Corpus fixtures are
byte-identical to the captured payloads; every behavioral claim traces to a logged
Session-5 probe or captured fixture. No mirrored internal decomposition, no shared
internal names, no source magic constants, no echoed comments. One closest-call
(library/algorithm combinator names in comments) considered and cleared, consistent
with Round 4. Findings that survive filtration: 0. No redo instructions issued.
VERDICT: PASS.

## Round 6 incremental audit — the record-cell wrap TRIGGER (per-group shared budget)

Scope (this round's delta ONLY; clean-room HEAD `8901219` → working tree, restricted
to `graphdot/`). Code files touched: `src/render.rs` (new `cell_budget`,
`MIN_CELL_BUDGET = 20`, `count_info_actions`, `wrap_cell_budget`; `run_layout` gains
`budget` + `drop_break_space`; `layout_tuple`/`layout_fact`/`layout_info` thread the
budget; `layout_info` gains the ≥2-action-vertical branch; +5 unit tests) and
`src/generate.rs` (`build_record` → `group_cells`, per-group budgeting via
`cell_budget`/`wrap_cell_budget`). Provenance docs: `workspace/BEHAVIOR.md` §3f (the
"RESOLVED (Session 6)" rewrite), `QUERIES.log` Session 6, `REPORT2.md` Round 6.
(`INTEGRATION_REPORT.md` moved this round too but is unit-G/deriv-check content —
out of scope here.) Haskell original, both sides, followed from the record-render
entry: `Constraint/System/Dot.hs` `mkNode` record assembly (l.310-312), `renderRow`
(l.357-361), `renderBalanced` (l.363-379), `scaleIndent` (l.375-379), and the info
action-list `ruleLabelM` (l.335-338).

### What upstream actually does (abstraction)

`mkNode` renders each of the three groups separately — `renderRow` is called once on
`psM`, once on `asM` (the info/rule-label row), once on `csM` — then assembles
`D.vcat $ map D.hcat $ … $ filter (not . null) [ps, as, cs]` (drop empty prem/concl
groups; `as` is never null so info is always kept). `renderRow` runs
`renderBalanced 100 (max 30 . round . (* 1.3)) (map snd annDocs)` over one group's
docs: it measures each doc's flat width by `OneLineMode`, computes
`ratio = 100 / Σ flats`, and renders doc *i* through HughesPJ `renderStyle` at
`lineLength = max 30 (round (1.3 · ratio · flat_i)) = max 30 (round (130 · flat_i / Σ))`
(then `scaleIndent` blows leading spaces up ×1.5, `ribbonsPerLine` default 1.5). The
info action list is `brackets (vcat $ punctuate comma lbl)` — `vcat` is intrinsically
vertical, one action per line for ≥2 actions.

### Filtration — the new content is behavior-dictated, and independently derived

- **Group independence + within-group shared budget.** The clean rule ("each of
  premises/info/conclusions lays out independently; cells in one group share a
  budget") is structurally the same fact upstream exhibits (`renderRow`-per-group;
  `renderBalanced 100` shared across that group's docs). But it is a black-box
  *observable* (a conclusion's wrap ignores premise/rule-name width; adding one
  sibling flips it) and it is traced to a **logged probe** — QUERIES §6 probe1
  (`:3211`: Out flat87 stays flat under In flat 69/103/198 and rule-name len 40;
  one preceding conclusion flat12 makes it wrap). Behavior-dictated / merger — the
  black box has exactly one such structure to reproduce.

- **The FORMULA is a different expression and is measurably *less* exact than the
  source — strong positive evidence of independent derivation.** Upstream is a
  *proportional* per-cell line-length `max(30, round(130·flat_i/Σ))` fed to HughesPJ.
  The clean rule is *additive*: `budget_i = max(87 − Σ_{j≠i} flat_j, 20)`, wrap iff
  `flat_i > budget_i` (≡ `Σall > 87 ∧ flat_i > 20`). A source-copier would have
  transcribed `130·flat/Σ` and reached ~100 %; instead the clean rule reaches
  98.324 % of records / 99.635 % of cells with a documented ±1 residual the author
  attributes to the pretty-printer's ribbon `fits`. The two forms only *coincide on
  the wrap decision* because the clean constants are the ribbon-collapsed images of
  the source's (see next). No shared symbolic form.

- **Constants — the source magic numbers are ABSENT; the clean constants are
  observable composite boundaries pinned by probes.** Upstream's 100 / 1.3 / 30 / 1.5
  appear **nowhere** in the new code (grep-confirmed: no `1.3`, `1.5`, `130`, `ratio`,
  `renderBalanced`, `scaleIndent`, `usedWidth`, `oneLineRender`). The clean constants
  are `87` and `20`. `87 = round(130/1.5)` and `20 = 30/1.5` are precisely what the
  black box exposes once the source line-length is collapsed through HughesPJ's
  `ribbonsPerLine = 1.5` — i.e. they are the *only* boundaries a probe can land on,
  and each was landed on directly: the `q`-fixed 2-cell sweeps (probe2/3/4:
  q=11→76/77, q=28→58/61, q=48→39/40, q=68→19/22, each giving `budget+q = 87`) and
  the floor probe (Fb flat98 → target fits ≤20, wraps ≥21). These are behavioral
  boundary constants (as `FILL_WIDTH = 87` already was, accepted Rounds 4-5), not the
  transcribed source parameters.

- **Info "≥2 actions ⇒ always vertical."** This is the behavioral image of upstream
  `vcat $ punctuate comma lbl` (`vcat` = one item per line; `punctuate comma` = the
  trailing `,` on each non-final line, `]` on the last — both reproduced). Derived,
  not read: QUERIES §6 probe5-8 (`TwoShort` flat34, `ThreeShort` flat43, `AB` flat22
  all wrapped though far under 87) plus the corpus census (0 non-wrapped info cells
  carry a top-level action comma). Implemented as an explicit action count that forces
  the vertical `sep`; no `vcat`/`punctuate` name or structure is mirrored.

- **`drop_break_space` (fact break drops the trailing space, tuple break keeps `, `).**
  Both shapes are present in the captured corpus (`,\l` fact/info breaks and `, \l`
  tuple-element breaks both grep-hit `oracle/dot_corpus/`); the fact case is byte-tested
  against `ref_raw_1_1` (Ack) and the tuple case against the Round-5 `Out(<'a01'..'a12'>)`
  fixture (`'a11', \l`). Grounded in captured output.

- **Record assembly (`build_record`: drop empty prem/concl groups, keep info; ports).**
  Mirrors the *observable* group structure = upstream `filter (not . null) [ps, as, cs]`
  with `as` never empty; expressed in graphviz record/port vocabulary, no
  tamarin-internal names. Behavior-dictated, consistent with Round 5.

### Identifier / comment lineage

No identifier overlap with the source (`cell_budget`, `MIN_CELL_BUDGET`,
`count_info_actions`, `wrap_cell_budget`, `group_cells`, `drop_break_space`, `budget`
are all clean-room-coined; the source names `renderBalanced`/`scaleIndent`/
`widthRender`/`oneLineRender`/`usedWidths`/`ratio`/`conv` appear nowhere in the crate).
No echoed comments. The only source-adjacent prose is the residual attribution to
"HughesPJ `fits`"/ribbon in BEHAVIOR.md/REPORT2.md — `Text.PrettyPrint.HughesPJ` is the
standard Haskell library, not a tamarin identifier; it occurs in prose about a
*not-implemented* residual, emits no byte, and the clean CODE diverges from it (an
additive approximation, not the proportional `renderBalanced`). Consistent with the
combinator-name closest-call cleared in Rounds 4-5 — cleared again.

### Non-blocking observations (no redo)

- `count_info_actions` is defined and unit-tested but is **not** called by the
  production layout (`layout_info` decides vertical via `split_top_commas(content).len()`).
  It is dead/duplicate clean-room code with no source counterpart — a quality nit, not
  a similarity concern.
- `ref_raw_1_1.dot` (the Wide-rule witness) is a prior-agent capture not present as a
  file in the current tree; the In-vs-Big / Ack / Out(h) behaviors it witnesses are
  independently corpus-validated (both break shapes occur in `oracle/dot_corpus/`; the
  98.3 %/99.6 % match) and byte-embedded in the new unit tests. Minor traceability note.

### VIOLATIONS (Round 6)

None. The new per-group shared-budget trigger reproduces an *observable* wrap boundary
with a formula (`max(87 − Σothers, 20)`) that is structurally different from — and
measurably less exact than — the source's proportional `renderBalanced`/`scaleIndent`
machinery; the source constants (100 / 1.3 / 130 / 30 / 1.5) are absent, and the clean
constants (87, 20) are probe-pinned observable boundaries. The info ≥2-actions-vertical
rule and the fact/tuple break-space split are behavioral images of `vcat`/`punctuate
comma` and captured-corpus break shapes, each traced to a logged Session-6 probe or a
byte-embedded fixture. No mirrored internal decomposition, no shared internal names, no
source magic constants, no echoed comments. Findings that survive filtration: 0. No redo
instructions issued.
VERDICT: PASS.

## Round 7 incremental audit — the record-cell wrap FILL budget (how a wrapped cell packs)

Delta this round (working tree vs clean HEAD 75807c0, `graphdot/` only):
`src/generate.rs` (`group_cells` gains a two-budget model), `src/render.rs`
(`layout_cell` extracted), `tests/generate_tests.rs` +
`tests/fixtures/wide_{group,record}.dot` (one new byte-exact test), and the workspace
records `BEHAVIOR.md` §3f, `QUERIES.log` Session 7, `REPORT2.md` Round 7. Round 6
already cleared the wrap *trigger* (the flat-sum budget); Round 7 adds only *how a cell
that has been triggered to wrap packs its elements* — the FILL width — plus a local
refactor, a test, and residual documentation.

Reference re-read: `lib/theory/src/Theory/Constraint/System/Dot.hs`
`mkNode`/`renderRow`/`renderBalanced`/`scaleIndent` (357–379) and its
`Text.PrettyPrint.Class` import (46–58: `renderStyle`, `defaultStyle`, `lineLength`,
`OneLineMode`, `fsep`, `vcat`, `punctuate`, `comma`).

### The source machinery this delta could plagiarise — and does not

Upstream lays out a record row in ONE proportional pass (`renderBalanced 100 (max 30 .
round . (* 1.3))`): `usedWidths_i = length(oneLineRender doc_i)`, `ratio = 100/Σusedᵢ`,
each cell rendered at `lineLength = max(30, round(1.3·ratio·usedᵢ))`, then `scaleIndent`
multiplies leading-space runs ×1.5. It is a single `zipWith`, no sort, no iteration, no
per-cell floor other than 30, no separate "does it break" vs "how does it pack" split,
and it hands the width straight to HughesPJ, whose internal `fits`/`fillSep` does the
actual element packing.

The Round-7 clean code is a different construction on every axis:

- **Two budgets, not one.** `group_cells` computes a flat-sum *trigger*
  (`cell_budget = max(87 − Σ other flats, 20)`, decides whether a cell breaks) AND a
  smallest-flat-first greedy *fill* budget (decides packing once broken), capped at
  `flat − 1`. Upstream has a single `lineLength`. The trigger was cleared Round 6; the
  fill allocation is the only new machinery.
- **Subtractive + iterative, not proportional + single-pass.** The fill loop sorts
  cells by flat width and accumulates `alloc[j] = min(flat_j, budget_j)`, giving each
  later cell `max(87 − Σ allocations, 20)`. Upstream multiplies a proportional share.
  On the very datum the code targets (`[Ack 25, Big 68, Out 11]`): upstream
  `renderBalanced` yields per-cell line-widths `[31, 85, 30]`; the clean trigger yields
  `[20, 51, 20]` and the clean fill `[20, 56, 20]` — no numeric resemblance. The clean
  model is deliberately *less* exact than a transcription would be (its own bake-off:
  flat-sum 41.45 %, sibling-occ 43.29 %, this greedy 44.11 % of multi-line cells; a
  source transcription would approach 100 %). A model that underfits the source is
  affirmative evidence of non-access, not of copying.
- **Source constants absent.** 100 / 1.3 / 130 / 30 / 1.5 (the `* 1.3`, `max 30`,
  `scaleIndent` ×1.5) appear nowhere. The clean constants are `FILL_WIDTH = 87` and
  `MIN_CELL_BUDGET = 20`, both probe-pinned observable boundaries (batchB "alone → 87
  exact"; batchC "floor 20"), already accepted Rounds 4–6. FILTRATION + logged
  provenance.

Abstraction–filtration–comparison: the *idea* "cells share a bounded row width, a
sibling that wraps frees room so a neighbour packs one more element, floor the budget"
is behaviour dictated by the observable DOT and merges to few expressions; the greedy
`min(flat,budget)` allocation, the trigger/fill split, and the `flat−1` cap are
clean-room expression that diverges from — and underperforms — the source's proportional
`renderBalanced`. No protectable expression carried over.

### `render.rs` `layout_cell` extraction — pure local DRY, no source structure

The refactor lifts the pre-existing `if flat.starts_with('#') && flat.contains('[')`
dispatch (info-cell vs fact-cell) out of `wrap_cell_budget` into a named helper. The
predicate sniffs the observable *string* shape (`#t : Rule[…]`) because the crate
consumes POST-render text; upstream instead dispatches on the *typed* `Doc` built
differently per cell kind in `renderRow`/`ruleLabelM` (`vcat`/`brackets` vs `fsep`),
never on a `'#'`/`'['` string test. Different mechanism, driven by the clean side
receiving strings the Haskell side never has. No structure mirrored.

### Test + fixtures — provenance anchored

`wide_conclusion_group_fill_byte_exact` embeds `wide_record.dot` (the crate's record
line) and is verified byte-equal to `wide_group.dot`. Provenance: the `Wide` rule is the
prior-agent `wide.spthy` / `ref_raw_1_1` capture logged at QUERIES.log:275; the
`[Ack 25, Big 68, Out 11]` → Big-packs-8 datum is corpus/probe-backed (Session-7 batchA,
`vc_fill` 44.11 %). `wide_group.dot` is a genuine external full-graph capture — it
carries the `isend` action node, ten `!KU` nodes and 22 edges that the clean crate
(single `RawRule` → single record, as `wide_record.dot` shows) cannot itself emit — so
the fixture is not circular. Its record line is byte-identical to the crate's output.

### Documentation (BEHAVIOR.md / QUERIES.log / REPORT2.md) — every claim logged, residuals honest

The Session-7 prose decomposes the trigger residual (false-positive occ-relief 1 320;
false-negative 1 459, of which 954 are decided on the *unabbreviated* term; ~505 ±1
`fits`) and the fill `[GAP]`. These describe mechanisms the crate deliberately does NOT
implement — attributed to the unavailable GPL `fillSep`/HughesPJ `fits` over the
*pre-abbreviation* document. Every quantitative claim traces to a logged artifact:
batchA–G live probes (ports 3200–3205) and `vc_r7`/`vc_resid`/`vc_expand`/`vc_fill`
corpus scripts, all present under `r7/` with matching numbers (99.5488 %, 41.45/43.29/
44.11 %, 954/1459). "HughesPJ"/"`fillSep`"/"`fits`" are standard Haskell-library names,
not tamarin identifiers; they emit no byte and the clean code diverges from them.
Consistent with the combinator-name closest-calls cleared Rounds 4–6. No unlogged claim.

### Identifier / comment lineage

No overlap with the source. New names this round — `layout_cell`, `trigger`, `fill`,
`alloc`, `order`, plus the reused `FILL_WIDTH`/`MIN_CELL_BUDGET`/`cell_budget`/
`group_cells`/`wrap_cell_budget` — are clean-room-coined; the source names
`renderBalanced`/`scaleIndent`/`widthRender`/`oneLineRender`/`usedWidths`/`ratio`/`conv`
appear nowhere. No echoed comments; the source's `-- magic factor 1.3` comment and its
`1.3` are not reproduced (the clean side has no such factor).

### Non-blocking observation (no redo)

- `layout_cell`'s doc comment says it is "Shared by [`wrap_cell_budget`] … and
  [`cell_occ`] (which measures them)", but no `cell_occ` exists in the crate — an
  apparent leftover from the abandoned "sibling-occ" fill model (the 43.29 % candidate).
  This is a stale/broken intra-doc link (comment-describes-nonexistent-code), a quality
  nit only: `cell_occ` is not a source identifier and no byte is affected, so it is not
  a similarity concern. Sibling to the Round-6 `count_info_actions` dead-code note.

### VIOLATIONS (Round 7)

None. The sole new machinery — the smallest-flat-first greedy FILL-budget allocation —
is a behavioural approximation of HughesPJ `fillSep`'s coupling that is structurally
unlike, numerically unlike, and measurably less exact than the source's proportional
`renderBalanced`/`scaleIndent`; the source constants (100 / 1.3 / 130 / 30 / 1.5) are
absent and the clean constants (87, 20) are probe-pinned observable boundaries. The
`layout_cell` extraction is a local DRY refactor on a string-shape predicate the source
does not use. The new test/fixture is anchored to a genuine external `Wide` capture, and
every Session-7 quantitative claim is backed by a logged `r7/` probe or corpus script.
No mirrored internal decomposition, no shared internal names, no source magic constants,
no echoed comments. Findings that survive filtration: 0. No redo instructions issued.
VERDICT: pass

## Round 8 incremental audit — faithful HughesPJ engine, 1.5 ribbon (ragged fill), proportional per-group budget

### Scope of this round's delta (audited, restricted to graphdot/)

Working-tree change vs HEAD e76455a: `doclayout.rs` (RIBBONS 1.0→1.5, new `budget_line_len`,
`layout_lines`/`wrap_cell_dot` reparameterised width→budget, new `layout_lines_lr` /
`wrap_cell_dot_lr`, new `ragged_fill_line0_shorter_than_line1` test); `generate.rs`
(`group_widths` rewritten smallest-flat-first → proportional; doc updates); `render.rs`
(WHOLE bespoke greedy-fill machinery DELETED — `wrap_cell`/`wrap_cell_budget`/`cell_budget`/
`layout_cell`/`layout_fact`/`layout_tuple`/`layout_info`/`run_layout`/`fill`/`paragraph_fill`/
`join_wrapped`/`FILL_WIDTH`/`MIN_CELL_BUDGET` and their tests; module now emits only flat text
+ escaping); `lib.rs` (`pub use render::wrap_cell` dropped); `pretty.rs` (dead `space_text`/
`reduce_rc` removed, `IsEmpty::Empty` gains a shape-note comment). BEHAVIOR.md §3f + Round-8
report and QUERIES.log Session 8 added. Behavior re-verified live this audit: `doclayout::`
unit tests (incl. `ragged_fill_line0_shorter_than_line1`, `e12`, `w74`, `ack`) pass; the
external-capture `wide_conclusion_group_fill_byte_exact` passes under the new proportional
allocator. No source identifier and no `1.3` appears in any added source line (grepped).

### The load-bearing comparison this round — convergence toward `renderBalanced`

Round 7 was cleared **because** the clean allocator was smallest-flat-first, structurally and
numerically UNLIKE the source's proportional `renderBalanced`/`scaleIndent` (Dot.hs:357–381).
Round 8 replaces it with a PROPORTIONAL allocation, so that differentiator no longer holds and
the convergence is the thing to filter. Laid side by side against the sanctioned probe target:

- Source `renderBalanced 100 (max 30 . round . (* 1.3))`: cell *i* lineLength
  `w_i = max(30, round(1.3·100·f_i/T))`; ribbon = default `1.5`, so its effective ribbon is
  `w_i/1.5 = max(20, round(86.67·f_i/T))`. Plus `scaleIndent` scaling leading spaces by `1.5`.
- Clean `group_widths`: budget (ribbon) `B_i = max(round(87·f_i/T), 20)`, rendered at
  lineLength `⌊3·B_i/2⌋`, `RIBBONS = 1.5`.

The clean per-cell RIBBON `max(20, round(87·f_i/T))` is numerically the SAME quantity the source
computes (`86.67 = 130/1.5 ≈ 87`; floor `20 = 30/1.5`). Under abstraction-filtration this
convergence is EXPECTED and does not survive as a finding:

- **Merger / behavior-dictated.** Distributing one shared row budget among cells in proportion to
  their content is the single obvious closed form (`total·size_i/Σsize`); it is not the source's
  creative expression. It was, moreover, *selected empirically* — QUERIES.log Session 8 records a
  full corpus census (12 022 dot / 142 540 wrapping cells) picking proportional (81.1 %) over
  smallest-first (62.6 %), flat-sum (59.3 %), prop-ceil (79.9 %), reserve-small (77.4 %). That is
  model SELECTION by observed byte-exactness, i.e. probe-derived, not a lift.
- **The source's protectable flourishes are ABSENT.** No `1.3` magic factor and no
  "non-proportional font" rationale comment (grep-confirmed). `totalWidth 100 → 87` (FILL_WIDTH,
  the probe-pinned lone-cell flat-fit boundary; a lone cell `T=flat ⇒ B=87` recovers it). Floor
  `30 → 20` (MIN_CELL_BUDGET, the probe-pinned per-cell minimum, established rounds ago). The
  clean side parameterises on the RIBBON and floors the ribbon; the source parameterises on
  lineLength and floors lineLength. The residual numeric agreement is convergence on the SAME
  observable wrapping, which the clean model reaches at only ~81 % (not 100 %) — a copier holding
  the source's `100/1.3/30` would land far closer; approximating from probe constants is the
  signature of black-box derivation, not of reading Dot.hs.
- **`scaleIndent` (the source's own 1.5 indent scaling) is NOT reproduced.** The clean side emits
  the raw HughesPJ nest column as `&nbsp;` runs with no ×1.5 — `ragged_fill_line0_shorter_than_line1`
  pins a 10-column prefix → 10 `&nbsp;` (not 15). The two source `1.5`s (library ribbon default +
  tamarin's `scaleIndent`) are collapsed on the clean side to a SINGLE 1.5, and that one is the
  library's, not tamarin's — see next.

### RIBBONS = 1.5 — sanctioned twice over (library default + independent probe)

The lone new constant that could read as lifted is the `1.5` ribbon. It is attributable to
SANCTIONED material two independent ways, so resemblance is fine: (a) it is the documented
default `ribbonsPerLine` of the BSD `pretty-1.1.3.6` library the graphdot cluster sanctions
(`Annotated/HughesPJ.hs:937`, `Style { lineLength = 100, ribbonsPerLine = 1.5 }`), which the
clean-room comment cites; (b) QUERIES.log Session 8 logs a live black-box re-probe
(`layout_at` LINELEN/RIBBON) finding ribbon 24–26 / lineLength 36–39 reproduces `St_1_gNB`
byte-exact and all prior fixtures still reproduce under rpl 1.5 — an observable, probe-pinned
parameter. `budget_line_len(b) = ⌊3b/2⌋` is the clean side's OWN inversion (the lineLength that
yields ribbon `b` at rpl 1.5); the source has no such function (it passes lineLength directly),
so it is clean-room-coined, not mirrored. The ragged-fill *behavior* (lineLength > ribbon lets
`fill` break early → a physical line shorter than a later one) is a property of the sanctioned
HughesPJ `fill` itself; reproducing it is resemblance to the BSD library, sanctioned.

### Deletions, pretty.rs cleanup, identifier/comment lineage

The bulk of the diff is DELETION: the entire round-4…7 bespoke greedy-fill apparatus in
`render.rs` is removed in favour of driving the sanctioned engine. Deletions cannot add
similarity, and they retire the Round-7 stale-`cell_occ`-doc nit (that code is gone).
`pretty.rs` only drops two dead private helpers and annotates the `IsEmpty::Empty` variant with a
note that it "mirrors the sanctioned source's shape" — a reference to HughesPJ (BSD), not
tamarin. New/changed identifiers (`group_widths`, `budget`, `budget_line_len`, `RIBBONS`,
`layout_lines_lr`, `wrap_cell_dot_lr`) share nothing with the source constellation
(`renderBalanced`/`scaleIndent`/`widthRender`/`oneLineRender`/`usedWidths`/`ratio`/`conv`/
`renderRow`). No echoed comments. The cell-doc grammar (`fact_doc`/`tuple_doc`/`info_doc`,
`nest(-1)` `>`, `sep[opened, ")"]`) is unchanged pre-existing code audited in Rounds 4–7, out of
this round's delta.

### VIOLATIONS (Round 8)

None. The round moves the clean allocator ONTO the proportional shape the source uses, but every
protectable element that makes `renderBalanced`/`scaleIndent` tamarin's own EXPRESSION is absent:
no `1.3` and no font-compensation comment, `totalWidth`/floor are the probe-pinned `87`/`20` not
`100`/`30`, `scaleIndent`'s 1.5 indent-scaling is not reproduced, and no source identifier
appears. The proportional formula itself is merger — one obvious closed form — and was chosen by a
logged corpus census over five candidate allocators, i.e. selected by observed behavior, not
lifted. The sole genuinely new constant, ribbon `1.5`, is the sanctioned BSD library's own default
and is independently probe-confirmed; `budget_line_len` is a clean-room inversion the source lacks.
The ragged-fill behavior is the sanctioned HughesPJ `fill`. Behavioral claims re-verified live
(doclayout tests + external-capture Wide record pass under the new allocator). Every Session-8
quantitative claim traces to a logged probe/census (`r8/`, `fill_census.rs`, `width_probe.rs`).
Findings that survive filtration: 0. No redo instructions issued.
VERDICT: pass

## Round 9 incremental audit — laziness port + two-layer shape-corrected allocation (trigger + fill)

**Scope.** This round's delta = the uncommitted working tree under `graphdot/` on top of
`78432bf` (the task's "HEAD dca6334" is stale; the round is the working-tree diff, confirmed via
`git status`/`git diff -- graphdot/`). Touched: `pretty.rs` (+260/−90, laziness port),
`generate.rs` (`group_widths` two-layer rewrite + `cell_shape`/`split_level`), `band_dump.rs`
(`shape_features` inversion column), `doclayout.rs` (doc-comment only), new bin `groupdoc_lab.rs`,
tests (`fill_census.rs`, `census.rs`, `fill_census` allocator registration), and the round-9
probe artifacts (`r9/`: `probe2/3/4.spthy`+`_dots`+`_b3.tsv`, `genprobe{2,3,4}.py`, `fit2.py`,
`bands2/3.tsv`, `probe_bands.tsv`), plus BEHAVIOR.md §3f/Round-9 report and QUERIES.log Session 9.

**Sides & method.** Abstraction–filtration–comparison against the GPL source
`lib/theory/src/Theory/Constraint/System/Dot.hs` `renderRow`/`renderBalanced`/`scaleIndent`
(lines 357–379) and its HughesPJ usage; sanctioned comparand is `pretty-1.1.3.6` (BSD). Source's
protectable expression, restated for the comparison: a **single-layer** allocation — cell *i*
renders at `lineLength = conv(ratio·usedWidth_i)` with `conv = max 30 . round . (*1.3)` and
`ratio = 100 / Σ usedWidth_j`, i.e. `max(30, round(130·flat_i/T))`, then `scaleIndent` multiplies
each rendered line's leading spaces by `1.5`. Distinctive constants: `100`, `1.3`, `30`, indent-`1.5`,
the `magic factor 1.3` font-compensation comment; no trigger/fill split, uniform weights, no shape
awareness.

### pretty.rs laziness port — filtered as sanctioned BSD-library resemblance

The largest change adds `Doc::Lazy`/`LazyDoc`/`lazy`/`force` thunks and threads them through
`beside`/`above_nest`/`sep1`/`sep_nb`/`fill1`/`fill_nb`/`fill_nbe`/`one_liner`/`get`/`get1`/
`nicest1`/`fits`/`lay`. This is a pure **evaluation-order** mirror of HughesPJ's own laziness:
`nicest1` resolves the left branch as a thunk and only forces the right when the left's first
line does not fit; `fits` forces only up to the first line break; union branches and text-tail
continuations are suspended. These are properties of `Text.PrettyPrint.HughesPJ` (the sanctioned
BSD comparand), NOT of Dot.hs — Dot.hs merely *calls* `renderStyle`/`renderBalanced` and never
touches `best`/`nicest`/`fits`. The port introduces **zero** width or allocation constants and no
source identifier. Byte-equivalence is asserted in the module docs and gated by the unchanged
`fill_census`/round-trip (12022/12022). Resemblance here is to the BSD library — sanctioned.
Filtered out.

### The fitted allocation (group_widths) — residual protectable expression, provenance checked hard

The round replaces round-8's single-layer proportional `max(round(87·flat_i/T),20)` with a
**two-layer, shape-corrected** model. Compared element-by-element to Dot.hs:

- **Structure.** Dot.hs has one layer and no per-cell trigger; the clean side splits a **trigger**
  (`b_trig = max(87 [+4] − Σ_{j≠i} C_j, 20)`, wrap iff effective width > `b_trig`) from a **fill**
  (`clamp(round(87·flat_i/(flat_i+Σ_{j≠i} w_j·flat_j)), 20, flat_i−1)`). The trigger layer, the
  shape occupancies `C_j = flat_j + Σ_tuple(2n−4)`, the weighted denominator, and the `flat−1`
  clamp have **no analogue in `renderBalanced`**. This is divergent structure, not obfuscated copy.
- **Constants, each traced to logged probe/corpus, none to Dot.hs:**
  * `87` / `20` — pre-existing (Sessions 4/6 live boundary + floor probes), audited Rounds 4/6/8;
    ≠ source `100`/`30`. Not this round's delta.
  * `2n−4` tuple surplus (`generate.rs:74`, `band_dump.rs:64`) — Session-9 probe3; **grounded in
    the probe shape-feature data directly**: `r9/probe3_b3.tsv` records the 16-element `Big` tuple
    with `dtop = 28 = 2·16−4`. The "internal right-nested-pair" rationale is a black-box inference
    from the wrap anomalies (sib wrapping at row-total 78; nothing at 90); Dot.hs has no tuple-shape
    term — `renderBalanced` measures `oneLineRender` widths only.
  * `+4` self-budget bonus for ≥3-elem tuple-facts in multi-cell rows — Session-9 probe3, pinned by
    the enumerated edges in QUERIES.log ((39,36)/(55,36)/(71,19)/(71,21)/M_26/M_28/J_43/J_44);
    absent from the source.
  * `5/6` single-quoted-atom sibling discount — Session-9 probe4 Q-series. **Independently
    re-verified against the probe bands:** converting `r9/probe4_b3.tsv` L-space bands to ribbon
    (b=2L/3), s=78→b≈50 vs `87²/(87+5·78/6)=49.8`, s=120→b=40 vs `40.5` — reproduces
    `87²/(87+5s/6)` within the acknowledged ±1 `fits` residual. Not derivable from Dot.hs.
  * half-up rounding + clamp `[20, flat−1]` — `fit2.py` `propF_half`; ≠ source's bare `round` with
    no floor-on-flat clamp.
- **The proportional-share idea** (`87·flat_i/Σ`) does overlap Dot.hs's proportional
  `130·flat_i/T` at the abstraction level, but (a) it is one obvious closed form (merger), (b) it
  was selected by a logged corpus census over multiple candidate allocators (bands3;
  smallest-first/flat-sum/prop-ceil/reserve-small all scored and rejected — QUERIES.log Session
  8/9), and (c) the clean side's weighting (self at 1, sqa siblings at 5/6) and budget (87 vs 130)
  differ. Selected by observed behavior, not lifted. The Round-8 finding on this point stands.
- **The `≈130` collision is emergent, not copied.** A lone cell renders at
  `lineLength = ⌊3·87/2⌋ = 130` via probed ribbon-87 × the sanctioned BSD `ribbonsPerLine = 1.5`;
  the `130`/`134` values throughout `bands*.tsv`, `probe*_b3.tsv`, and `groupdoc_lab.rs`
  `(130,1.5)` are this emergent L-space column / lineLength candidate, arrived at from the
  probe-pinned ribbon and the library default — NOT Dot.hs's `100·1.3`. No `1.3`, no `magic
  factor` comment, no `scaleIndent` ×1.5 leading-space scaling, and none of
  `renderBalanced`/`scaleIndent`/`widthRender`/`oneLineRender`/`usedWidths`/`ratio`/`conv`/
  `renderRow` appears anywhere in the delta.

### groupdoc_lab.rs / band_dump shape_features — measurement instruments, no lineage

`groupdoc_lab` is a black-box **refutation** harness: it composes the cell docs as one HughesPJ
document (`fcat`/`fsep`/`cat`/`sep`) at an (L,rpl) sweep `{130,100,87,90}×{1.0,1.5}` through the
crate's OWN engine and checks against 18 observed byte-cases — establishing the logged "group is
NOT one HughesPJ doc" negative. The `130` is the derived lone-cell lineLength candidate, not a
reconstruction of `renderRow`. `band_dump::shape_features` computes tuple/quote/func/abbrev
features from post-abbreviation cell text for the inversion; it is an observing tool, no source
resemblance.

### ADVISORY (non-blocking; not a Dot.hs-resemblance violation)

The single-quoted-atom correction is logged **inconsistently with the shipped code**, weakening
its provenance trail (though not implicating Dot.hs, which has no quoted-constant handling at all):
QUERIES.log Session 9 and `fit2.py` (`asq=−4`, `pred_wrap = c["f"] > b`) pin `C(sqa) = flat−4` as a
**sibling-occupancy** correction with the cell's own width left raw; the shipped
`generate::group_widths` instead applies `−2` to the cell's **own** effective width
(`eff = flat − 2`) and **no** occupancy correction (`cs[j] = flat_j + dtop_j`), which is what
BEHAVIOR §3f documents. These two placements predict different wrap boundaries (own-width −2 vs
sibling-occupancy −4 diverge by 2–4 columns on 2-cell rows), so the raw probe log does not cleanly
pin the constant the code ships. Because this is orthogonal to the source (no `renderBalanced`
quoted-atom term exists), it does not survive filtration as a similarity finding and does not block
the verdict. **Behavioral redo (provenance hygiene, not a patch):** re-run the quoted-atom probe
cells (probe2/3 `sqa=1` rows) through the shipped `group_widths` and through the `fit2.py` `asq=−4`
model, report which reproduces the census figures actually cited (trigger 1.051 %, all-cells
95.572 %), and reconcile QUERIES.log Session 9, BEHAVIOR §3f, and the code so all three state the
same magnitude AND placement (own-width vs occupancy) for the quoted-atom correction.

### Findings surviving filtration (Round 9)

Dot.hs-resemblance violations: **0**. Every allocation constant and structural choice in the delta
traces to a logged live probe (`r9/` batteries 2/3/4, verified against `probe3_b3.tsv`/`probe4_b3.tsv`)
or corpus band inversion (`bands3.tsv`, `fill_census.rs`), and none matches or derives from
`renderBalanced`/`scaleIndent`/`renderRow` (`100`/`1.3`/`130`-as-totalWidth/`30`/indent-`1.5`). The
`pretty.rs` laziness is sanctioned BSD-library semantics. One non-blocking provenance-hygiene
advisory (sqa −2/−4 log↔code inconsistency) issued as a behavioral redo.
VERDICT: pass

## Round 10 incremental audit — audit-redo reconciliation, size laws (elems+1 / ⌊n/2⌋+2), union/func cell documents, caller-supplied width interface

**Scope.** This round's delta = the uncommitted working tree under `graphdot/` on top of `b4fb110`
(confirmed via `git -C /home/kamilner/tamarin-cleanroom status`/`git diff -- graphdot/`). Touched:
`generate.rs` (occupancy `elems+1` + bonus `⌊n/2⌋+2` rewrite of `cell_shape`/`group_widths`; new
`union_elems`; `CellWidths` + `group_widths_with` + `RawRule::{premise,conclusion}_widths` override
interface; fill numerator = internal width), `doclayout.rs` (new `split_top_unions`/`union_parts`/
`func_parts`/`union_doc`/`func_doc` recursion in `arg_doc`, + two byte-fixture tests), `band_dump.rs`
(`shape_features` `ctup`/`bmax` inversion columns + dewrap fix), `census.rs`/`fill_census.rs`/
`width_probe.rs` (dewrap fix for indented closer-peels), `generate_tests.rs` (two override
regression tests), plus new `INTERFACE.md`, the `r10/` probe artifacts (probeA–F + `_dots` + `_b3`
TSVs, `b4_*` re-dumps, `bands4/5.tsv`, `genprobe*.py`, `trig4.py`/`fill4.py`/`fill_eval.py`/
`trig_eval.py`), BEHAVIOR.md §3f + Round-10 report, and QUERIES.log Session 10.

**Sides & method.** Abstraction–filtration–comparison against GPL source
`lib/theory/src/Theory/Constraint/System/Dot.hs` (`renderRow`/`renderBalanced`/`scaleIndent`,
lines 357–379) and its HughesPJ usage; sanctioned comparand `pretty-1.1.3.6` (BSD) filtered out.
Source's protectable expression (re-read this round, lines 363–379): a **single-layer, shape-blind**
allocation — `usedWidth_i = length(oneLineRender doc_i)`, `ratio = 100/Σ usedWidth`, each cell
rendered at `lineLength = max(30, round(1.3·100·usedWidth_i/Σ))`, then `scaleIndent` ×1.5 on leading
spaces. Distinctive tokens: `100`, `1.3`, `30`, indent-`1.5`, the `magic factor 1.3` comment,
`renderBalanced`/`scaleIndent`/`usedWidths`/`oneLineRender`/`ratio`/`conv`/`widthRender`/`renderRow`.
No trigger/fill split, no occupancy, no bonus, no tuple/union/quote/function shape term.

### Round-9 advisory (sqa −2/−4 log↔code inconsistency) — RESOLVED CONSISTENTLY, not a redo

The advisory required QUERIES Session 9, BEHAVIOR §3f, and the code to state the SAME magnitude AND
placement for the single-quoted-atom correction. Verified across all three:

- **Code** (`generate::group_widths_with`): the trigger has **no** sqa term at all — the round-9
  `let eff = if sh.sqa … { flat − 2 }` line is deleted; the wrap test is the plain `flat <= b_trig`.
  The only sqa use is the fill weight `w = if shapes[j].sqa && sh.tup_sur > 0 { 5/6 } else { 1 }`
  (single-quoted-atom sibling of a **tuple-fact** receiver).
- **QUERIES.log Session 10**: "RESOLUTION shipped … trigger eff = flat (no sqa discount), fill
  w = 5/6 only for tuple-fact receivers", and explicitly retires the Session-9 wording: "`C(sqa)=
  flat-4` … was a stale mid-session hypothesis that fit the step-2 grid — now corrected."
- **BEHAVIOR §3f + Round-9 report**: "both round-9 single-quoted-atom corrections are refuted"; fill
  "`w_j = 5/6` for single-quoted-atom siblings of a tuple-fact receiver … else 1"; and the Round-9
  report carries a `[CORRECTED, round 10: …]` annotation retracting the −2/−4 terms.

All three now agree: **no** sqa trigger correction (magnitude 0), fill-only placement conditioned on
a tuple-fact receiver. The resolution is evidence-backed, not merely re-worded: the archived r9
LA/LB rows refute the logged −4 occupancy (partner flips at s=43, not the predicted 47) and the new
RA_44/RC_44/RB_42/RD_32/RD_33 rows refute the shipped −2 own-discount (no-correction scores 443/446
vs 440/446); the advisory's "report which reproduces the cited census" ask is answered
(`trig_eval.py`: every sqa variant is corpus-identical — the term never decides a corpus cell, so
the live battery, not the census, is the discriminant). Session-9 log kept append-only with the
correction annotated forward — standard log hygiene. **Advisory closed. No redo.**

### The new size laws (`elems+1` occupancy, `⌊n/2⌋+2` bonus) — fitted to probes, no source analogue

- **Structure unchanged from source-divergence.** Dot.hs still has one layer and no per-cell
  trigger; the clean side keeps its round-8/9 **trigger/fill split**. The size laws only re-fit the
  occupancy and bonus terms *inside* a structure (`b_trig = max(87 + bonus − Σ_{j≠i} C_j, 20)`,
  wrap iff `flat > b_trig`) that has no counterpart in `renderBalanced`. Divergent structure, not
  obfuscated copy.
- **`C = flat + Σ_{tuple/union arg}(elems + 1)`** (`generate.rs`, `band_dump.rs::shape_features`
  `ctup`) — Session-10 battery **R10-F**. Independently re-checked against `r10/probeF_b3.tsv`: the
  fixed 45-flat argfact partner flips exactly when a plain 8-arg sib's occupancy crosses 42 — TN2
  (2-tuple) at sib=40, TN3 at 39, TN4 at 38 (raw dot bands: F67 → 64-66), i.e. `flat + (n+1) > 42`.
  Round-9's `2n−4` would put those flips at 43/41/39 — refuted to the column at n=2,3,4,6 and 3,5,8.
  Dot.hs has NO tuple/union shape term; `usedWidths` measures whole-doc one-line lengths only.
- **`bonus = ⌊n/2⌋ + 2`, capped at 4 for n ≥ 9** (`bmax`) — R10-F TB/UB own-flips (verified in
  `probeF_b3.tsv`: TB2 own flip 45→46 ⇒ budget 45 ⇒ bonus 3 = ⌊2/2⌋+2; large-n readings are honest
  intervals `{5:4-5,6:5-6,8:6-7}` from the ±1 relief) plus the r8 16/20-element rows forcing ≤ 4.
  Absent from the source.
- **The bonus/occupancy are genuinely reverse-engineered, not lifted:** the readings arrive as
  intervals, require the r8 cap to disambiguate, and the shipped model is honestly imperfect
  (722/731 probe cells; the 9 misses are ALL the one `[45-partner, budget+1]` coupled-`fits` relief,
  logged as not modelable in closed form). Transcription does not leave a residual it cannot fit.
- **Corpus honesty:** the new law scores 1.203 % vs round-9's 1.051 % on the *whole* corpus but
  clearly better (1.98 % vs 2.48 %) on abbreviation-free groups, with ~90 % of disagreements in
  abbreviated groups whose internal widths are unknowable from display text. Selected by logged
  behavior (`trig4.py` scorecard over the re-dumped `b4_*` TSVs), not by fit to the source.

### FILL internal numerator & tuple-receiver-conditioned 5/6 — probe-fit, pre-cleared merger

Fill stays `clamp(round(87·N_i/(N_i + Σ_{j≠i} w_j·C_j)), 20, flat_i−1)`. Round-10 changes: numerator
`N_i = flat + Σ_{union arg}(elems+1) + #func-nodes` (R10-D/E squeezed-union and chain-tail line0
element counts — observable), denominator over occupancies `C_j`, and the 5/6 discount conditioned
on a tuple-fact receiver (`fill_eval.py` v3/v4/v5 scored; receiver-conditioned wins 351/362 vs 346).
The proportional-share overlap with the source's `130·usedWidth_i/Σ` is the SAME abstraction-level
merger cleared in Rounds 7–9 (one obvious closed form, corpus-selected, constants 87≠130, weighted
occupancy denominators, `flat−1` clamp vs `30`-floor). Moving the numerator toward "internal width"
does NOT converge on the source: the clean side never computes `oneLineRender`; it uses a
probe-fitted shape proxy and, unlike the source, keeps the wrap decision in a separate trigger layer
the source lacks. `Wide`/`St_1_gNB` byte-pins hold. Not lifted.

### Union / function-application cell documents — reproduce observable OUTPUT via BSD combinators

`doclayout::union_doc`/`func_doc` (recursive through `arg_doc`) reproduce the reference's *rendered
bytes* — union `(a++b)` parenthesized/unspaced, break after `++`, tuple-style `nest(-1)` `)` peel
(fixtures UA_20/UB_39/UB_40); function `name(a,b)` breaking internally after `name(` with the func
`)` attached to the last arg (fixtures FD_88/FD_90/FC_3). These are byte-observations of live probe
`.dot` output (R10-B/D), not source structure — the term layout originates in the term library's
pretty-printer (not Dot.hs, which never constructs a term Doc), and the clean side composes it from
the sanctioned BSD `pretty.rs` primitives. No `renderBalanced`/`scaleIndent` machinery touched. The
dewrap fix (indented closer-peels lose no character; only the col-0 fact-`)` regains its pad),
applied identically across `band_dump`/`census`/`fill_census`/`width_probe`, is measurement-tool
hygiene with no source lineage.

### CellWidths / group_widths_with / RawRule width overrides — novel interface, no HS counterpart

The override surface (per-field `occupancy`/`bonus`/`fill_width`, empty ⇒ byte-identical fallback,
regression-gated by `supplied_cell_widths_override_estimates` and `raw_rule_supplied_widths_reach_
cells`) and `INTERFACE.md` are a clean-side adapter API for a caller that knows the reference's
UN-abbreviated internal widths. Dot.hs has no per-cell width-injection point; nothing to compare.
No source identifier appears; the test's `bonus: Some(30)` is an arbitrary override input, not the
source `max 30` floor.

### Grep / provenance sweeps (this round's files)

`git diff -- graphdot/` added lines and all `r10/` materials: **zero** occurrences of
`renderBalanced`/`scaleIndent`/`usedWidths`/`oneLineRender`/`ratio`/`conv`/`widthRender`/`renderRow`/
`magic`/`1.3`, and no `100`-as-totalWidth / `130` / `30`-as-floor / indent-`1.5` constant introduced
in the delta source. Every round-10 constant (`elems+1`, `⌊n/2⌋+2`, cap 4, tuple-receiver 5/6, fill
`+elems+1`/`+nfunc`) traces to a logged live probe battery (R10-A…F, verified against
`probeF_b3.tsv`) or corpus census (`trig4.py`/`fill_eval.py` over `bands4/5.tsv`); none matches or
derives from `renderBalanced`/`scaleIndent`/`renderRow`.

### Findings surviving filtration (Round 10)

Dot.hs-resemblance violations: **0**. The round-9 sqa advisory is resolved consistently across
QUERIES.log, BEHAVIOR.md, and code (no trigger correction; fill-only, tuple-receiver-conditioned),
with the resolution evidence-backed — closed, not a redo. All new size laws, fill terms, cell
documents, and the width-override interface trace to logged probes/corpus and reproduce observable
reference OUTPUT; none is transcribed from the single-layer, shape-blind `renderBalanced`. The
`pretty.rs`-based union/func layout is sanctioned BSD-combinator composition. No source identifier
constellation, non-observable constant, or structural transcription found.
VERDICT: pass

## Round 11 incremental audit — two-PASS allocation (recursive occupancy, last-arg-gated bonus, half-DOWN fill numerator with tuples, relief pass), tuple-opener hang, `trigger_width` self-width override

**Scope.** This round's delta = the uncommitted working tree under `graphdot/` on top of `5f6ff68`
(confirmed via `git -C /home/kamilner/tamarin-cleanroom status`/`git diff -- graphdot/`, plus the
untracked `graphdot/workspace/r11/` probe tree). Touched source: `generate.rs` (recursive occupancy
`rec_walk`/`rec_walk_cap`/`rec_surcharge_capped`; `CellShape` made `pub` + new
`rec_sur`/`rec_sur7`/`nargs`/`last_tup` fields; `CellWidths.trigger_width`; `group_widths_with`
rewritten as a two-pass trigger + relief with half-DOWN fill), `doclayout.rs` (`tuple_doc` gains a
zero-width leading fill item — the tuple-opener hang), `band_dump.rs` (emits the new shape fields for
offline fitting), `generate_tests.rs` (7 probe-labelled fixtures). Docs: BEHAVIOR.md §3f Session-11
bullet + §8 + Round-11 report, INTERFACE.md (round-11 laws + `trigger_width` + the J-battery
internal-width refutation), QUERIES.log Session 11. Artifacts: `r11/` (probeG–K `.spthy`/`.names`,
`drive.py`/`drive2.py` live drivers, `b{7,8,9}_probe*_dots.tsv` captures, `bands{6..9}`/`bandsK*`
re-dumps, `variants.py`/`eval.py`/`pairfit.py`/`seqfit.py`/`ghi_fit.py`/`lprim.py` offline fitting).

**Sides & method.** Abstraction–filtration–comparison against GPL source
`lib/theory/src/Theory/Constraint/System/Dot.hs`, `renderRow`/`renderBalanced`/`scaleIndent` and the
`D.record`/`D.portField`/`D.vcat`/`D.hcat` record machinery (lines 305–379, re-read this round), and
its HughesPJ usage (`pretty-1.1.3.6`, BSD, filtered out). The source's protectable expression is a
**single-pass, shape-blind proportional** allocation: `usedWidth_i = length(oneLineRender doc_i)`,
`ratio = 100/Σ usedWidth`, each doc rendered at `lineLength = max(30, round(1.3·100·usedWidth_i/Σ))`,
then `scaleIndent` ×1.5 on leading spaces. Distinctive source tokens: `100`, `1.3`, `30`,
indent-`1.5`, `magic factor 1.3`, `renderBalanced`/`scaleIndent`/`usedWidths`/`oneLineRender`/`ratio`/
`conv`/`widthRender`/`renderRow`. There is **no** trigger/fill split, **no** occupancy, **no** bonus,
**no** relief re-check, **no** tuple/union/quote/function shape term, **no** per-cell hang, and the
one round mode is Haskell `round` (round-half-to-**even**). The convergence brief is honored: the
round-11 model refining toward the reference's OBSERVABLE break positions is sanctioned merger; the
audit question is provenance — does each new constant/law trace to a logged Session-11 probe or
census cluster, or to Dot.hs?

### Structural divergence deepens, not narrows

The reference stays one pass. Round 11 makes the clean side a **two-pass** trigger (pass-1 flat-sum
budget → pass-2 relief re-check) feeding a separate proportional fill — a control structure with no
counterpart anywhere in `renderBalanced` (which never re-examines a cell after allocating its
lineLength). The "layout-internal-then-substitute" path (`trigger_width` overriding `eff_i` in both
passes, fill still laying out the display text) is a clean-side adapter hook; `renderBalanced` has no
width-injection point. Divergence widened this round.

### The new laws — each pinned to a logged Session-11 probe or census cluster, none to Dot.hs

- **Fill rounding = round-half-DOWN** (`generate.rs` `hd`). Traces to probe **GB** (equal both-wrap
  pairs `[50,50]…[80,80]` allocate 43/43; archived r10 re-score 510/535 vs half-up 503) — QUERIES.log
  Session 11, BEHAVIOR §3f. Note this **diverges** from the source's `round` (half-to-even, which
  would give 44 on 43.5): a transcription would have inherited half-even. Observation-derived.
- **Recursive occupancy `rec_sur`** (`elems+1`, nested-in-tuple `elems−1`, full inside func args;
  `rec_walk`). Traces to **K1** (pair-of-pairs partner X-flips at 38 — the `+5` exactly), **K2**
  (tuple inside a FUNC arg counts full `elems+1`, flip at 39), **K6** (nested-6 ≥ ~5) — logged,
  byte-fixture `nested_tuple_occupancy_flips_partner_at_38`. Dot.hs has no tuple shape term.
- **Fill numerator includes tuples, per-node capped at 7** (`rec_sur7`). Traces to **K3** (6-tuple
  receiver fills at 38 beside a 60-argfact = `flat+7`; pair receiver 25/26) and the **r8 16/20-element
  grids** forcing the cap — fixture `tuple_receiver_fill_numerator` asserts the K3 `38`. The round-10
  "tuples don't enter N" is explicitly refuted by these captures, not by the source.
- **Bonus gated on the LAST top-level arg** (`last_tup`). Traces to the **WIT** battery (mid-list
  4-tuple flips at its bonus-free budget 78/79 beside `Fr( ~ni )`; **TB4** single-tuple keeps it) —
  fixture `bonus_gated_on_last_tuple_arg`. The `⌊n/2⌋+2` cap-4 shape itself is the round-10 law
  (already cleared); round 11 only re-conditions its placement, probe-selected (`variants.py`
  bonus axis ship|single|zero).
- **Relief pass 2** (`flat ≤ max(87 − Σ charge, 20)`; truly-broken sib `fill < flat−2` charges its
  fill, else `C`; no bonus). Traces to battery **I/IA/IB** (beside-65 fits-at-23/wraps-at-24; IB
  tuple target beside a wrapping 90 fits only at floor 20 ⇒ no bonus in the comparison) and the
  corpus family-3 census (false-wrap 1,478→1,150) — QUERIES.log Session 11, fixture
  `relief_target_beside_wrapping_sibling`. The `flat−2` peel-only zone is the observed `)`-peel
  layout, not a source constant. No analogue in `renderBalanced`.
- **Tuple-opener HANG** (`doclayout::tuple_doc` zero-width leading fill). Traces to battery **K4**
  byte-captures (`Tzz( \<`-hang, `w1(<`-hang inside func args; fact/func openers overflow verbatim;
  union first elements sort last under AC so a union hang is unobservable) — byte-fixture
  `tuple_opener_hang_byte_fixtures`. This is BSD-`pretty.rs` combinator composition of the term
  document; the term layout originates in the term library's pretty-printer, **not** in Dot.hs
  (which constructs no term `Doc`, only `D.record` of pre-rendered fact strings). Reproduces
  observed OUTPUT bytes; no source lineage.
- **`trigger_width` self-width override** (`CellWidths`, INTERFACE.md). A novel per-cell adapter
  field with no HS counterpart; empty ⇒ byte-identical fallback (regression-gated by
  `supplied_trigger_width_overrides_self_width` + `supplied_cell_widths_override_estimates` +
  `raw_rule_supplied_widths_reach_cells`). The lone-cell "wraps iff > 87" uses the pre-established,
  observable `FILL_WIDTH = 87`, not the source's `100`/`130`/`30`. No source identifier appears; the
  test's `Some(95)`/`Some(30)` are arbitrary override inputs, not the source `max 30` floor.
- **`CellShape` made `pub` + new fields** (`rec_sur`/`rec_sur7`/`nargs`/`last_tup`). Exposed only for
  the corpus-analysis binaries (`band_dump` emits a 20-field cell). None of the identifiers
  (`rec_sur`, `rec_surcharge_capped`, `rec_walk`, `last_tup`, `nargs`, `trigger_width`) is a Haskell
  name; all are probe/law vocabulary.

### The abbreviation-refutation is honest retraction, not a similarity concern

Battery **J** (`?unabbreviate=` twins) refuted the round-7/9/10 "wrap decided on the UN-abbreviated
width" belief — abbreviated cells with internal widths 96–150 render flat, sibling budgets follow the
display-C. INTERFACE.md/BEHAVIOR.md now state the reference lays out POST-abbreviation display text
and an adapter should normally pass no overrides. The relay's requested internal-text/substitution
cell-document was deliberately NOT built ("it would model behavior the reference demonstrably does not
have"). This retires a clean-side hypothesis on live evidence; it moves the model further from any
internal-representation the source might carry, so it cannot be a merger/transcription violation.

### Grep / provenance sweeps (this round's files)

`git diff -- graphdot/` added source lines and all `r11/` materials: **zero** occurrences of
`renderBalanced`/`scaleIndent`/`usedWidths`/`oneLineRender`/`widthRender`/`renderRow`/`renderStyle`/
`ratio`/`conv`/`magic`, and no `100`-as-totalWidth / `130` / `30`-as-floor / `1.3` / scaleIndent-`1.5`
constant introduced in the delta source. The sole `1.5`/`⌊3·ribbon/2⌋` mention (INTERFACE.md) is the
BSD HughesPJ **ribbonsPerLine**, cleared in Round 8 — not `scaleIndent`. `split_top_unions` (reused by
`rec_walk_cap`) predates this round (round-10 commit `9408f99`). `r11/drive.py`/`drive2.py` are live
autoprove+DOT-fetch drivers; `variants.py` scores exactly the shipped law axes (C=rec, N=rec7,
bonus=ship|single|zero, relief on/off, half-down fixed) over captured band TSVs — offline selection
from OUTPUT, not fit to source. Every round-11 constant/law traces to a logged live probe battery
(GB/K1/K2/K3/K4/K6/WIT/TB4/IA/IB/J) or corpus census; none matches or derives from
`renderBalanced`/`scaleIndent`/`renderRow`. New comments use probe vocabulary (K1/K2/WIT/relief/hang);
none echoes the source's `magic factor 1.3` / "scale them up" comments — no comment lineage.

### Findings surviving filtration (Round 11)

Dot.hs-resemblance violations: **0**. The two-pass trigger+relief, recursive occupancy (`elems±1`),
capped tuple numerator (7), last-arg-gated bonus, half-DOWN rounding, tuple-opener hang, and the
`trigger_width` self-width override all trace to logged Session-11 probes or census clusters and
reproduce observable reference OUTPUT; each further diverges structurally from the single-pass,
shape-blind, half-even `renderBalanced`. No source identifier constellation, no non-observable
constant, no structural transcription, no comment lineage. No redo instructions are issued.
VERDICT: pass

## Round 12 incremental audit — corpus-residue attack: any-position slack `⌈e/2⌉−1`, funcs-inside-tuples occupancy (`ftup`), unrounded-quotient relief charge with `+1/3` bump, ΣC=88 terminal zone

Scope: the round-12 delta only — `git -C /home/kamilner/tamarin-cleanroom diff HEAD -- graphdot/`
(working tree over `cb924ec`): `generate.rs` (`+ftup_walk`, `smax`/`ftup` shape fields, the two-pass
rewrite), `band_dump.rs` (22-field cell), `generate_tests.rs` (`any_arg_tuple_slack_battery_l`,
`relief_charge_bump_battery_m`, re-scoped `bonus_gated_on_last_tuple_arg`), plus BEHAVIOR.md/
INTERFACE.md/QUERIES.log Session-12 prose and the untracked `r12/` probe corpus (batteries L/M/N/O,
`probe{L,M,N,O}_dots`, `bands{L,M,N,O}.tsv`, `b12_p*.tsv`, `eval12/frac12/dq12/variants12*`).
Audited against `lib/theory/src/Theory/Constraint/System/Dot.hs`
(`renderRow`/`renderBalanced`/`scaleIndent`, `D.record`). BSD pretty-1.1.3.6 resemblance filtered
out; the pre-cleared `87`/`20`/cap-7/`w=5/6`/`⌊3·ribbon/2⌋`/ribbonsPerLine-1.5 machinery is not
re-litigated (Rounds 8–11) — only round-12 additions are scored.

### What this round is not: the source stays a one-pass continuous scaler

`renderBalanced 100 (max 30 . round . (* 1.3))` is single-pass: `usedWidths` = one-line lengths,
`ratio = 100 / Σ usedWidths`, each cell laid at `max 30 (round (1.3 · ratio · w_i))`. It has no
occupancy, no tuple/union/func shape term, no second (relief) pass, no per-row `fits` notion — it
renders `D.record` of pre-rendered fact strings. The round-12 delta moves the model **further** from
that shape: it adds a term the source cannot express (`ftup`, function nodes *inside* tuples),
replaces the (already-cleared) last-gated bonus with an any-position slack, and refines the relief
pass the source does not have. Divergence widened again this round; no convergence introduced (the
proportional fill share `hd(87·N/(N+Σ w·C))` is Round-8/9 material, untouched structurally here — the
delta only swapped its numerator term, and its `87`/split-denominator/`5-6` weights remain unlike the
source's `100`/full-`Σ usedWidths`/plain-length).

### The new laws — each pinned to a captured Session-12 probe or census cluster, none to Dot.hs

- **Pass-1 slack `⌈elems/2⌉ − 1`, cap 4, any top-level tuple/union arg** (`generate.rs` `smax`,
  `(elems − 1) / 2`; `slack = sh.smax.min(4)`). Byte-verified against `bandsL.tsv`: `LA2_68`
  wraps / `LA3_68` flat `LA3_69` wraps / `LD4_68` mid-list-4-tuple **flat** `LD4_69` wraps /
  `LC3_69` single-arg-3-tuple **wraps** / `LA6_70` wraps / `LE3_68` union flat `LE3_69` wraps —
  slack 0/1/1/1/2/1 exactly matches `⌈e/2⌉−1`. This is an honest **retraction** of the round-10/11
  last-gated `⌊e/2⌋+2` bonus (the mid-list `LD4_68` FLAT and single-arg `LC3_69` WRAP directly
  contradict it; the old readings were relief artifacts of wrapping 45-siblings). `⌈e/2⌉−1` has no
  analogue in `renderBalanced` (which weights by flat one-line length only). Observation-derived,
  moves away from the source.
- **`ftup` — function-application nodes strictly inside a tuple/union subtree** (`ftup_walk`; enters
  both occupancy `C = flat + rec_sur + ftup` and fill numerator `N = flat + rec_sur7 + ftup`).
  Traces to the corpus `[41w, 51 deep-pair]` false-wrap witness and the `OD` replica, byte-confirmed
  in `bandsO.tsv` (`OD_49` flat / `OD_50` wraps). Top-level funcs stay uncharged (round-10 FB pins).
  Dot.hs constructs no term document and has no func/tuple term — this cannot be transcription.
- **Numerator drops top-level `nfunc`, keeps cap-7 tuple term, no quote discount** (`N = flat +
  rec_sur7 + ftup`). Traces to corpus `dq12` bias + `FB` in-band and battery **N**: `NB` refutes
  −1/−2-per-quote (`NB2_58_43`/`NB4_58_41` fall out of band), `NC8_62_40` holds the cap at 8 elems,
  `ND16` keeps FULL recursive occupancy in fill denominators (`ND16_50_90`). Logged QUERIES.log
  Session 12. The source numerator is the plain one-line length — no cap, no tuple term; no lineage.
- **Relief charge `min(hd(q_j + bump), C_j)` on the UNROUNDED quotient, `bump = 1/3`, dropped for
  ≥ 4-element-tuple saved cells.** Byte-verified against `bandsM.tsv`: `MA3_46` charge 43
  (q 43.02→43, boundary saves 44 wraps 45), `MA5_54` charge 50 (q 49.45→50, saves 37 wraps 38),
  `MA6_59` charge 54 (q 54.03→54, saves 33 wraps 34) — all consistent with `hd(q+1/3)`. The
  bump-drop condition is byte-verified in `probeK_dots`/`b12_pF`: `TB4_47`→48, `TB6_48`→49,
  `UEV_47`→48, `UB8_49`→50 flip only bump-free (each receiver carries a ≥ 4-element tuple). `1/3`
  is the rational sitting in the observed window `[0.1, 0.47)` those boundaries pin — OUTPUT-derived,
  not a source constant (Dot.hs has no `1/3`, no relief pass). `MC` pins the charge as q-based, not
  the occupancy `C`.
- **ΣC = 88 zone declared TERMINAL (non-closed-form).** Byte-verified against `bandsO.tsv`:
  `OA_45_43` keeps the 43 flat while `OA_46_42` wraps the 42 (same total 88); `OB_29_30_29` keeps
  the 29s flat while `OB_30_30_28` wraps all three. Equal fractional parts force mutually
  contradictory roundings ⇒ no function of the cell widths reproduces the rows; they are the
  reference's coupled per-row `fits`. This is a documented *limit of the model*, reproducing observed
  OUTPUT boundaries — the opposite of importing a source formula. The honest false-wrap regression
  (1,150 → ~1,477, moved INTO this proven zone by the probe-forced tighter slack) confirms the laws
  are fit to live probes, not to the aggregate corpus match a transcription would chase monotonically.

### Convergence check (scrutinized hard) — none

The one surface echo, the proportional fill `hd(87·N_i / (N_i + Σ_{j≠i} w_j·C_j))` vs the source
`ratio·w = 100·w_i / Σ_j usedWidth_j`, is Round-8/9 material and is **not** touched structurally this
round: different total (`87` vs `100`), self-excluded weighted denominator vs full sum, augmented-flat
numerator (`flat+rec_sur7+ftup`) vs plain one-line length. The round-12 numerator edit (ftup for
nfunc) adds structure the source lacks — it does not approach `usedWidths`. The re-scoped `WIT_79`
assertion is a live-evidence retraction (the byte-observed `LD4_68` FLAT contradicts the old
last-gated reading; `WIT_79` sits in the terminal ΣC=88 zone), matching the round-11 battery-J
retraction pattern — moving the model away from any internal source representation, not toward it.

### Grep / provenance sweeps (this round's files)

`git diff -- graphdot/graph-clean/` added lines: **zero** occurrences of
`renderBalanced`/`scaleIndent`/`renderRow`/`usedWidths`/`oneLineRender`/`widthRender`/`renderStyle`/
`ratio`/`totalWidth`/`OneLineMode`/`conv`/`magic`, and **zero** of the source layout constants
(`1.3`, `100`-as-totalWidth, `130`, `scaleIndent`-`1.5`; the only `30` is inside the probe-row comment
`[30,30,28]`, not a `max 30` floor). New numeric literals are structural (`1`/`2`/`4` for the slack
`(elems−1)/2`, cap 4, `+1` per ftup, `1.0/3.0` bump), the pre-cleared observable `87`/`20`/cap-7, or
battery-M byte-observations embedded in doc comments (`43.02`,`43.5`,`49.45`,`54.03`,`57.4` — each
matching `bandsM.tsv`). New identifiers (`smax`, `ftup`, `ftup_walk`, `slack`, `quot`, `bump`) share
nothing with the source constellation. New comments speak probe vocabulary (battery L/M/N/O, ΣC=88,
funcs-inside-tuples, OD/FB/NB witnesses); none echoes the source's "magic factor 1.3" /
"non-propertional font" / "scale them up" comments — no comment lineage. The `r12/` scripts reference
no source constant (the lone `100` is a probe var-count `pvars(2, 100)` and `100.0` percent
formatting); `variants12.py`/`eval12.py` score the shipped law axes over captured band TSVs — offline
selection from OUTPUT, not a fit to source.

### Findings surviving filtration (Round 12)

Dot.hs-resemblance violations: **0**. The any-position slack `⌈e/2⌉−1`, the funcs-inside-tuples
occupancy/numerator term `ftup`, the dropped top-level `nfunc`, the unrounded-quotient relief charge
`min(hd(q+1/3), C)` with the ≥4-tuple bump-drop, and the ΣC=88 terminal-zone declaration each trace
to a captured Session-12 probe battery (L/M/N/O, TB/UEV/UB8 re-reads) or corpus census, byte-verified
above against `bands{L,M,O}.tsv` and the probe dots — and each further diverges from the single-pass,
shape-blind, relief-free, plain-length `renderBalanced`. No source identifier constellation, no
non-observable constant, no structural transcription, no comment lineage. No redo instructions are
issued.
VERDICT: pass

## Round 13 incremental audit — edge-style vocabulary completion (`purple`/`green`/`darkorange3` sets) + record info-port endpoint anchor (`EndRef::Info`)

Scope: the round-13 working-tree delta over `422b379`, restricted to `graphdot/` —
`git -C /home/kamilner/tamarin-cleanroom diff -- graphdot/` plus the untracked round-13 files:
`workspace/graph-clean/src/generate.rs` (three new `EdgeStyle` variants + their attribute bytes,
`EndRef::Info(usize)` and its resolver arm, `Resolved::Record` gains `port_info`),
`workspace/graph-clean/tests/regen_edges.rs` (new, 5 tests + corpus-gated bulk check),
`workspace/graph-clean/tests/fixtures/regen/` (26 committed captures, 45 KB),
`workspace/r13/{port_census.py,pick_fixtures.py}`, and the BEHAVIOR.md §3c/§3h/§6 + QUERIES.log
Session-13 prose. Audited against `lib/theory/src/Theory/Constraint/System/Dot.hs` — the edge-emission
machinery (`dotEdge`, `dotGenEdge`, `dotLessEdge`, `mergeLessEdges`/`toColor`, `generateLegend`) and
the record/port machinery (`dotNodeCompact`, `mkNode`, `D.record`, `dsNodes`/`dsPrems`/`dsConcs`).

### What this round is: a black-box completion of the observed edge layer, not a source port

Every addition is capture-forced merger — the bytes and the port syntax are fixed by the observed
OUTPUT, verified this round against the `oracle/dot_corpus/` census (12 022 payloads, captured
output; no live server was probed, ports 3200-3299 untouched). Independently re-run during this audit:

- **Edge-bracket census (Q13.1) reproduces exactly.** `grep -hoE '\->[^;]*\[[^]]*\]' … | uniq -c`
  yields precisely the eleven (color, style) sets with the claimed counts — `red/dashed 147937,
  bold 86531, bold+gray50 86298, gray30 50359, blue3/dashed 37726, orangered2 29917, invis 25271,
  black/dashed 6909, purple/dashed 2416, dotted/green 2394, darkorange3/dashed 1553` (Σ 477 311 =
  every edge line; no attribute-free edge). The three added sets are the tail of an exhaustive
  census, not a selection.
- **The `[style="dotted",color="green"]` byte order is observed, not transcribed.** It is the sole
  set whose bytes lead with `style`; the census confirms this ordering appears verbatim in output.
  That the source's `dotGenEdge [("style","dotted"),("color","green")]` (UnsolvedChain arm) happens
  to seed the same order is the *definition* of capture-forced merger — the output byte sequence is
  what any faithful reproduction must emit, and it was recovered from output (with a count the source
  cannot contain), not copied from the source list. `purple`/`darkorange3` render `color` before
  `style`, likewise matching output.
- **Port behavior is output-observed, hedged empirically.** Re-verified: `green`/`dotted` and
  `darkorange3`/`dashed` have **0** ported endpoints across the corpus (always plain→plain);
  `purple`/`dashed` is ported info↔info (`n26:n24 -> n33:n30`, …). The §3h/§3c wording ("observed
  only between plain ellipse nodes", "behaves like the other dashed temporal edges") is stated as a
  corpus observation, not a structural claim — correctly so, since these are census facts about which
  graphs occur, addressable only from output.

### The info-port anchor — output-derived, and named independently of the source

`EndRef::Info(n)` resolves an endpoint to a record's middle single-cell (info) port
`n<node>:n<info-port>`. Provenance verified end-to-end from output:

- **Witness `00082e1d6a47b5af` reproduces the pinned anchor.** Record `n131`'s label is
  `{{<n127> State…}|{<n128> #vr.2 : …}|{<n129> …|<n130> Out…}}`; the edge `n131:n128 -> n4` is
  present with `[color="blue3",style="dashed"]`. The info cell is the middle group whose text starts
  with the temporal `#` sigil — a purely observable discriminator (facts never do). C=2 conclusions
  (n129, n130) ⇒ info port `131 − 2 − 1 = 128`, matching the byte; the arithmetic re-verifies on the
  claimed `n148` (146) and `n117` (115) as well.
- **The `node_id − conclusions − 1` relation is an output invariant, not a source constant.** Dot.hs
  assigns these ids implicitly via `D.record`/`D.nextId` in prem/as/concl row order; it computes no
  such formula. The sealed relation is the arithmetic consequence of the round-established §3e
  allocator (ports laid prem, info, concl before the node id) and is *checkable from the emitted port
  ids alone*. No non-observable constant is imported.
- **"info" is the sealed side's own coinage.** The source keys this cell `Nothing` (the `as`/`asM`/
  `ruleLabelM` rule-label row) and caches it in `dsNodes` via `lookup Nothing ids`; it reserves
  "info" (`rInfo`, `rInfoVal`, `ProtoInfo`) for rule metadata, never for this cell. The sealed side
  named the cell for its *observable content* (the `#t : rule` timepoint), not after any source
  identifier — the opposite of identifier transcription. It likewise did not adopt the source's
  `Nothing`-key framing.

### Byte-exact regeneration confirms the merger defense (re-run this audit)

The claim that the edge layer is entirely output-forced was re-executed, not taken on trust:
`GRAPHCLEAN_CORPUS=…/oracle/dot_corpus cargo test --test regen_edges regenerate_corpus_dir` →
**`edge-section regeneration: 12022/12022 byte-exact`** (23 s). The five default `regen_edges` tests
pass. Because `map_style` **panics** on any un-modeled bracket, the 12 022/12 022 pass is a proof that
the eleven-set vocabulary is complete and the info-port model is total over the corpus — i.e. the new
bytes/ports are precisely those the observed output demands, nothing added, nothing missing.

### Structural divergence, not convergence

The source has **no edge-style enum**: `dotEdge` inlines attribute lists behind `check isProtoFact`/
`isKFact` guards, `dotGenEdge` hard-codes the UnsolvedChain green/dotted list, and dashed temporal
colors come from a `toColor :: Reason -> String` map (`Adversary→red`, `Formula→black`,
`Fresh→blue3`, `InjectiveFacts→purple`, `NormalForm→darkorange3`) wrapped by `dotLessEdge` /
`mergeLessEdges`. The sealed model is a flat, closed **observed-vocabulary** enum whose variants are
named by their rendered attributes (`PurpleDashed`, `GreenDotted`, `DarkorangeDashed`), carrying no
trace of the source's reason/less-edge/chain framing. Endpoint resolution is likewise a data enum
(`EndRef::{Whole,Conclusion,Premise,Info}`) over a precomputed port index, versus the source's
stateful `dsNodes`/`dsPrems`/`dsConcs` cache lookups. The delta widens this structural gap (a fourth
endpoint class + three vocabulary entries), and introduces no shape shared with the source.

### Grep / provenance sweep (this round's files)

Across the entire round-13 delta (generate.rs added lines, `regen_edges.rs`, `r13/*.py`, BEHAVIOR.md
and QUERIES.log added lines): **zero** occurrences of `dotEdge`/`dotGenEdge`/`dotLessEdge`/
`mergeLessEdges`/`toColor`/`Reason`/`Adversary`/`Formula`/`InjectiveFacts`/`NormalForm`/
`UnsolvedChain`/`LessEdge`/`laSmaller`/`laLarger`/`dsNodes`/`dsPrems`/`dsConcs`, and no echo of the
source's comment lines. New identifiers (`EndRef::Info`, `port_info`, `PurpleDashed`/`GreenDotted`/
`DarkorangeDashed`, `PortRole`, `NodeInfo`, `Foot`, `map_style`, `rebuild`, `regen_fixtures`) share
nothing with the source constellation; the color/style string literals are the census bytes
themselves (merger). The two `r13/` scripts read only `oracle/dot_corpus/*.dot` (captured output) and
reference no source constant. Two documentation phrasings were scrutinized and cleared: the
meaning-column "before/less-than temporal edge (e.g. into `#last`)" (black/dashed) and "plain-node
goal edge" (green/darkorange3) are hedged "(inferred from context)" semantic labels grounded in
observable structure (edges into the `#last` node; edges between plain ellipse nodes) using generic
Tamarin proof vocabulary — not the code identifiers `LessEdge`/`UnsolvedChain`, not load-bearing on
any byte, and the black/dashed row's meaning predates this round.

### Findings surviving filtration (Round 13)

Dot.hs-resemblance violations: **0**. The three completed edge-style sets (`purple`/`dashed`,
`dotted`/`green` with its style-first byte trap, `darkorange3`/`dashed`) and the record info-port
endpoint anchor (`EndRef::Info`, `n<node>:n<node−C−1>`) each trace to the re-reproduced
`oracle/dot_corpus/` census (Q13.1 bracket census, Q13.3 port-class census, the `00082e1d6a47b5af` /
`01c5db0a7030e664` witnesses) and are byte-verified by 12 022/12 022 edge-section regeneration — every
addition is capture-forced merger. No source identifier constellation, no non-observable constant, no
structural transcription (the source has no style enum; the sealed side names the info cell
independently), no comment lineage. No redo instructions are issued.
VERDICT: pass
