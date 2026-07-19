# Pretty R7 — web-mode line-wrapping diagnosis (dirty-room / open-side)

Target blocker: BEHAVIOR.md "Web mode (R6) — LAYOUT BLOCKER". The sealed room
proved no single (width, ribbon) reproduces the captures with a faithful
HughesPJ engine, because a nest-3 rule body of content 66 (`c_mult`) WRAPS while
a nest-3 bracket-group premise of content 66 (`d_exp`) STAYS. This document
finds the actual upstream mechanism, reproduces the c_mult/d_exp arithmetic,
and cross-checks it corpus-wide.

**ROOT CAUSE (one sentence):** the interactive server pretty-prints the pane as
a single HughesPJ pass at the default style (width 100, ribbon 67), but the HTML
document transformer **escapes every text token *before* it enters the layout
engine**, so the engine measures the *HTML-escaped* width of each token — `<`
and `>` cost 4 columns, `&` 5, `"` 6, `'` 5 — not the display width. c_mult
contains the arrow `]->` (one `>`, +3), so its 66-display body is measured at 69
and overflows ribbon 67; the d_exp premise group contains no entities, is
measured at 66, and fits. The sealed model measures display widths and escapes
in a post-pass, which is why it cannot separate the two.

--------------------------------------------------------------------------------
## PART A — open-side findings (full detail)
--------------------------------------------------------------------------------

### A1. The render call chain and the per-call unit

File `src/Web/Theory.hs`:

* `htmlThyPath` dispatches the pane routes:
  * `TheoryMessage -> pp $ messageSnippet thy` (the `main/message` pane)
  * `TheoryRules   -> pp $ rulesSnippet thy`   (the `main/rules` pane)
* `messageSnippet thy = vcat [ ppSection "Signature" [prettySignatureWithMaude …]
  , ppSection "Construction Rules" (map prettyRuleAC …)
  , ppSection "Deconstruction Rules" (map prettyRuleAC …) ]` — the whole pane is
  **one** `HtmlDoc Doc`, a `vcat` of all sections. `ppSection` wraps each body in
  `withTag "p" [("class","monospace rules")] (vcat (intersperse (text "") s))`.
* `rulesSnippet` is analogous: one `vcat` of `Macros`, `Fact Symbols …`,
  `Multiset Rewriting Rules` (`map prettyClosedProtoRule` + intruder AC rules),
  `Restrictions …`.
* `pp d = renderHtmlDoc d` (local `where`), and `renderHtmlDoc` lives in
  `lib/utils/src/Text/PrettyPrint/Html.hs`:
  `renderHtmlDoc = postprocessHtmlDoc . render . getHtmlDoc`.
* `render` (`lib/utils/src/Text/PrettyPrint/Class.hs`) = `P.render` =
  HughesPJ `render = renderStyle style`, with
  `style = Style{ lineLength = 100, ribbonsPerLine = 1.5, mode = PageMode }` and
  `ribbonLen = round (100 / 1.5) = 67`. So the engine runs at **width 100,
  ribbon 67** — identical to the sealed pinned params.

**Per-call unit = the WHOLE PANE, one render call.** There is no per-element /
per-rule render; the signature and every rule share one `render`. HughesPJ resets
the ribbon at each newline to `min(width − indent, ribbon)`, so wrap state does
**not** accumulate across the stacked items — each line is measured independently
from its own indentation column. This refutes the "per-element render widths",
"cumulative-ribbon narrowing", and "bespoke rule printer with a tighter threshold"
hypotheses. There is exactly one printer, one style, one pass.

### A2. The measurement bug — escaping happens before layout

`Html.hs`, the `Document` instance for `HtmlDoc d`:

```
instance Document d => Document (HtmlDoc d) where
    char          = HtmlDoc . text . escapeHtmlEntities . return
    text          = HtmlDoc . text . escapeHtmlEntities
    zeroWidthText = HtmlDoc . zeroWidthText . escapeHtmlEntities
```

`escapeHtmlEntities` maps `< > & " '` to `&lt; &gt; &amp; &quot; &#39;`. Because
`text` feeds the **already-escaped** string into the inner `Doc`, the layout
engine's width of a token is the length of its escaped form:

| char | escaped | layout width |
|---|---|---|
| `<` | `&lt;`   | 4 |
| `>` | `&gt;`   | 4 |
| `&` | `&amp;`  | 5 |
| `"` | `&quot;` | 6 |
| `'` | `&#39;`  | 5 |
| others | — | 1 |

