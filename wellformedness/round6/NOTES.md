# Round 6 — final 8 corpus residuals: sealed-side disposition

Scope: the SEALED wellformedness checker (`workspace/wf-clean`). All behavioral
claims trace to the probes logged in `workspace/QUERIES.log`
(`round6/probes/{lhs1,lhs2,lhs3,pc1,pc2}.spthy`) and the reference blocks in
`round6/targets/`. Byte captures below use `$` for LF (from `cat -A`).

## Summary table

| file | family | disposition |
|---|---|---|
| esorics23-bluetooth/models/ble.spthy | listfmt/factusage | **FIXED sealed-side** |
| accountability/masters-thesis-morio/CentralizedMonitor.spthy | pubcap | sealed logic already correct — **pinned** (open side must integrate) |
| regression/trace/issue527.spthy | pubcap, listfmt, sortbody | sealed body already correct — **pinned**; sortbody is an assembly-seam duplication (see below) |
| loops/Axioms_and_Induction.spthy | lemanno | **open-side** (header) — sealed emits header+body |
| regression/trace/issue515.spthy | blank1 | **open-side** — sealed render emits the blank |
| sapic/deprecated/accountability-old/CertificateTransparency.spthy | blank1 | **open-side** — sealed render emits the blank |
| sapic/deprecated/accountability-old/OCSPS.spthy | blank1 | **open-side** — sealed render emits the blank |
| features/.../statVerifLeftRight/stateverif_left_right.spthy | blank1 | **open-side** — sealed render emits the blank |

## 1. FIXED sealed-side — ble listfmt/factusage

`checks::fact_lhs_occur_no_rhs` previously deduplicated the LHS-only list by
fact identity `(name, arity, multiplicity)`, collapsing a fact reused across
several rules into a single entry. The reference emits **one entry per premise
occurrence** with **no deduplication at all**, in **pure source order**:

- probe `lhs1`: `AA`@A, `BB`@A, `CC`@B, `AA`@C — the two `AA` entries are NOT
  grouped/adjacent → source order, not grouped by fact name.
- probe `lhs2`: `DD` appearing twice in one rule → two entries; `EE` arity 1 and
  arity 2 → two entries.
- probe `lhs3`: a fact identity present on ANY RHS suppresses EVERY one of its
  LHS occurrences (identity-level, per-occurrence exclusion).

Fix: the `seen` dedup vector was removed; the RHS-identity exclusion and the
right-aligned numbering ("   1." … "  10.", entries separated by a two-space
line) are unchanged. ble now yields the expected 10 entries.

Regression fixtures/tests (`tests/round6_tests.rs`,
`tests/fixtures/t6_ble_lhs.txt`):
`ble_lhs_not_rhs_per_rule_entries`, `lhs_not_rhs_no_dedup_source_order`,
`lhs_not_rhs_rhs_identity_suppresses_all_occurrences`.

## 2. Pinned (sealed already correct) — CentralizedMonitor pubcap

`checks::public_names_report` already produces the reference bytes. Probe `pc1`
(single translated rule "Init" using public constants 'c' and 'C') reproduces
the CentralizedMonitor target exactly:

```
Public constants with mismatching capitalization$
================================================$
$
Identifiers are case-sensitive, mismatched capitalizations are considered as different, i.e., 'ID' is different from 'id'. Check the capitalization of your identifiers.$
$
  1. rule "Init":  name 'C', 'c'$
```

Pinned by `centralizedmonitor_pubcap_block` /
`tests/fixtures/t6_centralizedmonitor_pubcap.txt`. The 7-line gate divergence
was the open side lacking this whole topic; the sealed checker emits it. The
open side must integrate the sealed `public_names_report` output. (The
`Accountability (RP check)` block above it in the full target is emitted by the
accountability subsystem, not the wf checker, and is not a sealed concern.)

## 3. Pinned + SEAM — issue527

The sealed report is byte-identical to the round6 target through the last
sealed topic. Verified by diffing the target (truncated before the Maude
`Message Derivation Checks` block) against `tests/fixtures/t5_issue527.txt`:
the ONLY delta is a single blank line at target line 110 — the join separator
the open side inserts before the out-of-scope `Message Derivation Checks`
block. So pubcap, fact-capitalization, fact-arity and the mismatching-sorts
body are all already correct sealed-side.

### The sortbody assembly seam ("Possible reasons:")

The sealed `mismatching_sorts` produces the COMPLETE topic body, whose exact
opening bytes (`WfError.message` for topic
`Variable with mismatching sorts or capitalization`) are:

```
Possible reasons:$
1. Identifiers are case sensitive, i.e.,'x' and 'X' are considered to be different.$
2. The same holds for sorts:, i.e., '$x', 'x', and '~x' are considered to be different.$
$
```

…immediately followed by the per-rule numbered groups (for issue527:
`  rule `Register_pk': $` / `    1. ~ltk, ltk$` / `  $` / `  rule `Test_1': $` /
`    1. $X, $x`). Rendered under the topic header the full block is byte-equal
to target lines 24–35.

**Seam instruction for the open side:** the sealed body ALREADY contains the
`Possible reasons:` preamble (the four lines above). The assembly layer must
**not** prepend its own copy — doing so is the "duplicated preamble" the round6
gate observed. The full topic body for this section is produced entirely by the
sealed checker; the open side should emit only the header + underline + blank +
the sealed body verbatim.

## 4. Open-side items — confirmed, no contradiction with the stated split

Probing does NOT contradict the OUT-OF-SCOPE classification:

- **Axioms_and_Induction (lemanno):** `check_theory` → `render_report` emits the
  `Lemma annotations` header (via `underline_topic`) AND the body
  `  Lemma `Exists_test': cannot reuse 'exists-trace' lemmas`. Pinned by the
  existing `axioms_and_induction_reference_block` test. The sealed checker DOES
  emit the finding and its header; a missing header on the open side is an
  assembly gap only.
- **blank1 (issue515, OCSPS, CertificateTransparency, stateverif_left_right):**
  `render_block` emits exactly one blank line after each topic underline
  (`format!("{}\n\n{}", underline, message)`) and `render_report` joins topics
  with one blank line. The sealed render carries the blank line the open side is
  said to drop. Confirmed via the passing `issue515_reference_block` and
  `sapic_lookup_rule_unbound_block` tests. Open-side assembly only.

## Test status

`cargo test` green: api_surface 1, checks_tests 74, corpus5 acceptance 1
(unchanged pin), round5 29, round6 4. Total 109 passing, 0 failing.
