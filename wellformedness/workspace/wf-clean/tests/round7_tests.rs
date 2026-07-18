//! Round-7 tests: the PER-FINDING entry count of `check_theory`.
//!
//! The integrated binary's trailing batch summary prints
//! `  WARNING: <N> wellformedness check failed!` where N is the LENGTH of the
//! report list (`check_theory(thy).len()` plus the caller-added out-of-scope
//! findings). Each of these tests builds the AST of a probe or corpus file and
//! asserts the multiset of findings `check_theory` returns, at the granularity
//! observed from the oracle footer (probes logged in workspace/QUERIES.log
//! under "ROUND 7"; the per-topic counting law is documented in BEHAVIOR.md).
//!
//! Byte-identity of the rendered block is covered by the round3-6 fixtures;
//! here we pin the COUNT. For issue515/issue527 we additionally re-assert the
//! byte-exact block (reusing the round5 fixtures) so an AST-copy typo cannot
//! silently change the multiset.

use std::collections::BTreeMap;
use wf_clean::ast::*;
use wf_clean::checks;
use wf_clean::*;

// ---- AST builders ---------------------------------------------------------

fn var(name: &str, sort: SortHint) -> VarSpec {
    VarSpec { name: name.into(), idx: 0, sort, typ: None }
}
fn mv(name: &str) -> Term { Term::Var(var(name, SortHint::Msg)) }
fn fresh(name: &str) -> Term { Term::Var(var(name, SortHint::Fresh)) }
fn pub_(name: &str) -> Term { Term::Var(var(name, SortHint::Pub)) }
fn node(name: &str) -> Term { Term::Var(var(name, SortHint::Node)) }
fn pl(name: &str) -> Term { Term::PubLit(name.into()) }
fn fl(name: &str) -> Term { Term::FreshLit(name.into()) }
fn app(name: &str, args: Vec<Term>) -> Term { Term::App(name.into(), args) }
fn pair(items: Vec<Term>) -> Term { Term::Pair(items) }
fn mult(a: Term, b: Term) -> Term { Term::BinOp(BinOp::Mult, Box::new(a), Box::new(b)) }

fn fact(name: &str, args: Vec<Term>) -> Fact {
    Fact { persistent: false, name: name.into(), args, annotations: vec![] }
}
fn pfact(name: &str, args: Vec<Term>) -> Fact {
    Fact { persistent: true, name: name.into(), args, annotations: vec![] }
}

fn rule(name: &str, prem: Vec<Fact>, act: Vec<Fact>, concl: Vec<Fact>) -> TheoryItem {
    rule_let(name, vec![], prem, act, concl)
}
fn rule_let(
    name: &str,
    let_block: Vec<LetBinding>,
    prem: Vec<Fact>,
    act: Vec<Fact>,
    concl: Vec<Fact>,
) -> TheoryItem {
    TheoryItem::Rule(Rule {
        name: name.into(),
        modulo: None,
        attributes: vec![],
        let_block,
        premises: prem,
        actions: act,
        conclusions: concl,
        embedded_restrictions: vec![],
        variants: vec![],
        left_right: None,
    })
}

fn lemma(name: &str, tq: TraceQuantifier, attrs: Vec<LemmaAttr>, formula: Formula) -> TheoryItem {
    TheoryItem::Lemma(Lemma {
        name: name.into(),
        modulo: None,
        attributes: attrs,
        trace_quantifier: tq,
        formula,
        proof: None,
        plaintext: String::new(),
    })
}

fn theory(name: &str, items: Vec<TheoryItem>) -> Theory {
    Theory { is_diff: false, name: name.into(), configuration: None, items }
}

// ---- Formula builders -----------------------------------------------------

fn action(name: &str, args: Vec<Term>, tv: &str) -> Formula {
    Formula::Atom(Atom::Action(fact(name, args), node(tv)))
}
fn imp(a: Formula, b: Formula) -> Formula {
    Formula::Implies(Box::new(a), Box::new(b))
}
fn forall(vs: Vec<VarSpec>, body: Formula) -> Formula {
    Formula::Forall(vs, Box::new(body))
}
fn exists(vs: Vec<VarSpec>, body: Formula) -> Formula {
    Formula::Exists(vs, Box::new(body))
}

// ---- Assertion helper -----------------------------------------------------

