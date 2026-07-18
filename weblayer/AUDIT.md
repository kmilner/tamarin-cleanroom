# Similarity audit — web layer clean-room crate

**Auditor role:** similarity auditor (may read both sides).
**Clean side:** `/home/kamilner/tamarin-cleanroom/weblayer/workspace/web-clean/src/` (12 modules).
**Haskell originals:** `tamarin-prover/src/Web/{Theory,Handler,Types,Hamlet,Dispatch}.hs`.
**Method:** abstraction–filtration–comparison. I filtered out (a) identical output strings
taken from observed responses (warning texts, HTML/DOT/JSON syntax, templates), (b) idea-level
algorithm/behaviour equivalence, (c) API/type names given in interface docs, and (d) structure
forced by the data model or the wire format. I looked for surviving residue: mirrored internal
decomposition, matching internal (non-API, non-output) names, echoed comments, algorithmic
*expression* matches where the observed behaviour admits materially different code, and any
content present in the Haskell source but absent from every observable output.

## Verdict

**PASS — no similarity violations survive filtration.**

Every point of resemblance between the clean crate and the Haskell originals reduces to observable
served output, idea-level behaviour, wire-format structure, or route-grammar tokens that appear in
served URLs. In addition the clean crate carries several strong *positive* signals of independent,
observation-only derivation (enumerated at the end). Details per module follow.

---

## envelope.rs vs Handler.hs `responseToJson` (617–631)

- Haskell builds `JsonHtml` as `object ["html" .= …, "title" .= …]` and `JsonRedirect` as
  `object ["redirect" .= …]`. Clean side: `Content{html,title}` → `{"html":…,"title":…}`,
  `Redirect{redirect}` → `{"redirect":…}` via `serde_json`.
- FILTERED: the two object shapes, their key names, and the `html`-before-`title` order are all
  literally present in the captured JSON response bodies. Compatibility content taken from output.
  The clean side uses a stock serializer (`serde_json`) over declaration-ordered struct fields —
  a materially different expression from Aeson's `object [...]` list.
- POSITIVE SIGNAL: Haskell has a third shape, `JsonAlert msg` → `{"alert":…}` (lines 625, 387–388,
  441, 723, …). The clean crate does **not** implement it and documents "one of exactly two"
  shapes — i.e. it reproduced only what the captures exercised, not what the source contains.

## escape.rs vs Blaze/HtmlDoc entity escaping

