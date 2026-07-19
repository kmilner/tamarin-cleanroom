# BEHAVIOR.md — inferred behavioral spec of the theory echo (R1: term core + signature block; R2: rule blocks; R3: restriction/lemma formulas)

Every claim traces to a logged oracle probe (QUERIES.log) or a pre-materialized
round-1 capture (`round1/targets/*.hs.txt` — itself oracle output). Notation:
"target:<file>" = observed in that capture; "probe:<name>" = constructed
.spthy run through `oracle/pretty_oracle.sh` (files under
workspace/scratchpad/probes/, outputs kept beside them as `.out`).

## Layout engine

HughesPJ Doc at line width 110, ribbon 73 (SPEC-provided parameters; all wrap
observations below are reproduced by the sanctioned BSD engine at these
settings). Key engine consequences confirmed against probes:

* A one-line candidate is kept iff it fits within ribbon 73 measured from the
  current line position; an exactly-73-column line still fits
  (probe:e_mid — 73-column `equations:` one-liner kept; probe:b_all —
  builtins item that would end at column 75 wraps).
* Fill continuations align at the fill's origin column, and that indent does
  NOT consume ribbon: continuation content is measured from the alignment
  column (probe:b_all functions line 2 reaches display column 79 = 11 + 68;
  probe:t_wide App continuation reaches display column 78).
* Nested content (e.g. rule bodies at indent 3) likewise measures its ribbon
  from the nest, so lines up to 76 display columns appear at nest 3
  (probe:t_edge).

* **Deep-structure robustness (R4 blocker 3, BYTE-NEUTRAL).** A rendered Doc is
  a per-LINE linked chain (`NilAbove`/`TextBeside`/`Nest` interleaved with
  memoised `Lazy(Forced)` thunks) as deep as the theory has lines. A huge
  `variants (modulo AC)` block reaches ~10 000 lines (C8's
  `Terminal_Receives_Records_LocalAuth_C8` is 9 886 lines; BP_IBS_2/3/4 ~5–6k),
  so the naive recursive `reduceDoc`/`reduceVert`/`reduceHoriz` (build), the
  display walk (`lay`), and — the last one to surface — the compiler-generated
  recursive `Drop` of the nested `Rc<Doc>` all overflow a normal 8 MB stack
  (measured: overflow between ~1 500 and ~2 000 two-line groups). The wide FILL
  machinery is already lazy and does not overflow; only these eager `Above`/
  spine/drop walks do. All four are rewritten to iterate over a heap `Vec`
  (spine unrolled then folded; `lay` a loop; an explicit-stack `Drop` for
  `Doc`) — a pure evaluation-order change, so every rendered byte is identical
  (whole existing R1/R2/R3 suites unchanged) and the four deep witness files
  render completely and byte-match the oracle. Proof: a 12 000-group block
  renders on a 2 MB stack, and the round-2 curated set (now including the four
  deep files) passes on a production 8 MB stack, not the former 512 MB one.

## Signature section

Overall shape (all targets/probes):

```
// Function signature and definition of the equational theory E
<blank>
builtins: …        (line ABSENT when no surviving entries)
functions: …       (always present — base symbols guarantee items)
equations: …       (always present — base equations)
```

The three declaration lines are adjacent (no blank lines between them).

### `builtins:` line

* Only these builtins keep a builtins-line entry: `diffie-hellman`,
  `bilinear-pairing`, `multiset`, `natural-numbers`, `xor`
  (probes:b_<name>). All others (hashing, signing, revealing-signing,
  symmetric-encryption, asymmetric-encryption, dest-*, locations-report)
  vanish from the line, contributing functions/equations instead
  (probes:b_*, b_locrep; target:classic_NSLPK3 has no builtins line).
* `bilinear-pairing` INDUCES `diffie-hellman` (probe:b_bilinear-pairing:
  echo `builtins: diffie-hellman, bilinear-pairing`; target:sp14_Joux).
* Canonical order regardless of source order: diffie-hellman <
  bilinear-pairing < multiset < natural-numbers < xor (probe:b_all — source
  order was scrambled; target:features_multiset_NumberSubtermTests).
* Duplicates collapse (probe:b_dupline).
* Layout: `text "builtins: " <> fsep (punctuate ',' names)` — items
  comma-separated, single space, fill-wrapped, continuation aligned at
  column 10 (after `builtins: `), comma attached to the preceding item
  (probe:b_all, target:features_multiset_NumberSubtermTests).

### `functions:` line

* Items render `name/arity` + attribute suffix, comma-separated.
* Attribute suffix (probe:f_attrs, probe:b_locrep, probes:b_dest-*):
  - public constructor → no suffix (`c/3`)
  - private constructor → `[private,constructor]`
  - public destructor → `[destructor]`
  - private destructor → `[private,destructor]`
  No space before `[`; no spaces inside.
* Sort: ASCII byte order on the name, case-sensitive (`Bb/1, Zz/1, a1/2, aa/1,
  cA/0` — probe:f_sort).
