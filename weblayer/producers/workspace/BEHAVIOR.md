# BEHAVIOR.md — observed spec of the web PRODUCER surface

Derived entirely from black-box observation: the 81 captured crawl manifests
(`oracle/captured_responses/`, captured OUTPUT) + live probing of the sanctioned
server (`QUERIES.log`). No prover source is read. Every claim traces to a probe
or a round-1 capture (`round1/targets/*.html`, themselves oracle output).

Terms: a **producer** renders a pre-computed prover value into response-body
CONTENT; the surrounding dispatch/route/envelope machinery is already
clean-roomed (`web-clean`). **Opaque content** = a pretty-printed string the
prover produced (formula / rule / signature / method text) that the producer
embeds but does not compute.

The acceptance gate (`scripts/web_parity.sh`) is **semantic / structural**, not
byte-identity: it canonicalizes away highlight `hl_*` spans, `<br/>`, `&nbsp;`,
attribute/JSON-key order, and volatile env fields, then compares element
structure, visible text, link hrefs, form actions, headings and JSON values. So
reproduce the capture bytes closely, but the BAR is: matching structure + text +
links after that canonicalization.

---

## 1. Fragment families (this cluster's surface)

`main/<section>` routes return the JSON envelope; `overview` returns the full
HTML page; both embed producer content. In scope here (pure-render): `main/
message`, `main/rules`, `main/tactic`, `main/help`; the `overview` west
(proof-script) pane; the proof-tree HTML; the index (`/`) page + housekeeping;
the theory-path grammar. Out of scope (opaque/solver/graph): the source/message
plain-text theory echo, `main/cases` (sources), `main/proof`/`main/method`
(constraint system + applicable methods), `interactive-graph-def`/`intdot`/
`graph` (DOT/SVG), and the Rust-only `proof-step` route.

## 2. Response envelope (R1 skin)

A `main/<section>` body is a JSON object with an html fragment + a pane title.
Observed shape (issue515 main/tactic capture raw bytes [S08], verbatim):

```
{"html":"<h2>Tactic(s)</h2><br/>\n<p class=\"monospace rules\"></p><br/>\n","title":"Tactics"}
```

- The `"html"` key precedes `"title"` — every one of the 16838 `{html,title}`
  bodies in the corpus matches `^{"html":"…","title":"…"}$` exactly (compact, no
  spaces) [S07][S12].
- JSON string escaping, from the full-corpus escape inventory [S09]: `"`→`\"`,
  `\`→`\\`, newline→`\n`, tab→`\t`. No other escape appears anywhere (no
  `\uXXXX`, no `\/`); non-ASCII (e.g. `∀`) is raw UTF-8. Other C0 controls are
  UNOBSERVED (none occur in any body); the clean impl emits standard JSON
  `\r`/`\b`/`\f`/`\u00XX` for them as a documented arbitrary choice.
- The other two envelope shapes:
  - `{"redirect":"<url>"}` — 1157 corpus instances [S07], e.g.
    `{"redirect":"/thy/trace/2/overview/proof/simp"}`.
  - `{"alert":"<msg>"}` — absent from the capture corpus; forced live via
    `del/path/help` [L04]: `{"alert":"Can't delete the given theory path!"}`.
    Same compact single-key shape (also documented by the prior sealed
    web-clean round: ../BEHAVIOR-cited `get_and_append`, `del/path`, method
    failure).

## 3. The per-line postprocess + entity escaping (R1 skin)

Inside an html fragment, the assembled document is emitted one logical line at a
time: EVERY line (first, last, and empty ones included) is emitted as its text
followed by the literal `<br/>` then a real newline, and a line's LEADING run of
spaces becomes a run of `&nbsp;` (one per space; interior space runs are left as
spaces) [S03][S10]. An empty line is therefore exactly `<br/>\n` (the rule
listings' blank separators, and the empty-macros slot §6). All of
message/rules/tactic end with `</p><br/>\n` — the trailing line also carries the
break [S10]. No tab ever appears in an R1 fragment [S10], so tab handling in the
postprocess is UNOBSERVABLE (tabs are passed through unchanged in the clean
impl).

Entity escaping — the full escape set, forced through a producer-owned channel
(the help env line, via an own-authored theory served from the metachar filename
`esc&"<>'probe.spthy` [L06]):

