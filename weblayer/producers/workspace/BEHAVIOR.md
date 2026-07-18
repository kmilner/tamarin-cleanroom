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

## 10. Open questions (R2–R5) + unobservables

- UNOBSERVABLE (recorded per protocol, arbitrary-but-documented choices):
  - empty Signature/Construction/Deconstruction bodies never occur → keep-mode
    assumed by analogy with tactic;
  - tabs and C0 controls other than `\n` in fragment text (none in corpus) →
    passed through / standard JSON escapes;
  - heading text is a fixed metachar-free vocabulary → escaping of headings
    unobservable (clean impl escapes them uniformly);
  - NAME/TIME escaping in the help env line (identifiers/clock text carry no
    metachars) → escaped uniformly like ORIGIN.
- The `overview` west pane assembly (R2): item labels/annotations, lemma
  declaration framing, the fresh `by sorry` line, edit/delete/add link shapes.
- Proof-tree indentation + by/case/next/qed grammar (R3).
- Index-page frame + row shape + banners (R4); path grammar + escaping (R5).
