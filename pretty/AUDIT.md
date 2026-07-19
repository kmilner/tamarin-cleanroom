# PRETTY cluster — both-sides similarity audit

Auditor: sealed-campaign similarity auditor (barrier-exempt; read both sides).
Method: abstraction–filtration–comparison (AFC). Byte-parity-forced output
(glyphs, spacing, orderings observable in the theory echo) is filtered as
merger/compatibility content; the residue examined for copied protectable
EXPRESSION is: upstream identifier constellations, internal helper structure
not forced by the boundary, non-observable constants, and comment lineage.

Open side (round delta): `pretty/workspace/pretty-clean/` — R1 term core +
signature block, plus the vendored `doc.rs` engine. Scope = the working-tree
delta over cleanroom HEAD `cbc511d` restricted to `pretty/` (scaffold at HEAD;
this round fills `src/{ast,doc,lib,signature,term}.rs`,
`tests/round1_term_signature.rs`, and adds `workspace/{BEHAVIOR.md,QUERIES.log}`).

Sealed side (upstream Haskell, `tamarin-rs/tamarin-prover/`):
* term printer — `lib/term/src/Term/Term.hs` `prettyTerm` (270–301);
  var/lit — `lib/term/src/Term/LTerm.hs` `Show LVar` (526–533), `sortPrefix`
  (190–195).
* signature block — `lib/term/src/Term/Maude/Signature.hs`
  `prettyMaudeSigExcept` (220–250); embedding —
  `lib/theory/src/TheoryObject.hs` `prettyTheory` (741–753); equation rule —
  `lib/term/src/Term/SubtermRule.hs` `prettyCtxtStRule` (120–123); builtin
  symbol/rule tables — `lib/term/src/Term/Builtin/{Signature,Rules}.hs`.
* Doc engine — BSD `Text.PrettyPrint.HughesPJ` (via `pretty-1.1.3.6`), wrapped
  by the GPL v3 module `lib/utils/src/Text/PrettyPrint/Class.hs`
  (`Highlight.hs`).

---

## Round 1 — term core + signature block

### 1. Provenance ledger (BEHAVIOR.md / QUERIES.log)

Cross-checked for the hardest-to-guess behaviors, per method requirement.

* **Operator → glyph / AC-paren / exp-flatten** — every case in term.rs carries
  a probe or target citation (BEHAVIOR.md "Term rendering" table + composite
  section; QUERIES.log t_xor, t_uni, t_nat, t_mult2, t_exp2, t_pair, t_diff,
  t_one, t_gone, t_frlit, t_num2, t_uniwide, t_xorwide3). The two constructs
  that genuinely cannot be forced through the no-prove MSR echo — `PatMatch`
  and `SortHint::Suffix` — are explicitly logged as UNOBSERVABLE and rendered
  as flagged placeholders, not guessed-as-pinned. Correct discipline.
* **Wrap decisions** — the ribbon-73 fit boundary, fill continuation alignment,
  and the sep (all-or-nothing) vs fsep (fill) distinction for the equations
  block are each tied to a probe (e_mid at exactly 73, e_conv for sep
  semantics, b_all for fsep wrap). The exp-chain and Mult/NatPlus wide-break
  choice is honestly flagged "by structural analogy … to be confirmed by the
  full-corpus gate" (the wide variant probes time out) rather than asserted.
* **Builtin symbol lists** — each expansion (hashing→h/1; asym→adec,aenc,pk;
  signing→pk,sign,true,verify; sym→sdec,senc; revealing→getMessage,pk,
  revealSign,revealVerify,true; locations-report→check_rep,get_rep,rep,report;
  base fst/pair/snd) and each induced equation is pinned to a `b_<name>` probe.
  These lists ARE observable in the `functions:`/`equations:` echo, so they are
  compatibility content whose provenance is logged. Verified faithful to
  upstream `Builtin/Signature.hs` + `Rules.hs` (as they must be — same
  observable target); the clean crate's REPRESENTATION (inline `decl(..)` /
  `eq(..)` match arms → `Vec` values) bears no resemblance to upstream's
  top-level `Set` bindings of `NoEqSym` tuples and `CtxtStRule`/`StRhs`.

Ledger is a genuine deliverable, not a post-hoc rationalization: it records
FAILED probes (t_exp/t_mult timeouts, e_samehead parse-reject, t_xorwide
500 s timeout) and the workarounds, which is the fingerprint of real
black-box experimentation.

### 2. Doc layout engine — BSD, not the GPL wrapper (key confirmation)

`src/doc.rs` is byte-identical to the already-audited graphdot clean-room BSD
port (`graphdot/workspace/graph-clean/src/pretty.rs`) except for: (a) the
module doc-comment (reworded, attribution made MORE explicit — names the
sanctioned source path); (b) one added public entry point `render_with`
(explicit integer ribbon; a 4-line adapter over the existing `best` +
`display_page`); (c) the test module (graphdot's exploratory
tuple-construction bench replaced with a `render_with` unit test). The entire
layout algebra — `Doc` constructors, lazy `force`/thunk mirror, `beside`/
`above`, `sep`/`cat`, `fill`/`fcat`/`fsep`, `best`/`nicest`/`fits`,
`render_page`, `lay` — is carried over unchanged.

Provenance is BSD, confirmed by construction of the GPL side:
`Text.PrettyPrint.Class` (Copyright 2011 Simon Meier, **GPL v3**) is a thin
`newtype Doc = Doc P.Doc` wrapper whose `Document` instance DELEGATES every
method to `P.*` (= BSD `Text.PrettyPrint.HughesPJ`): `sep = Doc . P.sep …`,
`fsep = P.fsep`, `fcat = P.fcat`, `nest = P.nest`, `$$ = P.$$`, etc.
(Class.hs 172–187). The layout ALGORITHM that doc.rs actually ports (the
union/`fits`/`nicest` fitting machinery) does not exist in the GPL wrapper — it
lives only in the BSD library. The wrapper's own additions — `keyword_`,
`lineComment_`, `nestBetween`, `nestShort`, `symbol`, `numbered`, `<->`
(=`<+>`), `$--$`, `HighlightDocument` — are GPL-original; a scan confirms NONE
of them appear in doc.rs. Every public combinator in doc.rs corresponds to a
BSD `HughesPJ` export (`punctuate` and `hang` included — both BSD exports;
`hang = sep [d1, nest n d2]` is the canonical BSD definition, not the
wrapper's contribution).

**License preservation:** the BSD `LICENSE` and `pretty.cabal` are present
in-repo at `graphdot/sanctioned/pretty-1.1.3.6/`; doc.rs (2–9) and
`Cargo.toml` both attribute the BSD provenance and name that sanctioned source.
Resemblance to the BSD source is licensed — no violation. (Non-blocking note,
below, on carrying a LICENSE copy alongside the crate at standalone
distribution time.)

### 3. Term core (src/term.rs) — AFC comparison

* **Glyphs / sigils / literals** — `ac_doc` glyphs (`*`,`⊕`U+2295,`++`,`%+`),
  `sortPrefix` sigils (``,`$`,`~`,`#`,`%`), `%1`/`one`/`DH_neutral`, quoting
  `'x'`/`~'n'`: all appear verbatim in the echo → byte-forced compatibility
  content, filtered. (Upstream counterparts: Term.hs 287–290, 280; LTerm.hs
  191–195, 528–533.) The `var_str` name/index rule (`name`, or `name.idx` when
  idx>0) is the observable projection of `Show LVar`; independent code, forced
  output.
* **AC operators and pairs — DIVERGENT construction (independent).** Upstream
  `ppTerms sep n lead finish` = `fcat . (text lead :) . (++[text finish]) .
  map (nest n) . punctuate (text sep)` — delimiters are ELEMENTS INSIDE the
  fcat and each operand is `nest 1`. The clean crate instead brackets from the
  OUTSIDE — `char('(') <> fcat(punctuate(op, leaves)) <> char(')')` (term.rs
  142–154) and `char('<') <> fcat(punctuate(", ", …)) <> char('>')` (100–105),
  no per-element nest. Both yield the observed "continuation aligned after the
  bracket" because placing the bracket before the fcat sets the fcat origin one
  column in — a different Doc expression reaching the same bytes. Graphdot's
  retained exploration bench shows the implementer empirically searched these
  constructions. Not a copy.
* **Exp — DIVERGENT (independent).** Upstream renders exp by simple recursive
  `ppTerm t1 <> "^" <> ppTerm t2` (Term.hs 278); flat output falls out of
  associativity, never wraps. The clean crate explicitly COLLECTS the exp spine
  (`collect_exp`) and `fcat`s it (term.rs 118–137) — a structurally distinct
  approach dictated by the clean crate's binary-`BinOp` input model (upstream
  stores AC/exp pre-flattened under `FApp`). Divergence, not copying; the fcat
  wrap latitude is the one honestly-flagged unpinned point.
* **Pair flattening** — upstream `split` right-peels the binary `FPair` spine;
  the clean crate's `collect_pair` recurses into the LAST element of its n-ary
  `Vec` when it is a `Pair` (term.rs 107–114). Same result (`<<x,y>,z>` keeps
  the left nest; right nest flattens) via a model-appropriate, independently
  written traversal.
* **Application — merger coincidence, filtered.** `app_doc` =
  `text(f++"(") <> fsep(punctuate(comma,args)) <> ")"` (term.rs 88–94) matches
  upstream `ppFun` (Term.hs 299–300). This idiom is compelled by the observed
  boundary — attached commas + fill spaces = `fsep(punctuate comma …)`,
  wrap-after-`(` = bracket before the fill, `)` attached last — and is the
  minimal Doc expression for those exact bytes (BEHAVIOR.md derives it from
  probe:t_wide directly). It is also the clean crate's own uniform
  "delimiters-outside" style, which DIVERGES from upstream for pair/AC in the
  same file; the app coincidence is merger, not transcription. Filtered; no
  finding.