* Dedup: identical (name,arity) between user decls and builtin expansions, or
  repeated user decls, appear once (probe:f_dedup). Conflicting arity or
  attribute redeclarations are REJECTED upstream by the tool
  (probe:f_dedup2 raw error), so the renderer never sees them.
* Typed declarations (`kdf(bitstring):skey`) erase to `kdf/1`
  (probe:f_typed).
* Base symbols always present: `fst/1, pair/2, snd/1` (probe:b_none);
  with `dest-pairing` instead: `fst/1[destructor], pair/2, snd/1[destructor]`
  (probe:b_dest-pairing).
* Per-builtin function expansions (compatibility content, from probes
  b_hashing / b_asymmetric-encryption / b_signing / b_symmetric-encryption /
  b_revealing-signing / b_locrep / b_dest-*):
  - hashing → `h/1`
  - asymmetric-encryption → `adec/2, aenc/2, pk/1`
    (dest-asymmetric-encryption: `adec/2[destructor]`)
  - signing → `pk/1, sign/2, true/0, verify/3`
    (dest-signing: `verify/3[destructor]`)
  - symmetric-encryption → `sdec/2, senc/2`
    (dest-symmetric-encryption: `sdec/2[destructor]`)
  - revealing-signing → `getMessage/1, pk/1, revealSign/2, revealVerify/3,
    true/0`
  - locations-report → `check_rep/2[destructor], get_rep/1[destructor],
    rep/2[private,constructor], report/1`
  - diffie-hellman / bilinear-pairing / multiset / natural-numbers / xor →
    none (their operators/constants — one, inv, DH_neutral, zero, ++, %+ —
    never appear in `functions:`; probes b_diffie-hellman, b_xor, …).
* Layout: `text "functions: " <> fsep (punctuate ',' items)` — fill wrap,
  continuation aligned at column 11 (probe:b_revealing-signing, probe:b_all,
  probe:e_long).

### `equations:` block

* Header `equations:`, or `equations [convergent]:` when the user block is
  declared `[convergent]` (probe:e_conv).
* Base equations always present: `fst(<x.1, x.2>) = x.1`,
  `snd(<x.1, x.2>) = x.2` (probe:b_none).
* Per-builtin equation expansions (probes as above):
  - asymmetric-encryption → `adec(aenc(x.1, pk(x.2)), x.2) = x.1`
  - signing → `verify(sign(x.1, x.2), x.1, pk(x.2)) = true`
  - symmetric-encryption → `sdec(senc(x.1, x.2), x.2) = x.1`
  - revealing-signing → `getMessage(revealSign(x.1, x.2)) = x.1`,
    `revealVerify(revealSign(x.1, x.2), x.1, pk(x.2)) = true`
  - locations-report → `check_rep(rep(x.1, x.2), x.2) = x.1`,
    `get_rep(rep(x.1, x.2)) = x.1`
  - hashing → none
* Sort: STRUCTURAL order on the (lhs, rhs) pair — NOT byte order of the
  rendered text (refuted by target:contract — `checkpcs(xc, xpk, …)` prints
  before `checkpcs(xc, pk(xsk), …)` though 'x' > 'p' — and by target:mesh's
  get_b1/get_b2/aes_cmac groups) and NOT declaration order in either
  direction (probes p_eqA/p_eqB and p_eqF/p_eqF2: reversed sources echo
  identically; contract/mesh source order is the reverse of the echo). The
  pinned comparison, lexicographic (lhs first, rhs breaks lhs ties —
  probes p_eqF/p_eqF2):
  - variables sort BELOW all applications regardless of names
    (probes p_eqC/p_eqC2: `f(zzz, …)` < `f(a0, …)` with a0/0 nullary;
    corpus witnesses contract `xpk` < `pk(xsk)`, mesh `cnf` <
    `aes_cmac(…)`);
  - variable vs variable: name bytes (NOT shortlex — probe:p_eqD `azz` <
    `b`), then index (probe:p_eqH user `x` < builtin `x.1`);
  - application vs application: head name bytes FIRST, then arguments
    left-to-right (contract checkpcs decided at argument 2; head-name byte
    order across groups: mesh `aes_ccm_dec` < `aes_cmac`, refuting
    shortlex); arity never discriminates observably except via `pair`
    (probe:p_eqG refutes arity-first: `pair`/2 < `z1`/1);
  - tuples compare as RIGHT-NESTED binary applications named `pair`
    (probe:p_eqE `g(…)` < `<x, y>`; probe:p_eqG `<x, y>` < `z1(…)`;
    probe:p_eqI `<x, zz>` < `<x, b, c>` — the binary view compares var `zz`
    vs app `pair(b, c)`; a flattened elementwise view would reverse them).
  UNOBSERVABLE ranks (cannot occur in an equations block, flagged): literal
  constants (rejected upstream — probe:e_samehead raw), exp/AC operators,
  diff, pattern-match. Implemented as application-rank under their rendered
  head spelling; corpus gate is the backstop if they ever occur.
* Dedup: EXACT duplicates collapse (probe:e_dup); alpha-equivalent but
  differently-named equations do NOT (probe:e_adedup keeps user
  `fst(<a, b>) = a` alongside the builtin fst equation).
