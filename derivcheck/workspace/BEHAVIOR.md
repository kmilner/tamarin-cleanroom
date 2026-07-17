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

## 4. Which RULES are probed / reported

- Only theory `Rule` items (protocol rules); intruder rules are not user rules and are out of scope.
- A rule carrying the `[no_derivcheck]` attribute (`RuleAttr::NoDerivCheck`) is skipped entirely.
- Rules are reported in **theory (source) order** (o3: `Zebra` before `Apple`).
- A rule with zero non-derivable variables produces no block (s4: `Good` omitted).
- Rule name is printed verbatim (case preserved), without any `(modulo E)` qualifier.
- If no rule has a non-derivable variable, the topic is absent entirely.

## 5. Which VARIABLES are reported (solver-decided; observed semantics)

The candidate set this unit hands the callback is the set of variables occurring syntactically in
the rule; the callback returns derivability. Observed callback verdicts (NOT reimplemented here):
- bare variable in `In(x)` → derivable (p1).
- variable inside a one-way / private function application in `In(h(x))`, `In(skf(x))` →
  NOT derivable (p6, d2).
- variables inside an invertible pair `In(<a,b>)` → derivable (p4).
- fresh variable bound by a `Fr()` premise → derivable (p3).
- fresh variable occurring only inside an `In` term `In(h(~n))` → NOT derivable (o2).
- public variable `$p` → always derivable, never flagged (o2).
- variable bound by a non-`In` state-fact premise `St(x)` → derivable (s3).
- a variable occurring multiple times is listed once (d1: de-duplicated).

## 6. Variable ordering (this unit's responsibility)

Reported ascending by the key `(sort_rank, name, index)`:
1. `sort_rank`: observed `Fresh < Msg` (s1: `~zzz` before `aaa`; o2: `~n` before `x`).
   Public is excluded (never a candidate result); Nat/Node do not occur as message-derivable
   candidates. Sort dominates name (s1 proves this).
2. `name`: **lexicographic string** comparison (o1: `v1 < v10 < v2`; `aa < mm < zz`;
   `m < r2 < sk2`).
3. `index`: **numeric** comparison (s2: `x.1 < x.2 < x.10`).
Rendering of each variable: fresh → `~name`, message → `name`; a nonzero index appends `.idx`
(e.g. `x.2`). This matches the pretty-printed variable spelling used in the warning.

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
