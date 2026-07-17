//! Unit E — `.spthy` macro expansion (clean-room).
//!
//! Two entry points, differing only in how much of the theory a single call is
//! allowed to touch:
//!  * [`expand`] — full-close view: every macro use at every use site (including
//!    acc-lemma / case-test formulas and derived `(modulo AC)` variant and
//!    left/right diff rule forms) is replaced by its transitively-substituted
//!    body.
//!  * [`expand_staged`] — the consumer's parse-stage view. Same expansion of
//!    ordinary lemmas / restrictions / predicates / processes / the PRIMARY rule
//!    form (and bare-nullary uses), but two carve-outs dictated by the consumer's
//!    pipeline staging: acc-lemma and case-test formulas are left UNTOUCHED (a
//!    later consumer stage owns their expansion [Q41]), and only the primary rule
//!    form is rewritten — derived variant / left-right rule forms are not
//!    recursed into, matching the parse-stage fact that a rule then exists only
//!    in its primary form [Q42].
//!
//! In both, the `macros:` declarations are retained in place (with their
//! original, unexpanded bodies). See ../BEHAVIOR.md for the observed semantics
//! this implements; every rule below traces to a `[Qn]` observation.
//!
//! Summary of the semantics implemented here:
//!  * a macro `name(f1..fk) = body`; call `name(a1..ak)` binds `fi := ai` and
//!    substitutes **simultaneously** (parallel / capture-avoiding)          [Q7]
//!  * a bare, untagged name equal to a NULLARY macro is a parenthesis-free use
//!    of that macro and resolves to its body; `~x`/`$x` and names of arity>=1
//!    macros are ordinary variables and do not resolve                      [Q32,Q33,Q34,Q35]
//!  * a formal matches a body variable by **full identity incl. sort**; `~x`
//!    and `$x` do NOT match an untagged formal `x` and stay free            [Q27,Q28]
//!  * a nullary macro reserves its name against a same-named formal: inside a
//!    body the name resolves to the macro, not the formal                   [Q36]
//!  * bodies may call only strictly-earlier macros ⇒ the macro dependency
//!    graph is a DAG, so expansion always terminates                        [Q8,Q19,Q20]
//!  * expansion is transitive: a body's own macro calls are expanded too    [Q9,Q18]
//!  * at the AST level a call's arg-count already equals the macro arity
//!    (the parser packs/rejects mismatches); `expand` no-ops on a mismatch)  [Q11,Q12,Q15,Q16,Q17]
//!  * the `macros:` declaration block is preserved unchanged in the output;
//!    the reference retains it in its pretty output and the consuming
//!    pipeline requires it in place                                          [Q37]

pub mod ast;

use std::collections::HashMap;
use ast::*;

/// Table of macros whose bodies are already fully expanded (macro-free),
/// keyed by macro name. Formals are matched against body variables by full
/// `VarSpec` identity, so the substitution map is keyed by `VarSpec`.
#[derive(Default)]
struct MacroTable {
    by_name: HashMap<String, (Vec<VarSpec>, Term)>,
}

impl MacroTable {
    fn get(&self, name: &str) -> Option<&(Vec<VarSpec>, Term)> {
        self.by_name.get(name)
    }
}

/// How much of the theory one expansion pass is allowed to touch. The two modes
/// share the whole term/formula/rule traversal and differ only at the two
/// consumer-staging carve-outs (acc-lemma & case-test formulas; derived rule
/// forms).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// Full-close view: expand every use site, including acc-lemma / case-test
    /// formulas and derived `(modulo AC)` variant / left-right diff rule forms.
    FullClose,
    /// Consumer's parse-stage view: leave acc-lemma & case-test formulas
    /// untouched (a later stage owns them [Q41]) and rewrite only the primary
    /// rule form, not derived variant / left-right forms [Q42].
    Staged,
}

/// Full-close macro expansion: expand every macro use at every use site, keeping
/// the `macros:` declarations in place.
///
/// The result, fed back to the oracle, reproduces the reasoning content the
/// oracle computes for the original (modulo-AC variants / expanded-formula /
/// guarded-formula), which is byte-identical to a hand-inlined equivalent
/// (verified by workspace/byteparity.sh and workspace/formula_parity.sh). The
/// `macros:` declaration items pass through unchanged (with their original,
/// unexpanded bodies): the reference retains the block in its pretty output and
/// the consuming pipeline requires it in place [Q37].
pub fn expand(theory: &Theory) -> Theory {
    expand_with(theory, Mode::FullClose)
}

