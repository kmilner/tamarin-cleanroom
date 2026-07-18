export const meta = {
  name: 'wf-sealed-round5',
  description: 'Sealed round: fix the 71 corpus wf divergences (guardedness over-report 49, sort-clash 15, tail 7) in the sealed wf workspace; audit; open-side re-sync + wf_gate confirm',
  phases: [
    { title: 'Implement', detail: 'sealed wf fixes (Fable): guardedness + sort-clash + tail', model: 'fable' },
    { title: 'Audit', detail: 'both-sides similarity audit of the delta' },
    { title: 'Gate', detail: 'open-side re-sync into the tree + full-corpus wf_gate.sh' },
  ],
}

const IMPL = `You are a SEALED implementer under the barrier protocol at /home/kamilner/tamarin-cleanroom/PROTOCOL.md — read it first and comply strictly.
ACCESS RULES: You must NOT read any file under /home/kamilner/tamarin-rs/crates/, and NOT read any Haskell source (*.hs) of tamarin-prover anywhere on disk (notably /home/kamilner/tamarin-rs/tamarin-prover/{lib,src}/ and /home/kamilner/tamarin-rs/tamarin-prover-testing/{lib,src}/). Do NOT read /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md. No web access. You MAY: run the Haskell reference via /home/kamilner/tamarin-cleanroom/wellformedness/oracle/wf_oracle.sh (it runs the HS binary on a .spthy and prints its output, wf block included — no --prove needed; pass extra flags after the file), read .spthy example files anywhere under /home/kamilner/tamarin-rs/tamarin-prover/examples/, and read/write freely inside /home/kamilner/tamarin-cleanroom/wellformedness/. No git commits.
DISCIPLINE: log every oracle interaction in the workspace QUERIES.log; record behavioral facts in BEHAVIOR.md with provenance; append a round-5 section to a REPORT file (fold into BEHAVIOR.md if .md creation is blocked). Code comments describe current behavior only. Every behavioral claim traces to a logged oracle probe or a captured target. Return a PLAIN-TEXT summary.

Crate: /home/kamilner/tamarin-cleanroom/wellformedness/workspace/wf-clean (self-contained, own AST; the wf checker logic lives in src/{checks.rs,formula.rs,report.rs}).

THE TASK — your wf checker diverges from the reference on 71 real corpus theories. The exact list is /home/kamilner/tamarin-cleanroom/wellformedness/round5/wf_gate_diffs.txt; each theory's CORRECT reference wf-block is pre-materialized (HS oracle output) at round5/targets/<path-with-slashes-as-underscores>.hs.txt; round5/families.tsv classifies each into one of three families. Make your checker reproduce the reference wf-block for all 71 (and keep every currently-passing theory passing). Probe the oracle freely to characterize correct behavior; the targets are your ground truth.

FAMILY 1 — guardedness OVER-report (49 files). Your checker emits a " Formula guardedness" block ("Lemma \`X' cannot be converted to a guarded formula: unguarded variable(s) …") for lemmas the reference accepts silently (the target blocks have NO guardedness section — they are mostly accountability protocols whose \`accounts for\` lemmas are translated into generated verification lemmas with primed variables like Hp/Sp/sid). Your guardedness DECISION is incomplete: it rejects formula shapes the reference converts successfully. Probe the reference's guardedness acceptance exhaustively — feed it constructed lemmas spanning the shapes in the 49 targets (nested quantifiers, disjunction/implication/negation structure, action-atom guards reachable through various connectives, quantified variables guarded transitively, the translated-accountability shapes) and pin exactly which formulas it accepts as guarded. Rework your guardedness conversion/decision so it accepts everything the reference accepts (and still reports the genuinely-unguarded cases that your existing passing fixtures cover). This is the hard core of the round.

FAMILY 2 — sort/capitalization OVER-report (15 files). Your checker emits "Variable with mismatching sorts or capitalization" where the reference does not (and in some cases duplicates the "Possible reasons" preamble). Probe which variable spellings the reference actually flags as sort/capitalization clashes vs which it accepts; fix the detection to match. Watch for the case where the same variable appears with a suffix-sort spelling in one place and a sigil spelling in another — the reference may treat these as the same variable.

FAMILY 3 — tail (7 files, families.tsv "tail"). Distinct smaller divergences to pin against their targets: (a) a "Lemma annotations" topic the reference emits that you omit; (b) the multi-entry "Facts occur in the left-hand-side but not in any right-hand-side" numbered list — the reference indents each numbered item with THREE leading spaces and separates consecutive entries with a "  " blank line, your rendering uses two spaces / no separators; (c) SAPIC/accountability-deprecated theories where the reference reports MORE checks than you (you under-report). Pin each against its target and fix.

Keep every existing wf-clean test green; add tests/fixtures pinning representative members of each family. Do NOT edit anything outside your workspace.`

