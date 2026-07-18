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

---

## Round 5 — del/path and verify theory-path routes (the last two families)

Rounds 1–4 are captured in REPORT2/3/4.md. Round 5 closes the two families the
round-4 report had logged as a NEGATIVE result — `del/path` and `verify` — after
`round5/ORACLE_NOTES.md` flagged that negative as a probe-shape artifact and staged
four genuine captured reference responses. Black-box only: live probing ([R50]–
[R57], ports 3100–3105, RevealingSignatures/issue193.spthy for trace, KCL07-UK1
diff theory for equiv) + the four staged captures. No file under
`/home/kamilner/tamarin-rs/` was read. All live servers stopped.

### The round-4 negative was a PROBE-SHAPE ARTIFACT — resolved

`del/path` and `verify` ARE registered in 1.13.0. The round-4 sweep ([R47]) used
bogus segments (`x`/`y`/`z`) that never parse as theory paths, so every probe hit
the "unparseable → route miss → 404" branch. Both take a further **theory
sub-path**; a parseable theory-path with a wrong method answers **405** (not 404) —
the registration proof the sweep missed. BEHAVIOR.md §14.8 is rewritten with the
resolution; the honest-negative note is retired.

### Behaviour pinned down (BEHAVIOR.md §15)

- **Shape / method / parse ordering.** `GET del/path/<theory-path>` and
  `GET verify/<theory-path>`, both GET-only. Route match = a PathMultiPiece parse
  of the theory-path; method dispatch happens AFTER it: parseable + non-GET → 405
  (same page as the other error routes); unparseable → 404 (echoing the full
  request path), for any method.
- **Mode-dependent grammar.** trace accepts help/message/rules/tactic/cases/lemma/
  proof/method/add/edit/delete; equiv accepts help/diffrules/diffProof/diffMethod.
  A head from the wrong mode → 404.
- **verify** — trace-only (absent for any method in equiv), never mutates or bumps.
  `proof/<present-lemma>[/path]` → `{redirect}` to `overview/proof/…` (verbatim
  path, same index; predicate = lemma existence — a bogus sub-node still
  redirects); every other path (incl. `proof/<absent-lemma>`) → the `{html,title}`
  help pane (byte-identical to `main/help`).
- **del/path** — a PROOF OPERATION: a deletable lemma/proof node allocates a fresh
  version off the same global monotonic counter as method/autoprove (base retained;
  the deletion persists in the new version), redirecting to `overview/` + the
  verbatim path. Non-deletable paths answer a JSON `{alert}` whose string is
  selected by the PATH TYPE: lemma-removal-failed / proof-step-removal-failed /
  "Can't delete the given theory path!" (the last for every non-lemma/non-proof
  head). Alerts allocate no version. Reconciles the §13/§14 version model: del/path
  is a proof op, NOT an in-place structural edit.

### Deliverables

- `src/route.rs` — `Handler::DelPath` / `Handler::Verify`; `Route::parse` handles
  `del` (requires the `path` literal) and `verify`; new `ThyPath` enum + mode-aware
  `parse` (trace/diff theory-path grammar). +2 route unit tests.
- `src/envelope.rs` — the three del/path alert constants.
- `src/dispatch.rs` — `Server::del_path` / `del_proof` / `verify`, the
  `overview_lemma_path` builder, two new `thy()` arms (del/path both kinds; verify
  trace-only → 404 in equiv). `ProverOps` grown by three callbacks: `lemma_present`,
  `del_lemma_path`, `del_proof_step`.
- `tests/dispatch5.rs` (14 tests) — the four staged captures, redirect-vs-help and
  deletability/alert rules, method/parse ordering, mode asymmetry, version-model
  reconciliation. The round-3/4 `FakeProver` impls gained the three callbacks; all
  prior tests unchanged in intent.
- BEHAVIOR.md §14.8 (resolution) + §15; QUERIES.log [R50]–[R57].

`cargo test` → 91 passing (26 unit + 15 dispatch + 17 dispatch4 + 14 dispatch5 + 19
parity), `cargo clippy --tests --examples` clean. Every asserted body was
live-verified byte-for-byte against a fresh oracle (9/9 parity checks: verify
redirect/help, the three del/path alert strings, del/path lemma+proof redirects,
the equiv diffProof redirect/alerts).

### Honesty / scope notes

- `ProverOps` returns opaque results (lemma presence + deletability `Option`s); the
  tests feed a lemma-existence model so the asserted bytes are the WEB-LAYER
  assembly (envelope/status/redirect/alert-selection/version), never re-derived
  prover logic. verify's help pane is the prover's `main/help` fragment; the tested
  claim is that verify round-trips that same envelope unchanged for every
  non-redirecting path.
- `HttpMethod` still models only GET/POST (round-4 limitation); the reference also
  answers 405 for PUT/DELETE on these routes — documented, unchanged from round 4.
- What each deletion removes from the theory (lemma vs proof subtree) is a prover
  concern held opaque behind the `Option`-returning callbacks; the web layer only
  decides version allocation, redirect target, and alert string.

---

## Round 6 — origin-aware page shell + state delegation

