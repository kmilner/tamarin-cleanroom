//! Minimal AST subset the renderer consumes.
//!
//! This is the clean crate's OWN model, shaped to stay drop-in compatible with
//! the integration surface in `../../interface/ast_types.rs` (same variant
//! names and field meanings) while only carrying what the R1 renderer needs.
//! Behavior is driven by the oracle (see workspace/BEHAVIOR.md), not by that
//! header.
//!
//! R2–R4 types are placeholders — flesh them out at those sub-targets.

// ── R1: terms (the deep core) ───────────────────────────────────────────────

/// Multiset-rewriting term. Rendered by `term::render`.
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Var(VarSpec),
    /// Public string constant; renders quoted: `'name'`.
    PubLit(String),
    /// Fresh name constant; renders `~'name'`.
    FreshLit(String),
    /// Natural-number literal, digits only; renders `%digits`.
    NatLit(String),
    /// Natural-number literal; renders `%n`.
    Number(u64),
    /// The DH multiplicative unit; renders `one`.
    NumberOne,
    /// The natural-number one; renders `%1`.
    NatOne,
    /// The DH group neutral element; renders `DH_neutral`.
    DhNeutral,
    /// Function application `f(a, b, …)`; arity 0 renders bare (`f`).
    App(String, Vec<Term>),
    /// Named binary algebra operator; `"exp"` renders infix `a^b`, anything
    /// else falls back to application form.
    AlgApp(String, Box<Term>, Box<Term>),
    /// Tuple `<a, b, c>`; a Pair in LAST position flattens into the enclosing
    /// tuple, a Pair in any other position keeps its own delimiters.
    Pair(Vec<Term>),
    /// Bi-system term `diff(l, r)`; renders in application form.
    Diff(Box<Term>, Box<Term>),
    BinOp(BinOp, Box<Term>, Box<Term>),
    /// Sapic pattern-match marker (`=t`). UNOBSERVABLE through the no-prove
    /// MSR echo — see BEHAVIOR.md; the rendering here is a flagged
    /// placeholder, not oracle-pinned.
    PatMatch(Box<Term>),
}

