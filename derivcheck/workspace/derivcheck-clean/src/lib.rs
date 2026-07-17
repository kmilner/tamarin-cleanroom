//! Unit G — Message Derivation Checks (clean-room reimplementation).
//!
//! For each protocol rule, the prover decides whether every variable of the rule
//! is derivable by the intruder from the rule's premises. Variables that are not
//! derivable are reported under the wellformedness topic `Message Derivation
//! Checks`. The derivability decision is the prover's solver and is NOT
//! reimplemented here: it is supplied by the caller through [`DerivabilitySolver`].
//!
//! This crate owns four things, all characterized purely by black-box observation
//! of the reference tool (see workspace/BEHAVIOR.md and QUERIES.log):
//!   1. probe construction — which rules and which variables are handed to the
//!      solver, computed on the macro/let-EXPANDED rule (rule filtering, macro and
//!      let expansion, candidate-variable enumeration);
//!   2. the decision logic around solver outcomes (deactivation, timeouts,
//!      derivable/not-derivable);
//!   3. byte-exact report text and variable ordering;
//!   4. the batched solver interface: one saturation question per rule, carrying
//!      all of that rule's candidate variables.

pub mod ast;

use std::collections::HashMap;
use std::collections::{BTreeSet, HashSet};

use ast::{Fact, Macro, Rule, RuleAttr, SortHint, Term, Theory, TheoryItem, VarSpec};

/// A single wellformedness finding: a topic and its message body. Mirrors the
/// `WfError` surface of the wellformedness unit so findings compose into the
/// shared report model. For this topic the `message` is the COMPLETE block,
/// heading included (see [`message_derivation_checks`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WfError {
    pub topic: String,
    pub message: String,
}

impl WfError {
    pub fn new(topic: impl Into<String>, message: impl Into<String>) -> Self {
        WfError { topic: topic.into(), message: message.into() }
    }
}

/// A flat report, in the order findings are rendered.
pub type WfReport = Vec<WfError>;

/// The exact topic string the reference tool prints for this check.
pub const DERIVATION_TOPIC: &str = "Message Derivation Checks";

/// The exact introductory paragraph the reference tool prints once, before the
/// per-rule blocks. Two leading spaces; no trailing whitespace.
pub const DERIVATION_INTRO: &str =
    "  The variables of the following rule(s) are not derivable from their premises, you may be performing unintended pattern matching.";

// --------------------------------------------------------------------------
// Caller-supplied decision callback
// --------------------------------------------------------------------------

/// The solver's verdict for one candidate variable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Derivability {
    /// The intruder can derive the variable from the premises.
    Derivable,
    /// The intruder cannot derive the variable — it will be reported.
    NotDerivable,
    /// The derivation search exceeded the timeout budget without deciding.
    TimedOut,
}

/// A batched derivability question: given one rule and ALL of its candidate
/// variables, decide each. The consuming solver saturates the rule's message
/// deduction once and answers every variable from that single saturation, so the
/// batched shape avoids the wasteful re-saturation a per-variable interface would
/// force. `variables` is the full candidate list for the rule; the returned
/// vector carries one verdict per entry, in the same order.
pub struct RuleProbe<'a> {
    pub rule_name: &'a str,
    pub rule: &'a Rule,
    pub premises: &'a [Fact],
    pub variables: &'a [VarSpec],
    pub timeout_secs: u64,
}

/// The batched decision callback this crate is parameterized over. In production
/// it wraps the prover's message-deduction solver (one saturation per rule); in
/// tests it is stubbed. A per-variable callback can be adapted to it via
/// [`PerVariable`].
pub trait DerivabilitySolver {
    /// Decide derivability for every variable in `probe.variables`, returning one
    /// verdict per variable in the same order. Implementations must return a
    /// vector of length `probe.variables.len()`.
    fn check_rule(&self, probe: &RuleProbe) -> Vec<Derivability>;
}

/// One derivability question for a single variable. This is the payload of the
/// thin per-variable adapter [`PerVariable`]; the full `rule` is provided so a
/// solver that needs more context than the premise facts can use it.
pub struct DerivProbe<'a> {
    pub rule_name: &'a str,
    pub rule: &'a Rule,
    pub variable: &'a VarSpec,
    pub premises: &'a [Fact],
    pub timeout_secs: u64,
}

/// A per-variable decision callback. Kept as a convenience for callers whose
/// solver answers one variable at a time; wrap it in [`PerVariable`] to obtain a
/// [`DerivabilitySolver`]. A production solver should implement
/// [`DerivabilitySolver`] directly to saturate each rule only once.
pub trait PerVariableSolver {
    fn check(&self, probe: &DerivProbe) -> Derivability;
}

/// Thin adapter turning a [`PerVariableSolver`] into a batched
/// [`DerivabilitySolver`] by calling it once per variable. Convenient but does
/// not share saturation across a rule's variables.
pub struct PerVariable<S>(pub S);

impl<S: PerVariableSolver> DerivabilitySolver for PerVariable<S> {
    fn check_rule(&self, probe: &RuleProbe) -> Vec<Derivability> {
        probe
            .variables
            .iter()
            .map(|v| {
                self.0.check(&DerivProbe {
                    rule_name: probe.rule_name,
                    rule: probe.rule,
                    variable: v,
                    premises: probe.premises,
                    timeout_secs: probe.timeout_secs,
                })
            })
            .collect()
    }
}

