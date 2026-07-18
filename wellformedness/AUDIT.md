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

---

## Round 3 incremental audit

Scope: the round-3 delta ONLY (BEHAVIOR.md §"Round 3 (Unit C)"): the term-based
`formula_terms` rewrite + `formula_terms_reducible`; the `pp_formula_wrapped`
multi-line formula printer (custom `Doc`/`lay` engine); `fact_capitalization`;
the two-mode guardedness detection; the lemma-sourced fact arity/multiplicity
gather; and the `fillSep`-69 wrapping (`fill_after_prefix`, and its reuse by the
fresh-const list). Method unchanged: abstraction–filtration–comparison against
`Wellformedness.hs`, `Theory/Constraint/System/Guarded.hs`,
`Theory/Model/Formula.hs`, `Theory/Text/Pretty.hs`, and
`Text/PrettyPrint/Class.hs`. Every suspicious identical string was cross-checked
against `QUERIES.log` and the byte-parity fixtures `tests/fixtures/r3_*.txt`
(each carries the oracle's `WARNING …`/`*/` framing = observed program output).

**Round-3 verdict: PASS. No violations survive filtration (0 redo).** Every
round-3 string, layout, width, indent, ordering, and trigger is corroborated as
observed oracle output by a named `r3_*` probe in `QUERIES.log` and by an
`r3_*.txt` fixture; where the code touches an algorithm it is either forced by
the observed output + the `ast.rs` data model, or a materially different (simpler
/ over-approximating) implementation than the Haskell. Two items a skeptical
reviewer would raise (`show_haskell_fact`, the `atom_terms` name) are analysed
explicitly below and both filter.

### R3-1. Formula terms — term-based rewrite + `formula_terms_reducible`
- Clean `formula_terms_reducible`/`formula_terms` (checks.rs:1116-1148),
  `term_is_ill_formed` (checks.rs:994-1018), `show_wf_term`/`show_wf_pair`
  (checks.rs:937-990), `atom_terms` (checks.rs:1023-1036), `collect_ill_terms`
  (checks.rs:1041-1075), `debruijn_index` (checks.rs:929-934) vs Haskell
  `checkTerms` (Wellformedness.hs:960-985), `formulaTerms`/`atomTerms`
  (Wellformedness.hs:908-920).
- The raw `` `Bound N' ``/`` `Free <pp>' ``/`f(a,b)`/`pair(a,pair(b,c))`/`'c'`
  renderings, the no-dedup source order, the temporal-first Action ordering, and
  reducibility gating are all observed (QUERIES r3_reducible/r3_debruijn*/
  r3_redfree/r3_nested/r3_freenest/r3_both/r3_shapes/r3_actord/r3_toppair;
  fixtures r3_reducible/r3_pair/r3_freenest). → **Filtered: observed output.**
- Filtration of the algorithm: Haskell's `allowed` predicate keys on an
  *irreducible* function-symbol set (`o \`S.member\` irreducible`) and specially
  whitelists `FUnion`; the clean side inverts to a caller-supplied **reducible**
  set (`reducible.contains(name)`, the interface entry point
  `formula_terms_reducible(thy,&reducible)` per required_api / BEHAVIOR.md) and
  has no union special-case. The clean side must also carry a binder **stack**
  (push/pop over `Forall`/`Exists`) to synthesise de Bruijn indices, because
  `ast.rs` represents quantified vars by NAME; Haskell needs none since its
  `LNFormula` already holds `Bound`/`Free`. Different key polarity + forced
  stack-tracking = materially different expression. → **Filtered: data-model-
  forced; not Haskell's predicate.**
- `atom_terms` (checks.rs:1023) is a non-observable internal helper whose name
  snake-cases Haskell's internal `atomTerms` (Wellformedness.hs:908). It survives
  filtration on two independent grounds: (a) it is a maximally obvious descriptive
  name — the function returns the terms of an `Atom`, and the `ast.rs` type is
  literally `Atom`, so the name merges with the idea (any competent author picks
  it); (b) the clean function's *behavior diverges* — it returns the temporal
  first then fact args for `Action`, and returns `Pred` argument terms, whereas
  Haskell `atomTerms` returns `[]` for the predicate atom (`Syntactic`). A shared
  obvious name over divergent behavior is not evidence of access. → **Filtered:
  obvious/merger name; divergent behavior. Noted, no redo.**
- `FORMULA_TERMS_HELP` (checks.rs:924) reproduces the fixed help paragraph with
  its exact wrap points and 2-space continuation indent. Haskell emits this via
  `wrappedText "…"` (Wellformedness.hs:968-973, an `fsep` re-wrap). The clean
  side hardcodes the *observed* wrapped bytes, present verbatim in
  `oracle/captures/…Typing_and_Destructors….out` and fixtures r3_guard_wide/
  r3_guard_and. → **Filtered: observed output; not the `fsep` mechanism.**

### R3-2. `fillSep`-69 wrapping (`fill_after_prefix`) and fresh-const reuse
- Clean `fill_after_prefix` (checks.rs:1081-1105), used by `formula_terms_entry`
  (checks.rs:1107-1112) and `fresh_public_constants` (checks.rs:1377-1404;
  fill at 1397) vs Haskell `prettyLNTermList = fsep . punctuate comma …`
  (Wellformedness.hs:146-147) and the `checkTerms` `fsep`
  (Wellformedness.hs:963-966) rendered through HughesPJ `P.fsep`
  (Class.hs:184, library default `lineLength=100`, ribbon 1.5).
- The token list wraps; a wrapper is behavior-forced. `fill_after_prefix` is a
  from-scratch greedy column fill (`col + 1 + w <= width`) that carries the
  comma inside each token and begins after a literal prefix, at the *empirically
  measured* width **69** with continuation indent **4** (QUERIES r3_wrap/r3_w2/
  r3_longname/r3_overflow/r3_freshwrap; fixtures r3_wrap6/r3_fresh_wrap). 69 is
  not a Haskell constant (HughesPJ uses 100/1.5); this is a distinct algorithm
  reproducing only the observed break column, and a second, prefix-aware variant
  of the already-cleared round-2 `fill_words` (checks.rs:170-194). → **Filtered:
  observed/empirical width; own algorithm.**

### R3-3. `fact_capitalization` + lemma-sourced arity/multiplicity gather
- Clean `fact_capitalization` (checks.rs:589-647), `render_fact_blocks`
  (checks.rs:567-580), `fact_arity`/`fact_multiplicity` (checks.rs:649-721),
  `gather_fact_uses` (checks.rs:526-565), `gather_formula_facts`
  (checks.rs:478-518), `show_haskell_fact` (checks.rs:464-474) vs Haskell
  `factUsage` (Wellformedness.hs:636-689: `capIssues`/`arityIssues`/
  `multipIssues`, `allClashes`, `formatCapIssue`, `capIssueMsg`) and the
  lemma-fact branch of `theoryFacts` (Wellformedness.hs:602-605).
- The topic "Fact capitalization issues", the two intro sentences, the
  `Fact \`send':` block, the item shape `Rule \`R1', capitalization "Send"`, the
  NO-dedup (every occurrence numbered), and the fact ordering are all observed
  (QUERIES r3_factcap/r3_capclean/r3_capord; fixture r3_fact_cap). → **Filtered:
  observed output.**
- `show_haskell_fact` reconstructs the string
  `Fact {factTag = ProtoFact Linear "Act" 2, factAnnotations = fromList [],
  factTerms = [Bound 2,Bound 1]}`. Although those tokens are Haskell record
  field / constructor names (`factTag`, `ProtoFact`, `factAnnotations`,
  `factTerms`, `fromList []`), this exact string is **emitted by the oracle**
  (Haskell `text (show fa)`, Wellformedness.hs:605) and is present verbatim in
  fixture r3_lemma_arity.txt and QUERIES r3_lemarity/r3_lemmult. Per PROTOCOL
  §"Forbidden … Exact output strings … MUST be taken from observed oracle
  output", it is compatibility content the clean side templates from the
  observation — not a memory reconstruction of an internal. The helper *name*
  labels what it reproduces (a Haskell `show`), it is not itself a Haskell
  identifier. This is the same category as the round-2 `(ProtoFact Linear
  "DiffIntrPriv" 1,1,Linear)` line (R2-2). → **Filtered: observed output.**
- Filtration of the decomposition: Haskell computes `allClashes` **once**
  (sort-by-lowercased-name → group → keep len>1) and *filters* that single list
  three ways (cap/arity/mult), and its `theoryFacts` puts **all rule facts before
  all lemma facts** (`<|>` order + stable `sortOn`). The clean side instead
  (a) gives `fact_capitalization` its **own** rule-only gather (premise/action/
  conclusion `Occ`s), separate from arity/mult; (b) shares a **single**
  `gather_fact_uses` across arity+mult only; and (c) gathers rules and lemmas
  **interleaved in theory-item source order** (BEHAVIOR.md:132) rather than
  rules-then-lemmas; and (d) explicitly dedups arity/mult items per
  `(label,owner,arity|persistent)` and, unlike Haskell `numbered'`
  (Class.hs:252-264), does not flush-right-pad the item numbers. Four structural
  divergences from `factUsage`. The one shared trait — grouping facts by
  lowercased name — is behavior (both checks key on per-name grouping, which is
  the observable group order). And the clean side factors the three near-identical
  Haskell formatters (`formatCapIssue`/`formatArityIssue`/`formatMultipIssue`)
  into one `render_fact_blocks` helper, a DRY refactor Haskell does not have.
  → **Filtered: behavior-driven grouping; decomposition diverges from Haskell.**

### R3-4. Two-mode guardedness detection
- Clean `formula_guardedness` (checks.rs:1154-1190), `GuardFail`
  (checks.rs:1215-1220), `find_guard_failure` (checks.rs:1239-1275), `guard_vars`
  (checks.rs:1194-1212), `guard_var_names` (checks.rs:1224-1228) vs Haskell
  `checkGuarded` (Wellformedness.hs:988-993) → `formulaToGuarded`
  (Guarded.hs:471-563: `convAll`/`convEx`, `noUnguardedVars`, `conjActionsEqs`,
  `remainingUnguarded`, `covered`).
- The wrapper message "Lemma \`{name}' cannot be converted to a guarded
  formula:", the two reason strings ("unguarded variable(s) '…' in the
  subformula", "universal quantifier without toplevel implication"), the
  failing-subtree selection (sub ≠ whole), and the `… in the formula …` framing
  are observed (QUERIES r3_gc/r3_and/r3_nestsub; fixtures r3_guard_wide/
  r3_guard_and). The two-variant `GuardFail` enum mirrors the **two observable
  outcomes**, not a concealed internal shape. → **Filtered: observed output.**
- Filtration of the algorithm: Haskell decides guardedness by the full
  polarity-tracking `formulaToGuarded` conversion — `conjActionsEqs` gathers only
  *top-level* conjuncts of the antecedent and treats `EqE` atoms as guards, and
  `remainingUnguarded`/`covered` reason about which side of an equation is
  covered. The clean `find_guard_failure` is a self-declared **permissive
  over-approximation**: `guard_vars` collects vars from *any* Action/Pred atom
  *anywhere* in the guard formula (no polarity, no `EqE` guard, no `covered`, no
  conjunctive-prefix restriction). The ∀ arm ("body must be `Implies`, else
  NoImplication; antecedent guards, then recurse guard/rest") and the ∃ arm are
  the natural realisation of the observed decision map (r3_gc) over a different,
  simpler engine. Nothing like Haskell's converter. → **Filtered: observed
  behavior; own (over-approximating) algorithm.**

### R3-5. `pp_formula_wrapped` multi-line formula printer
- Clean `pp_formula_wrapped` (formula.rs:168-170) and its engine — `Doc`
  (formula.rs:62-66), `flat` (68-74), `lay` (79-124), `parens` (127-129),
  `binop_doc` (131-141), `formula_doc` (143-156), `quantifier_doc` (158-163),
  `FORMULA_WIDTH=72` (55) — vs Haskell `prettyLFormula`
  (Formula.hs:471-511: `sep [opParens p' <-> ppOp op, opParens q']` for
  connectives, `sep [ Q vs. , nest 1 body ]` for quantifiers) rendered through
  HughesPJ `P.sep`/`P.nest` (Class.hs:96,182,186).
- The wrapped layout is observed byte-for-byte (fixtures r3_guard_wide/
  r3_guard_and; QUERIES r3_gwide/r3_gw/r3_qm/r3_and): quantifier breaks after the
  `.` with body hanging at base+2 (col 8); a binary connective breaks after its
  operator with the right operand hanging at the enclosing `(` column; operands
  are fully parenthesised (`opParens` is unconditional, Highlight.hs:58-59, and
  the parens appear in the oracle output). → **Filtered: observed output.**
- Filtration of the engine: the clean `Doc` is a bespoke 3-constructor mini-
  layout (`Text`/`Beside`/`Group(children,hang)` with a `flat`-fits test) — it
  does **not** mirror HughesPJ's `P.Doc` (Empty/NilAbove/TextBeside/Nest/Union).
  The "sep of two: [left+op] then [right]" and "[Q vars.] then [body]" groupings
  are *forced by the observed break-after-operator / break-after-dot layout*
  (there is no materially different way to emit exactly those breaks at those
  hang columns), and the clean HANG model even differs from Haskell's — clean
  hangs the quantifier body at `base+2` where Haskell uses `nest 1` relative to
  the `sep` column, and the effective page width **72** is an empirical
  measurement (r3_qm), not HughesPJ's 100. The "HughesPJ-style" comment is a
  generic algorithm descriptor (publicly-known pretty-printing), not an echo of
  the GPL Class.hs header. → **Filtered: observed layout; behavior-forced
  grouping; custom engine + empirical width.**

### Round-3 cross-checks
- No round-3 internal identifier matches a Haskell internal: `term_is_ill_formed`,
  `show_wf_term`, `show_wf_pair`, `collect_ill_terms`, `debruijn_index`,
  `fill_after_prefix`, `formula_terms_entry`, `gather_fact_uses`,
  `gather_formula_facts`, `show_haskell_fact`, `render_fact_blocks`,
  `fact_capitalization`, `GuardFail`, `find_guard_failure`, `guard_vars`,
  `guard_var_names`, `Doc`/`flat`/`lay`/`binop_doc`/`formula_doc`/
  `quantifier_doc`, `FORMULA_WIDTH` — none appear in the Haskell. The single
  near-collision, `atom_terms`↔`atomTerms`, is an obvious descriptive name over
  divergent behavior (R3-1). Public entry names (`formula_terms`,
  `formula_terms_reducible`, `formula_guardedness`, `fact_arity`,
  `fact_multiplicity`, `fresh_public_constants`) are topic-/interface-derived.
- No round-3 comment echoes a Haskell source comment (checked against
  Wellformedness.hs:634-635/691-694/893/917/959, Guarded.hs:463-465/521-522,
  Formula.hs:470/514, Class.hs:6-8); the clean comments describe observed
  behavior in the author's own words and cite `r3_*` probes / BEHAVIOR.md. The
  overlap between the `formula_terms` doc-comment and the emitted help paragraph
  tracks the *observable* help string, not the Haskell comment
  ("check that only bound variables and public names are used").
- No `.hs:` citation or Haskell-internal identifier appears in any round-3 clean
  source (grep of `clashesOn|conjActionsEq|remainingUnguarded|noUnguarded|
  theoryFacts|allClashes|factUsage|checkTerms|checkGuarded|factInfo|
  mostSimilar|universeBi|\.hs:` returned empty).

## Round 4 incremental audit

Scope: the working-tree delta since HEAD `63ed8a9` restricted to
`wellformedness/` — `checks.rs` (+395/−185 lines), `formula.rs`, `lib.rs`,
`checks_tests.rs` (+14 fixture-driven tests), `BEHAVIOR.md`, `QUERIES.log`, and
20 new `tests/fixtures/g4_*.txt` byte-parity fixtures + 78 `probes4/*.spthy`
inputs. Two closed gaps: (GAP 1) sort-aware quantifier binding + the new
`Quantifier sorts` topic; (GAP 2) a semantic guardedness decision procedure
replacing the round-3 over-approximation; plus the per-item formula-check bundle
assembly and a col-relative rewrite of the multi-line printer. Audited against
`Wellformedness.hs` (`checkQuantifiers` 948-957, `checkTerms` 960-985,
`checkGuarded` 988-993, `formulaReports` 999-1014, `prettyWfErrorReport`
118-125, `underlineTopic` 168-171) and `Guarded.hs` `formulaToGuarded`
(471-566). Every behavioral claim below was traced to a logged `probes4` entry
and, where load-bearing, re-run against the oracle.

### R4-1. Sort-aware binder stack (`SortClass`, `Binder`, `sort_class`/`var_class`)
- Clean `SortClass` (checks.rs:920), `sort_class` (928), `var_class` (938),
  `type Binder = (String, SortClass)` (943); the de Bruijn stack and
  `debruijn_index` (1019) now match on `(name, class)`, and every quantifier
  push carries the class (gather_formula_facts, collect_ill_terms). This
  replaces the round-2 name-only match.
- Behavior observed, not read: a node binder `#x` does not bind a message use
  `x` (`Free x`, fixture g4_g1_core.txt / probe g1_core); a message binder `x`
  does not bind a node use `#x` (`Free #x`, g4_g1_msgnode / g1_msgnode); all
  binders are counted so `All x #x. … h(x)` → `h(Bound 1)` (g4_g1_dbsort /
  g1_dbsort); same-name/different-sort binders coexist (g1_samename SUCCESS).
  `{Untagged, Msg, Suffix(Msg)}` collapse to one class — this is the round-2
  Untagged-vs-Msg false-positive fix re-expressed sort-aware, and Untagged is a
  *clean-side parser* tag, not an upstream one. → **Filtered: observed binding
  behavior; own `(name, class)` key over the clean AST's own sort model.** The
  upstream analogue is `frees`/`bvarToLVar` over `BVar`/`LSort`, structurally
  unrelated; no identifier or constant overlaps.

### R4-2. `Quantifier sorts` topic (`wrong_lsort`, `wrong_quant_sorts`, `quantifier_sorts_entry`)
- Clean `T_QUANT_SORTS = "Quantifier sorts"` (checks.rs:23), `wrong_lsort`
  (952), `wrong_quant_sorts` (962), `quantifier_sorts_entry` (984) vs upstream
  `checkQuantifiers` (Wellformedness.hs:948-957).
- Every token the clean code emits is boundary-observable: the header
  ``  Lemma `L' uses quantifiers with wrong sort:`` and each raw Haskell tuple
  `("x",LSortPub)` / `("y",LSortFresh)` appear verbatim in the oracle output
  (fixtures g4_qs_pub.txt, g4_qs_multi.txt) — upstream renders them with
  `text . show` of a `(name, LSort)` binder, so `LSortPub`/`LSortFresh` are
  *`show`-of-the-LSort at the boundary*, not internal constants lifted from
  source. The allowed set {Msg, Node, Nat} ↔ upstream's
  `[LSortMsg, LSortNode, LSortNat]` (line 957) is derived per-sort by probes
  qs_pub_prefix/qs_fresh_{prefix,suffix}/qs_nat_prefix/qs_msg_plain; binding
  order, Lemma/Restriction entities (qs_restr), and the width-69 fillSep
  (qs_multi, shared `fill_after_prefix` from R2/R3) are all observed. →
  **Filtered: observed output; per-sort-probed allow-set; shared fill helper.**
  The clean structure (walk binders, filter, format tuple) is the only way to
  reproduce the observed list and does not mirror upstream's `foldFormula`
  binder-collector.

### R4-3. Semantic guardedness (`collect_guard_vars`, `guard_key_set`, `unguarded_vars`, `find_guard_failure`)
- This is the strongest independence signal in the round. Upstream
  `formulaToGuarded` (Guarded.hs:471-566) is a **polarity-carrying converter**:
  `convert polarity` recurses with the De Morgan quantifier switch (`All↔Ex`
  under negation), `conjActionsEqs` collects **both** `Action` **and** `EqE`
  atoms as guard candidates, and `remainingUnguarded`/`covered` lets an
  equality guard a variable when one side's frees are disjoint from the
  unguarded set (the `GEqE` branch, lines 529-533). None of that appears in the
  clean code. The clean `collect_guard_vars` (checks.rs:1265) walks **Action
  atoms through `And` only**, `guard_key_set` (1280) keys them `(name, class)`,
  `unguarded_vars` (1288) set-differences, and `find_guard_failure` (1299)
  matches `Forall`/`Exists` directly with no polarity and no `EqE` handling.
- The clean model deliberately **under-approximates** upstream: it treats
  equality/ordering atoms as never-guarding. Confirmed against the oracle —
  gx_eq_ant (`All x. (x = x) ==> (x = x)`) reports `unguarded variable(s) 'x'`,
  matching the clean model — but note this probe does **not** distinguish the
  models, because upstream's `covered` test also fails when the same var sits on
  both sides. See the advisory below. The reasons, subformula selection
  (failing quantifier subtree, whole-lemma context), pre-order first-failure
  (antecedent-before-consequent gr_both, left-before-right gr_sib), and
  lemma-only scope (gr_restr: unguarded restriction is fatal, not a warning) are
  all probe-backed. → **Filtered: observed decision map realised by an
  independent, simpler engine that shares no structure, identifier, or constant
  with the GPL converter.**

### R4-4. Formula-check bundle assembly (`formula_items`, `merge_consecutive`, `formula_reports`)
- Clean `formula_items` (checks.rs:1379) = all lemmas (source order) then all
  restrictions; `formula_reports` (1414) emits per item in sub-order
  [Quantifier sorts, Formula terms, Formula guardedness(lemma-only)];
  `merge_consecutive` (1396) joins only **runs** of same-topic entries.
- This parallels upstream `formulaReports` (`annFormulas` = lemmas `<|>`
  restrictions; per-item `msum [QuantifierSorts, FormulaTerms, Guardedness]`,
  Wellformedness.hs:999-1014) and `prettyWfErrorReport = … map ppTopic .
  groupOn fst` (118-125), where `groupOn` groups **consecutive** equal keys.
  The parallel is with *observable emission/grouping order*, every facet of
  which is independently probed: lemmas-before-restrictions (qs_restr,
  ord_rl_ft2), per-item sub-order (syn_pubsuffix, ord_ftqs/ord_qsft),
  consecutive-only merge → three blocks for QS,FT,QS (ord_qs_ft_qs) vs merge for
  two adjacent QS (qs_twolem), and `Lemma annotations` as a separate global
  check after the bundle (ord_la_ft2). No `groupOn`/`msum`/`annFormulas`
  idiom or name is reproduced; `merge_consecutive` is a plain fold. → **Filtered:
  observed report order & grouping; independent assembly.**

### R4-5. Col-relative multi-line printer (`lay`, `relation_doc`, `atom_doc`)
- The round-4 printer change drops the `base` parameter and hangs at `col+hang`
  (`lay` formula.rs:79-124); adds `relation_doc` (148) / `atom_doc` (158) so
  relational atoms (`= < ⋖ ⊏`) break at the operator with **unparenthesised**
  term operands; quantifier body hang moves 2→1. All calibrated to observed
  break columns: top-level `A ∧ B` right operand at `col+0` (gt_and_two_all,
  fixing round-3's top-level off-by-one), nested quantifier body at
  `start+1` and relational/binop operand at `start+0` (gnest, fixture
  g4_gnest.txt), single-line action atoms (gact). Page width 72 stays an
  empirical measurement, not HughesPJ's default. The bespoke `Doc`
  (`Text`/`Beside`/`Group`) still does not mirror HughesPJ `P.Doc`. → **Filtered:
  observed layout; behavior-forced grouping; custom engine + empirical width.**

### Advisory (non-blocking; not a redo)
- The comment at checks.rs:1265-1279 and BEHAVIOR.md claim "equality/`<`/`⋖`/
  `⊏`/`last` atoms … contribute NO guards", citing probe **gx_eq_ant**. That
  probe (`All x. (x = x) ==> (x = x)`) only exercises an equality whose *both*
  sides carry the quantified var, which fails to guard under **both** the clean
  model and upstream's `covered`/`remainingUnguarded` (Guarded.hs:529-533).
  It does **not** probe the case upstream would guard — an equality with one
  covered (var-free/closed) side, e.g. `Ex x. x = 'c'` — so the general claim
  "equality never guards" *exceeds what the logged probe establishes* and is a
  latent behavioral divergence from the reference on unprobed inputs. This is a
  **correctness/completeness gap, not copied expression**: the clean code
  *under*-claims relative to the GPL source (it omits the `covered` equality
  logic entirely), which is consistent with clean-room independence rather than
  copying, and no committed `g4_*` fixture depends on it. Recommended (for the
  clean-room team, not a relicensing blocker): add a probe such as
  `Ex x. x = '<pub>'` / `All x. (x = '<pub>') ==> …` and either confirm the
  simplification or narrow the BEHAVIOR.md claim to "an equality both of whose
  sides mention an unguarded var does not guard it".

### Round-4 cross-checks
- New internal identifiers — `SortClass`, `sort_class`, `var_class`, `Binder`,
  `wrong_lsort`, `wrong_quant_sorts`, `quantifier_sorts_entry`,
  `collect_guard_vars`, `guard_key_set`, `unguarded_vars`, `guardedness_entry`,
  `formula_items`, `merge_consecutive`, `formula_reports`, `ill_terms`,
  `relation_doc`, `atom_doc`, `T_QUANT_SORTS` — none appears in the Haskell.
  Grep of the round-4 delta for `checkQuantifiers|checkTerms|checkGuarded|
  formulaToGuarded|conjActionsEqs|remainingUnguarded|openFormulaPrefix|convAll|
  convEx|annFormulas|boundTerms|avoidPrecise|gnotAtom|closeGuarded|sortGAtoms|
  atomToGAtom|foldFormula|prettyLNFormula|groupOn|\.hs:` returned empty.
- Boundary-observable strings only: `"Quantifier sorts"`, `("x",LSortPub)`,
  `("y",LSortFresh)`, `Bound n`/`Free v`, `uses quantifiers with wrong sort:`,
  `unguarded variable(s) … in the subformula`, `universal quantifier without
  toplevel implication`, `cannot be converted to a guarded formula`, and the
  `====` underline (matching `underlineTopic`'s `length topic` rule incl. the
  leading space of `" Formula guardedness"`) all appear verbatim in the
  captured `g4_*` fixtures — merger/behavior-dictated, not lifted constants.
- Comments: no round-4 comment echoes a Haskell source comment; they describe
  observed behavior and cite `g1_*`/`gu_*`/`ge_*`/`gx_*`/`gr_*`/`gt_*`/`qs_*`/
  `ord_*` probes. The lone `HughesPJ-style` descriptor (formula.rs:6) is a
  public pretty-printing-algorithm name, already accepted in R3-5 as a generic
  descriptor, not an echo of the GPL `Class.hs` header. (Consistency nit, not a
  violation: round-4 removed the other `HughesPJ`/`sep [...]` comment references
  but left this module-doc line; the team may wish to align it.)
- Tests are fixture-driven byte-parity (`expect(&thy,
  include_str!("fixtures/g4_*.txt"))`) over ASTs hand-built from the clean AST;
  expected strings come from captured oracle output, not transcribed source.

### Round-4 verdict
No copied protectable expression. All new strings, sort tokens, `Bound`/`Free`
renderings, wrap widths, topic underlines, and report ordering are
boundary-observable and each traces to a logged `probes4` entry; the guardedness
engine and printer are independent (and, for guardedness, strictly simpler than)
the GPL sources. One non-blocking correctness advisory (equality-guard claim
under-probed) is recorded for the clean-room team. **PASS.**

## Round 5 incremental audit

Delta scope (git HEAD 7980f8d, working tree): `workspace/wf-clean/src/checks.rs`,
`workspace/wf-clean/src/pretty.rs` (source), plus `BEHAVIOR.md`, `QUERIES.log`,
and untracked round5/ + tests/round5_tests.rs + tests/corpus5_acceptance.rs +
fixtures/probes. Reference read for comparison: `Guarded.hs`
(`formulaToGuarded`/`convert`/`convEx`/`convAll`/`conjActionsEqs`/
`remainingUnguarded`/`covered`/`sortGAtoms`/`atomToGAtom`/`openFormulaPrefix`),
`Wellformedness.hs` (`sortsClashCheck`/`clashesOn`/`publicNamesReport'`/
`mostSimilarName`/`factLhsOccurNoRhs'`/`checkEquationsSubtermConvergence`),
`SubtermRule.hs` (`isSubtermConvergentCtxtRule`), `Formula.hs`
(`openFormula`/`openFormulaPrefix`).

### This round closes the Round-4 advisory (equality guards) — the RIGHT way
Round 4 recorded a non-blocking advisory: the clean code *under*-claimed
("equality never guards", probe gx_eq_ant = `x=x` only) relative to upstream's
`covered`/`remainingUnguarded` (Guarded.hs:523-533), and recommended adding a
probe such as `Ex x. x='c'` to confirm or narrow. Round 5 did exactly this via
the black box: g5_e_eqbare (`Ex z. z='c'` -> SUCCESS), g5_e_eqinner,
g5_e_eqself/eqtwo (fail), g5_e_eqchain vs g5_e_revchain (single L-to-R pass, no
fixpoint), g5_e_actorder (actions before eqs), g5_e_unif (`<z,'c'>=<'c',w>` ->
unguarded: side-based, NOT unification). The resulting mechanism converges on
upstream's semantics, which is *merger* (the decision must accept exactly what
the reference accepts); the question for this audit is whether the convergence
was reached by copying protectable EXPRESSION or by probing. Evidence says
probing:

- **Discriminators are captured, not inferred from source.** The two behaviors
  hardest to guess without either the source or a probe — the SINGLE
  left-to-right equality pass (no fixpoint) and actions-resolved-before-
  equalities — are each pinned by a real oracle capture: g5_e_revchain.wfblock
  (`(w=h(z)) & (z='c')` -> "unguarded 'w'") and the g5_e_actorder probe. Upstream
  reaches "actions first" via `sortGAtoms = uncurry (++) . partition isGAction`
  and "single pass" via the `go` fold in `remainingUnguarded`; the clean code
  reaches the same observable set via `collect_guard_vars` (all actions) then a
  single loop over `collect_guard_eqs`. Different decomposition, identical
  boundary behavior.
- **Upstream's known incompleteness is reproduced as an observation, not a
  transcription.** Guarded.hs:527-533 has a FIXME that `covered` does not do
  full unification (whole-side var test only). The clean code has the same
  limitation, pinned by g5_e_unif — arrived at black-box, and its comment says
  "side-based, not unification" (upstream's word is "covered"; no lineage).

### Abstraction / filtration / comparison — guardedness (Family 1, 49 files)
- **Idea/merger (unprotectable):** guardedness DECISION (accept/reject, which
  var, which of the two reasons), same-kind prefix fusion, action-atom guards,
  one-clean-side equality resolution, (name, index, sort-class) binder identity.
  Each is behavior-forced and each traces to a probe capture (g5_e_*, g5_a_*,
  g5_idx) or to the corpus pin (684 formulas, tests/corpus5, all accepted).
- **Filtered clean expression:** `fuse_quantifiers` (recursive same-kind fusion
  over the whole tree) is NOT upstream's `openFormulaPrefix` (outermost-prefix
  opening inside a MonadFresh alpha-renamer); `resolved_guard_keys` +
  `unresolved_side` + `formula_var_key`/`VKey` is NOT
  `remainingUnguarded`/`covered`/`frees`/`\\`; the clean code carries NO polarity
  machinery, NO `gconj`/`gdisj`/`closeGuarded`/`GAtom`/`Precise.evalFreshT`/
  `avoidPrecise`, i.e. it implements only the wf-observable decision, not the
  guarded-formula datatype. The fused-form RENDERING (g5_a_nest_noimpl:
  "∀ x #i. A( x ) @ #i") is a property of the reference pretty-printer captured
  in g5_a_nest_noimpl.wfblock, not lifted from `ppFormula f0`.