* Layout: `sep (header : map (nest 4) (punctuate ',' eqDocs))` — i.e.
  all-or-nothing, NOT fill:
  - fits on one line (≤ 73) → `equations: e1, e2, e3` (probe:e_mid at
    exactly 73);
  - otherwise header alone, then EVERY equation on its own line at indent 4
    with trailing comma on all but the last — even when several would fit
    joined at indent 4 (probe:e_conv: joined length 71 still one-per-line).
* Each equation is `sep [lhsDoc, nest (-2) (text "= " <> rhsDoc)]`:
  one line `lhs = rhs`; when too long, `= rhs` drops to (equation indent − 2)
  (probe:e_long — `  = xlongvariablename1` at column 2 under equations at
  indent 4).

## Term rendering

Observed atoms:

| AST shape | rendered | provenance |
|---|---|---|
| var, untagged/msg | `x` | everywhere |
| var, fresh | `~x` | target:cav13 (`x:fresh` source → `~x`) |
| var, pub | `$A` | target:cav13 |
| var, node | `#i` | targets (lemma text) |
| var, nat | `%x` | target:NumberSubtermTests |
| var with index k>0 | `name.k` after the sigil (`~x.7`, `x.10`, `XB.10`) | targets: variants + builtin eqs |
| pub literal | `'g'`, `'hello_world'` | probe:t_pair |
| fresh literal | `~'n'` | probe:t_frlit |
| nat literal 1 (`%1`, `1:nat`) | `%1` | probe:t_nat |
| nat literal n (`2:nat`) | `%2` | probe:t_num2 |
| DH one (`1` in DH theory) | `one` | probe:t_one, probe:t_gone |
| DH neutral | `DH_neutral` | probe:t_gone, target:cav13 variant 7 |
| xor zero | `zero` (nullary-app form) | probe:t_xor |

Composite shapes:

* Application: `f(a1, a2)` — `text f <> "(" <> fsep (punctuate ',' args) <>
  ")"`; commas attach to the preceding arg, fill space between args, wrap
  aligns after the `(`, closing `)` attached to the last arg
  (probe:t_wide W2 — continuation line `ylongvariablename2,
  zlongvariablename3)`; probe:e_long). Nullary symbols render bare (`shk`,
  `zero`, `true`, `f` — targets + probes).
  The application `)` is the plain BESIDE `<> ")"`, NOT a droppable fill item:
  when the last argument is a multi-line tuple whose `>` drops to its own line,
  the `)` STAYS JOINED to it as `>)` (probe:appdrop `someop(…, <…, macf(…)>)`
  → `…dd>)`; probe:appdrop2 nested → `…>))`; the round-2 blocker-2 corpus —
  ake/dh UM_three_pass etc. — has MAC applications with this exact shape). This
  is the crux of R4 blocker 2: the enclosing-delimiter drop is operator-
  specific — an AC-operator/pair `)`/`>` DROPS (fill item), an application `)`
  does NOT (beside). Changing `app_doc` to drop the `)` would break these
  witnesses; the fix is to leave it beside.
* Pair: `<a, b, c>` — ONE fill (`fcat`) whose items are `<`, each element
  with its attached `", "` under `nest 1`, and `>`:
  `fcat ('<' : map (nest 1) (punctuate ", " elems) ++ ['>'])`. This single
  construction reproduces every observed shape (probe:p_pw1 wfa–wfd,
  target:mesh k2 equation, probe:t_wide W1):
  - fits → `<a, b>` on one line; a wrapped fill line ends with the trailing
    space of the attached `", "` (probe:t_wide W1 byte-checked);
  - overflow elements continue at (column of `<`) + 1 — the nest 1;
  - when the FIRST element's one-liner does not fit beside `<`, the `<`
    stays ALONE on its line (probe:p_pw1 wfb; mesh k2 inner tuple);
  - `>` sits beside the last element when that element ends a fill line
    (p_pw1 wfa/wfb/wfd), but drops to its own line at the column of `<`
    (no nest) when the last element is multi-line (p_pw1 wfc; mesh k2
    outer tuple) — the fill places items after a multi-line item below it.
  The earlier R1 law (`"<" <> fcat … <> ">"`, `<`/`>` always attached) was
  WRONG — falsified by mesh k2 / p_pw1; it agreed with the true law only on
  the shapes R1 observed (fits, and wfa/wfd-style wraps).
  Right-nested pairs flatten (`<x, <y, z>>` → `<x, y, z>`);
  a pair in NON-last position stays delimited: `<<x, y>, z>` (probe:t_pair).
  Source `pair(x, y)` arrives already normalized to a pair (probe:t_pair).
* Exponentiation: `a^b`, chains render FLAT for both left- and right-nested
  trees: `('g'^~x)^~y` AND `'g'^(~x^~y)` both echo `'g'^~x^~y`
  (probe:t_exp2). No parentheses are ever added around exp or its operands
  (operand classes that need delimiting — AC ops — are self-parenthesizing).
