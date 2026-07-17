# BEHAVIOR.md — observed spec of the tamarin-prover interactive web UI

Derived entirely from black-box observation: 81 crawl manifests in
`oracle/captured_responses/` (captured OUTPUT) plus live probing of the
sanctioned server (`QUERIES.log` [L0]–[L4]). No prover source was read.

Terms: **web layer** = the code that dispatches routes, escapes, and wraps
strings into envelopes/pages. **prover fragment** = a string the prover produced
(pretty-printed terms, constraint systems, method names, DOT bodies) that the
web layer embeds opaquely. This spec pins down the web layer; prover fragments
are treated as inputs.

---

## 1. Route grammar

Every dynamic route has the shape:

```
/thy/<theory-kind>/<index>/<handler>/<args…>
```

* `<theory-kind>` — analysis kind. Only `trace` observed (a `diff` kind is
  plausible but never appears in the corpus).
* `<index>` — theory **version** selector. In real requests it is a decimal
  number. The manifests store it as the literal token `#`; live probing shows
  `%23` returns **404**, so `#` is the crawl tool's placeholder meaning "the
  server's current version", and every URL the server *emits* uses the resolved
  numeric index. Applying a proof method creates a new version, so the index
  seen in a response's links is generally larger than the one requested.
* `<handler>` — selects the response family:

| handler                     | body kind | notes |
|-----------------------------|-----------|-------|
| `main/…`                    | JSON      | AJAX content pane; envelope `{html,title}` or `{redirect}` |
| `overview/…`                | HTML      | full theory-view page (see §5) |
| `intdot/…`                  | HTML      | graph mini-page embedding `<dot-graph-viz>` (§8) |
| `interactive-graph-def/…`   | DOT       | Graphviz source for a proof node (§8) |
| `next/…`, `prev/…`          | text      | a single navigation URL (§9) |
| `autoprove/…`               | JSON      | `{redirect}`; **text** with HTTP status 0 on timeout |
| `source`, `message`         | text      | the theory source, verbatim (§9) |

`main/*` sub-handlers (observed, exhaustive over the corpus):

| sub-route | meaning |
|-----------|---------|
| `main/help` | theory header + wellformedness report + keyboard help |
| `main/message` | signature + construction/deconstruction rules |
| `main/rules` | multiset rewriting rules (+ restrictions) |
| `main/tactic` | tactic(s) listing |
| `main/cases/{raw\|refined}/{level}/{n}` | source cases |
| `main/lemma/{name}` | lemma summary |
| `main/proof/{lemma}/{path…}` | constraint system + applicable methods at a proof node; `path` is a `/`-joined sequence of case-name segments, root `_` |
| `main/method/{lemma}/{n}` | apply method `n` → `{redirect}` |
| `main/add/{pos}` | add-lemma form (`pos` = a lemma name or `<first>` ⇒ `%3Cfirst%3E`) |
| `main/edit/{name}` | edit-lemma form |
| `main/delete/{name}` | delete-lemma confirmation |

`overview/*` mirrors `main/*` for the pages that render a full shell:
`overview/help`, `overview/proof/{lemma}/{path…}`. `intdot/*` and
`interactive-graph-def/*` take the same `proof/{lemma}/{path…}` (and
`cases/…`) tails.

### Authoritative handler/kind census (all 81 manifests, 48824 dynamic routes)

Enumerated with a template pass ([Q020]–[Q021]); the response `kind` is fixed
per handler:

| handler                  | kind        | count | argument shape |
|--------------------------|-------------|-------|----------------|
| `main`                   | json        | 17609 | see `main/*` table above |
| `interactive-graph-def`  | dot         | 14705 | `proof/{lemma}[/{path…}]` |
| `intdot`                 | html        | 14705 | `proof/{lemma}[/{path…}]` |
| `overview`               | html        | 473   | `help` \| `proof/{lemma}[/{path…}]` |
| `next`                   | text        | 392   | `normal/proof/{lemma}` |
| `prev`                   | text        | 392   | `normal/proof/{lemma}` |
| `autoprove`              | json (text on timeout) | 392 | `idfs/{bound}/False/proof/{lemma}` |
| `source`                 | text        | 81    | (none) |
| `message`                | text        | 81    | (none) |

