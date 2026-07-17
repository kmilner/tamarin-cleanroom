# Clean-room task: wellformedness checker

Build a standalone Rust crate `wf-clean` in workspace/ that reimplements,
from BLACK-BOX BEHAVIOR ONLY, the wellformedness checking of the
tamarin-prover binary (the oracle).

## What the oracle does
`oracle/wf_oracle.sh <file.spthy>` prints the tool's full output for a
security-protocol theory file. Within that output, wellformedness checking
produces either `/* All wellformedness checks were successful. */` or a
WARNING section listing check failures grouped under underlined topic
headers. Your job: given a parsed theory (data model in
interface/ast_types.rs), produce byte-identical report text and the
structured API in interface/required_api.md.

## Method requirements
- Derive the set of checks, their trigger conditions, their exact message
  strings, their grouping and ORDER purely by experimenting against the
  oracle: write minimal .spthy inputs that trip each check one at a time.
  37 pre-captured outputs are in oracle/captures/; example inputs (usable as
  starting points for mutations) in oracle/examples/. The .spthy language
  syntax may be learned from those examples and the oracle's parse errors.
- Log every oracle query's purpose in workspace/QUERIES.log (one line each).
- Maintain workspace/BEHAVIOR.md: the growing behavioral spec you infer
  (check name, trigger, message template, ordering). This document is a
  deliverable equal in importance to the code.
- Tests: for each check, a fixture .spthy + expected report snippet, asserted
  against your implementation AND spot-verified against the oracle.
- Dependencies: std only. You may NOT parse .spthy yourself; your crate's
  input is the ast_types.rs data model (write test values directly as Rust
  constructors; oracle fixtures verify message/ordering behavior).