### Abstraction / filtration / comparison — sort clash (Family 2, 15 files)
- Upstream `sortsClashCheck` keys on `removeSort lv = (lowerCase (lvarName lv),
  lvarIdx lv)` and lists variants via `clashesOn removeSort id` +
  `sortednubOn id` (LVar Ord = sort-then-name) + groups by `sortOn f`. The clean
  code's `key = (v.name.to_lowercase(), v.idx)`, `class_rank` ($<~<msg<%<#),
  `variant_string`, and group `sort_by(a.0)` reproduce this exact ordering.
  Merger on the key/order; the integer ranks 0..4 are the clean code's own,
  calibrated to s5_all4/s5_crossname/s5_capord captures (not the Haskell
  data-declaration Ord). Public-names rework (first-occurrence attribution,
  spelling sort, consecutive-same-rule merge, group sort) mirrors
  `sortednubOn (show.snd)` + `groupOn fst` + `clashesOn`, and is byte-pinned by
  the issue527 target (`rule "Four":  name 'firSt', rule "Two":  name 'first'`).
  No `clashesOn`/`sortednubOn`/`groupOn`/`removeSort`/`ppRuleAndName`/
  `findClashes`/`prettyVarList` identifiers appear in the clean code.

### Abstraction / filtration / comparison — tail (Family 3, 7 files)
- Builtin-fact normalization (`canon_builtin`/`canon_fact`): case-insensitive
  {k,ku,kd,in,out,fr} -> canonical name + persistence. Every element is
  boundary-observable — `!KU( x )` persistence and the case folding are in
  t5_ku_all.wfblock / t5_up_inout.wfblock; the K-participates-in-lhs/rhs and
  the builtins-are-excluded split reflects upstream `isProtoFact` and is pinned
  by the issue515/issue527 targets. Subterm convergence: `term_has_vars`
  reproduces upstream `isConstant rhs = null (frees rhs)` (SubtermRule.hs:109);
  the sorted listing reproduces the Set-ordering of `thyEquations`, pinned by
  t5_sub_order2 and the ble/mesh targets. Right-aligned item numbers and
  per-render arity/mult dedup are pinned by t5_align.wfblock / t5_lemdup.wfblock.
