# Round 3 (Unit A): handler semantics (the UI state machine)

Extend workspace/web-clean from templates to BEHAVIOR: the handler logic that
decides WHAT each route returns. Sources: the 81 manifests (your route census
covers 48k routes) + live probing (hs_server.sh is buggy; use your own
--port=<n> wrapper as before; ports 3100-3199 are yours).
Model, byte-exact where captures show determinism:
1. Proof-tree traversal semantics: next/prev ordering over proof states,
   path encoding, version increments across edits/autoprove.
2. autoprove variants (the a/b/all/characterization route forms in the
   census): which proof mutation each performs and the response envelope.
3. proof-step application responses, del/path semantics, add-lemma flow.
4. The two integration blockers: (a) the NON-LOCAL-origin page shell (the
   served shell differs when the client is not 127.0.0.1 — probe by
   connecting to the server via the machine's non-loopback interface IP);
   (b) dynamic edit-form textarea rows= (probe lemmas of varying line counts
   to find the rows formula).
ARCHITECTURE: parameterize over a ProverOps callback trait (parse theory,
apply proof method at path, autoprove, pretty-print fragments) — the ported
prover supplies these at integration; you implement the state machine, route
dispatch decisions, and response assembly through your existing templates.
Update BEHAVIOR.md/QUERIES.log; tests with capture fixtures; REPORT3.md +
10-line summary.
