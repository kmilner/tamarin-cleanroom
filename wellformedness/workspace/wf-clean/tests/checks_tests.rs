//! Byte-parity tests. Each fixture in tests/fixtures/ is an OBSERVED oracle
//! WARNING block (from `wf_oracle.sh` on the correspondingly-named probe). We
//! build the matching AST by hand and assert that render_report(check_theory)
//! reproduces the oracle output byte-for-byte.

use wf_clean::ast::*;
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

fn fact(name: &str, args: Vec<Term>) -> Fact {
    Fact { persistent: false, name: name.into(), args, annotations: vec![] }
}
fn pfact(name: &str, args: Vec<Term>) -> Fact {
    Fact { persistent: true, name: name.into(), args, annotations: vec![] }
}

fn rule(name: &str, prem: Vec<Fact>, act: Vec<Fact>, concl: Vec<Fact>) -> TheoryItem {
    TheoryItem::Rule(Rule {
        name: name.into(),
        modulo: None,
        attributes: vec![],
        let_block: vec![],
        premises: prem,
        actions: act,
        conclusions: concl,
        embedded_restrictions: vec![],
        variants: vec![],
        left_right: None,
    })
}

fn lemma(name: &str, formula: Formula) -> TheoryItem {
    TheoryItem::Lemma(Lemma {
        name: name.into(),
        modulo: None,
        attributes: vec![],
        trace_quantifier: TraceQuantifier::AllTraces,
        formula,
        proof: None,
        plaintext: String::new(),
    })
}

fn theory(name: &str, items: Vec<TheoryItem>) -> Theory {
    Theory { is_diff: false, name: name.into(), configuration: None, items }
}

fn diff_theory(name: &str, items: Vec<TheoryItem>) -> Theory {
    Theory { is_diff: true, name: name.into(), configuration: None, items }
}

fn flit(name: &str) -> Term { Term::FreshLit(name.into()) }

/// A bare Rule value (not wrapped in TheoryItem), for building diff left/right
/// projections.
fn plain_rule(name: &str, prem: Vec<Fact>, act: Vec<Fact>, concl: Vec<Fact>) -> Rule {
    Rule {
        name: name.into(),
        modulo: None,
        attributes: vec![],
        let_block: vec![],
        premises: prem,
        actions: act,
        conclusions: concl,
        embedded_restrictions: vec![],
        variants: vec![],
        left_right: None,
    }
}

fn diff_term(a: Term, b: Term) -> Term { Term::Diff(Box::new(a), Box::new(b)) }

/// Assert render_report(check_theory(thy)) equals the oracle fixture.
fn expect(thy: &Theory, fixture: &str) {
    let got = render_report(&check_theory(thy));
    let want = format!("/*\n{}", fixture.trim_end_matches('\n'));
    assert_eq!(got, want, "\n--- got ---\n{}\n--- want ---\n{}", got, want);
}

// ---- Core API -------------------------------------------------------------

#[test]
fn success_when_empty() {
    let thy = theory("Empty", vec![]);
    assert_eq!(check_theory(&thy).len(), 0);
    assert_eq!(render_report(&check_theory(&thy)), SUCCESS_LINE);
}

#[test]
fn underline_matches_length_including_spaces() {
    assert_eq!(underline_topic("abc"), "abc\n===");
    // leading space counts
    assert_eq!(underline_topic(" Formula guardedness").lines().nth(1).unwrap().len(), 20);
    // trailing space counts
    let t = "Facts occur in the left-hand-side but not in any right-hand-side ";
    assert_eq!(underline_topic(t).lines().nth(1).unwrap().len(), t.len());
}

#[test]
fn topics_are_distinct_sorted() {
    let r = vec![
        WfError::new("B", "x"),
        WfError::new("A", "y"),
        WfError::new("B", "z"),
    ];
    let t = topics(&r);
    assert_eq!(t.into_iter().collect::<Vec<_>>(), vec!["A".to_string(), "B".to_string()]);
}

#[test]
fn insert_wf_before_places_at_first_anchor() {
    let mut r = vec![WfError::new("first", "a"), WfError::new("anchor", "b")];
    insert_wf_before(&mut r, vec![WfError::new("PN", "p")], &["anchor"]);
    let ts: Vec<_> = r.iter().map(|e| e.topic.clone()).collect();
    assert_eq!(ts, vec!["first", "PN", "anchor"]);
}