/// How a per-rule timeout is treated. See the residual note in BEHAVIOR.md §7:
/// a real per-rule timeout was not observable, so the policy is explicit here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutPolicy {
    /// Suppress the whole rule's warning when any of its variables times out
    /// (fail-open; the chosen default, consistent with `timeout=0` suppressing
    /// all output).
    SuppressRule,
    /// Treat a timed-out variable like `Derivable` (skip only that variable) and
    /// still report the variables that were definitively not derivable.
    SkipVariable,
}

impl Default for TimeoutPolicy {
    fn default() -> Self {
        TimeoutPolicy::SuppressRule
    }
}

// --------------------------------------------------------------------------
// Entry points
// --------------------------------------------------------------------------

/// Run the message-derivation check over a theory with the default timeout
/// policy. `timeout_secs` mirrors `--derivcheck-timeout`: `0` deactivates the
/// check and returns an empty report without consulting the solver.
///
/// The returned report is either empty (no failing rule) or a single [`WfError`]
/// whose `message` is the complete topic block — the underlined
/// `Message Derivation Checks` heading, the intro paragraph, and one block per
/// failing rule — byte-exact as the reference prints it. The consuming report
/// renderer adds no per-topic heading of its own, so the heading lives here.
pub fn message_derivation_checks<S: DerivabilitySolver>(
    thy: &Theory,
    solver: &S,
    timeout_secs: u64,
) -> WfReport {
    message_derivation_checks_with(thy, solver, timeout_secs, TimeoutPolicy::default())
}

/// As [`message_derivation_checks`], with an explicit timeout policy.
pub fn message_derivation_checks_with<S: DerivabilitySolver>(
    thy: &Theory,
    solver: &S,
    timeout_secs: u64,
    timeout_policy: TimeoutPolicy,
) -> WfReport {
    // `--derivcheck-timeout=0` deactivates the check entirely.
    if timeout_secs == 0 {
        return Vec::new();
    }

    let macros = macro_table(thy);

    // Per-rule findings, kept in theory (source) order.
    let mut blocks: Vec<(String, Vec<VarSpec>)> = Vec::new();

    for item in &thy.items {
        let rule = match item {
            TheoryItem::Rule(rule) => rule,
            _ => continue, // only protocol rules; intruder rules are out of scope
        };
        // A rule tagged [no_derivcheck] is skipped.
        if rule.attributes.iter().any(|a| matches!(a, RuleAttr::NoDerivCheck)) {
            continue;
        }

        // Derivability is decided on the macro/let-expanded rule; candidates are
        // that rule's free variables.
        let expanded = expand_rule(rule, &macros);
        let candidates = candidate_variables(&expanded);
        if candidates.is_empty() {
            continue;
        }

        let probe = RuleProbe {
            rule_name: &expanded.name,
            rule: &expanded,
            premises: &expanded.premises,
            variables: &candidates,
            timeout_secs,
        };
        let verdicts = solver.check_rule(&probe);

        let mut flagged: Vec<VarSpec> = Vec::new();
        let mut rule_timed_out = false;
        for (var, verdict) in candidates.iter().zip(verdicts.iter().copied()) {
            match verdict {
                Derivability::Derivable => {}
                Derivability::NotDerivable => flagged.push(var.clone()),
                Derivability::TimedOut => match timeout_policy {
                    TimeoutPolicy::SuppressRule => {
                        rule_timed_out = true;
                        break;
                    }
                    TimeoutPolicy::SkipVariable => {}
                },
            }
        }

        if rule_timed_out {
            continue;
        }
        if !flagged.is_empty() {
            sort_variables(&mut flagged);
            blocks.push((expanded.name.clone(), flagged));
        }
    }

    if blocks.is_empty() {
        return Vec::new();
    }

    // The whole topic is one value: heading + intro + one block per failing rule.
    vec![WfError::new(DERIVATION_TOPIC, render_block(&blocks))]
}

// --------------------------------------------------------------------------
// Macro / let expansion (derivability is decided post-expansion)
// --------------------------------------------------------------------------

/// The theory's `macros:` definitions, keyed by macro name.
fn macro_table(thy: &Theory) -> HashMap<String, &Macro> {
    let mut table = HashMap::new();
    for item in &thy.items {
        if let TheoryItem::Macros(defs) = item {
            for m in defs {
                table.insert(m.name.clone(), m);
            }
        }
    }
    table
}

/// A rule with macro applications and `let` bindings fully substituted into its
/// premise/action/conclusion terms and its `let_block` cleared. Candidate
/// enumeration and the solver both operate on this expanded form, matching the
/// reference's post-expansion decision (BEHAVIOR.md §9).
fn expand_rule(rule: &Rule, macros: &HashMap<String, &Macro>) -> Rule {
    let lets = let_substitution(rule, macros);
    let map_facts = |facts: &[Fact]| -> Vec<Fact> {
        facts
            .iter()
            .map(|f| Fact {
                args: f.args.iter().map(|a| expand_term(a, macros, &lets)).collect(),
                ..f.clone()
            })
            .collect()
    };
    Rule {
        premises: map_facts(&rule.premises),
        actions: map_facts(&rule.actions),
        conclusions: map_facts(&rule.conclusions),
        let_block: Vec::new(),
        ..rule.clone()
    }
}

