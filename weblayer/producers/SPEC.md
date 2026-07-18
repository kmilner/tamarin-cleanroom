# Clean-room task: web UI fragment PRODUCERS

Build a standalone Rust crate `producers-clean` in `workspace/` that
reimplements, from BLACK-BOX BEHAVIOR ONLY, the tamarin-prover interactive web
UI's **fragment producers** — the code that renders pre-computed prover values
into the HTML/JSON response-body CONTENT the (already clean-roomed) dispatch
layer serves. Given a pre-computed value (a content pane, a theory index, a
proof tree, a loaded-theory list, a path), emit the response-body bytes and the
public API in `interface/required_api.md`.

This is the residual GPL hold for the web cluster. The dispatch/state machine,
route grammar, page shells, forms and JSON envelopes are DONE clean-side
(`../workspace/web-clean`, integrated as `web_clean`). What remains ported is
the CONTENT that fills the panes: the proof-script (west) pane, the theory-view
center fragments, the proof-tree HTML, the index/overview rows. These are the
files whose deletion finally drops their upstream citations — and with them
the pseudonymous web authors (Kanakanajm/"Jackie", YannColomb,
Esslingen-Security-Privacy/Schoop). See ../../PROTOCOL.md for the room rules.

## The observable boundary

The HTTP response BODIES of the sanctioned Haskell interactive server. Two
oracles, both permitted:

* `oracle/captured_responses/` — 81 captured crawl manifests (captured OUTPUT),
  each `{ manifest: { "<url>": {kind,status,body} } }` keyed by `sha256(source)`.
  URLs carry the literal token `#` for the version index. `oracle/
  extract_fragments.py` slices any fragment family out of them into byte
  targets. This is the primary corpus.
* `oracle/hs_server.sh start <file.spthy> [port]` — the live server for probing
  ambiguities a single capture leaves open (ports 3100-3199).
  `oracle/examples/` holds the observation-input theories.

A `main/<section>` response is the JSON envelope `{html,title}` wrapping an html
fragment; an `overview` response is the full theory-view page; the producers
supply the CONTENT embedded in each.

### The acceptance gate is SEMANTIC, not byte-identity (read this first)

