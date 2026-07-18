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
Each lowercased-key group -> one item "  N. " + occurrences. ROUND5 (issue527
target): each DISTINCT spelling is attributed to the rule of its FIRST
occurrence only ('second' used in four rules lists just `rule "One"`); the
spellings are ASCII-sorted and consecutive spellings sharing their first rule
merge into one segment: `rule "R1":  name 'Alice', 'alice'`; segments joined
", ": `rule "Four":  name 'firSt', rule "Two":  name 'first'` (the segment
order follows the sorted spellings, not the source rules). Groups are sorted
by their lowercased key. `public_names_report_from_pairs` takes raw
(name, location) pairs in occurrence order.

### Variable with mismatching sorts or capitalization  (round5: index-aware)
Trigger: within one rule, two variables share a base name AND numeric index but
differ in sort or capitalization. The grouping key is (lowercased name, index):
`$x.1` vs `x.2` is SILENT, `$x.1` vs `x.1` reports "1. $x.1, x.1" (probes
s5_idx/s5_idxsame/s5_capdiff). Suffix-sort spellings are the same variable as
their sigil form (`x:pub` = `$x`, silent - s5_suffix) and render with the
sigil (`x:pub` vs `~x` -> "$x, ~x" - s5_suffix2). Fixed header:
    "Possible reasons:\n1. Identifiers are case sensitive, i.e.,'x' and 'X' are considered to be different.\n2. The same holds for sorts:, i.e., '$x', 'x', and '~x' are considered to be different.\n"
per rule: "  rule `A5': \n    1. $s, ~s" (trailing space; numbered variant
groups). Variants are ordered by sort class $ < ~ < msg < % (s5_all4; Node
unobserved, placed last) and by exact name within a class ("$Ab, $aB" -
s5_capord; sort rank beats name order - s5_crossname "$x, ~X"). Groups are
sorted by their key and separated by a 4-space line (s5_groups). Rule entries
join with "\n  \n" (issue527 target).

### Reserved names  (round5: builtin normalization)
Trigger: rule uses a fact with a reserved name in a position. The reserved set
is POSITION-DEPENDENT (observed z9/z11/z12):
  - left-hand-side / right-hand-side: {K, KU, KD}
  - the middle (actions): {K, KU, KD, In, Out, Fr}  (the I/O facts are also
    reserved as actions; on LHS/RHS they are "Special facts" instead)
