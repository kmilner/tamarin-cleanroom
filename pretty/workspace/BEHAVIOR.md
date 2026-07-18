# BEHAVIOR.md — inferred behavioral spec of the theory echo (R1: term core + signature block; R2: rule blocks)

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
  Layout on overflow: `"(" <> fcat (punctuate op elems) <> ")"` — break
  between elements with the operator attached to the preceding element and no
  fill space (probe:t_uniwide: line ends `…aaaaaaaaaaaaaaaaa3++`,
  continuation aligned after the `(`).
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
* Attribute list attaches to the name: `Name[color=#abcdef, no_derivcheck,
  issapicrule, role='r']:` — canonical order color < no_derivcheck <
  issapicrule < role regardless of source order; the LAST color/role
  declaration wins; `process='…'` and external `x-…` attributes are DROPPED
  (probe:p_rattr, target:issue713). Spellings: `color=#<hex>` unquoted
  (lowercased upstream: '#AbCdEf' → `#abcdef`), `role='…'` quoted.
  Fill-wrap aligned after the `[`, `]:`  attached to the last item
  (probe:p_rattr R3, target:issue713 bla).

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

* `text "macros: " <> sep (punctuate ',' items)` — all-or-nothing: one line
  or EVERY macro on its own line aligned after `macros: ` (probe:p_mac1 —
  m2 gets its own line though it would fit beside m1; target:issue777
  one-line single macro).
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

## UNOBSERVABLE (recorded per protocol, not guessed as pinned behavior)

* PatMatch (`=x` pattern-match marker, sapic): cannot be forced through the
  MSR no-prove echo. Code renders `=` + term as a placeholder, flagged to
  this entry; must be pinned before any sapic-surface parity is claimed.
* SortHint::Suffix(s): the echo only ever shows sigil form (source `x:fresh`
  echoes `~x`, target:cav13), so suffix-tagged vars are rendered with the
  equivalent sigil.
* exp/mult element-level break choice: wide mult probes exceed the
  variant-computation budget (probe:t_mult timed out; 4-way
  probe:t_xorwide too). Union (probe:t_uniwide) and Xor (probe:t_xorwide3,
  3-way with long names: break after `⊕` attached to the preceding element,
  continuation after the `(`) pin the fcat construction directly;
  Mult/NatPlus/exp-chains use the same construction by structural analogy —
  flagged, to be confirmed by the full-corpus gate at integration.
* reliable-channel builtin: requires a sapic top-level process (tool refuses
  otherwise — probe:b_reliable-channel raw); its signature contribution is
  unpinned in R1.
* Adapter contract notes: `NatLit`/`Number` carry DIGITS (renderer prefixes
  `%`); `NumberOne` is the DH `one`; `NatOne` is `%1`.
