//! Input model — the pre-computed prover VALUES the producers render.
//!
//! These are the crate's OWN, minimal, behavior-named types for unit testing
//! (build a value with Rust constructors, assert its rendered fragment equals an
//! observed capture). At integration the open side wires thin adapters from the
//! live typed-theory / proof-state values into this shape; see
//! ../../interface/ for the integration surface.
//!
//! Everything here is a value the producer only RENDERS. The text of a formula,
//! rule, signature, or proof method is computed elsewhere (the pretty / solver
//! side) and reaches the producer as opaque `content` strings — the producer
//! adds the HTML/JSON skin (tags, links, escaping, line breaks, envelope) the
//! captures show around it. No field here re-derives prover content.

// ---------------------------------------------------------------------------
// R1 — theory-view CENTER section fragments (`main/<section>`)
// ---------------------------------------------------------------------------

/// A content pane addressed by a `main/<section>` route, rendered as the
/// `{title, html}` response envelope. The pane is an ordered list of headed
/// blocks; the HTML skin (heading tags, monospace paragraph, the per-line
/// break/indent postprocess) is the producer's to add.
pub struct ContentPane {
    /// Envelope title (e.g. the message / rules / tactic pane's title text).
    pub title: String,
    /// Headed blocks, in document order.
    pub blocks: Vec<HeadedBlock>,
}

/// One headed block: a heading line plus a monospace body.
pub struct HeadedBlock {
    /// Section heading text (a fixed vocabulary chosen per route).
    pub heading: String,
    /// Pre-rendered body content (opaque prover text, already line-structured
    /// and pre-skinned: entity-escaped with any emphasis spans in place).
    /// Empty is meaningful.
    pub body: Content,
    /// How the block renders when its body is empty (BEHAVIOR.md §4).
    pub when_empty: EmptyRender,
}

/// Empty-body rendering modes, all three observed (BEHAVIOR.md §4).
///
/// The interface header models this as `keep_when_empty: bool`; `Keep`/`Omit`
/// map onto true/false and `BlankLine` is the rules pane's macros slot, pinned
/// by live probe [L03] (macros present ⇒ the pane starts with `<h2>Macros</h2>`,
/// absent ⇒ a single blank line where the block would be — the corpus-wide
/// leading `<br/>`). The integration adapter picks the mode per section.
pub enum EmptyRender {
    /// Emit heading + an empty paragraph (tactic; message sections by analogy —
    /// never observed empty).
    Keep,
    /// Drop heading + paragraph but leave one blank line in the document (the
    /// rules pane's macros slot).
    BlankLine,
    /// Vanish without residue (the rules pane's restrictions section).
    Omit,
}

/// The `main/help` pane input (BEHAVIOR.md §8): the env line's fields plus the
/// pre-computed wellformedness banner. The static help block is the producer's
/// own fixed content.
pub struct HelpPane {
    pub theory_name: String,
    /// Load wall-clock time as displayed (`HH:MM:SS`).
    pub load_time: String,
    /// Load-origin text (e.g. `Local "/tmp/…/thy/file.spthy"`), un-escaped;
    /// the producer entity-escapes it.
    pub origin: String,
    /// Raw pre-rendered `<div class="wf-warning">…</div>` block, or empty when
    /// the theory loaded warning-free (opaque load-time input).
    pub wf_banner_html: String,
}

/// Opaque pre-rendered content: a sequence of logical lines plus, where known,
/// which spans are keyword / operator / comment emphasis (used only for the
/// byte-close HTML skin; the acceptance gate compares visible text, not skin).
pub struct Content {
    pub lines: Vec<String>,
}

impl Content {
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty() || self.lines.iter().all(|l| l.is_empty())
    }
}

// ---------------------------------------------------------------------------
// R2 — proof-script WEST pane (the theory index shown left of every page)
// ---------------------------------------------------------------------------

/// The whole proof-script pane input.
pub struct ProofScriptPane {
    pub theory_name: String,
    /// Resolved numeric version index used in every internal link.
    pub index: u64,
    /// Top navigation items (message / rules / tactic / sources), in order,
    /// each with its label and pre-computed annotation.
    pub items: Vec<NavItem>,
    pub lemmas: Vec<LemmaEntry>,
}