/// The multiset of topics present in a report (a topic that hosts several
/// findings appears with that count).
fn topic_counts(r: &WfReport) -> BTreeMap<String, usize> {
    let mut m = BTreeMap::new();
    for e in r {
        *m.entry(e.topic.clone()).or_insert(0) += 1;
    }
    m
}

/// Assert `check_theory` returns exactly the given per-topic finding multiset
/// (and hence `expected.iter().map(|(_, n)| n).sum()` total entries, which is
/// the wf portion of the batch footer count).
fn expect_counts(thy: &Theory, expected: &[(&str, usize)]) {
    let r = check_theory(thy);
    let got = topic_counts(&r);
    let want: BTreeMap<String, usize> =
        expected.iter().map(|(t, n)| (t.to_string(), *n)).collect();
    assert_eq!(got, want, "topic multiset mismatch; report = {:#?}", r);
    let total: usize = expected.iter().map(|(_, n)| n).sum();
    assert_eq!(r.len(), total, "entry count (footer N) mismatch");
}

// ===========================================================================
// PROBE FIXTURES (round7/probes) — the observed footer minus out-of-scope
// findings (Message Derivation Checks). See QUERIES.log ROUND 7.
// ===========================================================================

// -- Unbound variables: one finding PER RULE (probes deriv_1/deriv_2var/deriv_2rule)
#[test]
fn count_unbound_one_rule_one_var() {
    // deriv_1: rule R: [] --[A(x)]-> [Out(x)]  (footer 2 = Unbound 1 + MsgDeriv 1)
    let thy = theory("deriv_1", vec![rule(
        "R", vec![], vec![fact("A", vec![mv("x")])], vec![fact("Out", vec![mv("x")])],
    )]);
    expect_counts(&thy, &[(checks::T_UNBOUND, 1)]);
}

#[test]
fn count_unbound_one_rule_two_vars_is_one() {
    // deriv_2var: two unbound vars in ONE rule -> still one finding.
    let thy = theory("deriv_2var", vec![rule(
        "R",
        vec![],
        vec![fact("A", vec![mv("x"), mv("y")])],
        vec![fact("Out", vec![pair(vec![mv("x"), mv("y")])])],
    )]);
    expect_counts(&thy, &[(checks::T_UNBOUND, 1)]);
}

#[test]
fn count_unbound_two_rules_is_two() {
    // deriv_2rule: two rules each with an unbound var -> two findings.
    let thy = theory("deriv_2rule", vec![
        rule("R1", vec![], vec![fact("A", vec![mv("x")])], vec![fact("Out", vec![mv("x")])]),
        rule("R2", vec![], vec![fact("B", vec![mv("y")])], vec![fact("Out", vec![mv("y")])]),
    ]);
    expect_counts(&thy, &[(checks::T_UNBOUND, 2)]);
}

// -- Variable with mismatching sorts: one finding PER RULE (probes sort_*)
#[test]
fn count_sort_one_rule() {
    // sort_1rule: [Fr(~x), In(x)] --[]-> [Out(x)]  (~x vs x, one rule)
    let thy = theory("sort_1rule", vec![rule(
        "R",
        vec![fact("Fr", vec![fresh("x")]), fact("In", vec![mv("x")])],
        vec![],
        vec![fact("Out", vec![mv("x")])],
    )]);
    expect_counts(&thy, &[(checks::T_SORTS, 1)]);
}

#[test]
fn count_sort_two_groups_one_rule_is_one() {
    // sort_2grp_1rule: two variant groups in one rule -> one finding.
    let thy = theory("sort_2grp_1rule", vec![rule(
        "R",
        vec![
            fact("Fr", vec![fresh("x")]),
            fact("In", vec![mv("x")]),
            fact("Fr", vec![fresh("z")]),
            fact("In", vec![mv("z")]),
        ],
        vec![],
        vec![fact("Out", vec![pair(vec![mv("x"), mv("z")])])],
    )]);
    expect_counts(&thy, &[(checks::T_SORTS, 1)]);
}

#[test]
fn count_sort_two_rules_is_two() {
    // sort_2rule: two rules each with a conflict -> two findings.
    let thy = theory("sort_2rule", vec![
        rule("R1", vec![fact("Fr", vec![fresh("x")]), fact("In", vec![mv("x")])], vec![], vec![fact("Out", vec![mv("x")])]),
        rule("R2", vec![fact("Fr", vec![fresh("y")]), fact("In", vec![mv("y")])], vec![], vec![fact("Out", vec![mv("y")])]),
    ]);
    expect_counts(&thy, &[(checks::T_SORTS, 2)]);
}