- **Equation layout engine (`pretty::pp_equation`/`EqLayout`/`lay_eq_term`):**
  the one genuinely expression-heavy new block. It is a bespoke fill/break
  engine, NOT a port of Text.PrettyPrint.HughesPJ (no Doc algebra, no
  `Union`/`fits`/`nest` combinators). The constants `min(100, 67 + nest)` are
  the render-boundary line width (100) and ribbon (100/1.5≈67); both are
  boundary-recoverable — they set exactly where wrapping occurs, and are pinned
  byte-exact by t5_wl55..66, t5_last36/37.wfblock, t5_tup3/3b.wfblock and the
  mesh k2 ten-line reference block. Filtered: observed layout; custom engine;
  boundary-observable widths.

### Round-5 cross-checks
- **Identifier scan** of the delta (checks.rs, pretty.rs) for
  `guardConj|guardEx|openFormula|conjActionsEqs|remainingUnguarded|covered|
  sortGAtoms|atomToGAtom|GAtom|GAction|GEqE|closeGuarded|formulaToGuarded|
  noUnguardedVars|isSubtermConvergent|filterNonSubterm|clashesOn|removeSort|
  sortednubOn|mostSimilarName|prettyVarList|freshNamesReport|partitionEithers|
  bvarToLVar`: **empty**. New names (`canon_builtin`, `canon_fact`,
  `class_rank`, `variant_string`, `formula_var_key`, `VKey`, `fuse_quantifiers`,
  `collect_guard_eqs`, `resolved_guard_keys`, `unresolved_side`, `eq_margin`,
  `EqLayout`, `lay_eq_term`, `tuple_elems`, `term_has_vars`) — none appears in
  the Haskell.
