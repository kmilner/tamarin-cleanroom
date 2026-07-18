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

// ── R2: facts & rules (placeholders — flesh out at R2) ──────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct Fact {
    pub persistent: bool, // `!Name`
    pub name: String,
    pub args: Vec<Term>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub name: String,
    pub premises: Vec<Fact>,
    pub actions: Vec<Fact>,
    pub conclusions: Vec<Fact>,
}

/// Pre-computed AC-variant substitutions for a rule (from the ported solver);
/// the `variants (modulo AC)` block text renders from this. Shape TBD at R2.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct AcVariants;

// ── R3: formulas, restrictions, lemmas (placeholders — flesh out at R3) ─────

#[derive(Debug, Clone, PartialEq)]
pub enum Formula {
    False,
    True,
    Atom(Atom),
    Not(Box<Formula>),
    And(Box<Formula>, Box<Formula>),
    Or(Box<Formula>, Box<Formula>),
    Implies(Box<Formula>, Box<Formula>),
    Iff(Box<Formula>, Box<Formula>),
    Forall(Vec<VarSpec>, Box<Formula>),
    Exists(Vec<VarSpec>, Box<Formula>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    Eq(Term, Term),
    Less(Term, Term),
    Subterm(Term, Term),
    Action(Fact, Term), // Fact @ #i
    Last(Term),
    Pred(Fact),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Restriction {
    pub name: String,
    pub formula: Formula,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lemma {
    pub name: String,
    pub exists_trace: bool,
    pub formula: Formula,
}

/// Pre-computed guarded-formula negation shown in the `/* guarded formula
/// characterizing all counter-examples */` comment (from the ported guarded
/// transform). Shape TBD at R3.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Guarded;

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