#[test]
fn insert_wf_before_appends_without_anchor() {
    let mut r = vec![WfError::new("first", "a")];
    insert_wf_before(&mut r, vec![WfError::new("PN", "p")], &["nope"]);
    assert_eq!(r.last().unwrap().topic, "PN");
}

// ---- Per-check byte-parity tests -----------------------------------------

#[test]
fn public_names_same_rule() {
    // p32: rule R1: [Fr(~x)] --[A('Alice'), B('alice')]-> [Out(~x)]
    let thy = theory("P32", vec![rule(
        "R1",
        vec![fact("Fr", vec![fresh("x")])],
        vec![fact("A", vec![pl("Alice")]), fact("B", vec![pl("alice")])],
        vec![fact("Out", vec![fresh("x")])],
    )]);
    expect(&thy, include_str!("fixtures/p32_simpub.txt"));
}

#[test]
fn public_names_three_names_one_rule() {
    let thy = theory("FP3", vec![rule(
        "R1",
        vec![fact("Fr", vec![fresh("x")])],
        vec![fact("A", vec![pl("Node"), pl("node"), pl("NODE")])],
        vec![fact("Out", vec![fresh("x")])],
    )]);
    expect(&thy, include_str!("fixtures/f_pub3.txt"));
}

#[test]
fn public_names_cross_rule() {
    let thy = theory("P38", vec![
        rule("R1",
            vec![fact("Fr", vec![fresh("x")])],
            vec![fact("A", vec![pl("Server")])],
            vec![fact("Out", vec![pl("Server")])]),
        rule("R2",
            vec![fact("Fr", vec![fresh("y")])],
            vec![fact("B", vec![pl("server")])],
            vec![fact("Out", vec![pl("server")])]),
    ]);
    expect(&thy, include_str!("fixtures/p38_pub2.txt"));
}

#[test]
fn mismatching_sorts() {
    // p20: rule R1: [Fr(~x), In($x)] --> [Out(~x)]
    let thy = theory("P20", vec![rule(
        "R1",
        vec![fact("Fr", vec![fresh("x")]), fact("In", vec![pub_("x")])],
        vec![],
        vec![fact("Out", vec![fresh("x")])],
    )]);
    expect(&thy, include_str!("fixtures/p20_inconsistent_sort.txt"));
}

#[test]
fn reserved_name_nullary() {
    // f_nullary: rule R1: [Fr(~x)] --[K()]-> [Out(~x)]
    let thy = theory("FN", vec![rule(
        "R1",
        vec![fact("Fr", vec![fresh("x")])],
        vec![fact("K", vec![])],
        vec![fact("Out", vec![fresh("x")])],
    )]);
    expect(&thy, include_str!("fixtures/f_nullary.txt"));
}

#[test]
fn reserved_names_io_in_actions() {
    // z11: In/Out/Fr/K in the middle are all "reserved names on the middle".
    let thy = theory("Z11", vec![rule(
        "R1",
        vec![fact("Fr", vec![fresh("x")])],
        vec![fact("In", vec![fresh("x")]), fact("Out", vec![fresh("x")]), fact("K", vec![fresh("x")])],
        vec![fact("Out", vec![fresh("x")])],
    )]);
    expect(&thy, include_str!("fixtures/z11_io_action.txt"));
}

#[test]
fn fr_on_rhs_is_special_not_reserved() {
    // z12: Fr on the RHS is a Special-fact violation (not a reserved name).
    let thy = theory("Z12", vec![rule(
        "R1",
        vec![fact("In", vec![fresh("x")])],
        vec![],
        vec![fact("Fr", vec![fresh("x")])],
    )]);
    expect(&thy, include_str!("fixtures/z12_fr_rhs.txt"));
}

#[test]
fn special_facts_both_sides() {
    // p15: rule R1: [Out(x)] --> [In(x)]
    let thy = theory("SFP", vec![rule(
        "R1",
        vec![fact("Out", vec![mv("x")])],
        vec![],
        vec![fact("In", vec![mv("x")])],
    )]);
    expect(&thy, include_str!("fixtures/p15_inout.txt"));
}