Two adoption-critical items. Black-box only: live probing ([R60]–[R63], ports
3110-3112, Tutorial.spthy trace + KCL07-UK1 diff theory) plus four committed
captures. No file under `/home/kamilner/tamarin-rs/` was read. All servers stopped.

### Item 1 — origin-aware page shell (closes the round-3 [L15] honest negative)

Round 4's working `POST /` upload made the origin distinction observable at last.
Loading a theory from the **command line** (on-disk / **Local**) vs uploading it via
**`POST /`** (no on-disk file / **Upload**) changes the `overview` shell in **exactly
two** north-bar items — nothing else, and the HTTP headers are identical modulo the
derived `Date`/`Content-Length` ([R60], index-normalized full-page diff):

* **"Reload file"** (a POST form to `reload`, between *Index* and *Actions*) — present
  iff **Local**.
* **"Append modified lemmas to file"** (last *Actions* item) — present iff **Local AND
  trace**.

Full kind×origin matrix pinned ([R61]); origin is **inherited** through proof ops
([R62]), so it is a per-version theory property (`Meta.origin`), not a web-layer
choice. This reconciles round-4 [R48] (the equiv shell's absent Append is the kind
gate; its present Reload was the Local command-line load).

Threaded through rendering: `page::Origin { Local, Uploaded }` is a `PageParams`
field; `shell_template` gained a `§RELOAD§` slot (`RELOAD_ITEM`) beside `§APPEND§`;
`page::reload_item` / `page::append_item` fill them; `Server::get_overview` passes
`meta.origin`. **Fixtures** for all four cells
(`r6_overview_{trace,equiv}_{local,upload}.html`) with byte-parity tests
(`overview_shell_*`, 4) plus a dispatch-level thread-through and origin-inheritance
test (`dispatch6.rs`). Fresh-server determinism cross-check [R63] confirmed the
deterministic shell prefix/tail reproduce the fixtures.

### Item 2 — state-delegation redesign (`StateOps` backend)

`Server` no longer owns the version `BTreeMap`/counter. Every state operation it
performed is extracted to a **`StateOps`** trait so the consumer's asynchronous,
internally-caching backend can remain the single owner of theory state:

* `insert_new` (monotonic fresh-version allocation), `get` (retrieval by index),
  `replace` (in-place mutation for edit/reload), `entries` (enumeration for the root
  page), `remove` (deletion).
* `Server<P: ProverOps, S: StateOps<Theory = P::Theory> = InMemoryState<P::Theory>>`
  keeps ALL dispatch/transport/envelope logic and drives state only through `S`.
* The probed lifecycle (monotonic allocation, retention, in-place-vs-new-version) is
  the **documented contract** on the trait; `InMemoryState<T>` is the in-memory
  reference impl. `Server::new(ops, base)` uses it (base at index 1);
  `Server::with_state(ops, state)` injects a custom backend.

All prior tests keep running byte-identical (they resolve to the default
`InMemoryState`). `dispatch6.rs` adds a custom-backend dispatch test (proving the
Server is generic over the state owner) and `InMemoryState` contract tests
(monotonicity, retention, in-place replace, remove).

**Choice + honesty.** Separate `StateOps` (not folded into `ProverOps`) because the
interop story has the state backend as a distinct async component from the prover
fragment producer; the two share `type Theory`. `remove` (deletion) is on the trait
for backend ownership completeness but is **not invoked by any current route** — under
the observed retention invariant the web layer never drops a version; it is documented
as a contract-surface method, not a live path. The trait is a synchronous facade (its
`get` hands out a borrow); the consumer's async cache adapts behind it — the contract
this round pins is the *lifecycle*, which is what the observed behaviour constrains.

### Deliverables

- `src/page.rs` — `Origin` enum; `PageParams.origin`; `reload_item`/`append_item`.
- `src/shell_template.rs` — `RELOAD_ITEM` + `§RELOAD§` slot (Reload item was
  previously hardcoded); `APPEND_ITEM`/`§APPEND§` now gated on (kind, origin).
- `src/dispatch.rs` — `StateOps` trait + `InMemoryState`; `Server<P, S>` refactor
  (`new`/`with_state`); `Meta.origin`; `get_overview` threads origin.
- `tests/parity.rs` (+4 origin byte-parity tests, +`assert_shell_reproduces`),
  `tests/dispatch6.rs` (7 tests), four `r6_overview_*` fixtures; the round-3/4/5
  `FakeProver`s gained `Meta.origin`.
- BEHAVIOR.md §16; QUERIES.log [R60]–[R63].

`cargo test` → **102 passing** (26 unit + 15 dispatch + 17 dispatch4 + 14 dispatch5 +
7 dispatch6 + 23 parity), `cargo clippy --tests --examples` clean. Every ITEM-1 body
was live-captured and reproduced byte-for-byte; the fresh-server determinism check
confirms reproducibility.

---

## Round 7 — concurrency-safe dispatch (snapshot → compute → commit)

