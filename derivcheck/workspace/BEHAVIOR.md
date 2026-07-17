# BEHAVIOR.md — Unit G: Message Derivation Checks

All statements below are derived from black-box oracle observation (see QUERIES.log),
the captured example outputs, or the SPEC. No tamarin-prover source was read.

## 1. What the check is

For each protocol rule, the prover's solver decides whether every variable of the rule is
**derivable by the intruder from the rule's premises**. Variables that are NOT derivable are
reported under the wellformedness topic `Message Derivation Checks`, warning that the rule may
be "performing unintended pattern matching" on structure the adversary cannot freely produce.

The derivability decision itself is the prover's solver and is NOT reimplemented here — it is a
caller-supplied callback (see §7). This unit owns: rule/variable selection (probe construction),
the decision logic around callback outcomes, and byte-exact report text.

## 2. Report position and topic

- Topic string (exact): `Message Derivation Checks`.
- Underline: a run of `=` exactly as long as the title (25 chars).
- Order among wellformedness topics: emitted **last**, after `Subterm Convergence Warning`
  (observed order: "Facts occur in the left-hand-side but not in any right-hand-side" →
  "Subterm Convergence Warning" → "Message Derivation Checks").
- Driver phase-log lines (NOT part of the report): `[Theory <name>] Derivation checks started`
  and `... ended`. In `--diff` mode intermediate `[Saturating Sources]` lines appear between
  them. These are emitted by the driver around the check; this unit does not render them.

## 3. Byte-exact block format

Single failing rule:
```
Message Derivation Checks
=========================

  The variables of the following rule(s) are not derivable from their premises, you may be performing unintended pattern matching.

Rule R: 
Failed to derive Variable(s): w
```
Two failing rules (uniform blank-line separation between intro and each rule block):
```
Message Derivation Checks
=========================

  The variables of the following rule(s) are not derivable from their premises, you may be performing unintended pattern matching.

Rule reSign: 
Failed to derive Variable(s): m, r2, sk2

Rule RP_gets_idToken: 
Failed to derive Variable(s): pkA
```
Exact pieces:
- Header: `"Message Derivation Checks\n" + "="*25 + "\n"`.
- Blank line after header.
- Intro (2-space indent, no trailing space):
  `"  The variables of the following rule(s) are not derivable from their premises, you may be performing unintended pattern matching."`
- One block per rule: `"Rule " + name + ": \n"` (NOTE the trailing space after the colon)
  then `"Failed to derive Variable(s): " + vars` where `vars` is the sorted list joined by `", "`.
- Header, intro, and each rule block are separated by a single blank line, i.e. the message
  segments `[intro, rule1, rule2, …]` are joined by `"\n\n"` and appended after `header + "\n"`.

### 3a. Output contract — one value, heading included (R4 GAP1, INTEROP)
The consuming report renderer joins topic blocks with one blank line (`"\n\n"`) and adds NO
per-topic heading of its own (R4: the reference prints each topic's underlined heading as part
of that topic's block; blocks are separated by exactly one blank line — observed between the
`Subterm Convergence Warning` block and the `Message Derivation Checks` block). Therefore this
unit must emit the WHOLE topic as ONE value (not one-per-rule fragments) whose text is the
byte-exact block INCLUDING the `Message Derivation Checks` heading + 25-`=` underline. The
complete two-rule block is 297 bytes (no trailing newline); captured in
`probes4/fixture_two_rule.txt`.

## 4. Which RULES are probed / reported

- Only theory `Rule` items (protocol rules); intruder rules are not user rules and are out of scope.
- A rule carrying the `[no_derivcheck]` attribute (`RuleAttr::NoDerivCheck`) is skipped entirely.
- Rules are reported in **theory (source) order** (o3: `Zebra` before `Apple`).
- A rule with zero non-derivable variables produces no block (s4: `Good` omitted).
- Rule name is printed verbatim (case preserved), without any `(modulo E)` qualifier.
- If no rule has a non-derivable variable, the topic is absent entirely.

## 5. Which VARIABLES are reported (solver-decided; observed semantics)

