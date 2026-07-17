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
 7b. `Fact capitalization issues`                        (round3; before arity)
 8. `Fact arity issues`                                  (non-diff; diff merges these
 9. `Fact multiplicity issues`                            two into "Fact usage" - OUT OF SCOPE)
10. `Facts occur in the left-hand-side but not in any right-hand-side ` (TRAILING space)
10a. `Left rule` / `Right rule`                          (round2; DIFF MODE ONLY)
10b. `Quantifier sorts`                                  (round4; per-item bundle)
11. `Formula terms`
12. ` Formula guardedness`   (LEADING space)
    NOTE (round4): topics 10b/11/12 are NOT globally fixed - they are emitted
    per formula item (see "Formula-check bundle" below); their relative order
    depends on which item emits which topic first.
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
item i: "    {i}. {Label} `{owner}', arity {n}\n         {render}"; items joined
"\n    \n". Rules and LEMMAS both contribute (interleaved in theory-item order);
restrictions do NOT (r3_restrarity). Rule facts: Label=`Rule`, render=fact pp.
Lemma action facts: Label=`Lemma`, render = raw Haskell `Fact {factTag =
ProtoFact {Linear|Persistent} "{name}" {arity}, factAnnotations = fromList [],
factTerms = [{de-Bruijn terms}]}` (r3_lemarity). Rule items dedup per
(label,owner,arity); lemma facts likewise. Multiplicity is analogous and ALSO
gathers lemma facts (r3_lemmult, item = `... multiplicity (persistence) ...`).

### Fact multiplicity issues
Same structure keyed on multiplicity; header:
    "Same fact is used with different multiplicities, i.e., !Fact() (Persistent fact) exists along with Fact() (Linear) in your rules. \nCheck the multiplicity (persistence) of your facts.\n  " + factBlocks
item i: "    {i}. {Label} `{owner}', multiplicity (persistence) {Linear|Persistent}\n         {render}".

### Fact capitalization issues (round3; precedes Fact arity issues)
Trigger: two facts whose names are equal under ASCII-lowercasing but differ in
exact spelling (e.g. `Send` vs `SEND`; facts must start upper-case, so the
difference is in non-first letters). Same block shape as Fact arity, header:
    "Fact names are case-sensitive, different capitalizations are considered as different facts, i.e., Fact() is different from FAct(). \nCheck the capitalization of your fact names.\n  " + factBlocks
Unlike arity, EVERY occurrence is listed (NO per-(rule,cap) dedup - `Send` twice
in one rule -> two items; r3_capord). item i:
    "    {i}. Rule `{rule}', capitalization \"{exactName}\"\n         {factPP}".
Groups keyed by lowercased name, sorted; a group needs >=2 distinct spellings.
Gathers rule facts across premises/actions/conclusions (source order).

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
constants rendered by the term printer. The list is a fillSep at width 69 with a
4-space continuation indent, begun after the header `  rule `{name}': fresh
public constants are not allowed:` (round3: r3_freshwrap; identical mechanism to
the Formula-terms list). CLOSED - was a documented gap.

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

### Quantifier sorts  (round4)
Trigger: a lemma/restriction formula quantifies over a variable whose sort is
PUB or FRESH (message/temporal/nat quantifiers are allowed - probes
qs_pub_prefix/qs_fresh_prefix/qs_nat_prefix/qs_msg_plain). Per item:
    "  {Entity} `{name}' uses quantifiers with wrong sort: {tokens}"
Entity = `Lemma`/`Restriction` (both, probe qs_restr). Each offending variable
is a raw Haskell tuple token `("{name}",LSort{Pub|Fresh})`, collected in binding
order (outer binders before inner), rendered as a fillSep at width 69 with a
4-space continuation indent (same mechanism as Formula terms; probe qs_multi).
Multiple wrong-sort items join as separate emissions in the bundle (consecutive
ones merge with `\n  \n`; probe qs_twolem). This topic is emitted BEFORE Formula
terms within a single item (probe syn_pubsuffix).

