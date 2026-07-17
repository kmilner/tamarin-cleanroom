# Behavioral spec inferred from the oracle (wellformedness checker)

All facts below are derived from oracle observation (probes in
scratchpad/probes, captures in oracle/captures). Nothing here is taken from
tamarin-prover source.

## Overall report shape

The oracle emits the wellformedness result inside the pretty-printed theory,
wrapped in a block comment.

Success (empty report):

    /* All wellformedness checks were successful. */

Failure (>=1 topic):

    /*
    WARNING: the following wellformedness checks failed!

    <block1>

    <block2>
    ...
    */

Inner text (what `render_report` produces for a non-empty report) is:

    "WARNING: the following wellformedness checks failed!" + "\n\n" + join(blocks, "\n\n")

Each `block` = topic header + "\n" + underline + "\n\n" + body, where
`underline` = '=' repeated char_count(topic) (topic length INCLUDING any
leading/trailing space). Bodies may themselves contain blank lines.

`render_report` for the empty report returns the success line
`/* All wellformedness checks were successful. */`.
`underline_topic(t)` returns `t + "\n" + "=".repeat(char_count(t))`.

## Canonical topic order (report order)

Observed by kitchen-sink probes ks1 / ks3 and the round-2 ordering probes
(comb1, ord_tail, mega_tail, diff_mega, diff_place):

 1. `Unbound variables`
 2. `Fresh public constants`                             (round2; always-on)
 3. `Public constants with mismatching capitalization`   (public-names report)
 4. `Variable with mismatching sorts or capitalization`
 5. `Reserved names`
 5a. `Reserved prefixes`                                 (round2; DIFF MODE ONLY)
 6. `Fr facts must only use a fresh- or a msg-variable`
 7. `Special facts`
 8. `Fact arity issues`                                  (non-diff; diff merges these
 9. `Fact multiplicity issues`                            two into "Fact usage" - OUT OF SCOPE)
10. `Facts occur in the left-hand-side but not in any right-hand-side ` (TRAILING space)
10a. `Left rule` / `Right rule`                          (round2; DIFF MODE ONLY)
11. `Formula terms`
12. ` Formula guardedness`   (LEADING space)
13. `Lemma annotations`                                  (round2; always-on)
14. `Multiplication restriction of rules`                (round2; always-on)
15. `Nat Sorts`
16. `Subterm Convergence Warning`
17. `Message Derivation Checks` (appended by binary; needs Maude - OUT OF SCOPE)
    / `Derivation Checks` (timeout variant - OUT OF SCOPE)

Round-2 ordering evidence: comb1 -> `Unbound, Fresh public constants,
Public names, mismatching sorts, Reserved`; mega_tail -> `Formula terms,
Formula guardedness, Lemma annotations, Multiplication restriction, Nat Sorts,
Subterm`; diff_mega/diff_place -> `... Reserved names, Reserved prefixes,
Fr facts, ... lhs-not-rhs, Left/Right rule, Formula terms, ...`.