```
&  ->  &amp;      "  ->  &quot;     <  ->  &lt;
>  ->  &gt;       '  ->  &#39;
```

Backslash is NOT escaped in html (appears raw; JSON-escaped `\\` in the
envelope) [L06][S09]. The corpus-wide entity inventory of R1 fragments contains
exactly `&nbsp; &lt; &gt; &quot; &#39; &amp;` and nothing else [S10].

Keyword / operator / comment emphasis inside opaque bodies is wrapped in
`<span class="hl_keyword|hl_operator|hl_comment">` (an hl_comment span can
straddle several logical lines — span tags need not balance per line); these
arrive as part of the pre-rendered content and the gate canonicalizes them away.

Exception: the `main/help` fragment is a single-line template with NO `<br/>`
postprocess (§8).

## 4. The block skeleton shared by message/rules/tactic

A pane is a sequence of headed blocks laid out in a plain-text document, then
put through the §3 postprocess, then enveloped. One block =

```
<h2>HEADING</h2>
<p class="monospace rules">BODY-LINE-1
BODY-LINE-2
…</p>
```

i.e. the `<p …>` opener is glued to the first body line, `</p>` to the last;
an EMPTY body renders the single line `<p class="monospace rules"></p>`
(observed: empty tactic [S04][S08], the `None` line is input not skin). Blocks
follow each other directly — no blank line between blocks [S11][L03]. R1 center
fragments contain no `<a` links at all [S12].

Empty-body behavior is per-block, three observed modes:
- **keep**: block emitted with an empty `<p></p>` (tactic; assumed for the
  message sections, which are never empty in the corpus — UNOBSERVABLE there);
- **blank-line**: heading+paragraph vanish but leave ONE empty line in the
  document (the rules pane's macros slot [L03][S07] — this is the corpus-wide
  leading `<br/>`);
- **omit**: block vanishes without residue (the rules pane's restrictions
  section: absent ⇒ the pane ends right after the MSR block [S07][S10], present
  ⇒ it directly follows `</p><br/>\n` with no blank [S11]).

## 5. `main/message` — the message pane

Title `"Message theory"` (all 81) [S07]. Three sections, always emitted, in
order — heading vocabulary exactly [S07]:

```
Signature | Construction Rules | Deconstruction Rules
```

Bodies are opaque prover content (issue515: Signature = functions:/equations:
lines; construction/deconstruction = the intruder rule listing) [S03].

## 6. `main/rules` — the rules pane

Title is the CONSTANT `"Multiset rewriting rules and restrictions"` across all
81 captures, including the 43 theories with no restrictions [S07]. (The earlier
seeded claim that " and restrictions" is conditional is REFUTED.) Structure:

```
[macros slot]   <h2>Macros</h2> + body   OR   one blank line when no macros
<h2>Fact Symbols with Injective Instances</h2>  …opaque; "None" line when none…
<h2>Multiset Rewriting Rules</h2>               …opaque rule listing…
[<h2>Restrictions of the Set of Traces</h2> …]  omitted without residue if none
```

- The macros slot is FIRST: with macros the pane starts directly with
  `<h2>Macros</h2>` (live probe [L03]); without, the pane starts with the blank
  line `<br/>\n` (all 81 corpus captures — none has macros) [S07].
- The injective-facts body is the single pre-computed line `None` when the
  theory has none (62/81) else a short value line [S07]. Whether the `None`
  fallback text is chosen by the producer or upstream is unobservable at this
  boundary; the clean impl treats it as INPUT (the adapter supplies the line),
  since the tactic pane proves empty-body ⇒ empty `<p>` is the skin's behavior.
- Rule-listing internals (trailing blanks, `/* has exactly the trivial AC
  variant */`, multi-line hl_comment spans) are opaque content.

## 7. `main/tactic` — the tactic pane