### 4. Signature block (src/signature.rs) — AFC comparison

* **Byte-forced surface, filtered:** builtins canonical order (diffie-hellman <
  bilinear-pairing < multiset < natural-numbers < xor), bilinear-pairing
  inducing diffie-hellman, the four `showAttr` spellings
  (`[destructor]`/`[private,destructor]`/`[private,constructor]`/none), the
  `name/arity` form, ASCII-byte function sort, exact-dedup, the header comment
  string, and `equations [convergent]:` — all appear in the echo. (Upstream
  counterparts: Signature.hs 233–247; embedding TheoryObject.hs 746.)
  Compatibility content; provenance logged (b_all, f_attrs, f_sort, e_conv).
* **Equation block indent — DIVERGENT construction (independent), puzzle
  resolved.** Upstream produces the observed 4-space indent as TWO composed
  nests: `sep (header : map (nest 2) eqs)` (block, Signature.hs 224–226) plus
  `sep [nest 2 lhs, "=" <-> rhs]` inside each equation (prettyCtxtStRule,
  SubtermRule.hs 122–123). The clean crate folds these differently:
  `nest 4` around each equation (signature.rs 200–204) and, for the overlong
  case, `nest(-2)` on the `= rhs` element (equation_doc, 209–214) — i.e.
  `+4 / -2` where upstream uses `+2(block) / +2(lhs)`, "=rhs" landing at
  column 2 both ways. Same bytes (lhs@4, `= rhs`@2, probe:e_long), materially
  different Doc structure. A transcription would have reproduced upstream's
  `nest 2`/`nest 2` split; the `nest 4`/`nest -2` choice is the signature of
  observation ("= rhs at eq-indent minus 2") rather than source reading.
* **Builtin expansion tables** — see ledger §1; observable, faithful, own
  representation. `merged_function_items`/`merged_equations` sort+dedup on
  rendered text is oracle-derived (f_sort, e_adedup) and mirrors upstream's
  `S.toList` set ordering only in observable outcome.

### 5. AST + tests

* `src/ast.rs` is the clean crate's own minimal model, mirroring the
  integrator-supplied interop header `interface/ast_types.rs` (variant names
  are semantic — `PubLit`, `App`, `Pair`, `BinOp` — and do NOT mirror the
  upstream term algebra's `Lit`/`FApp`/`FunSym`/`NoEq`/`AC` constructor
  constellation). The interop header's provenance is the integrator's, upstream
  of this round.
* `tests/round1_term_signature.rs`: every expected string is annotated with its
  `probe:`/`target:` oracle origin; the `parity_*` fixtures rebuild each of the
  10 round-1 signatures from readable source declarations and assert against
  the capture files, with `parity_blocks_match_capture_files` re-extracting from
  the captures to guard the literals. 41/41 pass. No lifted upstream text.
* Identifier-leakage scan of `src/` for `ppTerm|ppFun|ppACOp|ppTerms|NoEqSym|
  CtxtStRule|StRhs|prettyMaudeSig|sortPrefix|ppNonEmptyList|ppFunSymb` and the
  pseudonymous author handles: only the legitimate BSD-attribution lines in
  doc.rs/lib.rs match. Clean.

### Findings

No violations. No protectable upstream expression copied; Doc engine provenance
is the sanctioned BSD source (not the GPL `Text.PrettyPrint.Class`/`Highlight`
wrapper); byte-forced surface is properly filtered and its provenance logged.
The four constructs where the clean crate could have transcribed upstream
(AC/pair bracketing, exp flattening, equation nest split, ppFun) each either
diverge structurally from upstream or reduce to boundary-forced merger, with
the divergences (outside-bracketing, spine-collection, +4/-2 nest) being
positive evidence of genuine black-box derivation.

Non-blocking notes (advisory, NOT redo instructions — do not gate this round):
1. *License hardening for distribution.* The BSD LICENSE lives only under
   `graphdot/sanctioned/`. When `pretty-clean` is eventually distributed
   standalone (post-GPL-erasure), carry a copy of the BSD `pretty-1.1.3.6`
   LICENSE + copyright notice alongside the crate to satisfy BSD clause-2 in
   that redistribution context. In-repo today the notice IS preserved and
   attributed — sufficient for the current audit.
2. *Forward integration caveat (R2+, not R1).* The public API returns
   fully-rendered `String`s per sub-target, each rendered standalone at
   column 0. This coincides with the embedded layout for R1 (signature at
   column 0; terms are leaves), so R1 parity is exact. For R2/R3, where rules
   and formulas sit at a nonzero theory nest and BEHAVIOR.md's own
   ribbon-within-nesting observation (probe:t_edge) applies, the whole-theory
   assembly must compose the `pub(crate) doc()`/`block_doc()` Docs and render
   ONCE — not concatenate independently rendered column-0 strings. The
   architecture already exposes the Doc-level helpers to support this.

VERDICT: pass

---

## Round 2 — equation ORDERING, tuple-wrap correction, R2 rule rendering

Scope = the working-tree delta over cleanroom HEAD restricted to `pretty/`:
`src/{ast,signature,term,rule,macros,lib}.rs`, the new `tests/round2_rules.rs`
and the expanded `tests/round1_term_signature.rs`, plus the provenance
additions (`workspace/{BEHAVIOR.md,QUERIES.log}`, `scratchpad/probes/p_eq*`,
`p_pw1`, `p_rattr`, `p_fann`, `p_var1`, `p_lbvar`, `p_lb2`, `p_mac1`, `p_arr1`,
`c_{loop,attrs,annot,restrict,ch07}`, the `contract`/`mesh` scratch captures)
and `round2/` (12 curated corpus captures, 72 rule blocks). The BSD doc engine
(`src/doc.rs`, `Cargo.toml`) is **untouched** this round — verified by diff;
round 1's provenance finding stands. Method as above (AFC; byte-forced output
filtered as merger, residue examined for protectable expression).

Sealed side, additional to round 1: rule layout —
`lib/theory/src/Theory/Model/Rule.hs` `prettyNamedRule` (1284–1292),
`prettyRuleRestrGen` (1231–1245), `prettyRuleAttribute` (1201–1215),
`prettyLoopBreakers` (1305–1312), `prettyProtoRuleACInfo`/`ppVariants`
(1294–1300); fact — `Theory/Model/Fact.hs` `prettyFact`/`nestShort'`
(539–547); substitutions — `Term/Substitution/SubstVFresh.hs`
`prettyDisjLNSubstsVFresh` (223–229) + `Text/PrettyPrint/Class.hs`
`numbered`/`numbered'` (252–264); macros — `TheoryObject.hs`
`prettyMacros`/`prettyMacro` (819–836); comment framing —
`Theory/Text/Pretty.hs` `multiComment` (102–103). **Equation order source:**
`Term/Maude/Signature.hs` `prettyMaudeSigExcept` (226) emits
`S.toList (stRules sig)`, i.e. the *derived* `Ord CtxtStRule`
(`SubtermRule.hs` 42) — `CtxtStRule LNTerm StRhs`, `StRhs [Position] LNTerm`,
both `deriving Ord`; term order is `deriving Ord` on `Term a = LIT a | FAPP …`
(`Term/Term/Raw.hs` 72–74) with the *custom* `Ord LVar`
(`LTerm.hs` 522–524: `compare x3 y3 <> compare x2 y2 <> compare x1 y1` over
fields `name,sort,idx`).

### 1. Equation ORDERING law — derivation provenance (special scrutiny)

The round-1 law ("byte order of the rendered equation text") was **falsified**
this round and replaced by a structural (lhs, rhs) term order
(`signature.rs` `equation_cmp`/`term_cmp`/`view_cmp`/`app_view`). Because the
echoed order is observable, the *law* matching upstream is expected merger; the
audit question is whether the law was **read off `S.toList`/the `Ord`
instances** or **derived black-box**. It is the latter, on four independent
grounds:

* **Corpus counter-examples drove it, and they are logged as such.** The
  scratchpad captures `contract.echo` (`checkpcs(xc, xpk, …)` **before**
  `checkpcs(xc, pk(xsk), …)` though 'x'>'p') and `mesh.echo` (`cnf` before
  `aes_cmac(…)` though 'c'>'a') are recorded (QUERIES.log) as *refutations* of
  the round-1 byte-order guess. Materialised as round-1 byte targets
  (families.tsv +2; `round1/targets/…contract….txt`, `…mesh….txt`) and
  asserted green (`contract_fixture`/`mesh_fixture` rebuild the signatures from
  readable declarations and match the captured opposite-of-byte order). This is
  the fingerprint of a wrong hypothesis caught by the oracle, not of source
  reading.
* **The isolating probes are genuine discriminators.** `p_eqA…p_eqI` each pin
  ONE comparison decision, most with a declaration-order-swap control (A/B,
  C/C2, F/F2 echo identically) so the law cannot be "source order": var-below-
  app (C/C2: `f(zzz,…)`<`f(a0,…)`, a0 nullary), name-bytes-not-shortlex
  (D: `azz`<`b`), name-first-not-arity (G: `pair`/2 < `z1`/1), rhs-breaks-ties
  (F/F2), and — decisively — `p_eqI` (`<x,zz>` < `<x,b,c>`) which distinguishes
  a **right-nested binary** tuple comparison from a flattened elementwise one
  (the flattened view reverses them). Building a theory whose output separates
  those two models is black-box work; a source reader would simply see
  `FAPP pairSym [a,b]` and need no such probe. All present and re-run green.