#[test]
fn fr_fact_pub() {
    // p14: rule R1: [Fr($x)] --> [Out($x)]
    let thy = theory("P14", vec![rule(
        "R1",
        vec![fact("Fr", vec![pub_("x")])],
        vec![],
        vec![fact("Out", vec![pub_("x")])],
    )]);
    expect(&thy, include_str!("fixtures/p14_fr_pub.txt"));
}

#[test]
fn fr_fact_multiple() {
    // f_fr: R1 [Fr($a),Fr($b)]-->[Out(<$a,$b>)]; R2 [Fr($c)]-->[Out($c)]
    let thy = theory("FFR", vec![
        rule("R1",
            vec![fact("Fr", vec![pub_("a")]), fact("Fr", vec![pub_("b")])],
            vec![],
            vec![fact("Out", vec![Term::Pair(vec![pub_("a"), pub_("b")])])]),
        rule("R2",
            vec![fact("Fr", vec![pub_("c")])],
            vec![],
            vec![fact("Out", vec![pub_("c")])]),
    ]);
    expect(&thy, include_str!("fixtures/f_fr.txt"));
}

#[test]
fn fact_arity_and_lhs_not_rhs() {
    // f_arity3 (see fixture): Foo used at arities 1,2,3; Bar at 1,2.
    fn app(f: &str, args: Vec<Term>) -> Term { Term::App(f.into(), args) }
    let _ = app; // (facts only here)
    let thy = theory("FA3", vec![
        rule("R1", vec![fact("Foo", vec![mv("x")])], vec![], vec![fact("Out", vec![mv("x")])]),
        rule("R2", vec![fact("Foo", vec![mv("x"), mv("y")])], vec![], vec![fact("Out", vec![mv("x")])]),
        rule("R3", vec![fact("In", vec![mv("x")])], vec![], vec![fact("Foo", vec![mv("x"), mv("x"), mv("x")])]),
        rule("R4", vec![fact("Bar", vec![mv("x")])], vec![], vec![fact("Out", vec![mv("x")])]),
        rule("R5", vec![fact("In", vec![mv("x")])], vec![], vec![fact("Bar", vec![mv("x"), mv("x")])]),
    ]);
    expect(&thy, include_str!("fixtures/f_arity3.txt"));
}

#[test]
fn fact_multiplicity() {
    // Bar linear in R1 RHS, persistent in R2 LHS (mirrors probe p03/ks1).
    let thy = theory("P03", vec![
        rule("R1", vec![fact("In", vec![mv("x")])], vec![], vec![fact("Bar", vec![mv("x")])]),
        rule("R2", vec![pfact("Bar", vec![mv("x")])], vec![], vec![fact("Out", vec![mv("x")])]),
    ]);
    let report = check_theory(&thy);
    // multiplicity topic present and correctly formatted
    let mult = report.iter().find(|e| e.topic == "Fact multiplicity issues").unwrap();
    assert_eq!(
        mult.message,
        "Same fact is used with different multiplicities, i.e., !Fact() (Persistent fact) exists along with Fact() (Linear) in your rules. \nCheck the multiplicity (persistence) of your facts.\n  \n\n  Fact `bar':\n\n    1. Rule `R1', multiplicity (persistence) Linear\n         Bar( x )\n    \n    2. Rule `R2', multiplicity (persistence) Persistent\n         !Bar( x )\n  "
    );
}

#[test]
fn formula_terms_free_var() {
    // p05: lemma L1: "All x #i. A(x) @ #i ==> x = y"  (y free)
    let f = Formula::Forall(
        vec![var("x", SortHint::Msg), var("i", SortHint::Node)],
        Box::new(Formula::Implies(
            Box::new(Formula::Atom(Atom::Action(fact("A", vec![mv("x")]), node("i")))),
            Box::new(Formula::Atom(Atom::Eq(mv("x"), mv("y")))),
        )),
    );
    let thy = theory("P05", vec![
        rule("R1", vec![fact("Fr", vec![fresh("x")])], vec![fact("A", vec![fresh("x")])], vec![fact("Out", vec![fresh("x")])]),
        lemma("L1", f),
    ]);
    expect(&thy, include_str!("fixtures/p05_lemma_free.txt"));
}