* AC operators mult `*`, xor `⊕` (U+2295), multiset `++`, nat-plus `%+`:
  always wrapped in parens, no spaces, arguments flattened across BOTH sides
  regardless of source nesting: `(x⊕y⊕z)`, `(a++b++c)`, `(%x%+%y%+%z)`,
  `('g'-exponent (~x*~y*~z))` (probes:t_xor, t_uni, t_nat, t_mult2, t_exp2 —
  the AC rule variants normalize (g^x)^y to `'g'^(~x*~y)`).
  The parens are intrinsic: they appear in every context (function argument
  `inv((~x.7*x.11))` — target:cav13 variants; fact argument `Out( (~x⊕~y) )`).
  Layout on overflow: the SAME single-fill construction as tuples, with the
  delimiters as fill items —
  `fcat ('(' : map (nest 1) (punctuate op elems) ++ [')'])`:
  break between elements with the operator attached to the preceding element
  and no fill space, continuation at (column of `(`) + 1 (probe:t_uniwide:
  line ends `…aaaaaaaaaaaaaaaaa3++`), and the `)` drops to its OWN line at
  the column of `(` when it does not fit beside the last element
  (round-3 target:alethea Universal_VerProofV_v1 — the union keeps both wide
  tuple elements on one 71-column fill line and only `)))"` drops). The
  earlier R1 law (`"(" <> fcat … <> ")"`, `)` always attached) agreed with
  the true construction on every R1-observed shape but was falsified by the
  alethea witness.
  The R4 blocker-2 corpus (ake/dh: UM_three_pass{,_combined,_combined_fixed},
  DHKEA{,_keyreg}) exercises the OTHER `)`-drop sub-case: the LAST union element
  is itself a multi-line tuple, so the tuple's `>` drops to its `<` column AND
  the union's `)` drops BELOW it to the `(` column, on SEPARATE lines
  (probe:uniondrop `(<'1', …>++<'2', dd, macf(…)>)` →
  `…>` / `…)` on their own lines; UM_three_pass echo lines ~54–55, whole-block
  byte-parity re-asserted). The alethea round-3 fix (`)` as a fill item)
  already covers this; the round-2 curated set now re-asserts it across the
  five ake/dh files. (Contrast the application `)` above, which stays `>)`.)
* diff: `diff(x, y)` — application form (probe:t_diff, run with --diff).
* mult inside exp exponent: `x.10^(x.11*inv(x.12))` (target:cav13) — normal
  AC-paren rule, no extra exp parens.

## Rule blocks (R2)

