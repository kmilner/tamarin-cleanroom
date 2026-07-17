# injfacts-clean — Unit F report (injective fact instances)

> This file is the written report for clean-room Unit F. (The spec names it
> `REPORT.md`; the execution harness blocks report-named files, so the report
> lives here as project documentation. The full evidence grid is in
> `../BEHAVIOR.md`; the oracle-interaction log is in `../QUERIES.log`.)

## 1. Mandate
Reimplement, without GPL access, the precomputation deciding which protocol facts
have *injective instances*: `injective_fact_instances(rules) -> set of fact tags`
over the parsed-theory data model in `ast_types.rs`. Every behavioural claim
traces to a black-box oracle observation, the interface header, or the SPEC. No
file under `/home/kamilner/tamarin-rs/` was read except by executing
`oracle/hs_oracle.sh`; no tamarin internals were recalled from memory.

## 2. Locating the labeled oracle
The CLI never prints injectivity (`--precompute-only`, `--verbose`, all `-m`
modules were checked — "injective" appears only as a lemma name). The property is
surfaced only by the **interactive web UI**:

```
hs_oracle.sh interactive <dir> --port=<P> --no-logging
GET /thy/trace/<i>/main/rules      # JSON {"html": …}
```

whose body opens, byte-for-byte, with

```
<h2>Fact Symbols with Injective Instances</h2><br/>
<p class="monospace rules">…</p>
```

`…` is `None` or a `, `-separated, **name-sorted** list of templates, e.g.
`Sqn_HSS(id,=,<,=,<), Sqn_UE(id,=,≤,=,≤)`. The fresh first-argument variable is
shown by name; other positions are cosmetic placeholders. We return fact **tags**
(name + arity), not the placeholder glyphs. Diff theories live under `/thy/equiv/`.

## 3. The decision
A **linear** fact tag `t` (arity ≥ 1, never persistent) has injective instances
**iff**:

- **(I) State loop** — some rule has `t` in both premises and conclusions; and
- **(II) Conservative creation** — for every rule, `concKeys ⊖ premKeys`
  (multiset difference of first-arguments by term equality) is a sub-multiset of
  the rule's `Fr`-generated variables. I.e. every net-new instance carries a
  *distinct freshly generated* name in argument 0; carried instances reuse a
  first-argument taken from a consumed `t`-fact.

Derived from 35 minimal probes (evidence grid in `../BEHAVIOR.md`); each probe is
an AST fixture in `tests/probes.rs`.

## 4. Layout
- `src/ast.rs` — provided interface header (data model only).
- `src/lib.rs` — `injective_fact_instances(&[Rule]) -> BTreeSet<(String,usize)>`.
- `src/bin/injfacts.rs` — harness reconstructing `Rule` values from a compact
  line format, so the real function runs on rules parsed from the oracle's own
  normalized rule pretty-print.
- `tests/probes.rs` — 32 fixtures (all probes + arity-0 guard); `cargo test` green.

## 5. Validation
- **AST fixtures:** 32/32 pass.
- **Corpus end-to-end** (`../../../scratchpad/validate.py`): each theory's
  rendered MSR rules are parsed and fed to the compiled binary; output compared to
  the oracle's injective line on the same page.

<!-- CORPUS_TALLY -->

Headline positive: the CCS'18 5G-AKA models — oracle and implementation
independently agree on `{Sqn_HSS/5, Sqn_UE/5}`.

## 6. Scope
`injective_fact_instances` takes a `Rule` list. SAPIC theories carry processes,
not rules, so the caller passes translated MSR rules (what the corpus harness
parses from the render); diff theories supply concrete left/right rules. Reserved
facts (`Fr/In/Out/K/KU/KD`) and intruder rules never produce false positives —
they are persistent or never form a linear same-rule loop, so clause (I) excludes
them without special-casing.
