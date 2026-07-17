# Clean-room task: constraint-system graph rendering (dot + JSON)

Build, from BLACK-BOX BEHAVIOR ONLY, a standalone crate `graph-clean` in
workspace/ that reproduces the graph outputs of the tamarin-prover web UI:
the graphviz DOT text and/or JSON graph payloads served for a proof state,
including node ABBREVIATION and graph SIMPLIFICATION/clustering behavior.

## Oracles
- oracle/captured_responses/ -> 81 crawl manifests of the Haskell web UI
  (JSON: url->response for whole theories; explore the format yourself).
  These are captured program OUTPUT — your primary corpus.
- oracle/hs_server.sh start <file.spthy> <port> serves a live instance for
  targeted probing (curl). Example inputs: ../wellformedness/oracle/examples/.

## Deliverables
1. workspace/BEHAVIOR.md — precise observed spec: graph payload format, node/
   edge structure, how node labels are abbreviated (rules, thresholds), what
   simplification levels do, cluster structure, exact DOT/JSON syntax.
2. `graph-clean` implementing the transforms over YOUR OWN graph data model
   (design it from observed payloads; an integration adapter will be written
   later by others). Priority: abbreviation + simplification transforms with
   tests against captured payload pairs.
3. workspace/QUERIES.log as per protocol.
Dependencies: std + serde/serde_json only.
