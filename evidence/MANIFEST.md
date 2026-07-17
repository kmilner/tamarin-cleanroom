# Evidence archive — tamarin-rs clean-room relicensing campaign

Assembled 2026-07-17 from the orchestrating session's persistent stores. Every
agent in this campaign ran under a harness that logs each tool invocation
(file reads, shell commands, web access) to a per-agent JSONL transcript;
those transcripts are collected here verbatim. `SHA256SUMS` covers all files.

## Clean-room implementer transcripts (core evidence)

These agents were REQUIRED to work only from black-box oracles and captured
program output (see ../PROTOCOL.md). Their transcripts prove non-access.
Final audit (audit/cleanroom_audit_results.txt): 7 transcripts, 537 tool
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

Corresponding workspaces (../wellformedness, ../graphdot, ../weblayer) hold
each agent's QUERIES.log, BEHAVIOR.md, REPORT*.md and code — cross-reference
the transcripts against those logs.

## Similarity-audit transcripts (both-sides reviewers, exempt from access rules)

| Path | Role |
|------|------|
| workflows/wf_7197ba11-003/agent-*.jsonl (3) | round-1 audits: wf / graph / web (verdicts: pass, 0 findings) |
| standalone-agents/agent-aac7737a5d3b7d6e7.jsonl | wf round-2 incremental audit (pass, 0) |
| standalone-agents/agent-ac1b65c4df499573b.jsonl | graph+web round-2 incremental audit (pass, 0) |

Findings text: ../wellformedness/AUDIT.md, ../graphdot/AUDIT.md, ../weblayer/AUDIT.md.

## Dirty-room and campaign-research transcripts

| Path | Role |
|------|------|
| main-session-dirty-room.jsonl | the orchestrating session: header generation, oracle/corpus construction, interoperability-header extraction, integration fixes, all audits as run |
| workflows/wf_610d94ab-e29/ | dirty-room integration agent (adapters only; see ../INTEGRATION_REPORT.md) |
| workflows/wf_53f48d60-3ef/ | licensing research (GPL doctrine/precedent/feasibility/literal-copy) |
| workflows/wf_8b8fd87b-72e/ | uncited-file derivation audit + institutional-ownership research |
| workflows/wf_5ec90b24-b26/ | tail-author affiliations, triviality classification, rewrite scoping |
| standalone-agents/agent-a9c7edd67d24711cc.jsonl | contributor-provenance analysis (withheld from public mirror) |

## Supporting material

- launch-scripts/ — the workflow scripts (contain the verbatim prompts given to
  every agent; prompts are also embedded in each transcript's first message).
- task-results/ — result summaries as returned to the orchestrator (rescued
  from volatile /tmp).
- journal.jsonl inside each workflow dir — the workflow engine's own record of
  each agent's launch and structured return value.
- audit/transcript_audit.py — the audit tool; audit/cleanroom_audit_results.txt
  — its final run over all clean-room transcripts.

## Integrity and limitations

- SHA256SUMS (300 files at publication) fixes the archive's content as of 2026-07-17. To
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