### Formula-variable sorts and sort-aware binding  (round4 - supersedes round2)
Formula quantifiers bind SORTED variables. A bound variable and a use are the
same variable only when they share a name AND a sort CLASS, so a temporal binder
`#x` does NOT bind a message-position use `x` (probe g1_core: `Free x`); a
message binder `x` does not bind a node use `#x` (g1_msgnode: `Free #x`). Sort
classes: {Msg, Untagged, Suffix(Msg)} collapse to Msg (this keeps a quantified
message var bound to its uses across the parser's Untagged-vs-Msg tagging - the
round2 false-positive fix, now done sort-aware instead of name-only);
Pub/Fresh/Node/Nat (and their Suffix forms) are each their own class. The de
Bruijn stack counts EVERY quantified var (temporals and sort-mismatched ones
included); the index is the distance to the innermost binder matching (name,
class) - probe g1_dbsort: `All x #x. ... h(x)` -> `h(Bound 1)` (msg x sits under
node #x). Same-name/different-sort binders coexist without capture (g1_samename:
`Ex x #x. A(x)@#x` -> SUCCESS).

### Formula terms  (round3: FULL coverage; round4: sort-aware)
Trigger: a lemma/restriction formula uses a TERM "of the wrong form". The unit
checked is each ARGUMENT TERM of each atom (Eq/Less/.. -> both sides; Action ->
the TEMPORAL first, then the fact args; Pred/Last -> args). A term is wrong iff
it contains a FREE variable OR a REDUCIBLE function symbol; the WHOLE top-level
term is then reported (not the offending subterm). Body:
    "  {Entity} `{name}' uses terms of the wrong form: {termlist}\n  \n" + fixed_help