Curated byte targets: round2/targets/*.hs.txt (12 corpus files, 72 rule
blocks, every block byte-verified by tests/round2_rules.rs round-trip
parity). A rule BLOCK is:

```
rule (modulo E) Name[attrs]:
   <body>
<blank line>
  // loop breaker(s): [..]        (optional, col 2)
  <variants comment>              (col 2)
```

### Header

* `rule (modulo E) Name:` — the modulo annotation is input data (`E` on the
  closed rule, `AC` on the variant re-render inside the comment).
* Attribute list attaches to the name: `Name[color=#abcdef, process="…",
  no_derivcheck, issapicrule, role='r']:` — canonical order color < process <
  no_derivcheck < issapicrule < role regardless of source order; the LAST
  color/process/role declaration wins; external `x-…` attributes are DROPPED
  (probe:p_rattr, target:issue713). Spellings: `color=#<hex>` unquoted
  (lowercased upstream: '#AbCdEf' → `#abcdef`), `role='…'` single-quoted.
  Fill-wrap aligned after the `[`, `]:`  attached to the last item
  (probe:p_rattr R3, target:issue713 bla).
* **SAPIC `process` attribute** (R4 blocker-1 correction; probe:p_process,
  target:ct — the earlier "process is DROPPED" law was WRONG, from a corpus
  that had no `issapicrule` files in the round-2 set). It is present on every
  SAPIC-generated (`issapicrule`) rule, ABSENT otherwise, and renders
  `process="<snippet>"` — DOUBLE quotes (contrast `role='…'`), positioned
  between `color` and `no_derivcheck`. The `<snippet>` is the translated
  process step, carried VERBATIM as one unbreakable text token: `process="|"`
  (parallel), `process="!"` (replication), `process="0"` (null),
  `process="in(x.1);"`, `process="event Ev( x.1 );"`,
  `process="out(<x.1, 'lbl'>);"`, `process="lookup <'att', h.3> as a.1"`,
  `process="insert <$ca.2, 'proofOfID', $s.1, pk(skS.1)>,'yes';"`. Its interior
  spaces, commas, `<>`, `()`, `;` and SINGLE-quoted string constants are part
  of the value, NOT layout — the fill never breaks inside it (target:ct 100+
  witnesses; probe:p_process). ESCAPING of `"`/`\` is UNOBSERVABLE: process
  snippets quote their constants with `'…'`, so no `"`/`\`/control char ever
  reaches the attribute; rendered with no escaping (flagged — a corpus with a
  process containing `"`/`\` would be needed to pin it).

### Body

* `nest 3 (sep [prem-group, nest (-1) arrow-group, concl-group])` — one line
  `[ … ] --[ … ]-> [ … ]` when it fits, otherwise three rows: groups at
  col 3, arrow at col 2. TRUE sep, not a fill: cav13 Step1 keeps `-->` on
  its own row although it would fit beside the premise row.
* Bracket groups (premises, conclusions, AND the action arrow — one
  construction, probe:p_arr1): `sep [sep [open, fsep facts], close]`:
  - one line `[ f1, f2 ]` when the whole group fits (kept at exactly
    ribbon 73 — probe:p_arr1 PR);
  - else `close` drops to the group's column; `open` keeps ALL facts beside
    it iff they fit that line as a unit (target:mesh DeviceWaitingUser at
    exactly 73 vs ProvisionerWaitingUser one column over);
  - else `open` is alone too, facts fill-wrap at the group's column
    (target:NSLPK3 R_1 three facts per fill line; target:Tutorial Serv_1 —
    `--[` alone although the first action alone would fit beside it).
  Empty list → `[ ]`; empty actions → the literal `-->`.
* Fact: `sep [name<>"(" <+> fsep args, ")"]` — `Name( a, b )`, args
  fill-wrap aligned after `Name( `, `)` drops alone to the fact's column
  when the args are multi-line (probe:t_wide); `Name( )` when nullary
  (target:mesh); `!` prefix on persistent facts (target:NSLPK3).
* Fact annotations attach directly after the `)`: `[+, -, no_precomp]` —
  canonical order + < - < no_precomp regardless of source order
  (probe:p_fann `[no_precomp,+]` → `[+, no_precomp]`;
  target:seqdfsneeded `[no_precomp,-]` → `[-, no_precomp]`).

### Loop breakers

* `// loop breaker: [0]` singular / `// loop breakers: [0,1]` plural,
  0-based premise indices, comma WITHOUT space (probes c_loop, p_lb2).
* E-rule side: on its own line at col 2, between the blank line and the
  variants comment. AC-rule side: INSIDE the comment at col 4, AFTER the
  variants list (probe:p_lbvar).

### The variants comment

* Trivial: `/* has exactly the trivial AC variant */` at col 2.
* Otherwise (col 2 `/*` … `*/`):
  ```
  /*
  rule (modulo AC) Name:
     <body, same layout, nested +2>
    variants (modulo AC)          (col 4; ABSENT when no substitutions —
    <numbered groups>              target:issue777 macro-expanded AC rule)
    // loop breaker: [..]         (optional, col 4)
  */
  ```
* Numbered groups at col 4: the index is RIGHT-ALIGNED to the widest
  `N.` (`1.` cav13/7 variants; ` 9.`/`12.` CH07; `  1.`/`160.` Joux),
  followed by one space; the group's substitution lines align after that
  prefix. Groups are separated by a line of BARE INDENT (four spaces at
  this nesting — trailing whitespace, byte-checked probe:p_var1); no
  separator after the last group.
* One substitution = `lhs $$ nest 6 ("= " <> rhs)` with HughesPJ overlap:
  a lhs narrower than 6 columns is padded to column 6 (`~lv2  = ~lv2.4`,
  `~ltkS = …` at exactly 5+1); a lhs of ≥ 6 columns pushes `= rhs` to its
  own line at column 6 relative to the entry (probe:p_var1
  `~longvariablenameone`, target:Tutorial `request`). The rhs is a full R1
  term with its own wrapping (target:mesh multi-line aes_cmac values).
  The lhs/rhs substitution DATA is solver-provided input; both sides render
  through the R1 term core.

### `macros:` block (rule-adjacent surface)

* `text "macros: " <> vcat (punctuate ',' items)` — the block ALWAYS breaks
  (R5 GAP-2 correction; the earlier `sep`/all-or-nothing law was WRONG). The
  first macro sits beside `macros: ` and every subsequent macro goes on its
  own line aligned after `macros: ` (col 8), REGARDLESS of fit
  (probe:r5_mac2 — two short macros that would fit one line still break;
  probe:p_mac1; target:issue777 / probe:r5_mac1 — a lone macro is trivially
  one line). Commas attach to the preceding macro; the last carries none.
  (`sep` agreed with the true `vcat` law only when the list overflowed, which
  every earlier witness happened to do — probe:r5_mac2 is the first
  fits-but-still-breaks witness.)
* Item: fact-style head with the `)` ATTACHED to the last param line
  (unlike facts), then `) =  body` — two spaces after `=` (the `= ` token
  plus hsep spacing); the body always sits beside and wraps internally
  (probe:p_mac1 wide macro).

### Out of R2 scope (recorded, not implemented here)

* The theory-level `/* looping facts with injective instances: A/1 */`
  comment (probe:c_loop) sits between the signature and the rules — top
  level assembly, not the rule block.
* Embedded rule restrictions (`_restrict`) never surfaced in any probed
  echo (TPM_DAA corpus files lift them to top-level restrictions upstream);
  no rendering law claimed.
* Diff-mode (`--diff`) rule surfaces (left/right variants) are not covered
  by the round-2 curated set; a later round must pin them before diff-file
  parity is claimed.

## Formula rendering (R3)

Curated byte targets: round3/targets/*.hs.txt (20 corpus files; 84
restriction + 139 lemma blocks, every block byte-verified by
tests/round3_formulas.rs round-trip parity).

### Glyphs and atoms

| construct | rendered | provenance |
|---|---|---|
| true / false | `⊤` / `⊥` | probe:q_w1 |
| conjunction / disjunction | `∧` / `∨` | probe:q_at1, q_p2 |
| implication / iff | `⇒` / `⇔` | probe:q_p2, q_r2 |
| negation | `¬(…)` — argument ALWAYS parenthesized | probe:q_p2 s8 |
| quantifiers | `∀` / `∃` | everywhere |
| action atom | `Fact( … ) @ tp` — spaces around `@` | probe:q_at1 |
| temporal/nat order | `t1 < t2` | probe:q_at1 |
| equality | `t1 = t2` | probe:q_at1 |
| subterm | `t1 ⊏ t2` (source `<<` and `⊏` both echo `⊏`) | probe:q_at1, target:NumberSubtermTests |
| last | `last(tp)` — NO interior spaces (unlike facts) | probe:q_at1 |

Terms inside atoms render through the R1 term core unchanged (probe:q_l4 —
pair trailing-`", "` wrap, AC self-parens); action facts through the R2 fact
construction unchanged, `!` prefix included (target:NSLPK3 `!KU( ni ) @ #j`,
probe:q_l3).

