//! Direct expansion checks (AST-level). Each fixture mirrors a `[Qn]` oracle
//! observation and, for the byte-parity set, the hand-inlined `.spthy` in
//! ../fixtures/ that workspace/byteparity.sh proves equal to the macro theory.

use super::ast::*;
use super::{expand, expand_staged};

// ---- tiny AST constructors ------------------------------------------------

fn var(name: &str, sort: SortHint) -> VarSpec {
    VarSpec { name: name.into(), idx: 0, sort, typ: None }
}
fn msg(name: &str) -> Term {
    Term::Var(var(name, SortHint::Untagged))
}
fn fresh(name: &str) -> Term {
    Term::Var(var(name, SortHint::Fresh))
}
fn pubv(name: &str) -> Term {
    Term::Var(var(name, SortHint::Pub))
}
fn pair(ts: Vec<Term>) -> Term {
    Term::Pair(ts)
}
fn app(name: &str, args: Vec<Term>) -> Term {
    Term::App(name.into(), args)
}
fn fact(name: &str, args: Vec<Term>) -> Fact {
    Fact { persistent: false, name: name.into(), args, annotations: vec![] }
}
fn mdef(name: &str, formals: &[VarSpec], body: Term) -> Macro {
    Macro { name: name.into(), args: formals.to_vec(), body }
}
/// A minimal rule with one action fact `Act(<action_arg>)`.
fn rule_act(name: &str, prems: Vec<Fact>, action: Fact) -> Rule {
    Rule {
        name: name.into(),
        modulo: None,
        attributes: vec![],
        let_block: vec![],
        premises: prems,
        actions: vec![action],
        conclusions: vec![],
        embedded_restrictions: vec![],
        variants: vec![],
        left_right: None,
    }
}
fn theory(items: Vec<TheoryItem>) -> Theory {
    Theory { is_diff: false, name: "T".into(), configuration: None, items }
}

/// The action term of the single rule in a single-rule theory (post-expand).
fn only_rule_action_arg(t: &Theory) -> Term {
    let r = t.items.iter().find_map(|it| match it {
        TheoryItem::Rule(r) => Some(r),
        _ => None,
    }).expect("a rule");
    r.actions[0].args[0].clone()
}

// ---- macros: declarations are preserved in place [Q37] --------------------

#[test]
fn preserves_macro_declarations() {
    // The `macros:` block is retained in place (with its original, unexpanded
    // body) while the use site is expanded. The consuming pipeline requires the
    // declaration block, and the reference retains it in its pretty output.
    let x = var("x", SortHint::Untagged);
    let macros = TheoryItem::Macros(vec![mdef("m", &[x.clone()], msg("x"))]);
    let t = theory(vec![
        macros.clone(),
        TheoryItem::Rule(rule_act("R", vec![fact("In", vec![msg("x")])],
                                  fact("Act", vec![app("m", vec![msg("x")])]))),
    ]);
    let out = expand(&t);
    // declaration item retained, unchanged, in its original position
    assert_eq!(out.items.len(), 2);
    assert_eq!(out.items[0], macros);
    // use site expanded: Act(m(x)) -> Act(x)
    assert_eq!(only_rule_action_arg(&out), msg("x"));
}

// ---- Q7: simultaneous (capture-avoiding) substitution ---------------------

#[test]
fn capture_simultaneous_substitution() {
    // swap(x,y) = <x,y>;  Act(swap(y,x))  ==>  Act(<y,x>)
    let (x, y) = (var("x", SortHint::Untagged), var("y", SortHint::Untagged));
    let t = theory(vec![
        TheoryItem::Macros(vec![mdef("swap", &[x.clone(), y.clone()],
                                     pair(vec![msg("x"), msg("y")]))]),
        TheoryItem::Rule(rule_act(
            "R",
            vec![fact("In", vec![msg("x")]), fact("In", vec![msg("y")])],
            fact("Act", vec![app("swap", vec![msg("y"), msg("x")])]),
        )),
    ]);
    assert_eq!(only_rule_action_arg(&expand(&t)), pair(vec![msg("y"), msg("x")]));
    // A sequential (buggy) substitution would give <x,x> or <y,y>.
    assert_ne!(only_rule_action_arg(&expand(&t)), pair(vec![msg("x"), msg("x")]));
    assert_ne!(only_rule_action_arg(&expand(&t)), pair(vec![msg("y"), msg("y")]));
}

// ---- Q4: pair macro + identity macro + nesting ----------------------------

