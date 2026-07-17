# BEHAVIOR.md — Unit F: injective fact instances

All statements here are derived from oracle observations (interactive web UI
`/thy/.../main/rules` page, section "Fact Symbols with Injective Instances")
and the input model `ast_types.rs`. No tamarin-prover source was read.

## Where the oracle surfaces the answer
The CLI (`--precompute-only`, `-m …`, `--verbose`) never prints the word
"injective" as a fact property — it appears only as lemma names. The labeled
oracle is the **interactive web UI**:

- Launch: `hs_oracle.sh interactive <workdir> --port=P --no-logging`
  (note: `--port=` must use `=`; a bare `--port P` makes `P` the positional WORKDIR).
- The page `/thy/trace/<idx>/main/rules` (AJAX, returns JSON `{"html": …}`)
  begins with, exact bytes:

  `<h2>Fact Symbols with Injective Instances</h2><br/>\n<p class="monospace rules">…</p>`

  where `…` is either `None` or a comma-space-separated, **name-sorted** list of
  fact-instance templates, e.g. `MB(id), ZA(id)`.
- Diff theories are served under `/thy/equiv/<idx>/…` instead of `/thy/trace/…`.

### Rendering of one injective template
`F( a0, a1, … )` renders with the fresh first-argument variable shown by its base
name (`~id` → `id`) and every non-first position shown as a placeholder (`=` when
that position is a bare variable, `.` when it is/varies as a non-variable). Only
the **first argument** participates in the injective key; the placeholder for the
other positions is cosmetic. Our deliverable extracts the fact **name + arity**
(the tag) from this render; we do not reproduce the placeholder glyph.

## The decision (characterised from 35 minimal probes, then corpus-validated)
Let the protocol rules be the input. For a fact **tag** `t = (name, arity)` that
is **linear** (not persistent `!`), define per rule `r`:

- `premKeys(t,r)` = multiset of first arguments of `t`-facts in `r.premises`
- `concKeys(t,r)` = multiset of first arguments of `t`-facts in `r.conclusions`
- `freshVars(r)`  = multiset of variables `v` such that a linear fact `Fr(v)`
  occurs in `r.premises`

`t` is **injective** iff BOTH:

- **(I) State loop.** There exists a rule where `t` occurs in *both* premises and
  conclusions (same rule).
- **(II) Conservative creation.** For *every* rule `r`, the multiset difference
  `netNew = concKeys(t,r) ⊖ premKeys(t,r)` is a **sub-multiset of** `freshVars(r)`.
  I.e. every `t`-conclusion whose first argument is not cancelled by a `t`-premise
  with the same first argument must be a distinct freshly-generated (`Fr`-bound)
  variable.

Persistent facts, and any tag that never appears in both a premise and a
conclusion of one rule, are excluded. Reserved facts (`Fr`, `In`, `Out`, `K`,
`KU`, `KD`) are excluded automatically by (I): none occurs in both premise and
conclusion of a single rule.

Arity-0 facts have no first argument and are never injective (guarded explicitly).

## Evidence grid (probe → observed → rule branch that explains it)
| probe | rule shape (fact F) | observed | explanation |
|---|---|---|---|
| p01 | Fr→F(~id); F(~id)→F(~id); F(~id)→· | F(id) | (I)✓ loop; (II)✓ |
| p02 | Fr→F(~id); F(~id)→· (no loop) | None | (I)✗ no same-rule loop |
| p03 | Fr→!F(~id) (persistent) | None | persistent excluded |
| p04 | In(x)→F(x); F(x)→F(x) | None | (II)✗ producer key x not fresh/carried |
| p05 | Fr→F('c',~id); loop | None | first arg is 'c' (pub const), not fresh |
| p06 | [Fr~a,Fr~b]→[F~a,F~b]; F(~x)→· | None | (I)✗ no loop |
| p07 | Fr→F(~id); [F(~id),Fr~id2]→F(~id2) | F(id) | (I)✓; (II)✓ (~id2 fresh) |
| p08 | ·→F('a'); F(x)→· | None | first arg const; (I)✗ |
| p09 | Fr→F(~id) AND In(x)→F(x); F(x)→· | None | (II)✗ (In producer) & (I)✗ |
| p11 | [Fr~id,In x]→F(~id,x); loop; consume | F(id,=) | (I)✓; (II)✓ only 1st arg matters |
| b01 | [F(~id),Fr~id2]→[F(~id),F(~id2)] +Init | F(id) | (II)✓ 1 carried + 1 fresh |
| b02 | 2 producers (both Fr), 1 consumer, no loop | None | (I)✗ |
| b03 | Fr→F(~id); F(~id)→F(~id) (loop only) | F(id) | (I)✓;(II)✓ |
| b04 | Fr→F(~id) only (produce only) | None | (I)✗ |
| b05 | first arg pub var $A + loop | None | (II)✗ $A not fresh |
| b06 | first arg h(~id) + loop | None | (II)✗ not a variable |
| b07 | [F~a,F~b]→[F~a]; loop | F(id) | (II)✓ consume 2 produce 1 |
| b08 | In(x)→F(x); F(x)→F(x); consume | None | (II)✗ In producer, x not fresh |
| c01 | good loop + extra consumer F(y) y msg | F(id) | consumers ignored (no conc) |
| c02 | two independent looped fresh facts | MB(id), ZA(id) | both injective; sorted list |
| c03 | good loop + producer In(z)→F(z) | None | (II)✗ that producer |
| c06 | Fr→F(~id); F(z)→F(z) (z carried, msg) | F(id) | (II)✓ carried, key eq |
| d01 | loop reproduces F(w), w from In | None | (II)✗ w neither carried nor fresh |
| d02 | loop reproduces F(w), w from non-F prem | None | (II)✗ carry must be from a t-premise |
| d04 | loop reproduces F(~w), ~w from !HH (not Fr) | None | (II)✗ fresh-sort ≠ Fr-bound |
| d05 | loop reproduces F(~z) carried | F(id) | (II)✓ |
| e01 | F(~id,'a'); loop F(~z,x)→F(~z,'b') | HH(id,.) | 1st arg carried; other free |
| e02 | loop + Dup F(~z)→F(~z),F(~z) | None | (II)✗ netNew {~z} not fresh |
| e03 | loop + Two Fr(~a)→F(~a),F(~a) | None | (II)✗ two same-key from one Fr |
| g01 | loop + [F~a,F~b,Fr~c]→[F~a,F~b,F~c] | F(id) | (II)✓ 2 carried + 1 fresh |
| g02 | loop + [F~a,F~b]→[F~a,F~a] | None | (II)✗ netNew {~a} not fresh |