/// Infix operators. Glyphs and parenthesization per BEHAVIOR.md: `Exp`
/// renders `a^b` with flat chains and no added parens; the four AC operators
/// render `(a<op>b<op>c)` — self-parenthesized, flattened, no spaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    Exp,     // ^
    Mult,    // *
    Union,   // ++
    Xor,     // ⊕ (U+2295)
    NatPlus, // %+
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VarSpec {
    pub name: String,
    /// Rendered as a `.idx` suffix when > 0 (`x.1`, `~x.7`).
    pub idx: u64,
    pub sort: SortHint,
    /// Source-level type annotation; not rendered in the theory echo.
    pub typ: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SortHint {
    Msg,
    Pub,   // $x
    Fresh, // ~x
    Node,  // #i
    Nat,   // %n
    /// Source-suffix form (`x:pub`); the echo always shows the sigil form,
    /// so this renders identically to the corresponding sigil sort.
    Suffix(SuffixSort),
    #[default]
    Untagged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SuffixSort {
    Msg,
    Pub,
    Fresh,
    Node,
    Nat,
}

// ── R1: signature block ─────────────────────────────────────────────────────

/// The DECLARED signature to render as `builtins:` / `functions:` /
/// `equations:`. The renderer itself performs the observable closure: builtin
/// expansion into function/equation entries, canonical builtin ordering,
/// dedup, and sorting (all oracle-pinned — BEHAVIOR.md "Signature section").
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Signature {
    /// Builtin names as declared (`"diffie-hellman"`, `"hashing"`, …).
    pub builtins: Vec<String>,
    /// User-declared function symbols.
    pub functions: Vec<FunctionDecl>,
    /// User-declared equations.
    pub equations: Vec<Equation>,
    /// True when the user equation block carries `[convergent]`.
    pub convergent: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub arity: usize,
    pub private: bool,
    pub destructor: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Equation {
    pub lhs: Term,
    pub rhs: Term,
}

// ── R2: facts & rules ───────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Fact {
    /// Renders a `!` prefix before the name.
    pub persistent: bool,
    pub name: String,
    pub args: Vec<Term>,
    /// Renders `[+, -, no_precomp]` attached after the closing paren, in
    /// that canonical order whatever the source order (probe:p_fann,
    /// target:seqdfsneeded).
    pub annotations: Vec<FactAnnotation>,
}

/// Fact annotations as observed in the echo (probe:p_fann): `+` / `-` /
/// `no_precomp`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FactAnnotation {
    SolveFirst, // +
    SolveLast,  // -
    NoSources,  // no_precomp
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub name: String,
    /// `Some("E")` renders `rule (modulo E) name`; the AC rule inside a
    /// variants comment carries `Some("AC")`; `None` renders plain
    /// `rule name`.
    pub modulo: Option<String>,
    pub attributes: Vec<RuleAttr>,
    pub premises: Vec<Fact>,
    pub actions: Vec<Fact>,
    pub conclusions: Vec<Fact>,
    /// 0-based premise indices annotated `// loop breaker(s): [..]`
    /// (probes c_loop, p_lb2).
    pub loop_breakers: Vec<usize>,
}

/// Rule attributes. Only color / no_derivcheck / issapicrule / role render,
/// in that canonical order (probe:p_rattr, target:issue713); `process=…` and
/// external attributes are dropped from the echo.
#[derive(Debug, Clone, PartialEq)]
pub enum RuleAttr {
    /// Renders `color=#<value>` (value as supplied, lowercased upstream).
    Color(String),
    NoDerivCheck,
    Role(String),
    IsSapicRule,
    Process(String),
    External(String, Option<String>),
}

/// Pre-computed AC-variant data for a rule (from the ported solver); the
/// `variants (modulo AC)` comment renders from this. `None` at the render
/// call site means the trivial-variant comment.
#[derive(Debug, Clone, PartialEq)]
pub struct AcVariants {
    /// The rule normalized modulo AC, re-rendered inside the comment.
    pub ac_rule: Rule,
    /// Numbered substitution groups; each entry is (variable, term), both
    /// rendered by the R1 term core.
    pub substitutions: Vec<Vec<(Term, Term)>>,
}

// ── R3: formulas, restrictions, lemmas ──────────────────────────────────────

/// Trace formula. Rendered by `formula::render`; the statement printer
/// parenthesizes EVERY operand of a binary connective and every ¬ argument,
/// keeps quantifier bodies bare, and mirrors the source association of
/// chains (probe:q_p2 — left/right chains echo distinctly).
#[derive(Debug, Clone, PartialEq)]
pub enum Formula {
    /// Renders `⊥`.
    False,
    /// Renders `⊤`.
    True,
    Atom(Atom),
    /// Renders `¬(…)` — the argument always parenthesized (probe:q_p2 s8).
    Not(Box<Formula>),
    /// Renders `(l) ∧ (r)`.
    And(Box<Formula>, Box<Formula>),
    /// Renders `(l) ∨ (r)`.
    Or(Box<Formula>, Box<Formula>),
    /// Renders `(l) ⇒ (r)`.
    Implies(Box<Formula>, Box<Formula>),
    /// Renders `(l) ⇔ (r)` (probe:q_r2).
    Iff(Box<Formula>, Box<Formula>),
    /// Renders `∀ v1 v2. body` — binders in declaration order with their
    /// sort sigils (probe:q_b1), body bare at nest 1 (probe:q_l2).
    Forall(Vec<VarSpec>, Box<Formula>),
    /// Renders `∃ v1 v2. body`.
    Exists(Vec<VarSpec>, Box<Formula>),
}

/// Formula atoms as observed in the echo (probes q_at1, q_l4, q_l5).
#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    /// `l = r`.
    Eq(Term, Term),
    /// `l < r` (temporal or nat ordering).
    Less(Term, Term),
    /// Multiset smaller. UNOBSERVABLE through the no-prove echo (no corpus
    /// witness, no reachable source spelling found) — rendered like `Less`
    /// as a flagged placeholder, see BEHAVIOR.md.
    LessMset(Term, Term),
    /// `l ⊏ r` (probe:q_at1, target:NumberSubtermTests).
    Subterm(Term, Term),
    /// `Fact( … ) @ tp` — `@ tp` attached beside the fact's last line
    /// (probe:q_l5).
    Action(Fact, Term),
    /// `last(tp)` — no interior spaces (probe:q_at1).
    Last(Term),
    /// Predicate fact atom. UNOBSERVABLE: predicates are expanded upstream
    /// of the echo (probe:q_pred1) — rendered as a bare fact, flagged.
    Pred(Fact),
}

