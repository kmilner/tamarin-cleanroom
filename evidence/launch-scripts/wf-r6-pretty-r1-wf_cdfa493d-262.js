export const meta = {
  name: 'wf-r6-pretty-r1',
  description: 'Parallel sealed rounds: wf residual closure (8 corpus diffs -> 0) and pretty R1 (term core + signature); both-sides audits; serialized open-side integration with full-corpus gates',
  phases: [
    { title: 'Implement', detail: 'wf-r6 residuals (Opus) in parallel with pretty R1 term core + signature (Fable)' },
    { title: 'Audit', detail: 'both-sides similarity audit per stream' },
    { title: 'Integrate', detail: 'wf-r6 re-sync + preamble-map fixes + wf_gate, then pretty R1 wiring + full gates' },
  ],
}

const SEAL_RULES = `You are a SEALED implementer under the barrier protocol at /home/kamilner/tamarin-cleanroom/PROTOCOL.md — read it first and comply strictly.
ACCESS RULES: You must NOT read any file under /home/kamilner/tamarin-rs/crates/, and NOT read any Haskell source of tamarin-prover anywhere on disk (notably /home/kamilner/tamarin-rs/tamarin-prover/{lib,src}/ and /home/kamilner/tamarin-rs/tamarin-prover-testing/{lib,src}/). The ONLY Haskell you may read is the SANCTIONED BSD library /home/kamilner/tamarin-cleanroom/graphdot/sanctioned/pretty-1.1.3.6/ (BSD-3, brought across the barrier deliberately). Do NOT read /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md. No web access. You MAY read .spthy example files anywhere under /home/kamilner/tamarin-rs/tamarin-prover/examples/. No git commits.
DISCIPLINE: log every oracle interaction (one line: probe file + purpose) in your workspace QUERIES.log; record inferred behavioral facts in BEHAVIOR.md with probe provenance; code comments describe current behavior only. Every behavioral claim must trace to a logged oracle probe or a captured target — never to memory of any implementation. The oracle scripts carry OOM guards (oom_score_adj/ulimit) — do not strip them; keep concurrent oracle invocations at 6 or fewer. Return a PLAIN-TEXT summary.`