BUILTIN NORMALIZATION (round5, t5_ku_lhs/t5_ku_all/t5_kd_mid/t5_up_inout +
issue515 target): fact names matching KU/KD/In/Out/Fr CASE-INSENSITIVELY
denote the builtin facts and render canonically - `Ku(x)` reports `!KU( x )`
(persistent), `Kd` -> `!KD( x )`, `OUT`/`FR` -> `Out( x )`/`Fr( x )`; an `IN`
premise is a legal In premise (silent). `K` is single-letter (only the exact
spelling parses). Normalized builtins never produce Fact-capitalization
conflicts (issue515 has Ku/KU/Kd/KD and no such topic).
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
factTerms = [{de-Bruijn terms}]}` (r3_lemarity). ROUND5: items dedup per
(label, owner, arity, RENDER) - the same lemma fact at two binder depths
yields TWO items (`Bound 3,2,1` and `Bound 4,3,2` - t5_lemdup, issue527
target). Multiplicity is analogous and ALSO gathers lemma facts (r3_lemmult,
item = `... multiplicity (persistence) ...`).

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
nothing close). ROUND5: the excluded facts are the normalized builtins
{KU,KD,In,Out,Fr} only - the reserved proto-fact `K` PARTICIPATES on both
sides (issue527 target lists a K-only premise with suggestion `F`; issue515's
K premise is satisfied by a K conclusion). Item numbers are RIGHT-ALIGNED to
the widest index with a 2-space margin: "   1." .. "  10." (t5_align; ble and
mesh targets).

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
Pub/Fresh/Node/Nat (and their Suffix forms) are each their own class. ROUND5:
the variable's numeric INDEX is part of its identity too - a binder `y` does
not bind a use `y.1` (probe g5_idx: `Ex y #i. A(y.1)@#i` -> `Free y.1` and an
unguarded binder; the accountability translation's primed variables `S.1`
depend on this). The de Bruijn stack counts EVERY quantified var (temporals
and sort-mismatched ones included); the index is the distance to the innermost
binder matching (name, index, class) - probe g1_dbsort: `All x #x. ... h(x)`
-> `h(Bound 1)` (msg x sits under node #x). Same-name/different-sort binders
coexist without capture (g1_samename: `Ex x #x. A(x)@#x` -> SUCCESS).

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
consequent, left before right - probes gr_both/gr_sib; ROUND5 revision):
  - QUANTIFIER FUSION: directly nested quantifiers of the SAME kind fuse into
    one binder list BEFORE the check (`∀x.∀#i.φ` = `∀x #i.φ` - g5_a_nest,
    `∃x.∃#i.φ` - g5_e_nest; predicate-expanded timepoints.spthy needs this).
    The report renders the FUSED form for both the failing subformula and the
    whole formula (g5_a_nest_noimpl prints "∀ x #i. A( x ) @ #i"). No
    cross-kind fusion (g5_a_fuse_ex errors; g5_e_fuse_all: a ∀ body guards
    nothing for the outer ∃).
  - `∀ vars. body`: body MUST be a top-level implication `guard ==> rest`;
    otherwise "without toplevel implication" (bare atom, conjunction,
    disjunction, negation, iff ALL fail this way - gu_bareatom/gu_conj/gu_disj/
    gu_neg/gu_iff). If it is `guard ==> rest`, every quantified var must be
    RESOLVED by the guard region (= the antecedent; equalities in the
    consequent do not guard - g5_a_conseq_eq), else "unguarded variable(s)";
    then recurse into guard then rest.
  - `∃ vars. body`: every quantified var must be RESOLVED by the guard region
    (= the whole body), else "unguarded variable(s)"; then recurse into body.
  - GUARD RESOLUTION of a region for quantified vars `vs` (round5 -
    supersedes the round-4 action-only guard set):
      1. every var occurring anywhere inside an ACTION atom reachable through
         CONJUNCTIONS only is resolved (deep args count - g5_e_actdeep; all
         action atoms are collected BEFORE the equality pass regardless of
         their position - g5_e_actorder);
      2. the conjunction-reachable EQUALITY atoms are processed in a SINGLE
         left-to-right pass: if one side of an equality contains NO unresolved
         quantified variables ("clean" - outer-bound and free vars are clean:
         g5_e_eqpair/g5_e_eqfree), all unresolved quantified vars of the OTHER
         side (however deep - g5_e_eqinner, whole tuples at once -
         g5_e_pairboth) become resolved. One pass only: `(z='c') & (w=h(z))`
         succeeds but `(w=h(z)) & (z='c')` leaves w unguarded
         (g5_e_eqchain/g5_e_revchain); side-based, not unification
         (g5_e_unif); both-sides-current fails (g5_e_eqself/g5_e_eqtwo);
         temporal equalities behave identically (g5_e_eqtemp). Equalities can
         guard ∀ vars in the antecedent the same way (g5_a_eqg1/g5_a_eqg2/
         g5_a_eqonly, and the round-4 gx_eq_ant `x=x` rejection is the
         both-sides-current case).
    Disjunction, negation, implication, `<`/`⋖`/`⊏`/`last` atoms, and nested
    quantifiers contribute NOTHING (gx_disj_ant/gx_neg_ant/gx_less/gx_last/
    gx_quant_g/g5_e_disjeq/g5_e_neg_eq/g5_e_lessonly/g5_e_lastonly/
    g5_e_subterm/g5_e_conjnest). Matching is (name, INDEX, sort-class)-aware
    (g5_idx: a binder `y` does not bind or guard `y.1`).
  - `¬`, `∧`, `∨`, `⇒`, `⇔` at the top: recurse into operands (left first).
  - Any non-quantified formula: no failure (gt_noquant/gt_topconj).
  - CORPUS PIN: the reference accepts all 459 lemma + 225 restriction
    formulas of the 71 round-5 gate-diff theories (captures in
    scratchpad/corpus5_out, extracted to wf-clean/tests/corpus5); the decision
    above accepts every one (test corpus5_acceptance).

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

### Subterm Convergence Warning  (round5: decision + layout closed)
Trigger: user equation l=r not subterm convergent. An equation IS convergent
(silent) when its RHS is a subterm of the LHS OR GROUND - no variables
(t5_sub_ground/t5_sub_groundapp; mesh's `aes_ccm_verify(..) = true_val` is
silent). Flagged equations are listed SORTED by their rendered form, not
source order (t5_sub_order/order2/order3; ble f4<f6<g2, mesh k1..k4<s1). Body:
    "  User-defined equations must be convergent and have the finite variant property. The following equations are not subterm convergent. If you are sure that the set of equations is nevertheless convergent and has the finite variant property, you can ignore this warning and continue \n\n" + eqs + "\n   \n For more information, please refer to the manual : https://tamarin-prover.com/manual/master/book/010_modeling-issues.html "
each eq "    {lhsPP} = {rhsPP}" on one line while its total width <= 71
(t5_wl55..66); wider equations put the LHS on its own 4-indented line and the
RHS on the next as "  = {rhs}" with the RHS laid out by the EQUATION LAYOUT
ENGINE (below). Entries joined "\n".

