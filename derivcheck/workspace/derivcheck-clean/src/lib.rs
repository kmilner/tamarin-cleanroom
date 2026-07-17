//! Unit G — Message Derivation Checks (clean-room reimplementation).
//!
//! For each protocol rule, the prover decides whether every variable of the rule
//! is derivable by the intruder from the rule's premises. Variables that are not
//! derivable are reported under the wellformedness topic `Message Derivation
//! Checks`. The derivability decision is the prover's solver and is NOT
//! reimplemented here: it is supplied by the caller through [`DerivabilitySolver`].
//!
//! This crate owns three things, all characterized purely by black-box
//! observation of the reference tool (see workspace/BEHAVIOR.md and QUERIES.log):
//!   1. probe construction — which rules and which variables are handed to the
//!      solver (rule filtering, candidate-variable enumeration);
//!   2. the decision logic around solver outcomes (deactivation, timeouts,
//!      derivable/not-derivable);
//!   3. byte-exact report text and variable ordering.

pub mod ast;

use std::collections::{BTreeSet, HashSet};

use ast::{Rule, RuleAttr, SortHint, Term, TheoryItem, Theory, VarSpec, Fact};

/// A single wellformedness finding: a topic and its message body. Mirrors the
/// `WfError` surface of the wellformedness unit so findings compose into the
/// shared report model.
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

/// One derivability question: can the intruder derive `variable` (X) from
/// `premises` (Y) of `rule`, within `timeout_secs`? The full `rule` is provided
/// so a solver that needs more context than the premise facts can use it.
pub struct DerivProbe<'a> {
    pub rule_name: &'a str,
    pub rule: &'a Rule,
    pub variable: &'a VarSpec,
    pub premises: &'a [Fact],
    pub timeout_secs: u64,
}

/// The solver's verdict for one probe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Derivability {
    /// The intruder can derive the variable from the premises.
    Derivable,
    /// The intruder cannot derive the variable — it will be reported.
    NotDerivable,
    /// The derivation search exceeded the timeout budget without deciding.
    TimedOut,
}

/// The decision callback this crate is parameterized over. In production it wraps
/// the prover's message-deduction solver; in tests it is stubbed.
pub trait DerivabilitySolver {
    fn check(&self, probe: &DerivProbe) -> Derivability;
}

/// How a per-rule timeout is treated. See the residual note in BEHAVIOR.md §7:
/// a real per-rule timeout was not observable, so the policy is explicit here.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeoutPolicy {
    /// Suppress the whole rule's warning when any of its probes times out
    /// (fail-open; the chosen default, consistent with `timeout=0` suppressing
    /// all output).
    SuppressRule,
    /// Treat a timed-out probe like `Derivable` (skip only that variable) and
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

        let candidates = candidate_variables(rule);
        let mut flagged: Vec<VarSpec> = Vec::new();
        let mut rule_timed_out = false;

        for var in &candidates {
            let probe = DerivProbe {
                rule_name: &rule.name,
                rule,
                variable: var,
                premises: &rule.premises,
                timeout_secs,
            };
            match solver.check(&probe) {
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
            blocks.push((rule.name.clone(), flagged));
        }
    }

    if blocks.is_empty() {
        return Vec::new();
    }

    // The intro paragraph is emitted once, then one entry per failing rule; all
    // share the topic so the report groups them under a single header.
    let mut report: WfReport = Vec::with_capacity(blocks.len() + 1);
    report.push(WfError::new(DERIVATION_TOPIC, DERIVATION_INTRO));
    for (name, vars) in blocks {
        report.push(WfError::new(DERIVATION_TOPIC, rule_block(&name, &vars)));
    }
    report
}

// --------------------------------------------------------------------------
// Probe construction
// --------------------------------------------------------------------------

