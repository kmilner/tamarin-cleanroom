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
