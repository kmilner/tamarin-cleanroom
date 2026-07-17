//! Probe fixtures: every minimal probe theory from the characterisation
//! campaign, encoded directly as `Rule` AST, checked against the label observed
//! from the oracle web UI. See workspace/BEHAVIOR.md for the evidence grid.

use injfacts_clean::ast::{Fact, Rule, SortHint, Term, VarSpec};
use injfacts_clean::{injective_fact_instances, FactTag};
use std::collections::BTreeSet;

// ---- tiny builders -------------------------------------------------------

fn fresh(n: &str) -> Term {
    Term::Var(VarSpec { name: n.into(), idx: 0, sort: SortHint::Fresh, typ: None })
}
fn msg(n: &str) -> Term {
    Term::Var(VarSpec { name: n.into(), idx: 0, sort: SortHint::Msg, typ: None })
}
fn pubv(n: &str) -> Term {
    Term::Var(VarSpec { name: n.into(), idx: 0, sort: SortHint::Pub, typ: None })
}
fn con(s: &str) -> Term {
    Term::PubLit(s.into())
}
fn app(f: &str, args: Vec<Term>) -> Term {
    Term::App(f.into(), args)
}
fn lin(name: &str, args: Vec<Term>) -> Fact {
    Fact { persistent: false, name: name.into(), args, annotations: vec![] }
}
fn per(name: &str, args: Vec<Term>) -> Fact {
    Fact { persistent: true, name: name.into(), args, annotations: vec![] }
}
fn fr(v: Term) -> Fact {
    lin("Fr", vec![v])
}
fn rule(prems: Vec<Fact>, concs: Vec<Fact>) -> Rule {
    Rule {
        name: "r".into(), modulo: None, attributes: vec![], let_block: vec![],
        premises: prems, actions: vec![], conclusions: concs,
        embedded_restrictions: vec![], variants: vec![], left_right: None,
    }
}
fn tags(names: &[(&str, usize)]) -> BTreeSet<FactTag> {
    names.iter().map(|(n, a)| (n.to_string(), *a)).collect()
}
fn none() -> BTreeSet<FactTag> {
    BTreeSet::new()
}

// ---- probes --------------------------------------------------------------

#[test]
fn p01_fresh_loop() {
    // Fr->AA(~id); AA(~id)->AA(~id); AA(~id)->  => AA injective
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("AA", vec![fresh("id")])]),
        rule(vec![lin("AA", vec![fresh("id")])], vec![lin("AA", vec![fresh("id")])]),
        rule(vec![lin("AA", vec![fresh("id")])], vec![]),
    ];
    assert_eq!(injective_fact_instances(&rs), tags(&[("AA", 1)]));
}

