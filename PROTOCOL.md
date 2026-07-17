# Clean-room reimplementation protocol — tamarin-rs relicensing

Purpose: independently recreate three modules of tamarin-rs (wellformedness
checker, graph/dot pipeline, web layer) without access to GPL-covered
expression, so the results are original works relicensable at will.

## Rooms
- DIRTY ROOM (Kevin Milner + main-session assistant): has read the GPL Haskell
  source and the ported Rust modules. Produces only: interoperability headers
  (type surfaces, entry-point signatures, comments stripped), oracle wrappers,
  and behavioral bug reports. Never contributes implementation text.
- CLEAN ROOM (isolated agents, one per cluster): receive ONLY the materials
  listed per cluster below. Their complete tool-call transcripts are retained
  as evidence of non-access.

## Clean-room materials (exhaustive)
1. This protocol, the cluster SPEC.md, and interface/ headers (type surfaces +
   boundary signatures only).
2. Black-box oracles: `wf_oracle.sh` (runs the compiled Haskell binary on an
   input file), `hs_server.sh` (serves a theory in the Haskell web UI for HTTP
   probing), pre-captured oracle outputs (oracle/captures/,
   oracle/captured_responses/), and GPL example .spthy files used solely as
   observation INPUTS.
3. Public Rust toolchain and general documentation.

## Forbidden to the clean room
- Reading ANY file under /home/kamilner/tamarin-rs/ — source, comments, git
  history — with the sole exceptions of executing the oracle scripts and
  reading oracle/captured_responses/ (captured program OUTPUT, not source).
- Fetching tamarin-prover source, docs describing its internals, or its GitHub.
- Reproducing any tamarin-prover implementation detail from model memory:
  every behavior must be derived from the oracle, the captures, or the SPEC.
  Exact output strings (warning texts, DOT syntax, HTML) MUST be taken from
  observed oracle output — that is compatibility content, not expression.

## Audit & integration (after implementation)
1. Transcript audit: verify from the retained agent transcripts that no
   forbidden path was accessed.
2. Similarity audit: a reviewer (may see both sides) compares the clean code
   against the Haskell originals; flagged similarities are sent back as
   redo-instructions, never patched directly.
3. Behavioral acceptance: byte-parity corpus must pass.
4. Integration: dirty room may write thin adapters and mechanical renames to
   fit the workspace API, but must not transplant logic from the replaced
   modules. The replaced modules are deleted; their GPL-pending headers go
   with them (history retains them under GPL).
