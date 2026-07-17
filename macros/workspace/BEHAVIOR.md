# BEHAVIOR.md — Unit E: `.spthy` macro expansion (clean-room)

Every statement below traces to an oracle observation in `QUERIES.log` (tagged
`[Qn]`) or to the interface AST in `../wellformedness/interface/ast_types.rs`.
Nothing here is recalled from tamarin-prover internals.

## 1. Macro definition syntax

```
macros: name1(a1, ..., ak) = body1,  name2(...) = body2,  ...
```

AST: `TheoryItem::Macros(Vec<Macro>)`, `Macro { name, args: Vec<VarSpec>, body: Term }`.

- A block is a comma-separated list of definitions; a theory may contain more
  than one `macros:` block (all are collected in declaration order).
- `args` are the formal parameters (variables), untyped in every observed use.
- **Duplicate formal names** in one macro are rejected: `m(x,x)=...` →
  `"m" have two arguments with the same name` [Q22].
- **Body** is a `Term` parsed against the *current* function signature. It may
  reference builtins / declared `functions:` / **earlier-defined macros only**.
  - Forward reference to a later macro → parse error [Q8].
  - Self reference → parse error (name not in scope in its own body) [Q19].
  - Mutual reference → parse error [Q20].
  - Reference to an undeclared function → parse error [Q24].
  - ⇒ macros form a **DAG** (each body depends only on strictly earlier macros);
    expansion therefore always terminates.
- **Arity 0** is allowed: `z() = body` [Q10]. A body may contain **free
  variables that are not formals**; those are left literally in the expansion
  and get captured by the surrounding scope (e.g. `z()=~x` under `Fr(~x)`).

### Name-collision rules (definition time)
- Macro name == another macro (**duplicate**) → `Conflicting name for macro <n>` [Q13].
- Macro name == an **actively declared** function/builtin →
  `Conflicting name for macro <n>` [Q14] (`h` + `builtins: hashing`).
- Same name with the builtin **absent** is fine (`macros: h(x)=x`, no hashing) [Q25].
  ⇒ a name is a macro XOR a function symbol; never both.

## 2. Call-site expansion semantics

A call `name(t1, ..., tn)` where `name` is a defined macro of arity `k`:

1. **Arity / argument packing (done by the PARSER, before the AST):**
   - `k >= 2`: exactly `k` arguments required; otherwise parse error
     (over [Q16] and under [Q12,Q17] both rejected).
   - `k == 1`: extra comma-args are folded into a right-nested **pair**:
     `m(a,b)` → `m(<a,b>)` [Q11]; `m(a,b,c)` → `m(<a,b,c>)` [Q15].
   - ⇒ **At the AST level every macro call already has arg-count == arity.**
     `expand()` may assume this (and defensively no-ops on a mismatch).

2. **Substitution:** bind each formal to its (already-expanded) argument and
   substitute **simultaneously** (parallel / capture-avoiding). `swap(x,y)=<x,y>`
   called `swap(y,x)` → `<y, x>`, not `<x,x>`/`<y,y>` [Q7].

3. **Formal matching is by FULL variable identity, including sort** [Q27,Q28]:
   a formal `x` (Untagged) matches a body occurrence `x` (Untagged) only.
   `~x` (Fresh) and `$x` (Pub) are *different* variables — they are NOT
   substituted and survive as free vars in the body. An argument whose formal
   never occurs in the body is simply dropped (`m(x)=~x`, `m(a)` → `~x`).

4. **Transitivity:** a body may call earlier macros; expansion is applied to the
   substitution result until no macro calls remain. `a→b→c→<x,x>` chains fully
   [Q9,Q18]. Termination guaranteed by the DAG property (§1).

5. **Bare nullary uses** (macro name written WITHOUT parentheses). A **0-ary**
   macro used as a plain name is a use of that macro, treated identically to the
   parenthesised form: `konst() = h('k')`, then bare `konst` expands to `h('k')`
   [Q32]. Rules:
   - Resolves in every term position — premise, action, conclusion, and nested
     inside functions (`h(konst)` → `h('k')`) [Q32]; and inside macro bodies,
     transitively (`wrap() = h(base)`, bare `wrap` → `h(h('k'))`) [Q33].
   - **Untagged sort only**: `~konst` / `$konst` (fresh/pub) are *not* the macro —
     they stay ordinary variables [Q34] (consistent with the sort-identity rule
     §2.3). A macro-name use cannot be sort-annotated (`konst:msg` is a parse
     error), so a bare use is always the plain untagged form [Q32].
   - **Nullary only**: a bare name equal to an **arity ≥ 1** macro is *not* a use;
     it is left as an ordinary (unbound) variable [Q35].
   - **The macro reserves its name**: when a formal shares a name with an existing
     nullary macro, the nullary macro wins inside the body — bare occurrences
     resolve to the macro, not the formal (`base()=h('k')`, `f(base)=<base,base>`,
     `f(a)` → `<h('k'),h('k')>`; the formal and its argument are dropped) [Q36].
   - **Consumer interface contract:** such a bare use reaches `expand` as a plain
     `Term::Var("konst")` (untagged), *not* an `App`; `expand` resolves it to the
     nullary macro exactly under the conditions above.