EQUATION LAYOUT ENGINE (pp_equation; calibrated on t5_tup*/t5_last* and the
ble/mesh reference blocks, all byte-exact):
  - Every fit check compares an end column against min(100, 67 + N) where N is
    the NEST of the current line - the continuation column that started it
    (2 for the "  = " line, else the fill column that broke).
  - A term is laid FLAT when its flat width fits from the current column.
  - Function application `f(a1, a2)`: when broken, args fill at the column
    after `f(`; a fitting arg is placed flat with `, ` between args mid-line
    but a bare `,` glued at a break (mesh: line ends "n),"); a non-fitting arg
    breaks to the continuation column and lays out in full. The closing `)`
    ALWAYS glues.
  - Tuple `<e1, ..., en>`: elements fill one column after `<`; each mid
    element carries its `", "` suffix (kept at the line end when the next
    element breaks - trailing space); a non-fitting element breaks to the
    continuation column (the FIRST element too, leaving `<` alone - mesh);
    an element following a MULTI-LINE element always breaks. The closing `>`
    glues after a single-line last element that fits, and otherwise drops to
    the TUPLE'S START column on its own line (t5_last36/t5_last37; mesh outer
    `>` at the start column after a multi-line last element).

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

## Round 5 (gate-diff families) - closed

Driven by the 71-theory wf-gate diff list (round5/wf_gate_diffs.txt) with the
Haskell reference blocks pre-materialized in round5/targets/. All facts above
tagged "round5" trace to probes in scratchpad/probes5 (QUERIES.log) or to
those targets.

- FAMILY 1 (guardedness over-report, 49 files): the decision procedure gained
  EQUALITY-ATOM GUARDS (one-clean-side resolution, single left-to-right pass,
  actions first) and SAME-KIND QUANTIFIER FUSION; binder identity now includes
  the numeric index. Pinned against every lemma/restriction formula the
  reference prints for the 71 theories (tests/corpus5, 684 formulas, all
  accepted) plus byte fixtures for the still-rejected shapes.
- FAMILY 2 (sort/capitalization over-report, 15 files): grouping is
  (lowercased name, index); suffix-sort = sigil sort; variant order = sort
  rank then name; groups sorted with a 4-space separator. Public-names
  listing reworked: distinct spellings attributed to their FIRST rule,
  sorted, consecutive same-rule spellings merged, groups sorted by key.
- FAMILY 3 (tail, 7 files): case-insensitive builtin-fact normalization
  (issue515), K participates in lhs-not-rhs (issue527), right-aligned item
  numbers (ble/mesh), lemma arity/multiplicity items deduped per-render
  (issue527), Lemma-annotations target pinned (Axioms_and_Induction), the
  SAPIC lookup-rule unbound block pinned (t5_lookup vs OCSPS/CT targets), and
  the Subterm Convergence decision (ground RHS), ordering (sorted) and wide-
  equation layout engine - the mesh k2 ten-line block renders byte-exact.

Round-5 report (REPORT5 folded here; .md creation blocked):
- Tests: 130 pass (was 74): +29 round-5 byte-parity/behavior tests
  (tests/round5_tests.rs, fixtures g5_*/s5_*/t5_* captured from oracle probes
  plus 4 reference-target extracts: issue515, issue527, Axioms_and_Induction,
  ble/mesh subterm blocks) and +1 corpus harness
  (tests/corpus5_acceptance.rs: a parser for the oracle's pretty-printed
  Unicode formula syntax runs the formula bundle over all 684 extracted
  formulas and asserts silence).
- The round-4 "equality atoms do NOT guard" fact (gx_eq_ant) was an artifact
  of the `x = x` self-case; the general mechanism is the one-clean-side
  resolution above. The round-4 (name, class) binder identity was likewise
  incomplete: the index is part of identity (g5_idx).
- Axioms_and_Induction / stateverif / OCSPS / CertificateTransparency: the
  sealed checks already produce the reference blocks given a faithful AST
  (pinned by t5_axioms_induction and t5_lookup); their gate divergence is on
  the integration side (which rules/lemmas are fed to the checker), not in
  the check logic.
- INTEGRATOR NOTES: (a) the adapter must carry variable INDICES into
  `VarSpec.idx` (formula binders and uses, and rule variables like `$x.1`) -
  index-blind mapping reintroduces both the family-1 and family-2 false
  positives; (b) `Message Derivation Checks`/`Derivation Checks` and the
  accountability "Accountability (RP check)" block remain out of scope
  (computed elsewhere); (c) public entry points unchanged;
  `pretty::pp_equation` is new (used by the subterm check).

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

---

## Round 6 — final 8 corpus residuals (behavioral facts)

Provenance: probes round6/probes/{lhs1,lhs2,lhs3,pc1,pc2}.spthy and the
round6/targets reference blocks (ble, CentralizedMonitor, issue527, Axioms,
issue515, OCSPS, CertificateTransparency, stateverif_left_right).