The candidate set this unit hands the callback is the set of variables occurring syntactically in
the (macro/let-EXPANDED, §9) rule's premises, actions, and conclusions; the callback returns
derivability. Observed callback verdicts (NOT reimplemented here):
- bare variable in `In(x)` → derivable (p1).
- variable inside a one-way / private function application in `In(h(x))`, `In(skf(x))` →
  NOT derivable (p6, d2).
- variables inside an invertible pair `In(<a,b>)` → derivable (p4).
- fresh variable bound by a `Fr()` premise → derivable (p3).
- fresh variable occurring only inside an `In` term `In(h(~n))` → NOT derivable (o2).
- public variable `$p` → always derivable, never flagged (o2; R4 g3_pub reconfirms).
- variable bound by a non-`In` state-fact premise `St(x)` → derivable (s3).
- a variable occurring multiple times is listed once (d1: de-duplicated).

### 5a. Candidate SCOPE — which variable classes can EVER be flagged (R4 GAP3)
- Candidate set = every FREE variable of the expanded rule (premises + actions + conclusions),
  not only In-fact / premise variables:
  - action-only variable → candidate (R4 g3_act: `[Fr(~n)]--[Ev(z)]->[Out(~n)]` flags `z`).
  - conclusion-only variable → candidate (R4 g3_concl: `[Fr(~n)]-->[Store(~n,k)]` flags `k`).
  (Both also independently trigger the separate "Unbound variables" wf topic; that topic is a
  different unit's concern.)
- Public-sort (`$p`) → in the candidate set but the solver always resolves it derivable, so it
  is NEVER flagged (o2, g3_pub). This unit does NOT pre-filter public — the solver decides.
- Natural-number sort (`%x`) → CAN be flagged, rendered `%x` (R4 g3_nat, builtins
  natural-numbers). Nat participates in ordering at sort-rank 2 (§6).
- Temporal / node variables → cannot appear in a message-term position at all: `In(h(#i))` is a
  PARSE error (R4 g3_temporal). They are therefore never candidates.
- A name that matches a user-declared nullary function is a CONSTANT, not a variable, and is
  never flagged (R4 g3_nullary `functions: c/0; In(h(c))` → no warning; g3_nullary_ctrl with
  `c` undeclared → flags `c`). Consequence for this unit: candidate enumeration collects only
  `Term::Var` nodes; a nullary function parses to `Term::App(name, [])`, contributing no
  variable — so this is handled correctly by the term walk, given a signature-aware parser.

## 6. Variable ordering (this unit's responsibility) — CORRECTED in R4

Reported ascending by the total order key **`(index, sort_rank, name)`** — index is PRIMARY.
(Round-1..3 recorded `(sort_rank, name, index)`; R4 discriminating probes refute that: the
earlier fixtures used index 0 throughout so they could not tell idx-first from name-first.)
1. `index`: **numeric** comparison, PRIMARY (R4 g2_freshidx: `In(h(~a.2)),In(h(b))` → `b, ~a.2`
   — the idx-0 message var precedes the idx-2 fresh var, so index outranks sort; g2_mixed
   `~c, b, a.2, b.2, a.10`; s2: `x.1 < x.2 < x.10`).
2. `sort_rank`: **`Fresh(0) < Msg(1) < Nat(2)`** (R4 g2_cross same-idx `~s` before `q`;
   g3_nat_order idx-0 `~c, a, %b`; g3_nat_idx idx-1 name-`n` `~n.1, n.1, %n.1`). Public and
   Node/temporal never appear (see §5). Sort dominates name at equal index.
3. `name`: **byte/ASCII lexicographic**, case-sensitive so uppercase precedes lowercase
   (R4 g2_case: `In(h(<apple,Zebra>))` → `Zebra, apple`, `Z`(90) < `a`(97); g2_nodot:
   `In(h(<x2,x10>))` → `x10, x2`). Rust `str::cmp` (byte order) reproduces this exactly.
Trailing digits WITHOUT a `.` stay in the NAME with index 0 (`v10`, `x2` render verbatim, no
dot; g2_nodot, o1). Only an explicit `.n` yields a nonzero numeric index.
Rendering of each variable: fresh → `~name`, message → `name`, **nat → `%name`**, public →
`$name` (never flagged); a nonzero index appends `.idx` (e.g. `x.2`). This matches the
pretty-printed variable spelling used in the warning.

Full R4 confirmation (g2_strong): `In(h(<z.1,a.2,~z.2,~a.1,m>))` → `m, ~a.1, z.1, ~z.2, a.2`
= idx0{m} · idx1{~a.1(fresh),z.1(msg)} · idx2{~z.2(fresh),a.2(msg)}.

## 7. Activation / timeout

- Controlled by `--derivcheck-timeout=INT` seconds (default 5). The optional argument REQUIRES
  `=`; `-d 1` is mis-parsed (the `1` becomes a file argument).
- `=0` fully DEACTIVATES the check: the `Message Derivation Checks` topic is absent; other wf
  topics are unaffected; the driver phase-log lines still print. In this unit, a timeout budget
  of 0 ⇒ empty report, short-circuiting before any callback.
- `=1..5`: the check runs; on all sane inputs the derivation computation is fast (even an 8-term
  xor input finishes the check quickly — the observed slowness there is later precomputation,
  not the check). A per-rule timeout could therefore not be forced with practical inputs.

### Residual (unobserved): per-rule timeout output text
Because the check is robustly fast, the exact behavior when a single rule's derivation check
exceeds the budget was not observable. The callback trait exposes a `TimedOut` outcome and this
unit's decision logic treats it **fail-open**: a timed-out variable/rule yields NO warning for
that rule. Rationale: the only timeout-related observation (`=0` ⇒ no warnings at all) shows the
tool suppresses output when the check cannot complete. This is documented as a residual; the
alternative policy (emit the partial/timed-out rule) is isolated to one match arm and one flag.

## 8. Count interaction (informational)

The driver's `WARNING: N wellformedness check failed!` counts distinct failing TOPICS
(POIDC_CMB: Subterm + Derivation = 2; OIDC_CodeFlow: LHS-facts + Subterm + Derivation = 3).
This whole topic contributes 1, regardless of how many rules/variables it lists. Counting is the
wellformedness driver's concern, not this unit's.

## 9. Macro / let pre-expansion (R4 GAP4)

Derivability is decided on the **fully macro- and let-EXPANDED rule**, and the reported variable
names are the POST-expansion (inner) names:
- `let y = h(w) in [ In(y) ] --> ...` → flags `w` (R4 g4_let). Pre-expansion the rule reads
  `In(y)` with `y` a bare received variable that would be derivable (no warning); the warning
  about `w` proves expansion happens first.
- `let y = w in [ In(h(y)) ] --> ...` → flags `w`, not `y` (R4 g4_let_rename): the let is
  substituted before the check, which sees `In(h(w))`.
- `macros: mac(x) = h(x)` with `[ In(mac(w)) ] --> ...` → flags `w` (R4 g4_macro): macro
  applications are expanded before the check.
- Combined macro+let (`wrap(a)=h(a); let y=w in In(wrap(y))`) → flags `w` (R4 g4_macro_rename).
- The let-bound LHS variable (`y`) is eliminated by expansion and is NEVER itself a candidate
  (R4 g4_let_twice `let y=h(w) in In(g(y))` → only `w`).

This unit therefore builds the candidate set from the rule AFTER (a) expanding macro
applications using the theory-level `macros:` table and (b) substituting the rule's `let`
bindings into its fact terms; the let LHS variables are dropped from the candidate set.

## 10. Solver interface shape (R4 GAP5, INTEROP)

The consuming solver saturates ONCE per rule and answers all of that rule's candidate variables
from the single saturation; asking it per-variable would force wasteful re-saturation. The
callback contract is therefore batched: ONE call per rule carrying all candidate variables and
returning a per-variable verdict vector (aligned to the input order). A per-variable callback is
retained only as a thin adapter over the batch call. This is an interface-shape requirement
derived from the consumer, recorded here as an interop fact (not a black-box oracle observation).
