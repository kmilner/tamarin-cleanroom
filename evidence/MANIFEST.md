# Evidence archive — tamarin-rs clean-room relicensing campaign

Assembled 2026-07-17 from the orchestrating session's persistent stores. Every
agent in this campaign ran under a harness that logs each tool invocation
(file reads, shell commands, web access) to a per-agent JSONL transcript;
those transcripts are collected here verbatim. `SHA256SUMS` covers all files.

## Clean-room implementer transcripts (core evidence)

These agents were REQUIRED to work only from black-box oracles and captured
program output (see ../PROTOCOL.md). Their transcripts prove non-access.
Final audit (audit/cleanroom_audit_results.txt): 22 transcripts, 1893 tool
calls, 0 forbidden accesses, 0 web accesses.

| Path | Agent | Role |
|------|-------|------|
| workflows/wf_d28aad0e-98c/agent-a169c9fd75138f333.jsonl | round 1 | wellformedness implementer |
| workflows/wf_d28aad0e-98c/agent-a29d812038b601a9c.jsonl | round 1 | graph/dot implementer |
| workflows/wf_d28aad0e-98c/agent-a9addc4f0d5b97ed0.jsonl | round 1 | web-layer implementer |
| workflows/wf_d28aad0e-98c/agent-a4b479b960c441512.jsonl | round 1 cont. | web-layer continuation |
| workflows/wf_6e112628-91d/agent-ae38332cc1eb5f790.jsonl | round 2 | wellformedness gap-closer |
| workflows/wf_db8cc474-1b6/agent-aa7e30d4b753e4fb7.jsonl | round 2 | graph abbreviation-selection miner |
| workflows/wf_db8cc474-1b6/agent-acd8fb9a8d13fc15f.jsonl | round 2 | web page-generality |
| workflows/wf_7f807521-670/agent-a27fa6413061022ae.jsonl | round 3 (unit C) | wellformedness completion |
| workflows/wf_7f807521-670/agent-ad2408a9cda7bbd15.jsonl | round 3 (unit D) | console/CLI implementer |
| workflows/wf_7f807521-670/agent-a2df8d6d0f01ac441.jsonl | round 3 (unit E) | macro-expansion implementer |
| workflows/wf_7f807521-670/agent-a842511ba769df7f3.jsonl | round 3 (unit F) | injective-facts implementer |
| workflows/wf_7f807521-670/agent-a4c19ed19405e64bb.jsonl | round 3 (unit G) | derivation-checks implementer |
| workflows/wf_7f807521-670/agent-a7d8ac2b487df1e33.jsonl | round 3 (unit A) | web interactive-UI state machine |
| workflows/wf_7f807521-670/agent-ac1c391bc2c48679f.jsonl | round 3 (unit B) | graph generation implementer |
| workflows/wf_bfd5a6a0-31b/agent-abad7225bcac84f4e.jsonl | round 3 cont. (unit F) | corpus-validation rebuild (36/36 agree) |
| workflows/wf_62e2c0ef-a3e/agent-ae1a3ead3cabd0a4c.jsonl | round 4 (unit B) | simplification/wrap re-probe |
| workflows/wf_1741f4ad-48d/agent-ad91aafe765192f45.jsonl | round 4 (unit C) | sort-aware binding + semantic guardedness |
| workflows/wf_1741f4ad-48d/agent-ac163fe10def094e0.jsonl | round 4 (unit D) | typed parse + stream-aware framing |
| workflows/wf_1741f4ad-48d/agent-aeaa95d99395a0136.jsonl | round 4 (unit E) | nullary/acc-lemma/case-test expansion |
| workflows/wf_1741f4ad-48d/agent-af76260f88f21bcaa.jsonl | round 4 (unit G) | report contract, ordering, scope, batching |
| workflows/wf_1741f4ad-48d/agent-ae1942fd5bd87afee.jsonl | round 5 (unit B) | clusters, wrap wiring, temporal nodes, RawRule entry |
| workflows/wf_1741f4ad-48d/agent-a92e0a97d8d241648.jsonl | round 4 (unit A) | full route surface + state machine |

Corresponding workspaces (../wellformedness, ../graphdot, ../weblayer) hold
each agent's QUERIES.log, BEHAVIOR.md, REPORT*.md and code — cross-reference
the transcripts against those logs.

## Similarity-audit transcripts (both-sides reviewers, exempt from access rules)

