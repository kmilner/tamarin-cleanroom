# Round 3 (Unit B): graph generation completion

Extend workspace/graph-clean from serialization to GENERATION. Sources:
oracle/dot_corpus/ (12k DOT payloads), the manifests' paired json/dot
responses, live probing (your --port= wrapper; ports 3200-3299 are yours).
Close, byte-exact:
1. Record-cell TERM RENDERING: mine (json term structure <-> dot record-cell
   text) pairs at scale; implement rendering in your term model including
   the observed line-wrapping behavior inside cells.
2. Shape vocabulary: the trapezium/invtrapezium (and any other) node shapes
   in the corpus — when each appears; extend your model/serializer.
3. Node-id/port ALLOCATION: infer the global <nN> id/port numbering scheme
   from corpus ordering; implement an allocator reproducing it.
4. Graph CONTENT: from paired json sequents and dot graphs, characterize
   which nodes/edges/clusters a proof state yields (the system->graph
   mapping), over your own input model.
5. SIMPLIFICATION LEVELS: the UI exposes simplification/abbreviation toggles
   in graph URLs (see census); diff level variants in captures or probe live
   to characterize each level's transform; implement.
Update BEHAVIOR.md/QUERIES.log; tests incl. generation fixtures (not just
round-trip); REPORT3.md + 10-line summary.