* **The law DIVERGES from the source `Ord` in the non-observable places** — the
  tell that settles provenance:
  - upstream `Ord LVar` compares **idx, then sort, then name**; the clean
    compares **name, then idx**, with **no sort field** in the key. A
    transcriber copies `x3<>x2<>x1`; the probes (all idx-0 for the name test,
    same-name for the idx test) yield exactly the clean's name-first key and
    nothing about sort.
  - upstream breaks lhs ties on `StRhs`'s **`[Position]` list first**, only
    then the rhs term; the clean breaks straight to the **rhs** and models no
    position list. `p_eqF` cannot see the difference (positions and rhs agree
    there), so the clean pinned the visible surrogate.
  - the clean's `CmpView::PairTail` re-binarises its own **n-ary** `Pair(Vec)`
    node on the fly to reproduce upstream's binary-`pair` order. Upstream stores
    pairs binary and needs no such device; this helper exists *only* because the
    clean AST diverges — it cannot have been transcribed.
  These are latent correctness deltas the corpus gate must backstop (an
  equations block that puts differently-named/differently-indexed variables at
  one comparison position, or same-lhs rules whose rhs order contradicts their
  subterm-position order, would expose them). They are **not** similarity
  findings; they are affirmative proof of non-transcription, and BEHAVIOR.md
  already flags the unobservable ranks (literals, exp/AC, diff, patmatch) as
  guesses to be caught by the gate rather than asserting them from source.

**Derivation = probe/corpus provenance, confirmed.** No copied ordering
expression: upstream's term order is compiler-derived (nothing hand-written to
lift), the one hand-written `Ord LVar` is *not* what the clean implements, and
`term_cmp`/`view_cmp`/`app_view`/`CmpView` are the clean's own construction.

### 2. Tuple-wrap correction (term.rs `pair_doc`) — byte-forced merger

Round 1's pair used outside-brackets (`'<' <> fcat(elems) <> '>'`, no per-elem
nest) and round 1 filed it as a *divergence*. Round 2 rewrites it to
`fcat('<' : map (nest 1) (punctuate ", " elems) ++ ['>'])` — which is
structurally upstream's `ppTerms ", " 1 "<" ">"` (`Term.hs` 291/298). This
convergence is merger, not transcription:

* It is **byte-forced** by the four `p_pw1` shapes (wfa–wfd, byte-checked): the
  "`<` **alone** on its line when the first element cannot sit beside it" shape
  forces `<` to be a *separate breakable fcat item* (round 1's glued `<` could
  never produce it — the recorded reason the R1 law was wrong); the continuation
  column `col('<')+1` forces `nest 1`; `>` landing at `col('<')` when the last
  element is multi-line forces `>` as an *unnested* fcat item; the trailing
  space on a wrapped line forces the attached-`", "` punctuate + `fcat`. Given
  those observations there is essentially no other Doc expression.
* It was **reached by iteration from a falsified guess** (R1 form → mesh k2 /
  p_pw1 failures → correction), the signature of black-box search, and
  BEHAVIOR.md records the falsification.
* **AC operators were NOT unified onto the same form** — `ac_doc` keeps the R1
  outside-bracket construction (term.rs unchanged there; the AC wide-wrap probe
  still times out, so it stays honestly unpinned). Upstream reuses the *single*
  `ppTerms` helper for pairs AND AC; a transcriber converges both, the clean
  converged only where the bytes compelled it. Divergent helper reach confirms
  independence.

### 3. R2 rule rendering — AFC over each construct

* **Body nest — divergent split, same columns.** Upstream
  `nest 2 (sep [nest 1 prems, arrow, nest 1 concls])` (prems/concls@3, arrow@2).
  Clean `nest 3 (sep [prems, nest(-1) arrow, concls])` — the identical **+3 /
  −1** vs upstream **+2 / +1** re-encoding seen at R1's equation block
  (+4/−2 vs +2/+2). Observing two columns (3 and 2) and encoding them with the
  minimal outer-nest/arrow-outdent split, rather than upstream's block/inner
  split, is the consistent non-transcription fingerprint. `sep` (true
  all-or-nothing, cav13 keeps `-->` on its own row though it would fit)
  is observable.
* **Bracket group — divergent combinator.** Upstream
  `ppFactsList = fsep ["[", fsep facts, "]"]` (one fill, brackets as items) and
  the arrow as a *separate* `fsep ["--[", …, "]->"]`. Clean unifies both under
  one `bracket_group = sep [sep [open, fsep facts], close]` — a **nested
  all-or-nothing sep**, not a fill, and a shared helper upstream does not have.
  Its graded three-way layout is pinned to `p_arr1` + mesh
  DeviceWaitingUser/ProvisionerWaitingUser at col 73, NSLPK3 R_1, Tutorial
  Serv_1. Own construction, own decomposition.
* **Fact — divergent construction, same bytes.** Upstream
  `nestShort' (n++"(") ")" . fsep . punctuate comma` = `sep [lead $$ nest
  (len+1) (fsep args), ")"]`. Clean `sep [head <+> fsep args, ")"]` — a plain
  `<+>` beside instead of the `$$`/`nest (len+1)` overlap; both put args at
  `col(len+1)` and drop `)` alone. `!` prefix, `Name( )` nullary, args-fill:
  observable (target:mesh, probe:t_wide). The clean did **not** use
  `nestShort'`.
* **Fact annotations — observable canonical order.** `[+, -, no_precomp]`
  regardless of source order (probe:p_fann `[no_precomp,+]`→`[+, no_precomp]`;
  target:seqdfsneeded). Upstream renders via `S.toList` on `Set
  FactAnnotation`; the clean sorts/dedups its own enum. The order is in the echo
  → merger; the enum spellings are byte-forced.
* **Header / attributes — observable.** Canonical color<no_derivcheck<
  issapicrule<role, last-color/role-wins, `process=`/external dropped,
  `#hex` lowercased, `role='…'` quoted, `color=#…` unquoted — every token
  observable (probe:p_rattr, incl. the logged first-run FAILURE on a non-RGB
  color; target:issue713). `attr_items` is the clean's own match/accumulate,
  bearing no resemblance to upstream's `catMaybes [fmap … ]`.
* **Loop breakers — observable spelling.** singular/plural noun, `[i,j]`
  no-space, col-2 on the E side vs col-4 *inside* the comment after the variant
  list on the AC side (probes c_loop, p_lb2, p_lbvar). Upstream's twin
  `prettyLoopBreakers`/`prettyInstLoopBreakers` are collapsed into the clean's
  one `breaker_doc` — the clean's decomposition, not upstream's.
* **Variants comment.** `/* */` framing: clean stacks explicitly
  (`above_op` chain) where upstream uses `multiComment = comment $ fsep
  ["/*", d, "*/"]`; divergent construction, same bytes on the always-tall rule
  body. Numbered groups: right-align to `len(str(count))`, `". "`, `vcat` of
  equations beside the prefix, bare-indent separator line between groups — the
  layout is observable (cav13 1-digit / CH07 2-digit / Joux 3-digit, byte-
  checked separator p_var1) and the clean **inlines** it in `substitutions_doc`
  rather than replicating upstream's `numbered'`→`numbered`→`ppConj`→`flushRight`
  chain (no such helpers appear).
* **Substitution `lhs $$ nest 6 ("= " <> rhs)` — byte-forced merger.** Equal to
  upstream `prettyEq` (`prettyNTerm (Var a) $$ nest 6 ("=" <-> rhs)`). The
  observed two-shape HughesPJ *overlap* — a short lhs padded to col 6 on the
  same line (`~lv2  = …`), a ≥6-col lhs pushing `= rhs` to its own line at
  col 6 — uniquely identifies the `$$`+`nest 6` construction, and `p_var1`
  built `~longvariablenameone` precisely to observe the second shape. `6` is the
  observed column of `= rhs`, not a lifted constant. Merger, same category as
  R1's `ppFun`.
* **`macros:` block.** Divergent tokenisation: clean
  `hsep [name( <+> params <+> ), "= ", body]` vs upstream
  `(op++"(") <+> prettyVarList <+> ") = " <+> out` (the `)` split off the head,
  `= ` a separate item, `fsep` not `prettyVarList`, `<+>` head not
  `sep(map(nest 4))`). All-or-nothing item list, `) =  body` two-space, body-
  always-beside pinned to probe:p_mac1 / target:issue777.
* **Doc engine dependency.** The clean reaches for `above_plus` = BSD `$+$`
  **directly**; upstream's `prettyNamedRule` uses the GPL-wrapper alias `$-$`
  (`Class.hs` 36: `$-$ = P.$+$`). The clean depends only on the sanctioned BSD
  primitive, never the GPL wrapper — reinforcing R1 §2. `$--$` (wrapper-
  original) is absent.

### 4. Identifier / decomposition / test scan

* Leakage grep of `src/` for the R2 upstream surface (`prettyProtoRule`,
  `prettyNamedRule`, `prettyRuleRestr`, `ppFactsList`, `nestShort`, `numbered'`,
  `ppConj`, `prettyEq`, `prettyDisjLNSubst`, `prettyFact`, `ppFact`, `ppAnn`,
  `multiComment`, `flushRight`, `ppVariants`, `prettyLoopBreaker`, `CtxtStRule`,
  `StRhs`, `RuleAttributes`, `rgbToHex`, `showFactAnnotation`, `ppTerms`) and
  author handles: only the English word "numbered" in doc comments. Clean.
* `ast.rs` `RuleAttr`/`FactAnnotation`/`AcVariants{ac_rule,substitutions}` are
  the clean's semantic model; they do not mirror upstream's `RuleAttributes`
  record / `FactAnnotation` / `ProtoRuleACInfo`+`Disj LNSubstVFresh`
  constellation.
* `tests/round2_rules.rs`: probe-pinned unit fixtures (each annotated with its
  `probe:`/`target:` origin) plus a whole-block corpus parity harness whose
  echo-parser **discards inter-token whitespace** (so layout can only originate
  in the renderer) and **re-derives and asserts** the group numbering rather
  than trusting it. 9/9 green; 72 blocks (`checked > 50`). Round-1 suite
  extended to 47/47 incl. the p_eq order tests and contract/mesh parity. Both
  ran green here.

### Findings

