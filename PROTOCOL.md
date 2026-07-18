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
3. Behavioral acceptance: the FULL-CORPUS gate for the slice must pass, not a
   fixture subset (see "Full-corpus gates" below). A narrow fixture suite that
   passes while the corpus gate fails is not acceptance.
4. Integration: the open side may write thin adapters and mechanical renames to
   fit the workspace API, but must not transplant logic from the replaced
   modules. The replaced modules are deleted; their GPL-pending headers go
   with them (history retains them under GPL). Every integrator that touches a
   slice with a full-corpus gate MUST run that gate before declaring green.

## Full-corpus gates (standard, not optional)

Each slice has a fast oracle gate over the whole 419-file example corpus,
diffing the slice's output against the Haskell reference cache. These are the
sealed implementer's primary data source (run between build and the HS cache
every iteration) AND the integrator's mandatory acceptance check. Both take
`RS_PATH` (point at any build) and `ALLOWLIST` (subset while iterating).

- Wellformedness: `scripts/wf_gate.sh` — runs the binary WITHOUT `--prove`
  (the wf block prints at theory-load, ~1s/file; full corpus ~72s at JOBS=6),
  extracts the `/* WARNING … */` block, diffs vs `scripts/.hs_file_cache`.
- Web: `scripts/web_parity.sh` — crawls HS and RS interactive servers per
  theory, semantic-diffs the response manifests vs `scripts/.web_hs_cache`.
- Full batch (proof output, slow ~30-60min): `scripts/corpus_file_diff.sh` —
  the ground-truth `--prove` byte diff; run at milestones, not every iteration.

Once a slice's ported code is DELETED, the integrated module IS the sealed
reimplementation, so running the gate against the tree observes the sealed
code's own output vs the reference — legitimate feedback, not observation of
replaced code. For a not-yet-integrated slice the tree still holds the ported
code, so gate only the workspace build (via `RS_PATH`), never the stock tree.
