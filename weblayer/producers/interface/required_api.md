# Required public API (interoperability surface) — producers-clean

The clean crate must expose these entry points so the open side can wire it in
behind thin adapters that translate the live pre-computed prover values into the
crate's input shape (`fragment_inputs.rs`). Each function returns the exact
response-body bytes the corresponding fragment shows in the captures. Names are
indicative; the boundary that matters is "one entry point per observable
sub-target, each independently gate-checkable against the capture corpus".

```rust
// ── R1: theory-view CENTER section fragment (`main/message` / `main/rules`
//    / `main/tactic` / `main/help`) — the whole response body, envelope
//    included. The deepest, most-reused leaf. ──
pub fn render_content_pane(pane: &ContentPane) -> String;

//    …built on the shared HTML skin every producer reuses:
pub fn escape_text(s: &str) -> String;           // text → the observed entities
pub fn postprocess_lines(assembled: &str) -> String; // lines → breaks + indents
pub fn html_envelope(title: &str, html: &str) -> String;  // → the observed JSON
pub fn redirect_envelope(url: &str) -> String;
pub fn alert_envelope(msg: &str) -> String;

// ── R2: proof-script WEST pane (the theory index left of every page) ──
pub fn render_proof_script(index: &ProofScriptPane) -> String;

// ── R3: proof-tree + proof-method HTML (embedded in the west pane and the
//    per-path proof fragment) ──
pub fn render_proof_tree(index: u64, lemma: &str, tree: &ProofTree) -> String;

// ── R4: welcome / index page (`/`) + housekeeping bodies ──
pub fn render_welcome(w: &Welcome) -> String;

// ── R5: theory-path grammar (URL <-> structured path), pure, HTML-free ──
pub fn parse_path(raw: &str) -> Option<ThyPath>;
pub fn render_path(path: &ThyPath) -> Vec<String>;
```

`ContentPane`, `HeadedBlock`, `Content`, `ProofScriptPane`, `NavItem`,
`LemmaEntry`, `ProofDisplay`, `ProofTree`, `Highlight`, `Welcome`, `Banner`,
`TheoryRow`, `ThyPath`: see `fragment_inputs.rs`.

Every value these functions receive is pre-computed by the prover/pretty side
and only RENDERED here; the crate never re-derives prover content. All observed
bytes (tag skeletons, headings, titles, link targets, escaping, line breaks, the
envelope shape) are learned from the capture corpus + the live oracle — none are
specified in this file. Byte targets are captured OUTPUT only.