const WF6_IMPL = `${SEAL_RULES}

Workspace: /home/kamilner/tamarin-cleanroom/wellformedness/ (read/write freely inside it). Crate: workspace/wf-clean. Oracle: oracle/wf_oracle.sh <file.spthy> [flags] — runs the Haskell reference and prints its output including the wf block.

THE TASK — round 6, the final 8 corpus residuals. round6/wf_gate_diffs.txt lists the 8 theories; round6/targets/<path-underscored>.hs.txt is each one's CORRECT reference wf-block; round6/families.tsv classifies them. Below, each family's divergence is stated from direct observation of the reference targets. Your scope is the SEALED-side families; two families are open-side assembly gaps handled outside the room (listed at the end).

FAMILY pubcap (CentralizedMonitor 7 lines, issue527 partly) — the reference emits an entire topic your checker lacks:
  Public constants with mismatching capitalization
  ================================================
  (blank)
  Identifiers are case-sensitive, mismatched capitalizations are considered as different, i.e., 'ID' is different from 'id'. Check the capitalization of your identifiers.
  (blank)
  entries like:  1. rule "Init":  name 'C', 'c'
In issue527 the same topic has two entries — "1. rule \\"Four\\":  name 'firSt', rule \\"Two\\":  name 'first'" and "2. rule \\"One\\":  name 'seconD', 'second'" — separated by a line containing exactly two spaces. Note the two entry shapes: when the mismatched spellings first occur in DIFFERENT rules each spelling carries its own rule attribution; same rule collapses to a name list. Probe with constructed theories (public constants 'c'/'C'/'cC' spread across rules) to pin: the grouping key, first-occurrence attribution, entry ordering, numbering, singular/plural forms, wrapping.

FAMILY factusage (ble.spthy, 20 lines; the "factName \`X' arity: N multiplicity: Linear" list) — two divergences: (1) GRANULARITY: your list emits one entry per distinct factName; the reference emits one entry per (rule, factName) — ble expects 10 entries (RespChooseKeysize listed separately for 4 rules, InitChooseKeysize for 4). (2) ALIGNMENT: with >= 10 entries the reference right-aligns the index to the widest ("   1." three leading spaces … "  10." two), your render uses fixed two-space indent. Entries are separated by a line of exactly two spaces (you already do this). Probe multi-rule arity clashes to pin entry order and the alignment rule (right-align to widest index width — confirm at 9 vs 10 entries).

FAMILY 527-topics (issue527, 13 lines total) — the reference also contains two topics you omit entirely: "Fact capitalization issues" and "Fact arity issues" (each: underline, blank, numbered entries). Additionally your "Variable with mismatching sorts or capitalization" section renders a "Possible reasons:" paragraph at a position the reference does not have one (duplicated preamble — the assembly layer outside the room also prepends this paragraph; make your body byte-match the target's BODY and record in round6/NOTES.md exactly which bytes you emit for this topic, so the assembly seam can be reconciled). Target: round6/targets/regression_trace_issue527.spthy.hs.txt is your ground truth for all of these.

OUT OF SCOPE (open-side assembly, fixed outside the room): (a) four SAPIC files each missing ONE blank line after the "Unbound variables"/"Special facts" topic underline; (b) Axioms_and_Induction missing the "Lemma annotations" topic HEADER (your body lines already match). If your probing contradicts this split — e.g. your checker fails to emit the lemma-annotation finding at all given a faithful input — say so in round6/NOTES.md.

Keep every existing wf-clean test green, including the corpus5 acceptance pin. Add fixtures pinning each fixed family. Write round6/NOTES.md: which residuals you fixed sealed-side, which you confirm as open-side, and the exact bytes the assembly layer should expect at each seam. Do NOT edit anything outside /home/kamilner/tamarin-cleanroom/wellformedness/.`

const WF6_AUDIT = `You are a BOTH-SIDES similarity auditor for the tamarin-rs sealed relicensing campaign (exempt from barrier access rules; you may read everything). Audit ONLY this round's delta for the wellformedness slice.
/home/kamilner/tamarin-cleanroom is a git repo whose HEAD (cbc511d) predates this round: run git -C /home/kamilner/tamarin-cleanroom status/diff restricted to wellformedness/ to see the change. This round added: a public-constants capitalization check, fact-capitalization and fact-arity topics, per-(rule,fact) granularity + right-alignment in the fact-usage list, and sort-clash body reconciliation. Audit against the upstream Haskell — lib/theory/src/Theory/Tools/Wellformedness.hs (the public-names/fact checks) in /home/kamilner/tamarin-rs/tamarin-prover/.
Method: abstraction-filtration-comparison. Report content dictated by byte-parity with the observable wf block is merger/compatibility, not copying; flag copied protectable EXPRESSION: internal identifier constellations, non-observable magic constants, structural transcription beyond what the boundary forces, comment lineage. Cross-check that every behavioral claim in the delta's BEHAVIOR.md/QUERIES.log traces to a logged probe or captured target, not to Wellformedness.hs.
Append a round-6 section to /home/kamilner/tamarin-cleanroom/wellformedness/AUDIT.md. Violations get a BEHAVIORAL redo instruction (never a patch). End with exactly one line: VERDICT: pass  — or —  VERDICT: fail`