#[test]
fn lemmas_and_restrictions_fixture() {
    // Full mirror of MacroInLemmasAndRestrictions vs lemmas_expanded.spthy.
    let x = var("x", SortHint::Untagged);
    let y = var("y", SortHint::Untagged);
    let macros = TheoryItem::Macros(vec![
        mdef("m", &[x.clone()], msg("x")),
        mdef("m2", &[x.clone(), y.clone()], pair(vec![msg("x"), msg("y")])),
        mdef("m3", &[x.clone()], msg("x")),
    ]);
    let macros_kept = macros.clone(); // declarations are preserved in place [Q37]
    // rule A: [In(x)] --[A(m(x))]-> []
    let a = rule_act("A", vec![fact("In", vec![msg("x")])],
                     fact("A", vec![app("m", vec![msg("x")])]));
    // rule B: [In(x),In(y)] --[B(m2(x,y))]-> []
    let b = rule_act("B", vec![fact("In", vec![msg("x")]), fact("In", vec![msg("y")])],
                     fact("B", vec![app("m2", vec![msg("x"), msg("y")])]));
    // restriction: All x #i. A(m(m3(x)))@i ==> Ex y. x=y
    let restr = TheoryItem::Restriction(Restriction {
        name: "OnlyValidProcessing".into(),
        attributes: vec![],
        formula: Formula::Forall(
            vec![x.clone(), var("i", SortHint::Node)],
            Box::new(Formula::Implies(
                Box::new(Formula::Atom(Atom::Action(
                    fact("A", vec![app("m", vec![app("m3", vec![msg("x")])])]),
                    Term::Var(var("i", SortHint::Node)),
                ))),
                Box::new(Formula::Exists(
                    vec![y.clone()],
                    Box::new(Formula::Atom(Atom::Eq(msg("x"), msg("y")))),
                )),
            )),
        ),
    });
    // lemma M: exists-trace "Ex x #i. A(m(x))@i"
    let lem = TheoryItem::Lemma(Lemma {
        name: "M".into(),
        modulo: None,
        attributes: vec![],
        trace_quantifier: TraceQuantifier::ExistsTrace,
        formula: Formula::Exists(
            vec![x.clone(), var("i", SortHint::Node)],
            Box::new(Formula::Atom(Atom::Action(
                fact("A", vec![app("m", vec![msg("x")])]),
                Term::Var(var("i", SortHint::Node)),
            ))),
        ),
        proof: None,
        plaintext: String::new(),
    });
    let out = expand(&theory(vec![macros, TheoryItem::Rule(a), TheoryItem::Rule(b), restr, lem]));

    // Expected macro-free equivalent.
    let a_e = rule_act("A", vec![fact("In", vec![msg("x")])], fact("A", vec![msg("x")]));
    let b_e = rule_act("B", vec![fact("In", vec![msg("x")]), fact("In", vec![msg("y")])],
                       fact("B", vec![pair(vec![msg("x"), msg("y")])]));
    let restr_e = TheoryItem::Restriction(Restriction {
        name: "OnlyValidProcessing".into(),
        attributes: vec![],
        formula: Formula::Forall(
            vec![x.clone(), var("i", SortHint::Node)],
            Box::new(Formula::Implies(
                Box::new(Formula::Atom(Atom::Action(
                    fact("A", vec![msg("x")]), Term::Var(var("i", SortHint::Node)),
                ))),
                Box::new(Formula::Exists(
                    vec![y.clone()],
                    Box::new(Formula::Atom(Atom::Eq(msg("x"), msg("y")))),
                )),
            )),
        ),
    });
    let lem_e = TheoryItem::Lemma(Lemma {
        name: "M".into(),
        modulo: None,
        attributes: vec![],
        trace_quantifier: TraceQuantifier::ExistsTrace,
        formula: Formula::Exists(
            vec![x.clone(), var("i", SortHint::Node)],
            Box::new(Formula::Atom(Atom::Action(
                fact("A", vec![msg("x")]), Term::Var(var("i", SortHint::Node)),
            ))),
        ),
        proof: None,
        plaintext: String::new(),
    });
    let expected = theory(vec![
        macros_kept, TheoryItem::Rule(a_e), TheoryItem::Rule(b_e), restr_e, lem_e,
    ]);
    assert_eq!(out, expected);
}

// ---- Q9/Q18: transitive chain a -> b -> c -> <x,x> ------------------------