## 3. Where expansion is visible vs. preserved

| context                              | macro call | expanded form |
|--------------------------------------|------------|---------------|
| `--parse-only`                       | preserved  | (not produced — no expansion) [Q2] |
| `macros:` declaration block (closed) | preserved (original bodies) | — [Q3,Q9,Q37] |
| function signature                   | —          | macro NOT added [Q3] |
| rule `(modulo E)` primary            | preserved  | — [Q3,Q4] |
| rule `(modulo AC)` variant           | —          | **expanded** [Q3,Q4] |
| restriction `/* expanded formula */` | (primary keeps call) | **expanded** [Q4] |
| lemma `/* guarded formula ... */`    | (primary keeps call) | **expanded** [Q4] |
| acc-lemma → generated lemmas' `/* guarded formula */` | (primary/predicate keep call) | **expanded** [Q38] |
| case-test → predicate + generated lemmas' `/* guarded formula */` | (primary/predicate keep call) | **expanded** [Q39] |
| `--diff` left/right projections      | —          | **expanded** (then diff-projected) [Q26] |
| Sapic process → translated rules     | —          | **expanded** [Q29] |

**Accountability [Q38] & case tests [Q39].** `--parse-only` preserves the calls
in both the `test <name>: "..."` case-test formula and the `... accounts for
"..."` acc-lemma formula. On close the acc lemma is translated into several
generated lemmas (`_suff`, `_verif_empty`, `_verif_nonempty`, `_min`, `_uniq`,
`_inj`, `_single`) plus a `predicate:` derived from the case test; their PRIMARY
renderings keep the macro call while the `/* guarded formula characterizing ...
*/` blocks show the expansion — the same primary-keeps-call / guarded-shows-
expansion pattern as ordinary lemmas [Q4]. `formula_parity.sh` verifies the
macro theory's guarded content is byte-identical to a hand-inlined equivalent's.
The consumer's AST holds these as `TheoryItem::AccLemma.formula` /
`CaseTest.formula`; `expand` rewrites both.

A rule that contains a macro **always** emits an explicit `(modulo AC)` variant
block (showing the expansion), even when the expanded term is AC-trivial; a rule
with no macro (and no AC operator) instead prints
`/* has exactly the trivial AC variant */` [Q3,Q4]. This is the only reason the
macro theory and its hand-expanded equivalent differ in the AC section.

### Byte-parity invariant (basis of the test harness)
For a macro theory `T` and its hand-inlined equivalent `T'`, the oracle outputs
differ **only** in three cosmetic categories: (a) the preserved `macros:` block,
(b) the primary modulo-E line showing a call vs the expanded term, (c) the
explicit AC-variant block vs the trivial-variant note. **The expanded content
the oracle computes for `T` (its AC-variant / expanded-formula / guarded-formula)
is byte-identical to the primary rendering of `T'`.** Verified on 6 pairs [Q5,
Q6,Q31].

## 4. `expand(theory) -> theory` (this unit's deliverable)

A source-to-source AST transform:

- Collect macros in declaration order; pre-expand each body against the
  already-fully-expanded earlier macros (§2.4). Result: a table of macro-free
  bodies. (Legal by the DAG property; cannot loop.)
- Rewrite **every term** occurring in the theory (rule premises/actions/
  conclusions/let-values/embedded-restrictions/variants/left-right, lemma &
  restriction & predicate & acc-lemma & case-test formulas, and Sapic process
  terms): bottom-up, replacing each macro call — and each bare untagged nullary
  macro name (§2.5) — with its substituted macro-free body (§2.2–2.3).
- **Preserve the `Macros` items in place**, unchanged (with their original,
  un-expanded bodies), keeping the order of all items [Q37]. The reference
  retains the `macros:` block in its pretty output, and the consuming pipeline
  requires the declaration block in place. (Only the *use sites* are expanded;
  the declarations remain so the pipeline can reprint the block. This is an
  interop requirement; it does not affect `formula_parity.sh` / `byteparity.sh`,
  which observe the oracle on the source theories, not on `expand`'s output.)

Out of scope for this unit (separate passes, only noted): let-binding inlining
[Q23], diff left/right projection [Q26], AC-variant computation [Q30], Sapic
process→rule translation [Q29]. `expand()` performs *only* macro substitution;
it leaves `diff(...)`, `let`, processes, etc. structurally intact with their
inner macro calls expanded.