const WF6_GATE = `You are the OPEN-SIDE integrator for the tamarin-rs sealed relicensing campaign, working in /home/kamilner/tamarin-rs. Read the wellformedness sections of /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md and PROTOCOL.md ("Full-corpus gates") first, then /home/kamilner/tamarin-cleanroom/wellformedness/round6/NOTES.md (the sealed implementer's handoff).
This round fixed the sealed-side residuals of the 8 remaining wf corpus divergences; TWO families are yours to fix in the OPEN-side assembly layer. Target: full-corpus wf_gate at 0 DIFF (from 8).
RULES: sealed sources re-sync is mechanical only (the documented recipe: sed crate::->super:: path fixes, drop 'pub mod ast;', 'pub use ast::*;' -> 'pub use crate::ast::*;', PRESERVE crates/tamarin-parser/src/wf/order.rs + its two mod-lines). Do NOT transplant logic into wf/; do NOT hand-edit the sealed checker; clean files stay headerless (a wf/ file acquiring a GPL header is a provenance violation — stop and report). No git commits. PATH needs /home/linuxbrew/.linuxbrew/bin.
STEPS:
1. Re-sync the round-6 wf-clean sources from /home/kamilner/tamarin-cleanroom/wellformedness/workspace/wf-clean/src into crates/tamarin-parser/src/wf/ per the recipe; verify reverse-transform byte-identity.
2. OPEN-SIDE fixes in crates/tamarin-theory/src/pretty_theory.rs, fn wf_headerless_preamble (~line 863): (a) the 4 SAPIC residuals (stateverif_left_right, CertificateTransparency, OCSPS, issue515) show the reference DOES print a blank line after the "Unbound variables" / "Special facts" underline on the headerless path — move those two topics to the underline+"\\n\\n" arm. A code comment there claims corpus files validated no-blank; the full gate is the arbiter — if flipping them regresses any currently-MATCH file, characterize the context split precisely instead of guessing. (b) Register "Lemma annotations" (reference: underline, blank, bodies — fixes loops/Axioms_and_Induction). (c) issue527 seam: the map prepends a "Possible reasons:" paragraph for "Variable with mismatching sorts or capitalization" while the round-6 sealed body may now carry its own — reconcile per round6/NOTES.md and the target bytes (round6/targets/regression_trace_issue527.spthy.hs.txt) so the paragraph appears exactly once, exactly where the reference has it.
3. cargo build --release.
4. Full gate: RESULTS_TSV=scripts/wf_gate_round6.tsv JOBS=6 bash scripts/wf_gate.sh — before/after vs the round-5 result (411 MATCH / 8 DIFF). Target 0 DIFF; for any residual give path + diff head. Also confirm zero regressions (every round-5 MATCH still MATCH).
5. Regression guard: cargo test -p tamarin-parser -p tamarin-theory -p tamarin-prover -p tamarin-server (report counts); scripts/gen_license_headers.py --check (0 stale; no wf/ file gained a header).
Append a dated round-6 section to INTEGRATION_REPORT.md. Return a plain-text summary with exact before/after gate numbers. Keep-and-report anything still divergent — do NOT hand-fix sealed code.`