// -- Reserved names: one finding PER (rule, side) (probe reserved_2fact_1side)
#[test]
fn count_reserved_two_facts_one_side_is_one_plus_lhs() {
    // reserved_2fact_1side: [K(x), KU(x)] --[]-> [Out(x)]  (footer 2 =
    // Reserved 1 [both facts on the left] + lhs-not-rhs 1 [K premise]).
    let thy = theory("reserved_2fact_1side", vec![rule(
        "R",
        vec![fact("K", vec![mv("x")]), fact("KU", vec![mv("x")])],
        vec![],
        vec![fact("Out", vec![mv("x")])],
    )]);
    expect_counts(&thy, &[(checks::T_RESERVED, 1), (checks::T_LHSRHS, 1)]);
}

// -- Special facts: one finding PER (rule, side) (probe special_2fact_1side)
#[test]
fn count_special_facts_per_side() {
    // special_2fact_1side: [Fr(~x)] --[]-> [In(~x), Out(x)]  (footer 4 =
    // Unbound 1 [msg x] + Sort 1 [~x vs x] + Special 1 [In on RHS] + MsgDeriv 1).
    let thy = theory("special_2fact_1side", vec![rule(
        "R",
        vec![fact("Fr", vec![fresh("x")])],
        vec![],
        vec![fact("In", vec![fresh("x")]), fact("Out", vec![mv("x")])],
    )]);
    expect_counts(&thy, &[
        (checks::T_UNBOUND, 1),
        (checks::T_SORTS, 1),
        (checks::T_SPECIAL, 1),
    ]);
}

// -- lhs-not-rhs: WHOLE-TOPIC one finding regardless of item count (probes lhs_1/lhs_2)
#[test]
fn count_lhs_not_rhs_one_item() {
    let thy = theory("lhs_1", vec![rule("R", vec![fact("Foo", vec![pl("a")])], vec![], vec![])]);
    expect_counts(&thy, &[(checks::T_LHSRHS, 1)]);
}

#[test]
fn count_lhs_not_rhs_two_items_still_one() {
    let thy = theory("lhs_2", vec![rule(
        "R", vec![fact("Foo", vec![pl("a")]), fact("Bar", vec![pl("b")])], vec![], vec![],
    )]);
    expect_counts(&thy, &[(checks::T_LHSRHS, 1)]);
}

// -- Public-constants / fact-capitalization / fact-arity / fact-multiplicity /
//    subterm: WHOLE-TOPIC one finding regardless of group count.
#[test]
fn count_public_constants_whole_topic() {
    // pub_1grp and pub_2grp both footer 1 (one group / two groups -> one).
    let one = theory("pub_1grp", vec![rule(
        "R", vec![], vec![fact("A", vec![pl("C"), pl("c")])], vec![fact("Out", vec![pl("C")])],
    )]);
    expect_counts(&one, &[(checks::T_PUBNAMES, 1)]);
    let two = theory("pub_2grp", vec![rule(
        "R", vec![], vec![fact("A", vec![pl("C"), pl("c"), pl("D"), pl("d")])], vec![fact("Out", vec![pl("C")])],
    )]);
    expect_counts(&two, &[(checks::T_PUBNAMES, 1)]);
}

#[test]
fn count_fact_capitalization_whole_topic() {
    // fcap_1grp / fcap_1grp_3occ / fcap_2grp all footer 1.
    let one = theory("fcap_1grp", vec![
        rule("R1", vec![], vec![], vec![fact("Send", vec![pl("a")])]),
        rule("R2", vec![], vec![], vec![fact("SEND", vec![pl("a")])]),
    ]);
    expect_counts(&one, &[(checks::T_FACT_CAP, 1)]);
    let two = theory("fcap_2grp", vec![
        rule("R1", vec![], vec![], vec![fact("Send", vec![pl("a")]), fact("Recv", vec![pl("a")])]),
        rule("R2", vec![], vec![], vec![fact("SEND", vec![pl("a")]), fact("RECV", vec![pl("a")])]),
    ]);
    expect_counts(&two, &[(checks::T_FACT_CAP, 1)]);
}