#[test]
fn formula_guardedness_unguarded_temporal() {
    // p21: lemma L1: "All x #i #j. A(x) @ #i ==> #i = #j"  (#j unguarded)
    let f = Formula::Forall(
        vec![var("x", SortHint::Msg), var("i", SortHint::Node), var("j", SortHint::Node)],
        Box::new(Formula::Implies(
            Box::new(Formula::Atom(Atom::Action(fact("A", vec![mv("x")]), node("i")))),
            Box::new(Formula::Atom(Atom::Eq(node("i"), node("j")))),
        )),
    );
    let thy = theory("P21", vec![
        rule("R1", vec![fact("Fr", vec![fresh("x")])], vec![fact("A", vec![fresh("x")])], vec![fact("Out", vec![fresh("x")])]),
        lemma("L1", f),
    ]);
    expect(&thy, include_str!("fixtures/p21_temporal_term.txt"));
}

#[test]
fn nat_sorts_and_reserved() {
    // t_nat: rule R1: [In(y), In(z)] --[K(y %+ z), K(%1)]-> [Out(y)]
    let natplus = Term::BinOp(BinOp::NatPlus, Box::new(mv("y")), Box::new(mv("z")));
    let thy = theory("TN", vec![rule(
        "R1",
        vec![fact("In", vec![mv("y")]), fact("In", vec![mv("z")])],
        vec![fact("K", vec![natplus]), fact("K", vec![Term::NatOne])],
        vec![fact("Out", vec![mv("y")])],
    )]);
    expect(&thy, include_str!("fixtures/t_nat.txt"));
}

#[test]
fn subterm_convergence_and_formula_terms() {
    // f_subterm: functions f/1,g/1,h/2; equations h(f(x),y)=g(y);
    //            rule R1 [Fr(~x)]--[Act(~x)]->[Out(f(~x))];
    //            lemma L1 "All x #i. Act(x) @ #i ==> z = x"  (z free)
    fn app(f: &str, a: Vec<Term>) -> Term { Term::App(f.into(), a) }
    let eq = Equation {
        lhs: app("h", vec![app("f", vec![mv("x")]), mv("y")]),
        rhs: app("g", vec![mv("y")]),
    };
    let lem = Formula::Forall(
        vec![var("x", SortHint::Msg), var("i", SortHint::Node)],
        Box::new(Formula::Implies(
            Box::new(Formula::Atom(Atom::Action(fact("Act", vec![mv("x")]), node("i")))),
            Box::new(Formula::Atom(Atom::Eq(mv("z"), mv("x")))),
        )),
    );
    let thy = theory("FST", vec![
        TheoryItem::Equations { convergent: false, eqs: vec![eq] },
        rule("R1", vec![fact("Fr", vec![fresh("x")])],
            vec![fact("Act", vec![fresh("x")])],
            vec![fact("Out", vec![app("f", vec![fresh("x")])])]),
        lemma("L1", lem),
    ]);
    expect(&thy, include_str!("fixtures/f_subterm.txt"));
}

#[test]
fn subterm_convergent_equation_is_ok() {
    // dec(enc(m,k),k) = m  -> m is a subterm of the LHS -> no warning
    fn app(f: &str, a: Vec<Term>) -> Term { Term::App(f.into(), a) }
    let eq = Equation {
        lhs: app("dec", vec![app("enc", vec![mv("m"), mv("k")]), mv("k")]),
        rhs: mv("m"),
    };
    let thy = theory("OK", vec![TheoryItem::Equations { convergent: false, eqs: vec![eq] }]);
    assert!(check_theory(&thy).is_empty());
}

// ---- Ordering / secondary entry points -----------------------------------

#[test]
fn canonical_order_public_names_second() {
    // Trigger unbound (#1) + public-names (#2) + reserved (#4). Public names
    // must land between unbound and reserved.
    let thy = theory("ORD", vec![rule(
        "R1",
        vec![],
        vec![fact("K", vec![mv("q")]), fact("A", vec![pl("Alice"), pl("alice")])],
        vec![fact("Out", vec![mv("w")])],
    )]);
    let ts: Vec<String> = check_theory(&thy).iter().map(|e| e.topic.clone()).collect();
    let pos = |t: &str| ts.iter().position(|x| x == t);
    assert!(pos("Unbound variables").unwrap() < pos("Public constants with mismatching capitalization").unwrap());
    assert!(pos("Public constants with mismatching capitalization").unwrap() < pos("Reserved names").unwrap());
}

