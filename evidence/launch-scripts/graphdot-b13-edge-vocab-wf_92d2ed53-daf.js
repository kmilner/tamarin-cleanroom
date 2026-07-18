export const meta = {
  name: 'graphdot-b13-edge-vocab',
  description: 'Sealed graphdot round 13: close the two mechanical clean-model gaps — 3 missing edge-style variants and the record info-port edge anchor; both-sides audit',
  phases: [
    { title: 'Implement', detail: 'sealed edge-vocabulary + info-port anchor (Opus)' },
    { title: 'Audit', detail: 'both-sides similarity audit of the round-13 delta' },
  ],
}

const IMPL = `You are a SEALED implementer under the barrier protocol at /home/kamilner/tamarin-cleanroom/PROTOCOL.md — read it first and comply strictly.
ACCESS RULES: You must NOT read any file under /home/kamilner/tamarin-rs/crates/, and NOT read any Haskell source of tamarin-prover anywhere on disk (notably /home/kamilner/tamarin-rs/tamarin-prover/{lib,src}/ and /home/kamilner/tamarin-rs/tamarin-prover-testing/{lib,src}/). The ONLY Haskell you may read is the SANCTIONED BSD library /home/kamilner/tamarin-cleanroom/graphdot/sanctioned/pretty-1.1.3.6/. Do NOT read /home/kamilner/tamarin-cleanroom/INTEGRATION_REPORT.md, and do NOT read any AUDIT.md anywhere in the cleanroom. No web access. You MAY read .spthy examples under /home/kamilner/tamarin-rs/tamarin-prover/examples/ and the flags map /home/kamilner/tamarin-rs/scripts/file_flags.tsv. No git commits.
DISCIPLINE: log every oracle interaction in workspace QUERIES.log (Session 13); record behavioral facts in BEHAVIOR.md with capture provenance. Preserve OOM guards; live HS servers only on ports 3200-3299; stop everything you start. Return a PLAIN-TEXT summary.

Workspace: /home/kamilner/tamarin-cleanroom/graphdot/ (read/write freely, except AUDIT.md). Crate: workspace/graph-clean. Your captured corpus (oracle/dot_corpus/ payloads and the manifests) is the primary material this round.

THE TASK — round 13, two CAPTURE-OBSERVABLE model gaps in your edge layer (both surfaced by whole-graph regeneration; your roundtrip harness masks them because it preserves parsed edges opaquely):
GAP 1 — EDGE-STYLE VOCABULARY: your EdgeStyle enum covers 8 of the 11 (color, style) combinations present in the captured corpus; the missing three are purple/dashed, dotted/green, and darkorange3/dashed — edges with those attributes are DROPPED by your generation path. Mine the corpus for every (color, style, attribute-set) edge combination (census with counts), extend the vocabulary to cover the full observed set, and render each byte-exactly (attribute order, quoting, spacing per the captures). Witness payload for a dropped-edge case: the capture whose sha-prefix is 01c5db0a7030e664.
GAP 2 — RECORD INFO-PORT EDGE ANCHOR: your edge endpoint model (EndRef) can address a record node but NOT a specific port inside it — the captures contain edges of the form n131:n128 -> n4 (source anchored at an interior info-port of a record). 94% of captured graphs contain at least one such edge (56,990 edges corpus-wide); your regeneration emits n131 -> n4, losing the :port. Extend the endpoint model to carry the optional port anchor, pin from captures: which edges carry ports (source only? target too?), how the port id relates to the record's cell ids, and the serialization bytes. Witness: capture sha-prefix 00082e1d6a47b5af.
Also: pin how these anchors and styles interact with your existing NodeIdAllocator (the port ids come from the same id space — verify against captures).
ACCEPTANCE: an edge-vocabulary census test asserting the full observed (color,style) set renders byte-exactly; roundtrip stays 12022/12022; a NEW regeneration-oriented test that rebuilds the edge set of at least 20 diverse captured graphs from structured form and byte-matches the payload edge sections (incl. info-ports and the 3 new styles); all existing suites green; absent-feature behavior unchanged. BEHAVIOR.md Session-13 sections. Do NOT edit anything outside /home/kamilner/tamarin-cleanroom/graphdot/.`

const AUDIT = `You are a BOTH-SIDES similarity auditor for the tamarin-rs sealed relicensing campaign (exempt from barrier access rules; you may read everything). Audit ONLY this round's delta: /home/kamilner/tamarin-cleanroom HEAD (422b379) contains through round 12 — git -C /home/kamilner/tamarin-cleanroom status/diff restricted to graphdot/ shows the round-13 delta (edge-style vocabulary completion + info-port endpoint anchor).
Audit against lib/theory/src/Theory/Constraint/System/Dot.hs in /home/kamilner/tamarin-rs/tamarin-prover/ (the edge emission and port machinery). Style names/attribute bytes and port syntax are capture-forced merger; verify capture/census provenance for every addition; flag identifier constellations, non-observable constants, structural transcription, comment lineage.
Append a round-13 section to /home/kamilner/tamarin-cleanroom/graphdot/AUDIT.md. Violations get BEHAVIORAL redo instructions. End with exactly one line: VERDICT: pass  — or —  VERDICT: fail`

const impl = await agent(IMPL, { label: 'seal:graph-b13', phase: 'Implement', model: 'opus' })
const audit = await agent(AUDIT, { label: 'audit:graph-b13', phase: 'Audit', model: 'opus' })
return { impl: (impl || '').slice(0, 3000), audit: (audit || '').slice(0, 2500), passed: /VERDICT:\s*pass/i.test(audit || '') }