- Clean side maps `& < > " '` → `&amp; &lt; &gt; &quot; &#39;`, single left-to-right char pass.
- FILTERED: this five-entity set (with `'`→`&#39;`) is the standard HTML escape and every member
  is witnessed in captured output (the module doc cites the concrete witnesses: `&#39;` in the
  "'smart'" comment, `&lt;/&gt;` in pair terms, `&quot;` in the loaded-from path, `&amp;` in a
  lemma formula). No corresponding hand-written escaper exists in the Web/*.hs sources (escaping
  comes from Blaze `toMarkup` / the HtmlDoc renderer); the char-match loop is the universal
  idiom, forced by the observed mapping. Idea-level + output-derived.

## route.rs vs Types.hs `parseTheoryPath` / `renderTheoryPath` / route table (364–657)

- Both recognise the same `main/*` tail tokens: help, message, rules, tactic, cases, lemma, add,
  edit, delete, method, proof.
- FILTERED: every one of those tokens appears in served URLs / hrefs in the captured pages, and in
  the request paths under probe — they are the wire grammar, not internal expression. The
  `cases/{kind}/{i}/{j}` and `proof/{lemma}/{path…}` shapes are dictated by the observed URLs.
- NOT MIRRORED (examined explicitly):
  - Match/dispatch **order** differs (HS: help, rules, message, tactic, lemma, cases, proof,
    method, edit, add, delete; clean: help, message, rules, tactic, cases, lemma, add, edit,
    delete, method, proof) and the clean side groups arms differently — not a copy of the HS
    case decomposition.
  - The clean side has **no** `unprefixUnderscore`/`prefixWithUnderscore` underscore-encoding
    (Types.hs 398–414); it treats `_` as a literal proof-path segment, a materially different
    parsing model derived from the raw observed URLs.
  - `Handler::Other`/`Main::Other` structured fall-throughs are the clean side's own design;
    Haskell returns `Maybe`/`Nothing`.
- POSITIVE SIGNAL: the clean side models the diff analysis kind as the guessed string `"diff"`
  and marks it "plausible but unobserved". The real Haskell route prefix is `equiv`
  (Types.hs 595–617). Guessing wrong here is direct evidence the router grammar was not read
  from source.

## page.rs + shell_template.rs vs Dispatch.hs `defaultLayout'` (686–723) + Hamlet.hs `overviewTpl`/`headerTpl`

- The three constants (`PAGE_PREFIX`/`PAGE_MID`/`PAGE_TAIL`) are the assembled page shell split at
  the two dynamic insertion points (west proof pane, center main pane).
- FILTERED: byte-for-byte page HTML is served output (the note in the task confirms shell_template
  is an observed-output byte copy). The four `§`-slots (NAME/IDX/VERSION/FILENAME) correspond to
  the only values that vary across captured pages.
- SOURCE-ONLY CHECK (per task instruction): I checked for Hamlet fragments that never reach output
  and found none leaked into the clean shell. Conversely the clean shell **omits** the source-only
  `$maybe msg <- message / <p.message>` flash-message branch (Dispatch.hs 712–713), which only
  renders when a flash message is set — confirming derivation from a concrete rendered page rather
  than from the template. The `<script>…</script></script>` double-close and attribute orderings
  in the constants are rendering artifacts of the served bytes, not of the Hamlet source text.

## proofscript.rs vs Theory.hs `theoryIndex` / `lemmaIndex` / `proofIndex` / `linkToPath` (204–416, markStatus 2150)

This is the most algorithmic clean module, so I scrutinised it hardest.

- Element sequence (header, blank, Message, blank, Rules, blank, Tactic, blank, Raw, blank,
  Refined, blank, add-`<first>`, blank, lemmas…, blank, end) matches `theoryIndex`'s list.
  FILTERED: this sequence, the lone-`<br/>\n` blank lines between items, the `hl_*` status
  classes, the `internal-link proof-step` / `remove-step` anchor pairs, the `hl_keyword` spans
  for by/case/next/qed, the two-blank-lines-when-zero-lemmas spacing, and the status-`<span>`
  wrapping the lemma header (`markStatus`) are **all** directly visible in the captured proof-pane
  HTML. The clean side documents each as "observed" and even explains the zero-lemma spacing as
  "matching the capture". Output-dictated; a faithful reproduction has no freedom here.
- DECOMPOSITION NOT MIRRORED: Haskell factors everything through one `linkToPath` combinator plus
  document operators `$-$`/`vcat`/`intersperse`/`nest` and the core `prettyProofWith` traversal.
  The clean side uses a flat `Vec<String>` element list with per-role helpers (`header_line`,
  `item_line`, `add_link`, `edit_delete_line`, `sorry_line`, `render_proof_line`). Different
  factoring; none of the clean helper names match HS internal names (`linkToPath`, `proofIndex`,
  `lemmaIndex`, `markStatus`, `stepLink`, `removeStep`, `overview`).
- The rules label, sources descriptions, and lemma declaration HTML are taken as **opaque
  prover-produced input** on the clean side, whereas Haskell computes them (`ruleLinkMsg`,
  `casesInfo`, `prettyLNFormula`). The clean side deliberately does not reimplement that logic.
- `indent depth = "&nbsp;&nbsp;".repeat(depth)` is output-derived (tests pin depth1→2 nbsp,
  depth4→8 nbsp). Idea-level.

## forms.rs vs Theory.hs `TheoryEdit` / `TheoryDelete` / `TheoryAdd` Hamlet blocks (1025–1133)

- The edit/delete/add form bodies match the Hamlet templates' visible content (labels, textarea,
  submit button, `<h3>` intros, `<noscript>` warning, the `<li>` bullet texts, the `.wrap-text`
  `<style>` block, `&zwnj;` breaks, and the `../../edit/{verb}/{name}` actions).
- FILTERED: each form is served as the `html` field of the `main/{edit,delete,add}` JSON
  envelope, so all of this is observed output — exactly the compatibility content the task flags
  as non-violating. SOURCE-ONLY CHECK: no Hamlet-only fragment (comment, unrendered conditional)
  appears in the clean forms.
- POSITIVE SIGNAL: Haskell computes `rows=#{textHeight}` with `textHeight = 2 + (# newlines)`
  (1066). The clean side hardcodes `rows="8"` from a single capture — it reproduced an observed
  constant, not the source formula (and will diverge for other lemma sizes). Also the reproduced
  `<span class="tamarin">Tamarin</span></span>` carries a spurious second `</span>` that exists in
  the rendered bytes but not in the one-`<span>` Hamlet source — proof of output capture.

## intdot.rs vs Dispatch.hs `intdotLayout` (727–744) + Handler.hs `getInteractiveDotGraphR` (897–906)

- `INTDOT_TEMPLATE` matches the served intdot mini-page (meta/title/inline-style/one stylesheet/
  one module script, then `<dot-graph-viz dotsrc=…>`); `EMPTY_GRAPH_DOT` is the empty-graph DOT
  skeleton served by `interactive-graph-def`.
- FILTERED: both are served-output byte copies (the DOT skeleton originates in the graph module,
  not Web/*.hs, and reaches the clean side only through the endpoint's output). The
  `</script></script>` double-close again evidences capture, not source reading. `dotsrc_path`
  just assembles the observed URL. Compatibility + output-derived.

## text.rs vs Handler.hs next/prev + source/message handlers (949+, 1442–1454)

- `source_body`/`nav_target` are identities; `main_proof_path` assembles the proof URL.
- FILTERED: Haskell computes navigation targets with `nextThyPath`/`nextSmartThyPath` (prover
  logic) and the source body with the theory pretty-printer, then the web layer passes the result
  through. The clean side correctly models these as pass-throughs of opaque prover output and does
  **not** reimplement `nextThyPath`. The URL builder is forced by the route grammar. No expression
  overlap.

## errors.rs + notfound_template.rs vs Yesod default error layout

- 404 page reuses the head + loading bar + `<h1>Not Found</h1>` + echoed HTML-escaped path + the
  dialog/contextMenu tail.
- FILTERED: this is the rendered output of Yesod's default `NotFound` handler through
  `defaultLayout'` (probed live per the module doc); every byte is observed. Path echoing +
  HTML-escaping is observed behaviour. No Web/*.hs expression is involved.

## lib.rs

- Module map / doc comments only; describes observations. No logic, no echoed Haskell comments.

---

## Positive independence signals (corroborating non-access)

1. Diff kind guessed as `"diff"`; the real route prefix is `equiv` (Types.hs) — wrong guess ⇒ not
   read from source.
2. `JsonAlert`/`{"alert":…}` shape absent — only the two exercised envelopes reproduced.
3. No underscore path-encoding (`prefixWithUnderscore`/`unprefixUnderscore`); `_` handled as a
   literal segment.
4. `rows="8"` hardcoded instead of the source's `2 + newline-count` formula.
5. Output-capture artifacts faithfully preserved (`</span></span>`, `</script></script>`) that do
   not exist in the Hamlet/Handler source text.
6. Different code architecture throughout (flat `Vec<String>` + serde structs vs Aeson `object`,
   Blaze/Hamlet widgets, and HtmlDoc document combinators); no internal HS names reused.
7. Source-only template branches (flash `<p.message>`) omitted from the shell.

## Modules reviewed

envelope.rs, errors.rs, escape.rs, forms.rs, intdot.rs, lib.rs, notfound_template.rs, page.rs,
proofscript.rs, route.rs, shell_template.rs, text.rs — all 12.

**Findings that survive filtration: 0. No redo instructions issued.**

---

## Round 2 incremental audit

Scope (round-2 additions identified from `workspace/REPORT2.md`, timestamps, and
the round-1 `REPORT.md`; git unavailable in the workspace):
- `examples/corpus_html.rs` — corpus-wide byte-parity harness (new).
- `tests/fixtures/html_sample.ndjson` — 20 captured `kind=="html"` bodies (new
  observation data).
- `tests/parity.rs::html_page_generality_sample_byte_identical` (373-413) — the
  one new test.

No template or model source was edited this round. REPORT2 states page parity was
obtained from the *existing* `render_intdot`/`render_page`; the solved-proof-tree
work in `proofscript.rs`/`envelope.rs`/`lib.rs` belongs to the round-1 continuation
session (documented in `REPORT.md` §"Work done this session" item 3) and was already
cleared in the round-1 audit above (proof-line `by`/`case`/`next`/`qed` grammar,
`markStatus` header wrapper). Confirmed by inspection that the round-2 files add no
rendering logic — the harness and the test both delegate to the round-1 modules.

Haskell originals consulted for this round: `src/Web/{Theory,Hamlet,Types,Handler}.hs`.

### corpus_html.rs (harness) + the generality test

- The harness has NO Haskell counterpart: `Web/*.hs` contains route handlers and
  Hamlet widgets, not a byte-parity test tool. `corpus_html.rs` reads captured
  output (NDJSON), classifies each record by its observed URL key, recovers the one
  capture-erased scalar (the request index, read back from an emitted link via
  `index_after`), renders through the round-1 `render_intdot`/`render_page`, and
  byte-compares. It reimplements none of the Haskell.
- `handler_and_tail` / `overview_subfamily` split `/thy/trace/#/<handler>/<tail>`
  and bucket `help`/`proof`/`intdot`. These are the wire URL grammar already
  filtered in the round-1 route.rs analysis (tokens present in served URLs), not
  internal HS expression. The `intdot`→`interactive-graph-def` swap is NOT in the
  harness — it stays inside the round-1 `dotsrc_path`, with `tail` passed through.
- Pane delimiters `WEST_OPEN` `<div class="monospace" id="proof">`, `WEST_CLOSE`
  `</div></div></div><div class="ui-layout-east">`, `CENTER_OPEN`
  `<div id="ui-main-display">`, `CENTER_CLOSE` `</div></div></div><div id="dialog">`
  are verified to be exactly the trailing/leading bytes of the round-1-audited shell
  constants `PAGE_PREFIX`/`PAGE_MID`/`PAGE_TAIL` (`shell_template.rs`). Using them to
  slice an observed body introduces no new expression; they are served output.
- `html_sample.ndjson` `b`-fields are captured response bodies (program OUTPUT used
  as observation input — expressly sanctioned by the protocol). URLs span `intdot/*`
  and `overview/{help,proof}` only.
- The test's `name`-from-sibling and `tail`-from-URL checks are non-circularity
  guards, not ported logic. No echoed comments, no HS internal names.

### Positive independence signals (this round)
- Harness carries an `overview/other` bucket and an "unexpected handlers" counter
  that both remain empty — it reproduces only the two families the captures exercise
  and presumes no source-side handler taxonomy.
- The one scalar taken from each target body is the request index the capture tool
  erased to `#`; name, version, filename and path tail all come from sibling
  artifacts or the URL key, so a byte match is an independent property, not a
  read-back.

### VIOLATIONS (Round 2)
None. Every round-2 resemblance reduces to served output, wire-format URL tokens,
or reuse of already-cleared round-1 shell/render functions. Findings that survive
filtration: 0. No redo instructions issued. VERDICT: PASS.

---

## Round 3 incremental audit

Scope (the round-3 delta per SPEC_ROUND3 / REPORT3): the UI **state machine** in
`src/dispatch.rs` (the `Server<ProverOps>` version-map + route dispatch), the
`src/route.rs` grammar extensions (`Autoprove`/`Nav`/`OverviewView`/`EditVerb`
parsers), and the `src/forms.rs` textarea-`rows` rule that replaced the round-1
hardcoded `rows="8"`. Haskell originals consulted (both sides — auditor privilege):
`src/Web/{Handler,Theory,Dispatch,Settings,Types}.hs`. Method: abstraction–
filtration–comparison, with the version-map / envelope / route semantics treated as
BEHAVIOR (live-probed, QUERIES.log [L7]–[L16]) — I flagged only expression-level
mirroring of the Haskell handler internals.

### dispatch.rs (`Server<ProverOps>`) vs Handler.hs `putTheory`/`modifyTheory`/`postTheoryEditR`/`getTheoryPathMR`

- **Version allocation.** Clean `commit_new_version` (dispatch.rs:330–335) hands out
  a stored monotonic counter (`next_index`, seeded to 2 with base at 1 in
  `Server::new` dispatch.rs:179–183). Haskell `putTheory` (Handler.hs:351–352)
  recomputes `idx = if M.null then 1 else fst (M.findMax theories) + 1` on every
  insert. Different EXPRESSION (a retained counter vs a re-derived `findMax+1`); they
  coincide only because nothing is ever removed from the version map, which is the
  observed invariant "new index = (global max)+1, all remain resolvable" ([L9],[L11]).
  Behavior-derived, not mirrored — and the divergent expression is mild independence
  evidence.
- **Proof op → `{redirect}` + new version.** Clean `get_method`/`proof_op_autoprove`
  (dispatch.rs:318–328) allocate a version then answer `render_redirect` to
  `/thy/trace/{i}/overview/proof/{lemma}[/focus…]`. Haskell `modifyTheory`
  (Handler.hs:736–744) does `putTheory` then `JsonRedirect (InteractiveOverviewR
  newThyIdx (fpath thy))`; `getTheoryPathMR`'s `TheoryMethod` arm (Handler.hs:1013–
  1016) and the autoprover `getProverR` (1065–1068) route through it. The `{redirect}`
  envelope, the `overview/proof/{lemma}/{focus}` URL, and the version bump are all
  observed output ([L8],[L9], envelope census [010]). Compatibility content.
- **`main/method` interception.** Clean `dispatch` (dispatch.rs:213–216) branches
  `main/method` to a proof op before the read-only `main/*` views; Haskell
  `getTheoryPathMR.go` (1013–1022) likewise special-cases `TheoryMethod` ahead of the
  generic render arm. This split is FORCED by observed behavior — `main/method`
  returns `{redirect}`+version-bump ([L8]) while every other `main/*` returns
  `{html,title}` ([002],[010]); any implementation must branch on it. Behavioral, not
  an expression mirror.
- **Structural edits mutate in place.** Clean `post_edit` (dispatch.rs:338–374)
  re-inserts at the SAME `index` and answers `303 See Other` with a `Location`
  (`overview/help` for delete, `overview/edit|add/{name}` for edit/add), re-rendering
  the full-page form with `200` on failure. Haskell `postTheoryEditR`
  (Handler.hs:845–880) reaches `replaceTheory … idx` (307–318, `M.insert idx`) — same
  index — and `redirect (InteractiveOverviewR i …)`; failure falls through to
  `overviewTpl` (200). Every one of these is a live-probed behavior
  ([L12] in-place + 303 Location targets; [L13] failure=200 form). Notably the clean
  `ProverOps` exposes SEPARATE `edit_lemma`/`add_lemma`/`delete_lemma` callbacks,
  whereas Haskell `editLemma` for `TheoryEdit` is internally a delete-then-add
  (Handler.hs:273–282). The clean side did NOT mirror that internal decomposition —
  independence evidence.
- **Architecture.** The whole `ProverOps` trait seam (dispatch.rs:131–165) is the
  clean side's own integration boundary (SPEC_ROUND3 asked for it); Haskell has no
  such callback abstraction and calls the prover inline (`applyMethodAtPath`,
  `modifyLemma`, `removeLemma`, `runProver`). Route dispatch is a hand-written
  `match (method, Route)` over the clean side's own `Route` enum, versus Yesod's
  TH-generated `mkYesodDispatch` typed-route table (Dispatch.hs:52, Types.hs:583–610).
  No shared internal names, no mirrored control flow.

### route.rs extensions vs Types.hs route table / `parseTheoryPath`

- `Autoprove::parse` (route.rs:186–199) reads `{strategy}/{bound}/{flag}/proof/{lemma}
  [/path…]`; `Nav::parse` (219–229) reads `{mode}/proof/{lemma}`; `EditVerb`/
  `OverviewView` mirror the observed `edit/{verb}/{name}` and `overview/{help|proof|
  edit|add}` tails. Every token is the wire URL grammar present in served hrefs / probe
  paths: the autoprove shape and its `idfs`/`characterize` × `{0,5}` × `True/False`
  matrix come from the corpus census [Q030]/[Q031]; `next|prev/normal/proof/L` and the
  `overview/*` tails from [Q021]/[L10]/[L12]. This matches the Yesod resource lines
  (Types.hs:583–610) and the `SolutionExtractor` tokens (Types.hs:627–634) only at the
  URL surface — compatibility content, already the disposition of the round-1 route.rs
  analysis.
- POSITIVE SIGNAL (continues from round 1): the clean parsers treat `_` as a LITERAL
  path segment (route.rs test `path: vec!["_","B_2"]`), with no analog of Haskell's
  `prefixWithUnderscore`/`unprefixUnderscore` empty/`_`-segment encoding
  (Types.hs:400–414). A materially different parsing model derived from the raw
  observed URLs.

### forms.rs `edit_rows` — CLOSE CALL (considered, CLEARED)

- Clean `edit_rows` (forms.rs:20–22): `text.matches('\n').count() + 2`, used at
  forms.rs:29.
- Haskell `textHeight` (Theory.hs:1066): `2 + length (filter (=='\n') lPlaintext)`.
- These are expression-identical ("2 + count of `'\n'` characters"). I examined this
  hard because the two are textually the same algorithm.
- FILTRATION — output-forced behavior: the formula was DERIVED from a live probe
  ([L14]) that measured 4 lemmas — newline counts 9/11/7/10 → rows 11/13/9/12. Those
  points pin `rows = newlines + 2` uniquely as a function of newline count (slope
  exactly 1 row per newline from the successive differences, intercept exactly 2);
  `count('\n') + 2` is the minimal arithmetic transcription of that observed relation,
  and there is no materially different implementation of *that function of the newline
  count*. This is the same disposition as the graphdot prefix-derivation close call:
  "identical behavior at the idea level" / "structure forced by the observed output,"
  not protectable expression.
- DECISIVE independence context: in round 1 the clean side HARDCODED `rows="8"` (a
  wrong constant that diverges for every other lemma size) — the round-1 audit logged
  that as a positive non-access signal. The correct `newlines+2` rule appears ONLY in
  round 3, obtained by documented live probing with its four data points in QUERIES.log.
  Had Theory.hs:1066 been read, the formula would have been present in round 1 rather
  than a placeholder constant. The provenance therefore affirmatively shows derivation
  from observation, not from source. CLEARED — no redo instruction issued.
- (Everything else in forms.rs — the edit/delete/add form bodies — is unchanged served
  output already cleared in the round-1 forms.rs analysis.)

### VIOLATIONS (Round 3)
None. The version-map/state-machine semantics reduce to live-probed behavior
([L7]–[L16]) reproduced through an independent architecture (a `ProverOps` trait, a
hand-rolled `Route` match, and a stored monotonic counter — none matching the Yesod
handler internals), the route.rs extensions are observed wire-grammar tokens, and the
one expression-identical point (the `edit_rows` newline formula) is an output-forced
arithmetic fit whose round-1→round-3 provenance proves observation-derivation. One
close call considered and cleared; findings that survive filtration: 0. No redo
instructions issued. VERDICT: PASS.

---

## Round 4 — both-sides similarity audit (weblayer delta)

Auditor scope: this round's uncommitted delta only (clean-room HEAD 63ed8a9 predates
it). Delta = `git status/diff` restricted to `weblayer/`: modified `src/dispatch.rs`,
`src/route.rs`, `src/page.rs`, `src/shell_template.rs`, `src/envelope.rs`,
`src/errors.rs`, `src/lib.rs`, `tests/dispatch.rs`; deleted `src/notfound_template.rs`;
new `src/assets.rs`, `tests/dispatch4.rs`, `tests/fixtures/r4_*`, `REPORT4.md`; plus
`BEHAVIOR.md` §14 and `QUERIES.log` [R40]–[R4B]. Compared against upstream
`src/Web/{Handler.hs,Dispatch.hs,Types.hs,Settings.hs,Hamlet.hs}` following the code.
Method: abstraction–filtration–comparison; every behavioral claim cross-checked to a
logged probe/fixture.

### What the round adds (abstraction)
The dispatcher grows from the interactive read/proof handlers (§13) to the whole
request path: top level (`/`, `/robots.txt`, `/favicon.ico`, `/kill`, `/static/**`),
theory-scoped `reload`/`download`/`get_and_append`, the diff (`equiv`) analogues
(`diffProof`/`diffMethod`/`diffrules`/`autoproveDiff`/`autoproveAll`), one global
version-index namespace (upload allocates like a proof op), the third JSON envelope
`{alert}`, redirect cache headers, and the 400/405 status pages generalised from the
round-1 404. All served through the same independent `ProverOps` callback boundary +
`Server` state machine — architecturally unrelated to Yesod's handler-per-route layout.

### Filtration — every wire constant traces to a logged probe, and matches upstream
*because it is the wire output*, which is exactly what compatibility content is:
- `METHOD_FAILED_ALERT = "Sorry, but the prover failed on the selected method!"` —
  upstream Handler.hs:1016 emits this identical string as the `JsonAlert` body; it is
  the observable envelope, logged [R49]. Boundary output, not copied expression.
- `ROBOTS_TXT "User-agent: *"` (Dispatch.hs:64, [R40]), `KILL_CANCELED "Canceled
  request!"` (Handler.hs:1430, [R46]), `KILL_NO_PATH_MSG "No path to kill specified!"`
  (Handler.hs:1431 `invalidArgs`, [R46]), flash `"Loaded new theory!"` /
  `"Post request failed."` (Handler.hs:807/790, index-page, [R45]), `STATIC_NOT_FOUND
  "File not found"` ([R41]), `FAVICON_TARGET "/static/img/favicon.ico"` ([R40]),
  `EXPIRES_PAST "Thu, 01 Jan 1970 05:05:05 GMT"` + `CACHE_CONTROL_NOCACHE` ([R40],[R4A]).
  All are HTTP-boundary strings; the Expires/Cache-Control pair is a Yesod-framework
  artifact observed on the wire, not tamarin-specific. Each is logged.
- Byte-exact templates (`PAGE_PREFIX`/`APPEND_ITEM`/`ROOT_TEMPLATE`/`SIMPLE_HEAD_A/B`/
  `SIMPLE_TAIL`) reproduce **rendered** `defaultLayout'`/`rootTpl` output (Types.hs
  686–723, Hamlet.hs 52–134), not Hamlet source structure. The `<head>` link set,
  loading bar, dialog/contextMenu tail, `<p class="message">` flash placement, and the
  index `Original`/`<em>Modified` cells are all in the composed rendered bytes; the
  round-4 fixtures (`r4_root_single.html`, `r4_equiv_overview_kcl.html`, the 400/405
  pages) are genuine captures (real temp path `/tmp/tmp.RTTW9GQKpy/…`), and the parity
  tests decompose those captures. Sanctioned compatibility content — cleared.

### Non-copying signals (comparison against Types.hs route table + Handler.hs)
Strong affirmative evidence the surface was derived from observed URLs, not from
`Types.hs` `parseRoutes`:
- Enum/handler names are URL-segment transliterations that DIVERGE from upstream
  constructor names: clean `GetAndAppend` vs upstream `AppendNewLemmasR`;
  `AutoproveDiff` vs `AutoDiffProverR`; `AutoproveAll` vs `AutoProverAllR`/
  `AutoProverAllDiffR`; `Reload`/`Download` vs `ReloadTheoryR`/`DownloadTheoryR`;
  `ShellKind::{Trace,Equiv}` (URL segments) vs upstream `TheoryInfo`/`DiffTheoryInfo`.
  A transcription of the route table would carry the `…R` constructor names; it does not.
- The clean side reproduces ONLY observable routes and OMITS the upstream routes it
  could not observe — `unload` (UnloadTheoryR), `mirror`/`interactive-mirror-def`
  (TheoryMirrorDiffR…), `del/path` (DeleteStepR), `verify` (TheoryVerifyR) are all in
  Types.hs but absent here, routed to `Handler::Other`→404, with [R47] documenting the
  observed 404. A copy of the route table would include them.
- `AutoproveAll::parse` requires exactly `{strategy}/{bound}` (the equiv shape,
  Types.hs:607, no path) rather than the trace shape's trailing `*TheoryPath`
  (Types.hs:584) — i.e. it follows the observed `diffProof` body links ([R49]), not the
  source. `autoproveDiff` correctly carries NO all-solutions flag (observed), unlike
  trace `autoprove`.
- `ProverOps` is a callback boundary grouped by the web layer's needs (fragment
  producers / mutations / diff ops); it does not mirror Handler.hs's HTTP-handler-per-
  route decomposition. No comment lineage: none of upstream's Hamlet/handler comments
  ("Robots file handler", "Template for root/welcome page", …) appear in the delta.
- `Rule_Destrd_0_fst` (dispatch4.rs canned diff focus) initially looked like a
  source-only internal name, but it appears VERBATIM in the captured equiv fixture body
  links (`…/diffProof/Observational_equivalence/Rule_Destrd_0_fst`) — a boundary-
  observable proof-state case token, not source-derived. Cleared.

### Minor notes (documentation completeness; NOT protectable-expression findings)
1. `assets::static_content_type` maps extensions beyond the probed set — [R41] observed
   only `.css/.js/.png/.ico`/no-ext, but the fn also maps `gif/jpg/jpeg/svg/html/htm/
   txt/json`. These are generic IANA/web MIME types (scenes-à-faire), NOT copied from
   tamarin (upstream delegates to wai-app-static's `defaultMimeMap`, not its own code),
   so no expression is taken; they merely exceed the logged evidence. Non-blocking.
2. `page::render_root_row` `Modified` branch (`<em>Modified`, unclosed) is not pinned by
   a committed round-4 fixture (only the `Original` case is in `r4_root_single.html`),
   but the Original/Modified index-cell behavior is traced to prior-round probes
   (QUERIES [172],[188]–[189]); a generic Hamlet table-cell artifact, not tamarin
   expression. Non-blocking.

### VIOLATIONS (Round 4)
None. Every wire string/template in the delta is boundary output backed by a logged
probe or a committed capture; the added route grammar is observed URL tokens whose
naming and omissions affirmatively show URL-derivation rather than transcription of the
Yesod route table; the architecture (Toplevel/Server/ProverOps) does not mirror
Handler.hs internal structure; no comment lineage. Two documentation-completeness notes
recorded (speculative extra MIME mappings; unfixtured Modified row) — neither is copied
protectable expression and neither requires redo. Findings that survive filtration: 0.
No redo instructions issued. VERDICT: PASS.

---

## Round 5 — both-sides similarity audit (weblayer delta: del/path + verify)

Auditor scope: this round's uncommitted delta only (clean-room HEAD 8901219 predates
it). Delta from `git status/diff` restricted to `weblayer/`: modified
`src/dispatch.rs`, `src/route.rs`, `src/envelope.rs`, `tests/dispatch.rs`,
`tests/dispatch4.rs`; new `tests/dispatch5.rs`, `round5/` (four staged `*.json`
captures + `ORACLE_NOTES.md`); plus `BEHAVIOR.md` §14.8-rewrite + §15,
`QUERIES.log` [R50]–[R57], `REPORT.md`. Compared against upstream
`src/Web/{Handler.hs,Types.hs,Dispatch.hs}` following the code from
`getDeleteStepR`/`getTheoryVerifyR`. Method: abstraction–filtration–comparison; every
behavioral claim cross-checked to a logged probe ([R50]–[R57]) or a committed capture
(`round5/*.json`).

This round REVERSES the round-4 "del/path & verify are absent" negative, which the
round-4 audit had (correctly, on the evidence then available) treated as anti-copying
("a copy of the route table would include them"). `round5/ORACLE_NOTES.md` re-probed
with theory-sub-path shapes and found both routes ARE registered; §14.8 is rewritten
and the two families implemented. The reversal is itself probe/fixture-driven, not a
transcription — audited below.

### What the delta reproduces, and the upstream it maps to

* `Server::del_path` / `del_proof` (dispatch.rs) ↔ `getDeleteStepR` (Handler.hs
  1587–1604) and `getDeleteStepDiffR` (1607–1635). Three-way branch
  lemma→proof-node→other with a distinct JSON alert per branch, plus a fresh-version
  redirect on a deletable node.
* `Server::verify` (dispatch.rs) ↔ `getTheoryVerifyR` (Handler.hs 833–841): a present
  lemma's `proof` path → same-index `overview/proof` redirect, everything else → the
  help pane. The clean predicate `lemma_present` maps to upstream's `editProof`→
  `lookupLemma name` existence check (Handler.hs 191–193).
* `ThyPath` + `ThyPath::parse(segs, diff)` (route.rs) ↔ `parseTheoryPath`
  (Types.hs 417–456) / `parseDiffTheoryPath` (459–556).
* `Handler::DelPath`/`Verify` route grammar (route.rs) ↔ Yesod route table
  (Types.hs 574 `verify` trace-only, 592 `del/path` trace, 615 `del/path` equiv; NO
  equiv `verify` row).
* Three `ProverOps` callbacks (`lemma_present`, `del_lemma_path`, `del_proof_step`).

### Filtration — shared elements that carry no protectable expression

1. **The three alert strings are verbatim upstream** — "Sorry, but removing the
   selected lemma failed!", "Sorry, but removing the selected proof step failed!",
   "Can't delete the given theory path!" (Handler.hs 1595/1601/1604, dup 1615/1621/
   1626/1632/1634). All three are boundary output a byte-compatible reimplementation
   MUST emit (merger/scenes-à-faire) and all three are captured: `del_path_bad.json`
   pins the "Can't delete" string; [R54] logs the other two live. Verbatim
   reproduction is required, not taken — same posture as METHOD_FAILED_ALERT in R4.
2. **The URL-grammar keyword set** (help·message·rules·tactic·cases·lemma·proof·
   method·add·edit·delete for trace; help·diffrules·diffProof·diffMethod for equiv;
   the `del/path` two-literal prefix; `verify` single literal) is the wire interface —
   the exact path tokens the client sends. Behavior-dictated and individually probed:
   [R51]/[R54] exercise every trace head, [R56]/[R57] the equiv heads and the
   `del`-without-`path` miss, [R55] the parse-before-method (unparseable→404-any-method,
   parseable+non-GET→405) ordering. Unprotectable boundary tokens.
3. **The redirect target forms** `overview/lemma/{name}`, `overview/proof/{lemma}
   [/verbatim-path]`, `overview/diffProof/{lemma}[/path]`, same-index for verify /
   fresh-index for del/path — all observable and probe-pinned ([R51],[R52],[R53],
   [R57]; fixtures `del_path.json`, `verify_proof.json`); the help-pane envelope
   byte-identical to `main/help` ([R50],[R51], fixture `verify.json`).
4. **The coarse handler branch structure** (lemma vs proof-node vs other for del;
   proof-of-present-lemma vs everything-else for verify) is dictated by the distinct
   observable outcomes (three alert strings / redirect-vs-help), not by upstream's
   textual arrangement.

### Comparison — affirmative evidence of independent (probe-derived) construction

The clean side does NOT track upstream's protectable expression where it could have:

* **Different parser architecture.** `parseTheoryPath` dispatches head-then-tail
  (`case x of "help" -> …; "lemma" -> parseLemma xs`) with per-head sub-parsers and
  `listToMaybe`/`(kind:y:z:_)` trailing-tolerance. `ThyPath::parse` matches the WHOLE
  slice with exact-length arms and numeric guards (`["cases", kind, level, n] if …`).
  These are structurally unrelated implementations with different edge-case behavior
  (see fidelity notes) — the opposite of a transcription.
* **Coarser abstraction than the source type.** Upstream's `TheoryPath` has 11
  constructors; the clean `ThyPath` keeps only the three the web layer branches on
  (`Lemma`/`Proof`/`DiffProof`) and collapses the rest to a single `Other`. A copy
  would carry TheoryHelp/TheoryRules/TheoryMessage/TheoryTactic/TheorySource/
  TheoryMethod/TheoryEdit/TheoryAdd/TheoryDelete individually.
* **No tamarin-internal identifiers leak.** Clean names (`ThyPath`, `DelPath`,
  `Verify`, `del_lemma_path`, `del_proof_step`, `lemma_present`, `del_proof`,
  variants `Lemma`/`Proof`/`DiffProof`/`Other`) share none of upstream's
  `TheoryPath`/`DeleteStepR`/`TheoryVerifyR`/`removeLemma`/`applyProverAtPath`/
  `sorryProver`/`editProof`/`modifyTheory`/`DiffTheoryDiffLemma` constellation. The
  ProverOps callbacks are grouped by the web layer's needs, not per upstream route.
* **`ProverOps` holds the deletion semantics opaque.** Upstream deletes a proof step
  via `applyProverAtPath thy lemma proofPath (sorryProver (Just "removed"))` (the
  "reset to by sorry" is source expression); the clean side never reproduces that —
  `del_proof_step` returns an opaque `Option<Theory>` and the observed "reset to by
  sorry" ([R53]) lives only in BEHAVIOR.md as a probe note.
* **No comment lineage.** Shipped `dispatch.rs`/`route.rs`/`envelope.rs` comments
  describe current behavior only; grep for `round-4|previously|R47|overturn|artifact`
  across the three source files is empty (the reversal narrative is confined to
  BEHAVIOR.md/QUERIES.log/REPORT.md, and the `[R47]`-overturn note to a dispatch5.rs
  test comment). No upstream Haddock/Hamlet comment appears.

### Non-blocking fidelity notes (untraced generalizations that DIVERGE from upstream — NOT similarity findings, no redo required)

Per method I flag behavioral claims the new code embeds that are not fully traced to a
probe. Both below are the mirror image of a copying risk: they generalize BEYOND the
probed shapes and, where they do, DIVERGE from upstream — so they are affirmative
non-copying evidence, but they are genuine byte-fidelity gaps the team may want to
close. Neither is copied protectable expression; neither affects the verdict.

1. **Equiv grammar omits well-formed diff heads (and `difflemma` entirely).**
   `parseDiffTheoryPath` (Types.hs 459–556) also accepts the per-side heads
   `rules/{LHS|RHS}/{Bool}`, `message/{LHS|RHS}/{Bool}`, `lemma/{LHS|RHS}/{name}`,
   `proof/{LHS|RHS}/{lemma}[/…]`, `method/{LHS|RHS}/{lemma}/{n}[/…]`,
   `cases/{LHS|RHS}/{kind}/{Bool}/{i}/{j}`, and standalone `difflemma/{name}` — for all
   of which `getDeleteStepDiffR` runs (a real delete, or the "Can't delete" alert for
   `difflemma`'s sibling heads). The clean equiv grammar accepts only
   `help`·`diffrules`·`diffProof`·`diffMethod` and 404s the rest. The equiv-404 probes
   ([R56]; dispatch5 `del_path_equiv_uses_diffproof_grammar`) tested only BARE or
   trace-shaped forms (`rules`, `message`, `proof/L`, `lemma/L`, `cases/raw/0/0`) —
   every one of which ALSO 404s upstream for lack of a `Side` segment — so the observed
   404s do not license the stronger "these heads never parse under --diff" claim the
   grammar bakes in; `difflemma` was never probed at all. Behavioral probe to close:
   against a live --diff oracle, GET `del/path/rules/LHS/True`,
   `del/path/lemma/LHS/<name>`, `del/path/proof/LHS/<lemma>`,
   `del/path/method/LHS/<lemma>/1`, `del/path/cases/LHS/raw/True/0/0`, and
   `del/path/difflemma/<name>`; record each status + body and widen the equiv grammar
   to whatever actually parses (expected: real deletes for lemma/proof/difflemma,
   "Can't delete" for rules/message/cases/method).
2. **Exact-length trace matching vs upstream trailing-tolerance / signed indices.**
   `parseTheoryPath` ignores trailing segments (`"help" -> Just TheoryHelp`,
   `listToMaybe`, `(kind:y:z:_)`), so `help/x`, `tactic/x`, `lemma/a/b`, `edit/a/b`,
   `cases/raw/0/0/x` all PARSE upstream; the clean arms (`["help"]`, `["lemma", name]`,
   `["cases", kind, level, n]`, `["edit", _]`, …) are exact-length and 404 them.
   Upstream `safeRead`s case/method indices as `Int` (negatives/large accepted); the
   clean side uses `usize`. Only canonical shapes were probed. Non-blocking; if
   byte-fidelity on malformed-but-accepted inputs matters, probe trailing-segment
   variants and a negative case index and relax the arms accordingly.

(Also noted, not actionable: verify's same-index redirect is probe-true [R51], but
upstream's `editProof` re-extends the proof in place via `replaceTheory … idx`; whether
that in-place re-derivation is observable is held opaque behind `ProverOps` and outside
this unit. And upstream `map unprefixUnderscore` strips a leading `_` from each
theory-path segment before parsing; the clean side passes segments verbatim — matched by
the probed `_`-containing proof paths [R51]/[R53], unprobed for other `_`-prefixed
segments. Both are prover/edge-case fidelity, not expression.)

### VIOLATIONS (Round 5)

None. Every wire string and redirect/envelope form in the delta is boundary output
backed by a logged probe ([R50]–[R57]) or a committed capture (`round5/*.json`); the
theory-path grammar is observed URL tokens whose whole-slice exact-match structure,
coarse `Other` collapse, per-side omissions, and non-upstream identifiers affirmatively
show URL/probe derivation rather than transcription of `parseTheoryPath`/
`parseDiffTheoryPath` or the Yesod route table; the handler branch shapes are dictated
by the three observable alert strings and the redirect-vs-help outcome, not by
Handler.hs's textual `go`/`goDiff` arrangement; the deletion semantics stay opaque
behind `ProverOps`; no comment lineage. Two non-blocking byte-fidelity notes recorded
(equiv grammar under-covers well-formed diff heads incl. `difflemma`; exact-length trace
matching vs upstream trailing-tolerance) — both are untraced generalizations that
DIVERGE from upstream, i.e. non-copying evidence, and neither is copied protectable
expression nor requires redo. Findings that survive filtration: 0. No redo instructions
issued. VERDICT: PASS.

---

## Round 6 — both-sides similarity audit (weblayer delta: origin-aware page shell + state delegation)

Delta audited: `git diff` against HEAD 75807c0, restricted to `weblayer/`. Two items.
**ITEM 1** — an origin-aware page shell: `page::Origin { Local, Uploaded }`, a
`PageParams.origin`/`Meta.origin` field, a new `§RELOAD§` template slot with
`RELOAD_ITEM`, and `page::reload_item`/`append_item` gating the north-bar "Reload
file" and "Append modified lemmas to file" items. **ITEM 2** — a behaviour-neutral
state-delegation refactor: the `StateOps` trait + `InMemoryState` reference impl, with
`Server<P, S>` now generic over the state backend. Upstream mapped: `Web/Hamlet.hs`
`headerTpl` (166–210) / `headerDiffTpl` (222–262); `Web/Types.hs` `TheoryOrigin`
(168), `GenericTheoryInfo` (174–183), `WebUI.theoryVar :: MVar TheoryMap` (96/142);
`Web/Handler.hs` `getTheory`/`putTheory`/`replaceTheory` (173–382), `checkReloadOrigin`
(386–387).

### Provenance cross-check (every embedded behavioural claim is logged)

| claim baked into the code | probe |
|---|---|
| Reload present ⇔ origin==Local (independent of kind) | [R61] matrix; [R60] two-item diff |
| Append present ⇔ origin==Local AND kind==trace | [R61] matrix |
| Reload `<li>` sits between *Index* and *Actions*; Append is last *Actions* item | [R60] index-normalized full-page diff |
| origin is inherited through proof ops (per-version property) | [R62] |
| deterministic shell prefix/tail reproduce the four fixtures | [R63]; `r6_overview_*` captures |
| version model StateOps documents (monotonic-from-1, retained, in-place-vs-new) | §§13.1/14.3, [R45] (prior rounds) |

No claim in the delta lacks logged provenance. The four `r6_overview_*` fixtures carry
exactly the probed kind×origin matrix (trace+Local 1/1, trace+Upload 0/0, equiv+Local
1/0, equiv+Upload 0/0). BEHAVIOR §16 states "No file under `/home/kamilner/tamarin-rs/`
was read."

### Filtration

**ITEM 1 — the two gated items are boundary output; the gating is a merger truth table.**
The rendered markup of both items (`<li><form class="ajax-form ajax-form-full
reload-confirm" method="POST" action="…/reload">…Reload file…` and the
`get_and_append` append form) is the literal bytes the reference server emits, probed
byte-for-byte ([R60]) and committed as fixtures. Crucially the reload markup is **not
new this round**: the diff shows it was already inline in the prior-round `PAGE_PREFIX`
(it passed earlier audits as boundary output); this round only **relocates** it verbatim
into the named `RELOAD_ITEM` constant and adds the `§RELOAD§` slot. The gating logic
(`reload_item`: Local→item else ""; `append_item`: (Trace,Local)→item else "") is the
minimal encoding of the observed 2×2 truth table — merger. FILTERED.

**ITEM 2 — a behaviour-neutral internal refactor with no observable surface.** `StateOps`
/ `InMemoryState` / `Server<P,S>` change no served byte (all prior parity/dispatch tests
pass byte-identical against the default `InMemoryState`). There is therefore no boundary
output to compare; the only similarity question is whether the internal decomposition or
names transcribe upstream's state model — see comparison below. The lifecycle *contract*
it documents (monotonic-from-1, retention, in-place mutation) is the already-probed
version model of §§13.1/14.3, not new content. FILTERED (idea-level behaviour + prior-probed
lifecycle).

### Comparison — affirmative evidence of independent, observation-only construction

* **`Origin` is a 2-way OBSERVED distinction, not upstream's 3-way TYPE.** Upstream
  `TheoryOrigin = Local FilePath | Upload String | Interactive` (Types.hs 168) is a
  three-constructor sum carrying payloads (a FilePath, a name). The clean `Origin {
  Local, Uploaded }` is a payload-free two-valued enum: it collapses upstream's two
  non-Local cases (`Upload`, `Interactive`) into one because they render byte-identical
  shells (both are `not isLocalOrigin` → no Reload, no Append). This is a textbook
  filtration result — the clean model reflects the observed shell outcomes (two), not
  the source type (three). Even the shared token `Local` is boundary-derived, not
  transcribed: the probed help-pane text prints `from Local "<path>"` / `from Upload
  "<name>"` ([R60]), and the enum name `Uploaded` (past participle) ≠ upstream
  constructor `Upload`.
* **No `isLocalOrigin` helper; a different template decomposition.** Upstream inlines
  `$if isLocalOrigin origin` guards **inside** two separate Hamlet templates
  (`headerTpl` for trace, `headerDiffTpl` for diff), the diff template structurally
  **omitting** the append block. The clean side has one `PAGE_PREFIX` with two
  substitution slots filled by `reload_item(origin)` / `append_item(kind, origin)` —
  its own pre-existing slot machinery, with the kind gate centralized in a `(kind,
  origin)` match rather than expressed by which of two templates you are in. The
  upstream helper name `isLocalOrigin` appears nowhere.
* **Rendered-attribute order confirms probe derivation.** The clean/emitted reload form
  orders attributes `class … method … action …`; the Hamlet source (Hamlet.hs 179)
  orders them `method … action … class …`. The clean side matched the **rendered**
  bytes (what Yesod emits), not the template source order — a black-box tell.
* **`Meta`/`PageParams` are grouped by shell need, not upstream's `TheoryInfo`.** The
  clean structs carry `{name, version, filename, origin}` / `{theory_name, index,
  version, filename, origin}`. Upstream `GenericTheoryInfo` (Types.hs 174–183) carries
  `{index, theory, time, parent, primary, origin, autoProver, errorsHtml}` — the
  proof-search/bookkeeping fields (`parent`, `primary`, `autoProver`, `errorsHtml`) are
  absent from the clean grouping. Only `origin`/`index` overlap, both
  behaviour-descriptive and inevitable. Notably the clean side models origin
  **inheritance** as a prover-reported `Meta.origin` ([R62]) and does **not** reproduce
  upstream's `parent :: Maybe TheoryIdx` pointer through which upstream propagates origin.
* **`StateOps` is a Rust-idiomatic pluggable backend, not upstream's MVar+Map.** Upstream
  owns version state as `WebUI.theoryVar :: MVar (M.Map TheoryIdx EitherTheoryInfo)`
  (Types.hs 96/142), mutated by `getTheory` (`M.lookup` under `withMVar`, Handler.hs
  173–176), `putTheory` / `replaceTheory` (`M.insert` under `modifyMVar`, 341–382).
  The clean trait exposes `insert_new` / `get` / `replace` / `remove` / `entries` over
  an abstract backend so a consumer can supply an async caching owner. No identifier
  overlaps beyond the generic verbs `get`/`replace`; `insert_new` ≠ `putTheory`,
  `entries` ≠ the `withMVar … pure` whole-map read, and the MVar/Map/`theoryVar`/
  `TheoryMap` machinery has no clean counterpart. A grep of the whole delta (src +
  tests) for upstream state/type identifiers (`TheoryInfo`, `theoryVar`, `TheoryMap`,
  `getTheory`, `putTheory`, `replaceTheory`, `isLocalOrigin`, `TheoryOrigin`,
  `modifyMVar`, `parent`, `primary`, `EitherTheory`, `WebUI`) returns nothing — the only
  `Interactive` hits are the pre-existing `interactive-graph-def` route, unrelated to
  the `TheoryOrigin.Interactive` constructor.
* **No upstream comment lineage.** No Haddock/Hamlet comment is reproduced; upstream's
  `-- Check if theory origin is a local file (needed for reload functionality)`
  (Hamlet.hs 208) has no clean echo (the clean docs describe the observed shell effect).

### Non-blocking notes (NOT similarity findings, no redo)

1. **Comment-hygiene regression — process narration in shipped source.** The new doc
   comments carry provenance/round narration: `page.rs` Origin doc says "Observed live
   (round 6): …", and the `dispatch.rs` `StateOps` doc says "(probed live; `BEHAVIOR.md`
   §§13.1/14.3, §16)" and "see the honesty note in `REPORT.md`." This is **not** upstream
   comment lineage and **not** copied protectable expression — so it does not bear on the
   verdict — but it regresses the clean-room's own "comments describe current state only"
   standard that rounds 4–5 held (round 5 verified shipped comments were narration-free).
   Team may want to strip the round/"probed live"/§-reference phrasing so the shipped
   comments describe only current behaviour, keeping provenance in BEHAVIOR/QUERIES/REPORT.
2. **`Origin::Uploaded` is behaviourally broader than its name.** It encodes "any
   non-Local origin," i.e. it also covers upstream's `Interactive` (interactively-created)
   case, which is likewise non-Local and produces the same (no Reload/no Append) shell.
   Behaviourally complete for the shell and affirmatively non-copying (upstream's third
   constructor is not reproduced). Only if the index-page origin **column** text
   (upstream renders `"(interactively created)"`, Hamlet.hs 139) ever enters web-layer
   scope would a third case be observable — currently that string is a non-deterministic
   prover fragment held opaque in `RootMeta`, out of scope, and unprobed this round.

### VIOLATIONS (Round 6)

None. ITEM 1's two gated items are boundary output (the reload markup relocated verbatim
from the already-audited `PAGE_PREFIX`, the append item pre-existing), each present/absent
per a logged kind×origin probe ([R60]–[R63]); the gating is the minimal truth-table
encoding (merger); the `Origin` model is a 2-way observed distinction that affirmatively
diverges from upstream's 3-way `TheoryOrigin` type, uses none of upstream's identifiers
(`isLocalOrigin`/`Upload`/`Interactive`/`parent`), and matches rendered — not
Hamlet-source — attribute order. ITEM 2 is a behaviour-neutral refactor whose `StateOps`
backend shares no identifier or structure with upstream's `MVar TheoryMap` +
`getTheory`/`putTheory` model. Two non-blocking notes recorded (shipped-comment process
narration to scrub; `Uploaded` broader than its name) — neither is copied protectable
expression nor requires redo. Findings that survive filtration: 0. No redo instructions
issued. VERDICT: PASS.

---

## Round 7 — both-sides similarity audit (weblayer delta: concurrency-safe dispatch)

Scope: this round's delta only. Baselined against clean-room HEAD `e76455a` (pre-round);
`git diff` restricted to `weblayer/`. Upstream reference read for this audit:
`src/Web/Handler.hs` (state/thread machinery: `getTheory` 173–176, `putTheory`/`putDiffTheory`
341–382, `replaceTheory`/`replaceDiffTheory` 300–338, `adjEitherTheory` 522–534, `delTheory`
488–495, `getTheories` 497–501, the `threadVar` subsystem `putThread`/`delThread`/`getThread`
560–592, `evalInThread` 634–652, `withTheory`/`withBothTheory`/`withDiffTheory`/`withEitherTheory`
656–706), `src/Web/Types.hs` (`WebUI.theoryVar :: MVar TheoryMap` / `threadVar :: MVar ThreadMap`
96/142, `TheoryMap`/`ThreadMap`/`TheoryIdx` 88–96, `GenericTheoryInfo` 174–183), `src/Web/Settings.hs`.

Delta contents: `src/dispatch.rs` (the substantive change); `tests/dispatch7.rs` (new, 2 tests);
`tests/dispatch{,4,5,6}.rs` (mechanical API adaptation); `BEHAVIOR.md` §17 + §16.2 note;
`QUERIES.log` [R70]–[R76]; `REPORT.md` Round-7 section.

### What the round changes (abstraction)

A behaviour-neutral concurrency refactor of the state façade. `StateOps` goes from a
`&mut self` / borrow-returning trait to an **interior-mutability, snapshot-handing** trait:
`get(index) -> Option<&T>` → `snapshot(index) -> Option<T>` (owned clone); `entries() ->
Vec<(u64,&T)>` → `indices() -> Vec<u64>`; `insert_new`/`replace`/`remove` become `&self`.
`InMemoryState<T>` wraps `Mutex<StateInner<T>>` (`StateInner = { versions: BTreeMap<u64,T>,
next_index: u64 }`), each method holding the lock only for the map/counter op. `Server::dispatch`
becomes `&self`; every theory-scoped handler takes one snapshot at entry (`thy()`), computes
lock-free, and commits via a separate atomic `StateOps` call — the get-snapshot → compute →
commit pipeline. No wire output changes (all prior byte-parity tests stay green under
`Server::new`/`InMemoryState`).

### Provenance cross-check — the concurrency contract is probe-derived, not source-derived

Every behavioural claim the redesign rests on is logged as a live black-box probe in
`QUERIES.log` [R70]–[R76] (ports 3100/3101; PKCS11 `cannot_obtain_key_ind` ~30s as the long op)
with concrete timings, and written up as behaviour in `BEHAVIOR.md` §17:

* Non-blocking under a long op ([R71], §17.1) — burst at t≈2.5s into a 30.56s autoprove; reads
  on other/same theories, a 2nd proof op on a different AND the same theory, upload, reload all
  returned 0.02–0.52s, all `200`, all before the op's 30.5s finish.
* Commit-time allocation ([R72]/[R73], §17.2) — the long op started first (t=0.014s) but committed
  last (t=30.57s) and got the highest index (10); fast ops that committed at t≈2.6s got 8/9;
  index-page polling never showed the pending index until after completion.
* Atomic under races ([R74], §17.3) — 12 simultaneous ops → indices 12..23, contiguous, no
  collision, no skip.
* Snapshot isolation ([R75], §17.4) — a long autoprove on idx3 completed (→ idx24) despite an
  in-place reload of idx3 mid-compute; retention reconfirmed.

These are emergent *runtime* properties (where the commit sits relative to the compute; visibility
timing; race behaviour) — precisely the things NOT legible from reading `Handler.hs`, and correctly
obtained by observation. `dispatch7.rs` re-encodes exactly this interleaving with a `GatedProver`
whose `autoprove` parks on a condvar gate (start-first / fast-commit-first → idx2 / pending idx
invisible + unresolvable / release → slow commits last → idx3; plus a 16-thread allocation race).
No prover source was needed to author it.

### Filtration — shared elements carry no protectable expression

* **The three-phase snapshot → compute → commit shape is merger / scenes-à-faire.** Given the
  observed constraints ([R71]/[R72]/[R74]/[R75]) — a multi-second prover call must not freeze
  other requests, allocation is atomic at completion, a concurrent in-place replace must not
  corrupt an in-flight compute — "read an owned snapshot, drop the lock, compute, re-acquire to
  commit atomically" is the single correct implementation any competent engineer converges on.
  It is also what upstream does, but for a language reason, not a copied one: Haskell values are
  immutable, so `getTheory`'s `withMVar … M.lookup` yields an owned immutable snapshot **for
  free**, the heavy `closeTheory`/autoprove runs on that value with no `MVar` held, and `putTheory`
  re-takes the `MVar` via `modifyMVar` only to `M.insert`. The clean side must reproduce the same
  isolation *explicitly* (`snapshot` returns a `clone()`, gated on a new `T: Clone` bound) — a
  Rust-necessity divergence driven by the observed behaviour, not by reading that Haskell gets it
  gratis. Convergent by external constraint; unprotectable.
* **Monotonic index allocation** (first = 1, never reused, atomic; §13.1/§17.2/§17.3) is
  behaviourally observed AND expressed differently from upstream. Upstream is stateless —
  `idx | M.null theories = 1 | otherwise = fst (M.findMax theories) + 1` recomputed per insert
  (Handler.hs 351–352/373–374); the clean side keeps a stored `next_index` counter that never
  rewinds even after `remove`. These are behaviourally distinguishable (delete-the-top-then-insert:
  upstream reuses the freed index, the clean counter does not) — affirmatively non-copying, not
  merely a rename.
* **`Mutex<BTreeMap + counter>` vs `MVar (Map …)`.** One lock guarding the version map is
  scenes-à-faire for "single owner of a version namespace"; `Mutex` is the idiomatic Rust analogue
  of a single guarding `MVar`, and a sorted map keyed by index is dictated by the ascending
  index-page enumeration. The nested `InMemoryState { inner: Mutex<StateInner> }` is an ordinary
  Rust wrapping idiom with no upstream counterpart (upstream is a bare `MVar` field on `WebUI`).
* **Resolve-or-404 per theory route.** `thy()` taking one snapshot and 404-ing on a missing (or
  `#`/current) index is the observable contract of every theory-scoped route; upstream expresses
  it as a family of typed combinators (`withTheory`/`withBothTheory`/`withDiffTheory`/
  `withEitherTheory`, Handler.hs 656–706), the clean side as a single snapshot threaded through
  `match` arms. Same behaviour, different decomposition.

### Comparison — affirmative evidence of independent, observation-only construction

* **Identifier constellation: zero overlap.** New/changed clean identifiers —
  `StateOps::snapshot`, `indices`, `insert_new`, `replace`, `remove`, `InMemoryState`, `StateInner`,
  `versions`, `next_index`, `Server::dispatch` — mirror none of upstream's state/thread
  vocabulary (`theoryVar`, `threadVar`, `TheoryMap`, `ThreadMap`, `TheoryIdx`, `getTheory`,
  `putTheory`, `replaceTheory`, `adjEitherTheory`, `delTheory`, `storeTheory`, `getTheories`,
  `evalInThread`, `putThread`, `delThread`, `getThread`). A grep of the whole delta (src + all
  tests) for upstream state/thread/type identifiers (`theoryVar`, `threadVar`, `TheoryMap`,
  `ThreadMap`, `TheoryIdx`, `getTheory`, `putTheory`, `replaceTheory`, `adjEitherTheory`,
  `delTheory`, `evalInThread`, `modifyMVar`, `withMVar`, `MVar`, `findMax`, `WebUI`,
  `EitherTheoryInfo`, `ThreadId`, `killThread`, `forkIO`) returns nothing.
* **Upstream's most distinctive concurrency expression is NOT reproduced.** The identifiable
  Handler.hs machinery here is the cancellable-thread subsystem: `threadVar :: MVar ThreadMap`
  keyed by the *rendered request URL*, `evalInThread` (fork the compute into a killable thread,
  register, wait, unregister), and `putThread`/`delThread`/`getThread` feeding a `/kill` →
  `killThread` route. The delta introduces **none** of it — no thread registry, no URL→ThreadId
  map, no cancellation. The clean model is a strict subset (pure snapshot/compute/commit), so the
  one subsystem a copier would most plausibly lift is affirmatively absent. (The `/kill` 400-page
  route exercised by `dispatch4` is boundary URL grammar audited in an earlier round, not the
  `threadVar` mechanism, and is not in this round's delta.)
* **No comment lineage.** The new doc comments describe the observed contract in the clean-room's
  own terms; no Haddock prose is echoed — upstream's "-- | Load a theory given an index."
  (getTheory), "-- | Store a theory, return index." (putTheory), "-- | Fully evaluate a value in a
  thread that can be canceled." (evalInThread) have no clean counterpart.
* **Test-file changes are mechanical.** `dispatch{,4,5,6}.rs` diffs are `let mut s` → `let s`,
  `body(&mut Server)` → `body(&Server)`, and the `WrapState` backend + `InMemoryState` contract
  tests re-pointed from `get`/`entries` to `snapshot`/`indices`. Generic clean identifiers
  throughout; no theory sources or upstream names introduced. `dispatch7.rs` uses only the clean
  public API and a self-contained condvar gate.

### Non-blocking note (NOT a similarity finding, no redo, does not bear on the verdict)

1. **Shipped-comment process narration expanded again.** The Round-6 audit already flagged
   provenance/round narration leaking into shipped `dispatch.rs`/`page.rs` doc comments as a
   regression of the clean-room's own "comments describe current state only" standard. This round
   *adds* to it: the module and trait docs now carry inline `BEHAVIOR.md §17` / `§17.2/§17.3` /
   `§17.4` section-references and an `([R71])` probe citation, plus phrasing like "matching the
   probed completion-order allocation" and "see the honesty note in `REPORT.md`". This is the
   clean-room's own provenance narration — the *opposite* of copied expression — so it has **no**
   effect on the similarity verdict; it is recorded only for hygiene consistency with the Round-6
   note. Team may want to move the §/[R]-references and "probed"/"matching the probed" phrasing
   out of shipped source into BEHAVIOR/QUERIES/REPORT, leaving comments to describe current
   behaviour only.

### VIOLATIONS (Round 7)

None. The delta is a behaviour-neutral concurrency refactor. Its snapshot → compute → commit
shape is dictated by the probed non-blocking / commit-time-allocation / atomicity / snapshot-
isolation behaviour ([R71]–[R75]) and is the single correct concurrent implementation (merger /
scenes-à-faire); its monotonic allocation is expressed via a stored counter that affirmatively
diverges from upstream's stateless `findMax + 1`; its `Mutex<BTreeMap + counter>` shares no
identifier or structure with upstream's `MVar TheoryMap` model; and it pointedly does **not**
reproduce upstream's distinctive `threadVar`/`evalInThread`/`killThread` cancellation subsystem.
Identifier-constellation overlap with `Handler.hs`/`Types.hs`: none. Comment lineage: none. One
non-blocking hygiene note recorded (shipped-comment process narration), which is not copied
protectable expression and requires no redo. Findings that survive filtration: 0. No redo
instructions issued.

VERDICT: pass

## Round 7-V — both-sides similarity audit (weblayer delta: verification-only re-corroboration, [R77] resumed session)

### What the round changes (abstraction)

`git status`/`git diff HEAD` restricted to `weblayer/` shows the entire delta is **three
workspace documents** — `workspace/BEHAVIOR.md` (+8), `workspace/QUERIES.log` (+19),
`workspace/REPORT.md` (+26). **Zero source, test, fixture, or `Cargo.*` files changed.** Every
`.rs` under `web-clean/` is byte-identical to HEAD (`git status` lists none as modified; the
Round-7 dispatch code — `dispatch.rs`, `dispatch7.rs` — was landed and audited in the prior
`## Round 7` section and is untouched here). This is a *verification* round: a fresh session
re-ran a live black-box probe and wrote up the corroboration, adding no code.

The three additions all encode the same thing — an independent live re-confirmation, logged as
`QUERIES.log [R77]`, of the §17.1–§17.3 concurrency contract that Round 7's dispatch already
implements:

* **QUERIES.log [R77]** — the probe log: reference `tamarin-prover` interactive on `scratch/r7/
  served` (idx1 Tutorial / idx2 NSLPK3 / idx3 NAXOS_eCK / idx4 RYY), port 3100, driven by
  `scratch/r7/probe_verify.sh`. Three observations with concrete timings — (a) non-blocking 5-way
  burst during an in-flight RYY autoprove (all `200`, 0.02–0.34s), (b) commit-order index
  allocation (start-first RYY committed last → higher index), (c) invisible-until-commit (a ~1.5s
  NAXOS autoprove's index absent from every poll across its compute window, appearing only
  post-commit, contiguous).
* **BEHAVIOR.md §17.7** — the same three facts restated as behaviour, explicitly flagged "my own
  hands, not the [R70]–[R76] logs."
* **REPORT.md "Round 7 — verification pass"** — restates the implementation contract in the
  clean-room's own vocabulary and cross-links the [R77] evidence; closes with "No file under
  `/home/kamilner/tamarin-rs/` was read this session; the only tamarin-rs touch was EXECUTING the
  sanctioned reference binary for live probing."

### Provenance cross-check — the added prose is probe-derived, not source-derived

The probe is real and black-box. `scratch/r7/probe_verify.sh` is present and is pure `curl` HTTP
against the running reference server (`B=http://127.0.0.1:3100`): it backgrounds one long
autoprove (`GET /thy/trace/4/.../proof/key_secrecy_PFS`), fires a concurrent burst of reads /
one method op / one `POST /` upload ~0.4s in, and polls `GET /` for version visibility on a
0.6s cadence, recording per-request start/end offsets and HTTP codes. The served corpus
(`served/{Tutorial,NSLPK3,NAXOS_eCK,RYY_PFS}.spthy`) matches the index map in [R77]. Every fact
in the three added blocks is an emergent *runtime* property — where the commit sits relative to
compute, visibility timing, non-blocking under load — exactly the class of property NOT legible
from reading `Handler.hs` and correctly obtained by observation. Timings (0.004/0.427/0.634/
1.536/1.564 s) and indices (5/6/7/8) are probe *outputs*, not constants lifted from source.

### Filtration — nothing added carries protectable expression

* **The behavioural claims are the observed contract, not upstream text.** Non-blocking under a
  long op, commit-time index allocation, invisible-until-commit — these are the same
  merger/scenes-à-faire runtime constraints filtered in the `## Round 7` Filtration section, here
  merely re-observed. No new design element is introduced.
* **Every identifier the prose names is the clean-room's own.** REPORT.md re-states
  `Server::dispatch(&self)`, `StateOps` (`snapshot`/`indices`; `insert_new`/`replace`/`remove`),
  `InMemoryState<T>` = `Mutex<BTreeMap + counter>`, `ProverOps`. Grepped against the live tree:
  all are defined in `web-clean/src/dispatch.rs` (`trait ProverOps` L233; `trait StateOps` L349
  with `snapshot`/`insert_new`/`replace`/`remove`/`indices`; `struct InMemoryState` L375 over
  `Mutex<StateInner{ versions, next_index }>`; `fn dispatch(&self)` L470) — so the narrative
  describes clean-room code truthfully, and the stored `next_index` (starts 1, never rewinds)
  remains the affirmative divergence from upstream's stateless `fst (M.findMax theories) + 1`
  (`Handler.hs` 352/374). None of these mirror upstream's `theoryVar`/`TheoryMap`/`TheoryIdx`/
  `getTheory`/`putTheory`/`replaceTheory`/`delTheory`/`WebUI` vocabulary.
* **The URL/form tokens in the probe are the observable wire boundary.** `/thy/trace/<i>/
  overview/help`, `/thy/trace/1/main/method/Client_auth/1`, `.../autoprove/idfs/0/False/proof/
  <lemma>`, `uploadedTheory=…` — the HTTP route grammar and form-field name a client necessarily
  hits, audited as boundary content in Rounds 4/5. Theory/lemma names (Tutorial, NSLPK3,
  NAXOS_eCK, RYY, `key_secrecy_PFS`, `eCK_same_key`) are example-corpus probe *inputs* (test
  fixtures), consistent with [R70]–[R76]; not web-layer source identifiers.

### Comparison — affirmative signals of independent, observation-only construction

* **Identifier constellation: still zero overlap.** A scan of the full added text finds no
  upstream state/thread/type name (`theoryVar`, `threadVar`, `TheoryMap`, `ThreadMap`,
  `TheoryIdx`, `getTheory`, `putTheory`, `replaceTheory`, `delTheory`, `modifyMVar`, `withMVar`,
  `MVar`, `findMax`, `WebUI`, `evalInThread`, `killThread`, `forkIO`). Upstream's distinctive
  cancellable-thread subsystem (`threadVar`/`evalInThread`/`/kill`→`killThread`) is neither
  reproduced nor referenced — the [R77] probe exercises only the snapshot/compute/commit subset.
* **No comment lineage.** The added prose is workspace documentation in the clean-room's own
  terms; no Haddock string is echoed (upstream's `-- | Load a theory given an index.` /
  `-- | Store a theory, return index.` have no counterpart here).
* **Explicit non-access affirmation.** REPORT.md records that no `/home/kamilner/tamarin-rs/`
  file was read this session and the only reference touch was *executing* the sanctioned binary
  for live probing — the correct clean-room posture, and consistent with the doc-only diff.

### Non-blocking note (NOT a similarity finding, no redo, does not bear on the verdict)

The `## Round 7` audit carried a hygiene note that shipped `dispatch.rs`/`page.rs` doc comments
had accreted `BEHAVIOR.md §`/`[R]` process narration. This round touches **no** shipped source,
so it neither worsens nor addresses that note; the newly added `§17`/`[R77]` cross-references all
live in the workspace docs (BEHAVIOR/QUERIES/REPORT), where provenance narration belongs. Recorded
only for continuity; no effect on the verdict.

### VIOLATIONS (Round 7-V)

None. The delta is documentation-only — a live black-box re-corroboration ([R77], `probe_verify.sh`
present) of the already-audited §17.1–§17.3 concurrency contract, adding no code. Behavioural
claims are probe-derived runtime properties (merger/scenes-à-faire, not legible from `Handler.hs`);
every identifier named is the clean-room's own (verified defined in `dispatch.rs`), with the stored
`next_index` counter still affirmatively diverging from upstream's `findMax + 1`; wire/URL tokens
and theory names are boundary/corpus content; no upstream state/thread vocabulary, structure beyond
behaviour, non-boundary magic constant, or comment lineage appears. Identifier-constellation overlap
with `Handler.hs`/`Types.hs`: none. Findings that survive filtration: 0. No redo instructions issued.

VERDICT: pass

---

# PRODUCERS cluster — Round 1 — dual audit (barrier hygiene + similarity)

New sub-cluster `weblayer/producers/` (untracked; repo HEAD `b4fb110` predates it). The
scaffold (`SPEC.md`, `README.md`, `interface/`, `oracle/`, `round1/`) is OPEN-side-authored;
the sealed side is `workspace/producers-clean/` + `workspace/{BEHAVIOR.md,QUERIES.log}`. R1
(center section fragments + shared HTML skin) is the only implemented surface; R2–R5 are
pure `unimplemented!()` stubs. Two audits, both run.

Upstream consulted (auditor privilege): `src/Web/Theory.hs`
(`messageSnippet`/`rulesSnippet`/`tacticSnippet`/`ppSection`/`ppWithHeader`,
`htmlThyPath`/`pp`, `helpHtml`, `titleThyPath`), `lib/utils/src/Text/PrettyPrint/Html.hs`
(`postprocessHtmlDoc`/`escapeHtmlEntities`/`withTag`), `src/Web/Handler.hs`
(`responseToJson`). Method: for audit 2, abstraction–filtration–comparison, every behavioural
claim cross-checked to a `QUERIES.log` probe or a `round1/` capture.

## AUDIT 1 — Barrier hygiene of the open-authored scaffold

**Interface files are CLEAN (the precedent vector).** The round-precedent (a `required_api.md`
that leaked splice-anchor names) does **not** recur: `interface/required_api.md` names only
indicative API entry points and states explicitly "none are specified in this file"; the fn
names (`render_content_pane`/`escape_text`/`postprocess_lines`/`html_envelope`/…) are behaviour
descriptive and share nothing with upstream internals. `interface/fragment_inputs.rs` carries an
expression-stripped header ("not a transcription of any existing data model") and names every
type/field for observable behaviour. `round1/{families.tsv,fetch_targets.sh}` and
`oracle/{extract_fragments.py,hs_server.sh}` operate on captured OUTPUT only — feature tags,
URL skeletons, capture hashes; no upstream identifier, no port seam, no `.hs` citation. A scan
of the whole scaffold for ~35 upstream Haskell *function* identifiers (`theoryIndex`,
`linkToPath`, `markStatus`, `postprocessHtmlDoc`, `escapeHtmlEntities`, `withTag`,
`caseEmptyDoc`, `titleThyPath`, `responseToJson`, `JsonHtml`, …) returns **zero** — no
expression-bearing upstream name reaches sealed-readable material.

**FINDING PH-1 (barrier hygiene — scaffold rewrite required).** `SPEC.md` cites upstream
Haskell **source-file paths** in sealed-readable text: the sub-target table's "HS citation it
advances" column and the "Author topology" paragraph name `src/Web/Theory.hs`,
`src/Web/Hamlet.hs`, `src/Web/Handler.hs`, `src/Web/Types.hs`, and `Text/PrettyPrint/Html.hs`
(and `README.md` repeats the glob `src/Web/*.hs`). This **diverges from the established
sibling norm**: `pretty/SPEC.md`, `graphdot/SPEC.md`, and `wellformedness/SPEC.md` cite **zero**
upstream source files, conveying the identical citation-yield / author-topology purely through
the *ported* `.rs` files that get deleted plus the per-file author lists. The leak here is
file-level licensing bookkeeping — no function name, no decomposition, no anchor, no expression
— and the pristine `interface/` contract the implementer compiles against is unaffected; but it
is an avoidable over-share that points the sealed reader straight at the upstream files and
weakens the black-box posture.
*Rewrite instruction (scaffold, `SPEC.md` + `README.md`):* recast the "HS citation it advances"
column and the Author-topology text in terms of the **ported `.rs` files** that carry those
citations (`handlers/theory_html.rs`, `proof_tree.rs`, `root.rs`, `path_parse.rs`, plus the
already-clean dispatch shells) exactly as `pretty/SPEC.md` does, and delete every
`src/Web/*.hs` / `Text/PrettyPrint/Html.hs` string from sealed-readable text. Keep the author
usernames and the port-seam file names (those are within norm).

**Within-norm (recorded, NOT findings).** (a) `SPEC.md` naming port-seam files + port-internal
fn names (`handlers/theory_html.rs`, `proof_tree.rs::render_proof_tree_html`, `root.rs`,
`path_parse.rs`, `proof_state`, `web_clean::proofscript`) and the pseudonymous authors
(Kanakanajm/YannColomb/Schoop) matches `pretty/SPEC.md` verbatim in kind (which names
`render_signature`/`render_ac_variants_block`/`rule_attributes_doc` etc. + per-file author
lists and passed audit). Advisory only: the two *function*-level names could be dropped to
file-level for extra margin, but they leak no upstream expression. (b) `SPEC.md` line 85's
"`withHeader` framing" is a **phantom** — no such identifier exists anywhere in the upstream
tree; it is a coined term, harmless, though its Haskell-ish look could be tidied.

## AUDIT 2 — Similarity of the sealed crate (abstraction–filtration–comparison)

**Abstraction.** R1 renders three panes (`main/message`/`rules`/`tactic`) as a headed-block
document — `<h2>HEADING</h2>` + `<p class="monospace rules">BODY</p>` per block — through a
shared skin (`html.rs`: entity-escape, per-line `&nbsp;`/`<br/>` postprocess, the
`{html,title}`/`{redirect}`/`{alert}` envelopes) and wraps `main/help` as a single-line env
line + a fixed static block. Upstream produces the same fragments via `messageSnippet`/
`rulesSnippet`/`tacticSnippet` (`ppSection`/`ppWithHeader` = `withTag "h2"` `$$` `withTag "p"
[("class","monospace rules")]`), `postprocessHtmlDoc`, `escapeHtmlEntities`, `helpHtml`, and
`titleThyPath`, enveloped by `responseToJson`.

**Filtration — every shared element is observable served output (merger / compatibility).**
The tag skeleton (`<h2>`, `<p class="monospace rules">`, `<br/>`, `&nbsp;`, the 5-entity escape
set, the `{html,title}` compact JSON with `html` key first), the fixed heading vocabulary
(Signature / Construction Rules / Deconstruction Rules; Macros / Fact Symbols with Injective
Instances / Multiset Rewriting Rules / Restrictions of the Set of Traces; Tactic(s)), the pane
titles ("Message theory" / "Multiset rewriting rules and restrictions" / "Tactics" / "Theory:
NAME"), the help env line `<p>Theory: NAME (Loaded at TIME from ORIGIN) BANNER</p>`, and the
verbatim static help block are all present in the captured response bodies and pinned by the
corpus sweep (324/324 raw-byte reassembly across 81 manifests) + fixture tests. Byte-forced
wire content; a faithful reimplementation has no freedom here. The `(Loaded at …)` parenthetical
and the highlight `hl_*` spans are additionally normalized away by the semantic acceptance gate,
i.e. not even on the bar.

**Comparison — affirmative evidence of independent, observation-only construction.**
- **Materially different postprocess expression.** Upstream `postprocessHtmlDoc = unlines . map
  (addBreak . indent) . lines` with `indent = … (first $ concatMap (const "&nbsp;")) . span
  isSpace` (whitespace incl. **tabs** via `isSpace`; `lines`/`unlines` drop the final empty
  segment). The clean `postprocess_lines` uses `split('\n')` + `trim_start_matches(' ')` +
  byte-length counting — **spaces only** (tabs pass through, documented as unobserved
  [S10]) and different trailing-newline semantics. Same observable bytes, divergent code.
- **Different empty-body decomposition.** Upstream mixes two mechanisms: `caseEmptyDoc emptyDoc
  (h2 $$ p) body` for vanish-when-empty and a separate `if null … then text empty` for the
  macros blank slot, and plain `ppSection` (no guard) for always-keep. The clean side unifies
  all three into one 3-variant `EmptyRender { Keep, BlankLine, Omit }` enum — its own
  abstraction, not upstream's control-flow split.
- **Different envelope expression.** `html_envelope` hand-builds the JSON with a manual
  `json_escape_into` string pass — unlike Aeson's `object ["html" .= …, "title" .= …]` and
  unlike the sibling `web-clean` crate's `serde_json`. Escape-arm order also differs from
  upstream's `escapeHtmlEntities` (`&`-first vs `<`-first).
- **Zero upstream-internal identifier overlap.** Grep of `src/` + `tests/` for the upstream
  constellation (`messageSnippet`/`rulesSnippet`/`tacticSnippet`/`ppSection`/`ppWithHeader`/
  `htmlThyPath`/`postprocessHtmlDoc`/`escapeHtmlEntities`/`withTag`/`caseEmptyDoc`/`helpHtml`/
  `titleThyPath`/`renderHtmlDoc`/`preEscapedToMarkup`/`errorsHtml`/`GenericTheoryInfo`/…)
  returns nothing. Clean names (`render_pane`/`render_help_pane`/`HeadedBlock`/`EmptyRender`/
  `ContentPane`/`HelpPane`) are all behaviour-descriptive.
- **No comment lineage.** Upstream comments ("Build the Html document showing the message
  theory", "converts the line-breaks of cs to `<br>` tags", "Copied from `blaze-html`") have no
  clean echo; grep for `blaze`/`Build the Html`/`converts the line-breaks` in `src/` is empty.
- **Source-only branches not reproduced.** Upstream's `pp` empty-string guard ("Trying to render
  document yielded empty string. This is a bug.") and its inline `if null injFacts then text
  "None"` fallback are **not** reimplemented — the clean side holds `None` as opaque adapter
  input (honestly logged as unobservable-at-boundary, BEHAVIOR §6). Rendered-byte artifacts that
  do **not** exist in the Hamlet source (the stray `</span>` after the Tamarin span) are
  reproduced byte-exactly — proof of capture-derivation, not source-reading.
- **Reproduced only what the captures exercise.** Full support for the two observed envelopes +
  the live-forced `{alert}`; the solver `monospace cases` pane is absent; R2–R5 are stubs.

**Provenance cross-check of the hardest-to-guess behavior (DISPOSITIVE).** `titleThyPath`
(Theory.hs 1589) makes the rules title an **unconditional constant** `"Multiset rewriting rules
and restrictions"` — it says "and restrictions" even for the 43 theories with zero restrictions.
`BEHAVIOR.md` §6 records that the sealed room **seeded the natural wrong hypothesis** (" and
restrictions" is *conditional*) and then **REFUTED it by observing all 81 captures** ([S07]).
Reading the source would have shown the constant immediately and no false hypothesis would ever
have been seeded; the refuted-hypothesis provenance is affirmative proof of black-box derivation.
The help title ("Theory: NAME", 1588), the "Tactic(s)" heading vs "Tactics" title distinction
(1591), and the `<p>Theory: NAME (Loaded at TIME from ORIGIN) BANNER</p>` env line (helpHtml
1187–1194) all match observable output; `QUERIES.log` [S01]–[S13] (corpus sweeps) + [L01]–[L07]
(live oracle, incl. an **own-authored** `EscProbe` theory + a metachar filename for the escape
set) carry no source-tree read.

## Non-blocking note (NOT a similarity finding; no redo; does not bear on the verdict)
Shipped-comment provenance narration — `src/html.rs`/`section.rs`/`model.rs` doc comments carry
inline probe citations (`[S07]`, `[L03]`, `BEHAVIOR.md §2`, …). This is the clean-room's own
provenance, the *opposite* of copied expression, so it has no similarity effect — but it
continues the Round-6/Round-7 hygiene note (the campaign's "comments describe current state only"
standard). Team may move the `[S..]`/`[L..]`/`§`-references into BEHAVIOR/QUERIES and leave the
shipped comments to describe current behaviour only.

## VIOLATIONS (Producers Round 1)
**Similarity: 0.** Every R1 resemblance to `Theory.hs`/`Html.hs`/`Handler.hs` reduces to
observable served output (tag skeleton, heading/title vocabulary, envelope, static help block,
help env line) — merger/compatibility content pinned by the 324/324 corpus sweep — while the
skin's *expression* (postprocess, empty-body modelling, envelope building, escape) is materially
divergent, the identifier constellation and comment lineage overlap is nil, source-only branches
are pointedly not reproduced, and the hardest behaviour carries refuted-hypothesis black-box
provenance. R2–R5 are `unimplemented!()` stubs with observable-behaviour doc comments only.

**Barrier hygiene: 1 finding (PH-1) — scaffold rewrite, non-contaminating.** `SPEC.md`/`README.md`
cite upstream `src/Web/*.hs` / `Text/PrettyPrint/Html.hs` file paths in sealed-readable text,
diverging from the sibling-SPEC norm; the rewrite instruction above recasts them onto the ported
`.rs` seams. The leak is file-level bookkeeping only — no function name, decomposition, anchor, or
expression crossed the barrier (verified scan), the `interface/` contract is pristine, and the
sealed crate is affirmatively, independently derived — so the barrier held where it counts and the
finding is a scaffold cleanup, not a contamination of the sealed work.

VERDICT: pass

---

# PRODUCERS cluster — Round 2 — both-sides similarity audit (R5 path grammar + R2 west-pane assembly)

Auditor scope: this round's delta only (clean-room HEAD `b4810fe` predates it). Delta from
`git status`/`git diff HEAD` restricted to `weblayer/producers/workspace/`: modified
`producers-clean/src/{path.rs,proofscript.rs,model.rs,lib.rs}`, `tests/corpus_sweep.rs`
(whitespace), `BEHAVIOR.md` (§§11–15 added), `QUERIES.log` ([S14]–[S18], [L08]–[L15]); new
`tests/{r5_path_grammar.rs,r2_west_pane.rs}`, `r2_live/`, `r2_panes/`, `r5_tails/`, and five
own tools (`harvest_hrefs.py`, `extract_r5_tails.py`, `extract_r2_panes.py`, `analyze_west.py`,
`probe_paths.sh`). Two implemented surfaces: **R5** the theory-path grammar (`path.rs`:
`parse`/`render`) and **R2** the proof-script west pane (`proofscript.rs`: `render_index`),
which constructs every link through R5. Upstream consulted (auditor privilege):
`src/Web/Types.hs` (`parseTheoryPath`/`renderTheoryPath` 360–456, `prefixWithUnderscore`/
`unprefixUnderscore` 400–414, `safeRead`=`listToMaybe . map fst . reads` 436, the Yesod route
table) and `src/Web/Theory.hs` (`theoryIndex` 371–416, `lemmaIndex` 296–329, `proofIndex`
223–260, `linkToPath` 204–213, `overview`/`ruleLinkMsg`/`casesInfo`/`reqCasesLink` 408–416,
`markStatus` 2148–2153). Method: abstraction–filtration–comparison; every grammar rule and
pane-assembly rule cross-checked to a logged probe/capture, not to the source.

## Provenance cross-check — every BEHAVIOR §11–§13 rule traces to a probe/capture

| rule baked into the code | probe / capture |
|---|---|
| split on `/` first, then percent-decode each segment; `%2F` does not split | [L09] (`me%73sage`==`message`, `proof/foo%2F_`→lemma `foo/_`) |
| decode: valid `%XX`→byte, UTF-8 w/ U+FFFD, invalid `%` literal, `+`≠space | [L12] (add-form echo channel) |
| heads exact + case-sensitive; extra segments after a complete match ignored | [L08]/[L09] (`MESSAGE`,`cases/RAW` 404; `help/extra`,`message/`,`cases/raw/0/0/extra` 200) |
| missing required arg rejects; name args accept any incl. empty; existence≠parse | [L08]/[L11] (`proof`,`cases/raw/1` 404; `edit/`,`proof//_` 200; `proof/nonexistent`→"No such…") |
| numeric segment = Haskell-`reads`-shaped int (parens/signs+space/radix/float-guard/`_`-quirk) | [L10] (68-item accept/reject battery, replayed verbatim in `r5_path_grammar.rs`) |
| numeric VALUE inert (0/0…9/9…-1/0 byte-identical); source-vs-case index unobservable | [L10] |
| version-index is a different, stricter grammar (out of R5) | [L10] |
| render: unreserved raw + uppercase `%XX`; only `[A-Za-z0-9_.]`+`%3C/%3E` observed | [S14]; [L13] (metachar-filename channel collapsed → beyond `%3C/%3E` UNOBSERVABLE) |
| west pane = logical lines through the §3 skin + ONE trailing space; 478 panes | [S16] (`analyze_west.py`; "SPEC's 473 undercounts by 5") |
| element order (header/5 items/add-first/lemmas/end); zero-lemma → TWO blanks | [S16] |
| `  or  ` (two spaces); header status span `hl_good`×3192 / `hl_bad`×146; sorry/incomplete unwrapped | [S16] |
| declaration `lemma NAME{ATTRS}:`, ATTRS opaque possibly multi-line | [S17] (46 multi-line decls) |
| inline iff assembled ESCAPED width ≤ 69, else vertical; metric = escaped chars | [S18] (minimal pair rules out visible-chars/bytes) + [L14] (live bisection pins 69/70 on 4 families) |

No claim in the delta lacks logged provenance. The five tools read only
`oracle/captured_responses/*.hs.json` (captured OUTPUT) or drive the live oracle over `curl`
(`probe_paths.sh`); none reads a source tree. Re-ran the suite: **24 tests green**
(R1 14 + R5 6 + R2 4), matching the QUERIES.log close.

## Filtration — every shared element is observable wire output (merger / compatibility)

* **R5 grammar tokens** (`help`·`message`·`rules`·`tactic`·`cases/{raw|refined}/i/j`·`lemma`·
  `proof`·`edit`·`add`·`delete`; the numeric leniency; `<first>`→`%3Cfirst%3E`) are the exact
  path strings the client sends / the server emits — the wire interface, individually probed
  ([L08]–[L13]) and pinned by the 40037-distinct-tail corpus round-trip ([S15]). Byte-forced.
* **R2 frame** (`theory NAME begin`/`end` keyword spans, `<a class="internal-link …">`,
  `<strong>LABEL</strong> ANN`, `proof-step sorry-step`, `add`/`edit`/`delete` class suffixes,
  the status wrapper `<span class="hl_good|hl_bad">`, `  or  `, the trailing space) is present
  byte-for-byte in the captured overview panes ([S16]) and pinned by the 478/478 slice-and-
  re-render sweep. A faithful reproduction has no freedom here.

## Comparison — affirmative evidence of independent, observation-only construction

* **The numeric grammar has NO upstream expression to copy.** Upstream `parseCases` merely calls
  `safeRead = listToMaybe . map fst . reads` — the parens/leading-`-`-with-space/`0x`·`0o`·`0b`/
  float-rejection/underscore-prefix quirks all live in GHC's Prelude `Read Int`, not in
  `Types.hs`. The clean side hand-rolls that behaviour (`parse_numeric`/`parse_int_item`/
  `parse_int_lexeme`/`skip_ws`) reconstructed entirely from the [L10] accept/reject battery.
  Reading `Types.hs` would have shown only `safeRead` and taught nothing about the quirks; the
  quirks are demonstrably probe-derived. This is the strongest independence signal of the round.
* **No `prefixWithUnderscore`/`unprefixUnderscore`.** Upstream maps `unprefixUnderscore` over
  every segment before parsing and `prefixWithUnderscore` over every segment when rendering
  (`Types.hs` 400–414); the clean side treats each segment as a literal (`decode_segment`/
  `encode_segment` only). The proof-path root marker `_` round-trips as the literal `_`
  (clean: `sub=["_"]`; upstream: `path=[""]` + prefix) to the SAME observed bytes — a materially
  different internal model reaching identical output. Same divergence the round-3/round-5 audits
  logged as anti-copying; still present.
* **Coarser type than the source.** `ThyPath` omits upstream's `TheoryMethod` (documented
  out-of-vocabulary; `method/…` routed to `None`, asserted on 497 corpus tails), models sources
  as `Sources { refined: bool, … }` rather than upstream's `TheorySource SourceKind i j`, and
  uses named fields (`source_idx`/`case_idx`/`sub`/`pos`) where upstream is positional. The parse
  match order (`help,message,rules,tactic,cases,lemma,proof,edit,add,delete`) diverges from
  upstream's (`help,rules,message,tactic,lemma,cases,proof,method,edit,add,delete`).
* **R2 holds MORE opaque than upstream computes.** Upstream `theoryIndex` computes the nav-item
  vocabulary inline (`ruleLinkMsg` = "Multiset rewriting rules" ± " and restrictions";
  `casesInfo`; `overview n info p`) and the formula via `prettyLNFormula`/`prettyTraceQuantifier`.
  The clean `render_index` takes `NavItem { label, annotation, target }` and the formula lines as
  OPAQUE adapter input — it reproduces none of that computation, and the `rules` label variation
  ([S16]: "Multiset rewriting rules"[" and restrictions"] 180/293, unlike the constant R1 title)
  is honestly recorded as opaque input, not reimplemented.
* **Materially different layout expression.** Upstream lays the quantifier/formula out with
  `nest 2 (sep [quantifier, doubleQuotes formula])` — HughesPJ's general fill algorithm at an
  ambient page width; the clean side special-cases the single-formula-line inline decision
  (`escaped_width(candidate) <= INLINE_WIDTH_LIMIT`, else vertical) with the boundary constant
  **69** and the escaped-char metric both bisected live ([L14]) rather than lifted from `sep`.
  The whole west pane is a flat `Vec<String>` + per-role helpers (`push_lemma`/`add_link`/
  `keyword`/`href`/`escaped_width`) versus upstream's `$-$`/`vcat`/`intersperse`/`markStatus`/
  `linkToPath`/`overview` combinator tower.
* **Identifier constellation: zero overlap.** Clean names (`parse`/`render`/`decode_segment`/
  `encode_segment`/`parse_numeric`/`parse_int_item`/`parse_int_lexeme`/`skip_ws`/`ThyPath`/
  `render_index`/`push_lemma`/`add_link`/`keyword`/`href`/`escaped_width`/`INLINE_WIDTH_LIMIT`)
  mirror none of upstream's (`parseTheoryPath`/`renderTheoryPath`/`prefixWithUnderscore`/
  `unprefixUnderscore`/`safeRead`/`TheoryPath`/`theoryIndex`/`lemmaIndex`/`proofIndex`/
  `linkToPath`/`markStatus`/`overview`/`ruleLinkMsg`/`casesInfo`/`reqCasesLink`). A grep of the
  sealed round-2 files for that constellation and for `src/Web`/`Hamlet`/`Blaze`/`.hs` returns
  nothing (the one `reads @Int` mention in `path.rs` names the OBSERVED integer shape, not a
  tamarin source symbol; the one `.hs.json` in `QUERIES.log` is the capture-file extension).
* **No comment lineage.** Upstream Haddock ("Render a theory path to a list of strings…",
  "Parse a list of strings into a theory path.", "Render the theory index.") has no clean echo.

## Tests are live byte gates, not vacuous
`r5_path_grammar.rs` replays the [L08]–[L12] battery (68 accept + 27 reject), the decode-echo
fixtures, parse⇄render round-trips, and the 40037-tail corpus sweep (497 `method/` asserted
`None`). `r2_west_pane.rs` STRICTLY inverts each pane (asserting every frame byte on the way
down, incl. the inline `≤69` / vertical `>69` witness) then re-renders and byte-compares
478/478, plus 3 live replays (PathProbe fresh, WProbe 35-lemma boundary, PathProbe v2 after a
live autoprove → proved `hl_good` tree) and the zero-lemma / minimal fixtures. QUERIES.log
records the three mutation checks (`  or  `→` or `, 69→68 corpus / 69→70 live-WProbe-only,
uppercase→lowercase `%XX`) each observed to FAIL then reverted — the gate is not vacuous.

## Barrier hygiene / PH-1 — the scaffold rewrite LANDED

The Producers-Round-1 finding **PH-1** (SPEC.md/README.md citing upstream `src/Web/*.hs` +
`Text/PrettyPrint/Html.hs` file paths in sealed-readable text) is **resolved**. `SPEC.md`'s
sub-target table now carries a "**ported file it retires**" column naming only ported `.rs`
seams (`handlers/theory_html.rs`, `proof_tree.rs`, `root.rs`, `path_parse.rs`), and the
"Author topology" section expresses citation-yield purely through those `.rs` files + the
pseudonymous usernames — exactly the sibling-SPEC norm (`pretty/SPEC.md`) the rewrite
instruction required. A scan of the ENTIRE sealed-readable tree (`SPEC.md`, `README.md`,
`interface/`, `oracle/`, `round1/`, `workspace/**` incl. the round-2 `.rs`/`BEHAVIOR.md`/
`QUERIES.log`/tools) for `src/Web`, `Text/PrettyPrint`, `Hamlet`, `Blaze`, and `*.hs`
source-file paths returns **zero** (the only `.hs` hits are the `*.hs.json` capture-data
extension). Nothing in the round-2 delta re-introduced an upstream citation. (Advisory,
unchanged from R1 and not a finding: SPEC.md line 85's coined "`withHeader` framing" phantom
and the R2 row's "473" undercount, both harmless and superseded by BEHAVIOR/[S16].)

## Non-blocking notes (NOT similarity findings; no redo; do not bear on the verdict)

1. **Shipped-comment provenance narration continues.** `path.rs`/`proofscript.rs`/`model.rs`
   doc comments carry inline `[S..]`/`[L..]`/`BEHAVIOR.md §` references — the clean-room's own
   provenance, the *opposite* of copied expression, so no verdict effect; it continues the
   Round-6/7/Producers-R1 hygiene note against the campaign's "comments describe current state
   only" standard. Two of the refs are additionally stale: `proofscript.rs` line 5 says "473"
   where [S16]/BEHAVIOR §12 say 478, and line 6 cites "BEHAVIOR.md §11" for the west-pane
   element order which lives in §12 (§11 is R5). Team may move the `[S..]`/`[L..]`/`§`-refs into
   BEHAVIOR/QUERIES and fix the two stale numbers.
2. **Render side omits `prefixWithUnderscore` — a latent byte-fidelity edge.** A lemma named
   literally `_foo` (or an empty proof-path segment) would render `lemma/_foo`/absent where
   upstream renders `lemma/__foo`/`_`. Unobservable in the corpus (the 40037-tail sweep is
   byte-identical; the `_` root marker round-trips as the literal), affirmatively non-copying,
   but a real divergence if `_`-prefixed identifiers ever occur. Non-blocking; if interop with
   upstream-generated `_`-segment URLs matters, probe a `_`-prefixed lemma name and re-add the
   prefix rule.
3. **`encode_segment` keeps `-` and `~` raw** though the corpus segment inventory is only
   `[A-Za-z0-9_.]` ([S14]) — an untraced generalization within the standard RFC3986 unreserved
   set (scenes-à-faire, same posture as the R4 extra-MIME note), honestly flagged UNOBSERVABLE
   in BEHAVIOR §11. Not copied protectable expression.

## VIOLATIONS (Producers Round 2)

**Similarity: 0.** Every R5/R2 resemblance to `Types.hs`/`Theory.hs` reduces to observable wire
output — the URL-token grammar (merger, pinned by [S14]/[L08]–[L13] and the 40037-tail sweep)
and the west-pane frame (byte-forced, pinned by the 478/478 sweep) — while the *expression* is
materially divergent: the numeric grammar reconstructs GHC `reads` behaviour from the [L10]
battery (upstream has no such code to copy), `prefixWithUnderscore`/`unprefixUnderscore` is
pointedly not reproduced, `ThyPath` is coarser than `TheoryPath` (omits `Method`), R2 holds the
nav vocabulary/formula/rules-label opaque where upstream computes them, and the layout is a
probe-bisected `≤69` escaped-width special-case over a flat `Vec<String>` rather than HughesPJ
`nest 2 (sep …)`. Identifier-constellation overlap: none. Comment lineage: none.

**Barrier hygiene: 0 — PH-1 resolved.** SPEC.md/README.md now cite only ported `.rs` seams +
usernames (sibling-SPEC norm); a full sealed-readable scan finds zero `src/Web/*.hs` /
`Text/PrettyPrint/Html.hs` paths, and the round-2 delta introduced none.

Three non-blocking notes recorded (shipped-comment provenance narration incl. two stale refs;
render-side `prefixWithUnderscore` omission; speculative `-`/`~` in the encode set) — none is
copied protectable expression and none requires redo. Findings that survive filtration: 0.

VERDICT: pass

# PRODUCERS cluster — Round 3 — both-sides similarity audit (R3 proof-tree/method HTML + R4 index/housekeeping)

Delta audited: the working-tree change on `weblayer/producers/` since `0359dd2` (HEAD `5f6ff68`
is a graphdot-only commit; the producers R3/R4 work is uncommitted). Sealed side: `prooftree.rs`
(new `render_tree`/`render_tree_lines`/`render_node`/`status_class`/`wrap_status`/`keyword`),
`welcome.rs` (new `render_welcome`/`render_invalid_args` + `ROBOTS_BODY`/`CANCEL_ACK_BODY`/
`FILE_NOT_FOUND_BODY` + six `include_str!` frame fragments), the `proofscript.rs`/`model.rs`/
`lib.rs` wiring, BEHAVIOR §16–§20, QUERIES [S19]–[S21]/[L16]–[L21], tests `r3_proof_tree.rs`/
`r4_welcome.rs`/`common/mod.rs` + the r2 sweep upgrade. Upstream compared: `Web/Theory.hs`
(`proofIndex` = `ppStep`/`ppCase`/`stepLink`/`superfluousStep`/`removeStep`/`invalidatedStep`;
`markStatus`; `linkToPath`), `Theory/Proof.hs` (`prettyProofWith`/`ppCases`/`ppCase`, the
by/case/next/qed traversal), `Web/Hamlet.hs` (`rootTpl`/`introTpl`/`theoriesTpl`/`theoryTpl`),
`Web/Handler.hs` (root POST flash, `robots`, `kill`, `invalidArgs`, static-miss).

## Abstraction — what each side does

R3 maps a pre-computed tree (SHAPE + opaque per-node method text + per-node status) to the
west-pane logical lines: per node a step line (optional `by ` prefix, an anchored `proof-step`
link OR a bare `hl_superfluous` span, optional `remove-step` anchor), then case framing (a
single unnamed continuation at the same indent; ≥1 named case at `d+2` with a `case NAME`
header, `next` between siblings, `qed` to close), with status→`<span class>` wrapping.
Upstream splits this across `Theory/Proof.hs::prettyProofWith`/`ppCases` (owns the by/case/
next/qed traversal, library-side) and `Web/Theory.hs::proofIndex` (`ppStep` colour case +
`ppCase = markStatus`). R4 emits the `/` page as a fixed frame around a flash slot, a version
slot and index-ascending theory rows, plus fixed housekeeping bodies; upstream builds the
same via whamlet widgets + `setMessage` + plain handlers.

## Provenance cross-check — the three hardest-to-guess behaviours trace to probes, not source

- **Indent law** (case at parent+2, subtree at case indent, `next`/`qed` at parent indent;
  unnamed continuation at same indent, segment `_`): BEHAVIOR §16 + QUERIES [S19] (corpus
  census over 478 panes) + the `fixture_two_case_good_tree` byte pin. Upstream expresses it as
  `nest 2`; the sealed re-derives the observable arithmetic. Logged, not source-lifted.
- **`next`/`qed` carry the PARENT status, the case line the CHILD status** (the non-obvious
  one — one would naïvely bind `next` to the following case): [S19] pins it corpus-wide
  (426612/426612 case==child) and explicitly refutes the prev-case (×117) and following-case
  (×99) alternatives; [L18] then FORCES the corpus-unreachable discriminator live with a mixed
  TreeProbe2 (bad parent, good following case → `next`/`qed` render `hl_bad`). Upstream
  `prettyCase ps kwNext`/`prettyCase (root prf)` matches, but the sealed reached it by a
  discriminating own-theory probe. Strong provenance.
- **Incomplete-subtree / replayed-leftover rendering** (a live `sorry /* invalid proof step
  encountered */` node — `sorry-step`, no `by` because it has a child, no remove-step — whose
  unnamed `_` child is the `hl_superfluous` leftover: span-not-anchor, keeps remove-step,
  paths continue through it): §17 matrix + [S20] (the single corpus instance, `d381650e`,
  dissected) + [L18] (reproduced 1:1 by doctoring an own theory's embedded proof script). The
  corpus had exactly one instance, so it was forced live — exactly the protocol posture.
- **Row ordering (R4)**: §19 + [L19] (index-ascending, 1-row and 3-version captures) + [L20]
  (upload appends an `<em>Modified` row). Upstream's `processMap`/`ntail 4` grouping+cap and
  the `Interactive -> "(interactively created)"` origin are correctly attributed to the ADAPTER
  (row SELECTION) and NOT reproduced in the producer — the right boundary, affirmatively drawn.

## Filtration — every shared element is byte-forced output (merger / compatibility)

- Class vocabulary `internal-link`/`proof-step`/`remove-step`/`sorry-step`/`hl_good`/`hl_bad`/
  `hl_medium`/`hl_superfluous`/`hl_keyword` and the keyword grammar `by`/`case`/`next`/`qed`:
  the served wire bytes, CSS-coupled — compatibility, and below even the SEMANTIC acceptance
  gate (which canonicalizes `hl_*` away), reproduced only for byte-fidelity to the reference.
- `by`-except-`SOLVED`: observable (612 `SOLVED` lines carry no `by` [S19]); the sealed models
  it as a `terminal_marker` bool the interface header lacks, DERIVED by the adapter from the
  step kind — no `Finished Solved` internal constructor is referenced.
- remove-step present iff the method is not `Sorry`: observable (595 sorry-METHOD steps lack
  it; 95 sorry-CLASS interiors keep it [S19]); modelled as `live`, not as `psMethod`.
- href path composition (`proof/{lemma}` + case segments, `_` for unnamed continuations):
  observable from hrefs, re-rendered through the round-1 escape + R5 encoder.
- R4 frame bytes (the doubled `</script></script>` closers, the UNCLOSED row `<em>`, the
  core-team block, licence text, logo path, contextMenu): captured OUTPUT of `rootTpl`/
  `introTpl`, byte-forced served page — reproducing served bytes is not copying template
  source. The doubled closer and unclosed tag are rendering artifacts a source-reader need not
  reproduce; that they are pinned byte-exact ([L19], `include_str!` fragments) is affirmative
  black-box evidence.
- Flash strings `Loaded new theory!` / `Post request failed.` and bodies `User-agent: *` /
  `Canceled request!` / `File not found` / `No path to kill specified!`: all observable via
  live routes ([L19]–[L21]); the load-error text is held opaque (`Banner::Custom`), and the
  kill message is a caller argument — none is a non-observable source constant.

## Comparison — affirmative evidence of independent, observation-only construction

- **Control structure differs.** The sealed `render_node` emits the step FIRST (with the by/
  terminal guard `cases.is_empty() && !terminal_marker` inline) then a 3-arm `match
  node.cases.as_slice()` (`[]` / `[(name,child)] if name.is_empty()` / `cases`). Upstream
  `ppCases` is a 4-clause pattern match (`Finished Solved`+`[]` / `[]` / `[("",prf)]` /
  `cases`) with step emission interleaved per clause, plus a separate 5-way colour `case` in
  `ppStep`. No clause-for-clause correspondence; the SOLVED/by split lives in different places.
- **Model differs.** The sealed collapses upstream's `(Maybe System, ProofStepColor)` pair into
  one 5-variant `Highlight` (`None`/`Good`/`Bad`/`Medium`/`Replayed`) and adds explicit `live`
  and `terminal_marker` fields the header interface does not carry (adapter-derived). Helper
  boundaries and names share nothing with upstream (`render_node`/`status_class`/`wrap_status`/
  `keyword` vs `ppCases`/`ppStep`/`markStatus`/`stepLink`/`superfluousStep`/`removeStep`).
- **Divergences that only an observer would ship.** The sealed does NOT reproduce upstream's
  `invalidatedStep` (the `Invalidated`→`hl_medium` step + the "verify it" `TheoryVerifyR` link)
  and honestly records `hl_medium` as an ASSUMED, never-observed class name (§18); the entire
  diff-proof path (`diffProofIndex`/`markStatusDiff`, equiv-kind hrefs) is absent (unobservable,
  §18); the R4 producer omits `processMap`/`ntail 4` and `"(interactively created)"` and does
  not reproduce upstream's `"Loaded new theory!" ++ warningMsg` concatenation (warnings routed
  through `Custom`). A copier carries these; an observer, lacking captures for them, does not.
- **Comment lineage: none.** Every doc comment cites the sealed side's own `[S..]`/`[L..]`/
  BEHAVIOR §-refs; no upstream file, function, or type name appears anywhere in the delta.

## Tests are live byte gates, not vacuous

`common/mod.rs::TreeInv` STRICTLY inverts each proof display into a `ProofTree`, asserting on
the way every frame byte, canonical step/remove href, `by`-wrapper==step-status, case==child /
next==qed==parent status, and the removability matrix — then `render_proof_script` re-renders
and byte-compares. The R2 sweep is upgraded to run this over all 478 panes; `r3_proof_tree.rs`
adds 4 own-theory live-pane replays (`r3_live/` present: proved/bounded/superfluous/mixed) + 5
constructed-input byte fixtures pinning each line form; `r4_welcome.rs` strictly inverts and
byte-replays 6 live index pages (`r4_live/` present, incl. metachar-escaping and multi-line
`Custom` flash) + housekeeping/Invalid-Arguments bytes. QUERIES records mutation checks
(`next`→following-case status; dropping the marker by-exception; closing the row `<em>`) each
observed to FAIL then reverted — the gate is not vacuous. Provenance artifacts (`probes/*.spthy`,
`r3_live/*.pane`, `r4_live/*`, `tools/analyze_prooftree.py`) all present on disk.

## Barrier hygiene — no sealed-readable file gained an upstream citation

A scan of the entire round-3 delta (`src/`, `tests/`, the six `*.html` frame fragments,
BEHAVIOR.md, QUERIES.log, `tools/analyze_prooftree.py`) for `src/Web`, `Theory.hs`/`Hamlet.hs`/
`Handler.hs`, `Text/PrettyPrint`, `whamlet`/`Yesod`/`Blaze`, and the Haskell identifier set
(`prettyProofWith`/`ppCases`/`ppStep`/`markStatus`/`psInfo`/`psMethod`/`ProofStepColor`/
`Finished Solved`/`Invalidated`/`kwBy`/`Unmarked`/`introTpl`/`theoriesTpl`/`theoryTpl`) returns
**zero**; the clean crate carries no copyright/SPDX/upstream citation. Additionally, the two
stale refs flagged non-blocking in Producers-R2 note #1 (`proofscript.rs` "473"→478 count and
"§11"→§12 pointer) were **corrected this round** (QUERIES "Housekeeping fix").

## Non-blocking notes (NOT similarity findings; no redo; do not bear on the verdict)

1. **Shipped-comment provenance narration continues.** `prooftree.rs`/`welcome.rs`/`model.rs`
   doc comments carry inline `[S..]`/`[L..]`/`§`-refs — the clean-room's own provenance, the
   *opposite* of copied expression, so no verdict effect; it continues the standing
   Round-6/7/Producers-R1/R2 hygiene note against the "comments describe current state only"
   standard. (The two stale numeric refs from R2 are now fixed; no new stale refs introduced.)
2. **`hl_medium` / `invalidatedStep` / diff-proof are latent fidelity edges, not copies.** An
   `Invalidated` proof step renders, upstream, as an `hl_medium` link plus a "verify it"
   affordance; the sealed emits a plain `hl_medium` link (assumed name) and no "verify it", and
   renders equiv-kind hrefs with the `trace` kind only. All three states are unobserved in
   corpus + probes (§18) and their omission is affirmative non-copying — but if an interactive
   `Invalidated`/`Unfinishable`/diff state ever reaches the producer, byte/behaviour parity
   would differ. If those states matter at integration, probe them and extend the renderer.

## VIOLATIONS (Producers Round 3)

**Similarity: 0.** Every R3/R4 resemblance to `Theory.hs`/`Proof.hs`/`Hamlet.hs`/`Handler.hs`
reduces to observable wire output — the proof-line grammar (indent, by/case/next/qed, status
spans, step+remove links, canonical hrefs; pinned by the 478/478 sweep, [S19]–[S21], and the
[L16]–[L18] live probes) and the index/housekeeping bytes (byte-forced served page, pinned by
6/6 live replays + [L19]–[L21]) — while the EXPRESSION is materially divergent: a step-first
3-arm traversal vs the 4-clause `ppCases`+5-way `ppStep`, a collapsed 5-variant `Highlight`
with adapter-derived `live`/`terminal_marker` vs the `(Maybe System, ProofStepColor)` pair, no
shared helper boundary or name, and pointed omission of `invalidatedStep`/"verify it"/`hl_medium`
/diff-proof/`processMap`+`ntail 4`/`"(interactively created)"`. Identifier-constellation
overlap: none. Comment lineage: none.

**Barrier hygiene: 0.** The full round-3-delta scan finds zero upstream source-file paths or
Haskell identifiers; the two Producers-R2 stale refs were corrected. No sealed-readable file
gained an upstream citation this round.

Two non-blocking notes recorded (continued own-provenance comment narration; latent
`hl_medium`/`invalidatedStep`/diff-proof fidelity edges) — neither is copied protectable
expression and neither requires redo. Findings that survive filtration: 0.

VERDICT: pass
