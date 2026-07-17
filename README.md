# tamarin-cleanroom

Clean-room reimplementation workspace and evidence archive for relicensing
parts of [tamarin-rs]. Three modules of the Rust port (wellformedness
checker, graph/DOT rendering, web layer) were reimplemented by isolated
agents from black-box oracles only — no access to the GPL Haskell source —
under the process documented in `PROTOCOL.md`.

- `wellformedness/`, `graphdot/`, `weblayer/` — per-cluster SPEC, oracle
  harnesses, and the clean workspaces (code, inferred behavioral specs,
  oracle query logs, audit reports).
- `evidence/` — complete tool-call transcripts of every clean-room
  implementer and similarity auditor, the audit tooling and results, and
  `SHA256SUMS` fixing the archive (including items withheld from this
  public mirror for contributor privacy; their hashes are anchored here).
- `INTEGRATION_REPORT.md` — how the clean crates were wired into the port.

Licensing: the clean-room workspaces are original works of their process
(see PROTOCOL.md and AUDIT.md files); no outbound license is granted yet.
`.spthy` files under `*/oracle/examples/` are unmodified example inputs from
[tamarin-prover] (GPL-3.0) used solely as black-box observation inputs.
Captured oracle outputs are program output of the GPL tamarin-prover binary.

[tamarin-rs]: https://github.com/kmilner
[tamarin-prover]: https://github.com/tamarin-prover/tamarin-prover
