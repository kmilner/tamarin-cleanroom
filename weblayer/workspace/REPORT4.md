# REPORT4.md — full route surface (the dispatcher owns the whole request path)

Round 4 of the `weblayer` clean-room cluster (Unit A, web dispatch). Rounds 1-3
reproduced the response bodies and the interactive read/proof/edit state machine.
This round extends `route::Route`/`route::Toplevel` and `Server<P: ProverOps>` to
the REST of the observed surface so the dispatcher owns the consumer's entire
request path.

Black-box only: live probing of the sanctioned oracle ([R40]-[R4B], ports
3140-3143, Tutorial/NSLPK3/KCL diff theory) plus a corpus key census. No file
under /home/kamilner/tamarin-rs/ was read; the only tamarin-rs touch was executing
the sanctioned binary (`interactive`, `--diff`, `--port=`).

## Surface added

* Top level (`route::Toplevel`): `GET /` (index) + `POST /` (upload),
  `/robots.txt`, `/favicon.ico` (-> static icon), `/kill` (cancel a search),
  `/static/**` (filesystem assets), plus `/thy/...`.
* Theory scope: `reload`, `download/{file}`, `get_and_append/{file}`, and the diff
  (`equiv`) analogues `diffProof`/`diffMethod`/`diffrules`/`autoproveDiff`/
  `autoproveAll`.
* Error family: the 404/400/405 Yesod default-layout pages, generalised from the
  round-1 single 404 template.

## Behaviour pinned down (BEHAVIOR.md 14)

* One global version-index namespace. Proof ops AND uploads each allocate a fresh
  index off the same monotonic counter; structural edits and `reload` mutate in
  place (counter untouched, versions retained). Reload does not reset the counter
  or drop modified versions ([R43],[R45]).
* Envelopes. Third JSON shape `{"alert":...}` (get_and_append; and a FAILED
  method/diffMethod, which does NOT bump the version). `download` = the `source`
  bytes with content type application/octet-stream, no Content-Disposition.
  `reload` = a JSON {redirect} (not a 303).
* Redirect caching. All 303s (edit/add/delete success, delete-not-found, favicon)
  carry Cache-Control: no-cache, must-revalidate + a fixed past Expires
  (Response.no_cache).
* Static. Content type by extension without charset; a miss is plain-text
  `File not found`.
* Delete branch (refines 13.3): found -> 303 overview/help; not found -> 303
  overview/delete/{name}. delete_lemma now returns Option.
* Diff mode (`--diff`, kind `equiv`): overview shell = trace shell with a
  `DiffTheory:` title, /thy/equiv/ links, and no "Append modified lemmas" item;
  diffMethod redirects to overview/diffProof/...

## Two documented negative results

* del/path and verify return 404 (not 405) for every method/shape in both modes
  and appear in ZERO manifest keys -- neither in the corpus nor live-observable in
  1.13.0. Routed to Handler::Other (-> 404); no bytes fabricated. ([R47])
* autoproveDiff/autoproveAll block the prover on the probe theory (curl timeouts),
  so their exact redirect bytes are unobservable here. Dispatched as proof ops
  (new version) with the redirect modelled by analogy to diffMethod/autoprove.
  ([R49])

## Deliverables

* src/route.rs -- `Toplevel` entry point; `Handler` gains Reload/Download/
  GetAndAppend/Edit/AutoproveDiff/AutoproveAll; `Main` gains DiffProof/DiffMethod/
  DiffRules; AutoproveDiff/AutoproveAll parsers; OverviewView gains DiffProof/Delete.
* src/dispatch.rs -- Server::dispatch now takes the whole Toplevel surface;
  Response gains no_cache; Request gains query; ProverOps grown minimally (pure
  data producers + Option-returning apply_method/delete_lemma).
* src/page.rs + src/shell_template.rs -- shell parameterised over ShellKind
  (trace/equiv); index page (render_root/render_root_row) + root/simple-page templates.
* src/assets.rs (new) -- static content-type map + fixed text bodies.
* src/errors.rs -- 404/400/405 from a shared simple-page template
  (notfound_template.rs removed).
* src/envelope.rs -- the {alert} shape.
* tests/dispatch4.rs (17 tests) + tests/fixtures/r4_* -- capture-backed
  byte-parity/behaviour tests; BEHAVIOR.md 14, QUERIES.log [R40]-[R4B].

`cargo test` -> 75 passing (24 unit + 15 dispatch + 17 dispatch4 + 19 parity),
`cargo clippy --tests --examples` clean. Byte-parity vs live captures for
robots.txt, the Invalid-Arguments (400) and Method-Not-Supported (405) pages,
`Canceled request!`, download==source, reload redirect, the index page
(decomposition), and the equiv overview shell (decomposition); all re-verified
byte-identical on a fresh server instance ([R4B]). Round-1..3 tests unchanged in
intent (round-3 FakeProver updated for the Option-typed callbacks; assertions hold).
All servers stopped.

## Honesty / scope notes

* ProverOps returns opaque fragments; tests feed canned fragments so the asserted
  bytes are the WEB-LAYER assembly (envelope/status/Location/version/content-type/
  cache), never re-derived prover logic.
* Index-page rows (load time, origin) and upload/diff proof-state content are
  non-deterministic/prover fragments; the root page is byte-tested by decomposition
  with the exact captured row values fed as inputs.
* Static file BYTES are on-disk environment assets: the model reproduces the
  content-type + 404 decision and passes a UTF-8 asset through exactly; binary
  passthrough is an integration-adapter concern (documented in dispatch.rs).