/// Build the substitution induced by a rule's `let` bindings. Bindings are
/// sequential: each value is expanded against the macros and the bindings seen so
/// far, so a later `let` may reference an earlier one. A `let <a, b> = <..>`
/// pattern binding destructures componentwise when both sides are tuples of the
/// same arity.
fn let_substitution(rule: &Rule, macros: &HashMap<String, &Macro>) -> HashMap<VarSpec, Term> {
    let mut lets: HashMap<VarSpec, Term> = HashMap::new();
    for lb in &rule.let_block {
        let value = expand_term(&lb.value, macros, &lets);
        bind_pattern(&lb.var, &value, &mut lets);
    }
    lets
}

/// Insert the binding(s) that make `pattern := value` hold. A bare variable binds
/// directly; a tuple pattern binds componentwise against a tuple value.
fn bind_pattern(pattern: &Term, value: &Term, lets: &mut HashMap<VarSpec, Term>) {
    match (pattern, value) {
        (Term::Var(v), _) => {
            lets.insert(v.clone(), value.clone());
        }
        (Term::Pair(ps), Term::Pair(vs)) if ps.len() == vs.len() => {
            for (p, v) in ps.iter().zip(vs.iter()) {
                bind_pattern(p, v, lets);
            }
        }
        _ => {} // unsupported pattern shape: leave its variables unexpanded
    }
}

/// Expand a term: apply the `let` substitution to variables and rewrite macro
/// applications, recursively. `lets` values are already fully expanded.
fn expand_term(t: &Term, macros: &HashMap<String, &Macro>, lets: &HashMap<VarSpec, Term>) -> Term {
    match t {
        Term::Var(v) => lets.get(v).cloned().unwrap_or_else(|| t.clone()),
        Term::App(name, args) => {
            let eargs: Vec<Term> = args.iter().map(|a| expand_term(a, macros, lets)).collect();
            if let Some(m) = macros.get(name) {
                if m.args.len() == eargs.len() {
                    let param_map: HashMap<VarSpec, Term> =
                        m.args.iter().cloned().zip(eargs.iter().cloned()).collect();
                    let body = subst_vars(&m.body, &param_map);
                    return expand_term(&body, macros, lets);
                }
            }
            Term::App(name.clone(), eargs)
        }
        Term::Pair(args) => Term::Pair(args.iter().map(|a| expand_term(a, macros, lets)).collect()),
        Term::AlgApp(op, a, b) => Term::AlgApp(
            op.clone(),
            Box::new(expand_term(a, macros, lets)),
            Box::new(expand_term(b, macros, lets)),
        ),
        Term::BinOp(op, a, b) => Term::BinOp(
            *op,
            Box::new(expand_term(a, macros, lets)),
            Box::new(expand_term(b, macros, lets)),
        ),
        Term::Diff(a, b) => Term::Diff(
            Box::new(expand_term(a, macros, lets)),
            Box::new(expand_term(b, macros, lets)),
        ),
        Term::PatMatch(a) => Term::PatMatch(Box::new(expand_term(a, macros, lets))),
        _ => t.clone(),
    }
}

/// Substitute variables by `map` throughout a term (used to instantiate a macro
/// body with its actual arguments).
fn subst_vars(t: &Term, map: &HashMap<VarSpec, Term>) -> Term {
    match t {
        Term::Var(v) => map.get(v).cloned().unwrap_or_else(|| t.clone()),
        Term::App(name, args) => {
            Term::App(name.clone(), args.iter().map(|a| subst_vars(a, map)).collect())
        }
        Term::Pair(args) => Term::Pair(args.iter().map(|a| subst_vars(a, map)).collect()),
        Term::AlgApp(op, a, b) => Term::AlgApp(
            op.clone(),
            Box::new(subst_vars(a, map)),
            Box::new(subst_vars(b, map)),
        ),
        Term::BinOp(op, a, b) => {
            Term::BinOp(*op, Box::new(subst_vars(a, map)), Box::new(subst_vars(b, map)))
        }
        Term::Diff(a, b) => Term::Diff(Box::new(subst_vars(a, map)), Box::new(subst_vars(b, map))),
        Term::PatMatch(a) => Term::PatMatch(Box::new(subst_vars(a, map))),
        _ => t.clone(),
    }
}

// --------------------------------------------------------------------------
// Probe construction
// --------------------------------------------------------------------------

/// The candidate variables handed to the solver for one rule: every distinct
/// variable occurring in the rule's premises, actions, and conclusions, in
/// first-occurrence order (order is irrelevant — the flagged subset is re-sorted
/// for display). The rule's own `let` bindings are expanded first, so a
/// let-bound name (the binding's left-hand side) is not itself a candidate; only
/// the variables of the substituted terms are. Full macro expansion additionally
/// requires the theory context and is applied by [`expand_rule`] before this
/// function is called; on an already-expanded rule this only re-applies the (now
/// empty) `let` substitution.
///
/// The solver decides which candidates are actually derivable, so this set is a
/// safe superset: public names, `Fr`-bound fresh names, and state-fact-bound
/// variables are all resolved to `Derivable` by the solver. A nullary function
/// (a declared constant) parses to `Term::App(name, [])` and contributes no
/// variable, so a name colliding with such a constant is never a candidate.
pub fn candidate_variables(rule: &Rule) -> Vec<VarSpec> {
    let lets = let_substitution(rule, &HashMap::new());
    let mut out: Vec<VarSpec> = Vec::new();
    let mut seen: HashSet<VarSpec> = HashSet::new();
    let mut push = |t: &Term| {
        let expanded = expand_term(t, &HashMap::new(), &lets);
        collect_vars(&expanded, &mut |v| {
            if seen.insert(v.clone()) {
                out.push(v.clone());
            }
        });
    };
    for f in rule.premises.iter().chain(&rule.actions).chain(&rule.conclusions) {
        for a in &f.args {
            push(a);
        }
    }
    out
}