### "Facts occur in the left-hand-side but not in any right-hand-side"
- GRANULARITY (probes lhs1/lhs2/lhs3): the check emits ONE entry per premise
  fact OCCURRENCE whose identity (name, arity, multiplicity) never appears on
  ANY right-hand side. There is NO deduplication whatsoever:
  * the same fact reused as an LHS-only premise across N rules -> N entries;
  * a fact repeated inside a single rule's premises -> one entry per copy
    (lhs2: `DD` twice in rule D -> two entries);
  * different arities of the same name are different identities (lhs2: `EE`
    arity 1 and arity 2 both listed).
- ORDER: pure SOURCE ORDER of (rule, premise-position). NOT grouped by fact
  name (lhs1: `AA`@A, `BB`@A, `CC`@B, `AA`@C -> the two `AA` entries are NOT
  adjacent). ble reference confirms: Oracle_f4, Oracle_passkey, then
  RespChooseKeysize x4 (its four contiguous rules), then InitChooseKeysize x4.
- RHS EXCLUSION is by identity and per-occurrence: if a fact identity occurs on
  any RHS, EVERY LHS occurrence of it is suppressed (lhs3: `AA` on RHS of rule
  Z suppresses both LHS uses; only `BB` remains).
- SUGGESTION unchanged: nearest RHS fact by name edit distance <= 3 appended as
  ". Perhaps you want to use the fact <render>" (lhs3: `BB` -> `AA`, dist 2).
- ALIGNMENT: index right-aligned to the widest index with a two-space margin
  (ble 10 entries: "   1." three leading spaces ... "  10." two). Entries
  separated by a line of exactly two spaces.
- SEALED FIX: fact_lhs_occur_no_rhs previously deduplicated by identity
  (name,arity,mult), collapsing ble's four RespChooseKeysize rules to one. The
  dedup is removed; every occurrence is now emitted. Pinned by round6 tests
  ble_lhs_not_rhs_per_rule_entries / lhs_not_rhs_no_dedup_source_order /
  lhs_not_rhs_rhs_identity_suppresses_all_occurrences.

### "Public constants with mismatching capitalization" (pubcap)
Already implemented in public_names_report; round6 pins it independently:
- Each DISTINCT spelling attributed to the rule of its FIRST occurrence.
- Spellings grouped by ASCII-lowercased key; only groups with >=2 distinct
  spellings are reported; groups sorted ascending by that key (pc2: apple<bird).
- Within a group, spellings sorted ASCII ascending ('C'<'c', 'Bird'<'bird',
  'Apple'<'apple', 'firSt'<'first', 'seconD'<'second').
- Consecutive sorted spellings sharing their first-occurrence rule collapse to
  one `rule "R":  name 'a', 'b'` segment; spellings whose first rules differ
  keep separate `rule "R":  name 'x'` segments joined by ", " (pc2 entry 2:
  `rule "R1":  name 'Bird', rule "R2":  name 'bird'`).
- Entries numbered "  N. ", separated by a line of exactly two spaces.
- CentralizedMonitor target = pc1 exactly: single translated rule "Init" with
  'c'/'C' -> `  1. rule "Init":  name 'C', 'c'`.

### issue527 "Variable with mismatching sorts or capitalization" — the seam
- The sealed mismatching_sorts BODY (WfError.message) for T_SORTS begins with
  the four-line preamble (exact bytes, `$` = LF):
    Possible reasons:$
    1. Identifiers are case sensitive, i.e.,'x' and 'X' are considered to be different.$
    2. The same holds for sorts:, i.e., '$x', 'x', and '~x' are considered to be different.$
    $
  then the per-rule numbered groups. The sealed body is byte-identical to the
  reference topic body (issue527_reference_block test). The open-side assembly
  layer must NOT prepend its own "Possible reasons:" paragraph; the sealed
  checker already emits the complete body (see round6/NOTES.md).

### Confirmations for the OUT-OF-SCOPE (open-side) split
- issue527 sealed topics are byte-identical to the target through the last
  sealed topic; the only delta is a single join blank line the open side adds
  before the Maude-computed "Message Derivation Checks" block (out of scope).
- Axioms_and_Induction: the sealed render_report emits BOTH the "Lemma
  annotations" header (via underline_topic) AND the body
  `  Lemma `Exists_test': cannot reuse 'exists-trace' lemmas`. A missing header
  on the open side is an assembly gap, not a sealed-logic gap; no contradiction.
- SAPIC blank1 files (issue515, OCSPS, CertificateTransparency,
  stateverif_left_right): render_block emits exactly one blank line after each
  topic underline and render_report joins topics with one blank line; the
  sealed render carries the blank the open side is said to drop. No
  contradiction with the stated split.