### Parenthesization

* EVERY operand of a binary connective (∧ ∨ ⇒ ⇔) is wrapped in `(…)`,
  whatever it is — atom, ⊤/⊥, ¬, quantifier, another connective
  (probe:q_p2 s1–s13; targets NSLPK3/Cronto/acc).
* `¬`'s argument is always wrapped (double wrap when ¬ is itself an
  operand: `(¬(x = 'd'))`).
* Quantifier BODIES and the top level are bare (probe:q_p2, q_w1).
* Chains render their source association faithfully — `a | b | c` parses
  left-nested upstream and echoes `((a) ∨ (b)) ∨ (c)`; no flattening,
  no re-association (probe:q_at1, probe:q_p2 s3–s6).

### Layout

* Binary connective = `sep [lhs-operand <+> glyph, rhs-operand]`: one line
  `(A) ∧ (B)` when it fits; otherwise the glyph stays on the lhs' LAST line
  and the rhs drops to the group origin (targets NSLPK3 types /
  Yubikey slightly_weaker_invariant / Cronto notSameRole deep left chain).
* Quantifier = `sep [glyph<>" "<>fsep binders<>".", nest 1 body]`:
  - binders are space-separated in source order with their R1 sigils and
    `.idx` suffixes (`∀ x.1 a.1 #i.1.` — probe:q_b1, target:acc lemmas);
  - the binder fill wraps aligned after `"∀ "` (origin+2 — probe:q_l2 bw1);
  - the `.` attaches to the last binder;
  - the body sits beside after one space, or drops to (quantifier origin+1)
    — the nest 1 (probe:q_l2 bw1/bw2; paren-nested `(((∃ #j.` → body at
    paren col + 2 confirms the same nest at depth).
* Relation atoms (`=` `<` `⊏`) = `sep [lhs-term <+> glyph, rhs-term]`: glyph
  attached to the lhs line, rhs drops to the atom origin (probe:q_l4 tw1–3).
* Action atom = `hsep [fact, "@", timepoint]`: `@ tp` NEVER drops alone —
  at overflow the FACT breaks internally (its `)` drops to the fact column,
  `@ tp` beside it: `) @ #i` — probe:q_l5 m63/m64 pins hsep over the
  sep-alternative, which would have kept `fact ) @` on one line).
* One-line fits follow the engine at 110/73 with the ribbon measured from
  the nest; the closing quote/parens attached after a formula count against
  its last line's fit (probe:q_l5 — 73-content + `"` broke).

## Restriction blocks (R3)

```
restriction Name:
  "formula"                (nest 2; quotes attach directly around the doc)
  // safety formula        (col 2 — iff the formula classifies safety)
<blank>
  /*                       (comment at nest 2)
  expanded formula:
  "formula"
  */
```

* The `axiom` keyword echoes as `restriction` with the identical wrapper
  (probe:q_ax1, target:Cronto_EA).
* **Statement vs expanded formula (R5 GAP-1).** The STATEMENT renders in MACRO
  form (as written); the `expanded formula:` comment renders in the
  macro/predicate-EXPANDED form. They are TWO DISTINCT formula values whenever
  the restriction uses a macro (target:MacroInLemmasAndRestrictions — statement
  `A( m(m3(x)) )`, expanded `A( x )`). PREDICATE expansion happens upstream of
  BOTH renderings, so a predicate-only restriction still shows the two
  identically (probe:q_pred1, target:features_predicates_minimal), and every
  macro-free restriction renders the same formula twice (the earlier "byte-
  identical in every observation" law — a special case, since no earlier probe
  had a macro). The expanded formula is a caller-supplied opaque input (the
  ported macro expansion), modeled as `Restriction.expanded`; safety is
  classified on the statement (macro expansion is term-level, so quantifier
  structure — hence the safety verdict — is identical either way).
