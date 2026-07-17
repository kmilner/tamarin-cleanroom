//! Unit E — `.spthy` macro expansion (clean-room).
//!
//! `expand(theory)` returns an equivalent macro-free theory: every macro call
//! at every use site is replaced by its transitively-substituted body, and the
//! `macros:` declarations are dropped. See ../BEHAVIOR.md for the observed
//! semantics this implements; every rule below traces to a `[Qn]` observation.
//!
//! Summary of the semantics implemented here:
//!  * a macro `name(f1..fk) = body`; call `name(a1..ak)` binds `fi := ai` and
//!    substitutes **simultaneously** (parallel / capture-avoiding)          [Q7]
//!  * a formal matches a body variable by **full identity incl. sort**; `~x`
//!    and `$x` do NOT match an untagged formal `x` and stay free            [Q27,Q28]
//!  * bodies may call only strictly-earlier macros ⇒ the macro dependency
//!    graph is a DAG, so expansion always terminates                        [Q8,Q19,Q20]
//!  * expansion is transitive: a body's own macro calls are expanded too    [Q9,Q18]
//!  * at the AST level a call's arg-count already equals the macro arity
//!    (the parser packs/【rejects mismatches); `expand` no-ops on a mismatch) [Q11,Q12,Q15,Q16,Q17]

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

/// Expand every macro use in `theory` and drop the macro declarations.
///
/// The result, fed back to the oracle, reproduces the reasoning content the
/// oracle computes for the original (modulo-AC variants / expanded-formula /
/// guarded-formula), which is byte-identical to a hand-inlined equivalent
/// (verified by workspace/byteparity.sh).
pub fn expand(theory: &Theory) -> Theory {
    let table = build_table(theory);
    let items = theory
        .items
        .iter()
        .filter(|it| !matches!(it, TheoryItem::Macros(_)))
        .map(|it| expand_item(it, &table))
        .collect();
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
        Term::Var(_)
        | Term::PubLit(_)
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

fn expand_rule(r: &Rule, table: &MacroTable) -> Rule {
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
        variants: r.variants.iter().map(|v| expand_rule(v, table)).collect(),
        left_right: r.left_right.as_ref().map(|(l, rr)| {
            (
                Box::new(expand_rule(l, table)),
                Box::new(expand_rule(rr, table)),
            )
        }),
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

fn expand_item(it: &TheoryItem, table: &MacroTable) -> TheoryItem {
    match it {
        TheoryItem::Rule(r) => TheoryItem::Rule(expand_rule(r, table)),
        TheoryItem::IntrRule(r) => TheoryItem::IntrRule(expand_rule(r, table)),
        TheoryItem::Restriction(r) => TheoryItem::Restriction(expand_restriction(r, table)),
        TheoryItem::LegacyAxiom(r) => TheoryItem::LegacyAxiom(expand_restriction(r, table)),
        TheoryItem::Lemma(l) => TheoryItem::Lemma(expand_lemma(l, table)),
        TheoryItem::AccLemma(a) => TheoryItem::AccLemma(AccLemma {
            name: a.name.clone(),
            attributes: a.attributes.clone(),
            formula: expand_formula(&a.formula, table),
            case_test_idents: a.case_test_idents.clone(),
        }),
        TheoryItem::CaseTest(c) => TheoryItem::CaseTest(CaseTest {
            name: c.name.clone(),
            formula: expand_formula(&c.formula, table),
        }),
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
        // Items with no macro-bearing terms are carried through unchanged. Macros
        // items are filtered out before this function is called.
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
