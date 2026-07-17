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
