# REPORT3.md — handler semantics (the interactive-UI state machine)

Round 3 of the `weblayer` clean-room cluster. Prior rounds reproduced the
response **bodies** (JSON envelopes, page shells, proof-script pane, forms, DOT,
404) byte-for-byte. This round adds the **behaviour**: what each route *returns*
and how the theory-version state evolves — the UI state machine — parameterised
over a `ProverOps` callback trait the ported prover supplies at integration.

Everything below is derived from black-box observation only: the 81 crawl
manifests in `oracle/captured_responses/` (captured OUTPUT) and live probing of
the sanctioned oracle binary ([L7]-[L16], Tutorial.spthy, ports 3137-3141). No
file under `/home/kamilner/tamarin-rs/` was read; the only tamarin-rs touch was
executing the sanctioned binary (incl. `interactive --help` and
`--interface=*4`, extra CLI flags the protocol permits).

## Deliverables

* `src/dispatch.rs` (new) — the `Server<T: ProverOps>` state machine:
  route dispatch, theory-version management, response-envelope assembly.
* `src/route.rs` — grammar refined: `main/method/{lemma}/{n}[/path...]` now carries
  the proof path (the number precedes it); structured parsers `Autoprove`, `Nav`,
  `OverviewView`, `EditVerb` for the dispatcher.
* `src/forms.rs` — `edit_rows` implements the real textarea `rows=` formula
  (blocker b), replacing the round-1 hardcoded `rows="8"`.
* `tests/dispatch.rs` (15 tests) + `tests/fixtures/r3_*` — capture-backed
  behaviour/byte-parity tests.
* `BEHAVIOR.md` section 13, `QUERIES.log` [Q029]-[L16].

`cargo test` -> **55 passing** (21 unit + 15 dispatch + 19 parity), `cargo clippy
--tests --examples` clean.

## The state machine (`ProverOps` + `Server`)

The web layer owns exactly two pieces of state: a `BTreeMap<u64, Theory>` of
theory versions and a monotonic `next_index`. `ProverOps` is the prover boundary
- parse/edit/add/delete a lemma, apply a proof method at a path, autoprove, the
next/prev traversal target, and the opaque pretty-printed fragments (west pane,
center content, lemma source, theory source, DOT). `Server::dispatch(req)` parses
the route, resolves the numeric index, and makes every web-layer decision:

### 1. Version increments (spec item 1)
* **Proof ops** (`main/method`, `autoprove`) allocate a **fresh** index
  `= (max ever allocated)+1` and leave the base version intact; every version
  stays resolvable ([L8],[L9],[L11]).
* **Structural edits** (POST `edit/{edit,add,delete}`) mutate the theory **in
  place** at the requested index - no new version ([L12]).
* Navigation/views never change the version set ([L10]).

Byte evidence: `GET main/method/Client_session_key_secrecy/1` on v1 ->
`{"redirect":"/thy/trace/2/overview/proof/Client_session_key_secrecy/_"}`
(identical to capture `r3_method_redirect.json`); a second identical request ->
`/thy/trace/3/...`; `POST edit/delete/Client_auth` -> `303`, no new index.

### 2. autoprove variants (spec item 2)
Route `autoprove/{strategy}/{bound}/{allSol}/proof/{lemma}[/path...]`. The
keyboard-help "a/b/all/characterization" matrix ([Q031]) decodes as: `strategy` in
{`idfs` prove, `characterize`}; `bound` `0`=unbounded(`a`/`A`), `5`=bounded
(`b`/`B`); `allSol` `False`=stop-at-first(lowercase), `True`=all-solutions
("all", uppercase). Every variant answers `200` + JSON
`{"redirect":"/thy/trace/{new}/overview/proof/{lemma}/{focus}"}` at a new version
([L9]). `Autoprove::parse` recovers the full matrix from the URL (tested).

### 3. proof-step / del / add (spec item 3)
* `main/method/{lemma}/{n}[/path...]` - number **before** the case-name path
  ([Q032]); `200` JSON redirect, new version.
* `POST edit/delete/{name}` -> `303` `Location: .../overview/help`, empty body, in
  place.
* `POST edit/edit/{name}` (`lemma-text`) -> success `303` `.../overview/edit/{name}`;
  **failure** `200` re-rendering the full edit-form page, theory unchanged
  ([L13]).
* `POST edit/add/{pos}` -> success `303` `.../overview/add/{pos}`; failure `200`
  add-form page. `overview/edit|add/...` are full pages that appear only as these
  POST-redirect targets.

### 4. Traversal & path encoding (spec item 1)
`next|prev/{mode}/proof/{lemma}` -> `200` `text/plain` bare URL at the same
version (`nav_target` is a prover computation; `mode` opaque). Proof paths are the
raw `/`-join of case-name segments (root `_`), segments `[A-Za-z0-9_]`, **no
percent-encoding** ([Q034]).

## Integration blockers

### (b) edit-form textarea `rows=` - SOLVED
`rows = (#newlines in the raw lemma source) + 2`, verified over 4 lemmas
(9/11/7/10 -> 11/13/9/12, [L14]). Implemented as `forms::edit_rows`;
`edit_form_envelope_byte_parity_with_rows_formula` reproduces the whole captured
edit-form JSON envelope for two lemmas (rows 11 and 9) byte-for-byte.

### (a) non-local page shell - NEGATIVE RESULT (environment-limited)
The spec expected the served shell to differ for a non-`127.0.0.1` client. I
bound the oracle to all IPv4 interfaces (`--interface=*4`) and fetched every route
BOTH via `127.0.0.1` and via the machine's non-loopback `212.100.173.110` - the
server access-log **confirms** the peer is `212.100.173.110`. Result: **every
route is byte-identical** (read views, forms, index, 404, and mutating GET/POST -
method/edit/delete/reload all succeed and match). Host and
`X-Forwarded-For`/`X-Real-IP` headers have no effect ([L15]).

So in oracle **1.13.0** the shell is **origin-independent**. The locality
predicate evidently classifies all of the machine's own interface addresses
(loopback + `212.x`) as local, so a genuinely foreign peer cannot be synthesised
on a single host - the non-local shell's bytes are **unobservable in this
environment**. I therefore did *not* fabricate a variant from memory. The
architecture leaves a clean hook: the shell renderer can be parameterised over an
`origin_local` flag the ported prover drives; the exact non-local bytes remain a
documented gap pending a multi-host capture (or a version where the difference is
observable at the HTTP boundary).

## Honesty / scope notes

* `ProverOps` returns opaque fragments; the tests feed **canned** fragments taken
  from live captures (focus paths `_`, next/prev targets, lemma sources) so the
  asserted bytes are the **web-layer** assembly (envelope, status, Location,
  version index, URL template), never re-derived prover logic.
* Center pane of overview pages = the sibling `main/*` html + one trailing space
  (round-2 finding), wired through `center_for`.
* The `#`/current index does not resolve (a literal `#` is a URL fragment; `%23`
  404s), so `dispatch` treats a non-numeric/absent index as `404` ([L2]).
* Status-0 corpus autoprove entries are the crawler's `TimeoutError` string, not a
  server response family - corrected in BEHAVIOR section 1/13.2 ([Q035]).
* Not modelled (side-effecting file ops, seen only as link targets):
  `reload`, `download/{file}`, `get_and_append/{file}`, `POST /` upload. The
  `diff`/`equiv` theory-kind never appears.