- **Comment-lineage scan** for `implicit negation|quantifier switch|FIXME|We do
  not consider the terms|not last-free|cannot happen|is not a guarded atom`:
  **empty**. Delta comments describe observed behavior and cite g5_*/s5_*/t5_*
  probes or named targets (issue515/issue527/Axioms_and_Induction/ble/mesh).
- **Boundary-observable strings only:** `!KU( x )`, `Fr( x )`, `Out( x )`,
  `∀ x #i.`, `∃ z w.`, `unguarded variable(s)`, `universal quantifier without
  toplevel implication`, the subterm-warning prose and manual URL all appear
  verbatim in captured `.wfblock`/target files.
- **Provenance of every behavioral claim:** oracle is the compiled HS binary
  via `oracle/wf_oracle.sh`; probes are `gprobe5.sh` -> `wf.sh` black-box
  captures (22 `.wfblock` files spot-checked: g5_a_nest_noimpl, g5_e_eqself,
  g5_e_fuse_all, g5_e_revchain, g5_e_vac, s5_all4/crossname, t5_up_inout/ku_all/
  lemdup/align/last37 all match their BEHAVIOR.md claims); the 71 targets are
  materialized by `fetch_hs_targets.sh` from the same binary on permitted
  corpus inputs; the 684-formula corpus pin (`tests/corpus5_acceptance.rs`)
  asserts acceptance of exactly what the reference prints. No claim traces to
  Guarded.hs/Wellformedness.hs rather than a probe or capture.
- Minor non-blocking note (not a redo): `canon_builtin` lower-cases before
  matching `"k"`, so it would canonicalize a lowercase `k` fact to `K`; because
  Tamarin fact names are uppercase-initial this branch only ever fires on `K`,
  so it is inert — a robustness nit, not a divergence and not a copyright issue.

### Round-5 verdict
No copied protectable expression. The guardedness/sort-clash/tail changes are a
faithful reproduction of reference BEHAVIOR (merger) reached through logged
black-box probes and captured targets, with independent expression: distinct
identifier constellation, no comment lineage, only boundary-observable strings
and widths, and a decision engine that is structurally simpler than (and carries
none of the internal datatype machinery of) the GPL sources. The Round-4
equality-guard advisory is now correctly closed by probe-derived evidence.
**PASS.**

VERDICT: pass