/// Walk a term, invoking `f` on every variable occurrence.
fn collect_vars(term: &Term, f: &mut impl FnMut(&VarSpec)) {
    match term {
        Term::Var(v) => f(v),
        Term::PubLit(_)
        | Term::FreshLit(_)
        | Term::NatLit(_)
        | Term::Number(_)
        | Term::NumberOne
        | Term::NatOne
        | Term::DhNeutral => {}
        Term::App(_, args) | Term::Pair(args) => {
            for a in args {
                collect_vars(a, f);
            }
        }
        Term::AlgApp(_, a, b) | Term::Diff(a, b) | Term::BinOp(_, a, b) => {
            collect_vars(a, f);
            collect_vars(b, f);
        }
        Term::PatMatch(a) => collect_vars(a, f),
    }
}

// --------------------------------------------------------------------------
// Variable ordering and rendering
// --------------------------------------------------------------------------

/// Sort flagged variables into report order and drop duplicates. The order is
/// ascending by `(index numerically, sort-rank, name byte-lexicographically)`.
pub fn sort_variables(vars: &mut Vec<VarSpec>) {
    vars.sort_by(var_order_key);
    vars.dedup();
}

/// Total order used to sort reported variables (see BEHAVIOR.md §6): index is the
/// primary key (numeric), then sort tag (`Fresh < Msg < Nat`), then name (byte /
/// ASCII lexicographic, so uppercase precedes lowercase).
fn var_order_key(a: &VarSpec, b: &VarSpec) -> std::cmp::Ordering {
    a.idx
        .cmp(&b.idx)
        .then_with(|| sort_rank(a.sort).cmp(&sort_rank(b.sort)))
        .then_with(|| a.name.cmp(&b.name))
}

/// Sort-tag rank among reportable variables: `Fresh < Msg < Nat`. Message-sort
/// variables carry either `Msg` or `Untagged` and rank together. `Pub` never
/// appears (public names are always derivable); `Node`/`Suffix` are not
/// expressible as message terms, so their relative rank is unobserved and placed
/// last.
fn sort_rank(sort: SortHint) -> u8 {
    match sort {
        SortHint::Fresh => 0,
        SortHint::Msg | SortHint::Untagged => 1,
        SortHint::Nat => 2,
        _ => 3,
    }
}

/// Render one variable as the reference tool spells it in the warning: fresh
/// names carry a `~` prefix, natural-number names a `%` prefix, public names a
/// `$` prefix, message names none; a nonzero index is appended as `.idx`
/// (e.g. `x.2`).
pub fn render_variable(v: &VarSpec) -> String {
    let prefix = match v.sort {
        SortHint::Fresh => "~",
        SortHint::Nat => "%",
        SortHint::Pub => "$",
        _ => "",
    };
    if v.idx == 0 {
        format!("{}{}", prefix, v.name)
    } else {
        format!("{}{}.{}", prefix, v.name, v.idx)
    }
}

/// The per-rule block body: `"Rule <name>: \nFailed to derive Variable(s): <v1>, <v2>, ..."`.
/// Note the trailing space after the colon on the first line.
fn rule_block(rule_name: &str, vars: &[VarSpec]) -> String {
    let list = vars.iter().map(render_variable).collect::<Vec<_>>().join(", ");
    format!("Rule {}: \nFailed to derive Variable(s): {}", rule_name, list)
}

/// Assemble the complete topic block for the failing rules: the underlined
/// heading, a blank line, the intro paragraph, then one block per rule, all
/// separated by a single blank line. No trailing newline — the consuming report
/// renderer joins topic blocks with a blank line of its own.
fn render_block(blocks: &[(String, Vec<VarSpec>)]) -> String {
    let mut segments: Vec<String> = Vec::with_capacity(blocks.len() + 1);
    segments.push(DERIVATION_INTRO.to_string());
    for (name, vars) in blocks {
        segments.push(rule_block(name, vars));
    }
    format!("{}\n{}", underline_topic(DERIVATION_TOPIC), segments.join("\n\n"))
}

// --------------------------------------------------------------------------
// Rendering helpers
// --------------------------------------------------------------------------

/// A topic header exactly as the reference tool renders it: the title, then a
/// newline, then a run of `=` as long as the title, then a newline.
pub fn underline_topic(title: &str) -> String {
    let bar: String = "=".repeat(title.chars().count());
    format!("{}\n{}\n", title, bar)
}

/// The distinct topics present in a report.
pub fn topics(report: &WfReport) -> BTreeSet<String> {
    report.iter().map(|e| e.topic.clone()).collect()
}

/// Render the `Message Derivation Checks` topic block byte-exactly. The report
/// carries the whole topic as a single [`WfError`] whose `message` already holds
/// the heading, so this returns that message directly (empty string when absent).
pub fn render_derivation_report(report: &WfReport) -> String {
    report
        .iter()
        .find(|e| e.topic == DERIVATION_TOPIC)
        .map(|e| e.message.clone())
        .unwrap_or_default()
}