`autoprove/{strategy}/{bound}/{allSol}/proof/{lemma}[/path…]` is the full
autoprove shape (see §13.2). It answers with a `{redirect}` to the resolved
`overview/proof/{lemma}` at a **new** version index — live probe [L6]/[L9] show
`autoprove/idfs/0/False/proof/types` on a version-1 theory returning
`{"redirect":"/thy/trace/2/overview/proof/nonce_secrecy"}`. The 6 corpus
"status 0" entries carry the body `REQUEST_ERROR: TimeoutError('timed out')`,
which is the **crawler's own** timeout string (the client gave up while the
prover ran), not a web-layer response family ([Q035] — corrects the earlier note).

`next`/`prev` are pure text: a single navigation URL. The target need not be a
proof node — live probe shows `prev/normal/proof/types` (the first lemma)
returning `/thy/trace/1/main/cases/refined/0/0`.

Content-types (live probe [L6]): json = `application/json; charset=utf-8`,
text = `text/plain; charset=utf-8`, html = `text/html; charset=utf-8`.

Static/other routes seen in emitted links (not in the crawl of dynamic content):
`GET /` (index), `POST /` (upload), `POST /thy/…/reload`,
`GET /thy/…/download/{file}`, `POST /thy/…/get_and_append/{file}`,
`GET /thy/…/edit/{edit|add|delete}/{name}` (form POST targets),
`/static/**`, `/static/LICENSE`, `/static/img/tamarin-logo-3-0-0.png`.

Unmatched paths → **404** full HTML page (§7).

---

## 2. Response envelopes

### JSON (`main/*`, `method/*`, `autoprove/*`)
Exactly two compact shapes, keys in this order, **no** insignificant whitespace,
**no** trailing newline:

```
{"html":<string>,"title":<string>}     — content pane
{"redirect":<string>}                  — client should navigate to the URL
```