/// The candidate variables handed to the solver for one rule: every distinct
/// variable occurring in the rule's premises, actions, conclusions, and let
/// bindings, in first-occurrence order (order is irrelevant — the flagged subset
/// is re-sorted for display). The solver decides which are actually derivable,
/// so this set is a safe superset: public names, `Fr`-bound fresh names, and
/// state-fact-bound variables are all resolved to `Derivable` by the solver.
pub fn candidate_variables(rule: &Rule) -> Vec<VarSpec> {
    let mut out: Vec<VarSpec> = Vec::new();
    let mut seen: HashSet<VarSpec> = HashSet::new();
    let push = |t: &Term, out: &mut Vec<VarSpec>, seen: &mut HashSet<VarSpec>| {
        collect_vars(t, &mut |v| {
            if seen.insert(v.clone()) {
                out.push(v.clone());
            }
        });
    };
    for f in &rule.premises {
        for a in &f.args {
            push(a, &mut out, &mut seen);
        }
    }
    for f in &rule.actions {
        for a in &f.args {
            push(a, &mut out, &mut seen);
        }
    }
    for f in &rule.conclusions {
        for a in &f.args {
            push(a, &mut out, &mut seen);
        }
    }
    for lb in &rule.let_block {
        push(&lb.var, &mut out, &mut seen);
        push(&lb.value, &mut out, &mut seen);
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
/// ascending by `(sort-rank, name lexicographically, index numerically)`.
pub fn sort_variables(vars: &mut Vec<VarSpec>) {
    vars.sort_by(var_order_key);
    vars.dedup();
}

/// Total order used to sort reported variables (see BEHAVIOR.md §6).
fn var_order_key(a: &VarSpec, b: &VarSpec) -> std::cmp::Ordering {
    sort_rank(a.sort)
        .cmp(&sort_rank(b.sort))
        .then_with(|| a.name.cmp(&b.name)) // lexicographic string comparison
        .then_with(|| a.idx.cmp(&b.idx)) // numeric index comparison
}

/// Sort-tag rank. Observed among reportable (non-derivable) variables:
/// `Fresh < Msg`. Message-sort variables carry either `Msg` or `Untagged` and
/// rank together. Other sorts (`Pub`, `Node`, `Nat`, `Suffix`) do not occur as
/// reported variables — public names are always derivable and node/nat are not
/// message terms — so their relative rank is unobserved and placed last.
fn sort_rank(sort: SortHint) -> u8 {
    match sort {
        SortHint::Fresh => 0,
        SortHint::Msg | SortHint::Untagged => 1,
        _ => 2,
    }
}

/// Render one variable as the reference tool spells it in the warning: fresh
/// names carry a `~` prefix, public names a `$` prefix, message names none; a
/// nonzero index is appended as `.idx` (e.g. `x.2`).
pub fn render_variable(v: &VarSpec) -> String {
    let prefix = match v.sort {
        SortHint::Fresh => "~",
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
    let list = vars
        .iter()
        .map(render_variable)
        .collect::<Vec<_>>()
        .join(", ");
    format!("Rule {}: \nFailed to derive Variable(s): {}", rule_name, list)
}

// --------------------------------------------------------------------------
// Rendering
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

/// Render the `Message Derivation Checks` topic block byte-exactly. Returns the
/// empty string for an empty report. The block has no trailing newline; the
/// surrounding report writer adds separators between topic blocks.
pub fn render_derivation_report(report: &WfReport) -> String {
    let messages: Vec<&str> = report
        .iter()
        .filter(|e| e.topic == DERIVATION_TOPIC)
        .map(|e| e.message.as_str())
        .collect();
    if messages.is_empty() {
        return String::new();
    }
    format!(
        "{}\n{}",
        underline_topic(DERIVATION_TOPIC),
        messages.join("\n\n")
    )
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
    fn pub_var(name: &str) -> VarSpec {
        VarSpec { name: name.to_string(), idx: 0, sort: SortHint::Pub, typ: None }
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
        Rule {
            name: name.to_string(),
            modulo: None,
            attributes: attrs,
            let_block: vec![],
            premises,
            actions: vec![],
            conclusions,
            embedded_restrictions: vec![],
            variants: vec![],
            left_right: None,
        }
    }

    fn theory(items: Vec<TheoryItem>) -> Theory {
        Theory { is_diff: false, name: "T".into(), configuration: None, items }
    }

    // A stub solver that flags exactly the variables whose rendered name is in
    // its set as NotDerivable; everything else Derivable.
    struct FlagByName {
        not_derivable: HashSet<String>,
    }
    impl FlagByName {
        fn new(names: &[&str]) -> Self {
            FlagByName { not_derivable: names.iter().map(|s| s.to_string()).collect() }
        }
    }
    impl DerivabilitySolver for FlagByName {
        fn check(&self, probe: &DerivProbe) -> Derivability {
            if self.not_derivable.contains(&probe.variable.name) {
                Derivability::NotDerivable
            } else {
                Derivability::Derivable
            }
        }
    }

    // A stub that always times out.
    struct AlwaysTimeout;
    impl DerivabilitySolver for AlwaysTimeout {
        fn check(&self, _: &DerivProbe) -> Derivability {
            Derivability::TimedOut
        }
    }

    // A stub that panics if consulted (to prove short-circuiting).
    struct NeverCalled;
    impl DerivabilitySolver for NeverCalled {
        fn check(&self, _: &DerivProbe) -> Derivability {
            panic!("solver must not be consulted");
        }
    }

    // ---- byte-exact output (matches captured oracle text) ----

    #[test]
    fn single_rule_block_is_byte_exact() {
        // Rule R: [ In(h(w)) ] --> [ Out('ok') ]; solver flags w.
        let r = rule(
            "R",
            vec![fact("In", vec![app("h", vec![v("w")])])],
            vec![fact("Out", vec![Term::PubLit("ok".into())])],
            vec![],
        );
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["w"]), 5);

        // `concat!` keeps the trailing space after "Rule R:" explicit.
        let expected = concat!(
            "Message Derivation Checks\n",
            "=========================\n",
            "\n",
            "  The variables of the following rule(s) are not derivable from their premises, you may be performing unintended pattern matching.\n",
            "\n",
            "Rule R: \n",
            "Failed to derive Variable(s): w",
        );
        assert_eq!(render_derivation_report(&report), expected);
        assert_eq!(topics(&report), ["Message Derivation Checks".to_string()].into_iter().collect());
    }

    #[test]
    fn two_rule_block_matches_poidc_cmb() {
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
        // Flag exactly the variables the oracle reported.
        let solver = FlagByName::new(&["m", "r2", "sk2", "pkA"]);
        let report = message_derivation_checks(&thy, &solver, 5);

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
        assert_eq!(render_derivation_report(&report), expected);
    }

    // ---- ordering ----

    #[test]
    fn variables_sorted_by_sort_then_name_then_index() {
        // Feed a deliberately scrambled candidate order; all flagged.
        let vars = vec![
            v("v2"),
            Term::Var(fresh_var("zzz")),
            Term::Var(msg_var_idx("x", 10)),
            v("v10"),
            Term::Var(msg_var_idx("x", 2)),
            v("v1"),
            Term::Var(msg_var_idx("x", 1)),
            Term::Var(fresh_var("nnn")),
        ];
        let r = rule(
            "R",
            vec![fact("In", vec![app("h", vec![Term::Pair(vars)])])],
            vec![],
            vec![],
        );
        let thy = theory(vec![TheoryItem::Rule(r)]);
        // Flag everything.
        let solver = FlagByName::new(&["v1", "v2", "v10", "x", "zzz", "nnn"]);
        let report = message_derivation_checks(&thy, &solver, 5);
        let rendered = render_derivation_report(&report);
        // Fresh (~) rank before Msg; within Msg: name lexicographic then idx numeric.
        assert!(rendered.ends_with(
            "Rule R: \nFailed to derive Variable(s): ~nnn, ~zzz, v1, v10, v2, x.1, x.2, x.10"
        ), "got: {}", rendered);
    }

    #[test]
    fn fresh_sort_orders_before_message_even_with_later_name() {
        // ~zzz (Fresh, name z) must precede aaa (Msg, name a): sort dominates name.
        let vars = vec![v("aaa"), Term::Var(fresh_var("zzz"))];
        let r = rule("R", vec![fact("In", vec![app("h", vars)])], vec![], vec![]);
        let thy = theory(vec![TheoryItem::Rule(r)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["aaa", "zzz"]), 5);
        assert!(render_derivation_report(&report).ends_with("Variable(s): ~zzz, aaa"));
    }

    #[test]
    fn duplicate_variable_listed_once() {
        // In(h(w)), In(h(w)) -> w appears twice, reported once.
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
        // Even though the solver would flag x, the rule is skipped.
        let report = message_derivation_checks(&thy, &FlagByName::new(&["x"]), 5);
        assert!(report.is_empty());
        assert_eq!(render_derivation_report(&report), "");
    }

    #[test]
    fn derivable_rule_is_omitted_and_theory_order_preserved() {
        // Good (all derivable) then Bad (w flagged); only Bad appears.
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
        // Zebra before Apple in source -> Zebra before Apple in output.
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
        // An IntrRule item, even if it would flag, is not a protocol rule.
        let ir = rule("iknows", vec![fact("In", vec![app("h", vec![v("x")])])], vec![], vec![]);
        let thy = theory(vec![TheoryItem::IntrRule(ir)]);
        let report = message_derivation_checks(&thy, &FlagByName::new(&["x"]), 5);
        assert!(report.is_empty());
    }

    // ---- activation / timeout decision logic ----

    #[test]
    fn timeout_zero_deactivates_without_consulting_solver() {
        let r = rule("R", vec![fact("In", vec![app("h", vec![v("x")])])], vec![], vec![]);
        let thy = theory(vec![TheoryItem::Rule(r)]);
        // NeverCalled panics if consulted; timeout 0 must short-circuit.
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
        // One var times out (yy), one is definitely not derivable (xx via a
        // mixed stub). Under SkipVariable, only xx is reported.
        struct Mixed;
        impl DerivabilitySolver for Mixed {
            fn check(&self, p: &DerivProbe) -> Derivability {
                match p.variable.name.as_str() {
                    "xx" => Derivability::NotDerivable,
                    "yy" => Derivability::TimedOut,
                    _ => Derivability::Derivable,
                }
            }
        }
        let r = rule(
            "R",
            vec![fact("In", vec![app("h", vec![v("xx"), v("yy")])])],
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
    fn public_variable_rendering_uses_dollar_and_index_suffix() {
        assert_eq!(render_variable(&pub_var("A")), "$A");
        assert_eq!(render_variable(&fresh_var("n")), "~n");
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