#[test]
fn transitive_chain() {
    let x = var("x", SortHint::Untagged);
    let macros = TheoryItem::Macros(vec![
        mdef("c", &[x.clone()], pair(vec![msg("x"), msg("x")])),
        mdef("b", &[x.clone()], app("c", vec![msg("x")])),
        mdef("a", &[x.clone()], app("b", vec![msg("x")])),
    ]);
    let t = theory(vec![
        macros,
        TheoryItem::Rule(rule_act("R", vec![fact("In", vec![msg("x")])],
                                  fact("Act", vec![app("a", vec![msg("x")])]))),
    ]);
    assert_eq!(only_rule_action_arg(&expand(&t)), pair(vec![msg("x"), msg("x")]));
}

// ---- Q15: arg already packed to a pair at AST level -----------------------

#[test]
fn over_application_packed_pair() {
    // m(x) = <x,x>; call already parsed as m(<a,b,c>)  ==>  <<a,b,c>,<a,b,c>>
    let x = var("x", SortHint::Untagged);
    let packed = pair(vec![msg("a"), msg("b"), msg("c")]);
    let t = theory(vec![
        TheoryItem::Macros(vec![mdef("m", &[x.clone()], pair(vec![msg("x"), msg("x")]))]),
        TheoryItem::Rule(rule_act(
            "R",
            vec![fact("In", vec![msg("a")]), fact("In", vec![msg("b")]), fact("In", vec![msg("c")])],
            fact("Act", vec![app("m", vec![packed.clone()])]),
        )),
    ]);
    assert_eq!(only_rule_action_arg(&expand(&t)), pair(vec![packed.clone(), packed]));
}

// ---- Q27/Q28: sort-sensitive matching; unmatched formal arg is dropped -----

#[test]
fn sort_sensitive_matching() {
    // m(x) = ~x ;  n(x) = x
    // M(m(a)) ==> M(~x)   (formal x[Untagged] != body ~x[Fresh]; arg a dropped)
    // N(n(~b)) ==> N(~b)  (formal x matches body x; arg ~b substituted)
    let x = var("x", SortHint::Untagged);
    let macros = TheoryItem::Macros(vec![
        mdef("m", &[x.clone()], fresh("x")),
        mdef("n", &[x.clone()], msg("x")),
    ]);
    let r = Rule {
        name: "R".into(), modulo: None, attributes: vec![], let_block: vec![],
        premises: vec![fact("In", vec![msg("a")]), fact("Fr", vec![fresh("b")])],
        actions: vec![
            fact("M", vec![app("m", vec![msg("a")])]),
            fact("N", vec![app("n", vec![fresh("b")])]),
        ],
        conclusions: vec![], embedded_restrictions: vec![], variants: vec![], left_right: None,
    };
    let out = expand(&theory(vec![macros, TheoryItem::Rule(r)]));
    let acts = match out.items.iter().find_map(|it| match it {
        TheoryItem::Rule(r) => Some(r), _ => None }).unwrap() {
        r => r.actions.clone(),
    };
    assert_eq!(acts[0], fact("M", vec![fresh("x")]));   // ~x survives, a dropped
    assert_eq!(acts[1], fact("N", vec![fresh("b")]));   // ~b substituted for x
}

#[test]
fn pub_sort_not_matched() {
    // p(x) = $x ; P(p(a)) ==> P($x)
    let x = var("x", SortHint::Untagged);
    let t = theory(vec![
        TheoryItem::Macros(vec![mdef("p", &[x.clone()], pubv("x"))]),
        TheoryItem::Rule(rule_act("R", vec![fact("In", vec![msg("a")])],
                                  fact("P", vec![app("p", vec![msg("a")])]))),
    ]);
    assert_eq!(only_rule_action_arg(&expand(&t)), pubv("x"));
}

// ---- Q10: nullary macro; free var in body captured -----------------------

#[test]
fn nullary_macro_free_var_body() {
    // z() = ~x ; rule [Fr(~x)] --[Act(z())]-> [Out(z())]
    let macros = TheoryItem::Macros(vec![mdef("z", &[], fresh("x"))]);
    let r = Rule {
        name: "R".into(), modulo: None, attributes: vec![], let_block: vec![],
        premises: vec![fact("Fr", vec![fresh("x")])],
        actions: vec![fact("Act", vec![app("z", vec![])])],
        conclusions: vec![fact("Out", vec![app("z", vec![])])],
        embedded_restrictions: vec![], variants: vec![], left_right: None,
    };
    let out = expand(&theory(vec![macros, TheoryItem::Rule(r)]));
    let rr = out.items.iter().find_map(|it| match it { TheoryItem::Rule(r) => Some(r), _ => None }).unwrap();
    assert_eq!(rr.actions[0], fact("Act", vec![fresh("x")]));
    assert_eq!(rr.conclusions[0], fact("Out", vec![fresh("x")]));
}