const P1_IMPL = `${SEAL_RULES}

Workspace: /home/kamilner/tamarin-cleanroom/pretty/ (read/write freely inside it). You may ALSO read (not write) /home/kamilner/tamarin-cleanroom/graphdot/workspace/graph-clean/src/pretty.rs — the already-sealed BSD-derived HughesPJ Doc engine — and its sanctioned source library graphdot/sanctioned/pretty-1.1.3.6/; REUSE that engine, do not re-derive layout logic any other way.
READ FIRST: pretty/SPEC.md (the observable boundary, sub-target decomposition, solver-entangled split, method requirements), pretty/README.md, interface/ast_types.rs, interface/required_api.md.

THE TASK — sub-target R1: the TERM CORE + SIGNATURE BLOCK of the theory echo, in the clean crate workspace/pretty-clean (scaffolded: src/{ast,doc,term,signature,...}.rs, tests/round1_term_signature.rs with ignored stubs).
Oracle: oracle/pretty_oracle.sh <file.spthy> [flags] prints the extracted theory echo — the exact bytes to reproduce (RAW=1 for full output). Byte targets for 10 curated files are pre-materialized in round1/targets/*.hs.txt (regenerate with round1/fetch_hs_targets.sh if needed); round1/families.tsv tags what each exercises.
Scope of R1 (per SPEC.md): (1) Term -> text: operator glyphs and precedence/parenthesization (exp ^, mult *, xor XOR/⊕ as observed, multiset ++, nat %+, pairing <a,b> nesting, diff(), sort sigils ~ $ # %, quoted public constants 'x', variable .N index suffixes, function application and arity-0 constants) — pin every case by probing the oracle with constructed .spthy theories whose rules/lemmas force the term shapes through the echo; (2) the signature block: "builtins:", "functions:", "equations:" declarations — item ordering, dedup, attribute rendering (private, destructor), comma placement, continuation-line indentation, wrap behavior (Doc engine at width 110, ribbon 73), and which builtin-induced symbols/equations the echo lists for each builtin (learn the lists from oracle output — they are compatibility content).
Method requirements: hand-built AST fixtures (you may NOT parse .spthy in the crate; input is the interface/ast_types.rs model — extend your crate-internal AST only compatibly with it); per-behavior tests asserting byte-exact fragments captured from the oracle; whole-signature-block parity asserted for all 10 round-1 files (extract the block from the target, build the corresponding AST fixture by reading the .spthy and the oracle output, assert byte equality). Maintain BEHAVIOR.md (operator table, precedence levels, spacing/wrap rules, section order — a deliverable equal in weight to the code) and QUERIES.log. All cargo tests green, zero warnings. If a term shape's rendering cannot be forced through the echo, record it as UNOBSERVABLE in BEHAVIOR.md rather than guessing.
Do NOT edit anything outside /home/kamilner/tamarin-cleanroom/pretty/.`

const P1_AUDIT = `You are a BOTH-SIDES similarity auditor for the tamarin-rs sealed relicensing campaign (exempt from barrier access rules; you may read everything). Audit ONLY this round's delta: the NEW pretty cluster round-1 implementation (term core + signature block) under /home/kamilner/tamarin-cleanroom/pretty/ (repo HEAD cbc511d contains only the scaffold — git -C /home/kamilner/tamarin-cleanroom diff/status restricted to pretty/ shows the round's delta).
Audit against the upstream Haskell in /home/kamilner/tamarin-rs/tamarin-prover/: the term pretty-printer (lib/term/src/Term/ — Raw/Pretty instances), the signature/theory printers (lib/theory/src/Theory/Text/Pretty.hs and the prettyTheory/signature rendering in lib/theory/src/), and confirm the Doc engine reuse derives from the SANCTIONED BSD pretty-1.1.3.6 (resemblance to BSD source is licensed and fine — verify license preservation) rather than from any GPL wrapper module.
Method: abstraction-filtration-comparison. Byte-parity-forced output (glyphs, spacing, orderings observable in the echo) is merger/compatibility content; flag copied protectable EXPRESSION: upstream identifier constellations, internal helper structure not forced by the boundary, non-observable constants, comment lineage. Cross-check BEHAVIOR.md/QUERIES.log provenance for the hardest-to-guess behaviors (precedence table, wrap decisions, builtin symbol lists).
Write /home/kamilner/tamarin-cleanroom/pretty/AUDIT.md (create; round-1 section). Violations get a BEHAVIORAL redo instruction (never a patch). End with exactly one line: VERDICT: pass  — or —  VERDICT: fail`