#[test]
fn count_fact_arity_whole_topic() {
    // arity_1grp / arity_2grp both footer 1.
    let two = theory("arity_2grp", vec![
        rule("R1", vec![], vec![], vec![fact("Foo", vec![pl("a")]), fact("Bar", vec![pl("a")])]),
        rule("R2", vec![], vec![], vec![fact("Foo", vec![pl("a"), pl("b")]), fact("Bar", vec![pl("a"), pl("b")])]),
    ]);
    expect_counts(&two, &[(checks::T_ARITY, 1)]);
}

#[test]
fn count_fact_multiplicity_whole_topic() {
    // mult_2grp: two multiplicity groups -> one finding.
    let thy = theory("mult_2grp", vec![
        rule("R1", vec![], vec![], vec![pfact("Foo", vec![pl("a")]), pfact("Bar", vec![pl("a")])]),
        rule("R2", vec![], vec![], vec![fact("Foo", vec![pl("a")]), fact("Bar", vec![pl("a")])]),
    ]);
    expect_counts(&thy, &[(checks::T_MULT, 1)]);
}

#[test]
fn count_subterm_whole_topic() {
    // subterm_2eq: two non-convergent equations -> one finding.
    let thy = theory("subterm_2eq", vec![
        TheoryItem::Equations {
            convergent: false,
            eqs: vec![
                Equation { lhs: app("f", vec![mv("x")]), rhs: app("h", vec![mv("x"), mv("x")]) },
                Equation { lhs: app("g", vec![mv("y")]), rhs: app("h", vec![mv("y"), mv("y")]) },
            ],
        },
        rule("R", vec![fact("In", vec![mv("x")])], vec![], vec![fact("Out", vec![app("f", vec![mv("x")])])]),
    ]);
    expect_counts(&thy, &[(checks::T_SUBTERM, 1)]);
}

// -- Fr facts: one finding PER offending Fr fact (probe fr_2fact_1rule)
#[test]
fn count_fr_facts_per_fact() {
    let thy = theory("fr_2fact_1rule", vec![rule(
        "R",
        vec![fact("Fr", vec![pub_("a")]), fact("Fr", vec![pub_("b")])],
        vec![],
        vec![fact("Out", vec![pair(vec![pub_("a"), pub_("b")])])],
    )]);
    expect_counts(&thy, &[(checks::T_FR, 2)]);
}

// -- Fresh public constants: one finding PER RULE (probes freshpub_*)
#[test]
fn count_fresh_public_constants_per_rule() {
    let two = theory("freshpub_2rule", vec![
        rule("R1", vec![], vec![], vec![fact("Out", vec![fl("a")])]),
        rule("R2", vec![], vec![], vec![fact("Out", vec![fl("b")])]),
    ]);
    expect_counts(&two, &[(checks::T_FRESH_PUB, 2)]);
    let one = theory("freshpub_2const_1rule", vec![rule(
        "R", vec![], vec![], vec![fact("Out", vec![pair(vec![fl("a"), fl("b")])])],
    )]);
    expect_counts(&one, &[(checks::T_FRESH_PUB, 1)]);
}

// -- Multiplication restriction: one finding PER RULE (probe multrestrict_2rule_dh)
#[test]
fn count_multiplication_restriction_per_rule() {
    let thy = theory("multrestrict_2rule_dh", vec![
        rule("R1", vec![fact("In", vec![mv("x")]), fact("In", vec![mv("y")])], vec![], vec![fact("Out", vec![mult(mv("x"), mv("y"))])]),
        rule("R2", vec![fact("In", vec![mv("a")]), fact("In", vec![mv("b")])], vec![], vec![fact("Out", vec![mult(mv("a"), mv("b"))])]),
    ]);
    expect_counts(&thy, &[(checks::T_MULRESTRICT, 2)]);
}