The highlight span markers are the *only* zero-width thing: `withTag`/`highlight`
emit the `<span …>`/`</span>` tags through `unescapedZeroWidthText`
(`= HtmlDoc . zeroWidthText`, no escape, size 0). So the sealed room's
"spans are zero-width sentinels" assumption is correct and complete — spans do not
affect layout. What *does* affect layout, and what the sealed model omits, is that
the **content** text is measured at escaped width. This is the whole story.

`postprocessHtmlDoc` (leading-space → `&nbsp;`, `\n` → `<br/>`) runs *after*
`render`, so indentation is still pure nesting during layout (measured as the nest,
not against the ribbon) — unchanged from the sealed model.

### A3. The rule-body printer

`lib/theory/src/Theory/Model/Rule.hs`. `prettyRuleAC` → `prettyNamedRule` →
`prettyRule = prettyRuleRestr … []` → `prettyRuleRestrGen`:

```
prettyNamedRule prefix ppInfo ru =
    prefix <-> name <> attrs <> colon $-$
    nest 2 (prettyRule prems acts concls) $-$
    nest 2 (ppInfo …)

prettyRuleRestrGen ppFact _ prems acts concls _ =
    sep [ nest 1 (ppFactsList prems)
        , if null acts then operator_ "-->"
                       else fsep [operator_ "--[", ppList acts, operator_ "]->"]
        , nest 1 (ppFactsList concls) ]
  where ppList        = fsep . punctuate comma
        ppFactsList l = fsep [operator_ "[", ppList (map ppFact l), operator_ "]"]
```

Structure (matches BEHAVIOR.md R2 exactly): the **whole body** is a `sep` (all
three groups on one line, else stacked) under `nest 2`; the first/third elements
carry an extra `nest 1`, so premise/conclusion groups sit at column 3 and the
arrow at column 2. Each **bracket group** and the **arrow** are `fsep`
(paragraph-fill). `operator_ "["`, `"]"`, `"--["`, `"]->"` are highlighted
zero-width-wrapped text whose *escaped* content is what the fill measures — so
`]->` measures 6 (`]-&gt;`), `-->` measures 5 (`--&gt;`).

Note: the sealed model of the bracket group (`sep [sep [open, fsep facts],
close]`) differs from the upstream `fsep [open, fsep facts, close]`, but that
difference is **not** the cause here — with escaped widths both the whole-body
`sep` and the bracket-group `fsep` share the same one-line ceiling (see A5).

### A4. The c_mult / d_exp arithmetic (the pinned contradiction, resolved)

Ribbon = 67, both constructs at nest 3 (measured from their own indentation).

**c_mult** (whole body, a `sep`):
`[ !KU( x ), !KU( x.1 ) ] --[ !KU( (x*x.1) ) ]-> [ !KU( (x*x.1) ) ]`
- display width = 66
- entities: one `>` in `]->`  → +3
- **escaped width = 69 > 67 → WRAPS** ✓ (capture shows 3 rows)

**d_exp premise group** (a `fsep` bracket group):
`[ !KD( x.5^(x.4*x.6*inv((x.2*x.7))) ), !KU( (x.2*x.3*inv(x.4)) ) ]`
- display width = 66
- entities: none
- **escaped width = 66 ≤ 67 → STAYS** ✓ (keeps its `]`)

Both are display-66 at nest 3; the display-width engine measures them identically
(hence the sealed impossibility). The escaped-width engine separates them by the
single `>` in c_mult's arrow. Confirmed against the real pretty-1.1.3.6 library:
reconstructing the exact docs and rendering at `Style{100, 1.5}` gives c_mult on
**one** line when text is *not* escaped, and c_mult **wrapped** when the arrow's
`>` is escaped to `&gt;` before layout.

Same rule for the neighbours in the Scott message pane:
- `c_exp` `… --[ !KU( x^x.1 ) ]-> …` display 62, one `>` → esc 65 ≤ 67 → stays ✓
- `d_exp` deconstruction `… --> …` display 64, one `>` in `-->` → esc 67 ≤ 67 → stays ✓ (exactly on the ribbon)

### A5. Corpus-wide cross-check

Prediction tested: a rule body / bracket group stays on one line **iff its
HTML-escaped one-line width ≤ 67**.

Every `main/message` + `main/rules` pane (82 theories), spans stripped, `&nbsp;`
→ space, entities un-escaped to recover display text, then re-escaped-width
computed:

* **Whole bodies (sep).** One-line bodies: max escaped width = **67**. Wrapped
  bodies: min escaped width = **69** (the lone "55" is a body whose action term
  itself wrapped — a reconstruction artifact, genuinely long). Clean 3-line/1-line
  bodies judged: **3426 message bodies → 0 mispredictions (100%)**; **467 rules
  bodies → 2 "misses"**, both `[ Fr( ~ska ) ] --> [ !KeyTable( h(), ~ska ) ]`
  where `h()` shows an internally-wrapped argument (artifact, not a counter-example).