Entity = `Lemma` or `Restriction`. Terms are collected in source order and are
NOT deduplicated (`x=y & x=y` -> `Free y', `Free y'`). Reducibility is decided by
the CALLER, which supplies a set of reducible function-symbol names; the checker
only consumes it (entry point `formula_terms_reducible(thy, &reducible)`;
`formula_terms(thy)` = the empty-set convenience wrapper - free variables only).

Raw term rendering (each term wrapped in `` `...' ``):
  - bound variable   -> `Bound N`   (de Bruijn: 0 = innermost binder; EVERY
                                      quantified var, incl. temporals, pushes one)
  - free variable    -> `Free <pp_var>`  (keeps the sort prefix, e.g. `Free #j`)
  - function app      -> `f(a,b)`   (args comma-joined, NO space)
  - tuple             -> `pair(a,pair(b,c))`  (right-nested binary pairs)
  - public constant   -> `'name'`
De Bruijn evidence: `All x y #i. h(x,y)` -> `h(Bound 2,Bound 1)`; nesting adds
depth (`All x #i.(Ex y #j. h(x))` -> `h(Bound 3)`). Free-inside-term evidence:
`f(y)` (f non-reducible, y free) -> `f(Free y)`; `f(h(x))` -> `f(h(Bound 1))`.

Term-list layout is a fillSep at width 69 with a 4-space continuation indent,
begun after the header (`  {Entity} `{name}' uses terms of the wrong form:`): a
token (term + trailing comma, except the last) stays on the line while
`col + 1 + width <= 69`, else wraps. A long entity/name can push the entire list
to line 2; an over-wide single term also wraps to line 2. Multiple wrong lemmas:
each full block (header+list+help) is joined by `\n  \n`.
fixed_help = "  The only allowed terms are public constants and bound node and\n  message variables. If you encounter free message variables, then\n  you might have forgotten a #-prefix. Sort prefixes can only be\n  dropped where this is unambiguous. Moreover, reducible function\n  symbols are disallowed."

### Formula guardedness (topic has LEADING space)  (round4: decision procedure)
Trigger: a LEMMA formula not convertible to guarded form. (An unguarded
RESTRICTION is a FATAL error, not a warning -- observed gr_restr/z1; lemmas
only.) Body:
    "  Lemma `{name}' cannot be converted to a guarded formula:\n    {reason}\n      \"<sub>\"\n    in the formula\n      \"<whole>\""
where <sub> is the FAILING QUANTIFIER SUBTREE and <whole> is the full lemma
formula. Exactly two reasons:
  - `unguarded variable(s) '<v1>', '<v2>' in the subformula` (vars in binding
    order, ", " joined, each `'<pp_var>'`; probes ge_unguard/qs_multi)
  - `universal quantifier without toplevel implication`

DECISION PROCEDURE (pre-order; first failure reported, antecedent before
consequent, left before right - probes gr_both/gr_sib):
  - `∀ vars. body`: body MUST be a top-level implication `guard ==> rest`;
    otherwise "without toplevel implication" (bare atom, conjunction,
    disjunction, negation, iff ALL fail this way - gu_bareatom/gu_conj/gu_disj/
    gu_neg/gu_iff). If it is `guard ==> rest`, every quantified var must be in
    the guard set, else "unguarded variable(s)"; then recurse into guard then
    rest.
  - `∃ vars. body`: every quantified var must be in the guard set of `body`,
    else "unguarded variable(s)" (a non-conjunction body - disjunction/
    implication/negation - guards nothing: ge_disj/ge_impl/ge_neg); then recurse
    into body. A bare action atom or a conjunction of them is a valid guard
    (ge_bareact/ge_conj_ok).
  - GUARD SET of a formula = the (name, sort-class) keys of variables occurring
    in ACTION atoms reachable through CONJUNCTIONS only. Disjunction, negation,
    implication, equality/`<`/`⋖`/`⊏`/`last` atoms, and nested quantifiers
    contribute NO guards (gx_disj_ant/gx_neg_ant/gx_eq_ant/gx_less/gx_last/
    gx_quant_g). An action atom guards all its fact-arg vars AND its temporal
    (gx_pair). Guard matching is sort-aware (syn_pubsuffix: pub `$x` is NOT
    guarded by a msg `x` in `A(x)`). This is the only semantic change from the
    round-3 heuristic (which recursed guards through Or/Not/quantifiers).
  - `¬`, `∧`, `∨`, `⇒`, `⇔` at the top: recurse into operands (left first).
  - Any non-quantified formula: no failure (gt_noquant/gt_topconj).

MULTI-LINE FORMULA PRINTER (pp_formula_wrapped, col-relative; round4 fix).
Page width 72 (single line fits at total col 72, breaks by 74). The formula is
embedded as `      "..."`, so its first char is at column 7 (`pp_formula_wrapped(
f, 7)`). Continuation lines break relative to the enclosing group's START column
`col`, not a fixed base (round3's base+hang was off by one at the top level -
gt_and_two_all):
  - Action/Last/Pred atom -> single-line text (never breaks).
  - relational atom `a OP b` (=,<,⋖,⊏): breaks AFTER the operator; the right
    term hangs at `col+0`; the term operands are NOT parenthesised (gnest).
  - logical binary op `(a) OP (b)`: breaks after OP; right operand hangs at
    `col+0` (the group's start column, i.e. just after any enclosing `(`).
  - `¬` -> `¬` beside a parenthesised operand (never breaks at the ¬).
  - quantifier `Q vars. body`: breaks after the `.`; body hangs at `col+1`
    (measured gnest: a nested quantifier at col 9 -> body at col 10).
When a formula fits on one line the printer is byte-identical to the single-line
`pp_formula`, so narrow-formula fixtures (p21/r3_guard_*) are unaffected.
GAP: the printer does NOT do capture-avoiding alpha-renaming; when a bound var
name collides with a free var of the same name the oracle renames the bound one
(`$x.1` in syn_pubsuffix). Fixtures avoid such collisions (guardedness fixtures
use distinct names, and the Quantifier-sorts fixtures use consistently-bound
prefix vars so no free collision arises).

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

## Formula-check bundle assembly (round4)
The three per-formula checks - Quantifier sorts, Formula terms, Formula
guardedness - are NOT independent global checks. They are run item-by-item:
  - PROCESSING ORDER: every LEMMA (source order) first, then every RESTRICTION
    (source order) - lemmas precede restrictions even when a restriction is
    source-first (probes qs_restr, ord_rl_ft2).
  - Per item the topics are emitted in the SUB-ORDER [Quantifier sorts, Formula
    terms, Formula guardedness] (guardedness for lemmas only) - probe
    syn_pubsuffix.
  - The emission sequence of (topic, entry) is then rendered by MERGING ONLY
    CONSECUTIVE same-topic entries under one header (separator `\n  \n`). A
    topic that recurs after an intervening different topic starts a FRESH block
    with its own underline (probe ord_qs_ft_qs: QS,FT,QS renders THREE blocks;
    qs_twolem: two adjacent QS merge into one).
  - The bundle occupies the report slot after diff Left/Right and before Lemma
    annotations; `Lemma annotations` is a SEPARATE global check AFTER the bundle,
    NOT part of the per-item sub-order (probe ord_la_ft2: FT of a later lemma
    precedes Lemma annotations of an earlier one). Implemented as
    `checks::formula_reports(thy, reducible)`.

## Formula variable identity (round2 fix -> superseded by round4 sort classes)
The round2 fix bound formula variables by NAME only to remove a spurious
"Formula terms"/" Formula guardedness" report on round2/exists_trace_reuse. Name-
only OVER-binds: it wrongly binds a temporal `#x` to a message use `x`
(g1_core). Round4 replaces it with sort-CLASS matching (see "Formula-variable
sorts and sort-aware binding"): {Untagged, Msg} collapse to one class (fixing the
round2 case) while `#x`/`$x`/`~x`/`%x` are distinct classes (fixing g1_core).
exists_trace_reuse still reports only "Lemma annotations".

## Round 3 (Unit C) - closed and residual

CLOSED with byte-parity fixtures:
- Formula terms FULL coverage: reducible functions + de Bruijn `Bound N`, free
  vars inside terms, no-dedup source order, fillSep(69) list wrap, multi-lemma
  `\n  \n` separator (`formula_terms_reducible` / `check_theory_with_reducible`).
- Fact capitalization issues topic.
- Lemma-sourced fact arity/multiplicity (raw Haskell `Fact {..}` render).
- Fresh-const list wrapping; multi-group public-names separator.
- Guardedness: nested-subformula selection, the "universal quantifier without
  toplevel implication" mode, and the multi-line formula printer (implies-break
  and And-break byte-exact).

## Round 4 (Unit C) - closed and residual

CLOSED with byte-parity fixtures (the two round-3 gaps that blocked full
post-elaboration replacement):
- GAP 1 - sort-aware quantifier binding: the wrong-form-terms de Bruijn stack
  now matches (name, sort-class); temporal/pub/fresh/nat binders no longer bind
  message uses. NEW topic "Quantifier sorts" (pub/fresh quantifiers) implemented
  with the fillSep list, Lemma/Restriction entities, and per-item ordering.
- GAP 2 - semantic guardedness: guard extraction is now a decision procedure
  (action atoms through conjunctions only; sort-aware guard matching), replacing
  the round-3 over-approximation that recursed guards through Or/Not/quantifiers.
  Universal/existential sub-modes, recursion order, and lemma-only scope all
  byte-exact.
- Formula-check bundle assembly (lemmas-then-restrictions, per-item sub-order,
  consecutive-merge) and the col-relative multi-line printer (top-level binop
  break and relational-atom break fixed).

## Out-of-scope / known gaps
- Message Derivation Checks & Derivation Checks: computed by Maude after
  translation; not derivable from the AST. Not produced by check_theory.
- AC operator argument ordering (xor/multiset) needs the term ordering; the raw
  Formula-terms show of BinOp/AlgApp operators is a best-effort name guess.
- RULE-PRINTER wide wrapping (RESIDUAL, item 4b): a wide rule wraps to a
  multi-line form - `[` / facts-fillSep / `]` for a broken fact list, `-->` on
  its own line (indent 6), header at 4 - and in the Multiplication context is
  co-printed with the out-of-scope Maude derivation reprint. The single-line rule
  printer stays byte-exact for short rules (all current fixtures). Structure
  recorded from r3_rulewrap; a dedicated rule-layout engine is future work.
- FORMULA-PRINTER capture-avoidance: the oracle renames a bound var that collides
  with a same-name free var (`$x.1`); the clean printer does not. Fixtures avoid
  collisions. Also: quantifier HEAD wrapping (very long variable lists) and
  internal fact-argument wrapping are unprobed (fixtures stay within widths).
- Predicate atoms in formulas: the oracle expands `predicates:` before the wf
  check (gx_pred_grd renders `Pr(x)` as its definition `x = x`), so Pred atoms
  are not expected to reach the checker; guard extraction treats them as
  non-guarding defensively.
- Nat-sort inference / subterm-convergence decision: structural scaffolding only.
