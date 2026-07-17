# Similarity audit — wellformedness cluster (`wf-clean`)

Auditor role: clean-room similarity review. I read both the clean side and the
GPL Haskell originals (and, as a secondary leak check, the replaced Rust module
`crates/tamarin-parser/src/wf.rs`). Method: abstraction–filtration–comparison.

- CLEAN SIDE: `workspace/wf-clean/src/{pretty,report,checks,formula,lib}.rs`
  (`ast.rs` is the provided interface header — not audited).
- HASKELL: `tamarin-prover/lib/theory/src/Theory/Tools/Wellformedness.hs`
  (+ its pretty-printing dependencies).
- INTERFACE GIVEN TO CLEAN ROOM: `interface/required_api.md`,
  `interface/ast_types.rs`, `SPEC.md`.
- CLEAN-ROOM EVIDENCE OF DERIVATION: `workspace/QUERIES.log` (per-probe oracle
  interaction log), `workspace/BEHAVIOR.md` (inferred behavioral spec),
  `oracle/captures/*.out` (37 pre-captured oracle outputs).

## Verdict

**PASS. No violations survive filtration.** Every clean/Haskell (or
clean/wf.rs) similarity resolves to one of the three filtered categories:
(a) a name or signature explicitly listed in `required_api.md`; (b) an exact
message string present in observed oracle output (captures) or logged as a live
oracle observation in `QUERIES.log`/`BEHAVIOR.md`; or (c) structure forced by
the `ast_types.rs` data model or by the block/underline output format.

The clean code additionally *diverges* from `wf.rs` at every point the interface
does not constrain (see §6), which is positive evidence of independent
derivation rather than reproduction. One **process caveat** (not a clean-side
authorship violation, no redo) is recorded in §7: the interface header transmits
a port-specific function decomposition to the clean room.

---

## 1. report.rs

- `WfError { topic: String, message: String }`, `impl WfError::new(topic: impl
  Into<String>, message: impl Into<String>)`, and `pub type WfReport =
  Vec<WfError>` (report.rs:7-22) are **byte-for-byte the surface in
  `required_api.md:4-6`**. Haskell models this as `type WfError = (Topic, Doc)` /
  `type WfErrorReport = [WfError]` (Wellformedness.hs:112-114). The struct field
  name `message` and the shortened `WfReport` come from the interface header, not
  from Haskell (which has an unnamed `Doc` and `WfErrorReport`).
  → **Filtered: API names given in interface docs.**
- `underline_topic` (report.rs:29-33) renders `title + "\n" + "="*n`. Haskell
  `underlineTopic` (Wellformedness.hs:168-171) and `wf.rs:400-408` BOTH append a
  trailing `"\n"`; the clean version does **not** — it assembles the blank line
  between header and body in `render_block`/`render_report` instead. Divergence
  from both prior implementations; behavior (the `=` underline) is observable.
  → **Filtered (observed); note the divergence as independence evidence.**
- `SUCCESS_LINE`, the `/* WARNING: ... */` wrapper, and the block layout
  (`render_report`, report.rs:41-57) reproduce observed oracle output verbatim
  (BEHAVIOR.md §"Overall report shape", captures). → **Filtered: observed output.**
- `insert_wf_before` (report.rs:61-76) has the exact signature from
  `required_api.md:23`. Its body (`position(anchor)` → `split_off` → splice) is
  the only reasonable implementation of the documented contract and differs in
  expression from `wf.rs:115-126` (clean uses `match`, wf.rs uses
  `unwrap_or(len)`). Haskell has no analogue (it builds the report by a fixed
  `concatMap`). → **Filtered: API-given signature; body forced by contract.**

## 2. lib.rs

- `check_theory` (lib.rs:36-56) runs the checks in the **observed** canonical
  topic order (BEHAVIOR.md §"Canonical topic order", ks1/ks3 probes) and then
  splices `public_names_report` via `insert_wf_before(after_public_names_topics())`.
  This run-then-splice shape matches `wf.rs:146-195` and is *not* Haskell's shape
  (Haskell simply lists `publicNamesReport` 4th in a `concatMap`,
  Wellformedness.hs:1271-1283). The shape is transmitted by the interface, which
  exposes `insert_wf_before` and `after_public_names_topics` as required API.
  → **Filtered: API-given.** (See §7 process caveat.)
