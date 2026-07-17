# REPORT.md — clean-room web layer (`web-clean`)

Final report for the `weblayer` cluster. Everything below was derived from
black-box observation only: the 81 pre-captured crawl manifests in
`oracle/captured_responses/` (captured program OUTPUT) and live probing of the
sanctioned oracle binary (`QUERIES.log` [L0]–[L6]). No file under
`/home/kamilner/tamarin-rs/` was read; the only tamarin-rs touch was EXECUTING
the sanctioned oracle binary for live probing.

## Components (crate `web-clean`, std + serde/serde_json only)

| module | LOC | responsibility |
|--------|----:|----------------|
| `route.rs` | 217 | parse `/thy/<kind>/<index>/<handler>/<args…>` into a structured route (descriptive grammar; not a dispatcher) |
| `envelope.rs` | 65 | the two JSON response shapes `{"html","title"}` / `{"redirect"}` (compact, fixed key order, no trailing newline) |
| `escape.rs` | 54 | HTML entity escaping (`& < > " '` → 5 entities; UTF-8 pass-through) |
| `page.rs` + `shell_template.rs` | 59 | full theory-view HTML shell (`overview/*`), 4 slots: NAME/IDX/VERSION/FILENAME |
| `proofscript.rs` | 317 | proof-script (west) pane: header, item links, add/edit/delete links, `by sorry`, and the solved proof-tree line grammar (step/case/next/qed/by, indentation, status wrapper) |
| `forms.rs` | 119 | edit / delete / add-lemma form bodies |
| `intdot.rs` | 55 | `intdot` mini-page + empty-graph DOT skeleton |
| `errors.rs` + `notfound_template.rs` | 37 | 404 Not Found page |
| `text.rs` | 47 | plain-text bodies (`source`/`message`, `next`/`prev`) + proof URL builder |
| `lib.rs` | 35 | module map / crate docs |
| **src total** | **1005** | |
| `tests/parity.rs` | 334 | byte-parity tests against captured fixtures |

Byte-exact scaffolding copied from observed oracle output (compatibility
content) lives in `shell_template.rs` and `notfound_template.rs`.

## Tests

`cargo test` → **37 passing, 0 failing** (19 unit + 18 integration/parity),
`cargo clippy` clean.

Parity coverage (byte-identical against captured/observed output):
- JSON envelopes: **2450 distinct bodies** deduped across all 81 manifests
  (1583 content + 867 redirect), plus every body of the Chaum and issue515
  manifests. Reproduced byte-for-byte by `serde_json` default serialization.
- Full theory-view page: 3 pages — `overview/help` for two theories, and
  `overview/proof/exec` (a proof view at version 3, a distinct page type/index).
- Proof-script west pane: unproven (0-lemma issue515, 2-lemma Chaum) **and** a
  fully solved 2-lemma proof tree (40 proof lines with branching cases,
  `next` separators, `qed` nesting to depth 5, and by-prefixed final steps).
- Forms (edit/delete/add, incl. `<first>` percent/entity encoding), intdot
  mini-page, empty DOT, 404 page, source/next text pass-through.
- Unit tests: route grammar, escaping, envelope shapes, proof-line grammar.

The solved-tree test feeds the crate's own observation model
(`proof_lines_{exec,unforgeability}.json`) and the crate's URL builder through
`render_proof_script`, asserting the whole west pane byte-for-byte; only the
proof-method HTML and case names are treated as opaque prover fragments.

Live probe [L6] independently corroborated the model on a *fresh* theory
(NSLPK3): index page, `overview/help` (version 1.13.0), `next`/`prev` text
targets, the `autoprove` redirect with its index bump (1→2), the `main/proof`
`{html,title}` envelope with `&#39;` escaping, a redirect JSON with no trailing
newline, `%23` index → 404, and an empty DOT that is **byte-identical** (`cmp`)
to the committed fixture from a different theory.

## Oracle query count

QUERIES.log records **29** logged interactions: 23 manifest-exploration steps
([001]–[023], source A) + 6 live-server steps ([L0]–[L6], one a tooling-bug
finding, source B). The `hs_server.sh --port` parsing bug ([L0]) was worked
around with a local wrapper (`scratchpad/wl_server.sh`, `--port=<n>` form).

## Work done this (continuation) session

1. Verified the inherited crate builds and all inherited parity tests actually
   execute and pass (the reported "0 tests" was stale — fixtures were already
   wired; confirmed 35 → now 37).
2. Enumerated the authoritative handler/kind census over all 48 824 dynamic
   routes and folded it into BEHAVIOR §1 (autoprove/next/prev/overview shapes).
3. Found and reproduced the **solved proof tree** (the named coverage gap):
   discovered the proven-lemma header wrapper (`<span class="hl_good">…</span>`
   spanning decl + edit/delete line), refined the `Proof` model accordingly,
   parsed the two Chaum proofs into a JSON observation model, and added two
   new byte-parity tests (solved west pane + proof-view page).
4. Live-corroborated the route/envelope model on a fresh theory; fixed clippy.
5. Extended BEHAVIOR.md (§1 census, §6 proved-header wrapper, §11 coverage) and
   QUERIES.log; wrote this report.

## Gaps / notes (for the similarity audit + later integration)

- **Prover fragments, by design out of scope**: pretty-printed signatures,
  rules, lemma formulas, constraint systems / "Applicable Proof Methods" center
  bodies, proof-method HTML, case names, source cases, non-empty DOT graphs,
  wf-warning text, and the theory `source` string. The crate reproduces the
  web-layer scaffolding *around* these and takes them as opaque inputs.
- **Not byte-tested (non-deterministic)**: the index page rows (load timestamp,
  temp origin path) and the source footer (compile time, git rev, Maude
  version). Structure documented in BEHAVIOR §7/§10; template not asserted.
- **Unobserved, hence unimplemented**: the `diff` theory-kind (only `trace`
  appears); proof-status classes other than `hl_good` / `sorry-step`
  (`hl_bad`/`hl_dead` are plausible for falsified/dead but never captured);
  the `POST` upload/reload/append/download bodies (seen only as emitted link
  targets). `render_proof_line` accepts an arbitrary status string, so
  falsified/dead colors need only a caller supplying the class — no code change.
- **Integration**: render functions take an explicit input model (version index,
  version string, filename, timestamps, and all prover fragments), so a caller
  supplying the observed values reproduces the observed bytes. Adapters mapping
  the workspace's internal types onto this model are a later (dirty-room) step.