// -- Quantifier sorts / Formula terms: one finding PER FORMULA ITEM (probes qs_*/ft_2lem)
#[test]
fn count_quantifier_sorts_per_item() {
    // qs_1lem / qs_2lem: "All $x #i. (A($x)@#i ==> A($x)@#i)" (pub quantifier).
    let mk = |v: &str, tv: &str| {
        forall(
            vec![var(v, SortHint::Pub), var(tv, SortHint::Node)],
            imp(action("A", vec![pub_(v)], tv), action("A", vec![pub_(v)], tv)),
        )
    };
    let one = theory("qs_1lem", vec![
        rule("R", vec![fact("In", vec![mv("x")])], vec![fact("A", vec![mv("x")])], vec![fact("Out", vec![mv("x")])]),
        lemma("L1", TraceQuantifier::AllTraces, vec![], mk("x", "i")),
    ]);
    expect_counts(&one, &[(checks::T_QUANT_SORTS, 1)]);
    let two = theory("qs_2lem", vec![
        rule("R", vec![fact("In", vec![mv("x")])], vec![fact("A", vec![mv("x")])], vec![fact("Out", vec![mv("x")])]),
        lemma("L1", TraceQuantifier::AllTraces, vec![], mk("x", "i")),
        lemma("L2", TraceQuantifier::AllTraces, vec![], mk("y", "j")),
    ]);
    expect_counts(&two, &[(checks::T_QUANT_SORTS, 2)]);
}

#[test]
fn count_formula_terms_per_item() {
    // ft_2lem: "All #i. (A(y)@#i ==> A(y)@#i)" (y free) in two lemmas.
    let mk = |free: &str, tv: &str| {
        forall(
            vec![var(tv, SortHint::Node)],
            imp(action("A", vec![mv(free)], tv), action("A", vec![mv(free)], tv)),
        )
    };
    let thy = theory("ft_2lem", vec![
        rule("R", vec![fact("In", vec![mv("x")])], vec![fact("A", vec![mv("x")])], vec![fact("Out", vec![mv("x")])]),
        lemma("L1", TraceQuantifier::AllTraces, vec![], mk("y", "i")),
        lemma("L2", TraceQuantifier::AllTraces, vec![], mk("z", "j")),
    ]);
    expect_counts(&thy, &[(checks::T_FORMULA_TERMS, 2)]);
}

// -- Formula guardedness / Lemma annotations: one finding PER LEMMA
#[test]
fn count_guardedness_per_lemma() {
    // guard_2lem: "All x #i. A(x)@#i" (universal without toplevel implication).
    let mk = |v: &str, tv: &str| {
        forall(
            vec![var(v, SortHint::Msg), var(tv, SortHint::Node)],
            action("A", vec![mv(v)], tv),
        )
    };
    let thy = theory("guard_2lem", vec![
        rule("R", vec![fact("In", vec![mv("x")])], vec![fact("A", vec![mv("x")])], vec![fact("Out", vec![mv("x")])]),
        lemma("L1", TraceQuantifier::AllTraces, vec![], mk("x", "i")),
        lemma("L2", TraceQuantifier::AllTraces, vec![], mk("y", "j")),
    ]);
    expect_counts(&thy, &[(checks::T_GUARD, 2)]);
}

#[test]
fn count_lemma_annotations_per_lemma() {
    // lemanno_2lem: two [reuse] exists-trace lemmas.
    let mk = |v: &str, tv: &str| exists(
        vec![var(v, SortHint::Msg), var(tv, SortHint::Node)],
        action("A", vec![mv(v)], tv),
    );
    let thy = theory("lemanno_2lem", vec![
        rule("R", vec![fact("In", vec![mv("x")])], vec![fact("A", vec![mv("x")])], vec![fact("Out", vec![mv("x")])]),
        lemma("L1", TraceQuantifier::ExistsTrace, vec![LemmaAttr::Reuse], mk("x", "i")),
        lemma("L2", TraceQuantifier::ExistsTrace, vec![LemmaAttr::Reuse], mk("y", "j")),
    ]);
    expect_counts(&thy, &[(checks::T_LEMMA_ANNOT, 2)]);
}

// ===========================================================================
// FIVE CORPUS FILES — finding multisets (batch footer N = these counts plus
// the caller-added out-of-scope "Message Derivation Checks", which is one per
// theory that has any underivable rule).
// ===========================================================================

// -- features/.../statVerifLeftRight/stateverif_left_right.spthy  (footer 3 =
//    Unbound 2 + MsgDeriv 1) : two lookup rules with an unbound `status`.
#[test]
fn count_corpus_stateverif_left_right() {
    let lookup = |name: &str| rule(
        name,
        vec![fact("In", vec![mv("sk")])],
        vec![fact("A", vec![mv("status")])],
        vec![fact("Out", vec![mv("status")])],
    );
    let thy = theory("stateverif_left_right", vec![
        lookup("lookup_s_as_status_0_1111112111"),
        lookup("lookup_s_as_status_0_111112111"),
    ]);
    expect_counts(&thy, &[(checks::T_UNBOUND, 2)]);
}