- `after_public_names_topics` (lib.rs:19-33) lists topic strings — all observable
  output headers — in observed order. The clean list contains **only** the
  AST-derivable topics and omits wf.rs's `"Fact capitalization issues"`,
  `"Lemma annotations"`, `"Multiplication restriction of rules"`,
  `"Message Derivation Checks"` (wf.rs `WF_TOPIC_ORDER`, wf.rs:83-100). Divergence
  from wf.rs. → **Filtered: API-given function; content is observed topic order.**
- `check_if_lemmas_in_theory` (lib.rs:62-74) — API-given signature
  (required_api.md:19). The clean body is a plain set-difference with a
  self-documented "message text is a gap" (it does not reproduce Haskell's
  `checkIfLemmasInTheory` wording, Wellformedness.hs:1156-1171). → **Filtered.**

## 3. checks.rs

Public check function names are all derived from the **topic strings**
(observable output): `unbound_variables`, `mismatching_sorts`, `reserved_names`,
`fr_facts`, `special_facts`, `fact_arity`, `fact_multiplicity`, `formula_terms`,
`formula_guardedness`, `nat_sorts`, `subterm_convergence`. None match Haskell's
internal names (`unboundCheck`, `sortsClashCheck`, `reservedFactNameRules'`,
`freshFactArguments'`, `specialFactsUsage'`, `factUsage`, `checkTerms`,
`checkGuarded`, `natWellSortedReport`, `checkEquationsSubtermConvergence`) and
none match wf.rs's internal names (`unbound_report`, `variable_sort_clashes`,
`reserved_fact_name_rules`, `fresh_fact_arguments`, `special_facts_usage`,
`fact_usage`, `nat_well_sorted_report`, `subterm_convergence_report`).
→ **Filtered: names derived from observable topics.**

Internal helpers (`collect_term_vars`, `collect_fact_vars`, `collect_pub_lits`,
`var_key`, `sort_rank`, `dedup_vars`, `variant_repr`, `gather_fact_uses`,
`render_fact_blocks`, `nearest_rhs`, `edit_distance`, `free_vars_formula`,
`atom_vars`, `guard_vars`, `find_unguarded`, `collect_nat_issues`, `is_subterm`,
`FactUse`, `FactId`, `protocol_rules`, `is_reserved`/`is_special`/
`is_builtin_factname`) — none share a name with a Haskell internal. Per-check
notes:

- **Unbound variables** (checks.rs:132-176): message text is observed
  (BEHAVIOR.md §Unbound; probes p01/ub_pub/z5). The bound/used partition
  (bound = premises ∪ let-bound; used = actions ∪ conclusions ∪ let-RHS) and the
  `var_needs_binding` set {Fresh, Msg, Untagged, Nat} **diverge** from Haskell
  `unboundCheck` (Wellformedness.hs:493-511), which excludes only `LSortPub` and
  the special `NOW` node and has no explicit let handling (it relies on already-
  substituted facts + `originatesFromLookup`). Clean excludes `Node` entirely and
  handles `let` explicitly. → **Filtered: not Haskell's expression; observable.**
- **Mismatching sorts** (checks.rs:181-241): header text observed (p20/ms2,
  BEHAVIOR.md:86). Clean groups by `to_lowercase(name)` with variant =
  sort-prefix+name (index ignored). Haskell `sortsClashCheck`/`clashesOn`
  (Wellformedness.hs:154-161, 258-272) groups by `(lowerCase name, idx)` with
  variant = full `LVar`. Different normalization keys. → **Filtered.**
- **Reserved names** (checks.rs:246-277): position-dependent reserved sets
  ({K,KU,KD} on L/R; +{In,Out,Fr} in the middle) and the phrase strings are
  explicitly observed (QUERIES.log z9/z11/z12; BEHAVIOR.md §Reserved names).
  → **Filtered: observed output + behavior.**