// ---- Q3: exp/pub literal (issue777 pk(x)='g'^x) ---------------------------

#[test]
fn exp_and_pub_literal_binop() {
    // pk(x) = 'g'^x ; Out(pk(~x)) ==> Out('g'^~x)
    let x = var("x", SortHint::Untagged);
    let body = Term::BinOp(BinOp::Exp, Box::new(Term::PubLit("g".into())), Box::new(msg("x")));
    let r = Rule {
        name: "A".into(), modulo: None, attributes: vec![], let_block: vec![],
        premises: vec![fact("Fr", vec![fresh("x")])],
        actions: vec![],
        conclusions: vec![fact("Out", vec![app("pk", vec![fresh("x")])])],
        embedded_restrictions: vec![], variants: vec![], left_right: None,
    };
    let out = expand(&theory(vec![
        TheoryItem::Macros(vec![mdef("pk", &[x.clone()], body)]),
        TheoryItem::Rule(r),
    ]));
    let rr = out.items.iter().find_map(|it| match it { TheoryItem::Rule(r) => Some(r), _ => None }).unwrap();
    let expected = Term::BinOp(BinOp::Exp, Box::new(Term::PubLit("g".into())), Box::new(fresh("x")));
    assert_eq!(rr.conclusions[0], fact("Out", vec![expected]));
}

#[test]
fn algapp_path_is_recursed() {
    // Mirror of the exp case using the AlgApp representation to cover that arm.
    let x = var("x", SortHint::Untagged);
    let body = Term::AlgApp("exp".into(), Box::new(Term::PubLit("g".into())), Box::new(msg("x")));
    let t = theory(vec![
        TheoryItem::Macros(vec![mdef("pk", &[x.clone()], body)]),
        TheoryItem::Rule(rule_act("A", vec![fact("Fr", vec![fresh("x")])],
                                  fact("Act", vec![app("pk", vec![fresh("x")])]))),
    ]);
    let expected = Term::AlgApp("exp".into(), Box::new(Term::PubLit("g".into())), Box::new(fresh("x")));
    assert_eq!(only_rule_action_arg(&expand(&t)), expected);
}

// ---- Q26: macro expands inside a diff() term (projection is separate) ------

#[test]
fn expands_inside_diff() {
    let x = var("x", SortHint::Untagged);
    let call = Term::Diff(Box::new(msg("x")), Box::new(app("m", vec![msg("x")])));
    let t = theory(vec![
        TheoryItem::Macros(vec![mdef("m", &[x.clone()], pair(vec![msg("x"), msg("x")]))]),
        TheoryItem::Rule(rule_act("R", vec![fact("In", vec![msg("x")])],
                                  fact("Act", vec![call]))),
    ]);
    let expected = Term::Diff(Box::new(msg("x")), Box::new(pair(vec![msg("x"), msg("x")])));
    assert_eq!(only_rule_action_arg(&expand(&t)), expected);
}

// ---- Q29: macro expands inside a Sapic process ----------------------------

#[test]
fn expands_inside_process() {
    let x = var("x", SortHint::Untagged);
    // process: event A(m(y)); 0
    let proc = Process::Action {
        action: SapicAction::Event(fact("A", vec![app("m", vec![msg("y")])])),
        body: Box::new(Process::Null),
    };
    let out = expand(&theory(vec![
        TheoryItem::Macros(vec![mdef("m", &[x.clone()], pair(vec![msg("x"), msg("x")]))]),
        TheoryItem::TopLevelProcess(proc),
    ]));
    let got = out.items.iter().find_map(|it| match it {
        TheoryItem::TopLevelProcess(p) => Some(p.clone()), _ => None }).unwrap();
    let expected = Process::Action {
        action: SapicAction::Event(fact("A", vec![pair(vec![msg("y"), msg("y")])])),
        body: Box::new(Process::Null),
    };
    assert_eq!(got, expected);
}

// ---- defensive: arity mismatch at AST level -> unexpanded -----------------

#[test]
fn arity_mismatch_left_unexpanded() {
    // m expects 2 formals but the call carries 1 arg (should not happen from a
    // valid parse). expand() must not panic; it leaves the call in place.
    let (x, y) = (var("x", SortHint::Untagged), var("y", SortHint::Untagged));
    let t = theory(vec![
        TheoryItem::Macros(vec![mdef("m", &[x.clone(), y.clone()], pair(vec![msg("x"), msg("y")]))]),
        TheoryItem::Rule(rule_act("R", vec![fact("In", vec![msg("a")])],
                                  fact("Act", vec![app("m", vec![msg("a")])]))),
    ]);
    assert_eq!(only_rule_action_arg(&expand(&t)), app("m", vec![msg("a")]));
}

