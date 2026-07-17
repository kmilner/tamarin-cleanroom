# Unit F: injective fact instances

Build crate injfacts-clean in workspace/. The binary computes which protocol
facts are "injective" during precomputation; `oracle/hs_oracle.sh <file>
--precompute-only` prints the precomputation — find where injective facts
appear in that output (and/or in normal output/web captures) and use it as
your labeled oracle. Characterize the decision: for each fact tag, when is it
injective (probe: rules where a fact is created with a fresh name, consumed,
re-produced, multiple instances, etc. — design minimal probe theories).
Input model: rules from ../wellformedness/interface/ast_types.rs. Deliverable:
injective_fact_instances(rules) -> set of fact tags, corpus-validated against
oracle/examples/ (37 theories) plus your probes; byte-match any printed
representation you rely on. BEHAVIOR.md + QUERIES.log + tests; REPORT.md +
10-line summary.
