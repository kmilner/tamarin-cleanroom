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