#[test]
fn public_names_from_pairs_direct() {
    let r = public_names_report_from_pairs(vec![
        ("Alice".into(), "R1".into()),
        ("alice".into(), "R1".into()),
    ]);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].topic, "Public constants with mismatching capitalization");
    assert!(r[0].message.ends_with("  1. rule \"R1\":  name 'Alice', 'alice'"));
}

#[test]
fn public_names_from_pairs_no_conflict() {
    // identical spelling -> no conflict
    let r = public_names_report_from_pairs(vec![
        ("Alice".into(), "R1".into()),
        ("Alice".into(), "R2".into()),
    ]);
    assert!(r.is_empty());
}

#[test]
fn after_public_names_topics_are_the_anchors() {
    let anchors = after_public_names_topics();
    assert_eq!(anchors[0], "Variable with mismatching sorts or capitalization");
    assert!(anchors.contains(&"Subterm Convergence Warning"));
    assert!(!anchors.contains(&"Unbound variables"));
    assert!(!anchors.contains(&"Public constants with mismatching capitalization"));
}

#[test]
fn check_if_lemmas_in_theory_logic() {
    let thy = theory("L", vec![
        lemma("secrecy", Formula::True),
        lemma("agreement", Formula::True),
    ]);
    let present = check_if_lemmas_in_theory(&["secrecy".into(), "agreement".into()], &thy);
    assert!(present.is_empty());
    let missing = check_if_lemmas_in_theory(&["secrecy".into(), "nope".into()], &thy);
    assert_eq!(missing.len(), 1);
}

// ---- Round 2: new topics --------------------------------------------------

#[test]
fn lemma_annotations_exists_trace_reuse() {
    // exists_trace_reuse: rule Setup + `[reuse]` exists-trace lemma. The lemma
    // formula deliberately tags the quantified `x` (Untagged) differently from
    // its occurrence in Setup(x) (Msg): name-based binding must treat them as
    // one variable so NO "Formula terms" / "Formula guardedness" false positive
    // is emitted -- only "Lemma annotations".
    let f = Formula::Exists(
        vec![var("x", SortHint::Untagged), var("i", SortHint::Node)],
        Box::new(Formula::Atom(Atom::Action(fact("Setup", vec![mv("x")]), node("i")))),
    );
    let lem = TheoryItem::Lemma(Lemma {
        name: "exist_reuse".into(),
        modulo: None,
        attributes: vec![LemmaAttr::Reuse],
        trace_quantifier: TraceQuantifier::ExistsTrace,
        formula: f,
        proof: None,
        plaintext: String::new(),
    });
    let thy = theory("ExistsTraceReuse", vec![
        rule("Setup",
            vec![fact("Fr", vec![fresh("x")])],
            vec![fact("Setup", vec![fresh("x")])],
            vec![fact("Out", vec![fresh("x")])]),
        lem,
    ]);
    // Regression guard for the reported false positive: only one topic.
    assert_eq!(
        topics(&check_theory(&thy)).into_iter().collect::<Vec<_>>(),
        vec!["Lemma annotations".to_string()]
    );
    expect(&thy, include_str!("fixtures/r2_lemma_annotations.txt"));
}

#[test]
fn fresh_public_constant_literal() {
    // fresh_public_constant: rule Bad: [] --> [Out(~'foo')]
    let thy = theory("FreshPubConst", vec![rule(
        "Bad",
        vec![],
        vec![],
        vec![fact("Out", vec![flit("foo")])],
    )]);
    expect(&thy, include_str!("fixtures/r2_fresh_public_constant.txt"));
}

#[test]
fn multiplication_restriction_in_conclusion() {
    // multiplication_in_rule_lhs: [Fr(~x), In(a*b)] --> [Out(<~x, a*b>)]
    let ab = Term::BinOp(BinOp::Mult, Box::new(mv("a")), Box::new(mv("b")));
    let thy = theory("MultLhs", vec![rule(
        "Bad",
        vec![fact("Fr", vec![fresh("x")]), fact("In", vec![ab.clone()])],
        vec![],
        vec![fact("Out", vec![Term::Pair(vec![fresh("x"), ab])])],
    )]);
    // check_theory reproduces the multiplication topic byte-for-byte; the
    // oracle additionally emits the Maude-computed Message Derivation block,
    // which is out of scope (see BEHAVIOR.md).
    expect(&thy, include_str!("fixtures/r2_multiplication.txt"));
}