JSON string escaping is standard: only `"` `\` and control chars are escaped;
`/ < > &` are **not** escaped; non-ASCII is emitted as **literal UTF-8** (no
`\uXXXX`). `serde_json`'s default output reproduces this byte-for-byte
(verified on 2450 distinct bodies).

### HTML (`overview/*`, `intdot/*`, `/`, 404)
`text/html; charset=utf-8`. Full documents beginning `<!DOCTYPE html>\n<html>…`
and ending `</html>` with **no** trailing newline.

### text (`source`, `message`, `next`, `prev`)
`text/plain`. Returned verbatim; `source`/`message` end at the theory's final
`end` with no trailing newline; `next`/`prev` are a bare URL path.

### DOT (`interactive-graph-def/*`)
Graphviz source; the empty-graph skeleton ends with `}\n`.

---

## 3. Escaping rules

The web layer's HTML text escape maps five characters and passes everything else
(including all prover-emitted unicode operators) through unchanged:

```
&  ->  &amp;
<  ->  &lt;
>  ->  &gt;
"  ->  &quot;
'  ->  &#39;
```

Applied to: the "loaded from" path, lemma names/text in the edit form, the add
form's position marker (`<first>` → `&lt;first&gt;` in the form action), the
echoed path in the 404 page. Each mapping was witnessed in a capture. Prover
fragments arrive already escaped (e.g. pair terms show `&lt; … &gt;`), so the
web layer does not double-escape them.

URL segments are percent-encoded independently: `<first>` appears as
`%3Cfirst%3E` in hrefs.

---

## 4. What is web-layer vs prover-produced

Web layer (reproduced by `web-clean`): route dispatch; the JSON envelopes; the
page shell / index / 404 / intdot templates; the proof-script pane scaffolding
(theory header link, item links, add/edit/delete links, `by sorry`, blank-line
spacing, `end`); the proof-tree **line** markup (indentation, anchors,
`case`/`next`/`qed`/`by` keywords); the form bodies; HTML escaping.

Prover fragments (supplied as opaque inputs): pretty-printed signatures, rules,
lemma formulas, constraint systems, applicable-method names, source cases,
proof-method texts, non-empty DOT graphs, the theory source string, wf-warning
text, the rules label/count and sources descriptions.

---

## 5. Full theory-view page (`overview/*`)

Fully determined by six values: `NAME`, `IDX` (numeric), `VERSION`, `FILENAME`,
the west-pane inner HTML, and the center-pane inner HTML. After substituting
`IDX`/`NAME`/`FILENAME`/`VERSION`, the scaffolding is byte-identical across
theories. Layout:

```
<!DOCTYPE html>\n<html><head><title>Theory: NAME</title> …fixed link/script set… </head>
<body><p class="loading">Analyzing, please wait…  <a id=cancel href='#'>Cancel</a></p>
  <div class="ui-layout-north"> …header: "Running Tamarin VERSION", nav (Index,
      Reload, Actions[source/download FILENAME/append], Options[toggles])… </div>
  <div class="ui-layout-west"><h1 class="pane-head">Proof scripts</h1>
      …<div class="monospace" id="proof">WEST_INNER</div>… </div>
  <div class="ui-layout-east"> …"Debug information", always empty… </div>
  <div class="ui-layout-center"><h1 …>Visualization display</h1>
      …<div id="ui-main-display">CENTER_INNER</div>… </div>
  <div id="dialog"></div><div id="confirm-dialog"></div>
  <ul id="contextMenu"><li class="autoprove"><a href="#autoprove">Autoprove</a></a></li></ul>
</body></html>
```

`CENTER_INNER` is the currently-selected content — essentially the same HTML the
matching `main/*` route returns in its `html` field (modulo a trailing-whitespace
difference on the initial help view). Every internal link uses `IDX`.

The exact scaffolding is stored byte-for-byte in `src/shell_template.rs`
(`PAGE_PREFIX`/`PAGE_MID`/`PAGE_TAIL`, §-delimited slots).

---

## 6. Proof-script (west) pane

A flat sequence of logical lines; render emits each as `TEXT + "<br/>\n"`, then a
final single space `" "`. Blank lines are empty `TEXT`. Element order:

1. `theory NAME begin` header line (NAME links to `main/help`).
2. For each theory item, a blank line then a link line, in order: **Message
   theory**, **rules** (`<label> (<count>)`; label is `Multiset rewriting rules`,
   optionally `… and restrictions`), **Tactic(s)**, **Raw sources** (`(<desc>)`),
   **Refined sources** (label carries a trailing space).
3. Blank + the `add lemma` link for position `<first>` (`add/%3Cfirst%3E`).
4. Per lemma: blank, the lemma body lines, blank, the lemma's trailing `add lemma`
   link (`add/<name>`).
5. Blank + `end`.

With **zero** lemmas, step 4 is empty, leaving **two** blank lines before `end`
(exactly as captured). Lemma body (unproven state):

```
<decl_html>                      # prover: "lemma NAME:" + quantifier + formula
<edit lemma link>  or  <delete lemma link>
<span class="hl_keyword">by</span> <a class="internal-link proof-step sorry-step"
      href="/thy/trace/IDX/main/proof/NAME"><span class="hl_keyword">sorry</span></a>
```

**Proved state** — when a lemma carries a proof (not `sorry`), the whole lemma
**header** (the declaration *and* its edit/delete line) is wrapped in one status
span reflecting the lemma's overall proof status; the wrapper opens right before
`<decl_html>` and closes right after the delete anchor, spanning the intervening
`<br/>` breaks. The proof-tree lines follow, unwrapped:

```
<span class="STATUS"><decl_html>          # STATUS = hl_good for the proved corpus
<edit lemma link>  or  <delete lemma link></span>
<proof-tree line 0>
<proof-tree line 1>
…
```

So the *only* rendering difference between an unproven and a proven lemma header
is (a) the `<span class="STATUS">…</span>` wrapper and (b) the trailing lines
being a proof tree instead of the single `by sorry` step. This was reproduced
byte-for-byte for both Chaum lemmas at version 3 ([Q022]–[Q023]).

### Proof-tree line grammar (solved/partial proofs)
Indentation is `&nbsp;&nbsp;` repeated `depth` times. Status class is `hl_good`
(proved / on a found trace) or `sorry-step` (open); `hl_bad`/`hl_dead` are
plausible for falsified/dead but unobserved.

```
step : {indent}[BY]<a class="internal-link proof-step {status}" href="{href}">{method_html}</a>
                    <a class="internal-link remove-step" href="{href}">{annotation}</a>
       where BY = <span class="{status}"><span class="hl_keyword">by</span> </span>  (only for a final "by" step)
case : {indent}<span class="{status}"><span class="hl_keyword">case</span> {name}</span>
next : {indent}<span class="{status}"><span class="hl_keyword">next</span></span>
qed  : {indent}<span class="{status}"><span class="hl_keyword">qed</span></span>
```

`method_html` (e.g. `simplify`, `solve( … )`, `SOLVED // trace found`,
`contradiction /* cyclic */`) and `name` come from the prover. The tree shape
(depths, case/next/qed placement) follows the prover's proof tree.

---

## 7. Index page (`/`) and 404

**Index** (`GET /`, 200): `<title>Welcome to the Tamarin prover</title>`, the
same head set, a north header, the Tamarin logo, a static credits/warranty
block, a table of loaded theories, and an upload form. Theory rows:

```
<tr><td><a href="/thy/trace/IDX/overview/help">NAME</a></td>
    <td>TIME</td><td>VERSION</td><td>ORIGIN</td></tr>
