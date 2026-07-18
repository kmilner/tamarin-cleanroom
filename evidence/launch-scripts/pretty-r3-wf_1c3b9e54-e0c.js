export const meta = {
  name: 'pretty-r3',
  description: 'Sealed pretty round 3: restriction/lemma FORMULA rendering (quantifiers, connectives, binder allocation, guarded-formula comment text) — the linchpin sub-target; both-sides audit',
  phases: [
    { title: 'Implement', detail: 'sealed formula rendering (Fable)', model: 'fable' },
    { title: 'Audit', detail: 'both-sides similarity audit of the round-3 delta' },
  ],
}

const IMPL = `You are a SEALED implementer under the barrier protocol at /home/kamilner/tamarin-cleanroom/PROTOCOL.md — read it first and comply strictly.
ACCESS RULES: You must NOT read any file under /home/kamilner/tamarin-rs/crates/, and NOT read any Haskell source of tamarin-prover anywhere on disk (notably /home/kamilner/tamarin-rs/tamarin-prover/{lib,src}/ and /home/kamilner/tamarin-rs/tamarin-prover-testing/{lib,src}/). The ONLY Haskell you may read is the SANCTIONED BSD library /home/kamilner/tamarin-cleanroom/graphdot/sanctioned/pretty-1.1.3.6/. Do NOT read /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md, and do NOT read any AUDIT.md anywhere in the cleanroom. No web access. You MAY read .spthy examples under /home/kamilner/tamarin-rs/tamarin-prover/examples/. No git commits.
DISCIPLINE: log every oracle interaction in workspace QUERIES.log; record behavioral facts in BEHAVIOR.md with probe provenance; comments describe current behavior only. Oracle scripts carry OOM guards — preserve them; at most 6 concurrent oracle invocations. Return a PLAIN-TEXT summary.

Workspace: /home/kamilner/tamarin-cleanroom/pretty/ (read/write freely, except AUDIT.md). Crate: workspace/pretty-clean (DONE: R1 term core + signature — both round-2 law fixes in; R2 rule rendering; macros surface). Re-read SPEC.md, BEHAVIOR.md, QUERIES.log first. Oracle: oracle/pretty_oracle.sh <file.spthy> [flags].

THE TASK — sub-target R3 per SPEC.md: restriction / lemma FORMULA rendering, the largest remaining pretty surface (corpus: 384 lemmas, 179 restrictions, 296 exists-trace). Scope:
- The formula grammar as echoed: quantifiers (∀/∃ or All/Ex as the echo actually prints them — pin the exact glyphs/keywords), binder lists and their variable rendering (sorts, #temporal, .idx), connectives ⇒ ∧ ∨ ¬ (or ==> & | not — pin), ⊤/⊥ forms, atoms: action @ timepoint, timepoint ordering <, equality =, subterm ⊏ (or << — pin), last(..), predicate atoms if echoed.
- Precedence/parenthesization: which nestings parenthesize, which do not — probe systematically (constructed lemmas with every connective pairing, both associations).
- LAYOUT: the wrap behavior of long formulas (the Doc engine at 110/73 — how binder lists, nested quantifiers, long conjunctions break and indent; probe threshold shapes like your round-2 p_pw1 methodology).
- WRAPPERS: the lemma line ('lemma NAME [attributes]:' + quantifier kind 'all-traces'/'exists-trace' + the quoted formula block) and restriction wrapper; attribute rendering and canonical order (reuse-, use_induction, sources, hide_lemma, left/right/diff attributes as observed); the guarded-formula comment TEXT ('guarded formula characterizing all counter-examples:' etc.) given the guarded formula as OPAQUE pre-computed input — pin the comment frame and layout, treat the transformed formula content as input.
- Terms inside atoms render via your R1 term core — extend only where formula context differs (probe!).
Create round3/ materials yourself: curated ~14 corpus files spanning formula variety (deep nesting, long binder lists, subterm/nat atoms, accountability-generated lemmas with primed/indexed variables, diff-lemmas if echoed, restriction-heavy files), materialize lemma/restriction block targets via the oracle, and iterate. ACCEPTANCE: whole-lemma-block and whole-restriction-block byte parity for every lemma/restriction in the curated set, asserted in tests; probe-pinned laws in BEHAVIOR.md for every glyph/precedence/wrap decision; all cargo tests green (R1/R2 suites untouched); zero warnings; UNOBSERVABLE register for anything unreachable. Do NOT edit anything outside /home/kamilner/tamarin-cleanroom/pretty/.`

const AUDIT = `You are a BOTH-SIDES similarity auditor for the tamarin-rs sealed relicensing campaign (exempt from barrier access rules; you may read everything). Audit ONLY this round's delta: /home/kamilner/tamarin-cleanroom HEAD (c29e244) contains pretty rounds 1-2 — git -C /home/kamilner/tamarin-cleanroom status/diff restricted to pretty/ shows the round-3 delta (formula rendering + lemma/restriction wrappers).
Audit against the upstream Haskell in /home/kamilner/tamarin-rs/tamarin-prover/: the formula pretty-printer (lib/theory/src/Theory/Text/Pretty.hs, the prettySyntacticLNFormula/prettyGuarded machinery and the Model/Formula pretty instances). Byte-forced content (glyphs, precedence visible in output, wrap positions) is merger; scrutinize the precedence/parenthesization DERIVATION (must trace to logged pairing probes, not the source's precedence table), the binder-allocation approach, and helper decomposition. Flag identifier constellations, non-observable constants, comment lineage.
Append a round-3 section to /home/kamilner/tamarin-cleanroom/pretty/AUDIT.md. Violations get BEHAVIORAL redo instructions. End with exactly one line: VERDICT: pass  — or —  VERDICT: fail`

const impl = await agent(IMPL, { label: 'seal:pretty-r3', phase: 'Implement', model: 'fable' })
const audit = await agent(AUDIT, { label: 'audit:pretty-r3', phase: 'Audit', model: 'opus' })
return { impl: (impl || '').slice(0, 3000), audit: (audit || '').slice(0, 2500), passed: /VERDICT:\s*pass/i.test(audit || '') }