- **Fr facts / Special facts** (checks.rs:282-348): trigger conditions and the
  `on left-hand-side:` / `on right-hand-side:` / lowercase-`rule` wording are
  observed (p14/p15/sf_pos/z12; BEHAVIOR.md). → **Filtered.**
- **Fact arity / multiplicity** (checks.rs:399-471): intro strings and the
  numbered block layout are observed (p02/p03/f_arity3; BEHAVIOR.md:112-121).
  Clean shares one `gather_fact_uses` + `render_fact_blocks` across both checks;
  Haskell shares one `theoryFacts`/`allClashes` across cap/arity/mult
  (Wellformedness.hs:636-689). This shared-grouping shape is the natural
  consequence of both checks keying on per-fact-name grouping (behavior), and the
  clean side factors the two *formatters* into one shared helper whereas Haskell
  keeps `formatArityIssue`/`formatMultipIssue` separate. → **Filtered: behavior-
  driven DRY; decomposition differs from Haskell.**
- **fact_lhs_occur_no_rhs** (checks.rs:500-582): API-given name
  (required_api.md:20). Identity triple `(name, arity, persistent)` is literally
  printed in the output line (`factName \`X' arity: N multiplicity: Linear`), so
  it is observation-derived, not lifted from Haskell's `factInfo`
  (Wellformedness.hs:174-175). The `edit_distance ≤ 3` suggestion threshold and
  "nearest RHS by name distance" behavior are observed (th2/thN; BEHAVIOR.md:128).
  `edit_distance` is textbook two-row Levenshtein; the name is generic. Clean's
  index numbering does **not** left-pad to width (unlike wf.rs `numbered_index_
  width`, wf.rs:1353/431), a divergence. → **Filtered.**
- **Public names** (checks.rs:588-654): API-given `public_names_report` /
  `public_names_report_from_pairs` (required_api.md:21-22). Header text observed
  (p32/p38; BEHAVIOR.md:74-81). Clean's pair order is **`(name, rule)`** — the
  *opposite* of wf.rs's **`(rule, name)`** (wf.rs:1455) — and clean groups by
  first-seen order, whereas both Haskell (`clashesOn` → `sortOn`) and wf.rs sort
  by lowercased name. Strong divergence. → **Filtered; independence evidence.**
- **Formula terms** (checks.rs:657-755): the `` `Free x' `` rendering was
  observed live (QUERIES.log p05 `-> "Formula terms" (\`Free y')`), and the help
  text appears in `oracle/captures/…Typing_and_Destructors…out`. Clean tracks a
  binder stack (`free_vars_formula` push/pop on quantifiers) because
  `ast_types.rs` represents quantified vars by *name* (`Forall(Vec<VarSpec>,…)`),
  not de Bruijn — so the stack is forced by the data model. Haskell `checkTerms`
  (Wellformedness.hs:960-985) needs no stack because its LNFormula already carries
  `Bound`/`Free`. `atom_vars` (checks.rs:696-712) maps each `Atom` variant to its
  constituent terms — forced by the `Atom` enum in `ast_types.rs:311-320` — and
  even diverges from Haskell `atomTerms` (Wellformedness.hs:908-915) by including
  predicate-atom facts (Haskell returns `[]` for `Syntactic`). → **Filtered.**
- **Formula guardedness** (checks.rs:761-836): the message body
  ("cannot be converted to a guarded formula" / "unguarded variable(s) … in the
  subformula … in the formula") is documented as observed (QUERIES.log
  p21/p39; BEHAVIOR.md:140-145). The detection (`find_unguarded`/`guard_vars`:
  a quantifier is "guarded" if its bound vars all appear under some Action/Pred
  atom) is a self-declared partial heuristic and is nothing like Haskell's real
  `formulaToGuarded` conversion. → **Filtered: observed output; own algorithm.**
- **Nat Sorts** (checks.rs:842-888): message observed (t_nat; BEHAVIOR.md:147).
  Clean collects offending *vars* (sort ≠ Nat) under a `%+`; Haskell
  `nonWellSorted` (Wellformedness.hs:293-304) collects offending *terms* recursing
  through nested nat-plus. Different traversal. → **Filtered.**