* LEMMAS with macros have NO separate expanded-formula comment: the STATEMENT
  is the macro form (`l.formula`) and the guarded comment carries the EXPANDED
  form as opaque input (already handled by `Guarded`) — confirmed for
  exists-trace (target:MacroInLemmasAndRestrictions) and all-traces
  (probe:r5_allmacro: statement `Ev( g(x) )`, guarded `Ev( h(x) )`). No lemma
  model change was needed.
* Safety classification (pinned by probes q_s1/q_s2 + q_w1 + 84-block corpus
  parity): a formula is safety iff its negation-normal form contains NO
  existential quantifier, msg-sort and temporal alike. Every ⇒-antecedent
  flips polarity; `¬∃` in a conclusion (∀ in NNF) keeps safety, `¬∃` in an
  antecedent (∃ in NNF) defeats it; a non-temporal `∃ y.` conclusion defeats
  it; top-level `¬(∃ …)` restrictions are safety (target:Yubikey).
* Sort-order of the comment lines: statement, safety line, blank, comment —
  the safety line sits BETWEEN the formula and the blank (probe:q_w1 r_eq).

## Lemma blocks (R3)

```
lemma Name [attr1, attr2]:
  all-traces|exists-trace "formula"    (sep at nest 2 — one line iff it fits)
/*                                     (comment at col 0)
guarded formula characterizing all counter-examples:      (| all satisfying traces:)
"<guarded block>"                      (opaque input, verbatim lines)
*/
by sorry                               (| the embedded proof, verbatim)
```

* Header: `lemma Name:`; with attributes `lemma Name [` + fill + `]:` — a
  SPACE before `[` (unlike rule attributes), items comma+space separated,
  fill-wrapped aligned after the `[` with `]:` attached to the last item;
  the first item stays beside the `[` even past the width (target:5G_AKA
  weakagreement_… at 120 display columns).
* Attributes render in SOURCE order, duplicates kept — NO canonicalization
  (probe:q_la1 `[hide_lemma=la1, use_induction, hide_lemma=la3, reuse]`).
  Spellings: `sources`, `reuse`, `use_induction`, `hide_lemma=<name>`,
  `heuristic=<value>` with the goal-ranking value verbatim (`S`, `{mytac}` —
  probes q_la2/q_la3; corpus `{sqn}` etc.).
* The trace-quantifier keyword is ALWAYS on the statement line, never the
  header line, even for `"⊤"` (probe:q_w1 l_top).
* Guarded comment at col 0, directly after the statement (no blank line):
  header `guarded formula characterizing all counter-examples:` for
  all-traces, `… all satisfying traces:` for exists-trace (probe:q_w1). The
  quoted content is OPAQUE pre-computed input (the ported guarded transform's
  rendering — note its style differs from the statement printer: n-ary ∧
  rows, lone `∧`/`⇒` lines, parenthesized ∃-body atoms; never produced by
  this crate).
* Failed conversion variant (probe:q_r1): header
  `conversion to guarded formula failed:` followed by the transform's error
  text with every line indented +2 (the raw error spelling, observed
  unindented in the fatal-restriction case, gains exactly 2 columns here).
* Tail: `by sorry` at col 0 when the source had no proof; a source-embedded
  proof re-renders after the `*/` instead (target:Yubikey `induction … qed`)
  — that proof text is PORTED-renderer output, carried verbatim as input.
* Restrictions outside the guarded fragment are a FATAL load error
  (probe:q_p1 raw), so the restriction renderer never sees them; lemmas
  outside it load fine and take the failed-conversion comment (probe:q_r1).

## Theory frame (R5 GAP-3)

Whole-echo assembly `theory <name> begin … end`. Curated byte targets: the
round1-3 `targets/*.hs.txt` whole captures; 29 diverse files
(tests/round5_theory.rs `whole_echo_frame_parity`) byte-verified end-to-end
(builtins variety, macros, rules ± variants, loop breakers, SAPIC process
attrs, restrictions ± safety ± expanded, lemmas all/exists ± attrs ± embedded
proof, predicates, heuristic, tactic, section comments).

* **Header / footer.** Always `theory NAME` then a blank, `begin`, a blank
  (all 69 surveyed captures). No `configuration` string observed. Close: the
  extracted echo ALWAYS ends with exactly THREE blank lines then `end` (all 44
  surveyed captures). Those blanks are the residue the gate extraction leaves
  after dropping the two trailing comment blocks that are OUT of this crate's
  span — the wellformedness report and the `Generated from:` stamp — each a
  single blank-separated slot, plus the blank before `end` (RAW tail of
  target:MacroInLemmasAndRestrictions verified: `by sorry` · blank · wf-line ·
  blank · Generated-block · blank · `end`).
* **Signature first.** The signature block (`// Function signature …` + blank +
  `builtins:`/`functions:`/`equations:`) is ALWAYS the first thing after
  `begin`, even ahead of `tactic:` blocks (target:5G_AKA) and `heuristic:`
  (target:contract). It is rendered from the `Signature`, never a theory item.