* **Bracket groups (fsep).** One-line groups: max escaped width = **67** (6229
  witnesses); 1525 lines are a lone `[` (group whose facts overflowed). Same
  ceiling as bodies.
* Three+ independent near-boundary witnesses in non-Scott theories:
  - `secrecy_4_passive_IN…`  c_mult-analog display 66, `]->` → esc 69 → WRAPS ✓
  - `TAK1…`  `[ !KU( x ), !KD( pmult(x.2, x.3) ) ] --> …` display 66, `-->` → esc 69 → WRAPS ✓
  - `Chaum_Unforgeability…`  display 67, `-->` → esc 70 → WRAPS ✓
  - `…transform…` display 64 `]->`, `aead` `--> ` display 64 → esc 67 → STAY ✓

The law survives every cleanly-reconstructable case.

### A6. Pair / AC / application delimiter drops — same mechanism

The "delimiter collapsing to a space" (`…>) )` vs `…>)`) is the identical
escaped-width effect applied to the pair/AC/application `fcat` fills (the `<`,
`>` delimiters and `'…'` string constants are all fill/text tokens measured at
escaped width). Concrete captures (rules panes), previous-line *display* end
column vs *escaped* end column:

* `BP_IBS_4`: `Out( <'AUTH', pmult(~IBMasterPrivateKey, 'P')` — display end col
  **48**, escaped end col **67** (`<`=+3, two `'…'` pairs = +16). The closing `>`
  does not fit after escaped col 67, so it **drops** to its own line. A
  display-width engine sees end col 48 and keeps `…'P')>` attached — exactly the
  sealed-vs-capture divergence.
* `BP_IBS_4`: `Out( <'SIGN', GetIBMasterPublicKey(~IBMasterPrivateKey)` display
  58 → escaped 69 → `>` drops. Same.

So the `>`/`)` drops the sealed sweep flagged are not a separate bug; they are the
pair/AC/app fills breaking one item earlier under escaped-width measurement.

### A7. The signature-vs-rule "width tension" — dissolved

There is **no** per-section width difference. All sections share one pass at
(100, 67). The apparent tension was:
* signature `functions:` / `builtins:` fills reach absolute column 78
  (= nest 11 + 67) → looked like it needed width ≥ 78, ribbon 67;
* rule bodies wrap at display content 66 (nest 3) → looked like it needed
  effective width ≈ 67.

Resolution: signature `functions:`/`builtins:` fill **items carry no entities**
(`name/arity`, builtin names), so their escaped width equals their display width;
they wrap exactly at the true ribbon 67 from their nest, and (100, 67) reproduces
all 139 of them (as the sealed room found). Rule bodies and equation pairs *do*
carry entities (`]->`, `-->`, `<…>` pairs, `'…'` constants), so they are measured
wider than they display and wrap "early" in display terms. One uniform style; the
only variable is how many escapable characters a given line contains.

Per-family render table (all identical — that is the finding):

| pane / family | render unit | width | ribbon | measured width basis |
|---|---|---|---|---|
| message: Signature | whole pane (one pass) | 100 | 67 | HTML-escaped |
| message: Construction/Deconstruction rules | whole pane (one pass) | 100 | 67 | HTML-escaped |
| rules: Macros / Fact symbols / MSR rules / Restrictions | whole pane (one pass) | 100 | 67 | HTML-escaped |

--------------------------------------------------------------------------------
## PART B — sealed-safe behavioral instruction block
--------------------------------------------------------------------------------

(For the sealed implementer who has read BEHAVIOR.md "Web mode (R6)". This block
contains only observable rendering laws and numbers. Read it in place of Part A.)

**B0. Setting.** All the web-mode wrap laws below hold at the parameters you
already pinned: page width **100**, ribbon **67**, one layout pass over the whole
pane document (the signature block and every rule/restriction are laid out in the
same single pass; there is no per-rule or per-section re-parameterisation, and the
ribbon does not narrow as the document gets deeper — each line is judged
independently from its own indentation column). Keep width 100 / ribbon 67; do
**not** introduce per-family widths.

**B1. The one law you are missing: measure the ESCAPED width, not the display
width.** In web mode, the width the layout engine charges for each character is
its width *after* the entity escaping you already apply — i.e. the layout must be
computed on the escaped form, not on the glyphs. Concretely, when deciding whether
something fits on a line, count each character as:

| character | columns charged to layout |
|---|---|
| `<` | 4 |
| `>` | 4 |
| `&` | 5 |
| `"` | 6 |
| `'` | 5 |
| any other visible character | 1 |