/// Staged (parse-stage) macro expansion — the entry the consumer's pipeline
/// calls.
///
/// Identical to [`expand`] except for two carve-outs the consumer's staging
/// requires, because it invokes this pass before a later stage that owns them:
///  * acc-lemma (`... accounts for "..."`) and case-test (`test <name>: "..."`)
///    formulas are left byte-identical — untouched — so the later stage can
///    expand them itself [Q41];
///  * rules exist only in their primary form at this stage, so only the primary
///    premises / actions / conclusions / let-block / embedded restrictions are
///    rewritten; derived variant and left-right diff rule forms are not recursed
///    into [Q42].
///
/// Everything else — ordinary lemmas, restrictions, predicates, processes,
/// equations, the primary rule form, and bare-nullary resolution — is expanded
/// exactly as in full close. The `macros:` declarations pass through unchanged
/// [Q37].
pub fn expand_staged(theory: &Theory) -> Theory {
    expand_with(theory, Mode::Staged)
}

fn expand_with(theory: &Theory, mode: Mode) -> Theory {
    let table = build_table(theory);
    let items = theory.items.iter().map(|it| expand_item(it, &table, mode)).collect();
    Theory {
        is_diff: theory.is_diff,
        name: theory.name.clone(),
        configuration: theory.configuration.clone(),
        items,
    }
}

/// Build the macro table with fully-expanded (macro-free) bodies.
///
/// Macros are processed in declaration order; each body is expanded against the
/// macros defined strictly before it. Forward/self/mutual references are parse
/// errors upstream [Q8,Q19,Q20], so "earlier only" is complete and cannot loop.
fn build_table(theory: &Theory) -> MacroTable {
    let mut table = MacroTable::default();
    for item in &theory.items {
        if let TheoryItem::Macros(ms) = item {
            for m in ms {
                let body = expand_term(&m.body, &table);
                table.by_name.insert(m.name.clone(), (m.args.clone(), body));
            }
        }
    }
    table
}

// ---- term-level expansion -------------------------------------------------

/// Bottom-up macro expansion of a term. Arguments are expanded first, then a
/// macro call is replaced by its body with formals bound to the expanded args.
fn expand_term(t: &Term, table: &MacroTable) -> Term {
    match t {
        // A bare, untagged name equal to a NULLARY macro is that macro used
        // without parentheses; resolve it to the (already-expanded) body. A
        // fresh/pub-sorted name or the name of an arity>=1 macro is an ordinary
        // variable and is left untouched [Q32,Q33,Q34,Q35]. A nullary macro
        // reserves its name even against a same-named formal [Q36].
        Term::Var(v) => match table.get(&v.name) {
            Some((formals, body)) if v.sort == SortHint::Untagged && formals.is_empty() => {
                body.clone()
            }
            _ => t.clone(),
        },

        Term::PubLit(_)
        | Term::FreshLit(_)
        | Term::NatLit(_)
        | Term::Number(_)
        | Term::NumberOne
        | Term::NatOne
        | Term::DhNeutral => t.clone(),

        Term::App(name, args) => {
            let args: Vec<Term> = args.iter().map(|a| expand_term(a, table)).collect();
            if let Some((formals, body)) = table.get(name) {
                // At the AST level arg-count == arity for a valid parse. If it
                // ever differs (defensive), leave the call unexpanded.
                if formals.len() == args.len() {
                    let subst = build_subst(formals, &args);
                    // `body` is already macro-free; a simultaneous substitution
                    // of macro-free args therefore yields a macro-free term.
                    return substitute_term(body, &subst);
                }
            }
            Term::App(name.clone(), args)
        }

        Term::AlgApp(name, a, b) => Term::AlgApp(
            name.clone(),
            Box::new(expand_term(a, table)),
            Box::new(expand_term(b, table)),
        ),
        Term::Pair(ts) => Term::Pair(ts.iter().map(|x| expand_term(x, table)).collect()),
        Term::Diff(a, b) => Term::Diff(
            Box::new(expand_term(a, table)),
            Box::new(expand_term(b, table)),
        ),
        Term::BinOp(op, a, b) => Term::BinOp(
            *op,
            Box::new(expand_term(a, table)),
            Box::new(expand_term(b, table)),
        ),
        Term::PatMatch(a) => Term::PatMatch(Box::new(expand_term(a, table))),
    }
}