Adoption blocker: the round-6 `Server::dispatch(&mut self)` required exclusive
ownership per request, so behind one lock a long `autoprove` (seconds-to-minutes)
would freeze the whole server. Black-box only: live probing ([R70]–[R76], ports
3100/3101, Tutorial/NSLPK3/NAXOS_eCK/RYY_PFS trace + the `sapic/slow` PKCS11 theory
as the long op) plus three committed probe scripts (`scratch/r7/*.sh`). No file under
`/home/kamilner/tamarin-rs/` was read. All live servers stopped.

### STEP 1 — the reference's concurrency contract (probed; BEHAVIOR §17)

Using PKCS11 `cannot_obtain_key_ind` (~30s, terminating) as an in-flight long op:

* **Fully non-blocking ([R71]).** A burst fired ~2.5s into the 30s autoprove — reads
  on other theories, reads on the SAME theory being proved, reads of other versions, a
  second proof op on a different theory, a second proof op on the SAME theory, an
  upload (`POST /`), and a reload (incl. the same base index) — every one returned in
  0.02–0.52s, all `200`, all long before the long op finished. Nothing blocked.
* **Commit-time allocation ([R72]).** The long op STARTED first (t=0.014s) but
  COMMITTED last (t=30.57s) and got the HIGHEST index (10); the two fast ops that
  committed at t≈2.6s got the lower indices 8/9. Indices follow **completion** order.
* **Invisible until commit ([R73]).** Index-page polling across the whole 30s never
  showed the long op's index until after it completed — the to-be-allocated version is
  unresolvable during the computation.
* **Atomic under races ([R74]).** 12 truly-simultaneous fast proof ops allocated
  indices 12..23 — distinct, contiguous, zero collisions, zero skips.
* **Snapshot isolation ([R75]).** A long autoprove on idx3 completed fine (→ idx24)
  even though idx3 was reloaded in place mid-computation; the reload was itself served
  concurrently. Retention reconfirmed (a capped-out index still resolves `200`).
* Incidental ([R76]): autoprove can fail with `{"alert":"Sorry, but the autoprover ()
  failed!"}` — documented gap, not addressed this round (concurrency-scoped change).

### STEP 2 — the redesign

`Server::dispatch` now takes **`&self`** (one server shared across concurrent
requests). Each request runs **get-snapshot → compute → commit**: resolve + take a
cheap owned snapshot via `StateOps::snapshot` (lock released), run the `ProverOps`
call — including the slow ones — on the snapshot with **no state lock held**, then
commit atomically (`insert_new` allocates the fresh monotonic index **at commit**;
`replace` for in-place reload/edits). The only serialized sections are the
microsecond-scale atomic `StateOps` calls. Design note: I kept a single `dispatch`
entry point (not a static cheap/slow route split) because the snapshot→compute→commit
pipeline already isolates the lock-free compute from the two tiny atomic state calls,
and even a "read" route may invoke a non-trivial fragment producer — classifying
routes as cheap/slow would be a lossy approximation of "hold no lock during any
`ProverOps` call".

`StateOps` was reshaped to an interior-mutability, snapshot-handing trait (all `&self`;
`get`→`snapshot` returns an owned copy; `entries`→`indices`+`snapshot`). The reference
`InMemoryState<T: Clone>` is now a `Mutex<BTreeMap + counter>`; each method holds the
lock only for the map/counter op, never across compute. `Server`/`InMemoryState`
therefore compose to `Send + Sync`, so a consumer wraps the server in an `Arc` and
serves requests from any async task; a real backend implements the same `&self` façade
over its own async cache. The probed lifecycle (commit-time monotonic allocation,
atomicity, retention, in-place mutation, snapshot isolation) is the documented contract.

### Deliverables

- `src/dispatch.rs` — `dispatch(&self)`; `StateOps` reshaped (`snapshot`/`indices`,
  all `&self`); `InMemoryState` behind a `Mutex`; every handler restructured to take
  the request's single snapshot and commit via a separate atomic `StateOps` call.
- `tests/dispatch7.rs` (2 tests) — a `GatedProver` whose `autoprove` parks mid-compute
  on a controllable gate: `slow_op_in_flight_is_non_blocking_and_commits_last` asserts
  the full probed interleaving (non-blocking read while parked; a fast op commits first
  → index 2; the parked op's index invisible/unresolvable pre-commit; on release it
  commits last → index 3; final set contiguous), and `concurrent_allocations_never_
  collide_or_skip` races 16 threads and asserts a contiguous unique index block ([R74]).
- `tests/dispatch6.rs` — `WrapState` custom backend + `InMemoryState` contract tests
  updated to the `&self` snapshot API. All round-3/4/5/6 tests unchanged in intent
  (`let mut s` → `let s`; `dispatch5`'s `body` helper takes `&Server`).
- BEHAVIOR.md §17 (+ §16.2 superseding note); QUERIES.log [R70]–[R76].

`cargo test` → **104 passing** (26 unit + 15 dispatch + 17 dispatch4 + 14 dispatch5 +
7 dispatch6 + **2 dispatch7** + 23 parity); `cargo clippy --tests --examples` clean;
`dispatch7` verified non-flaky over 20 consecutive runs. All prior byte-parity tests
stay green (the state reshape is behind `Server::new`/`InMemoryState`, unchanged output).