#[test]
fn diff_left_rule_inconsistent() {
    // diff_left_right_mismatch: explicit `left` has an extra premise vs the
    // left projection of the parent diff rule.
    let left = plain_rule("Send",
        vec![fact("Fr", vec![fresh("k")]), fact("In", vec![mv("extra")])],
        vec![],
        vec![fact("Out", vec![fresh("k")])]);
    let right = plain_rule("Send",
        vec![fact("Fr", vec![fresh("k")])],
        vec![],
        vec![fact("Out", vec![fresh("k")])]);
    let mut parent = plain_rule("Send",
        vec![fact("Fr", vec![fresh("k")])],
        vec![],
        vec![fact("Out", vec![diff_term(fresh("k"), fresh("k"))])]);
    parent.left_right = Some((Box::new(left), Box::new(right)));
    let thy = diff_theory("DiffLR", vec![TheoryItem::Rule(parent)]);
    expect(&thy, include_str!("fixtures/r2_left_rule.txt"));
}

#[test]
fn diff_right_rule_inconsistent() {
    // diff_right_rule_mismatch: left projection matches; the explicit `right`
    // has an extra premise -> "Right rule" (and NOT "Left rule").
    let left = plain_rule("Send",
        vec![fact("Fr", vec![fresh("k")])],
        vec![],
        vec![fact("Out", vec![fresh("k")])]);
    let right = plain_rule("Send",
        vec![fact("Fr", vec![fresh("k")]), fact("In", vec![mv("extra")])],
        vec![],
        vec![fact("Out", vec![fresh("k")])]);
    let mut parent = plain_rule("Send",
        vec![fact("Fr", vec![fresh("k")])],
        vec![],
        vec![fact("Out", vec![diff_term(fresh("k"), fresh("k"))])]);
    parent.left_right = Some((Box::new(left), Box::new(right)));
    let thy = diff_theory("DiffRR", vec![TheoryItem::Rule(parent)]);
    expect(&thy, include_str!("fixtures/r2_right_rule.txt"));
}

#[test]
fn diff_left_right_consistent_is_silent() {
    // Explicit left and right both equal their projections -> no warning.
    let left = plain_rule("Send",
        vec![fact("Fr", vec![fresh("k")])], vec![], vec![fact("Out", vec![fresh("k")])]);
    let right = plain_rule("Send",
        vec![fact("Fr", vec![fresh("k")])], vec![], vec![fact("Out", vec![fresh("k")])]);
    let mut parent = plain_rule("Send",
        vec![fact("Fr", vec![fresh("k")])], vec![],
        vec![fact("Out", vec![diff_term(fresh("k"), fresh("k"))])]);
    parent.left_right = Some((Box::new(left), Box::new(right)));
    let thy = diff_theory("DiffOK", vec![TheoryItem::Rule(parent)]);
    assert!(check_theory(&thy).is_empty());
}

#[test]
fn reserved_prefix_diff_only() {
    // diff_reserved_prefix: rule Bad uses a `DiffIntr`-prefixed fact name.
    let thy = diff_theory("DiffReservedPrefix", vec![rule(
        "Bad",
        vec![fact("Fr", vec![fresh("x")])],
        vec![],
        vec![fact("DiffIntrPriv", vec![fresh("x")])],
    )]);
    expect(&thy, include_str!("fixtures/r2_reserved_prefixes.txt"));

    // Reserved prefixes are diff-mode only: the same rule in a non-diff theory
    // is silent (observed).
    let non_diff = theory("DiffReservedPrefix", vec![rule(
        "Bad",
        vec![fact("Fr", vec![fresh("x")])],
        vec![],
        vec![fact("DiffIntrPriv", vec![fresh("x")])],
    )]);
    assert!(check_theory(&non_diff).is_empty());
}

// ---- Round 2: ordering ----------------------------------------------------