#[test]
fn p02_consume_only() {
    // Fr->BB(~id); BB(~id)->  (no loop) => None
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("BB", vec![fresh("id")])]),
        rule(vec![lin("BB", vec![fresh("id")])], vec![]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn p03_persistent() {
    // Fr->!CC(~id); !CC(~id)->Out  => None (persistent)
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![per("CC", vec![fresh("id")])]),
        rule(vec![per("CC", vec![fresh("id")])], vec![lin("Out", vec![fresh("id")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn p04_from_in() {
    // In(x)->DD(x); DD(x)->DD(x)  => None (first arg not fresh)
    let rs = vec![
        rule(vec![lin("In", vec![msg("x")])], vec![lin("DD", vec![msg("x")])]),
        rule(vec![lin("DD", vec![msg("x")])], vec![lin("DD", vec![msg("x")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn p05_fresh_second() {
    // Fr->EE('c',~id); EE(a,b)->EE(a,b)  => None (fresh in 2nd position)
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("EE", vec![con("c"), fresh("id")])]),
        rule(vec![lin("EE", vec![msg("a"), msg("b")])], vec![lin("EE", vec![msg("a"), msg("b")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn p06_two_produced_noloop() {
    // [Fr~a,Fr~b]->[FF~a,FF~b]; FF(~x)->  => None (no loop)
    let rs = vec![
        rule(vec![fr(fresh("a")), fr(fresh("b"))], vec![lin("FF", vec![fresh("a")]), lin("FF", vec![fresh("b")])]),
        rule(vec![lin("FF", vec![fresh("x")])], vec![]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn p07_reproduce_newfresh() {
    // Fr->GG(~id); [GG(~id),Fr~id2]->GG(~id2)  => GG injective
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("GG", vec![fresh("id")])]),
        rule(vec![lin("GG", vec![fresh("id")]), fr(fresh("id2"))], vec![lin("GG", vec![fresh("id2")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), tags(&[("GG", 1)]));
}

#[test]
fn p08_const() {
    // ->HH('a'); HH(x)->  => None
    let rs = vec![
        rule(vec![], vec![lin("HH", vec![con("a")])]),
        rule(vec![lin("HH", vec![msg("x")])], vec![]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn p09_mixed_first() {
    // Fr->II(~id) AND In(x)->II(x); II(x)->  => None
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("II", vec![fresh("id")])]),
        rule(vec![lin("In", vec![msg("x")])], vec![lin("II", vec![msg("x")])]),
        rule(vec![lin("II", vec![msg("x")])], vec![]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn p11_fresh_first_extra() {
    // [Fr~id,In x]->KK(~id,x); KK(~id,x)->KK(~id,x); KK(~id,x)->  => KK/2 injective
    let rs = vec![
        rule(vec![fr(fresh("id")), lin("In", vec![msg("x")])], vec![lin("KK", vec![fresh("id"), msg("x")])]),
        rule(vec![lin("KK", vec![fresh("id"), msg("x")])], vec![lin("KK", vec![fresh("id"), msg("x")])]),
        rule(vec![lin("KK", vec![fresh("id"), msg("x")])], vec![]),
    ];
    assert_eq!(injective_fact_instances(&rs), tags(&[("KK", 2)]));
}

#[test]
fn b01_loop_double() {
    // Init Fr->LL(~id); Dup [LL~id,Fr~id2]->[LL~id,LL~id2]; Fin LL~id->  => LL injective
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("LL", vec![fresh("id")])]),
        rule(vec![lin("LL", vec![fresh("id")]), fr(fresh("id2"))],
             vec![lin("LL", vec![fresh("id")]), lin("LL", vec![fresh("id2")])]),
        rule(vec![lin("LL", vec![fresh("id")])], vec![]),
    ];
    assert_eq!(injective_fact_instances(&rs), tags(&[("LL", 1)]));
}

#[test]
fn b02_two_producers_noloop() {
    // A Fr->MM(~id); B [In x,Fr~id]->MM(~id); C MM(~id)->  => None (no loop)
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("MM", vec![fresh("id")])]),
        rule(vec![lin("In", vec![msg("x")]), fr(fresh("id"))], vec![lin("MM", vec![fresh("id")])]),
        rule(vec![lin("MM", vec![fresh("id")])], vec![]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn b03_loop_only() {
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("OO", vec![fresh("id")])]),
        rule(vec![lin("OO", vec![fresh("id")])], vec![lin("OO", vec![fresh("id")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), tags(&[("OO", 1)]));
}

#[test]
fn b04_produce_only() {
    let rs = vec![rule(vec![fr(fresh("id"))], vec![lin("QQ", vec![fresh("id")])])];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn b05_pubvar_first() {
    // Fr,In$A -> NN($A,~id); NN(a,b)->NN(a,b) => None
    let rs = vec![
        rule(vec![fr(fresh("id")), lin("In", vec![pubv("A")])], vec![lin("NN", vec![pubv("A"), fresh("id")])]),
        rule(vec![lin("NN", vec![msg("a"), msg("b")])], vec![lin("NN", vec![msg("a"), msg("b")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn b06_funterm_first() {
    // Fr->PP(h(~id)); PP(x)->PP(x) => None (first arg not a variable)
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("PP", vec![app("h", vec![fresh("id")])])]),
        rule(vec![lin("PP", vec![msg("x")])], vec![lin("PP", vec![msg("x")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn b07_consume_double() {
    // Init Fr->RR(~id); Merge [RR~a,RR~b]->RR~a; Fin RR~id-> => RR injective
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("RR", vec![fresh("id")])]),
        rule(vec![lin("RR", vec![fresh("a")]), lin("RR", vec![fresh("b")])], vec![lin("RR", vec![fresh("a")])]),
        rule(vec![lin("RR", vec![fresh("id")])], vec![]),
    ];
    assert_eq!(injective_fact_instances(&rs), tags(&[("RR", 1)]));
}

#[test]
fn b08_in_then_loop() {
    // In(x)->TT(x); TT(x)->TT(x); TT(x)->  => None
    let rs = vec![
        rule(vec![lin("In", vec![msg("x")])], vec![lin("TT", vec![msg("x")])]),
        rule(vec![lin("TT", vec![msg("x")])], vec![lin("TT", vec![msg("x")])]),
        rule(vec![lin("TT", vec![msg("x")])], vec![]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn c01_consumer_msg_first() {
    // good loop + extra consumer UU(y) y msg => UU injective
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("UU", vec![fresh("id")])]),
        rule(vec![lin("UU", vec![fresh("id")])], vec![lin("UU", vec![fresh("id")])]),
        rule(vec![lin("UU", vec![msg("y")])], vec![]),
    ];
    assert_eq!(injective_fact_instances(&rs), tags(&[("UU", 1)]));
}

#[test]
fn c02_two_facts() {
    let rs = vec![
        rule(vec![fr(fresh("a"))], vec![lin("ZA", vec![fresh("a")])]),
        rule(vec![lin("ZA", vec![fresh("a")])], vec![lin("ZA", vec![fresh("a")])]),
        rule(vec![fr(fresh("b"))], vec![lin("MB", vec![fresh("b")])]),
        rule(vec![lin("MB", vec![fresh("b")])], vec![lin("MB", vec![fresh("b")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), tags(&[("ZA", 1), ("MB", 1)]));
}

#[test]
fn c03_producer_msg_with_loop() {
    // good loop + Extra In(z)->VV(z) => None
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("VV", vec![fresh("id")])]),
        rule(vec![lin("VV", vec![fresh("id")])], vec![lin("VV", vec![fresh("id")])]),
        rule(vec![lin("In", vec![msg("z")])], vec![lin("VV", vec![msg("z")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn c06_loop_msg_carried() {
    // Fr->WW(~id); WW(z)->WW(z) (z msg carried) => WW injective
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("WW", vec![fresh("id")])]),
        rule(vec![lin("WW", vec![msg("z")])], vec![lin("WW", vec![msg("z")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), tags(&[("WW", 1)]));
}

#[test]
fn d01_reproduce_in() {
    // Fr->F1(~id); [F1(z),In(w)]->F1(w) => None
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("F1", vec![fresh("id")])]),
        rule(vec![lin("F1", vec![msg("z")]), lin("In", vec![msg("w")])], vec![lin("F1", vec![msg("w")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn d02_reproduce_nonf_msg() {
    // Gen Fr->HH(~k); Init Fr->F2(~id); Loop [F2(z),HH(w)]->F2(w) => None
    let rs = vec![
        rule(vec![fr(fresh("k"))], vec![lin("HH", vec![fresh("k")])]),
        rule(vec![fr(fresh("id"))], vec![lin("F2", vec![fresh("id")])]),
        rule(vec![lin("F2", vec![msg("z")]), lin("HH", vec![msg("w")])], vec![lin("F2", vec![msg("w")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn d04_reproduce_freshsort_nonfr() {
    // Loop [F4(~z), !HH(~w)] -> F4(~w): ~w fresh-sorted but from persistent, not Fr => None
    let rs = vec![
        rule(vec![fr(fresh("k"))], vec![per("HH", vec![fresh("k")])]),
        rule(vec![fr(fresh("id"))], vec![lin("F4", vec![fresh("id")])]),
        rule(vec![lin("F4", vec![fresh("z")]), per("HH", vec![fresh("w")])], vec![lin("F4", vec![fresh("w")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn d05_reproduce_carry_fresh() {
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("F5", vec![fresh("id")])]),
        rule(vec![lin("F5", vec![fresh("z")])], vec![lin("F5", vec![fresh("z")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), tags(&[("F5", 1)]));
}

#[test]
fn e01_multiarg_carry() {
    // Fr->HH(~id,'a'); Loop HH(~z,x)->HH(~z,'b') => HH/2 injective
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("HHm", vec![fresh("id"), con("a")])]),
        rule(vec![lin("HHm", vec![fresh("z"), msg("x")])], vec![lin("HHm", vec![fresh("z"), con("b")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), tags(&[("HHm", 2)]));
}

#[test]
fn e02_dup_carried() {
    // loop + Dup GG(~z)->GG(~z),GG(~z) => None (count increase)
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("GG", vec![fresh("id")])]),
        rule(vec![lin("GG", vec![fresh("id")])], vec![lin("GG", vec![fresh("id")])]),
        rule(vec![lin("GG", vec![fresh("z")])], vec![lin("GG", vec![fresh("z")]), lin("GG", vec![fresh("z")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn e03_two_same_fresh() {
    // loop + Two Fr(~a)->JJ(~a),JJ(~a) => None
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("JJ", vec![fresh("id")])]),
        rule(vec![lin("JJ", vec![fresh("id")])], vec![lin("JJ", vec![fresh("id")])]),
        rule(vec![fr(fresh("a"))], vec![lin("JJ", vec![fresh("a")]), lin("JJ", vec![fresh("a")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn g01_grow_matched() {
    // loop + Grow [GA~a,GA~b,Fr~c]->[GA~a,GA~b,GA~c] => GA injective
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("GA", vec![fresh("id")])]),
        rule(vec![lin("GA", vec![fresh("id")])], vec![lin("GA", vec![fresh("id")])]),
        rule(vec![lin("GA", vec![fresh("a")]), lin("GA", vec![fresh("b")]), fr(fresh("c"))],
             vec![lin("GA", vec![fresh("a")]), lin("GA", vec![fresh("b")]), lin("GA", vec![fresh("c")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), tags(&[("GA", 1)]));
}

#[test]
fn g02_bad_match() {
    // loop + Bad [GB~a,GB~b]->[GB~a,GB~a] => None
    let rs = vec![
        rule(vec![fr(fresh("id"))], vec![lin("GB", vec![fresh("id")])]),
        rule(vec![lin("GB", vec![fresh("id")])], vec![lin("GB", vec![fresh("id")])]),
        rule(vec![lin("GB", vec![fresh("a")]), lin("GB", vec![fresh("b")])],
             vec![lin("GB", vec![fresh("a")]), lin("GB", vec![fresh("a")])]),
    ];
    assert_eq!(injective_fact_instances(&rs), none());
}

#[test]
fn arity0_never_injective() {
    // A 0-ary fact in a loop must never be injective (no first argument).
    let rs = vec![rule(vec![lin("Sig", vec![])], vec![lin("Sig", vec![])])];
    assert_eq!(injective_fact_instances(&rs), none());
}