const AUDIT = `You are a BOTH-SIDES similarity auditor for the tamarin-rs sealed relicensing campaign (exempt from barrier access rules; you may read everything). Audit ONLY this round's delta for the wellformedness slice.
/home/kamilner/tamarin-cleanroom is a git repo whose HEAD (7980f8d) predates this round: run git -C /home/kamilner/tamarin-cleanroom status/diff restricted to wellformedness/ to see the change. Audit the guardedness/sort-clash/tail fixes against the upstream Haskell — start from lib/theory/src/Theory/Constraint/System/Guarded.hs (formulaToGuarded and the guardedness machinery) and lib/theory/src/Theory/Tools/Wellformedness.hs.
Method: abstraction-filtration-comparison. The guardedness DECISION is behavior-dictated (it must accept exactly what the reference accepts — that is merger, not copying), but flag any copied protectable EXPRESSION: the reference's specific conversion structure/identifier constellations (guardConj/guardEx/openFormula/etc.), internal names, magic constants not observable at the wf boundary, comment lineage. Cross-check that every behavioral claim in the delta's BEHAVIOR.md/QUERIES.log traces to a logged oracle probe or a captured target, not to Guarded.hs.
Append a round-5 section to /home/kamilner/tamarin-cleanroom/wellformedness/AUDIT.md. Violations get a BEHAVIORAL redo instruction (never a patch). End with exactly one line: VERDICT: pass  — or —  VERDICT: fail`

const GATE = `You are the OPEN-SIDE integrator for the tamarin-rs sealed relicensing campaign, working in /home/kamilner/tamarin-rs. Read the wellformedness sections of /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md and PROTOCOL.md ("Full-corpus gates") first.
This round refined the sealed wf checker (workspace wf-clean) to fix 71 corpus wf divergences. Your job: propagate the refinement into the tree and confirm with the full-corpus gate.
${''}
RULES: mechanical re-sync only (the documented recipe: sed crate::->super:: path fixes, drop \`pub mod ast;\`, \`pub use ast::*;\`->\`pub use crate::ast::*;\`, and PRESERVE crates/tamarin-parser/src/wf/order.rs + its two mod-lines in wf/mod.rs). Do NOT transplant logic; do NOT hand-edit the sealed checker; clean files stay headerless (a wf/ file acquiring a GPL header is a provenance violation — stop and report). No git commits.
STEPS:
1. Re-sync the round-5 wf-clean sources (checks/formula/pretty/report/mod) from /home/kamilner/tamarin-cleanroom/wellformedness/workspace/wf-clean/src into crates/tamarin-parser/src/wf/, preserving order.rs. The OPEN-SIDE wf-prep adapters live OUTSIDE wf/ (run.rs let-substitution + normalize_wf_vars, wf_adapt.rs, pretty_theory.rs preamble map + subterm blank-lines, theory_io.rs) — do NOT touch or revert those; only wf/ is re-synced.
2. cargo build --release (PATH needs /home/linuxbrew/.linuxbrew/bin).
3. Run the fast full-corpus gate: RESULTS_TSV=scripts/wf_gate_round5.tsv JOBS=6 bash scripts/wf_gate.sh — report MATCH/DIFF. Compare against the pre-round baseline (71 DIFF); the target is 0 DIFF (or a small, precisely-characterized residual — for any file still DIFF, give its path + the diff head).
4. Regression guard: cargo test -p tamarin-parser -p tamarin-theory -p tamarin-prover -p tamarin-server (report pass counts); scripts/gen_license_headers.py --check (expect 0 stale, and confirm no wf/ file gained a header).
Append a dated round-5 section to INTEGRATION_REPORT.md. Return a plain-text summary with the exact before/after gate numbers and any residual files. Do not delete anything (the ported wf was already removed in an earlier round). If a file still diverges after re-sync, keep-and-report precisely — do NOT hand-fix the sealed code.`

const impl = await agent(IMPL, { label: 'seal:wf-r5', phase: 'Implement', model: 'fable' })
const audit = await agent(AUDIT, { label: 'audit:wf-r5', phase: 'Audit', model: 'opus' })
const passed = /VERDICT:\s*pass/i.test(audit || '')
const gate = passed
  ? await agent(GATE, { label: 'gate:wf-r5', phase: 'Gate', model: 'opus' })
  : 'SKIPPED — audit did not pass; not integrated.'
return {
  impl: (impl || '').slice(0, 3000),
  audit: (audit || '').slice(0, 2500),
  passed,
  gate: (gate || '').slice(0, 3500),
}