// ---- non-macro App is preserved, its args expanded ------------------------

#[test]
fn non_macro_app_preserved() {
    // h is NOT a macro (no such macro defined); m is. h(m(x)) => h(<x,x>)
    let x = var("x", SortHint::Untagged);
    let t = theory(vec![
        TheoryItem::Macros(vec![mdef("m", &[x.clone()], pair(vec![msg("x"), msg("x")]))]),
        TheoryItem::Rule(rule_act("R", vec![fact("In", vec![msg("x")])],
                                  fact("Act", vec![app("h", vec![app("m", vec![msg("x")])])]))),
    ]);
    assert_eq!(only_rule_action_arg(&expand(&t)),
               app("h", vec![pair(vec![msg("x"), msg("x")])]));
}

// ===========================================================================
// GAP 1 — bare nullary macro uses (a macro name written without parentheses)
// ===========================================================================

fn publit(s: &str) -> Term {
    Term::PubLit(s.into())
}

// ---- Q32/Q33: a bare untagged name equal to a nullary macro resolves ------

#[test]
fn bare_nullary_untagged_resolves() {
    // konst() = h('k') ; the bare name `konst` (untagged, no parens) resolves
    // to the body h('k') in premise, action and conclusion positions [Q32,Q33].
    let body = app("h", vec![publit("k")]);
    let macros = TheoryItem::Macros(vec![mdef("konst", &[], body.clone())]);
    let r = Rule {
        name: "R".into(), modulo: None, attributes: vec![], let_block: vec![],
        premises: vec![fact("In", vec![msg("konst")])],
        actions: vec![fact("A", vec![msg("konst")])],
        conclusions: vec![fact("Out", vec![msg("konst")])],
        embedded_restrictions: vec![], variants: vec![], left_right: None,
    };
    let out = expand(&theory(vec![macros, TheoryItem::Rule(r)]));
    let rr = out.items.iter().find_map(|it| match it {
        TheoryItem::Rule(r) => Some(r), _ => None }).unwrap();
    assert_eq!(rr.premises[0], fact("In", vec![body.clone()]));
    assert_eq!(rr.actions[0], fact("A", vec![body.clone()]));
    assert_eq!(rr.conclusions[0], fact("Out", vec![body]));
}

// ---- Q34: a bare fresh/pub-sorted name is NOT a nullary-macro use ----------

#[test]
fn bare_nullary_wrong_sort_not_resolved() {
    // konst() = h('k') ; ~konst (fresh) and $konst (pub) stay ordinary vars.
    let body = app("h", vec![publit("k")]);
    let macros = TheoryItem::Macros(vec![mdef("konst", &[], body)]);
    let r = Rule {
        name: "R".into(), modulo: None, attributes: vec![], let_block: vec![],
        premises: vec![fact("In", vec![msg("x")])],
        actions: vec![fact("A", vec![fresh("konst")]), fact("B", vec![pubv("konst")])],
        conclusions: vec![], embedded_restrictions: vec![], variants: vec![], left_right: None,
    };
    let out = expand(&theory(vec![macros, TheoryItem::Rule(r)]));
    let rr = out.items.iter().find_map(|it| match it {
        TheoryItem::Rule(r) => Some(r), _ => None }).unwrap();
    assert_eq!(rr.actions[0], fact("A", vec![fresh("konst")]));
    assert_eq!(rr.actions[1], fact("B", vec![pubv("konst")]));
}

// ---- Q35: a bare name equal to an arity>=1 macro is NOT a use --------------

#[test]
fn bare_name_nonnullary_not_resolved() {
    // m(x) = <x,x> ; the bare name `m` (no args) is left as an ordinary var.
    let x = var("x", SortHint::Untagged);
    let t = theory(vec![
        TheoryItem::Macros(vec![mdef("m", &[x.clone()], pair(vec![msg("x"), msg("x")]))]),
        TheoryItem::Rule(rule_act("R", vec![fact("In", vec![msg("x")])],
                                  fact("A", vec![msg("m")]))),
    ]);
    assert_eq!(only_rule_action_arg(&expand(&t)), msg("m"));
}

// ---- Q33: a bare nullary use inside another macro body expands transitively -

