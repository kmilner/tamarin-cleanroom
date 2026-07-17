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