Title `"Tactics"` (all 81) [S07]. Single always-present section
`<h2>Tactic(s)</h2>` + one monospace paragraph. Empty tactic ⇒ empty `<p>` (69
corpus captures + lemma-less live theory [L06]); otherwise the opaque tactic
text lines (12 corpus captures, same skin) [S09].

## 8. `main/help` — the help pane

Title `"Theory: <name>"` [S07]. A SINGLE-line body — no `<br/>` postprocess, no
trailing newline; ends `</table></div></p>` [S10]. Shape:

```
<p>Theory: NAME (Loaded at TIME from ORIGIN) BANNER</p><STATIC>
```

- NAME = theory name; TIME = `HH:MM:SS`; ORIGIN = the load-origin text (e.g.
  `Local "/tmp/…/thy/file.spthy"`), entity-escaped by the producer (`&quot;`,
  and the full §3 set — forced via the metachar filename [L06]).
- BANNER: empty string when the theory loaded warning-free — leaving the bytes
  `) </p>` (32/81 [S09], live [L05]) — else the opaque
  `<div class="wf-warning">…</div>` block produced at load time (49/81; its
  internal `<br />`/`<br/>` mix is inside the opaque input).
- STATIC: a fixed help block, byte-identical across all 81 captures [S09],
  starting `<div id="help"><h3>Quick introduction</h3>…` — taken verbatim from
  observed output into the clean impl (compatibility content). It contains a
  stray `</span>` after the Tamarin span — reproduced byte-exactly.
- The `(Loaded at …)` parenthetical is volatile (timestamp + temp path) and is
  normalized away by the acceptance gate on both sides.

## 9. Round-1 validation status

The R1 spec above is implemented in `producers-clean` (`src/html.rs` skin +
`src/section.rs` panes) and validated by:
- `tests/corpus_sweep.rs::corpus_sweep_all_manifests` — all 81 manifests × 4
  center fragments: opaque content sliced out of the capture, re-rendered,
  byte-compared against the RAW response body (envelope included): 324/324.
- `round1_materialized_targets` — the 44 curated `round1/targets/` files.
- `live_probe_replays` — 8 raw bodies captured live [L07] from theories NOT in
  the corpus (metachar-filename EscProbe; macros-bearing MacroGlobalVarNSPK3,
  the only macros-present rules pane observed anywhere).
- fixture tests pinning exact observed bytes ([S08] envelope, [L03] macros
  prefix, [L04] alert, [L06] escapes).
A mutation check (break marker doctored to `<br />`) makes the sweep fail —
the byte gate is live, not vacuous.

## 10. Round-1 unobservables (recorded per protocol)

- empty Signature/Construction/Deconstruction bodies never occur → keep-mode
  assumed by analogy with tactic;
- tabs and C0 controls other than `\n` in fragment text (none in corpus) →
  passed through / standard JSON escapes;
- heading text is a fixed metachar-free vocabulary → escaping of headings
  unobservable (clean impl escapes them uniformly);
- NAME/TIME escaping in the help env line (identifiers/clock text carry no
  metachars) → escaped uniformly like ORIGIN.

---

## 11. R5 — the theory-path grammar (`src/path.rs`)

The wildcard tail after `/thy/trace/<idx>/main/` (the same grammar `del/path/…`
and `verify/…` take [L11]). Pinned from BOTH sides: the corpus href inventory
[S14][S15] (render) and the live acceptance batteries [L08]–[L13] (parse).

**Segment model.** Split the raw tail on `/` FIRST, then percent-decode each
segment independently: `me%73sage` reaches `message`; an encoded `%2F` does NOT
split (`proof/foo%2F_` names lemma `foo/_`) [L09]. Decoding: valid `%XX` → the
byte, byte string read as UTF-8 with U+FFFD replacement (`caf%C3%A9`→café,
`a%FFb`→`a�b`); INVALID sequences stay literal (`a%zzb`, `a%`, `a%2`, `a%G1b`);
`+` is NOT a space [L12]. Heads match exactly and case-sensitively (`MESSAGE`,
`cases/RAW` → 404) [L08][L09].