// -- regression/trace/issue515.spthy  (footer 14, no MsgDeriv) : Reserved names
//    carries 12 rule-side findings, Special facts carries 2 (test2 lhs + rhs).
#[test]
fn count_corpus_issue515() {
    let thy = theory("issue515", vec![
        rule("test", vec![fact("K", vec![mv("x")])], vec![fact("KU", vec![mv("x")])], vec![fact("KD", vec![mv("x")])]),
        rule("test2", vec![fact("Out", vec![mv("x")])], vec![fact("KD", vec![mv("x")])], vec![fact("In", vec![mv("x")])]),
        rule("test3", vec![fact("K", vec![mv("x")])], vec![fact("In", vec![mv("x")])], vec![fact("K", vec![mv("x")])]),
        rule("test4", vec![fact("Ku", vec![mv("x")])], vec![fact("Ku", vec![mv("x")])], vec![fact("Out", vec![mv("x")])]),
        rule("test5", vec![fact("Kd", vec![mv("x")])], vec![fact("Kd", vec![mv("x")])], vec![fact("Ku", vec![mv("x")])]),
    ]);
    expect_counts(&thy, &[(checks::T_RESERVED, 12), (checks::T_SPECIAL, 2)]);
    // Guard the AST copy: the rendered block is byte-identical to the round5
    // reference (which the round5 test already pins against the oracle).
    assert_eq!(
        render_report(&check_theory(&thy)),
        format!("/*\n{}", include_str!("fixtures/t5_issue515.txt").trim_end_matches('\n'))
    );
}

// -- regression/trace/issue527.spthy  (footer 14 = 13 + MsgDeriv 1).
#[test]
fn count_corpus_issue527() {
    let bt = |a: Term, b: Term, k: Term| fact("B_TEST", vec![a, b, k]);
    let register = TheoryItem::Rule(Rule {
        name: "Register_pk".into(),
        modulo: None, attributes: vec![], let_block: vec![],
        premises: vec![fact("Fr", vec![fresh("ltk")])],
        actions: vec![],
        conclusions: vec![
            pfact("Ltk", vec![pub_("A"), mv("ltk")]),
            pfact("Pk", vec![pub_("A"), app("pk", vec![fresh("ltk")])]),
        ],
        embedded_restrictions: vec![], variants: vec![], left_right: None,
    });
    let one = TheoryItem::Rule(Rule {
        name: "One".into(),
        modulo: None, attributes: vec![],
        let_block: vec![LetBinding { var: mv("m1"), value: pair(vec![pl("1"), pub_("A"), fresh("Na")]) }],
        premises: vec![],
        actions: vec![fact("OneResultingIn", vec![pl("second")]), fact("Fact", vec![])],
        conclusions: vec![fact("OneResultingIn", vec![pl("seconD")]), fact("Out", vec![mv("m1")])],
        embedded_restrictions: vec![], variants: vec![], left_right: None,
    });
    let four = TheoryItem::Rule(Rule {
        name: "Four".into(),
        modulo: None, attributes: vec![],
        let_block: vec![LetBinding { var: mv("m"), value: pl("msg") }],
        premises: vec![],
        actions: vec![bt(pl("firSt"), pl("second"), mv("m1"))],
        conclusions: vec![fact("OneresltingIn", vec![pl("second")]), fact("Out", vec![pl("1")])],
        embedded_restrictions: vec![], variants: vec![], left_right: None,
    });
    let lemma_formula = exists(
        vec![
            var("A", SortHint::Untagged), var("B", SortHint::Untagged),
            var("k", SortHint::Untagged), var("i", SortHint::Node),
        ],
        Formula::And(
            Box::new(Formula::And(
                Box::new(Formula::And(
                    Box::new(Formula::Atom(Atom::Action(bt(mv("A"), mv("B"), mv("k")), node("i")))),
                    Box::new(exists(
                        vec![var("j", SortHint::Node)],
                        Formula::And(
                            Box::new(Formula::Atom(Atom::Action(bt(mv("A"), mv("B"), mv("k")), node("j")))),
                            Box::new(Formula::Atom(Atom::Less(node("j"), node("i")))),
                        ),
                    )),
                )),
                Box::new(Formula::Not(Box::new(exists(
                    vec![var("r", SortHint::Node)],
                    Formula::Atom(Atom::Action(fact("Register_pk", vec![mv("A")]), node("r"))),
                )))),
            )),
            Box::new(Formula::Not(Box::new(exists(
                vec![var("a", SortHint::Node)],
                Formula::Atom(Atom::Action(fact("Register_pk", vec![mv("B")]), node("a"))),
            )))),
        ),
    );
    let thy = theory("issue527", vec![
        register,
        rule("Test_1",
            vec![
                fact("In", vec![app("aenc", vec![pair(vec![pl("1"), pub_("A"), mv("m")]), mv("pkB")])]),
                fact("F", vec![pub_("X")]),
                fact("Vars", vec![mv("rhs")]),
            ],
            vec![fact("Fr", vec![pub_("x")])],
            vec![fact("F", vec![pub_("x")]), fact("Test_1", vec![pub_("A")])]),
        one,
        rule("Two", vec![], vec![fact("B_TEST", vec![pl("first")])], vec![fact("OneresultingIn", vec![pl("second")])]),
        rule("three", vec![], vec![fact("B_TEST", vec![pl("third")])], vec![fact("OneResltingIn", vec![pl("second")])]),
        four,
        rule("test", vec![fact("K", vec![mv("x")])], vec![fact("KU", vec![mv("x")])], vec![fact("KD", vec![mv("x")])]),
        lemma("AB_key_honst", TraceQuantifier::ExistsTrace, vec![], lemma_formula),
    ]);
    expect_counts(&thy, &[
        (checks::T_UNBOUND, 3),
        (checks::T_PUBNAMES, 1),
        (checks::T_SORTS, 2),
        (checks::T_RESERVED, 4),
        (checks::T_FACT_CAP, 1),
        (checks::T_ARITY, 1),
        (checks::T_LHSRHS, 1),
    ]);
    assert_eq!(
        render_report(&check_theory(&thy)),
        format!("/*\n{}", include_str!("fixtures/t5_issue527.txt").trim_end_matches('\n'))
    );
}