#[test]
fn fresh_public_constants_before_public_names() {
    // Unbound (#1), Fresh public constants (#2), Public names (#3), Reserved.
    let thy = theory("Ord", vec![
        rule("U",
            vec![fact("In", vec![mv("x")])],
            vec![fact("K", vec![mv("z")])],
            vec![fact("Out", vec![Term::Pair(vec![mv("y"), flit("foo")])])]),
        rule("Caps",
            vec![fact("Fr", vec![fresh("s")])],
            vec![fact("A", vec![pl("Alice"), pl("alice")])],
            vec![fact("Out", vec![fresh("s")])]),
    ]);
    let ts: Vec<String> = check_theory(&thy).iter().map(|e| e.topic.clone()).collect();
    let pos = |t: &str| ts.iter().position(|x| x == t).unwrap();
    assert!(pos("Unbound variables") < pos("Fresh public constants"));
    assert!(pos("Fresh public constants") < pos("Public constants with mismatching capitalization"));
    assert!(pos("Public constants with mismatching capitalization") < pos("Reserved names"));
}

#[test]
fn lemma_annotations_between_guardedness_and_nat() {
    // Formula terms (#) < Formula guardedness < Lemma annotations < Subterm.
    let ft = lemma_all("ft", Formula::Forall(
        vec![var("a", SortHint::Msg), var("i", SortHint::Node)],
        Box::new(Formula::Implies(
            Box::new(Formula::Atom(Atom::Action(fact("A", vec![mv("a")]), node("i")))),
            Box::new(Formula::Atom(Atom::Eq(mv("a"), mv("b")))),
        )),
    ));
    let guard = lemma_all("guard", Formula::Forall(
        vec![var("x", SortHint::Msg), var("i", SortHint::Node), var("j", SortHint::Node)],
        Box::new(Formula::Implies(
            Box::new(Formula::Atom(Atom::Action(fact("A", vec![mv("x")]), node("i")))),
            Box::new(Formula::Atom(Atom::Eq(node("i"), node("j")))),
        )),
    ));
    let la = TheoryItem::Lemma(Lemma {
        name: "la".into(), modulo: None, attributes: vec![LemmaAttr::Reuse],
        trace_quantifier: TraceQuantifier::ExistsTrace,
        formula: Formula::Exists(
            vec![var("a", SortHint::Msg), var("i", SortHint::Node)],
            Box::new(Formula::Atom(Atom::Action(fact("A", vec![mv("a")]), node("i"))))),
        proof: None, plaintext: String::new(),
    });
    let bad_eq = TheoryItem::Equations { convergent: false, eqs: vec![Equation {
        lhs: Term::App("hh".into(), vec![Term::App("ff".into(), vec![mv("x")]), mv("y")]),
        rhs: Term::App("gg".into(), vec![mv("y")]),
    }]};
    let thy = theory("Tail", vec![
        bad_eq,
        rule("R", vec![fact("Fr", vec![fresh("x")])],
            vec![fact("A", vec![fresh("x")])],
            vec![fact("Out", vec![Term::App("ff".into(), vec![fresh("x")])])]),
        ft, guard, la,
    ]);
    let ts: Vec<String> = check_theory(&thy).iter().map(|e| e.topic.clone()).collect();
    let pos = |t: &str| ts.iter().position(|x| x == t).unwrap();
    assert!(pos("Formula terms") < pos(" Formula guardedness"));
    assert!(pos(" Formula guardedness") < pos("Lemma annotations"));
    assert!(pos("Lemma annotations") < pos("Subterm Convergence Warning"));
}

fn lemma_all(name: &str, formula: Formula) -> TheoryItem {
    TheoryItem::Lemma(Lemma {
        name: name.into(), modulo: None, attributes: vec![],
        trace_quantifier: TraceQuantifier::AllTraces,
        formula, proof: None, plaintext: String::new(),
    })
}

#[test]
fn fact_lhs_occur_no_rhs_no_suggestion_when_far() {
    // A single LHS-only fact with no close RHS neighbour -> no "Perhaps".
    let thy = theory("NS", vec![
        rule("Q", vec![fact("NotInAnyRHS", vec![])], vec![], vec![fact("Out", vec![mv("x")])]),
        rule("D", vec![fact("In", vec![mv("x")])], vec![], vec![fact("Completely", vec![])]),
    ]);
    let r = fact_lhs_occur_no_rhs(&thy);
    assert_eq!(r.len(), 1);
    assert!(!r[0].message.contains("Perhaps"), "unexpected suggestion: {}", r[0].message);
}