#[test]
fn bare_nullary_transitive_in_body() {
    // base() = h('k') ; wrap() = h(base) ; bare `wrap` => h(h('k')) [Q33].
    let base_body = app("h", vec![publit("k")]);
    let wrap_body = app("h", vec![msg("base")]); // `base` bare in wrap's body
    let macros = TheoryItem::Macros(vec![
        mdef("base", &[], base_body.clone()),
        mdef("wrap", &[], wrap_body),
    ]);
    let t = theory(vec![
        macros,
        TheoryItem::Rule(rule_act("R", vec![fact("In", vec![msg("x")])],
                                  fact("A", vec![msg("wrap")]))),
    ]);
    assert_eq!(only_rule_action_arg(&expand(&t)), app("h", vec![base_body]));
}

// ---- Q36: a nullary macro reserves its name against a same-named formal ----

#[test]
fn nullary_macro_reserves_name_over_formal() {
    // base() = h('k') ; f(base) = <base,base> ; f(a) => <h('k'),h('k')> (the
    // nullary macro wins inside the body; the formal `base` and arg `a` are
    // dropped) [Q36].
    let hk = app("h", vec![publit("k")]);
    let base_formal = var("base", SortHint::Untagged);
    let macros = TheoryItem::Macros(vec![
        mdef("base", &[], hk.clone()),
        mdef("f", &[base_formal], pair(vec![msg("base"), msg("base")])),
    ]);
    let t = theory(vec![
        macros,
        TheoryItem::Rule(rule_act("R", vec![fact("In", vec![msg("a")])],
                                  fact("A", vec![app("f", vec![msg("a")])]))),
    ]);
    assert_eq!(only_rule_action_arg(&expand(&t)), pair(vec![hk.clone(), hk]));
}

// ===========================================================================
// GAP 2/3 — macros inside accountability-lemma and case-test formulas
// ===========================================================================

/// A formula `Ex sid #i. <fact>(<arg>)@i`, used to carry a macro call.
fn exists_action(fname: &str, arg: Term) -> Formula {
    Formula::Exists(
        vec![var("sid", SortHint::Untagged), var("i", SortHint::Node)],
        Box::new(Formula::Atom(Atom::Action(
            fact(fname, vec![arg]),
            Term::Var(var("i", SortHint::Node)),
        ))),
    )
}

// ---- Q38: a macro call in an accountability-lemma formula is expanded ------

#[test]
fn acc_lemma_formula_expanded() {
    // af(x) = <x,x> ; acc-lemma formula uses af(sid); expand rewrites it while
    // keeping the case_test_idents and the preserved macros block [Q38].
    let x = var("x", SortHint::Untagged);
    let macros = TheoryItem::Macros(vec![mdef("af", &[x.clone()], pair(vec![msg("x"), msg("x")]))]);
    let acc = TheoryItem::AccLemma(AccLemma {
        name: "acc".into(),
        attributes: vec![],
        formula: exists_action("Unequal", app("af", vec![msg("sid")])),
        case_test_idents: vec!["blamed".into()],
    });
    let out = expand(&theory(vec![macros.clone(), acc]));
    // macros preserved in place
    assert_eq!(out.items[0], macros);
    let got = out.items.iter().find_map(|it| match it {
        TheoryItem::AccLemma(a) => Some(a.clone()), _ => None }).unwrap();
    assert_eq!(got.formula, exists_action("Unequal", pair(vec![msg("sid"), msg("sid")])));
    assert_eq!(got.case_test_idents, vec!["blamed".to_string()]);
}

// ---- Q39: a macro call in a case-test formula is expanded ------------------

#[test]
fn case_test_formula_expanded() {
    // ct(x) = <x,x> ; case-test formula uses ct(sid); expand rewrites it [Q39].
    let x = var("x", SortHint::Untagged);
    let macros = TheoryItem::Macros(vec![mdef("ct", &[x.clone()], pair(vec![msg("x"), msg("x")]))]);
    let ctest = TheoryItem::CaseTest(CaseTest {
        name: "blamed".into(),
        formula: exists_action("Blame", app("ct", vec![msg("sid")])),
    });
    let out = expand(&theory(vec![macros, ctest]));
    let got = out.items.iter().find_map(|it| match it {
        TheoryItem::CaseTest(c) => Some(c.clone()), _ => None }).unwrap();
    assert_eq!(got.formula, exists_action("Blame", pair(vec![msg("sid"), msg("sid")])));
}

// ===========================================================================
// ROUND 5 — staged (parse-stage) mode: `expand_staged`
//
// The consumer's pipeline calls `expand_staged` at a stage where a later stage
// owns acc-lemma & case-test expansion [Q41] and rules exist only in their
// primary form [Q42]. So `expand_staged` leaves acc-lemma / case-test formulas
// and derived variant / left-right rule forms UNTOUCHED, while still fully
// expanding ordinary lemmas / restrictions / the primary rule form and resolving
// bare-nullary uses. Full close (`expand`) stays unchanged (its behaviour is
// pinned by every test above).
// ===========================================================================