No violations. The two constructs whose Doc expression now equals upstream
(tuple-wrap `ppTerms` form; substitution `$$ nest 6`) are byte-forced merger,
each pinned to a discriminating probe and — for the tuple — reached by
correcting a logged wrong guess while the parallel AC path stayed divergent. The
equation-ORDERING law's derivation is probe/corpus provenance (discriminating
p_eq probes + logged contract/mesh refutations of the R1 byte-order law), and it
*diverges from the source `Ord`* in every non-observable slot (LVar
idx-sort-name vs the clean's name-idx; the `StRhs [Position]` tie-break omitted;
n-ary `CmpView::PairTail` re-binarisation with no upstream analogue) — affirmative
non-transcription evidence, not merely absence of copying. Rule body/bracket/
fact/attribute/loop-breaker/comment/macro constructions each diverge structurally
(nest split, nested-sep vs fill, `<+>` vs `nestShort'`, own accumulators, own
helper decomposition) while reaching the observable bytes. Identifier and
decomposition scans clean.

Non-blocking notes (advisory, do NOT gate this round):
1. *Latent order deltas vs source, gate-backstopped.* The LVar key
   (name-then-idx, sort dropped) and the omitted `StRhs [Position]` lhs-tie-break
   can disagree with upstream on unprobed equations blocks (differently-named &
   differently-indexed variables at one comparison slot; same-lhs convergent
   rules whose rhs order contradicts subterm-position order). Correct for the
   probed + curated corpus; the full-corpus signature gate is the required
   backstop before wider parity is claimed. This is a correctness caveat, not a
   similarity issue (indeed it is the non-transcription evidence).
2. *R1 forward-integration caveat still open.* The public API still renders each
   sub-target standalone at column 0; R2 rule blocks sit at a nonzero theory nest
   in the embedded echo. Whole-theory assembly must compose the `Doc`s and render
   once (the crate exposes the Doc helpers). Carried from R1 §note 2.
3. *R2 scope boundaries honestly recorded* (BEHAVIOR.md "Out of R2 scope"):
   embedded `_restrict` rule surfaces, diff-mode left/right rule variants, and
   the theory-level looping-facts comment are named as unpinned and deferred, not
   guessed.

VERDICT: pass

---

## Round 3 — formula rendering + lemma / restriction wrappers

Scope = the working-tree delta over cleanroom HEAD `c29e244` (rounds 1–2)
restricted to `pretty/`: `src/{ast,formula,lemma,lib}.rs` filled in,
`src/{rule,term}.rs` touched (`fact_doc`/`var_str` widened to `pub(crate)`;
`ac_doc` corrected — below), the new `tests/round3_formulas.rs`, the
provenance additions (`workspace/{BEHAVIOR.md,QUERIES.log}`, the `q_*`
probes under `workspace/scratchpad/probes/`, the corpus `*.echo`/`*.time`
scratch captures) and `round3/` (20 curated corpus captures — 84 restriction
+ 139 lemma blocks, `families.tsv`, `fetch_hs_targets.sh`). The BSD doc engine
(`src/doc.rs`, `Cargo.toml`) is **untouched** this round — verified by diff;
R1 §2 provenance stands, and every R3 combinator used (`above_op`,
`above_plus`, `beside_op`, `beside_space`, `sep`, `hsep`, `fsep`, `vcat`,
`nest`, `punctuate`, `render_with`) is a pre-existing BSD `HughesPJ` export.
Method as before (AFC; byte-forced output filtered as merger, residue examined
for protectable expression). Full suite green here: R1 47, R2 9, R3 9, lib 6.

Sealed side (upstream Haskell, `tamarin-rs/tamarin-prover/`):
* formula printer — `Theory/Model/Formula.hs` `prettyLFormula` (471–511),
  `prettyLNFormula`/`prettySyntacticLNFormula` (515–522);
* atoms — `Theory/Model/Atom.hs` `prettyProtoAtom` (212–224), `prettyNAtom`
  (232), `prettySyntacticNAtom`/`prettyPred` (236–239);
* operators — `Text/PrettyPrint/Highlight.hs` `operator_` (56), **`opParens`
  (58–59)**; glyph table `Theory/Text/Pretty.hs` (165–183);
* comment framing — `Pretty.hs` `multiComment`/`multiComment_` (102–106),
  `lineComment` (97);
* lemma wrapper — `Lemma.hs` `prettyLemma`/`ppLNFormulaGuarded` (116–141),
  `prettyLemmaName`/`prettyLemmaAttribute` (91–107), `prettyTraceQuantifier`
  (178–181);
* restriction wrapper — `TheoryObject.hs` `prettyRestriction` (846–858);
* safety — `Theory/Constraint/System/Guarded.hs` `isSafetyFormula`/
  `noExistential` (156–164); guarded printer `prettyGuarded` (824–867) —
  **not ported** (its output is opaque input here).

### 1. Parenthesization / precedence DERIVATION (special scrutiny)

The clean fully parenthesizes: every operand of `∧ ∨ ⇒ ⇔` and every `¬`
argument is wrapped in `(…)`, quantifier bodies and the top level bare
(`formula.rs` `doc`/`connective_doc`/`parens`). The audit question is whether
this was **read off a source precedence table** or **derived black-box**. Two
findings settle it:

* **There is no precedence table upstream to copy.** `opParens`
  (Highlight.hs 59) is `operator_ "(" <> d <> operator_ ")"` — an
  *unconditional* wrap, applied to *both* operands in `pp (Conn op p q) =
  sep [opParens p' <-> ppOp op, opParens q']` (Formula.hs 490–498) and to the
  `¬` argument in `pp (Not p) = operator_ "¬" <> opParens p'` (485–488). No
  operator-priority comparison exists anywhere in the formula printer; a
  textbook precedence-aware printer would *omit* the redundant parens the
  oracle in fact emits. Matching the unusual always-parens behavior is
  therefore observational, not a lift.
* **The derivation traces to a genuine discriminator, `q_p2` (s1–s13,
  captured).** It separates full-paren-plus-association from any n-ary
  flattening: `s3` `(((x='b')∨(x='c'))∨(x='d'))` vs `s4`
  `((x='b')∨((x='c')∨(x='d')))` echo **distinctly** — left/right chains keep
  their source association, no re-association, no flattening; `s1` wraps every
  nested-connective operand; `s8` `¬((x='b')∧(¬(¬(x='c'))))` pins the
  ¬-arg-always-wrap and double-negation; `s9–s12` wrap quantifiers-under-
  connectives while `s13`/bodies stay bare. A source reader who had seen
  `opParens p' … opParens q'` would need none of these; building theories whose
  echo separates "preserve association + full paren" from "flatten n-ary" is
  black-box work. Decisively, the guarded-formula block emitted in the *same*
  `q_p2.out` uses a **different** paren discipline — flat n-ary
  `… ∧ (¬(x='b')) ∧ (¬(x='c')) ∧ (¬(x='d'))`, bare `¬(x=y)` — which the clean
  does **not** reproduce (it consumes that block verbatim as opaque input,
  §3), positive evidence the clean pinned the statement printer's rule
  specifically rather than echoing one paren rule everywhere.

`connective_doc` = `sep [beside_space(parens(doc l), glyph), parens(doc r)]`
equals upstream's `sep [opParens p' <-> ppOp op, opParens q']` in Doc *shape*.
That convergence is **byte-forced merger**: the observable output (full parens
both operands; glyph on the lhs' last line; rhs dropping to the group origin on
overflow — targets NSLPK3/Cronto/Yubikey) uniquely determines this minimal Doc,
the same category as R1 `ppFun` and R2 substitution `$$ nest 6`. The clean
reaches BSD `sep`/`beside_space` and its own `parens` helper — never the GPL
`operator_`/`opParens`/`<->` wrappers (`<->` = `<+>`, Class.hs). Pinned q_p2;
overflow layout corpus-witnessed.

### 2. Binder-allocation approach — DIVERGENT (independent)

Upstream renders a quantifier prefix by **inventing** its variable names:
`pp fm@(Qua …) = scopeFreshness $ do (vs,qua,fm') <- openFormulaPrefix fm; …
sep [ppQuant qua <> ppVars vs <> operator_ ".", nest 1 d']`, with
`ppVars = fsep . map (text . show)`, over the de-Bruijn `ProtoLFormula`, using
`MonadFresh` + `avoidPrecise` to allocate collision-free names and collapse a
run of same-quantifiers into one binder list (Formula.hs 500–511, 515–522).

