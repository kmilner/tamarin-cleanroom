//! Minimal AST subset the renderer consumes.
//!
//! This is the clean crate's OWN model — you refine it as the oracle teaches
//! you what each construct renders to. The integration-time surface (what the
//! open side adapts live values into) is `../../interface/ast_types.rs`; keep
//! these shapes compatible with it as you go, but drive the design from the
//! oracle, not from that header.
//!
//! Seeded for R1 (term + signature). R2–R4 types are placeholders — flesh them
//! out when you reach those sub-targets.

// ── R1: terms (the deep core) ───────────────────────────────────────────────

/// Multiset-rewriting term. Rendered by `term::render`.
#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Var(VarSpec),
    PubLit(String),   // 'name'
    FreshLit(String), // ~'name'-style fresh constant
    NatLit(String),
    Number(u64),
    NumberOne,
    NatOne,
    DhNeutral,
    App(String, Vec<Term>),           // f(a, b, …)
    AlgApp(String, Box<Term>, Box<Term>), // exp(a,b) -> a^b, etc.
    Pair(Vec<Term>),                  // <a, b, c>
    Diff(Box<Term>, Box<Term>),       // diff(a, b)
    BinOp(BinOp, Box<Term>, Box<Term>),
    PatMatch(Box<Term>),
}

/// Associative-commutative / infix operators and their surface glyphs — learn
/// the exact glyph and precedence/parenthesization from the oracle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    Exp,     // ^
    Mult,    // *
    Union,   // ++
    Xor,     // ⊕  (U+2295)
    NatPlus, // %+
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VarSpec {
    pub name: String,
    pub idx: u64, // rendered as `.idx` suffix when > 0
    pub sort: SortHint,
    pub typ: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SortHint {
    Msg,
    Pub,    // $x
    Fresh,  // ~x
    Node,   // #i
    Nat,    // %n
    #[default]
    Untagged,
}

// ── R1: signature block ─────────────────────────────────────────────────────

/// The CLOSED signature to render as `builtins:` / `functions:` / `equations:`.
/// Supplied pre-merged/pre-sorted by the ported closure; you render its text.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Signature {
    pub builtins: Vec<String>,
    pub functions: Vec<FunctionDecl>,
    pub equations: Vec<Equation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub arity: usize,
    pub private: bool,
    pub constructor: bool, // vs destructor
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
/// you render the `variants (modulo AC)` block text. Shape TBD at R2.
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