| Path | Role |
|------|------|
| workflows/wf_7197ba11-003/agent-*.jsonl (3) | round-1 audits: wf / graph / web (verdicts: pass, 0 findings) |
| standalone-agents/agent-aac7737a5d3b7d6e7.jsonl | wf round-2 incremental audit (pass, 0) |
| standalone-agents/agent-ac1b65c4df499573b.jsonl | graph+web round-2 incremental audit (pass, 0) |
| workflows/wf_bfd5a6a0-31b/agent-a44d06c824ea88b18.jsonl | round-3 audits: console/macros/injective/derivcheck (all pass, 0) |
| workflows/wf_bfd5a6a0-31b/agent-adb12daa8ab777100.jsonl | round-3 audits: weblayer + graphdot deltas (pass, 0) |
| workflows/wf_bfd5a6a0-31b/agent-a69817b344aadd7bd.jsonl | round-3 audit: wellformedness delta (pass, 0) |
| workflows/wf_62e2c0ef-a3e/agent-af3d04b49cd4b056d.jsonl | round-4 audit: graphdot re-probe delta (pass, 0; notes clean side's "levels inert" conclusion contradicts the source — affirmative non-access evidence) |
| workflows/wf_1741f4ad-48d/agent-a*.jsonl (6 auditor transcripts among 14) | round-4/5 per-unit delta audits: C/D/E/G/B/A (all pass, 0 findings; D audit notes the clean flag-precedence order DIVERGES from the source Either-applicative order — affirmative non-access evidence) |

Findings text: ../wellformedness/AUDIT.md, ../graphdot/AUDIT.md, ../weblayer/AUDIT.md,
../console/AUDIT.md, ../macros/AUDIT.md, ../injective/AUDIT.md, ../derivcheck/AUDIT.md.

## Dirty-room and campaign-research transcripts

| Path | Role |
|------|------|
| main-session-dirty-room.jsonl | the orchestrating session: header generation, oracle/corpus construction, interoperability-header extraction, integration fixes, all audits as run |
| workflows/wf_610d94ab-e29/ | dirty-room integration agent (adapters only; see ../INTEGRATION_REPORT.md) |
| workflows/wf_62e2c0ef-a3e/agent-ac4e655d83d5c3d56.jsonl | dirty-room integrator: units A+B re-sync (rewire blocked, see ../INTEGRATION_REPORT.md) |
| workflows/wf_62e2c0ef-a3e/agent-a2eac17c5efa35c13.jsonl | dirty-room integrator: units C–G (partial; keeps reported per protocol) |
| workflows/wf_1741f4ad-48d/ (2 integrator transcripts among 14) | round-4/5 integrators: theory C-swap+delete (check_terms.rs removed, 134->133) and server A/B (blocked; edit-form closure byte-verified) |
| workflows/wf_53f48d60-3ef/ | licensing research (GPL doctrine/precedent/feasibility/literal-copy) |
| workflows/wf_8b8fd87b-72e/ | uncited-file derivation audit + institutional-ownership research |
| workflows/wf_5ec90b24-b26/ | tail-author affiliations, triviality classification, rewrite scoping |
| standalone-agents/agent-a9c7edd67d24711cc.jsonl | contributor-provenance analysis (withheld from public mirror) |

## Supporting material

- launch-scripts/ — the workflow scripts (contain the verbatim prompts given to
  every agent; prompts are also embedded in each transcript's first message).
  Exception: the round-3 implementation wave (wf_7f807521-670) has no persisted
  launch script; its prompts survive only in the transcripts' first messages.
- task-results/ — result summaries as returned to the orchestrator (rescued
  from volatile /tmp).
- journal.jsonl inside each workflow dir — the workflow engine's own record of
  each agent's launch and structured return value.
- audit/transcript_audit.py — the audit tool; audit/cleanroom_audit_results.txt
  — its final run over all clean-room transcripts.

## Integrity and limitations

- SHA256SUMS (300 files at publication; refreshed to 727 files the same day
  after the round-3/4 waves — every original entry retained, new cluster
  workspaces and transcripts added) fixes the archive's content as of 2026-07-17. To
  timestamp-anchor it, commit this directory (or just SHA256SUMS) to a git
  repository and/or push to a remote — the commit chain provides an ordered,
  hard-to-backdate record. OpenTimestamps is a stronger option if wanted.
- These transcripts are harness-produced records under the operator's control,
  not third-party attestations. Their evidentiary weight comes from being
  contemporaneous, complete (every tool call), internally consistent with the
  workspaces' QUERIES.log/BEHAVIOR.md, and fixed by hash before any dispute.
- One known caveat is documented in ../PROTOCOL.md: the implementing model's
  training data may contain the GPL source; the mitigations (no-recall
  instruction, independent both-sides similarity audits, divergence evidence
  recorded in the AUDIT.md files) are part of this record.

## Public-mirror note (added at publication)

The public mirror at github.com/kmilner/tamarin-cleanroom withholds the
campaign-research transcripts, the orchestrating-session transcript, and the
contributor-analysis outputs (privacy: they de-pseudonymize contributors).
Withheld files remain in the local archive and are integrity-anchored by
their entries in SHA256SUMS, which is published in full.
