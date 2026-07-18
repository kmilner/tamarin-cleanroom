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