/// Build the simultaneous substitution `formal_i := arg_i`, keyed by the full
/// `VarSpec` of each formal (so sort/idx participate in matching) [Q27,Q28].
fn build_subst(formals: &[VarSpec], args: &[Term]) -> HashMap<VarSpec, Term> {
    formals
        .iter()
        .cloned()
        .zip(args.iter().cloned())
        .collect()
}

/// Apply a simultaneous substitution to `t`. A `Var` that matches a key is
/// replaced by the bound term **without** re-descending into it (parallel
/// substitution — no capture) [Q7]. Non-matching variables (incl. differently
/// sorted `~x`/`$x`) are left untouched [Q27,Q28].
fn substitute_term(t: &Term, subst: &HashMap<VarSpec, Term>) -> Term {
    match t {
        Term::Var(v) => match subst.get(v) {
            Some(bound) => bound.clone(),
            None => t.clone(),
        },
        Term::PubLit(_)
        | Term::FreshLit(_)
        | Term::NatLit(_)
        | Term::Number(_)
        | Term::NumberOne
        | Term::NatOne
        | Term::DhNeutral => t.clone(),
        Term::App(name, args) => Term::App(
            name.clone(),
            args.iter().map(|a| substitute_term(a, subst)).collect(),
        ),
        Term::AlgApp(name, a, b) => Term::AlgApp(
            name.clone(),
            Box::new(substitute_term(a, subst)),
            Box::new(substitute_term(b, subst)),
        ),
        Term::Pair(ts) => Term::Pair(ts.iter().map(|x| substitute_term(x, subst)).collect()),
        Term::Diff(a, b) => Term::Diff(
            Box::new(substitute_term(a, subst)),
            Box::new(substitute_term(b, subst)),
        ),
        Term::BinOp(op, a, b) => Term::BinOp(
            *op,
            Box::new(substitute_term(a, subst)),
            Box::new(substitute_term(b, subst)),
        ),
        Term::PatMatch(a) => Term::PatMatch(Box::new(substitute_term(a, subst))),
    }
}

// ---- structural recursion through the theory ------------------------------

fn expand_fact(f: &Fact, table: &MacroTable) -> Fact {
    Fact {
        persistent: f.persistent,
        name: f.name.clone(),
        args: f.args.iter().map(|a| expand_term(a, table)).collect(),
        annotations: f.annotations.clone(),
    }
}

fn expand_atom(a: &Atom, table: &MacroTable) -> Atom {
    match a {
        Atom::Eq(x, y) => Atom::Eq(expand_term(x, table), expand_term(y, table)),
        Atom::Less(x, y) => Atom::Less(expand_term(x, table), expand_term(y, table)),
        Atom::LessMset(x, y) => Atom::LessMset(expand_term(x, table), expand_term(y, table)),
        Atom::Subterm(x, y) => Atom::Subterm(expand_term(x, table), expand_term(y, table)),
        Atom::Action(f, t) => Atom::Action(expand_fact(f, table), expand_term(t, table)),
        Atom::Last(t) => Atom::Last(expand_term(t, table)),
        Atom::Pred(f) => Atom::Pred(expand_fact(f, table)),
    }
}

fn expand_formula(phi: &Formula, table: &MacroTable) -> Formula {
    match phi {
        Formula::False => Formula::False,
        Formula::True => Formula::True,
        Formula::Atom(a) => Formula::Atom(expand_atom(a, table)),
        Formula::Not(p) => Formula::Not(Box::new(expand_formula(p, table))),
        Formula::And(p, q) => Formula::And(
            Box::new(expand_formula(p, table)),
            Box::new(expand_formula(q, table)),
        ),
        Formula::Or(p, q) => Formula::Or(
            Box::new(expand_formula(p, table)),
            Box::new(expand_formula(q, table)),
        ),
        Formula::Implies(p, q) => Formula::Implies(
            Box::new(expand_formula(p, table)),
            Box::new(expand_formula(q, table)),
        ),
        Formula::Iff(p, q) => Formula::Iff(
            Box::new(expand_formula(p, table)),
            Box::new(expand_formula(q, table)),
        ),
        Formula::Forall(vs, p) => {
            Formula::Forall(vs.clone(), Box::new(expand_formula(p, table)))
        }
        Formula::Exists(vs, p) => {
            Formula::Exists(vs.clone(), Box::new(expand_formula(p, table)))
        }
    }
}

fn expand_let(b: &LetBinding, table: &MacroTable) -> LetBinding {
    // A macro call may appear in the let value [Q23]; the bound pattern `var`
    // is a variable/pattern and is expanded too for uniformity.
    LetBinding {
        var: expand_term(&b.var, table),
        value: expand_term(&b.value, table),
    }
}