* **Item order + spacing.** All other blocks are theory items in SOURCE order;
  the frame stacks them with ONE blank line between successive items. Item
  types observed: `macros:`, `predicate:` (grouped), rules, restrictions,
  lemmas, `heuristic:`, `tactic:`, `section{* … *}` formal comments, and the
  theory-level `/* looping facts with injective instances: … */` note.
  Assembly = `join(["theory NAME", "begin", sig, items…], "\n\n") + "\n\n\n\nend"`.
* **`heuristic:` line.** A single `heuristic: <value>` item (targets contract,
  ct, running-example, accountability — value e.g. `p`, `o "oracle" …`).
* **`predicate:` block.** One `predicate: <fact><=><formula>` per predicate; a
  contiguous run renders as one item, blank-line separated. NO spaces around
  `<=>`; the head is the R2 fact (`True( x )`), the body the R3 formula. The
  body wraps at ABSOLUTE MARGIN 0 (column-1 nest), INDEPENDENT of the header
  width (target:dmn-basic — `Sender_duplicate` and `Mixer_duplicate` bodies
  both wrap at col 1 despite different name lengths, and a 66-column body row
  fits ribbon 73 measured from col 1, not from the `<=>` column). A
  `<>`-beside composition would instead indent the body under `<=>` — the
  sanctioned HughesPJ `display` threads the current line width into each
  `NilAbove` continuation (`lay2 k (NilAbove p) = nl `txt` lay k p`;
  pretty-1.1.3.6 confirmed) — so the renderer splices the header textually onto
  the margin-0 formula's first line instead of `<>`-ing it.
* **Opaque / verbatim items.** `tactic:` (the whole contiguous region — sub-
  tactics `presort:`/`prio:` carry NO blank-line separators, so the region is
  one block; target:5G_AKA/alethea), `section{* … *}` formal comments
  (target:Joux/Yubikey/contract), and top-level `/* … */` comments (looping-
  facts note) are carried through the frame VERBATIM — their interior layout is
  outside this crate's erasure surface, like guarded/proof text.

## UNOBSERVABLE (recorded per protocol, not guessed as pinned behavior)

* PatMatch (`=x` pattern-match marker, sapic): cannot be forced through the
  MSR no-prove echo. Code renders `=` + term as a placeholder, flagged to
  this entry; must be pinned before any sapic-surface parity is claimed.
* SortHint::Suffix(s): the echo only ever shows sigil form (source `x:fresh`
  echoes `~x`, target:cav13), so suffix-tagged vars are rendered with the
  equivalent sigil.
* exp/mult element-level break choice: wide mult probes exceed the
  variant-computation budget (probe:t_mult timed out; 4-way
  probe:t_xorwide too). Union (probe:t_uniwide, round-3 target:alethea `)`
  drop) and Xor (probe:t_xorwide3, 3-way with long names: break after `⊕`
  attached to the preceding element, continuation after the `(`) pin the
  fill construction directly; Mult/NatPlus/exp-chains use the same
  construction by structural analogy — flagged, to be confirmed by the
  full-corpus gate at integration.
* reliable-channel builtin: requires a sapic top-level process (tool refuses
  otherwise — probe:b_reliable-channel raw); its signature contribution is
  unpinned in R1.
* Adapter contract notes: `NatLit`/`Number` carry DIGITS (renderer prefixes
  `%`); `NumberOne` is the DH `one`; `NatOne` is `%1`.
* Atom::LessMset (multiset smaller): no corpus witness and no reachable
  source spelling found (`(<)` absent from the whole corpus); rendered like
  `Less` as a flagged placeholder — must be pinned before any claim.
* Atom::Pred (predicate fact atoms): predicates are expanded upstream of the
  echo in BOTH the restriction statement and the expanded-formula comment
  (probe:q_pred1) and in sapic lemmas (corpus timepoints.spthy), so a Pred
  atom never reaches the no-prove renderer; rendered as a bare fact,
  flagged.
* Diff-mode surfaces (`--diff`): diffLemma items, lemma/restriction
  `left`/`right` attributes, `diff_reuse`. The parity corpus EXCLUDES all
  --diff files (RS cannot load them), so these never reach the gate; not
  modeled in the crate AST. A later round must pin them before any
  diff-file parity is claimed.
* Lemma modulo annotation (`lemma (modulo E) …`): never appears in any
  round-3 capture (grep across all 20); not modeled.
* Lemma attributes beyond the observed five (`output=…`, `Hint`, prover
  internals): no corpus/probe witness under the no-prove echo; not modeled.
* Fact annotations (`[+]`/`[-]`/`[no_precomp]`) on ACTION atoms inside
  formulas: never observed in any formula echo; the test parser rejects
  them loudly if they ever appear.
* ⇔ inside a safety-relevant restriction: no witness (⇔ restrictions are
  rare-to-absent in the corpus); the classifier treats each ⇔ side as
  occurring in both polarities (the NNF-consistent reading), flagged.
* Guarded-formula content and embedded proof scripts are PORTED-renderer
  output consumed verbatim as input — their interior layout is not a claim
  of this crate (only the comment frame and headers are).
* Restrictions whose formula falls outside the guarded fragment: fatal
  upstream error (probe:q_p1), unreachable for the renderer.