// --------------------------------------------------------------------------
// Tests (solver stubbed)
// --------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use ast::*;

    // ---- construction helpers ----

    fn msg_var(name: &str) -> VarSpec {
        VarSpec { name: name.to_string(), idx: 0, sort: SortHint::Msg, typ: None }
    }
    fn msg_var_idx(name: &str, idx: u64) -> VarSpec {
        VarSpec { name: name.to_string(), idx, sort: SortHint::Msg, typ: None }
    }
    fn fresh_var(name: &str) -> VarSpec {
        VarSpec { name: name.to_string(), idx: 0, sort: SortHint::Fresh, typ: None }
    }
    fn fresh_var_idx(name: &str, idx: u64) -> VarSpec {
        VarSpec { name: name.to_string(), idx, sort: SortHint::Fresh, typ: None }
    }
    fn pub_var(name: &str) -> VarSpec {
        VarSpec { name: name.to_string(), idx: 0, sort: SortHint::Pub, typ: None }
    }
    fn nat_var(name: &str) -> VarSpec {
        VarSpec { name: name.to_string(), idx: 0, sort: SortHint::Nat, typ: None }
    }
    fn nat_var_idx(name: &str, idx: u64) -> VarSpec {
        VarSpec { name: name.to_string(), idx, sort: SortHint::Nat, typ: None }
    }

    fn fact(name: &str, args: Vec<Term>) -> Fact {
        Fact { persistent: false, name: name.to_string(), args, annotations: vec![] }
    }
    fn app(name: &str, args: Vec<Term>) -> Term {
        Term::App(name.to_string(), args)
    }
    fn v(name: &str) -> Term {
        Term::Var(msg_var(name))
    }

    fn rule(name: &str, premises: Vec<Fact>, conclusions: Vec<Fact>, attrs: Vec<RuleAttr>) -> Rule {
        rule_full(name, vec![], premises, vec![], conclusions, attrs)
    }

    fn rule_full(
        name: &str,
        let_block: Vec<LetBinding>,
        premises: Vec<Fact>,
        actions: Vec<Fact>,
        conclusions: Vec<Fact>,
        attrs: Vec<RuleAttr>,
    ) -> Rule {
        Rule {
            name: name.to_string(),
            modulo: None,
            attributes: attrs,
            let_block,
            premises,
            actions,
            conclusions,
            embedded_restrictions: vec![],
            variants: vec![],
            left_right: None,
        }
    }

    fn theory(items: Vec<TheoryItem>) -> Theory {
        Theory { is_diff: false, name: "T".into(), configuration: None, items }
    }

    // A batched stub solver: flags exactly the variables whose name is in its set
    // as NotDerivable; everything else Derivable.
    struct FlagByName {
        not_derivable: HashSet<String>,
    }
    impl FlagByName {
        fn new(names: &[&str]) -> Self {
            FlagByName { not_derivable: names.iter().map(|s| s.to_string()).collect() }
        }
    }
    impl DerivabilitySolver for FlagByName {
        fn check_rule(&self, probe: &RuleProbe) -> Vec<Derivability> {
            probe
                .variables
                .iter()
                .map(|var| {
                    if self.not_derivable.contains(&var.name) {
                        Derivability::NotDerivable
                    } else {
                        Derivability::Derivable
                    }
                })
                .collect()
        }
    }

    // A per-variable stub, exercised through the PerVariable adapter, to prove the
    // thin wrapper preserves behavior.
    struct FlagByNamePerVar {
        not_derivable: HashSet<String>,
    }
    impl PerVariableSolver for FlagByNamePerVar {
        fn check(&self, probe: &DerivProbe) -> Derivability {
            if self.not_derivable.contains(&probe.variable.name) {
                Derivability::NotDerivable
            } else {
                Derivability::Derivable
            }
        }
    }

    // A batched stub that always times out.
    struct AlwaysTimeout;
    impl DerivabilitySolver for AlwaysTimeout {
        fn check_rule(&self, probe: &RuleProbe) -> Vec<Derivability> {
            probe.variables.iter().map(|_| Derivability::TimedOut).collect()
        }
    }

    // A batched stub that panics if consulted (to prove short-circuiting).
    struct NeverCalled;
    impl DerivabilitySolver for NeverCalled {
        fn check_rule(&self, _: &RuleProbe) -> Vec<Derivability> {
            panic!("solver must not be consulted");
        }
    }

    // ---- byte-exact output (matches captured oracle text) ----

    #[test]
    fn single_rule_block_is_byte_exact_and_one_value() {
        // Rule R: [ In(h(w)) ] --> [ Out('ok') ]; solver flags w.
        let r = rule(
            "R",
            vec![fact("In", vec![app("h", vec![v("w")])])],
            vec![fact("Out", vec![Term::PubLit("ok".into())])],
            vec![],
        );
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["w"]), 5);

        // GAP1: the whole topic is exactly ONE error value, heading included.
        assert_eq!(report.len(), 1);
        assert_eq!(report[0].topic, DERIVATION_TOPIC);
        let expected = concat!(
            "Message Derivation Checks\n",
            "=========================\n",
            "\n",
            "  The variables of the following rule(s) are not derivable from their premises, you may be performing unintended pattern matching.\n",
            "\n",
            "Rule R: \n",
            "Failed to derive Variable(s): w",
        );
        assert_eq!(report[0].message, expected);
        assert_eq!(render_derivation_report(&report), expected);
        assert_eq!(topics(&report), [DERIVATION_TOPIC.to_string()].into_iter().collect());
    }

    #[test]
    fn two_rule_block_matches_poidc_cmb_fixture() {
        // reSign: In(<sk1,r1>), In(sign(m,r2,sk2)) --> Out(sign(m,r1,sk1))
        let re_sign = rule(
            "reSign",
            vec![
                fact("In", vec![Term::Pair(vec![v("sk1"), v("r1")])]),
                fact("In", vec![app("sign", vec![v("m"), v("r2"), v("sk2")])]),
            ],
            vec![fact("Out", vec![app("sign", vec![v("m"), v("r1"), v("sk1")])])],
            vec![],
        );
        // RP_gets_idToken: premise mentions pkA (only pkA is flagged).
        let rp = rule(
            "RP_gets_idToken",
            vec![fact("In", vec![app("raenc", vec![v("x"), v("rndA"), v("pkA")])])],
            vec![fact("Out", vec![Term::PubLit("ok".into())])],
            vec![],
        );
        let thy = theory(vec![TheoryItem::Rule(re_sign), TheoryItem::Rule(rp)]);
        let solver = FlagByName::new(&["m", "r2", "sk2", "pkA"]);
        let report = message_derivation_checks(&thy, &solver, 5);

        assert_eq!(report.len(), 1);
        // The expected string is the captured reference block (probes4/fixture_two_rule.txt,
        // trailing newline removed).
        let expected = concat!(
            "Message Derivation Checks\n",
            "=========================\n",
            "\n",
            "  The variables of the following rule(s) are not derivable from their premises, you may be performing unintended pattern matching.\n",
            "\n",
            "Rule reSign: \n",
            "Failed to derive Variable(s): m, r2, sk2\n",
            "\n",
            "Rule RP_gets_idToken: \n",
            "Failed to derive Variable(s): pkA",
        );
        assert_eq!(report[0].message, expected);
        assert_eq!(report[0].message.len(), 297);
    }

    // ---- ordering (GAP2): index primary, sort (Fresh<Msg<Nat), name ASCII ----

    #[test]
    fn ordering_index_is_primary_over_sort() {
        // g2_freshidx: In(h(~a.2)), In(h(b)) -> "b, ~a.2" (idx0 b before idx2 ~a.2).
        let r = rule(
            "R",
            vec![
                fact("In", vec![app("h", vec![Term::Var(fresh_var_idx("a", 2))])]),
                fact("In", vec![app("h", vec![v("b")])]),
            ],
            vec![],
            vec![],
        );
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["a", "b"]), 5);
        assert!(render_derivation_report(&report).ends_with("Variable(s): b, ~a.2"));
    }

    #[test]
    fn ordering_full_key_matches_g2_strong() {
        // g2_strong: In(h(<z.1, a.2, ~z.2, ~a.1, m>)) -> "m, ~a.1, z.1, ~z.2, a.2".
        let vars = vec![
            Term::Var(msg_var_idx("z", 1)),
            Term::Var(msg_var_idx("a", 2)),
            Term::Var(fresh_var_idx("z", 2)),
            Term::Var(fresh_var_idx("a", 1)),
            v("m"),
        ];
        let r = rule("R", vec![fact("In", vec![app("h", vec![Term::Pair(vars)])])], vec![], vec![]);
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let solver = FlagByName::new(&["z", "a", "m"]);
        let report = message_derivation_checks(&thy, &solver, 5);
        assert!(
            render_derivation_report(&report)
                .ends_with("Variable(s): m, ~a.1, z.1, ~z.2, a.2"),
            "got: {}",
            render_derivation_report(&report)
        );
    }

    #[test]
    fn ordering_name_is_ascii_case_sensitive() {
        // g2_case: In(h(<apple, Zebra>)) -> "Zebra, apple" (uppercase 'Z' < 'a').
        let vars = vec![v("apple"), v("Zebra")];
        let r = rule("R", vec![fact("In", vec![app("h", vec![Term::Pair(vars)])])], vec![], vec![]);
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["apple", "Zebra"]), 5);
        assert!(render_derivation_report(&report).ends_with("Variable(s): Zebra, apple"));
    }

    #[test]
    fn ordering_nat_sort_ranks_after_message() {
        // g3_nat_order: In(h(<%b, a, ~c>)) -> "~c, a, %b" (Fresh < Msg < Nat at idx0).
        let vars = vec![Term::Var(nat_var("b")), v("a"), Term::Var(fresh_var("c"))];
        let r = rule("R", vec![fact("In", vec![app("h", vec![Term::Pair(vars)])])], vec![], vec![]);
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["a", "b", "c"]), 5);
        assert!(render_derivation_report(&report).ends_with("Variable(s): ~c, a, %b"));
    }

    #[test]
    fn nat_variable_renders_with_percent() {
        assert_eq!(render_variable(&nat_var("x")), "%x");
        assert_eq!(render_variable(&nat_var_idx("x", 2)), "%x.2");
    }

    #[test]
    fn duplicate_variable_listed_once() {
        let r = rule(
            "R",
            vec![
                fact("In", vec![app("h", vec![v("w")])]),
                fact("In", vec![app("h", vec![v("w")])]),
            ],
            vec![],
            vec![],
        );
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["w"]), 5);
        assert!(render_derivation_report(&report).ends_with("Variable(s): w"));
    }

    // ---- candidate scope (GAP3) ----

    #[test]
    fn action_only_variable_is_a_candidate() {
        // g3_act: [Fr(~n)] --[Ev(z)]-> [Out(~n)] ; z occurs only in the action.
        let r = rule_full(
            "R",
            vec![],
            vec![fact("Fr", vec![Term::Var(fresh_var("n"))])],
            vec![fact("Ev", vec![v("z")])],
            vec![fact("Out", vec![Term::Var(fresh_var("n"))])],
            vec![],
        );
        let cands = candidate_variables(&r);
        assert!(cands.iter().any(|c| c.name == "z"), "action var z must be a candidate");
        // And it is reported when the solver flags it.
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["z"]), 5);
        assert!(render_derivation_report(&report).ends_with("Variable(s): z"));
    }

    #[test]
    fn conclusion_only_variable_is_a_candidate() {
        // g3_concl: [Fr(~n)] --> [Store(~n, k)] ; k occurs only in the conclusion.
        let r = rule(
            "R",
            vec![fact("Fr", vec![Term::Var(fresh_var("n"))])],
            vec![fact("Store", vec![Term::Var(fresh_var("n")), v("k")])],
            vec![],
        );
        let cands = candidate_variables(&r);
        assert!(cands.iter().any(|c| c.name == "k"), "conclusion var k must be a candidate");
    }

    #[test]
    fn nullary_function_constant_is_not_a_candidate() {
        // g3_nullary: In(h(c)) with c a declared 0-ary function -> App("c", []) -> no variable.
        let r = rule(
            "R",
            vec![fact("In", vec![app("h", vec![app("c", vec![])])])],
            vec![],
            vec![],
        );
        let cands = candidate_variables(&r);
        assert!(cands.is_empty(), "a nullary function is a constant, not a candidate");
    }

    #[test]
    fn public_candidate_is_resolved_by_solver_not_prefiltered() {
        // The unit hands $p to the solver (superset); the solver resolves it Derivable.
        let r = rule("R", vec![fact("In", vec![app("h", vec![Term::Var(pub_var("p"))])])], vec![], vec![]);
        let cands = candidate_variables(&r);
        assert!(cands.iter().any(|c| c.sort == SortHint::Pub && c.name == "p"));
        // A realistic solver marks public derivable -> no warning.
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&[]), 5);
        assert!(report.is_empty());
    }

    // ---- pre-expansion (GAP4) ----

    #[test]
    fn let_binding_is_expanded_before_candidate_enumeration() {
        // g4_let: let y = h(w) in [ In(y) ] -> candidate is w (inner), not y.
        let r = rule_full(
            "R",
            vec![LetBinding { var: v("y"), value: app("h", vec![v("w")]) }],
            vec![fact("In", vec![v("y")])],
            vec![],
            vec![],
            vec![],
        );
        let cands = candidate_variables(&r);
        assert!(cands.iter().any(|c| c.name == "w"), "inner w must be a candidate");
        assert!(!cands.iter().any(|c| c.name == "y"), "let LHS y must not be a candidate");
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["w"]), 5);
        assert!(render_derivation_report(&report).ends_with("Variable(s): w"));
    }

    #[test]
    fn let_rename_reports_post_expansion_inner_name() {
        // g4_let_rename: let y = w in [ In(h(y)) ] -> flags w, never y.
        let r = rule_full(
            "R",
            vec![LetBinding { var: v("y"), value: v("w") }],
            vec![fact("In", vec![app("h", vec![v("y")])])],
            vec![],
            vec![],
            vec![],
        );
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["w", "y"]), 5);
        assert!(render_derivation_report(&report).ends_with("Variable(s): w"));
    }

    #[test]
    fn macro_application_is_expanded_before_the_check() {
        // g4_macro: macros mac(x) = h(x); [ In(mac(w)) ] -> flags w.
        let mac = Macro { name: "mac".into(), args: vec![msg_var("x")], body: app("h", vec![v("x")]) };
        let r = rule("R", vec![fact("In", vec![app("mac", vec![v("w")])])], vec![], vec![]);
        let thy = theory(vec![TheoryItem::Macros(vec![mac]), TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["w", "x"]), 5);
        assert!(render_derivation_report(&report).ends_with("Variable(s): w"));
    }

    #[test]
    fn macro_and_let_expansion_compose() {
        // g4_macro_rename: wrap(a)=h(a); let y=w in [ In(wrap(y)) ] -> flags w.
        let mac = Macro { name: "wrap".into(), args: vec![msg_var("a")], body: app("h", vec![v("a")]) };
        let r = rule_full(
            "R",
            vec![LetBinding { var: v("y"), value: v("w") }],
            vec![fact("In", vec![app("wrap", vec![v("y")])])],
            vec![],
            vec![],
            vec![],
        );
        let thy = theory(vec![TheoryItem::Macros(vec![mac]), TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["w", "y", "a"]), 5);
        assert!(render_derivation_report(&report).ends_with("Variable(s): w"));
    }

    // ---- rule filtering ----

    #[test]
    fn no_derivcheck_attribute_suppresses_rule() {
        let r = rule(
            "R",
            vec![fact("In", vec![app("h", vec![v("x")])])],
            vec![],
            vec![RuleAttr::NoDerivCheck],
        );
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["x"]), 5);
        assert!(report.is_empty());
        assert_eq!(render_derivation_report(&report), "");
    }

    #[test]
    fn derivable_rule_is_omitted_and_theory_order_preserved() {
        let good = rule("Good", vec![fact("In", vec![v("x")])], vec![fact("Out", vec![v("x")])], vec![]);
        let bad = rule("Bad", vec![fact("In", vec![app("h", vec![v("w")])])], vec![], vec![]);
        let thy = theory(vec![TheoryItem::Rule(good), TheoryItem::Rule(bad)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["w"]), 5);
        let rendered = render_derivation_report(&report);
        assert!(rendered.contains("Rule Bad: "));
        assert!(!rendered.contains("Rule Good"));
    }

    #[test]
    fn rules_reported_in_theory_source_order() {
        let zebra = rule("Zebra", vec![fact("In", vec![app("h", vec![v("xz")])])], vec![], vec![]);
        let apple = rule("Apple", vec![fact("In", vec![app("h", vec![v("ya")])])], vec![], vec![]);
        let thy = theory(vec![TheoryItem::Rule(zebra), TheoryItem::Rule(apple)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["xz", "ya"]), 5);
        let rendered = render_derivation_report(&report);
        let zi = rendered.find("Rule Zebra").unwrap();
        let ai = rendered.find("Rule Apple").unwrap();
        assert!(zi < ai, "Zebra must precede Apple: {}", rendered);
    }

    #[test]
    fn intruder_rules_are_ignored() {
        let ir = rule("iknows", vec![fact("In", vec![app("h", vec![v("x")])])], vec![], vec![]);
        let thy = theory(vec![TheoryItem::IntrRule(ir)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["x"]), 5);
        assert!(report.is_empty());
    }

    // ---- batched solver interface (GAP5) ----

    #[test]
    fn solver_is_saturated_once_per_rule_with_all_candidates() {
        use std::cell::RefCell;
        // Records how many times check_rule is called and the batch sizes seen.
        struct Recording {
            calls: RefCell<Vec<usize>>,
        }
        impl DerivabilitySolver for Recording {
            fn check_rule(&self, probe: &RuleProbe) -> Vec<Derivability> {
                self.calls.borrow_mut().push(probe.variables.len());
                probe.variables.iter().map(|_| Derivability::Derivable).collect()
            }
        }
        // One rule with three distinct candidate variables.
        let r = rule(
            "R",
            vec![fact("In", vec![app("h", vec![Term::Pair(vec![v("a"), v("b"), v("c")])])])],
            vec![],
            vec![],
        );
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let rec = Recording { calls: RefCell::new(vec![]) };
        let _ = message_derivation_checks(&thy, &rec, 5);
        let calls = rec.calls.borrow();
        assert_eq!(calls.len(), 1, "solver must be consulted exactly once per rule");
        assert_eq!(calls[0], 3, "the one call carries all three candidate variables");
    }

    #[test]
    fn per_variable_adapter_matches_batched_behavior() {
        let r = rule(
            "R",
            vec![fact("In", vec![app("h", vec![Term::Pair(vec![v("w"), v("q")])])])],
            vec![],
            vec![],
        );
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let per = PerVariable(FlagByNamePerVar {
            not_derivable: ["w".to_string()].into_iter().collect(),
        });
        let report = message_derivation_checks(&thy, &per, 5);
        assert!(render_derivation_report(&report).ends_with("Variable(s): w"));
    }

    // ---- activation / timeout decision logic ----

    #[test]
    fn timeout_zero_deactivates_without_consulting_solver() {
        let r = rule("R", vec![fact("In", vec![app("h", vec![v("x")])])], vec![], vec![]);
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &NeverCalled, 0);
        assert!(report.is_empty());
    }

    #[test]
    fn timeout_suppresses_rule_by_default() {
        let r = rule("R", vec![fact("In", vec![app("h", vec![v("x")])])], vec![], vec![]);
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &AlwaysTimeout, 5);
        assert!(report.is_empty(), "default policy suppresses a timed-out rule");
    }

    #[test]
    fn skip_variable_policy_reports_definite_failures_only() {
        // xx not derivable, yy times out; under SkipVariable only xx is reported.
        struct Mixed;
        impl DerivabilitySolver for Mixed {
            fn check_rule(&self, probe: &RuleProbe) -> Vec<Derivability> {
                probe
                    .variables
                    .iter()
                    .map(|var| match var.name.as_str() {
                        "xx" => Derivability::NotDerivable,
                        "yy" => Derivability::TimedOut,
                        _ => Derivability::Derivable,
                    })
                    .collect()
            }
        }
        let r = rule(
            "R",
            vec![fact("In", vec![app("h", vec![Term::Pair(vec![v("xx"), v("yy")])])])],
            vec![],
            vec![],
        );
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let report =
            message_derivation_checks_with(&thy, &Mixed, 5, TimeoutPolicy::SkipVariable);
        assert!(render_derivation_report(&report).ends_with("Variable(s): xx"));
    }

    // ---- helpers ----

    #[test]
    fn underline_matches_title_length() {
        assert_eq!(
            underline_topic("Message Derivation Checks"),
            "Message Derivation Checks\n=========================\n"
        );
    }

    #[test]
    fn variable_rendering_prefixes_and_index_suffix() {
        assert_eq!(render_variable(&pub_var("A")), "$A");
        assert_eq!(render_variable(&fresh_var("n")), "~n");
        assert_eq!(render_variable(&nat_var("k")), "%k");
        assert_eq!(render_variable(&msg_var("x")), "x");
        assert_eq!(render_variable(&msg_var_idx("x", 2)), "x.2");
    }

    #[test]
    fn empty_theory_produces_no_report() {
        let thy = theory(vec![]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&[]), 5);
        assert!(report.is_empty());
        assert_eq!(render_derivation_report(&report), "");
    }
}