The clean has **no such machinery**: `Formula::Forall(Vec<VarSpec>, body)`
carries already-named binders (integrator's parser), and `quantifier_doc`
(`formula.rs`) just `fsep`s them through the R1 `term::var_str` and stacks
`sep [head, nest 1 body]`. No de-Bruijn, no `MonadFresh`, no
`scopeFreshness`/`avoidPrecise`, no fresh allocation — a fundamentally
different binder model. Only the **output** coincides: `fsep binders` matches
`fsep . map (text . show)` (byte-forced; q_b1 pins sigils/source-order/
no-sort-erasure), and the head/body layout (`∀ ` glued, binders fill-wrapping
at origin+2, `.` on the last binder, body at `nest 1` = origin+1) is pinned by
`q_l2` (bw1 long-binder wrap; bw2 paren-nested body). Divergent approach,
observable bytes. Clean.

### 3. Helper decomposition — AFC over each construct

* **Relation atoms — DIVERGENT unification.** Upstream *splits*: `EqE` and
  `Subterm` are `sep [ppT l <-> op, ppT r]` (Atom.hs 219–222) but `Less` is
  `text (show u) <-> opLess <-> text (show v)` (223) — **no `sep`** (no
  break point) and **raw `show`**, bypassing the term printer. The clean folds
  all three into one `relation_doc` = `sep [beside_space(term::doc l, glyph),
  term::doc r]` routing every side through the R1 term core. The `=`/`⊏` arms
  are byte-forced merger (q_l4); the `<` arm **diverges** (the clean gives it a
  breakable `sep` and the term renderer where upstream gives it neither) — a
  non-observable delta (temporal/nat `<` operands are short, never break, never
  differ), i.e. the clean's own decomposition, not a transcription.
* **Action atom — byte-forced merger, own routing.** `hsep [fact_doc, '@',
  term::doc tp]` = upstream `prettyFact ppT fa <-> opAction <-> text (show v)`
  (Atom.hs 216–217) in bytes (both are non-breaking `<+>` folds), pinned to
  `q_l5` (m63/m64: at overflow the *fact* breaks internally to `) @ #i`, `@ tp`
  never dropping — discriminating `hsep` over a `sep`). The clean reuses its R2
  `fact_doc` unchanged (probe:q_l3) and its R1 `term::doc` for the timepoint,
  where upstream uses `prettyFact`/`text (show v)`.
* **`last` / `¬` / `⊤⊥`** — `last(` glued vs upstream `operator_ "last" <>
  parens (…)`; `¬ <> (arg)` vs `operator_ "¬" <> opParens p'`; `⊤`/`⊥` vs
  `operator_ "⊤"/"⊥"`. Same bytes, glyphs byte-forced (q_at1, q_p2 s8, q_w1);
  the clean's `\u{…}` escapes and glued tokens are its own representation.
* **Safety classification — DIVERGENT algorithm (the standout).** Upstream
  computes it via the **whole guarded transform**: `safety = isSafetyFormula
  (formulaToGuarded_ expandedFormula)` where `isSafetyFormula gf = null
  (frees [gf]) && noExistential gf` and `noExistential` walks the *already-
  NNF'd guarded structure* (`GAto`/`GGuarded Ex/All`/`GDisj`/`GConj`) with **no
  polarity tracking** — the transform did that (Guarded.hs 156–164;
  TheoryObject.hs 851/858). The clean does **none** of this: `is_safety` =
  `!has_existential_in_nnf(f, false)`, a direct polarity-tracking scan of the
  *source* `Formula` (a `negated` flag flipped through `Not`, the `Implies`
  antecedent, both `Iff` sides, `Forall`-under-negation, `Exists`-in-positive-
  polarity). It reimplements the transform's NNF-existential detection inline,
  without ever building a guarded form. Derived black-box from `q_s1` (s5 msg-∃
  **and** s7 temporal-∃ conclusions both defeat safety; s8 `¬∃` conclusion
  keeps it) and `q_s2` (u4 `¬∃` in an *antecedent* defeats it — pins the
  implication-polarity flip). No upstream analogue exists to transcribe; this
  is affirmative non-transcription evidence.
* **Comment / block framing — DIVERGENT construction.** The clean **stacks**
  `/* … */` with explicit `above_op` chains (`guarded_comment_doc`,
  `restriction_doc`) where upstream uses `multiComment d = comment $ fsep
  ["/*", d, "*/"]` (a *fill*, Pretty.hs 102–103) — the identical divergence
  filed at R2 for the variants comment. Between major blocks the clean uses
  `above_op` (`$$`) / `above_plus` (`$+$`) where upstream uses `$-$` (the GPL
  alias = `$+$`, Class.hs), again depending only on BSD primitives. The lemma
  header splits `"lemma NAME ["` + `fsep(punctuate ',' attrs)` + `"]:"` onto
  text tokens where upstream composes `kwLemma <-> (text name <-> brackets $
  fsep $ punctuate comma …) <> colon` — the same `[`/`]:`-split-off-the-head
  fingerprint as R2's macro `)`/`= ` split. All same-bytes, all pinned
  (q_w1, q_r1, q_la1, targets 5G_AKA).

### 4. Byte-forced surface, filtered (merger; provenance logged)

Observable in every echo, hence compatibility content: the glyph set
(`⊤ ⊥ ∧ ∨ ⇒ ⇔ ¬ ∀ ∃ ⊏ @ last`), the wrapper string literals (`restriction `,
`lemma `, `all-traces`/`exists-trace`, `// safety formula`, `expanded
formula:`, `guarded formula characterizing all counter-examples:` /
`… all satisfying traces:`, `conversion to guarded formula failed:`,
`by sorry`), the attribute spellings (`sources`, `reuse`, `use_induction`,
`hide_lemma=<n>`, `heuristic=<v>` verbatim incl. braces), the always-emitted
expanded-formula comment (84 restrictions ↔ 84 comments in the corpus,
verified — the oracle sets `ogFormula` on every load, so the clean's
unconditional emit is faithful), the conditional safety line (68/84), the
statement `nest 2` / body `nest 1` / error `+2` indents, and the trace-
quantifier keyword mapping. Provenance logged (q_w1, q_ax1, q_pred1, q_la1–3,
q_l2/l4/l5, q_r1, q_s1/s2). The clean's `\u{…}` escapes, glued tokens, and
`match`-to-`String` attribute table are its own representation; upstream's
ghost ASCII comments (`-- "T"`, `-- "&"`, `-- "==>"`, …) are **not** reproduced
(verified).

### 5. `ac_doc` correction (R1 source-file change this round)

`term.rs` `ac_doc` was rewritten from R1's outside-bracket form
(`'(' <> fcat(punctuate op elems) <> ')'`, `)` beside-attached) to the
single-fill `fcat('(' : map (nest 1) (punctuate op elems) ++ [')'])` — i.e.
now structurally upstream's `ppTerms` (Term.hs), the **same** convergence R2
already made for `pair_doc`, and for the same reason: **byte-forced** by a
newly-materialised witness (round-3 target `alethea Universal_VerProofV_v1`:
the union keeps both wide elements on one fill line and only `)))"` drops to
the `(` column — a shape the R1 `)`-attached law cannot produce). R2 had left
AC divergent *only* because its wide-wrap probe timed out and honestly flagged
it; the corpus witness has now closed that gap. Reached by falsifying a logged
wrong guess (BEHAVIOR.md records the R1 law "agreed on every R1-observed shape
but was falsified by the alethea witness"), the fingerprint of black-box
search. Merger, not transcription; same category as R2 §2. R1/R2 suites stay
green under the change.

### 6. Identifier / constant / comment / test scan

* Leakage grep of `src/` for the R3 upstream surface (`prettyLFormula`,
  `prettyLNFormula`, `prettyProtoAtom`, `prettyNAtom`, `opParens`,
  `operator_`, `opLAnd`/`opImp`/`opForall`/…, `ppQuant`, `ppVars`,
  `openFormulaPrefix`, `scopeFreshness`, `avoidPrecise`, `prettyGuarded`,
  `noExistential`, `isSafetyFormula`, `formulaToGuarded`, `prettyLemmaName`,
  `prettyLemmaAttribute`, `prettyRestriction`, `multiComment`, `GGuarded`/
  `GDisj`/`GConj`, `EqE`, `ProtoAtom`, `SourceLemma`/`InvariantLemma`/…) and
  the pseudonymous author handles: **no matches**.
* AST enums DIVERGE from upstream's constructor constellations. `Atom
  {Eq,Less,LessMset,Subterm,Action,Last,Pred}` vs upstream `ProtoAtom
  {Action,EqE,Subterm,Less,Last,Syntactic (Pred)}` — the clean **adds**
  `LessMset`, renames `EqE→Eq` and `Syntactic (Pred)→Pred`. `LemmaAttr
  {Sources,Reuse,UseInduction,HideLemma,Heuristic}` is a *witnessed subset* of
  upstream's nine-constructor `LemmaAttribute` with the clean's own names
  (`Sources`≠`SourceLemma`, `UseInduction`≠`InvariantLemma`). `Guarded
  {Formula,Failed}` and `TraceQuantifier {AllTraces,ExistsTrace}` are the
  clean's own two-value carriers (the latter's names are the observable
  keyword tokens, byte-forced, not a protectable constellation).
* Non-observable constants (`nest 1` body, `nest 2` statement/comment, `+2`
  error indent) are each probe-pinned (q_l2, q_w1/corpus, q_r1) — none lifted
  as a magic number (cf. R2 substitution `nest 6`).
* Comment lineage: the only "upstream" tokens in `src/` are pipeline-semantic
  ("expanded *upstream of the echo*", "lowercased upstream") — no source-file
  narration, no line-number citations, no reproduced ghost comments.
* `tests/round3_formulas.rs`: probe-pinned unit fixtures (each annotated with
  its `probe:`/`target:` origin) plus a whole-block corpus parity harness whose
  echo-parser **discards all inter-token whitespace** (so layout can only
  originate in the renderer) and carries guarded-comment content / embedded
  proofs as **verbatim opaque inputs** (correctly isolating the frame from the
  un-ported guarded transform). 9/9 green; the parity test byte-checks 84
  restriction + 139 lemma blocks (`checked_r > 40 && checked_l > 100`).

### Findings

No violations. The parenthesization is the round's convergence-risk point, and
it resolves cleanly: upstream has **no precedence table** (unconditional
`opParens` on both operands), the clean matched that observable always-parens
behavior via a genuine discriminator (`q_p2` s3/s4 left/right-chain separation)
rather than a lift, and the guarded block in the same capture — which uses a
different paren discipline — is consumed verbatim, not reproduced. The
`connective_doc`/`=`/`⊏`/action Doc shapes that equal upstream are byte-forced
merger (each pinned to a discriminating probe), while the binder-allocation
approach (named-AST vs de-Bruijn+`MonadFresh`+`avoidPrecise`), the safety
classifier (direct polarity-NNF scan vs guarded-transform+`noExistential`), the
relation unification (`<` given break+term-printer where upstream gives
neither), and the comment/header framing (`above_op` stacking + tokenised
`[`/`]:` vs `multiComment` fill + `brackets`) each diverge structurally while
reaching the observed bytes. The `ac_doc` correction is the same byte-forced
`ppTerms` convergence as R2's tuple, reached by falsifying a logged guess.
Identifier, enum-constellation, constant, comment and test scans clean.

Non-blocking notes (advisory, do NOT gate this round):
1. *Latent safety-classifier deltas vs source, gate-backstopped.* The clean's
   `is_safety` omits upstream's `null (frees …)` closedness conjunct (harmless —
   restriction/lemma formulas are always closed) and reads a safety-relevant
   `⇔` as occurring in both polarities (the NNF-consistent guess, honestly
   flagged UNOBSERVABLE — no `⇔` restriction witness in the corpus). Correct
   for the probed + curated set; the full-corpus signature gate is the backstop
   before wider parity is claimed. A correctness caveat, not a similarity issue
   (indeed the divergent algorithm is the non-transcription evidence).
2. *Unobservable atoms are flagged placeholders, not guessed-as-pinned.*
   `Atom::LessMset` (no corpus/source witness — rendered like `Less`) and
   `Atom::Pred` (predicates expanded upstream of the echo — rendered as a bare
   fact) are both registered UNOBSERVABLE and must be pinned before any claim
   over them; correct discipline.
3. *Binder-prefix collapse is the AST-producer's contract, not the renderer's.*
   Upstream `openFormulaPrefix` collapses a run of same-quantifiers into one
   binder list; the clean renders whatever `Forall(Vec<VarSpec>, …)` it is
   handed. The integrator's parser must deliver pre-collapsed prefixes (the
   corpus captures already are); a nested-same-quantifier AST would render
   un-collapsed. Interface caveat, not similarity.
4. *R1/R2 forward-integration caveat still open* (carried from R1 §note 2 /
   R2 §note 2): the public API renders each sub-target standalone at column 0;
   whole-theory assembly must compose the crate's `Doc`s and render once so
   embedded formula/rule blocks nest correctly. The Doc-level helpers remain
   exposed for this.
5. *Diff-mode surfaces deferred, not guessed* (BEHAVIOR.md UNOBSERVABLE
   register): `--diff` lemma/restriction `left`/`right`/`diff_reuse`,
   `diffLemma`, `output=…`, and `lemma (modulo E) …` are named as unpinned and
   unmodeled (the parity corpus excludes all `--diff` files), deferred to a
   later round before diff-file parity is claimed.

VERDICT: pass

## Round 4 — three corpus-scale rule blockers (process attr, bracket-drop, deep-recursion)

Delta audited: working tree vs committed HEAD `60bea3f` (rounds 1–3), restricted
to `pretty/`. Source touch is exactly two files — `src/rule.rs` (SAPIC `process`
attribute) and `src/doc.rs` (the deep-recursion fix); plus test-only additions
(`tests/round4_blockers.rs`, deep-file fixtures in `round1_term_signature.rs`,
a `process="` parse arm and an 8 MB-not-512 MB stack in `round2_rules.rs`),
provenance notes (BEHAVIOR.md, QUERIES.log), and probe captures. The three
"blockers" map to: **(1)** the SAPIC `process` attribute — byte-forced merger;
**(2)** operator-specific closing-bracket drop — byte-forced merger, **zero
source change** (round-3 fill fix already covers it, round-4 only pins it);
**(3)** the deep-recursion stack-overflow fix — pure engineering on the
BSD-derived engine. Whole suite green (47 + 9 + 9 + 6 = 71 tests), including the
new deep-file parity on an 8 MB stack and a 12 000-group render on a 2 MB stack.

### 1. SAPIC `process` attribute (rule.rs) — byte-forced merger, provenance verified

The clean renders `process="<snippet>"` between `color` and `no_derivcheck`,
DOUBLE-quoted, snippet verbatim, absent when the rule is not `issapicrule`.
Every one of those observable facts is forced by a real oracle capture:

* **Order + spelling + quoting** are read straight off `round2/targets/
  probe_process.hs.txt` (the materialised `r4/probe_process.echo` oracle run):
  `Init[color=#ffffff, process="in(x.1);", issapicrule, role='Process']:` — the
  canonical order `color < process < no_derivcheck < issapicrule < role`, the
  double quotes on `process` vs single quotes on `role`, and the between-color-
  and-no_derivcheck slot are all directly visible. `target:ct`
  (`r4/ct.echo`, 100+ snippets) and `target:running-example` (`r4/running-
  example.echo`, the `process/no_derivcheck/issapicrule` witness) widen it.
* Upstream (`Model/Rule.hs:1201` `prettyRuleAttribute`) emits the same order —
  `color`, `ppProcess`, `no_derivcheck`, `issapicrule`, `role` — with
  `ppProcess p = text "process=" <> text ("\"" ++ … ++ "\"")` (double quotes)
  and `role=\'…\'` (single). The bytes agree, and they agree **because the
  probe forces them**, not because the clean lifted the table: this is the
  merger point, provenance-pinned, exactly the round-2/round-3 discipline.
* **Structural divergence confirms non-transcription.** Upstream re-renders the
  process AST with `prettySapicTopLevel'` (a whole SAPIC pretty-printer) and
  wraps the result. The clean carries `RuleAttr::Process(String)` — the already-
  textual snippet parsed off the source — and prints it verbatim with no
  re-rendering and no escaping. Different mechanism, same bytes. `ppProcess`,
  `ruleProcess`, `prettySapicTopLevel'`, `catMaybes`, `preferRight` do **not**
  appear in the clean. The comment stays in observable-token space (`process=`,
  `role='…'`, provenance tags), naming no upstream source symbol.
* **Escaping honestly flagged UNOBSERVABLE.** Both sides emit the snippet with
  no `"`/`\` escaping; the corpus never puts a `"`/`\` in a process (constants
  are single-quoted), so escaping is unpinnable. The clean registers this in
  BEHAVIOR.md rather than guessing a pinned rule — correct discipline. (Note:
  upstream would also not escape, so no divergence is being masked.)
* Canonical "last color/process/role wins" matches upstream's `preferRight`
  merge on `RuleAttributes`; a reasonable, upstream-consistent canonicalisation.

### 2. Operator-specific closing-bracket drop (blocker 2) — byte-forced, zero source change

The multiset-union `)` drops to its own line at the `(` column when the last
union element is a multi-line tuple, while an application `)` stays JOINED as
`>)`. Both are read off real captures — `round2/targets/probe_uniondrop.hs.txt`
(union → `…>` and `…)` on separate lines) and `probe_appdrop.hs.txt` (app →
`…dd>)`), materialised from `r4/probe_uniondrop.echo` / `probe_appdrop.echo`
and cross-checked against the five ake/dh blocker-2 files (`UM_three_pass`,
`UM_combined{,_fixed}`, `DHKEA{,_keyreg}`). Crucially there is **no round-4
change to the term/fact printer** — the diff touches only rule.rs and doc.rs.
The behaviour is the round-3 `fcat`/fill-item fix (`)` droppable for AC
operators/pairs, plain beside for application); round-4 only adds the pinning
tests `blocker2_union_paren_drops_below_tuple` / `_application_paren_stays_joined`.
Byte-forced merger, provenance verified, nothing new to transcribe.

### 3. Deep-recursion stack-overflow fix (doc.rs) — pure engineering, NON-transcription (special scrutiny)

This is the round's flagged item: it has **no upstream counterpart forced by
the boundary**, so it must not be a transcription of upstream's recursion. It
is not, on every axis checked:

* **It operates on BSD-derived code, not the GPL surface.** Round 1 §2 already
  established doc.rs is the sanctioned `pretty-1.1.3.6` (BSD, Copyright S. Meier
  et al.) HughesPJ port; the layout algorithm — `reduceDoc`/`reduceHoriz`/
  `reduceVert`/`lay` — lives ONLY in the BSD library, and the GPL wrapper
  (`Text.PrettyPrint.Class`) is a thin delegating newtype containing none of
  it. So the pre-existing `reduce_doc`/`reduce_horiz`/`reduce_vert`/`lay` names
  and the `"display lay2 …"` panic strings are **licensed BSD resemblance**,
  not a GPL carry, and there is no GPL upstream recursion to transcribe here.
* **Structure diverges from the recursion it replaces.** Upstream is recursive:
  `reduceDoc (Beside p g q) = beside p g (reduceDoc q)` (Annotated/HughesPJ.hs
  490), likewise `reduceHoriz`/`reduceVert` (554/558) and the mutually-recursive
  display `lay`/`lay1`/`lay2` (1036–1071). The clean's round-4 forms unroll the
  right spine onto a heap `Vec<Frame>` / `Vec<(Doc,bool)>` and fold it back, and
  fuse `lay`/`lay1`/`lay2` into one `loop` driven by a `mid_line` state flag —
  explicit-stack iteration, the deliberate opposite of the recursion. Fold order
  and `beside`/`above`/`eliminate_empty` applications are preserved, so it is
  byte-neutral (the whole R1/R2/R3 suite is unchanged and the deep files
  byte-match on an 8 MB stack) — but the control structure is original.
* **The `Drop` impl has no counterpart on either side.** Haskell is GC'd; the
  BSD `pretty` `Doc` is immutable and never explicitly torn down. The
  explicit-stack `Drop` (placeholder-swap via `EMPTY_DOC`/`EMPTY_LAZY`,
  `detach_children`, drain loop) is a standard Rust idiom for dismantling a deep
  owned chain and is wholly novel to the clean.
* **No upstream identifiers introduced by the delta.** Every new name is
  Rust-engineering vocabulary — `detach_children`, `take`, `empty_doc_placeholder`,
  `empty_lazy_placeholder`, `EMPTY_DOC`, `EMPTY_LAZY`, `Frame`, `mid_line`. The
  merge of `lay1`/`lay2` into one `lay` actually **removes** two BSD-derived
  helper names. Net count of the `"display lay …"` panic strings is unchanged
  (10 in HEAD, 10 in the working tree): the fix relocates the pre-existing,
  sanctioned-BSD, unobservable invariant-violation strings, it does not add any.

### Findings

No violations. The two byte-forced surfaces (the `process` attribute order/
quoting/slot; the operator-specific `)`-drop columns) each resolve to a real
oracle capture (`r4/*.echo` → `round2/targets/*.hs.txt`), so the bytes that
equal GPL upstream are merger, not lift — and where the clean could have lifted
(the process value) it instead diverges structurally (verbatim snippet vs
`prettySapicTopLevel'` re-render). The stack-overflow fix is the round's
engineering centre and is clean on every requested axis: it sits on the
BSD-licensed engine, replaces recursion with explicit-stack iteration (a
divergence, not a copy, of the recursive form), adds an owned-teardown `Drop`
that exists on neither side, and introduces zero upstream identifiers while
shedding two (`lay1`/`lay2`). Identifier, string-constant, comment, and test
scans over the delta are clean. Suite green at 71/71.

Non-blocking notes (advisory, do NOT gate this round):
1. *Byte-neutrality of the iterative engine rests on the test suite, not a
   proof.* `reduce_doc`/`reduce_horiz`/`reduce_vert`/`lay` are asserted
   equivalent to their recursive forms via the full R1–R4 parity corpus (deep
   files on 8 MB, 12 000-group on 2 MB, all prior blocks unchanged). This is
   strong behavioural evidence and is a correctness property, not a similarity
   one — indeed the divergent control flow is itself the non-transcription
   evidence. Any future change to `beside`/`above`/`eliminate_empty` must re-run
   the deep-file parity, since the fold order is the load-bearing invariant.
2. *Engine comments narrate the rewrite ("iterative rewrite of the recursive
   form", "exactly as the recursive form did", "the recursive form was in").*
   Per the standing "comments describe current state only" directive this is
   mild history-narration and could be trimmed to state the current iterative
   invariant (spine-unroll + fold; `mid_line` = start-of-line vs mid-line state)
   without the "was recursive" framing. It is not a similarity issue: the
   referenced "recursive form" is the clean's OWN prior-round BSD-derived code,
   names no GPL symbol, and serves a legitimate byte-neutrality rationale for a
   perf refactor. Advisory only.
3. *Escaping of `"`/`\` in a process snippet remains UNOBSERVABLE* (carried in
   BEHAVIOR.md): no corpus process contains either character (constants are
   single-quoted), so the no-escaping render is unpinned. A process carrying a
   literal `"`/`\` would be needed to pin it; both sides currently emit verbatim,
   so no divergence is hidden. Correct to flag rather than claim.
4. *Blocker-2 is pinned, not implemented, this round.* The `)`-drop behaviour is
   the round-3 fill-item fix; round-4 adds only tests. Should the term printer be
   revisited, `app_doc` must keep the application `)` as a plain beside (NOT a
   fill item) or the `>)` witnesses (probe:appdrop / the ake-dh MAC applications)
   break. Interface caveat, not a similarity finding.

VERDICT: pass

---

## Round 5 — restriction expanded-field split, macros always-break, theory frame

Delta audited: working tree vs committed HEAD `8341919` (rounds 1–4), restricted
to `pretty/`. Source touch is five files — `src/theory.rs` (the `theory … begin
… end` frame, previously an `unimplemented!` stub), `src/macros.rs` (macros
block `sep`→`vcat`; `render_predicates` implemented), `src/lemma.rs`
(restriction comment renders `r.expanded`), `src/ast.rs` (`Restriction.expanded`
field; `TheoryItem` reshaped with `Rule(_, Option<AcVariants>)` /
`Lemma(_, Option<Guarded>)` / `Heuristic` / `Verbatim`), `src/lib.rs` (public
`render_macros`/`render_predicates` re-exports + doc) — plus test additions
(`tests/round5_theory.rs`, the `expanded` field threaded through
`round3_formulas.rs`'s fixture + parser and a new macro-restriction test), the
provenance notes (BEHAVIOR.md "Theory frame"/macros/restriction, QUERIES.log R5
block), and three macro probes. Whole suite green here (6+47+9+10+6+5 = 83).
Method as before (AFC; byte-forced output filtered as merger, residue examined
for protectable expression).

Sealed side (upstream Haskell, `tamarin-rs/tamarin-prover/`), additional to
prior rounds: theory frame — `TheoryObject.hs` `prettyTheory` (732–765) with
`foldTheoryItem` (221–239) over the eight-constructor `TheoryItem`
(`RuleItem`/`LemmaItem`/`TextItem`/`ConfigBlockItem`/`RestrictionItem`/
`PredicateItem`/`MacroItem`/`TranslationItem`), `vsep = foldr ($--$) emptyDoc`
(`Theory/Text/Pretty.hs` 83–84), `kwTheoryName`/`kwTheoryBegin`/`kwEnd`
(119–130); macros — `prettyMacros`/`prettyMacro` (819–840); restriction —
`prettyRestriction` (846–858); predicate — `prettyPredicate` (799–803).

### 1. Theory-frame TRAVERSAL structure (special scrutiny)

The mandate: the frame must not mirror upstream's item-fold decomposition
beyond what output forces. It does not, on every axis.

* **Decomposition diverges — erasure-driven, not upstream's semantic fold.**
  Upstream dispatches through `foldTheoryItem` with **eight** handlers over an
  eight-constructor `TheoryItem`. The clean's `render_item` is a **seven**-arm
  `match` over its OWN `TheoryItem` enum (`Macros`, `Predicates`, `Rule`,
  `Restriction`, `Lemma`, `Heuristic`, `Verbatim`). The partition is by CLEAN's
  erasure surface — blocks that need a ported renderer (Rule/Restriction/Lemma/
  Macros/Predicates) vs blocks that are opaque pre-rendered input (`Verbatim`
  absorbs upstream's `TextItem`, `ConfigBlockItem`, `TranslationItem`, plus
  `tactic:`, `options`, top-level `/* … */` comments) — NOT upstream's
  semantic-item partition. The match-arm order (`Macros` first) does not follow
  `foldTheoryItem`'s argument order (`fRule` first). The enum bundles the
  solver inputs INTO the item (`Rule(Rule, Option<AcVariants>)`,
  `Lemma(Lemma, Option<Guarded>)`) where upstream keeps them inside the rule/
  lemma types and the fold passes `ppRule`/`prettyLemma ppPrf` — a different
  carrier shape. `Heuristic` is promoted to an item; `Predicates` is a grouped
  `Vec` (one item per contiguous run) where upstream folds one `PredicateItem`
  at a time. This is a materially different traversal, not a re-encoding of the
  fold.
* **No hoisting / no config-to-front — a structural divergence (and latent
  delta).** Upstream `prettyTheory` PARTITIONS: config-block items are
  `filter`ed to the FRONT (before `begin`); `thyTactic`/`thyHeuristic`/`thyCache`
  are pulled from separate theory FIELDS into a FIXED slot right after the
  signature; only the non-config items keep source order. The clean does none of
  this — it emits `theory NAME`, `begin`, signature, then ALL `thy.items` in
  raw source order, with `heuristic:`/`tactic:`/config rendered wherever they sit
  in the item list. For the corpus (heuristic/tactic declared early, no config
  block) the output coincides; a late `heuristic:`/`tactic:` or any
  `configuration:` would place it differently than upstream. That is a
  correctness caveat (Note 1), and affirmatively it is the OPPOSITE of mirroring
  the fold — a transcriber would have reproduced the front-filter and the
  fixed-slot hoist.
* **String assembly, not Doc composition.** Upstream builds a `[Doc]` and
  `vsep`s it (`foldr ($--$)`), rendering the whole theory in ONE layout pass.
  The clean renders each item to a **`String`** through its R1–R4 entry point
  (each at column 0) and `join`s with `"\n\n"` (`parts.join`), then appends
  `"\n\n\n\nend"`. Independent per-item rendering is only byte-correct because
  theory items sit at column 0 in the echo (verified — the prior rounds' "render
  once" caveat is thereby discharged for the frame, not by Doc composition but
  by the col-0 observation). This is a distinct construction from `vsep` over
  Docs, reaching the observed bytes.

### 2. Item ordering / spacing / frame tail — byte-forced merger, provenance verified

Everything the frame contributes is observable in the whole echo, hence
compatibility content; provenance is pinned to real captures.

* **Header / signature-first / one-blank-between-items** — `theory NAME`·blank·
  `begin`·blank (69 surveyed captures); the signature block always first, ahead
  of even `tactic:`/`heuristic:` (targets 5G_AKA, contract); items blank-line
  separated in source order. All read off `round1-3/targets/*.hs.txt` and the
  29-file `whole_echo_frame_parity` reconstruction. Byte-forced; matches
  upstream's `vsep`/lineComment/`kwTheoryBegin` only in observable outcome.
* **Three blank lines before `end`** — independently re-verified here across the
  real captures (`*/`·`by sorry`·∅·∅·∅·`end`, exact on BP_IBS_2/3/4, C8,
  cav13, NSLPK3 and the frame test's assertion). This is an EMPIRICAL constant
  (44 surveyed captures, RAW-tail of MacroInLemmasAndRestrictions logged as
  `by sorry`·wf-line·Generated-block·`end` blank-separated), not a lifted
  magic number; the clean's "residue of the stripped wf report + `Generated
  from:` stamp" is a rationalization of a pinned observation, not a mechanistic
  read of `vsep`/`$--$` empty-handling. Byte-forced merger, provenance solid.

### 3. Macros block `sep`→`vcat` (always-break) — byte-forced, divergent construction

R2's macros block used `sep` (all-or-nothing; collapses to one line when it
fits). R5 corrects it to `vcat` (always vertical). This is byte-forced and
non-transcription:

* **Forced by a logged falsification.** `probe:r5_mac2` (`aa(x)=h(x),
  bb(x)=x` — two macros that fit ribbon on one line) still stacks `bb` on its
  own col-8 line in the oracle; that refutes the R2 `sep` law (which every
  earlier witness happened not to expose because it overflowed). BEHAVIOR.md/
  QUERIES.log record the falsification. `vcat` (always-vertical) is THE
  combinator that reproduces "always break", and it is a pre-existing BSD
  `HughesPJ` export. `r5_mac1`/issue777 pin the trivial one-macro line.
* **The BLOCK construction stays divergent from upstream.** Upstream is
  `keyword_ "macros:" $$ nest 4 (vcat [prettyMacro <> comma | …])`: the col-8
  layout arises from the **inner** `nest 4` on `prettyMacro`'s head compounding
  with the outer `nest 4` to 8 > `len "macros:"` (7), so `$$`'s overlap inlines
  the first macro beside `macros:` (confirmed: the read-upstream `$$ nest 4`
  DOES produce the col-8 echo — the oracle and source agree). The clean reaches
  the identical bytes by a completely different route: `text "macros: " <> vcat
  (punctuate ',' items)` — a plain `<>`-beside off an 8-wide prefix, **no**
  double-nest, **no** `$$`/overlap, and `punctuate` for the commas where
  upstream hand-rolls `if i == length m - 1 then … else … <> comma`. Same
  bytes, structurally unrelated Doc expression. A transcriber would have used
  `$$`/`nest 4`; the clean did not (and, per QUERIES.log, derived `vcat` from
  `r5_mac2`, not from the GPL integrator's blocker note that names upstream's
  `$$`/vcat).

### 4. Restriction statement/expanded split — own field model, safety divergence byte-equivalent

R3 rendered `r.formula` in BOTH the statement and the `expanded formula:`
comment (the then-valid "byte-identical in every observation" law). R5 adds
`Restriction.expanded` and renders it in the comment.

* **Byte-forced by a macro witness.** `target:MacroInLemmasAndRestrictions`
  shows statement `A( m(m3(x)) )` vs expanded `A( x )` — two distinct formulas.
  The clean models this as `formula` (statement, macro-form) + `expanded`
  (comment, macro/predicate-expanded), the latter a **caller-supplied opaque
  input** (the ported macro expansion), not derived in-crate. For macro-free
  restrictions the two are equal, reproducing the R3 same-formula-twice bytes.
* **Field model DIVERGES from upstream.** Upstream stores `Restriction rstrName
  expandedFormula ogFormula` with `ogFormula :: Maybe`, statement =
  `fromMaybe expandedFormula ogFormula`, comment = `expandedFormula` (guarded by
  `case ogFormula of Just _`). The clean carries the **rendered roles**
  (`formula` = the resolved statement, always present; `expanded` = the comment
  body) with NO `Maybe`/`ogFormula` construct and its own field names — not
  upstream's `expandedFormula`/`ogFormula` constellation.
* **Safety on the statement vs upstream's on `expandedFormula` — divergent but
  byte-equivalent.** Upstream computes `isSafetyFormula (formulaToGuarded_
  expandedFormula)`; the clean keeps its R3 `is_safety(&r.formula)` (the
  statement). These agree on every input because tamarin macros are **term**
  macros (they substitute terms inside atoms and never add/remove quantifiers,
  so the NNF existential structure — the sole safety input — is invariant) and
  predicates are expanded upstream of BOTH forms. BEHAVIOR.md states exactly
  this reasoning. A divergence in algorithm that is provably byte-neutral, i.e.
  non-transcription, not a latent delta.

### 5. Predicates block — byte-forced margin-0 splice (derived, not read)

`render_predicates` emits `predicate: <fact><=><formula>` per predicate, a
contiguous run blank-line separated. Upstream `prettyPredicate = kwPredicate <>
colon <-> text (factstr ++ "<=>" ++ formulastr)` where `factstr`/`formulastr`
are each `render`ed independently. The clean's `format!("predicate: {}<=>{}",
render_fact(fact), formula::render(body))` reaches the same bytes by the same
CATEGORY of technique (render sub-parts to strings at margin 0, then splice) —
but this convergence is byte-forced, not a lift:

* The splice is **compelled by an observable**: `target:dmn-basic` shows a
  wrapping predicate body breaking at absolute margin 0 (column 1)
  **independent of the header width** (`Sender_duplicate` vs `Mixer_duplicate`
  bodies both at col 1; a 66-col row fits ribbon 73 measured from col 1). Only
  a margin-0 standalone render + textual splice yields that; a `<>`-beside
  would thread the current line width into each `NilAbove` continuation and
  indent the body under `<=>`. The clean pinned this from dmn-basic and
  **confirmed the `<>` threading by reading the SANCTIONED BSD
  `pretty-1.1.3.6` `display`/`lay2`** (protocol-permitted), explicitly logging
  "no tamarin HS source read". Derivation, not transcription.
* The clean builds the head from a synthesised `Fact` routed through its own R2
  `render_fact` and the body through its R3 `formula::render`, where upstream
  uses `prettyFact prettyLVar`/`prettyLNFormula`. Own routing; `predicate:`/
  `<=>`/no-surrounding-spaces are byte-forced.

### 6. Identifier / constellation / constant / comment / test scan

* Leakage grep of the R5-touched `src/` for the frame/macros/restriction/
  predicate upstream surface (`prettyTheory`, `foldTheoryItem`, `prettyMacros`,
  `prettyMacro`, `prettyPredicate`, `prettyRestriction`,
  `prettyTranslationElement`, `ppItem`, `prettyConfigBlock`, the eight
  `*Item` constructors, `thyItems`/`thySignature`/`thyHeuristic`/`thyTactic`/
  `thyCache`, `kwTheory*`/`kwEnd`, `ppNonEmptyList`, `prettyVarList`,
  `ogFormula`/`expandedFormula`, `formulaToGuarded`/`isSafetyFormula`,
  `multiComment`, `keyword_`/`lineComment_`, `prettyGoalRankings`, `vsep`,
  `ppCache`) and the pseudonymous author handles: **no matches**.
* AST constellation DIVERGES. The clean's `TheoryItem` (7 variants, `Verbatim`/
  `Heuristic`, item-bundled `Option<AcVariants>`/`Option<Guarded>`) does not
  mirror upstream's eight `*Item` constructors; `Restriction {name, formula,
  expanded}` is the clean's rendered-role model, not upstream's
  `expandedFormula`/`ogFormula :: Maybe`.
* Non-observable constants: the `\n\n\n\n` pre-`end` tail is empirically pinned
  (44 captures, re-verified here), not a lifted number; the margin-0 predicate
  splice is derived from dmn-basic; the one-blank-between-items is observed.
  None lifted.
* Comment lineage: the only external references in the delta are to the
  SANCTIONED BSD library (`pretty-1.1.3.6` `display`/`lay2`; permitted) and to
  the clean's own probes/targets — no tamarin source-file narration, no line
  citations, no reproduced ghost comments.
* `tests/round5_theory.rs`: probe-pinned unit fixtures (gap2 macros always-
  break, predicate one-liner/group, margin-0 wrap, gap3 frame glue) plus
  `whole_echo_frame_parity` — a **layout-insensitive** reconstruction that
  reparses 29 diverse REAL captures (discarding all whitespace, so bytes can
  only come from the renderer), rebuilds `Theory`+`Signature`, and byte-matches
  `render_theory` against each capture (asserts ≥15). The parser asserts the
  frame invariants (3-blank tail, signature-first) structurally rather than
  trusting them. No lifted upstream text. Green here (5/5; frame parity 29/29).

### Findings

No violations. The theory frame is the round's scrutiny centre and is clean on
the mandated axis: the traversal is a coarser, erasure-driven `match` (7 arms,
`Verbatim`-collapse, item-bundled solver inputs, `Heuristic` promoted) that does
NOT mirror upstream's eight-handler `foldTheoryItem`, performs none of
upstream's config-to-front / tactic-heuristic-cache fixed-slot hoisting, and
assembles by column-0 string-join rather than `vsep` over `[Doc]`. The parts
that equal upstream — item order, one-blank separation, `theory`/`begin`/`end`
keywords, the 3-blank pre-`end` tail — are byte-forced merger pinned to 44+29
real captures. The macros `sep`→`vcat` correction is byte-forced by a logged
`r5_mac2` falsification while the block's Doc construction stays divergent from
upstream's `$$ nest 4`/overlap; the restriction `expanded` split uses the
clean's own rendered-role field model (safety-on-statement is a divergent but
provably byte-neutral algorithm, macros being term-level); the predicate
margin-0 splice is derived from dmn-basic and confirmed against the sanctioned
BSD engine, not from `prettyPredicate`. Identifier, constellation, constant,
comment and test scans over the delta are clean.

Non-blocking notes (advisory, do NOT gate this round):
1. *Frame reordering deltas vs source, gate-backstopped.* The clean emits
   `heuristic:`, `tactic:` and (unmodeled) `configuration:` in raw source-item
   order, whereas upstream hoists `thyHeuristic`/`thyTactic` to a fixed slot
   after the signature and `filter`s config blocks to the front (before
   `begin`). Correct for the probed + curated corpus (these appear early / are
   absent); a theory declaring `heuristic:`/`tactic:` after other items, or any
   `configuration:` block, would order differently than upstream. This is a
   correctness caveat — and affirmatively the non-mirroring of the fold — not a
   similarity finding; the full-corpus frame gate is the backstop before wider
   parity is claimed.
2. *No `ppCache` slot modeled.* Upstream renders `ppCache thyCache` in the fixed
   post-heuristic slot; for the no-prove echo the cache is `emptyDoc`/out of
   span, so the clean omits it. A closed-theory cache surfaced in span would be
   unmodeled — deferred/unobserved, consistent with the `--diff` deferrals of
   prior rounds.
3. *3-blank tail is empirically pinned, not derived.* It rests on the invariant
   that the gate strips exactly the wf report + `Generated from:` stamp (each
   one blank-separated slot). Any change to what the extraction drops must
   re-verify the tail; strong across 44 captures today.
4. *Predicate margin-0 splice rests on the sanctioned-BSD width threading.* The
   textual splice (rather than `<>`) is required because BSD `display`/`lay2`
   threads the line width into `NilAbove` continuations. A future doc-engine
   change touching `NilAbove`/continuation nesting must re-check the dmn-basic
   margin-0 predicate wrap.

VERDICT: pass