/// A one-restriction/one-lemma/one-rule + acc-lemma + case-test + bare-nullary
/// theory exercising every staged carve-out at once.
fn staged_mixed_theory() -> (Theory, TheoryItem) {
    let x = var("x", SortHint::Untagged);
    let macros = TheoryItem::Macros(vec![
        mdef("m", &[x.clone()], pair(vec![msg("x"), msg("x")])),
        mdef("konst", &[], app("h", vec![publit("k")])),
    ]);
    // ordinary rule (primary form): [In(x)] --[A(m(x))]-> [Out(konst)]
    let r = Rule {
        name: "R".into(), modulo: None, attributes: vec![], let_block: vec![],
        premises: vec![fact("In", vec![msg("x")])],
        actions: vec![fact("A", vec![app("m", vec![msg("x")])])],
        conclusions: vec![fact("Out", vec![msg("konst")])], // bare-nullary use
        embedded_restrictions: vec![], variants: vec![], left_right: None,
    };
    // ordinary restriction: All x #i. A(m(x))@i ==> Ex y. x=y
    let y = var("y", SortHint::Untagged);
    let restr = TheoryItem::Restriction(Restriction {
        name: "Restr".into(), attributes: vec![],
        formula: Formula::Forall(
            vec![x.clone(), var("i", SortHint::Node)],
            Box::new(Formula::Implies(
                Box::new(Formula::Atom(Atom::Action(
                    fact("A", vec![app("m", vec![msg("x")])]),
                    Term::Var(var("i", SortHint::Node)),
                ))),
                Box::new(Formula::Exists(vec![y.clone()],
                    Box::new(Formula::Atom(Atom::Eq(msg("x"), msg("y")))))),
            )),
        ),
    });
    // ordinary lemma: exists-trace Ex x #i. A(m(x))@i
    let lem = TheoryItem::Lemma(Lemma {
        name: "L".into(), modulo: None, attributes: vec![],
        trace_quantifier: TraceQuantifier::ExistsTrace,
        formula: exists_action("A", app("m", vec![msg("sid")])),
        proof: None, plaintext: String::new(),
    });
    // acc-lemma & case-test both use the macro in their formula
    let acc = TheoryItem::AccLemma(AccLemma {
        name: "acc".into(), attributes: vec![],
        formula: exists_action("Unequal", app("m", vec![msg("sid")])),
        case_test_idents: vec!["blamed".into()],
    });
    let ctest = TheoryItem::CaseTest(CaseTest {
        name: "blamed".into(),
        formula: exists_action("Blame", app("m", vec![msg("sid")])),
    });
    let t = theory(vec![
        macros.clone(), TheoryItem::Rule(r), restr, lem, acc, ctest,
    ]);
    (t, macros)
}

// ---- acc-lemma & case-test formulas are byte-identical through staged mode --

#[test]
fn staged_leaves_acc_and_case_test_untouched() {
    let (t, _) = staged_mixed_theory();
    let out = expand_staged(&t);
    // The acc-lemma and case-test items are byte-identical to their inputs
    // (whole item unchanged, macro call preserved).
    let acc_in = t.items.iter().find(|it| matches!(it, TheoryItem::AccLemma(_))).unwrap();
    let ct_in = t.items.iter().find(|it| matches!(it, TheoryItem::CaseTest(_))).unwrap();
    let acc_out = out.items.iter().find(|it| matches!(it, TheoryItem::AccLemma(_))).unwrap();
    let ct_out = out.items.iter().find(|it| matches!(it, TheoryItem::CaseTest(_))).unwrap();
    assert_eq!(acc_out, acc_in);
    assert_eq!(ct_out, ct_in);
    // Concretely: the macro call still stands, un-expanded, in both formulas.
    assert_eq!(
        match acc_out { TheoryItem::AccLemma(a) => a.formula.clone(), _ => unreachable!() },
        exists_action("Unequal", app("m", vec![msg("sid")])),
    );
    assert_eq!(
        match ct_out { TheoryItem::CaseTest(c) => c.formula.clone(), _ => unreachable!() },
        exists_action("Blame", app("m", vec![msg("sid")])),
    );
}

// ---- the `macros:` block is preserved in place through staged mode ----------

#[test]
fn staged_preserves_macros_block() {
    let (t, macros) = staged_mixed_theory();
    let out = expand_staged(&t);
    assert_eq!(out.items[0], macros); // unchanged, original bodies, first position
}