fn expand_rule(r: &Rule, table: &MacroTable, mode: Mode) -> Rule {
    Rule {
        name: r.name.clone(),
        modulo: r.modulo.clone(),
        attributes: r.attributes.clone(),
        let_block: r.let_block.iter().map(|b| expand_let(b, table)).collect(),
        premises: r.premises.iter().map(|f| expand_fact(f, table)).collect(),
        actions: r.actions.iter().map(|f| expand_fact(f, table)).collect(),
        conclusions: r.conclusions.iter().map(|f| expand_fact(f, table)).collect(),
        embedded_restrictions: r
            .embedded_restrictions
            .iter()
            .map(|p| expand_formula(p, table))
            .collect(),
        // Derived rule forms. Full close expands them — the `(modulo AC)` variant
        // and left/right diff projections show the expansion [Q3,Q4,Q26]. In the
        // staged view a rule exists only in its primary form [Q42], so these are
        // carried through untouched.
        variants: match mode {
            Mode::FullClose => r.variants.iter().map(|v| expand_rule(v, table, mode)).collect(),
            Mode::Staged => r.variants.clone(),
        },
        left_right: match mode {
            Mode::FullClose => r.left_right.as_ref().map(|(l, rr)| {
                (
                    Box::new(expand_rule(l, table, mode)),
                    Box::new(expand_rule(rr, table, mode)),
                )
            }),
            Mode::Staged => r.left_right.clone(),
        },
    }
}

fn expand_sapic_action(a: &SapicAction, table: &MacroTable) -> SapicAction {
    match a {
        SapicAction::New(v) => SapicAction::New(v.clone()),
        SapicAction::Insert(k, v) => {
            SapicAction::Insert(expand_term(k, table), expand_term(v, table))
        }
        SapicAction::Delete(k) => SapicAction::Delete(expand_term(k, table)),
        SapicAction::ChIn { chan, msg } => SapicAction::ChIn {
            chan: chan.as_ref().map(|c| expand_term(c, table)),
            msg: expand_term(msg, table),
        },
        SapicAction::ChOut { chan, msg } => SapicAction::ChOut {
            chan: chan.as_ref().map(|c| expand_term(c, table)),
            msg: expand_term(msg, table),
        },
        SapicAction::Lock(t) => SapicAction::Lock(expand_term(t, table)),
        SapicAction::Unlock(t) => SapicAction::Unlock(expand_term(t, table)),
        SapicAction::Event(f) => SapicAction::Event(expand_fact(f, table)),
        SapicAction::Msr { prems, acts, concs, restrictions } => SapicAction::Msr {
            prems: prems.iter().map(|f| expand_fact(f, table)).collect(),
            acts: acts.iter().map(|f| expand_fact(f, table)).collect(),
            concs: concs.iter().map(|f| expand_fact(f, table)).collect(),
            restrictions: restrictions.iter().map(|p| expand_formula(p, table)).collect(),
        },
    }
}

fn expand_condition(c: &Condition, table: &MacroTable) -> Condition {
    match c {
        Condition::Eq(x, y) => Condition::Eq(expand_term(x, table), expand_term(y, table)),
        Condition::Formula(p) => Condition::Formula(expand_formula(p, table)),
    }
}

fn expand_comb(c: &ProcessComb, table: &MacroTable) -> ProcessComb {
    match c {
        ProcessComb::Parallel => ProcessComb::Parallel,
        ProcessComb::Ndc => ProcessComb::Ndc,
        ProcessComb::Cond(cond) => ProcessComb::Cond(expand_condition(cond, table)),
        ProcessComb::Lookup(t, v) => ProcessComb::Lookup(expand_term(t, table), v.clone()),
        ProcessComb::Let { pat, value } => ProcessComb::Let {
            pat: expand_term(pat, table),
            value: expand_term(value, table),
        },
    }
}

fn expand_process(p: &Process, table: &MacroTable) -> Process {
    match p {
        Process::Null => Process::Null,
        Process::Action { action, body } => Process::Action {
            action: expand_sapic_action(action, table),
            body: Box::new(expand_process(body, table)),
        },
        Process::Comb { comb, left, right } => Process::Comb {
            comb: expand_comb(comb, table),
            left: Box::new(expand_process(left, table)),
            right: Box::new(expand_process(right, table)),
        },
        Process::Replication(b) => Process::Replication(Box::new(expand_process(b, table))),
        Process::Call { name, args } => Process::Call {
            name: name.clone(),
            args: args.iter().map(|a| expand_term(a, table)).collect(),
        },
        Process::AtAnnotation(b, t) => {
            Process::AtAnnotation(Box::new(expand_process(b, table)), expand_term(t, table))
        }
    }
}