```

`TIME` (load time) and `ORIGIN` (temp source path) are non-deterministic; the
rest is fixed. Not byte-tested for that reason; structure documented.

**404** (unmatched route): full HTML page, `<title>Not Found</title>`, the same
head/tail as the overview page, body = loading bar + `<h1>Not Found</h1>\n<p>{ECHOED_PATH}</p>\n`.
The echoed path is HTML-escaped. Byte-exact template in `src/notfound_template.rs`.

---

## 8. Graph routes

`intdot/{tail}` (200 HTML): a standalone mini-page whose only content is
`<dot-graph-viz dotsrc="/thy/trace/IDX/interactive-graph-def/{tail}"></dot-graph-viz>`
(the `intdot` handler is swapped for `interactive-graph-def`, same tail). Ends
`</html>`, no trailing newline.

`interactive-graph-def/{tail}` (200 DOT): Graphviz. For a proof node with no
constructed graph, the fixed empty skeleton:

```
digraph "G" {
nodesep="0.3";
ranksep="0.3";
node[fontsize="8",fontname="Helvetica",width="0.3",height="0.2"];
edge[fontsize="8",fontname="Helvetica"];

}
```

Non-empty graphs are prover-produced (out of scope for this web-layer spec).

---

## 9. Forms and text bodies

**Edit / delete / add forms** are near-static templates (`html` field of the
`main/edit|delete|add` envelope). Slots: the lemma name (echoed in the label /
prompt / add link) and the form `action` (`../../edit/{verb}/{name}`, with
`{name}` HTML-escaped). The edit textarea holds the raw lemma source,
HTML-escaped, and uses `rows="8"`. Titles: `Edit Lemma: {name}`, `Delete {name}`,
`Add new Lemma`.

**`source`/`message`** return the theory source verbatim (identical to each
other). **`next`/`prev`** return one navigation URL, e.g.
`/thy/trace/IDX/main/proof/unforgeability`.

---

## 10. State-dependence & non-determinism

* **Theory version index** — every emitted link carries the current numeric
  version; changes as proofs are applied.
* **Load timestamp** (`Loaded at HH:MM:SS`), **temp source path**
  (`/tmp/tmp.XX’…`), and the source footer (`Compiled at …`, git revision,
  Maude version) — non-deterministic; appear in `help`/`message`/`source`/index.
* **wf-warning block**, **proof status colors**, **applicable-method ordering** —
  depend on the theory and proof state (prover fragments).

`web-clean` render functions take the version index, version string, filename,
timestamps, and all such prover fragments as explicit inputs, so a caller that
supplies the observed values reproduces the observed bytes.

---

## 11. Coverage map (BEHAVIOR → crate → tests)

| behavior | module | byte-parity test |
|----------|--------|------------------|
| route grammar | `route` | unit tests; census in §1 |
| JSON envelopes | `envelope` | 2450 distinct bodies + 2 whole manifests |
| HTML escaping | `escape` | unit + via forms/404 |
| page shell (help view) | `page` + `shell_template` | 2 full pages (2 theories) |
| page shell (proof view) | `page` + `shell_template` | 1 full page (`overview/proof/exec`, idx 3) |
| proof-script pane (unproven) | `proofscript` | west pane, 0-lemma + 2-lemma |
| proof-script pane (solved tree) | `proofscript` | full 2-lemma proved west pane (40 proof lines) |
| proof-tree lines | `proofscript` | unit line-grammar + the solved-tree parity test |
| edit/delete/add forms | `forms` | 4 envelopes |
| intdot / empty DOT | `intdot` | 2 bodies (+ live byte-cmp on a 2nd theory) |
| 404 page | `errors` + `notfound_template` | 1 body |
| source/next text | `text` | pass-through |

37 tests total (19 unit + 18 parity), all byte-parity where the captures are
deterministic. The solved-tree test drives the crate's proof-line model
(`proof_lines_{exec,unforgeability}.json`, my observation model) plus the
crate's URL builder and asserts the full west pane byte-for-byte; only the
proof-method HTML and case names are treated as opaque prover fragments.

Gaps are listed in the final report; the largest is that solved-proof
constraint-system HTML (the `main/proof` / center-pane `<h3>Applicable Proof
Methods…` body) and non-empty DOT graphs are prover fragments, so their *inner*
content is not reproduced here (only the surrounding web-layer markup).

---

## 12. Whole-corpus `html` generality (continuation round; see REPORT2.md)

The captured `html` surface is **exactly two template families**, both closed
byte-for-byte over ALL 81 manifests via `examples/corpus_html.rs` (bulk harness):
**15178 / 15178 = 100.00%**.

| family | bodies | byte-parity |
|--------|-------:|:-----------:|
| `intdot/*` | 14705 | 100.00% (fully model-driven: `render_intdot`) |
| `overview/help` | 81 | 100.00% (shell `render_page`, panes opaque) |
| `overview/proof/*` | 392 | 100.00% (shell `render_page`, panes opaque) |

No other html handler and no `overview/other` subfamily exists in the corpus, so
there are no uncovered html families (proof methods are JSON `main/method`, the
source view is `text/plain`, no `diff` kind appears).

Key observations that made this exact:
* **`intdot` transform**: body dotsrc = `/thy/trace/{IDX}/interactive-graph-def/{TAIL}`
  where `{TAIL}` is a byte-exact passthrough of the request URL's `/intdot/`
  tail (verified over 14705: 0 tail-guard failures) and `{NAME}` is the theory
  name from the sibling `overview/help` title (0 name-guard failures). `{IDX}`
  is the theory-version index — the ONE value the capture erased (URL keys are
  normalised to `#`); it genuinely varies per node (a single manifest can emit
  both `1` and `16`, from nodes crawled before/after autoprove advanced the
  version), so it is a request/state parameter, not a web-layer choice.
* **`overview` decomposition**: every body = `PAGE_PREFIX(name,idx,ver,file)` +
  WEST + `PAGE_MID` + CENTER + `PAGE_TAIL`. The shell scaffolding is byte-exact
  across 71 theories / 4 indices / 81 filenames (version is `1.13.0` in all 81).
* **CENTER = sibling `main/*` html + one trailing space** (ties the center pane
  to the 100%-parity JSON envelope family). Holds `help` 81/81 (masking the
  non-deterministic `Loaded at HH:MM:SS`) and `proof` 309/392 exact; the 83
  `proof` residuals are cross-version capture skew (main captured at an earlier
  version than the overview page), differing only in link indices + proof-state
  content, both prover fragments. Center pane ends in `" "` for 473/473.
* **WEST** (proof-script pane) is web-layer output, constant per (theory,
  version); it is byte-reproduced from a parsed model only for Chaum so far, and
  treated as an opaque per-theory fragment for the other 70 — the single
  remaining modeling gap.

Permanent regression: `html_page_generality_sample_byte_identical`
(`tests/fixtures/html_sample.ndjson`, 20 bodies across 6 theories, indices
1/3/8/16, both families) locks generality into `cargo test` (38 tests at round 2;
**55** after the round-3 `dispatch` suite — see §13).

---

## 13. Handler semantics — the UI state machine (Round 3)

Derived from live probing of the sanctioned oracle ([L7]–[L16], Tutorial.spthy,
ports 3137-3141) plus corpus route/link census ([Q029]–[Q035]). This section
pins down *what each route returns and how theory-version state evolves*; the
per-body bytes are §§1–12. Reproduced in `src/dispatch.rs` over a `ProverOps`
callback trait (the prover supplies fragments/mutations; the web layer decides
version allocation, route dispatch, and the response envelope).

### 13.1 Version model (spec item 1)
* **Version 1** = the theory as loaded ("Original"); higher indices are
  "Modified". Every version stays **resolvable by index** forever, even after it
  scrolls out of the index-page table.
* **Proof operations** — `main/method/{lemma}/{n}[/path…]` and `autoprove/…` —
  allocate a **fresh** index `= (max ever allocated) + 1` (monotonic global
  counter, independent of the base index), leaving the base untouched ([L8],[L9]).
* **Structural edits** — POST `edit/{edit,add,delete}/{name}` — mutate the theory
  **in place at the same index** (no new version) ([L12]).
* **Navigation / views** (`overview`, `main/*` reads, `next`, `prev`, `source`,
  `intdot`, `interactive-graph-def`) never change the version set ([L10]).
* The index page lists the Original row plus a **capped window** of the most
  recent Modified rows (= the manifests' `capped` flag); dropped rows still
  resolve ([L11]). Row `Time`/`Origin` are non-deterministic (§7).

### 13.2 autoprove variants (spec item 2)
Route: `autoprove/{strategy}/{bound}/{allSol}/proof/{lemma}[/path…]`.
* `strategy` ∈ {`idfs` (solve/prove), `characterize` (characterization — e.g.
  exists-trace / observational goals)} ([Q030]).
* `bound` — a numeric depth bound; observed `0` (unbounded) and `5` (bounded).
* `allSol` ∈ {`False`, `True`}.
Keyboard-help mapping ([Q031], the "a/b/all/characterization" matrix):

| key | meaning | route projection |
|-----|---------|------------------|
| `a` | autoprove focused step, stop after first solution | `idfs/0/False` |
| `A` | …search for **all** solutions | `idfs/0/True` |
| `b` | **bounded**-depth autoprove, stop at first | `idfs/5/False` |
| `B` | bounded, all solutions | `idfs/5/True` |
| `s`/`S` | autoprove **all** lemmas (stop / all) | idfs, per-lemma |
| — | characterization | `characterize/{0,5}/{False,True}` |

So "all" = the `True` (all-solutions) flag and "characterization" = the
`characterize` strategy. **Response:** HTTP `200` + JSON
`{"redirect":"/thy/trace/{new}/overview/proof/{lemma}/{focus}"}`, `{new}` the
freshly allocated index and `{focus}` the prover's resulting focus path ([L9]).

### 13.3 Proof-step application, del, add (spec item 3)
* **Apply method** `GET main/method/{lemma}/{n}[/path…]` — the method number
  precedes the case-name path ([Q032]). Response = `200` + JSON
  `{"redirect":"/thy/trace/{new}/overview/proof/{lemma}/{focus}"}` ([L8]).
* **Delete lemma** `POST edit/delete/{name}` — `303 See Other`, `Location:
  /thy/trace/{v}/overview/help`, empty body, in place ([L12]).
* **Edit lemma** `POST edit/edit/{name}` (form field `lemma-text`) — on success
  `303` → `overview/edit/{name}`; on parse/wf failure `200` re-rendering the
  full-page **edit form** (theory unchanged) ([L12],[L13]).
* **Add lemma** `POST edit/add/{pos}` (`lemma-text`) — on success `303` →
  `overview/add/{pos}`; failure `200` add-form page. `{pos}` is a lemma name or
  `%3Cfirst%3E` ([L12]). These `overview/edit|add/…` full pages appear ONLY as
  POST-redirect targets (not in the crawl).

### 13.4 Proof-tree traversal & path encoding (spec item 1)
* `GET next|prev/{mode}/proof/{lemma}` → `200` `text/plain`, a **bare URL** at the
  **same** version. Target is the prover's `nextThyPath`-style computation: an
  adjacent proof node (`main/proof/{lemma}[/path]`) or a non-proof node
  (`main/cases/refined/0/0` before the first lemma); the last lemma's `next` is
  itself ([L10]). `mode` (`normal` in the corpus; `smart`/others accepted live)
  is passed opaquely to the prover.
* **Proof path** = raw `/`-join of case-name segments, root marker `_`. Segments
  are prover identifiers `[A-Za-z0-9_]` (up to ~112 chars observed); **no
  percent-encoding** anywhere in proof paths ([Q034]). The redirect focus after a
  proof op already includes the leading `_`.

### 13.5 Integration blocker (a): non-local page shell — NEGATIVE RESULT
Bound the oracle to all IPv4 interfaces (`--interface=*4`, `=` form; the space
form hits the workdir-positional bug) and fetched every route BOTH via loopback
`127.0.0.1` and via the machine's non-loopback `212.100.173.110` (the server LOG
confirms the peer address is `212.100.173.110`). Result: **byte-identical** on
every route — read views, forms, index `/`, 404, and mutating GET/POST (method,
edit, delete, reload all succeed and match). Host header
(`127.0.0.1`/`localhost`/`212.x`/`evil.com`) and `X-Forwarded-For`/`X-Real-IP`/
`X-Forwarded-Host` have **no** effect ([L15]). => In oracle **1.13.0** the served
shell is **origin-independent**. The locality predicate evidently treats all of
the machine's own interface addresses (loopback + `212.x`) as local, so a foreign
peer cannot be synthesised on a single host; the non-local shell's bytes are
**unobservable here**. The renderer is therefore parameterised over an
`origin_local` hook (see §13.7 / REPORT3) for the ported prover to drive, with
the exact non-local bytes left as a documented gap pending a multi-host capture.

### 13.6 Integration blocker (b): edit-form textarea `rows=` — SOLVED
`rows = (count of '\n' in the raw lemma source) + 2`. Verified over 4 lemmas
(newlines 9/11/7/10 → rows 11/13/9/12) ([L14]); HTML-escaping preserves newline
count, so it is computed on the raw text. Implemented as `forms::edit_rows`,
replacing the round-1 hardcoded `rows="8"` (a single-capture constant flagged by
the similarity audit). The **add** form's textarea has no `rows` attribute (fixed
placeholder `Enter your new Lemma`).

### 13.7 Content-types & the ProverOps boundary
Content-types ([L6]/verified [L16]): JSON `application/json; charset=utf-8`; HTML
`text/html; charset=utf-8`; text, `next`/`prev`, and **DOT**
`text/plain; charset=utf-8`. Proof ops answer `200` (JSON body), structural POSTs
`303` (Location header). `src/dispatch.rs` implements the `Server<T: ProverOps>`
state machine: it owns the version map + monotonic counter and makes all of the
above decisions; `ProverOps` supplies parse/edit/add/delete, apply-method,
autoprove, next/prev target, and the opaque pretty-printed fragments (west pane,
center content, lemma source, source text, DOT).

---

## 14. Full route surface — top-level + reload/download/kill/upload + diff (Round 4)

Derived from live probing of the sanctioned oracle ([R40]–[R4B], ports
3140-3143, Tutorial/NSLPK3/KCL diff theory) plus a corpus key census. This
section extends the dispatcher from the interactive read/proof handlers (§13) to
the **entire** request path. Reproduced in `src/dispatch.rs` over the same
`ProverOps` boundary (grown minimally) with one global version-index namespace.

### 14.1 Top-level (non-`/thy`) routes
`src/route.rs::Toplevel` splits the surface into `/` · `/robots.txt` ·
`/favicon.ico` · `/kill` · `/static/**` · `/thy/…` · other.

| route | method | response |
|-------|--------|----------|
| `GET /` | GET | `200` `text/html` index page (§7); no flash paragraph |
| `POST /` | POST | `200` `text/html` index page; success flash `<p class="message">Loaded new theory!</p>`, failure `…Post request failed.</p>` |
| `GET /robots.txt` | GET | `200` `text/plain` body `User-agent: *` (13 bytes, no NL) |
| `GET /favicon.ico` | GET | `303` → `/static/img/favicon.ico`, no-cache (see 14.4) |
| `GET /kill?path=…` | GET | `200` `text/plain` `Canceled request!` (17 bytes, no NL); cancels a running search, server stays up |
| `GET /kill` (no `path`) | GET | `400` `text/html` Invalid-Arguments page, body `<ul><li>No path to kill specified!</li>\n</ul>\n` |
| `GET /static/<path>` | GET | `200`, content-type by extension (14.5), body = file bytes; missing → `404` `text/plain` `File not found` |

Wrong method on any of the above → `405` `text/html` **Method Not Supported**
page (`<p>Method <code>M</code> not supported</p>`). These `405`/`400` pages and
the `404` page share the standard head + tail (`src/errors.rs`, from
`shell_template::SIMPLE_*`) and differ only in `<title>`/`<h1>`/body — the same
Yesod default-layout error family.

### 14.2 Theory-scoped additions
| handler | method | response |
|---------|--------|----------|
| `download/{file}` | GET | `200` `application/octet-stream`, **no** `Content-Disposition`; body == the `source` body verbatim (the `{file}` segment is decorative) |
| `reload` | POST | `200` JSON `{"redirect":"/thy/<kind>/<idx>/overview/help"}`; re-reads the theory **in place** at the same index |
| `get_and_append/{file}` | POST | `200` JSON `{"alert":"Appended lemmas to <path>"}` (the third envelope shape; path non-deterministic) |
| `edit/{verb}/{name}` | POST | structural edit (§13.3 / 14.6) |

`source` and `message` are GET-only; a POST to them (or to `download`) is `405`.

### 14.3 Version lifecycle — one global index namespace (spec item: state)
Every theory-version lives at a distinct index in **one** monotonically growing
namespace (`= max ever allocated + 1`), regardless of how it was produced:
* **Proof ops** (`method`, `diffMethod`, `autoprove*`) — new index, base retained.
* **Upload** (`POST /`) — new index off the **same** counter (NSLPK3 → 5 when
  1..4 existed). The response is the index page (200), **not** a redirect.
* **Structural edits** and **`reload`** — mutate in place at the same index; the
  counter is untouched and other versions stay resolvable (a proof op after a
  reload of v1 still got index 3 when 1,2 existed). Reload does **not** reset the
  counter or drop modified versions.
* Every allocated index stays resolvable forever (index-page window is a display
  cap only).

### 14.4 Redirect caching headers
All `303 See Other` responses — `edit`/`add`/`delete` success, delete-not-found,
and the favicon redirect — carry `Cache-Control: no-cache, must-revalidate` and
`Expires: Thu, 01 Jan 1970 05:05:05 GMT` (modelled by `Response.no_cache`). The
JSON `{redirect}`/`{alert}` bodies (`method`, `autoprove`, `reload`,
`get_and_append`) are ordinary `200`s with no cache headers.

### 14.5 Static content types
`/static/**` is a filesystem handler (chunked, **no** caching headers). Content
type is chosen by the last path segment's extension, **without** a `charset`:
`.css`→`text/css`, `.js`→`application/javascript`, `.png`→`image/png`,
`.ico`→`image/vnd.microsoft.icon`, no/unknown extension → `application/octet-stream`
(e.g. `/static/LICENSE`). A missing file yields the plain-text `File not found`
(not the dynamic HTML 404). `src/assets.rs::static_content_type`.

### 14.6 Structural-edit branches (refines §13.3)
* **delete** `POST edit/delete/{name}`: lemma **found** → `303` →
  `overview/help` (removed in place); **not found** → `303` →
  `overview/delete/{name}` (theory unchanged). `delete_lemma` therefore returns
  `Option` (`None` = not found).
* **edit** `POST edit/edit/{name}`: success → `303` → `overview/edit/{name}`;
  parse failure **or unknown lemma** → `200` full-page edit form (theory
  unchanged).
* **add** `POST edit/add/{pos}`: success → `303` → `overview/add/{pos}`; failure
  → `200` add-form page.
* **method/diffMethod** failure: `200` JSON `{"alert":"Sorry, but the prover
  failed on the selected method!"}`, **no** version bump (`apply_method` /
  `apply_diff_method` return `Option`).

### 14.7 Diff (observational-equivalence) mode — theory-kind `equiv`
Started with `--diff`; the index links use `/thy/equiv/…`. The overview shell is
the trace shell parameterised by `page::ShellKind::Equiv`: `<title>DiffTheory:
NAME`, `/thy/equiv/` links, and **no** Actions "Append modified lemmas" item
(MID/TAIL byte-identical to trace; reproduced byte-for-byte,
`equiv_overview_shell_byte_identical`). New handlers:
* `main/diffProof/{lemma}[/path…]` — JSON content pane (diff proof view).
* `main/diffMethod/{lemma}/{n}[/path…]` — proof op → `200` JSON
  `{"redirect":"/thy/equiv/<new>/overview/diffProof/{lemma}/{focus}"}` (new
  version) or the `{alert}` on failure.
* `main/diffrules` — JSON content pane.
* `autoproveDiff/{strategy}/{bound}/diffProof/{lemma}[/side…]` — **no** all-sol
  flag (unlike trace `autoprove`).
* `autoproveAll/{strategy}/{bound}` — autoprove every lemma.
`intdot`/`interactive-graph-def` tails in diff mode carry a `graph/` prefix
(opaque passthrough). autoproveDiff/autoproveAll **block the prover** on the
probe theory, so their exact redirect bytes are unobservable here; they are
dispatched as proof ops (new version) with the redirect modelled by analogy to
`diffMethod`/`autoprove` (documented gap).

### 14.8 Negative result — `del/path` and `verify` absent in 1.13.0
`del`, `verify`, and `path` handlers return `404` (not `405`) for every method
and every path shape, in both trace and equiv mode, top-level and theory-scoped;
and appear in **zero** manifest keys. They are neither in the corpus nor
live-observable in this build. `route.rs` routes them to `Handler::Other`
(→ `404`), matching the oracle; the exact bytes of any such route in a newer
tamarin remain a documented gap (no fabrication).

### 14.9 ProverOps growth (Round 4)
Added pure data/fragment producers — `root_meta`, `append_message`,
`static_file`, `load_theory`, `reload`, and the diff ops (`apply_diff_method`,
`autoprove_diff`, `autoprove_all`) — and refined `apply_method` and
`delete_lemma` to return `Option` (method-failure alert / delete-not-found
branch). All transport (content types, status, `Location`, cache headers, version
allocation, envelope shape) stays in `Server`.