const P1_GATE = `You are the OPEN-SIDE integrator for the tamarin-rs sealed relicensing campaign, working in /home/kamilner/tamarin-rs. Read /home/kamilner/tamarin-cleanroom/pretty/{SPEC.md,README.md}, interface/required_api.md, the pretty sections of /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md if any, and PROTOCOL.md ("Full-corpus gates") first. The wf round-6 integrator ran before you — read its INTEGRATION_REPORT.md section; your wf_gate baseline is scripts/wf_gate_round6.tsv.
This round produced the sealed pretty-clean crate's R1: term core + signature block (/home/kamilner/tamarin-cleanroom/pretty/workspace/pretty-clean). Your job: attempt adoption in the tree behind thin adapters, gated by byte-parity on the FULL corpus.
RULES: bring sealed sources across mechanically (path fixes only; document the recipe); clean files stay HEADERLESS (a clean file acquiring a GPL header is a provenance violation — stop and report); adapters you write live in open-side files and only translate live tamarin_parser/tamarin_term values into the clean AST — never transplant render logic out of the clean files. No git commits. PATH needs /home/linuxbrew/.linuxbrew/bin. The gate scripts carry OOM guards — preserve them.
STEPS:
1. Place the sealed crate in the tree (your choice, precedent: crates/tamarin-parser/src/wf/ was the wf swap; a crates/tamarin-theory/src/pretty_clean/ module or a workspace crate both acceptable). Verify reverse-transform byte-identity to the sealed sources.
2. Adapters: route (a) term rendering (crates/tamarin-term/src/pretty.rs surface) and (b) the signature block (render_signature_block path in crates/tamarin-theory/src/pretty_theory.rs) through the clean crate. Partial adoption is fine — adopt whichever of (a)/(b) is byte-green and keep-and-report the other with byte-precision blockers (file, first diverging line).
3. Gates, in order, each must be green before the next: cargo build --release; cargo test full workspace; RESULTS_TSV=scripts/pretty_gate_r1.tsv JOBS=6 bash scripts/pretty_gate.sh — expect EXACTLY the baseline 403 MATCH / 16 DIFF (the 16 are the known auto-sources closure gap; ANY new DIFF is a blocker); JOBS=6 bash scripts/wf_gate.sh with RESULTS_TSV=scripts/wf_gate_after_pretty.tsv — expect no regression vs scripts/wf_gate_round6.tsv; bash scripts/web_parity.sh (term rendering feeds the web pane) — no new divergences vs its cache.
4. scripts/gen_license_headers.py --check (0 stale; no clean file gained a header).
5. If adoption is fully blocked, leave the ported renderer in place and record byte-precision blockers for the next sealed round.
Append a dated pretty round-1 section to INTEGRATION_REPORT.md with exact gate numbers and what was adopted vs kept. Return a plain-text summary. No deletions of ported code this round unless the deleted path's every caller is byte-green through the clean route.`

const streams = await parallel([
  async () => {
    const impl = await agent(WF6_IMPL, { label: 'seal:wf-r6', phase: 'Implement', model: 'opus' })
    const audit = await agent(WF6_AUDIT, { label: 'audit:wf-r6', phase: 'Audit', model: 'opus' })
    const passed = /VERDICT:\s*pass/i.test(audit || '')
    const gate = passed
      ? await agent(WF6_GATE, { label: 'gate:wf-r6', phase: 'Integrate', model: 'opus' })
      : 'SKIPPED — wf-r6 audit did not pass; not integrated.'
    return { impl, audit, passed, gate }
  },
  async () => {
    const impl = await agent(P1_IMPL, { label: 'seal:pretty-r1', phase: 'Implement', model: 'fable' })
    const audit = await agent(P1_AUDIT, { label: 'audit:pretty-r1', phase: 'Audit', model: 'opus' })
    return { impl, audit, passed: /VERDICT:\s*pass/i.test(audit || '') }
  },
])
const wf = streams[0] || { impl: 'wf stream died', audit: '', passed: false, gate: 'stream died before integration' }
const pr = streams[1] || { impl: 'pretty stream died', audit: '', passed: false }
log('implement+audit complete (wf integrated inside its stream); pretty integration ' + (pr.passed ? 'starting' : 'skipped'))
const prGate = pr.passed
  ? await agent(P1_GATE, { label: 'gate:pretty-r1', phase: 'Integrate', model: 'opus' })
  : 'SKIPPED — pretty-r1 audit did not pass (or stream died); not integrated.'
return {
  wf: { impl: (wf.impl || '').slice(0, 2200), audit: (wf.audit || '').slice(0, 1200), passed: wf.passed, gate: (wf.gate || '').slice(0, 2500) },
  pretty: { impl: (pr.impl || '').slice(0, 2500), audit: (pr.audit || '').slice(0, 1200), passed: pr.passed, gate: (prGate || '').slice(0, 2500) },
}