// -- sapic/deprecated/accountability-old/CertificateTransparency.spthy
//    (footer 5 = Unbound 4 + MsgDeriv 1).
#[test]
fn count_corpus_certificate_transparency() {
    let lookup = |name: &str, unbound: Term| rule(
        name,
        vec![fact("In", vec![mv("sk")])],
        vec![fact("A", vec![unbound.clone()])],
        vec![fact("Out", vec![unbound])],
    );
    let thy = theory("CertificateTransparency", vec![
        lookup("lookup_proofOfID_S_pk_as_ignored_0_111211", mv("ignored")),
        lookup("lookup_log_LA_S_pk_c1_as_x1_0_11211", mv("x1")),
        lookup("lookup_log_LA_S_pk_s1_as_x2_0_112111", mv("x2")),
        lookup("lookup_log_LA_S_pk_s1_as_x2_0_112112", mv("x2")),
    ]);
    expect_counts(&thy, &[(checks::T_UNBOUND, 4)]);
}

// -- sapic/deprecated/accountability-old/OCSPS.spthy
//    (footer 6 = Unbound 4 + lhs-not-rhs 1 + MsgDeriv 1).
#[test]
fn count_corpus_ocsps() {
    let lookup = |name: &str, unbound: Term| rule(
        name,
        vec![fact("In", vec![mv("sk")])],
        vec![fact("A", vec![unbound.clone()])],
        vec![fact("Out", vec![unbound])],
    );
    let thy = theory("OCSPS", vec![
        lookup("lookup_time_as_t_0_111111112111", fresh("t")),
        lookup("lookup_time_as_t_0_11121111", fresh("t")),
        lookup("lookup_ocspstatus_signpksk_sk_CA__as_status_0_111211111", mv("status")),
        lookup("lookup_time_as_L_tc_0_11211", fresh("L_tc")),
        // out_c_sk_OCSP__2_21 has an LHS-only fact `Ack' of arity 2.
        rule("out_c_sk_OCSP__2_21", vec![fact("Ack", vec![mv("a"), mv("b")])], vec![], vec![]),
    ]);
    expect_counts(&thy, &[(checks::T_UNBOUND, 4), (checks::T_LHSRHS, 1)]);
}