**Acceptance.** Heads: `help` `message` `rules` `tactic` (no args) ·
`cases/{raw|refined}/{i}/{j}` · `lemma/{name}` · `proof/{lemma}[/seg…]` ·
`edit/{name}` `add/{pos}` `delete/{name}`. Extra segments after a complete
match are IGNORED (`help/extra`, `message/`, `cases/raw/0/0/extra` accepted);
missing required args reject (`proof`, `cases/raw/1` → 404); name args accept
ANY decoded text including empty (`edit/`, `proof//_` parse); lemma EXISTENCE
is resolution, not parse (`proof/nonexistent` → 200 "No such lemma or proof
path") [L08][L11]. `sources` is NOT a head; `method/{lemma}/{n}[/…]` is
server-accepted but outside the producer link vocabulary (no ThyPath
constructor → `parse` returns None; documented interface scope).

**Numeric segments** (the two `cases` indices) parse as a Haskell-`reads`-shaped
integer prefix [L10]: optional whitespace, balanced parens, optional `-` (space
allowed after), one integer lexeme — decimal / `0x` `0o` `0b` any case — then
arbitrary junk IGNORED at top level (`1abc`, `1_`, `(1)x` accept) but not inside
parens (`(1x)` rejects); a decimal lexeme continuing as a FLOAT rejects (`1.0`,
`1e2`); a LEADING underscore rejects (`_1` lexes as an identifier — the
underscore-prefix quirk) while interior/trailing ones are junk (`1_0`, `0x_1`).
`+1` and `--1` reject. The VALUE is behaviorally inert (bodies for 0/0, 0/1,
1/0, 9/9, -1/0 are byte-identical [L10]) — which index is source vs case is
UNOBSERVABLE; clean impl clamps out-of-usize values (documented choice). The
VERSION-index segment before the handler is a different, stricter grammar
(`01`,`+1` accepted; `0x1`, spaces rejected [L10]) and is out of R5 (producers
render it as plain decimal — all hrefs carry plain decimals [S14]).

**Render.** Corpus-wide, rendered segments contain only `[A-Za-z0-9_.]` raw
plus the single escape pair `%3C`/`%3E` in `add/%3Cfirst%3E` (946 corpus + live
hrefs). Encoding of any OTHER byte is UNOBSERVABLE — the metachar-filename
channel collapsed (download/get_and_append URLs derive from the theory NAME,
not the filename [L13]) — clean impl: RFC3986 unreserved raw, everything else
uppercase `%XX` per UTF-8 byte (reproduces every observed href byte; gated by
the 40037-distinct-tail corpus round-trip [S15]).

## 12. R2 — the west (proof-script) pane frame (`src/proofscript.rs`)

The pane is the content of the page's proof-script container: logical lines
each emitted as `TEXT<br/>\n` (leading spaces → `&nbsp;`, i.e. the §3
postprocess) plus ONE trailing space — all 478 overview captures (82 help +
396 proof views; the SPEC's 473 undercounts) [S16]. Element order [S16]:

1. `theory NAME begin` (keyword spans; NAME is a `main/help` link);
2. per nav item: blank, then
   `<a class="internal-link" href="…/main/TAIL"><strong>LABEL</strong> ANN</a>`
   — exactly five, fixed order message / rules / tactic / cases/raw/0/0 /
   cases/refined/0/0. LABEL+ANN are opaque input: `Message theory`/`Tactic(s)`
   with empty ANN (leaving `</strong> </a>`), rules `Multiset rewriting rules`
   ± ` and restrictions` (varies with the theory, unlike the R1 title) with
   `(count)`, `Raw sources`/`Refined sources ` (trailing space in the label)
   with the cases description;
3. blank + the `add lemma` link for `add/%3Cfirst%3E`;
4. per lemma: blank · declaration (§13) · the quantifier/formula block (§13) ·
   `<a … edit/NAME>edit lemma</a>  or  <a … delete/NAME>delete lemma</a>`
   (two spaces around `or`) · the proof display · blank ·
   `add lemma` → `add/NAME`;
5. blank + `end`. ZERO lemmas leave TWO blanks before `end` (both lemma-less
   corpus panes).

**Proof display.** Unproven: the single line `by <a class="internal-link
proof-step sorry-step" href="…/main/proof/NAME">sorry</a>` (keyword spans), no
header wrapper. Proved/disproved: the lemma HEADER (declaration through the
delete anchor) is wrapped in ONE status span — `hl_good` ×3192 / `hl_bad` ×146
— opening before the declaration and closing right after the delete anchor;
the proof lines follow UNWRAPPED, structured by the §16 tree grammar (the
wrapper class is the tree ROOT's status). An INCOMPLETE proof (root step
`sorry-step`, e.g. a half-done induction) leaves the header unwrapped like
sorry [S16]. Every href is
`/thy/trace/{idx}/main/` + an R5-rendered path.

## 13. R2 — lemma declaration + formula layout

Declaration: `lemma NAME{ATTRS}:` — ATTRS empty or starting `" ["` (observed
vocabulary: reuse / use_induction / sources / heuristic={…} / hide_lemma=… and
combinations), and possibly MULTI-LINE (46 corpus declarations wrap long
heuristic lists; the continuation indent is baked into the opaque ATTRS text)
[S17]. The `:` ends the declaration.

Quantifier/formula block at indent 2. A SINGLE-line formula inlines onto the
quantifier line (`  all-traces &quot;F&quot;`) iff the assembled line's
ESCAPED width is ≤ 69, where escaped width = character count with tags
stripped and entities counted at their escaped length (`&lt;` = 4, `&quot;` =
6, unicode operators 1 each). Otherwise the quantifier stands alone and each
formula line follows at 2 + its own relative indent. Provenance: visible
chars/bytes DO NOT separate the corpus (minimal pair at 55 visible: `(a++a)`
inline vs `<a, a>` vertical [S18]); escaped chars separate (65 vs 71), and the
live WProbe bisection pinned the boundary to exactly 69/70 on four formula
families, ruling out a byte-based metric [L14]. Quantifier vocabulary observed:
`all-traces` / `exists-trace`.

## 14. Round-2 validation status

R5 is `path.rs` (`parse`/`render`), R2 is `proofscript.rs` (`render_index`,
links via R5), validated by:
- `tests/r5_path_grammar.rs` — live acceptance battery replay (68 accepted +
  27 rejected probes [L08]–[L12]), decode-echo fixtures, parse⇄render
  round-trip, and the corpus sweep: all 40037 distinct `main/*` href tails
  re-render byte-identically (497 `method/` tails asserted out-of-vocabulary).
- `tests/r2_west_pane.rs` — `corpus_sweep_all_overview_panes`: all 478 pane
  bodies sliced (strict inversion asserting every frame byte) and re-rendered
  byte-identically; `live_probe_pane_replays`: 3 panes from never-captured
  theories (PathProbe fresh; WProbe, the 35-lemma width-boundary theory;
  PathProbe v2 after a LIVE autoprove — proved `hl_good` tree) [L15]; fixtures
  pinning the frame + zero-lemma spacing.
- Mutation checks (all observed to fail, then reverted): `  or  `→` or `
  breaks corpus+live+fixtures; width 69→68 breaks the corpus sweep while
  69→70 breaks ONLY the live WProbe replay (the live bisection pins what the
  corpus cannot); uppercase→lowercase `%XX` breaks the R5 byte tests.
- `cargo test`: 24 green; `cargo clippy --all-targets`: zero warnings.

## 15. Round-2 unobservables

- UNOBSERVABLE (documented choices): href %-encoding beyond `%3C/%3E` (§11);
  which cases index is source vs case (§11); escaping of nav-item
  labels/annotations and attribute text (metachar-free in all observations —
  passed through opaque; lemma/theory NAMES escaped uniformly like R1);
  formula layout for a single-line formula at widths the corpus/live probes
  cannot reach is fixed by the ≤69 rule [L14].

---

## 16. R3 — the proof-tree line grammar (`src/prooftree.rs`)

A lemma's proof display is a TREE of method-labelled nodes rendered as
logical lines in the west pane (then §3-postprocessed with everything else).
Pinned from the 478-pane corpus [S19]–[S21] plus live forcing on own
theories [L16]–[L18]. Per node at indent `d`, URL path `P` = `proof/{lemma}`
+ one segment per case on the root-to-node walk (`_` for an unnamed
continuation; case display names == href segments 1:1, all `[A-Za-z0-9_]+`
in observation — names rendered via the R1 escape + R5 segment encoding):

```
step : {d spaces}[BY]{STEP}[REMOVE]
BY     = wrap(S, '<span class="hl_keyword">by</span> ')   — iff the node has
         NO cases and is not a terminal MARKER (SOLVED-style lines never
         carry `by`; contradiction / zero-case solve / sorry always do)
STEP   = <a class="internal-link proof-step CLS" href="/thy/trace/{idx}/main/P">METHOD</a>
         (CLS = hl_good | hl_bad | sorry-step-for-status-less)
       | wrap(S, METHOD)                                   — Replayed nodes:
         no proof-step link, the method sits in the status span
REMOVE = <a class="internal-link remove-step" href="…same…"></a> — on every
         step EXCEPT the sorry slots (status-less nodes whose only
         continuation, if any, is replayed); replayed leftovers KEEP it
```

- a single unnamed case continues at the SAME indent (segment `_`), no
  case/next/qed framing;
- named cases (1 or more): per case `{d+2}wrap(S_child, 'case NAME')`, the
  child subtree at `d+2`; siblings separated by `{d}wrap(S_parent, 'next')`;
  the block closes `{d}wrap(S_parent, 'qed')`. `next`/`qed` carry the
  PARENT's status — pinned against prev-case (x117) / following-case (x99)
  alternatives corpus-wide [S19] and by the live mixed tree where the case
  after a bad-parent `next` is GOOD [L18]; the case line carries the
  CHILD's status (426612/426612 [S19]).
- `wrap(S, x)` = `<span class="CLS">x</span>` for a statused node, bare `x`
  for a status-less one. Status→class: Good→hl_good, Bad→hl_bad,
  Replayed→hl_superfluous [S20]; Medium→hl_medium is an ASSUMED name (§18).
- METHOD text is opaque pre-rendered input (keyword/operator/comment spans,
  possibly MULTI-LINE with continuation indents baked in — the anchor and
  the superfluous span both straddle physical lines).
- The lemma HEADER wrapper (§12) is the ROOT's status class; a status-less
  root (incomplete proof) leaves it unwrapped [S16][L17].
- The R2 `by sorry` line (§12) is exactly this grammar on a status-less
  sorry leaf; `ProofDisplay::Unproven` remains as the convenience form.

## 17. R3 — statuses and link modes (observed matrix)

| node | anchor class | by-wrap | remove-step |
|------|--------------|---------|-------------|
| proven step (leaf or interior) | `proof-step hl_good` | `hl_good` | yes |
| attack-path step | `proof-step hl_bad` | `hl_bad` | yes |
| incomplete interior (real children) | `proof-step sorry-step` | — | yes [L17] |
| sorry leaf (incl. `/* bound N hit */`, `/* invalid proof step encountered */`) | `proof-step sorry-step` | bare | NO |
| invalid-step sorry carrying a replayed leftover | `proof-step sorry-step` | none (has a child) | NO [S20][L18] |
| replayed leftover step | no anchor; method in `hl_superfluous` span | `hl_superfluous` | yes [S20][L18] |

Crate model: `ProofTree { method_text, status, live, terminal_marker,
cases }` — `live` gates the remove-step (false exactly on the sorry slots),
`terminal_marker` marks the SOLVED-style terminal lines (never `by`; the
interface header carries no such field — the adapter derives it from the
step kind, as it derives `live` from sorry-ness). `ProofDisplay::Rendered`
(round-2 opaque lines) is REPLACED by `ProofDisplay::Tree`.

## 18. R3 unobservables (recorded per protocol)

- `hl_medium` NEVER occurs (corpus census [S21]; live attempts: bounded
  autoprove [L17], characterize mixed tree [L18] — both render without it).
  The Medium→`hl_medium` class name is an assumed pattern extension.
- A live (anchored) Replayed node, a non-live Good/Bad node, and a
  status-less NON-live interior are unobserved; the renderer keys the
  span-vs-anchor choice on Replayed status and the remove-step on `live`.
- Equiv-kind (`/thy/equiv/`) panes: no equiv theory in the corpus and the
  oracle wrapper cannot pass `--diff` — proof-step hrefs are rendered with
  the observed `trace` kind only.
- A case literally NAMED `_`, an empty case name among named siblings, and
  multiple unnamed siblings never occur (`_` segments are only ever inline
  continuations [S14][S21]).
- The superfluous replay drops solver comments from method text (live
  `contradiction` leftover lost `/* from formulas */` [L18]) — method text
  is input; nothing for the renderer.

## 19. R4 — the welcome/index page + housekeeping (`src/welcome.rs`)

No non-`/thy` body exists in the capture corpus — all pinned live
[L19]–[L21]. The `/` page is a fixed frame (verbatim captured segments,
including the doubled `</script></script>` closers; NO trailing newline)
with three slots:

1. FLASH — first child of `<body>`: absent on GET;
   `<p class="message">Loaded new theory!</p>` after a successful upload;
   `<p class="message">Post request failed.</p>` for a POST without a
   usable file; the entity-escaped multi-line load-error text otherwise
   (raw newlines kept) [L20].
2. VERSION — `Running <a href=/><span class="tamarin">Tamarin</span></a> `
   + the version text in the north header [L19].
3. ROWS — between the fixed `<thead>` and `</table>`, one per version,
   index ascending:
   `<tr><td><a href="/thy/trace/IDX/overview/help">NAME</a></td><td>TIME
   </td><td>Original|<em>Modified</td><td>ORIGIN</td></tr>` — the Modified
   cell's `<em>` is UNCLOSED (reproduced byte-exactly); NAME/TIME/ORIGIN
   opaque, entity-escaped by the producer (metachar-filename upload [L21]);
   ORIGIN = load path for a disk theory, bare uploaded filename for an
   upload; initial load = Original, every derived/uploaded version =
   Modified [L19][L20].

Housekeeping bodies [L19]: `robots.txt` → `User-agent: *`; `/kill?path=…`
→ `Canceled request!`; missing `/static` file → `File not found` (all
plain text, no trailing NL); `/kill` without a path → the 400
Invalid-Arguments page: the standard error shell around
`<h1>Invalid Arguments</h1>\n<ul>` + `<li>MSG</li>\n` per message +
`</ul>\n` (single observed instance: `No path to kill specified!`).
`/favicon.ico` is an empty-body 303 (dispatch-side, no producer body).

R4 unobservables: multi-item Invalid-Arguments lists (only the one-item
kill instance is reachable); equiv-kind row hrefs (no `--diff` oracle);
the index-page display cap on many versions is row SELECTION (adapter
state), not rendering.

## 20. Round-3 validation status

R3 is `prooftree.rs` (render_tree_lines / render_tree, consumed by
`proofscript` for every ProofDisplay::Tree), R4 is `welcome.rs`
(render_welcome + render_invalid_args + the plain-text body constants),
validated by:
- tests/r2_west_pane.rs — the corpus sweep UPGRADED: every proof display in
  all 478 panes parsed to a structured tree (frame bytes, status placement,
  canonical hrefs asserted; only method text opaque) and re-rendered
  byte-identically; the 3 round-2 live panes replay unchanged.
- tests/r3_proof_tree.rs — 4 live-probe pane replays (proved TreeProbe,
  bounded TreeProbe, doctored ScriptProbe with a superfluous leftover,
  mixed-status TreeProbe2 characterize) + fixtures pinning every line form.
- tests/r4_welcome.rs — 6 live index-page byte replays (strict slot
  inversion) + housekeeping/Invalid-Arguments byte tests.
- cargo test: 35 green; cargo clippy --all-targets: zero warnings; std-only.
- Mutation checks (observed to fail, reverted): next→following-case status
  breaks the corpus sweep + mixed replay; dropping the marker by-exception
  breaks SOLVED lines everywhere; closing the row `<em>` breaks all index
  replays.