`scripts/web_parity.sh` compares HS vs RS **structurally**: it canonicalizes
away highlight `<span class="hl_*">` wrappers, `<br/>`, `&nbsp;`, `<pre>`,
attribute/JSON-key order, the DOT serialization, and volatile env fields
(version index, timestamps, temp paths), then diffs what survives — element
structure, visible text, link hrefs + text, form actions, embedded resource
URLs, JSON values, section headings. **Consequence for the producer surface:**
the exact highlight spans and the `&nbsp;`/`<br/>` skin are NOT part of the
acceptance bar; the tag SKELETON, HEADINGS, TITLES, LINK TARGETS, PRESENCE
rules, and VISIBLE TEXT are. Reproduce the capture bytes closely (they are the
reference), but a slice is ACCEPTED when it matches structure + text + links
after canonicalization. (This differs from the pretty cluster's byte gate.)

## Sub-target decomposition (each independently gate-checkable via ALLOWLIST)

Ranked to take ONE cleanly-observable producer at a time. Corpus counts are
fragment captures across the 81 manifests. "Pure render" = a
`value → bytes` function; the CONTENT it embeds is opaque input (see
"Solver-entangled inputs").

| # | sub-target | exercised by (family, count) | isolation / reuse | ported file it retires |
|---|------------|------------------------------|-------------------|------------------------|
| **R1** | **center section fragments + the shared HTML skin** — `main/message` (Signature / Construction / Deconstruction), `main/rules` (Macros / Fact-Symbols / MSR / Restrictions), `main/tactic`, `main/help`; AND the leaf every fragment reuses: entity-escaping, the per-line `<br/>`/`&nbsp;` postprocess, the `{html,title}` / `{redirect}` / `{alert}` envelope | `main/message` 81, `main/rules` 81, `main/tactic` 81, `main/help` 81 | **HIGHEST** — one route → one self-contained fragment; body 100% opaque; the frame + postprocess + envelope is the skin R2/R3 also reuse | part of `handlers/theory_html.rs` |
| **R2** | **proof-script WEST pane** — the theory index: `theory NAME begin`, item links (message/rules/tactic/sources w/ labels + annotations), `add lemma`, per-lemma declaration (name + attrs + quantifier + opaque formula) + proof display + edit/delete/add links, `end` | `overview` 473 (west pane; the lemma declaration is proof-invariant, so the fresh no-prove state is pure-render) | HIGH reuse (present on every page), MODERATE isolation (reuses R1 skin + R5 links; the line grammar is already modeled in `web_clean::proofscript`, so this round is the ASSEMBLY that feeds it) | rest of `handlers/theory_html.rs` (`proof_state`) |
| **R3** | **proof-tree + proof-method HTML** — lay a pre-computed proof tree out as nested HTML: per-node proof-step + remove-step links carrying the opaque method text, `case`/`next`/`qed` grammar, indent, status classes | proof-tree inside proved `overview` panes (396) | MODERATE — reuses R1; the tree SHAPE + method TEXT are pre-computed inputs; the constraint-system / applicable-methods panes are OUT (solver) | `handlers/proof_tree.rs` (`render_proof_tree_html`) |
| **R4** | **welcome / index page + housekeeping** — `/` (core-team frame + loaded-theory table rows + upload banner), the static help block, robots / cancel-ack / invalid-args bodies | `/` (index) + `main/help` static block + housekeeping | VERY HIGH isolation, small; nearly all producer-owned (only per-row name/time/origin opaque) | `handlers/root.rs` |
| **R5** | **theory-path grammar** — parse the wildcard URL segment ⇄ structured path; the percent-decode + underscore-prefix quirks the fragment links use | link hrefs in every fragment | HIGHEST isolation (pure, HTML-free grammar); low LOC | `handlers/path_parse.rs` (**carries Kanakanajm directly**) |
| — | source/message theory echo; `main/cases` (sources); `main/proof`/`main/method`; graph DOT/SVG; `proof-step` | (text / json / dot) | — | — | **STAYS PORTED / other cluster** (see below) |

Recommended order: **R1 → R2 → R3 → R4 → R5**. R1 is the deepest reused leaf
(the skin R2/R3 embed content through); landing it first de-risks the rest,
exactly as the pretty cluster took term-core first.

### Recommended round-1 target: R1 (center section fragments)

Prefer the most isolated, most reused, cleanly-observable family. R1 wins on
every axis:

* **Isolation** — each `main/<section>` fragment is one route → one body; the
  block BODIES are 100% opaque prover content (sliceable straight from the
  capture), so the producer's job is purely the frame: section `<h2>` + monospace
  `<p>`, the presence rules (Signature/Construction/Deconstruction and
  Fact-Symbols/MSR always emitted; Macros/Restrictions vanish when empty), the
  fixed heading + title vocabulary, the leading blank slot, the per-line
  postprocess, and the `{html,title}` envelope. No cross-lemma assembly, no
  proof tree, no path grammar, no solver call in the producer.
* **Reuse** — the postprocess + envelope + `withHeader` framing is the skin R2
  (west pane) and the sources/proof panes all reuse; nailing it first is
  leverage.
* **Byte targets** — 81 deterministic captures per section; `round1/` already
  materializes the 11 curated labels' four fragments, spot-validated: the live
  oracle on `oracle/examples/issue515.spthy` reproduces the captured
  `main/message` body BYTE-IDENTICALLY.
* **Author topology** — it is the first cut into `theory_html.rs`,
  the file whose deletion (with R2/R3) drops its citations.

The mission flagged the proof-script pane as a likely round-1; it is ranked R2
here rather than R1 because its line grammar is already clean-roomed
(`web_clean::proofscript`), it reuses R1's skin, and it needs the R5 link shapes
and pre-computed item annotations — so it composes better AFTER R1. R1 is the
truer isolated leaf.

## Author topology — what actually retires the pseudonymous authors

`gen_license_headers.py` blames each Rust file's cited upstream ranges; an
author's citation disappears only when EVERY Rust file citing its source range
is deleted. The pseudonymous set (Kanakanajm; YannColomb, Esslingen-Schoop) is cited on the
dispatch shells (`theory.rs`, `state.rs`, `lib.rs`, ALREADY clean-covered by
`web_clean` but not yet deletable) AND on these producers. So **no single
producer deletion drops a pseudonymous author** (the round-9 integration
finding); the cluster must land R1–R5 so that `theory_html.rs`, `proof_tree.rs`,
`root.rs`, `path_parse.rs` all delete, at which point the dispatch shells can
delete too and their citations vanish together. R5 additionally carries
**Kanakanajm** on `path_parse.rs` directly. Treat the
per-sub-target "yield" as: it removes one file's citation, contributing to the
joint deletion — not as an independent erasure.

## Solver-entangled inputs — pure render vs. "stays ported / other cluster"

The producers are `value → bytes` ONLY. Some embedded content is COMPUTED by the
ported solver/pretty side; the clean crate RENDERS it, handed pre-computed, and
the computation stays where it is.

CLEANLY REIMPLEMENTABLE (pure render — R1–R5):
* the section frame + postprocess + envelope (R1); the theory-index line grammar
  and lemma-declaration framing (R2); the proof-tree HTML layout given the tree
  + method text (R3); the index page + housekeeping (R4); the path grammar (R5).

OPAQUE INPUT / STAYS PORTED (not `value → bytes`):
* **the pretty-printed content text** — the signature block, the rule/formula/
  method text, the tactic body — is produced by the theory pretty-printer (the
  `pretty` cluster's plain-text surface) and, for solver-derived pieces (intruder
  rules, injective-fact instances, AC-variant comments, source cases), by the
  ported closure/solver. The producer embeds these as opaque strings.
* **the constraint-system pane + applicable-proof-methods listing**
  (`main/proof`/`main/method`) — solver output; only the proof-tree SHAPE is in
  scope (R3), not the per-node system.
* **the source-case listing** (`main/cases`) — saturation/refinement is solver
  work; only its `<h2>/<h3>/static-graph` frame would be producer, and it is
  deferred (heavy solver adjacency).
* **graph DOT/SVG** (`interactive-graph-def`/`intdot`/`graph`) — the graph
  cluster (Unit B); the Rust-only `proof-step` route — no HS counterpart. Both
  OUT.
* **the plain-text theory echo** (`source`/`message` text routes) — the pretty
  cluster's byte surface, not HTML producer.

## Method requirements (per PROTOCOL.md)

* Derive every heading, title, tag skeleton, presence rule, link shape, escaping
  and postprocess by reading the captures and experimenting against
  `oracle/hs_server.sh`; take exact output strings from observed output
  (compatibility content, never memory). Byte targets are captured OUTPUT only.
* Log every oracle interaction's purpose in `workspace/QUERIES.log` (one line).
* Maintain `workspace/BEHAVIOR.md`: the growing behavioral spec (envelope shape,
  postprocess rule, per-family section vocabulary + presence rules, link shapes).
  A deliverable equal in weight to the code.
* Tests: per sub-target, a fixture (input value as Rust constructors) + expected
  fragment snippet, asserted against your impl AND spot-checked against the
  oracle. Integration truth is the corpus web-parity gate.
* Dependencies: std only. You may NOT read the source tree; your input is the
  `interface/fragment_inputs.rs` behavioral shape.

## Acceptance ladder

1. **Fragment fixtures** — per sub-target, constructed-input → observed-snippet
   unit tests (`workspace/producers-clean/tests/`), spot-checked vs the oracle.
2. **Capture-corpus sweep (self-contained)** — for the producers whose input is
   reconstructable from the capture itself (R1 especially): slice the opaque
   sub-parts out of each captured fragment with `oracle/extract_fragments.py`,
   feed them back through the producer, and assert the reassembly matches the
   capture across all 81 manifests (subset via `--only` while iterating). This
   exercises the frame/postprocess/envelope over the whole corpus WITHOUT a
   prover, and is the sealed room's primary iteration gate.
3. **Integration gate (full crawl)** — `scripts/web_parity.sh` (RS_PATH → the
   workspace build, ALLOWLIST → a subset): boots HS + RS interactive servers per
   theory, crawls, and semantic-diffs the manifests. This is the integrator's
   mandatory acceptance check once the clean producers are wired behind the
   dispatch and the ported files are deleted — the FULL-corpus gate green, not a
   fixture subset (the wf-regression lesson).