## Corpus validation
See workspace/corpus_results.tsv (oracle ground truth) vs the reference
implementation's output. Sapic theories: the oracle computes injectivity over the
*translated* MSR rules; our function operates on a `Rule` list, so for sapic
inputs the caller must pass translated rules. Diff theories expose left/right
rule sets under `/thy/equiv/…`.

## Corpus validation — rebuilt & persistent (workspace/validation/)
The 017/018 harness ran in a temp dir that was lost; it was rebuilt in place under
`workspace/validation/` so scripts and results persist:
- `scrape.sh FILE MODE PORT` — isolated single-theory interactive server; blocks
  on the log line `Theory closed`, then captures the rendered rules page
  (`/thy/trace/<i>/main/rules` for trace, `--diff` + `/thy/equiv/<i>/main/diffrules`
  for diff) to `corpus_html/<slug>.<route><i>.json`.
- `run_corpus.sh [PAR] [LIST]` — runs `scrape.sh` over every theory, PAR in flight.
- `parse_and_compare.py` — extracts the oracle's "Fact Symbols with Injective
  Instances" line, renders the page's MSR rules into the injfacts binary grammar,
  runs the crate, and diffs the `(name,arity)` tag sets → `corpus_results.tsv`.
- `audit_rulecounts.py` — sanity audit of rules parsed per theory (guards against
  a spurious `None`/`None` match caused by an empty parse).

### Tally (49 theories = all of oracle/examples/*.spthy)
| bucket | n | result |
|---|---|---|
| trace, oracle = None → clean = None | 34 | MATCH |
| trace, oracle non-None (both 5G AKA) | 2 | MATCH — oracle & clean both `Sqn_HSS/5, Sqn_UE/5` |
| diff (needs `--diff`) | 13 | n/a — oracle surfaces no injective section in diff mode |
| **total** | **49** | **36/36 trace agree, 0 mismatch, 0 fail** |

Only the two `ccs18-5G` theories carry injective facts in the whole corpus; the
oracle renders `Sqn_HSS(id,=,<,=,<), Sqn_UE(id,=,≤,=,≤)` (arity 5) and the crate,
run on the parsed rendered rules, returns exactly `Sqn_HSS/5, Sqn_UE/5`. Every
other trace theory (incl. stateful Yubikey/PKCS11 — 52/125 parsed rules) is `None`
in both. No theory showed the oracle contradicting the crate, so no rule change was
warranted.

### Findings that shaped the rebuild (all from oracle observation)
- **Diff mode has no injective section.** Loaded with `--diff`, a theory is served
  only under `/thy/equiv/<i>/`; its rules live at `/main/diffrules` (parent + left
  + right projections) and NO "Fact Symbols with Injective Instances" header exists
  there or on any per-side path — injective-instance analysis is trace-mode only.
  Diff theories are therefore recorded n/a, not compared.
- **Sapic rule headers** render as `rule (modulo E) NAME[color=…,process="…",`
  `issapicrule,…]:` — a multiline attribute block bearing colons; the rules parser
  must consume it before the header-terminating `:` (else only the two intruder
  rules parse; PKCS11 went 2→125 rules once fixed).
- **Oracle-heuristic theories** (`heuristic: o "./oracle-…"`) need that script on
  disk at load; a no-op passthrough stub in the workdir lets the theory close and
  does not affect the injective precomputation (accountability/dmn-message-tracing).
- **Large theories** register in the web server a moment after logging
  `Theory closed`; the scraper polls the root for the theory index before scraping.