// ---- ordinary lemmas / restrictions / (primary) rules still fully expand ----

#[test]
fn staged_expands_ordinary_lemma_restriction_rule() {
    let (t, _) = staged_mixed_theory();
    let out = expand_staged(&t);

    // primary rule form expanded: A(m(x)) -> A(<x,x>)
    let r = out.items.iter().find_map(|it| match it {
        TheoryItem::Rule(r) => Some(r), _ => None }).unwrap();
    assert_eq!(r.actions[0], fact("A", vec![pair(vec![msg("x"), msg("x")])]));

    // restriction formula expanded
    let restr = out.items.iter().find_map(|it| match it {
        TheoryItem::Restriction(r) => Some(r), _ => None }).unwrap();
    let expected_restr = Formula::Forall(
        vec![var("x", SortHint::Untagged), var("i", SortHint::Node)],
        Box::new(Formula::Implies(
            Box::new(Formula::Atom(Atom::Action(
                fact("A", vec![pair(vec![msg("x"), msg("x")])]),
                Term::Var(var("i", SortHint::Node)),
            ))),
            Box::new(Formula::Exists(vec![var("y", SortHint::Untagged)],
                Box::new(Formula::Atom(Atom::Eq(msg("x"), msg("y")))))),
        )),
    );
    assert_eq!(restr.formula, expected_restr);

    // ordinary lemma formula expanded
    let lem = out.items.iter().find_map(|it| match it {
        TheoryItem::Lemma(l) => Some(l), _ => None }).unwrap();
    assert_eq!(lem.formula, exists_action("A", pair(vec![msg("sid"), msg("sid")])));
}

// ---- bare-nullary resolution is still active in staged mode -----------------

#[test]
fn staged_bare_nullary_still_resolves() {
    let (t, _) = staged_mixed_theory();
    let out = expand_staged(&t);
    let r = out.items.iter().find_map(|it| match it {
        TheoryItem::Rule(r) => Some(r), _ => None }).unwrap();
    // Out(konst) -> Out(h('k'))
    assert_eq!(r.conclusions[0], fact("Out", vec![app("h", vec![publit("k")])]));
}

// ---- staged mode touches ONLY the primary rule form, not derived variants ---

#[test]
fn staged_does_not_recurse_into_rule_variants() {
    // A rule carrying a `(modulo AC)` variant (and a left/right diff form), each
    // holding a macro call. Staged mode expands the primary form but leaves the
    // derived forms verbatim [Q42]; full close expands all of them.
    let x = var("x", SortHint::Untagged);
    let macros = TheoryItem::Macros(vec![mdef("m", &[x.clone()], pair(vec![msg("x"), msg("x")]))]);
    let variant = rule_act("R", vec![], fact("A", vec![app("m", vec![msg("v")])]));
    let lr_side = rule_act("R", vec![], fact("A", vec![app("m", vec![msg("l")])]));
    let primary = Rule {
        name: "R".into(), modulo: None, attributes: vec![], let_block: vec![],
        premises: vec![], actions: vec![fact("A", vec![app("m", vec![msg("p")])])],
        conclusions: vec![], embedded_restrictions: vec![],
        variants: vec![variant.clone()],
        left_right: Some((Box::new(lr_side.clone()), Box::new(lr_side.clone()))),
    };
    let t = theory(vec![macros, TheoryItem::Rule(primary)]);

    // staged: primary expanded, variant + left/right untouched.
    let staged = expand_staged(&t);
    let sr = staged.items.iter().find_map(|it| match it {
        TheoryItem::Rule(r) => Some(r), _ => None }).unwrap();
    assert_eq!(sr.actions[0], fact("A", vec![pair(vec![msg("p"), msg("p")])])); // primary expanded
    assert_eq!(sr.variants, vec![variant.clone()]);                             // variant verbatim
    assert_eq!(
        sr.left_right,
        Some((Box::new(lr_side.clone()), Box::new(lr_side.clone()))),           // diff form verbatim
    );

    // full close: every form expanded (contrast — pins the mode difference).
    let full = expand(&t);
    let fr = full.items.iter().find_map(|it| match it {
        TheoryItem::Rule(r) => Some(r), _ => None }).unwrap();
    assert_eq!(fr.actions[0], fact("A", vec![pair(vec![msg("p"), msg("p")])]));
    assert_eq!(fr.variants[0].actions[0], fact("A", vec![pair(vec![msg("v"), msg("v")])]));
    let (fl, _) = fr.left_right.as_ref().unwrap();
    assert_eq!(fl.actions[0], fact("A", vec![pair(vec![msg("l"), msg("l")])]));
}