- **Subterm convergence** (checks.rs:894-930): intro + manual-URL string appears
  in captures (OIDC/POIDC/eccDAA). `is_subterm` is a self-declared structural
  approximation, unrelated to Haskell's `filterNonSubtermCtxtRule`. → **Filtered.**

## 4. pretty.rs

Every rule traces to an observed probe (t_terms/t_xor/t_nat/f_nullary;
BEHAVIOR.md §"Term pretty-printer"). Sort sigils `~ $ % #`, pair flattening
`<…>`, `f(a, b)`, `'lit'`, `^` without parens, AC operators parenthesized,
`%1`, and `DH_neutral` (present in `oracle/captures/…RYY_PFS…out`) are all
observed output. Independent-derivation signals vs. wf.rs:

- `Term::NumberOne => "1"` (pretty.rs:87) vs. wf.rs `NumberOne => "one"`
  (wf.rs:466). **Opposite rendering.**
- Only `Xor` operands are sorted, by **rendered-string** sort (pretty.rs:71-74);
  wf.rs sorts **all** AC operators (Mult/Union/Xor/NatPlus) by a term `Ord`
  (`cmp_wf_term`, wf.rs:517-521). Different scope and different comparator.

→ **Filtered: observed output; clear divergence from wf.rs.**

## 5. formula.rs

Single-line unicode printer `pp_formula` (formula.rs:10-42). Operator glyphs
(⊥ ⊤ ¬ ∧ ∨ ⇒ ⇔ ∀ ∃ = < ⋖ ⊏ @ last(…)) are formula-rendering output; the
module header states it was "calibrated from the oracle (e.g. ks1 lemma L2)".
Clean **fully parenthesizes** every binary connective (`({}) ∧ ({})`), unlike
Haskell's precedence-minimized `prettyLNFormula`. → **Filtered: observed/
calibrated; own parenthesization.** (`⋖` LessMset / `⊏` Subterm may be rarer in
the probed lemmas, but both are standard notation and the module is declared a
documented partial; not worth a redo.)

## 6. Secondary check — divergence from replaced `wf.rs`

Instruction: flag only where clean matches `wf.rs` structure in ways Haskell and
behavior do not explain. **Result: no such match.** Every clean/wf.rs match is
covered by `required_api.md` (WfError, new, WfReport, insert_wf_before,
after_public_names_topics, public_names_report[_from_pairs], topics,
underline_topic, render_report, check_theory, check_if_lemmas_in_theory,
fact_lhs_occur_no_rhs). The clean code diverges from wf.rs everywhere the API is
silent:

| Aspect | clean | wf.rs |
|---|---|---|
| `_from_pairs` tuple order | `(name, rule)` | `(rule, name)` |
| public-names grouping | first-seen order | sorted (`clashesOn`) |
| `NumberOne` render | `"1"` | `"one"` |
| AC operand sort | Xor only, string sort | all AC, term `Ord` |
| `underline_topic` trailing `\n` | none | present |
| numbered index padding | none | width-padded |
| internal sort order (`sort_rank`) | Msg<Pub<Fresh<Node<Nat | Pub<Fresh<Msg<Node<Nat |
| check fn names | topic-derived | port-derived |

No Haskell-internal identifiers or `.hs:` source citations appear anywhere in the
clean sources (grep confirmed empty).

## 7. Process caveat (NOT a clean-side violation; no redo)

`required_api.md` hands the clean room three functions whose *existence* is a
port-specific implementation seam with **no behavioral justification derivable
from the oracle**: `insert_wf_before`, `after_public_names_topics`, and the
`public_names_report` / `public_names_report_from_pairs` split. The oracle never
exposes these seams; the clean room implemented them only because the interface
required them. Per the protocol these are "API names given in the provided
interface docs" and are explicitly *not* clean-room violations, so no redo is
issued to `wf-clean`. This is flagged upward to the process owners: the
interoperability header for future clusters should prefer conveying *behavior/
ordering* over port-internal function decompositions, to keep the clean room from
inheriting the port's architecture. The clean implementations of these functions
are, in any case, independently expressed (see §1, §2, §3).

