export const meta = {
  name: 'pretty-r5',
  description: 'Sealed pretty round 5: restriction expanded-formula field, macros vertical layout, theory-frame assembly — completing the batch echo; both-sides audit',
  phases: [
    { title: 'Implement', detail: 'sealed echo completion (Opus)' },
    { title: 'Audit', detail: 'both-sides similarity audit of the round-5 delta' },
  ],
}

const IMPL = `You are a SEALED implementer under the barrier protocol at /home/kamilner/tamarin-cleanroom/PROTOCOL.md — read it first and comply strictly.
ACCESS RULES: You must NOT read any file under /home/kamilner/tamarin-rs/crates/, and NOT read any Haskell source of tamarin-prover anywhere on disk (notably /home/kamilner/tamarin-rs/tamarin-prover/{lib,src}/ and /home/kamilner/tamarin-rs/tamarin-prover-testing/{lib,src}/). The ONLY Haskell you may read is the SANCTIONED BSD library /home/kamilner/tamarin-cleanroom/graphdot/sanctioned/pretty-1.1.3.6/. Do NOT read /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md, and do NOT read any AUDIT.md anywhere in the cleanroom. No web access. You MAY read .spthy examples under /home/kamilner/tamarin-rs/tamarin-prover/examples/ and the flags map /home/kamilner/tamarin-rs/scripts/file_flags.tsv. No git commits.
DISCIPLINE: log every oracle interaction in workspace QUERIES.log; record behavioral facts in BEHAVIOR.md with provenance. Preserve OOM guards; at most 6 concurrent oracle invocations. Return a PLAIN-TEXT summary.

Workspace: /home/kamilner/tamarin-cleanroom/pretty/ (read/write freely, except AUDIT.md). Crate: workspace/pretty-clean (R1-R4 done). Re-read BEHAVIOR.md and QUERIES.log first. Oracle: oracle/pretty_oracle.sh <file.spthy> [flags].

THE TASK — round 5, three observable gaps that block full batch-echo assembly (witnesses from corpus-scale measurement):
GAP 1 — RESTRICTION EXPANDED FORMULA: your Restriction model carries ONE formula, rendered for both the statement and the '/* expanded formula: */' comment. The reference prints the statement in MACRO form (as written) but the comment in macro-EXPANDED form — two different formula values. Witness: features/macros/MacroInLemmasAndRestrictions.spthy. Extend the model with a second (opaque, caller-supplied) expanded-formula input and render each in its place; probe whether the same statement/expanded split applies to LEMMAS with macros (check the witness + constructed probes) and model accordingly.
GAP 2 — MACROS BLOCK LAYOUT: your macros_doc renders the whole block on one line when it fits. The reference ALWAYS breaks: the first macro sits beside 'macros:' and every subsequent macro goes on its own line at indent 8, regardless of fit. Same witness file; probe multi-macro theories (2, 3, many; short and long) to pin the exact columns, separators (commas? where?), and the single-macro case.
GAP 3 — THEORY FRAME: your theory::render entry point is unimplemented. Implement the full echo assembly: 'theory NAME begin' ... 'end' — the exact blank-line rhythm and ITEM ORDERING between blocks (signature, macros, rules with their variants, restrictions, lemmas, and any interior sections the echo carries between them — study whole echoes across your round1-3 target files plus fresh diverse captures; the wellformedness comment and Generated-from stamp are OUT of your span per SPEC.md). Every block renders via your existing R1-R4 entry points; the frame contributes ordering, headers like 'restriction NAME:' vs 'lemma NAME:' (already in wrappers), and inter-block spacing. Acceptance for this gap: WHOLE-ECHO byte parity (minus the excluded trailing comments) for at least 15 diverse corpus files spanning all block types, asserted in tests against oracle targets.
ACCEPTANCE overall: all existing suites green; new tests per gap; BEHAVIOR.md updated with each law. Do NOT edit anything outside /home/kamilner/tamarin-cleanroom/pretty/.`

const AUDIT = `You are a BOTH-SIDES similarity auditor for the tamarin-rs sealed relicensing campaign (exempt from barrier access rules; you may read everything). Audit ONLY this round's delta: /home/kamilner/tamarin-cleanroom HEAD (8341919) contains pretty rounds 1-4 — git -C /home/kamilner/tamarin-cleanroom status/diff restricted to pretty/ shows the round-5 delta (restriction expanded-field, macros layout, theory-frame assembly).
Audit against the upstream Haskell in /home/kamilner/tamarin-rs/tamarin-prover/ (prettyOpenTheory/prettyClosedTheory and the theory-item traversal; the macro block printer; restriction printing). Item ordering and spacing are byte-forced merger — verify whole-echo capture provenance; scrutinize the frame TRAVERSAL structure (must not mirror upstream's item-fold decomposition beyond what output forces); flag identifier constellations, non-observable constants, comment lineage.
Append a round-5 section to /home/kamilner/tamarin-cleanroom/pretty/AUDIT.md. Violations get BEHAVIORAL redo instructions. End with exactly one line: VERDICT: pass  — or —  VERDICT: fail`

const impl = await agent(IMPL, { label: 'seal:pretty-r5', phase: 'Implement', model: 'opus' })
const audit = await agent(AUDIT, { label: 'audit:pretty-r5', phase: 'Audit', model: 'opus' })
return { impl: (impl || '').slice(0, 3000), audit: (audit || '').slice(0, 2500), passed: /VERDICT:\s*pass/i.test(audit || '') }