Public names (#3) is generated separately and *inserted before* the first of a
set of anchor topics = every topic from #4 (mismatching sorts) onward.
`after_public_names_topics()` returns that anchor list; `insert_wf_before(
report, errs, anchors)` inserts `errs` immediately before the first report
entry whose topic is in `anchors` (appends at end if none present). Unbound
variables (#1) and Fresh public constants (#2) are emitted before the
public-names insertion point (so they are NOT anchors).

### Diff-mode-only vs. always-on

Gated on `theory.is_diff`: `Reserved prefixes`, `Left rule`, `Right rule`
(observed silent in non-diff mode; diff_reserved_prefix without `--diff` emits
nothing). Always-on (both modes): `Fresh public constants`, `Lemma
annotations`, `Multiplication restriction of rules`.

## Per-check triggers and message templates

### Unbound variables
Trigger: a protocol rule uses a variable in actions/conclusions not bound by a
premise. Entries joined by "\n  \n":
    "  rule `NAME' has unbound variables: \n    v1, v2"
(trailing space after "variables:"). Vars sorted, ", " joined, term-printed.

### Public constants with mismatching capitalization (public_names_report)
Trigger: two public constants whose names are equal under ASCII-lowercasing but
differ in capitalization. Body:
    "Identifiers are case-sensitive, mismatched capitalizations are considered as different, i.e., 'ID' is different from 'id'. Check the capitalization of your identifiers.\n\n" + numbered items
Each lowercased-key group -> one item "  N. " + occurrences. Occurrences group by
location; same loc: `rule "R1":  name 'Alice', 'alice'`; different locs joined
", ": `rule "R1":  name 'Server', rule "R2":  name 'server'`. Names ASCII-sorted.
`public_names_report_from_pairs` takes raw (location, name) pairs.

### Variable with mismatching sorts or capitalization
Trigger: within one rule, two variables share a base name but differ in sort
prefix ($x/x/~x) or capitalization. Fixed header:
    "Possible reasons:\n1. Identifiers are case sensitive, i.e.,'x' and 'X' are considered to be different.\n2. The same holds for sorts:, i.e., '$x', 'x', and '~x' are considered to be different.\n"
per rule: "  rule `A5': \n    1. $s, ~s" (trailing space; numbered variant groups).

### Reserved names
Trigger: rule uses a fact with a reserved name in a position. The reserved set
is POSITION-DEPENDENT (observed z9/z11/z12):
  - left-hand-side / right-hand-side: {K, KU, KD}
  - the middle (actions): {K, KU, KD, In, Out, Fr}  (the I/O facts are also
    reserved as actions; on LHS/RHS they are "Special facts" instead)
Per (rule, position) entry joined "\n  \n":
    "  Rule `R1' contains facts with reserved names on left-hand-side:\n    <facts>"
Position phrases: LHS `on left-hand-side:`, actions `on the middle:`,
RHS `on the right-hand-side:`. Facts via fact printer, fsep comma-joined.

### Fr facts must only use a fresh- or a msg-variable
Trigger: `Fr(t)` where t is not fresh/msg var. Entries joined "\n  \n":
    "  rule `R1' fact: Fr( $p )"

### Special facts
Trigger: reserved I/O fact in disallowed position (Out/Fr/K in premises,
In/K in conclusions). Per (rule, side) joined "\n  \n":
    "  rule `R1' uses disallowed facts on left-hand-side:\n    Out( x )"
(sides `on left-hand-side:`/`on right-hand-side:`; lowercase "rule").

### Fact arity issues
Trigger: same fact name with >1 arity. Body:
    "Same fact is used with different arities, i.e., Fact('A','B') is different from Fact('A'). \nCheck the arguments of your facts.\n  " + factBlocks
facts sorted by lowercased name; factBlock (each prefixed "\n"):
    "\n  Fact `foo':\n\n" + items + "\n  "
item i: "    {i}. Rule `R', arity {n}\n         {factPP}"; items joined "\n    \n".
Lemma-sourced facts render as raw Haskell `Fact {factTag = ...}` (gap).

### Fact multiplicity issues
Same structure keyed on multiplicity; header:
    "Same fact is used with different multiplicities, i.e., !Fact() (Persistent fact) exists along with Fact() (Linear) in your rules. \nCheck the multiplicity (persistence) of your facts.\n  " + factBlocks
item i: "    {i}. Rule `R', multiplicity (persistence) {Linear|Persistent}\n         {factPP}".

### Facts occur in the left-hand-side but not in any right-hand-side  (fact_lhs_occur_no_rhs)
Trigger: user fact identity (name,arity,mult) in some premise, never in any
conclusion. Numbered items joined "\n  \n":
    "  1. in rule \"r1\":  factName `X' arity: 0 multiplicity: Linear" +
      optional ". Perhaps you want to use the fact in rule \"r1\":  factName `Y' arity: 0 multiplicity: Linear"
suggestion = nearest RHS fact by edit distance under a threshold (omitted when
nothing close). Special facts excluded from both sides.

### Fresh public constants (round2; always-on)
Trigger: a fresh-name literal (`~'foo'`, AST `Term::FreshLit`) used directly in
a rule. Constants collected in the order premises, conclusions, actions
(probe fpc_positions). Per rule with >=1 hit, entries joined "\n  \n":
    "  rule `NAME': fresh public constants are not allowed: ~'foo', ~'bar'"
constants rendered by the term printer, ", " joined. GAP: for a long constant
list the oracle word-wraps with a 4-space continuation indent (fpc_positions);
implementation emits a single line, so byte-parity fixtures use short lists.

### Reserved prefixes (round2; DIFF MODE ONLY)
Trigger (diff theories only): a fact whose name starts with `DiffIntr` or
`DiffProto`. Facts collected in the order premises, actions, conclusions
(probe rp_multi). Per rule the body is:
    "  The Rule `NAME' contains facts with reserved prefixes ('DiffIntr',\n  'DiffProto') inside names:\n  \n" + factblocks
The HEADER is word-filled (Wadler `fillSep`) at WIDTH 69, indent 2 (measured via
rp_long/rp_med/rp_w47: a line breaks before the next word once column would
exceed 69). Words treat the rule name `\`NAME'` and each of `('DiffIntr',` /
`'DiffProto')` as single tokens. Each factblock (joined "\n  \n"):
    "    <fact_pp>\n    (ProtoFact <Mult> \"<name>\" <arity>,<arity>,<Mult>)"
The 2nd line is a raw Haskell Show of a (FactTag, arity, Multiplicity) tuple;
<Mult> is `Linear`/`Persistent`. GAP: multi-RULE joining unprobed (fixtures use
one rule).

### Left rule / Right rule (round2; DIFF MODE ONLY)
Trigger (diff theories only): a diff rule (`Rule.left_right = Some((l,r))`)
whose explicit `left`/`right` projection differs from the parent rule projected
to that side (`diff(a,b)` -> a on left, -> b on right). Per rule the LEFT
projection is tested first; if inconsistent the RIGHT is not reported for that
rule (probe diff_both: both sides bad -> only "Left rule"). Body per entry
(joined "\n  \n"):
    "  Inconsistent left rule\n" + indent4(explicit_rule_pp) +
    "\n  \n  w.r.t.\n  \n" + indent4(parent_rule_pp)
("left"/"right" and topic "Left rule"/"Right rule"). Consistency compared via
the projected vs. explicit fact lists. Left rule topic precedes Right rule.

### Lemma annotations (round2; always-on)
Trigger: a lemma with attribute `reuse` AND quantifier `exists-trace` (reuse on
`all-traces` is fine - probe la_alltraces). Entries joined "\n  \n":
    "  Lemma `NAME': cannot reuse 'exists-trace' lemmas"
(Topic may host other annotation conflicts not yet observed.)

### Multiplication restriction of rules (round2; always-on)
Trigger: a rule whose CONCLUSIONS contain a multiplication (`*`, AST
`BinOp::Mult`) term. `*` only in premises or only in actions does NOT trigger
(probes mul_multi, mul_act); a `*` in a premise is not listed even when the
conclusion also has one (mul_pc). Per rule, entry (joined "\n  \n"):
    "  The following rule is not multiplication restricted:\n" + indent4(rule_pp) +
    "\n  \n  After replacing reducible function symbols in lhs with variables:\n" + indent4(rule2_pp) +
    "\n  \n    Terms with multiplication:  <terms>"
<terms> = maximal `*`-subterms of the conclusions, ", " joined (mul_terms).
GAPS: (a) rule2 (LHS reducible symbols replaced by fresh vars) is rendered equal
to rule1 - correct only when the LHS has no reducible symbols (probe mul_exp
shows `g^c` -> `x.1` otherwise); (b) the alternate failure mode
"Variables that occur only in rhs:  <vars>" (mul_exp) is not implemented; (c)
the co-emitted `Message Derivation Checks` block is Maude-derived, OUT OF SCOPE;
(d) rule_pp wrapping for long rules (see printer note).

### Formula terms
Trigger: lemma/restriction uses a term that is not a public constant nor a bound
node/message variable (free var or reducible function). Body:
    "  Lemma `L1' uses terms of the wrong form: `Free y'\n  \n" + fixed_help
Short single term -> inline; longer/complex -> indented next line
`    `t1', `t2'` (reducible terms use de Bruijn `Bound N` - gap).
fixed_help = "  The only allowed terms are public constants and bound node and\n  message variables. If you encounter free message variables, then\n  you might have forgotten a #-prefix. Sort prefixes can only be\n  dropped where this is unambiguous. Moreover, reducible function\n  symbols are disallowed."
Entity label `Lemma` or `Restriction`.

### Formula guardedness (topic has LEADING space)
Trigger: a LEMMA formula not convertible to guarded form. (An unguarded
RESTRICTION is instead a FATAL error, not a warning -- observed z1; so this
check applies to lemmas only.) Body:
    "  Lemma `L2' cannot be converted to a guarded formula:\n    unguarded variable(s) '#j' in the subformula\n      \"<pp>\"\n    in the formula\n      \"<pp>\""
<pp> = unicode formula pretty-print (full printer + guardedness algorithm partial - gap).

### Nat Sorts
Trigger: var used in %+ (nat) context not of sort nat. Entries joined "\n  \n":
    "  y in term (y%+z) must be of sort nat" (nat-sort inference - structural only, gap).

### Subterm Convergence Warning
Trigger: user equation l=r not subterm convergent. Body:
    "  User-defined equations must be convergent and have the finite variant property. The following equations are not subterm convergent. If you are sure that the set of equations is nevertheless convergent and has the finite variant property, you can ignore this warning and continue \n\n" + eqs + "\n   \n For more information, please refer to the manual : https://tamarin-prover.com/manual/master/book/010_modeling-issues.html "
each eq "    {lhsPP} = {rhsPP}" (subterm decision + printer partial - gap).

## Term pretty-printer (observed)
- Var: sortprefix + base + ("."+idx if idx>0). Fresh `~`, Pub `$`, Nat `%`,
  Node `#`, Msg/Untagged ``.
- Pair / App("pair",[a,b]): `<a, b, ...>` (right-nested pairs flattened).
- App(f,args): `f(a, b)` (nullary -> `f`).
- PubLit s: `'s'`.
- BinOp: Exp `a^b` (no outer parens); Mult `(a*b)`; Union `(a++b)`;
  Xor `(a(+)b)` U+2295 AC-sorted; NatPlus `(a%+b)`.
- NatOne -> `%1`.
- Fact: [`!` if persistent] + name + "(" + (" "+args.join(", ")+" )" if args else " )").

## Rule pretty-printer (observed; round2)
Used by Left/Right rule and Multiplication restriction. Base form (single line
when short):
    "rule (modulo E) NAME:\n   [ prems ] <arrow> [ concls ]"
- 3-space indent before the fact-list line; fact lists `[ f1, f2 ]`, `[ ]` empty.
- <arrow> = `-->` (no actions) or `--[ acts ]->` (actions present).
- `(modulo E)` (the AST modulo field, default "E").
- When embedded in a warning the whole 2-line block is indented (4 for
  Left/Right and Multiplication, giving 4 and 4+3=7 spaces).
GAP: long rules wrap - the arrow drops to its own line indented 6 and the
conclusions to indent 7 (probe mul_terms). Byte-parity fixtures use short rules.

## Formula variable identity (round2 fix)
For free-variable detection (Formula terms) and guardedness the oracle treats a
quantified message variable and its occurrences as ONE variable even when the
parsed AST tags them with different-but-compatible sorts (Msg vs the Untagged
default; a temporal written `@ i` vs a quantifier `#i`). Matching on the full
sort tag produced spurious "Formula terms"/" Formula guardedness" reports on
round2/exists_trace_reuse (the integration bug). FIX: bind/compare formula
variables by NAME only. All prior fixtures (p05, p21, f_subterm) use consistent
sorts and are unaffected.

## Out-of-scope / known gaps
- Message Derivation Checks & Derivation Checks: computed by Maude after
  translation; not derivable from the AST. Not produced by check_theory.
- AC operator argument ordering (xor/multiset) needs the term ordering.
- de Bruijn `Bound N` term rendering + reducible detection in Formula terms /
  raw lemma-fact rendering in arity/multiplicity: partial.
- Full unicode formula pretty-printer + guardedness algorithm: partial.
- Nat-sort inference / subterm-convergence decision: structural scaffolding only.
