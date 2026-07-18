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