/// `restriction name: "formula"` plus the conditional `// safety formula`
/// line and the `/* expanded formula: … */` comment (probe:q_w1). The
/// `axiom` keyword echoes as a restriction too (probe:q_ax1).
#[derive(Debug, Clone, PartialEq)]
pub struct Restriction {
    pub name: String,
    pub formula: Formula,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceQuantifier {
    AllTraces,
    ExistsTrace,
}

/// Lemma attributes as observed (probes q_la1, q_la2, q_la3, target:5G_AKA).
/// Rendered in SOURCE order, duplicates kept — no canonicalization
/// (probe:q_la1). Attributes without a corpus/probe witness (left/right,
/// output=…, diff-mode spellings) are NOT modeled — see the UNOBSERVABLE
/// register in BEHAVIOR.md.
#[derive(Debug, Clone, PartialEq)]
pub enum LemmaAttr {
    /// `sources`.
    Sources,
    /// `reuse`.
    Reuse,
    /// `use_induction`.
    UseInduction,
    /// `hide_lemma=<name>`.
    HideLemma(String),
    /// `heuristic=<value>` — the value string is carried verbatim, braces
    /// included (`S`, `{mytac}` — probes q_la2/q_la3).
    Heuristic(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lemma {
    pub name: String,
    pub attributes: Vec<LemmaAttr>,
    pub trace_quantifier: TraceQuantifier,
    pub formula: Formula,
    /// Embedded proof script, re-rendered by the PORTED proof printer and
    /// handed here verbatim (column-0 multi-line text). `None` renders the
    /// no-prove placeholder `by sorry` (probe:q_w1, target:Yubikey).
    pub proof: Option<String>,
}

/// Pre-computed guarded-formula comment body (from the ported guarded
/// transform), rendered inside `/* … */` after the lemma statement. The
/// header line is chosen by the renderer from the lemma's trace quantifier;
/// the CONTENT is opaque input (probe:q_w1, probe:q_r1).
#[derive(Debug, Clone, PartialEq)]
pub enum Guarded {
    /// Successful conversion: the quoted multi-line guarded-formula block
    /// (quotes included), emitted verbatim at column 0.
    Formula(String),
    /// Failed conversion: the raw error text (column-0 lines, as the
    /// transform reports it); the comment frame indents every line by 2
    /// under a `conversion to guarded formula failed:` header (probe:q_r1).
    Failed(String),
}

// ── R4: macros / predicates ─────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Macro {
    pub name: String,
    pub params: Vec<Term>,
    pub body: Term,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Predicate {
    pub name: String,
    pub params: Vec<VarSpec>,
    pub body: Formula,
}

// ── top-level theory ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Theory {
    pub name: String,
    pub items: Vec<TheoryItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TheoryItem {
    Macros(Vec<Macro>),
    Predicates(Vec<Predicate>),
    Rule(Rule),
    Restriction(Restriction),
    Lemma(Lemma),
    FormalComment { header: String, body: String },
}
