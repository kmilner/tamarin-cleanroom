// INTEROPERABILITY HEADER (dirty-room provided, expression-stripped):
// type surface of the parsed-theory data model the checker consumes.
// Comments and all impl bodies removed. Provided solely so the clean
// implementation can compile against the existing data model.



#[derive(Debug, Clone, PartialEq)]
pub struct Theory {
    pub is_diff: bool,
    pub name: String,
    pub configuration: Option<String>,
    pub items: Vec<TheoryItem>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TheoryItem {
    Builtins(Vec<String>),
    Functions(Vec<FunctionDecl>),
    Equations { convergent: bool, eqs: Vec<Equation> },
    Macros(Vec<Macro>),
    Predicates(Vec<Predicate>),
    Options(Vec<String>),
    Heuristic(String),
    Tactic(Tactic),
    Restriction(Restriction),
    LegacyAxiom(Restriction),
    Rule(Rule),
    IntrRule(Rule),
    Lemma(Lemma),
    DiffLemma(DiffLemma),
    AccLemma(AccLemma),
    CaseTest(CaseTest),
    ProcessDef(ProcessDef),
    TopLevelProcess(Process),
    EquivLemma(Process, Process),
    DiffEquivLemma(Process),
    Export { tag: String, body: String },
    FormalComment { header: String, body: String },
    Define(String),
    Include(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub name: String,
    pub arg_types: Vec<Option<String>>,
    pub out_type: Option<String>,
    pub private: bool,
    pub destructor: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Equation {
    pub lhs: Term,
    pub rhs: Term,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Macro {
    pub name: String,
    pub args: Vec<VarSpec>,
    pub body: Term,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Predicate {
    pub fact: Fact,
    pub formula: Formula,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Restriction {
    pub name: String,
    pub formula: Formula,
    pub attributes: Vec<RestrictionAttr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RestrictionAttr {
    LeftRestriction,
    RightRestriction,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rule {
    pub name: String,
    pub modulo: Option<String>,
    pub attributes: Vec<RuleAttr>,
    pub let_block: Vec<LetBinding>,
    pub premises: Vec<Fact>,
    pub actions: Vec<Fact>,
    pub conclusions: Vec<Fact>,
    pub embedded_restrictions: Vec<Formula>,
    pub variants: Vec<Rule>,
    pub left_right: Option<(Box<Rule>, Box<Rule>)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuleAttr {
    Color(String),
    NoDerivCheck,
    Role(String),
    IsSapicRule,
    Process(String),
    External(String, Option<String>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LetBinding {
    pub var: Term,
    pub value: Term,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Lemma {
    pub name: String,
    pub modulo: Option<String>,
    pub attributes: Vec<LemmaAttr>,
    pub trace_quantifier: TraceQuantifier,
    pub formula: Formula,
    pub proof: Option<ProofSkeleton>,
    pub plaintext: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DiffLemma {
    pub name: String,
    pub attributes: Vec<LemmaAttr>,
    pub proof: Option<ProofSkeleton>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AccLemma {
    pub name: String,
    pub attributes: Vec<LemmaAttr>,
    pub formula: Formula,
    pub case_test_idents: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CaseTest {
    pub name: String,
    pub formula: Formula,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TraceQuantifier {
    AllTraces,
    ExistsTrace,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LemmaAttr {
    Sources,
    Reuse,
    DiffReuse,
    UseInduction,
    HideLemma(String),
    Heuristic(String),
    Output(Vec<String>),
    Left,
    Right,
    Hint(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProofSkeleton {
    pub raw: String,
    pub tree: Option<ParsedProofTree>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParsedProofTree {
    pub method: ParsedMethod,
    pub cases: Vec<(String, ParsedProofTree)>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsedMethod {
    Sorry,
    Contradiction,
    Simplify,
    Induction,
    SolveGoal(GoalSpec, String),
    SolvedLeaf,
    Unfinishable,
    Invalidated,
    Other(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum GoalSpec {
    Action {
        fact: Fact,
        time_var: String,
        time_idx: u32,
    },
    Premise {
        fact: Fact,
        prem_idx: usize,
        time_var: String,
        time_idx: u32,
    },
    Disj { alts: Vec<DisjAlt>, alt_texts: Vec<String> },
    Chain {
        src_var: String,
        conc_idx: u32,
        tgt_var: String,
        prem_idx: u32,
    },
    Subterm { small_raw: String, big_raw: String },
    Split { split_id: i64 },
    Raw(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DisjAlt {
    All { n_vars: usize },
    Ex { n_vars: usize },
    NonQuant,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tactic {
    pub name: String,
    pub raw: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessDef {
    pub name: String,
    pub vars: Option<Vec<VarSpec>>,
    pub body: Process,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Process {
    Null,
    Action {
        action: SapicAction,
        body: Box<Process>,
    },
    Comb {
        comb: ProcessComb,
        left: Box<Process>,
        right: Box<Process>,
    },
    Replication(Box<Process>),
    Call { name: String, args: Vec<Term> },
    AtAnnotation(Box<Process>, Term),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SapicAction {
    New(VarSpec),
    Insert(Term, Term),
    Delete(Term),
    ChIn { chan: Option<Term>, msg: Term },
    ChOut { chan: Option<Term>, msg: Term },
    Lock(Term),
    Unlock(Term),
    Event(Fact),
    Msr { prems: Vec<Fact>, acts: Vec<Fact>, concs: Vec<Fact>, restrictions: Vec<Formula> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum ProcessComb {
    Parallel,
    Ndc,
    Cond(Condition),
    Lookup(Term, VarSpec),
    Let { pat: Term, value: Term },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Condition {
    Eq(Term, Term),
    Formula(Formula),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fact {
    pub persistent: bool,
    pub name: String,
    pub args: Vec<Term>,
    pub annotations: Vec<FactAnnotation>,
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum FactAnnotation {
    SolveFirst,
    SolveLast,
    NoSources,
}

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
    LessMset(Term, Term),
    Subterm(Term, Term),
    Action(Fact, Term),
    Last(Term),
    Pred(Fact),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Term {
    Var(VarSpec),
    PubLit(String),
    FreshLit(String),
    NatLit(String),
    Number(u64),
    NumberOne,
    NatOne,
    DhNeutral,
    App(String, Vec<Term>),
    AlgApp(String, Box<Term>, Box<Term>),
    Pair(Vec<Term>),
    Diff(Box<Term>, Box<Term>),
    BinOp(BinOp, Box<Term>, Box<Term>),
    PatMatch(Box<Term>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    Exp,
    Mult,
    Union,
    Xor,
    NatPlus,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VarSpec {
    pub name: String,
    pub idx: u64,
    pub sort: SortHint,
    pub typ: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SortHint {
    Msg,
    Pub,
    Fresh,
    Node,
    Nat,
    Suffix(SuffixSort),
    #[default]
    Untagged,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SuffixSort { Msg, Pub, Fresh, Node, Nat }

#[derive(Debug, Clone, PartialEq)]
pub enum FlagFormula {
    Atom(String),
    Not(Box<FlagFormula>),
    And(Box<FlagFormula>, Box<FlagFormula>),
    Or(Box<FlagFormula>, Box<FlagFormula>),
}