Leading indentation is still free of the ribbon (measured as the indent, as
today). The highlight span markers are still zero width (as today). Only the
*content/operator text* changes: its layout width becomes its escaped length.
Equivalently: escape first, then lay out; do not lay out on glyphs and escape
afterward.

Prediction to check: in batch mode (no escaping) the very same construct is
measured at display width, so batch layout is unchanged — your existing R1–R5
byte parity must stay green. Only web layout shifts.

**B2. Rule-body / bracket-group ceiling.** A rule body (the whole
`[ … ] --[ … ]-> [ … ]`) stays on one line iff its **escaped** width, measured
from its indentation column, is **≤ 67**; otherwise it breaks into the three
stacked rows. A single bracket group (a premise or conclusion `[ … ]`, or the
action group) likewise stays on one line iff its escaped width ≤ 67. Both share
the same ceiling of 67.

Predictions to check against the captures (all at indentation column 3):
* `[ !KU( x ), !KU( x.1 ) ] --[ !KU( (x*x.1) ) ]-> [ !KU( (x*x.1) ) ]` — 66 glyphs
  but one `>` in `]->` → escaped 69 → **must wrap to three rows**.
* `[ !KD( x.5^(x.4*x.6*inv((x.2*x.7))) ), !KU( (x.2*x.3*inv(x.4)) ) ]` — 66 glyphs,
  no escapable characters → escaped 66 → **must stay one line, keeping `]`**.
* A deconstruction body `[ … ] --> [ … ]` of 64 glyphs has one `>` in `-->`
  → escaped 67 → **stays** (sits exactly on the ribbon).
* A construction body `… --[ … ]-> …` of 62 glyphs → escaped 65 → **stays**.
* Corpus scale: across all message-pane rule bodies, the maximum escaped width of
  a one-line body is exactly 67 and the minimum escaped width of a wrapped body is
  69; classifying every cleanly-formed body by "escaped ≤ 67 ⇒ one line" matches
  the captures with zero exceptions (the only apparent misses are bodies whose
  inner fact arguments themselves wrapped).

**B3. Pair / AC / application delimiter drops.** Apply the same escaped-width
measurement inside the pair (`<…>`), AC-operator (`(a*b)`, xor, `++`, `%+`), and
application (`f(…)`) fills you already build. Because `<` and `>` each cost 4 and
`'…'` string constants cost 5 per quote, these fills reach the ribbon sooner than
their glyph count suggests, so a trailing `>` / `)` that a glyph-width engine keeps
attached will instead drop to its own line (or leave the fill's trailing space).
Nothing about *which* delimiter drops changes (your R1/R4 drop rules stand); only
*when* it drops shifts, because the fit is judged on escaped width.

Prediction to check: `Out( <'AUTH', pmult(~IBMasterPrivateKey, 'P')` — the glyph
end column is 48 but the escaped end column is 67, so the closing `>` of the pair
**must drop below** to the pair's opening column, not stay attached as `…'P')>`.
Likewise `Out( <'SIGN', GetIBMasterPublicKey(~IBMasterPrivateKey)` (escaped end
column 69) drops its `>`.

**B4. Why the signature block still matched at display width.** The
`functions:` / `builtins:` fill items contain no escapable characters, so their
escaped width equals their glyph width and they wrap at the true ribbon 67 — which
is why your signature sweep already passes at (100, 67). Equation lines and any
line containing `<`, `>`, `&`, `"`, `'` must now be judged on escaped width like
everything else; expect equation pairs (`<x.1, x.2>`) to be charged +3 per angle
bracket.

Prediction to check: a signature line reaching absolute column 78 (a long
`functions:` continuation) has no entities, so its escaped and glyph widths agree
and it is unaffected by this change; a rule body of the same glyph length that
contains an arrow wraps, because the arrow's `>` pushes its escaped width past 67.
The two are no longer in tension: identical parameters, different escapable-char
counts.

**B5. Summary of the fix.** Do not change the parameters, the section structure,
the span vocabulary, the escaping output, or the one-pass whole-pane rendering.
Change exactly one thing: make the wrap/fit decision measure the **escaped**
width of each token (`<`,`>` = 4; `&` = 5; `"` = 6; `'` = 5; else 1), for rule
bodies, bracket groups, quantifier and formula group layouts, and the
pair/AC/application fills alike. That single change turns the "impossible" c_mult-vs-d_exp split into
a forced consequence and reproduces the delimiter drops.

_Certification: Part B contains no upstream identifiers, file paths, code quotes,
or expressions — only observable web-mode rendering laws, numeric column costs,
and capture-checkable predictions._
