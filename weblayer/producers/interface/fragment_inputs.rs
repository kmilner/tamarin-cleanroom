// INTEROPERABILITY HEADER (open-side provided, expression-stripped).
//
// The BEHAVIORAL input surface for the web fragment PRODUCERS: the shape of the
// pre-computed prover values each producer renders into a response-body
// fragment. Provided so the clean implementation can compile against a stable
// contract at integration time; the open side writes thin adapters from the
// live values into this shape.
//
// This is a BEHAVIORAL contract, not a transcription of any existing data
// model: every type/field below is named for what the producer OBSERVABLY does
// with it. No item here re-derives prover CONTENT — the text of a formula,
// rule, signature block, or proof method is computed elsewhere and reaches the
// producer as opaque `content`/`text` strings. The producer's whole job is the
// HTML/JSON SKIN the captures show around that content: tags, headings, links,
// escaping, line breaks, and the response envelope.
//
// The clean implementer MAY define an equivalent minimal model for unit testing
// (see workspace/producers-clean/src/model.rs) and only needs this exact shape
// to drop into the workspace at integration. All observable bytes are learned
// from the captures + the oracle; nothing about them is specified here.

// ── R1: theory-view CENTER section fragment (`main/<section>`) ──────────────

/// A content pane: an ordered list of headed blocks + the pane's envelope title.
pub struct ContentPane {
    pub title: String,
    pub blocks: Vec<HeadedBlock>,
}

/// A heading line + a monospace body. `keep_when_empty` distinguishes the
/// always-present sections from the ones that vanish when their body is empty.
pub struct HeadedBlock {
    pub heading: String,
    pub body: Content,
    pub keep_when_empty: bool,
}

/// Opaque pre-rendered content: a sequence of logical lines (each becomes one
/// laid-out line in the fragment). Empty is meaningful.
pub struct Content {
    pub lines: Vec<String>,
}

// ── R2: proof-script WEST pane (the theory index) ──────────────────────────

pub struct ProofScriptPane {
    pub theory_name: String,
    pub index: u64,
    pub items: Vec<NavItem>,
    pub lemmas: Vec<LemmaEntry>,
}

pub struct NavItem {
    pub section: String,
    pub label: String,
    pub annotation: String,
}

pub struct LemmaEntry {
    pub name: String,
    pub attributes: String,
    pub quantifier: String,
    pub formula: Content,
    pub proof: ProofDisplay,
}

pub enum ProofDisplay {
    Unproven,
    Tree(ProofTree),
}

// ── R3: proof-tree + proof-method HTML ─────────────────────────────────────

/// A method-labelled node with named child cases. Pre-computed; the producer
/// only lays it out as nested HTML.
pub struct ProofTree {
    pub method_text: String,
    pub status: Highlight,
    pub live: bool,
    pub cases: Vec<(String, ProofTree)>,
}

pub enum Highlight {
    None,
    Good,
    Bad,
    Medium,
    Replayed,
}

// ── R4: welcome / index page + housekeeping ────────────────────────────────

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

pub struct TheoryRow {
    pub index: u64,
    pub name: String,
    pub time: String,
    pub modified: bool,
    pub origin: String,
}

// ── R5: theory-path grammar (URL <-> structured path) ──────────────────────

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