---

## Round 2 incremental audit

Scope: only the round-2 additions — six new checks (`fresh_public_constants`,
`reserved_prefixes`, `diff_left_right` → "Left rule"/"Right rule",
`lemma_annotations`, `multiplication_restriction`), the rule pretty-printer
(`pp_rule`/`pp_fact_list`), the `fillSep`-style wrapper (`fill_words`, width 69),
`indent_block`, the diff-mode term/fact projection
(`project_term`/`project_fact`/`project_facts`/`rule_matches_projection`), and
the `lib.rs` ordering changes. Method unchanged: abstraction–filtration–
comparison against `Wellformedness.hs`, `Model/Rule.hs`, `Model/Fact.hs`,
`Term/Term.hs`, and `Text/Pretty.hs`.

**Round-2 verdict: PASS. No violations survive filtration (0 redo).** Every
round-2 string, layout, indent, width, ordering, and trigger is corroborated as
observed oracle output by a named probe in `QUERIES.log` and `BEHAVIOR.md`, and
by the byte-parity captures in `wf-clean/tests/fixtures/r2_*.txt` (each carries
the oracle's `WARNING …`/`*/` framing). The exact strings are therefore
compatibility content per PROTOCOL §"Forbidden … Exact output strings … MUST be
taken from observed oracle output". Where the round-2 code touches an algorithm,
it is either forced by the observed output and the `ast.rs` data model, or a
materially different (usually simpler / gapped) implementation than the Haskell —
divergences are recorded below as independence evidence.

### R2-1. Fresh public constants
- Clean `fresh_public_constants` (checks.rs:1038-1063), `collect_fresh_lits`
  (checks.rs:117-132) vs Haskell `freshNamesReport'`
  (Wellformedness.hs:444-452).
- Message "rule `NAME': fresh public constants are not allowed: …" and the
  premises→conclusions→actions collection order are observed (probe
  fpc_positions; BEHAVIOR.md:151-158). Haskell selects `LSortFresh`-sorted names
  via a generic `universeBi` traversal; the clean side walks the three fact lists
  explicitly and renders each literal through its own term printer (`~'foo'`).
  Different traversal, own name (`fresh_public_constants` is topic-derived; the
  Haskell internal is `freshNamesReport`). → **Filtered: observed output; not
  Haskell's expression. No redo.**

### R2-2. Reserved prefixes (diff mode only)
- Clean `reserved_prefixes` (checks.rs:1070-1111), `reserved_prefix_header_words`
  (checks.rs:1113-1131), `RESERVED_PREFIXES` (checks.rs:31) vs Haskell
  `reservedPrefixReport`/`reservedPrefixFactName` (Wellformedness.hs:796-808).
- The header text, the per-fact `<fact_pp>` line, and the second line
  `(ProtoFact <Mult> "<name>" <arity>,<arity>,<Mult>)` are observed verbatim
  (probes rp_multi/rp_decode; BEHAVIOR.md:160-172; fixture r2_reserved_prefixes
  shows `(ProtoFact Linear "DiffIntrPriv" 1,1,Linear)`). That second line is a raw
  Haskell `show` of a tuple, but it is emitted by the oracle, so it is
  compatibility content the clean side reconstructs from parts — not a memory
  reconstruction of an internal. Divergence: the clean prefix test is
  case-sensitive `starts_with` on `["DiffIntr","DiffProto"]` (checks.rs:1080)
  whereas Haskell lowercases first (`take 8 (map toLower name) == "diffintr" …`);
  independence evidence (and a latent behavioral difference, not a similarity).
  → **Filtered: observed output. No redo.**

### R2-3. `fill_words` — fillSep-style wrapper at width 69
- Clean `fill_words` (checks.rs:169-193), `FILL_WIDTH = 69` (checks.rs:36) vs
  Haskell `wrappedText = fsep . map text . words` (Wellformedness.hs:150-151)
  rendered through `Text.PrettyPrint.Highlight`'s `fsep`
  (Text/Pretty.hs re-exports; render via HughesPJ `P.render`, Class.hs:77-78 at
  the library default `lineLength = 100`, ribbon 1.5).
- The observed output wraps, so a wrapper is behavior-forced. `fill_words` is a
  from-scratch greedy column-fill (`col + 1 + wlen <= width`) at an *empirically
  measured* width 69 with a 2-space indent (BEHAVIOR.md:165-167 — "measured via
  rp_long/rp_med/rp_w47"); 69 is NOT a Haskell constant (HughesPJ uses 100/1.5).
  This is a materially different algorithm from the library's paragraph fill that
  merely reproduces the observed break column. → **Filtered: observed/empirical;
  own algorithm. No redo.**

### R2-4. Left rule / Right rule + diff projection
- Clean `diff_left_right` (checks.rs:1141-1164), `inconsistent_entry`
  (checks.rs:1166-1173), `rule_matches_projection` (checks.rs:1176-1180),
  `facts_pp` (checks.rs:1182-1184), `project_facts`/`project_fact`/`project_term`
  (checks.rs:1186-1221) vs Haskell `leftRightRuleReportDiff`
  (Wellformedness.hs:397-414), `getLeftRule`/`getRightRule` (Model/Rule.hs:824-831),
  `getLeftFact`/`getRightFact` (Model/Fact.hs:469-476),
  `getLeftTerm`/`getRightTerm`/`getSide` (Term/Term.hs:216-230).
- Body layout ("Inconsistent left/right rule", indent-4 rules, "w.r.t.", the
  `\n  \n` blank spacing) and the LEFT-before-RIGHT + "only the left is reported
  when both differ" behavior are observed (probes diff_left_right_mismatch,
  diff_right_rule_mismatch, diff_both; BEHAVIOR.md:174-184; fixtures
  r2_left_rule/r2_right_rule).
- Diff projection (the explicit round-2 focus): both replace `diff(a,b)` with `a`
  (left) / `b` (right) recursively, structurally preserving everything else — the
  only reasonable realization of the observed projection. But the clean
  decomposition is driven by the `ast.rs` `Term` enum (dedicated `Term::Diff`
  arm plus `App`/`AlgApp`/`Pair`/`BinOp`/`PatMatch` arms), not by Haskell's
  uniform `FAPP sym ts` with an `o == diffSym` test; the clean side parameterizes
  on a plain `bool left` where Haskell uses a 4-way `DiffType`
  (DiffLeft/Right/Both/None); and consistency is decided by comparing
  pretty-printed fact lists (`facts_pp`) rather than Haskell's structural
  `equalUpToAddedActions` on a reconstructed rule. Names diverge entirely
  (`project_*` vs `getLeft*`/`getSide`). → **Filtered: forced by observed
  behavior + data model; materially different expression. No redo.**

### R2-5. Lemma annotations
- Clean `lemma_annotations` (checks.rs:1227-1245) vs Haskell
  `lemmaAttributeReport` (Wellformedness.hs:924-932).
- String "Lemma `NAME': cannot reuse 'exists-trace' lemmas" and the trigger
  (`reuse` attribute AND `exists-trace`, not reuse alone) are observed (probes
  exists_trace_reuse/la_alltraces/la_multi; BEHAVIOR.md:186-190). The clean
  function name is topic-derived ("Lemma annotations") and differs from the
  Haskell internal `lemmaAttributeReport`. → **Filtered: observed output +
  behavior. No redo.**

### R2-6. Multiplication restriction of rules
- Clean `multiplication_restriction` (checks.rs:1254-1278), `collect_mult_terms`
  (checks.rs:137-155) vs Haskell `multRestrictedReport'`
  (Wellformedness.hs:1047-1099), `multTerms` (Wellformedness.hs:1094-1096).
- The three intro strings, the two indent-4 rule blocks, and
  "Terms with multiplication:  <terms>" (two spaces) are observed (probes
  mul_multi/mul_act/mul_pc/mul_terms; BEHAVIOR.md:192-206; fixture
  r2_multiplication). The clean check is materially simpler than Haskell's:
  Haskell abstracts reducible lhs symbols to fresh vars (`abstractRule`) and also
  reports "Variables that occur only in rhs:"; the clean side prints the SAME
  rule twice and omits the vars-only-in-rhs branch (both documented gaps,
  BEHAVIOR.md:201-204) — strong independence evidence. `collect_mult_terms`
  collects maximal `*`-subterms (stop at a product, else recurse) exactly as the
  observed output requires; the recursion arms follow the `ast.rs` `Term`
  variants, not Haskell's `FApp (AC Mult)` view. → **Filtered: observed output;
  own (gapped) algorithm; behavior-forced maximal-product collection. No redo.**

### R2-7. Rule pretty-printer (`pp_rule` / `pp_fact_list`)
- Clean `pp_rule` (pretty.rs:147-158), `pp_fact_list` (pretty.rs:129-136) vs
  Haskell `prettyNamedRule` (Model/Rule.hs:1280-1292), `prettyRuleRestrGen`
  (Model/Rule.hs:1253-1269), `prettyRule` (Model/Rule.hs:1276-1277).
- The whole layout — "rule (modulo E) NAME:", 3-space indent, `[ … ]` /`[ ]`
  fact lists, `-->` vs `--[ … ]->` — is observed (BEHAVIOR.md:244-254; fixtures).
  `pp_fact_list` mirrors Haskell's `ppFactsList` only in that both emit `[ … ]`,
  a trivial output-driven helper with a different name. The clean printer emits a
  single-line body and documents that the oracle wraps long rules (gap,
  BEHAVIOR.md:253-254) — Haskell's `sep`/`fsep` wraps; divergence. → **Filtered:
  observed output; trivial DRY; own single-line form. No redo.**

### R2-8. `indent_block`
- Clean `indent_block` (checks.rs:158-164) vs Haskell `nest`
  (used at Wellformedness.hs:403-411, 1056-1064). Generic "prefix every line with
  N spaces" utility; behaviorally the block-indent the observed output shows, with
  a generic name unrelated to `nest`. → **Filtered: generic helper. No redo.**

### R2-9. `lib.rs` ordering changes
- Clean `check_theory` (lib.rs:42-67) and the round-2 anchors added to
  `after_public_names_topics` (lib.rs:20-39 — T_RESERVED_PREFIX, T_LEFT, T_RIGHT,
  T_LEMMA_ANNOT, T_MULRESTRICT; T_FRESH_PUB deliberately omitted as it precedes
  public-names) vs Haskell `checkWellformednessDiff`/`checkWellformedness`
  (Wellformedness.hs:1248-1286).
- The sequence is the observed report/topic order (BEHAVIOR.md:65-84, probes
  comb1/diff_mega/diff_place). Notably the clean side lists the fact-family and
  diff checks as *flat, separate* functions, whereas Haskell buries them inside
  the `factReports`/`checkWellformednessDiff` `concatMap` groups — a different
  decomposition. The public-names splice via `insert_wf_before` is the same
  port-API seam already recorded in §7 (Round 1) and is not re-litigated here.
  → **Filtered: observed ordering; decomposition diverges from Haskell. No redo.**

### Round-2 cross-checks
- No round-2 internal identifier matches a Haskell internal (`project_*`,
  `collect_fresh_lits`, `collect_mult_terms`, `fill_words`, `indent_block`,
  `inconsistent_entry`, `rule_matches_projection`, `facts_pp`,
  `reserved_prefix_header_words`, `RESERVED_PREFIXES`, `FILL_WIDTH` — all absent
  from the Haskell). Public check names are topic-derived; `pp_*` follow the
  clean printer convention.
- No round-2 comment echoes a Haskell comment (checked against the block comments
  at Wellformedness.hs:1040-1046/1102-1109, 443/454/458, 795, 924); every round-2
  comment is behavior/probe-cited in the clean author's own words.
- No `.hs:` citation or Haskell-internal identifier appears in any round-2 clean
  source (grep-confirmed empty).