/// One top-of-index navigation item.
///
/// The interface header carries the link target as a `section` string (the
/// `main/<…>` wildcard tail); this crate-local model holds it as the R5
/// [`ThyPath`] instead so the link is CONSTRUCTED through the theory-path
/// grammar — the integration adapter maps the live value via `path::parse`.
pub struct NavItem {
    /// The `main/<…>` route the item links to (rendered via R5).
    pub target: ThyPath,
    /// Bold label text (opaque pre-rendered input; the observed vocabulary is
    /// fixed per slot — BEHAVIOR.md §12).
    pub label: String,
    /// Trailing annotation text (a count / cases summary), possibly empty
    /// (opaque pre-rendered input).
    pub annotation: String,
}

/// One lemma's index entry: its declaration plus how its proof displays.
pub struct LemmaEntry {
    pub name: String,
    /// Attribute list text (already assembled by the pretty side), e.g.
    /// `" [reuse]"`, or empty. May span several logical lines (long heuristic
    /// lists wrap, with the continuation indent baked into the text).
    pub attributes: String,
    /// Trace-quantifier keyword (`all-traces` / `exists-trace`).
    pub quantifier: String,
    /// Pre-rendered formula body (opaque prover text, line-structured), at
    /// indents RELATIVE to the declaration body: the first line carries no
    /// leading spaces, continuation lines their own deeper indent. The
    /// renderer adds the 2-space block indent and decides the inline/vertical
    /// quantifier layout (BEHAVIOR.md §13).
    pub formula: Content,
    pub proof: ProofDisplay,
}

/// How a lemma's proof renders in the index.
pub enum ProofDisplay {
    /// Unproven: a single trailing `by sorry` step (no status wrapper).
    Unproven,
    /// A proof carried as pre-rendered display lines (each already a complete
    /// HTML logical line), plus the status class the lemma HEADER wrapper span
    /// carries (`None` for an incomplete proof — no wrapper). This is the
    /// R2-level opaque form; R3 structures the tree ([`ProofTree`]) and will
    /// render into exactly these lines.
    Rendered {
        header_status: Option<String>,
        lines: Vec<String>,
    },
    /// A structured proof tree (R3 — not yet rendered by this crate).
    Tree(ProofTree),
}

// ---------------------------------------------------------------------------
// R3 — proof-tree + proof-method HTML
// ---------------------------------------------------------------------------

/// A proof tree: a method-labelled node with named child cases. Pre-computed;
/// the producer only lays it out as nested HTML (indent, keywords, links).
pub struct ProofTree {
    /// Pre-rendered proof-method text for this node (opaque prover value).
    pub method_text: String,
    /// Highlight status this node's structural keywords carry.
    pub status: Highlight,
    /// Whether this node was reached by real proof search (vs replayed) — gates
    /// whether a remove-step affordance is emitted.
    pub live: bool,
    /// Named child cases, in order; the empty name means a single unnamed
    /// continuation (no `case` label).
    pub cases: Vec<(String, ProofTree)>,
}

/// The per-step highlight the observed status classes encode.
pub enum Highlight {
    None,
    Good,
    Bad,
    Medium,
    Replayed,
}

// ---------------------------------------------------------------------------
// R4 — welcome / index page + housekeeping
// ---------------------------------------------------------------------------

/// The index (`/`) page input: the loaded-theory table plus a one-shot banner.
pub struct Welcome {
    pub version: String,
    pub banner: Banner,
    pub rows: Vec<TheoryRow>,
}

pub enum Banner {
    None,
    Loaded,
    Failed,
    Custom(String),
}

/// One row of the loaded-theory table (all fields supplied verbatim).
pub struct TheoryRow {
    pub index: u64,
    pub name: String,
    pub time: String,
    /// A modified (derived) version renders differently from a primary one.
    pub modified: bool,
    pub origin: String,
}

// ---------------------------------------------------------------------------
// R5 — theory-path grammar (URL <-> structured path)
// ---------------------------------------------------------------------------

/// A theory-internal path parsed from / rendered to the wildcard URL segment
/// after `/thy/trace/<idx>/<handler>/`.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ThyPath {
    Help,
    Message,
    Rules,
    Tactic,
    Sources { refined: bool, source_idx: usize, case_idx: usize },
    Lemma(String),
    Proof { lemma: String, sub: Vec<String> },
    Edit(String),
    Add(String),
    Delete(String),
}