fn expand_restriction(r: &Restriction, table: &MacroTable) -> Restriction {
    Restriction {
        name: r.name.clone(),
        formula: expand_formula(&r.formula, table),
        attributes: r.attributes.clone(),
    }
}

fn expand_lemma(l: &Lemma, table: &MacroTable) -> Lemma {
    Lemma {
        name: l.name.clone(),
        modulo: l.modulo.clone(),
        attributes: l.attributes.clone(),
        trace_quantifier: l.trace_quantifier.clone(),
        formula: expand_formula(&l.formula, table),
        proof: l.proof.clone(),
        plaintext: l.plaintext.clone(),
    }
}

fn expand_predicate(p: &Predicate, table: &MacroTable) -> Predicate {
    Predicate {
        fact: expand_fact(&p.fact, table),
        formula: expand_formula(&p.formula, table),
    }
}

fn expand_item(it: &TheoryItem, table: &MacroTable, mode: Mode) -> TheoryItem {
    match it {
        TheoryItem::Rule(r) => TheoryItem::Rule(expand_rule(r, table, mode)),
        TheoryItem::IntrRule(r) => TheoryItem::IntrRule(expand_rule(r, table, mode)),
        TheoryItem::Restriction(r) => TheoryItem::Restriction(expand_restriction(r, table)),
        TheoryItem::LegacyAxiom(r) => TheoryItem::LegacyAxiom(expand_restriction(r, table)),
        TheoryItem::Lemma(l) => TheoryItem::Lemma(expand_lemma(l, table)),
        // acc-lemma & case-test formulas: full close expands them (the generated
        // lemmas' guarded forms show the expansion [Q38,Q39]); the staged view
        // leaves them untouched because a later consumer stage owns them [Q41].
        TheoryItem::AccLemma(a) => match mode {
            Mode::FullClose => TheoryItem::AccLemma(AccLemma {
                name: a.name.clone(),
                attributes: a.attributes.clone(),
                formula: expand_formula(&a.formula, table),
                case_test_idents: a.case_test_idents.clone(),
            }),
            Mode::Staged => it.clone(),
        },
        TheoryItem::CaseTest(c) => match mode {
            Mode::FullClose => TheoryItem::CaseTest(CaseTest {
                name: c.name.clone(),
                formula: expand_formula(&c.formula, table),
            }),
            Mode::Staged => it.clone(),
        },
        TheoryItem::Predicates(ps) => {
            TheoryItem::Predicates(ps.iter().map(|p| expand_predicate(p, table)).collect())
        }
        TheoryItem::ProcessDef(pd) => TheoryItem::ProcessDef(ProcessDef {
            name: pd.name.clone(),
            vars: pd.vars.clone(),
            body: expand_process(&pd.body, table),
        }),
        TheoryItem::TopLevelProcess(p) => {
            TheoryItem::TopLevelProcess(expand_process(p, table))
        }
        TheoryItem::EquivLemma(a, b) => {
            TheoryItem::EquivLemma(expand_process(a, table), expand_process(b, table))
        }
        TheoryItem::DiffEquivLemma(p) => {
            TheoryItem::DiffEquivLemma(expand_process(p, table))
        }
        // Equations carry terms; expand uniformly (a no-op when they contain no
        // macro call — not observed in the corpus, but handled for robustness).
        TheoryItem::Equations { convergent, eqs } => TheoryItem::Equations {
            convergent: *convergent,
            eqs: eqs
                .iter()
                .map(|e| Equation {
                    lhs: expand_term(&e.lhs, table),
                    rhs: expand_term(&e.rhs, table),
                })
                .collect(),
        },
        // Items with no macro-bearing terms are carried through unchanged. The
        // `macros:` declaration items pass through here too — retained in place
        // with their original, unexpanded bodies [Q37].
        TheoryItem::Macros(_)
        | TheoryItem::Builtins(_)
        | TheoryItem::Functions(_)
        | TheoryItem::Options(_)
        | TheoryItem::Heuristic(_)
        | TheoryItem::Tactic(_)
        | TheoryItem::DiffLemma(_)
        | TheoryItem::Export { .. }
        | TheoryItem::FormalComment { .. }
        | TheoryItem::Define(_)
        | TheoryItem::Include(_) => it.clone(),
    }
}

#[cfg(test)]
mod tests;
