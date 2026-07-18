//! Round-5 byte-parity tests. Each fixture in tests/fixtures/ is an OBSERVED
//! oracle WARNING block (from `wf_oracle.sh` on the correspondingly-named
//! probe in scratchpad/probes5, or extracted from the round-5 reference
//! targets). We build the matching AST by hand and assert byte-for-byte
//! reproduction.

use wf_clean::ast::*;
use wf_clean::*;

// ---- AST builders ---------------------------------------------------------

fn var(name: &str, sort: SortHint) -> VarSpec {
    VarSpec { name: name.into(), idx: 0, sort, typ: None }
}
fn ivar(name: &str, idx: u64, sort: SortHint) -> VarSpec {
    VarSpec { name: name.into(), idx, sort, typ: None }
}
fn mv(name: &str) -> Term { Term::Var(var(name, SortHint::Msg)) }
fn fresh(name: &str) -> Term { Term::Var(var(name, SortHint::Fresh)) }
fn pub_(name: &str) -> Term { Term::Var(var(name, SortHint::Pub)) }
fn pl(name: &str) -> Term { Term::PubLit(name.into()) }
fn app(name: &str, args: Vec<Term>) -> Term { Term::App(name.into(), args) }
fn cnst(name: &str) -> Term { Term::App(name.into(), vec![]) }
fn pair(items: Vec<Term>) -> Term { Term::Pair(items) }

fn fact(name: &str, args: Vec<Term>) -> Fact {
    Fact { persistent: false, name: name.into(), args, annotations: vec![] }
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

fn lemma_q(name: &str, q: TraceQuantifier, formula: Formula) -> TheoryItem {
    TheoryItem::Lemma(Lemma {
        name: name.into(),
        modulo: None,
        attributes: vec![],
        trace_quantifier: q,
        formula,
        proof: None,
        plaintext: String::new(),
    })
}
fn lemma_ex(name: &str, f: Formula) -> TheoryItem {
    lemma_q(name, TraceQuantifier::ExistsTrace, f)
}
fn lemma_all(name: &str, f: Formula) -> TheoryItem {
    lemma_q(name, TraceQuantifier::AllTraces, f)
}

fn theory(name: &str, items: Vec<TheoryItem>) -> Theory {
    Theory { is_diff: false, name: name.into(), configuration: None, items }
}

fn equations(eqs: Vec<(Term, Term)>) -> TheoryItem {
    TheoryItem::Equations {
        convergent: false,
        eqs: eqs.into_iter().map(|(lhs, rhs)| Equation { lhs, rhs }).collect(),
    }
}

fn eq(a: Term, b: Term) -> Formula { Formula::Atom(Atom::Eq(a, b)) }
fn act(f: Fact, t: Term) -> Formula { Formula::Atom(Atom::Action(f, t)) }
fn conj(a: Formula, b: Formula) -> Formula { Formula::And(Box::new(a), Box::new(b)) }
fn disj(a: Formula, b: Formula) -> Formula { Formula::Or(Box::new(a), Box::new(b)) }
fn imp(a: Formula, b: Formula) -> Formula { Formula::Implies(Box::new(a), Box::new(b)) }
fn forall(vs: Vec<VarSpec>, g: Formula) -> Formula { Formula::Forall(vs, Box::new(g)) }
fn exists(vs: Vec<VarSpec>, g: Formula) -> Formula { Formula::Exists(vs, Box::new(g)) }

/// The probe rule shared by the g5_* guardedness probes:
/// `[ Fr(~x) ] --[ A(~x), B(~x,~x), C(~x) ]-> [ Out(~x) ]`.
fn probe_rule() -> TheoryItem {
    rule(
        "R1",
        vec![fact("Fr", vec![fresh("x")])],
        vec![
            fact("A", vec![fresh("x")]),
            fact("B", vec![fresh("x"), fresh("x")]),
            fact("C", vec![fresh("x")]),
        ],
        vec![fact("Out", vec![fresh("x")])],
    )
}

fn expect(thy: &Theory, fixture: &str) {
    let got = render_report(&check_theory(thy));
    let want = format!("/*\n{}", fixture.trim_end_matches('\n'));
    assert_eq!(got, want, "\n--- got ---\n{}\n--- want ---\n{}", got, want);
}

fn expect_silent(thy: &Theory) {
    let r = check_theory(thy);
    assert!(
        r.is_empty(),
        "expected success, got: {}",
        render_report(&r)
    );
}

// ===========================================================================
// FAMILY 1 - guardedness: equality guards, quantifier fusion, single pass
// ===========================================================================

#[test]
fn guard_eq_both_sides_unguarded() {
    // g5_e_eqself: Ex z. z = z (both sides contain the quantified var).
    let thy = theory("T", vec![
        probe_rule(),
        lemma_ex("L", exists(vec![var("z", SortHint::Untagged)], eq(mv("z"), mv("z")))),
    ]);
    expect(&thy, include_str!("fixtures/g5_e_eqself.txt"));
}

#[test]
fn guard_eq_single_left_to_right_pass() {
    // g5_e_revchain: Ex z w. (w = h(z)) & (z = 'c') - the first equality sees
    // unresolved z, and there is no second pass, so w stays unguarded.
    let thy = theory("T", vec![
        probe_rule(),
        lemma_ex("L", exists(
            vec![var("z", SortHint::Untagged), var("w", SortHint::Untagged)],
            conj(
                eq(mv("w"), app("h", vec![mv("z")])),
                eq(mv("z"), pl("c")),
            ),
        )),
    ]);
    expect(&thy, include_str!("fixtures/g5_e_revchain.txt"));
}

#[test]
fn guard_eq_disjunction_does_not_guard() {
    // g5_e_disjeq: Ex z. (z = 'c') | (z = 'd').
    let thy = theory("T", vec![
        probe_rule(),
        lemma_ex("L", exists(
            vec![var("z", SortHint::Untagged)],
            disj(eq(mv("z"), pl("c")), eq(mv("z"), pl("d"))),
        )),
    ]);
    expect(&thy, include_str!("fixtures/g5_e_disjeq.txt"));
}

#[test]
fn guard_eq_partial_reports_only_vacuous_var() {
    // g5_e_vac: All #i x. A(x)@#i ==> Ex y z. y = 'c'  (y eq-guarded, z not).
    let thy = theory("T", vec![
        probe_rule(),
        lemma_all("L", forall(
            vec![var("i", SortHint::Node), var("x", SortHint::Untagged)],
            imp(
                act(fact("A", vec![mv("x")]), Term::Var(var("i", SortHint::Node))),
                exists(
                    vec![var("y", SortHint::Untagged), var("z", SortHint::Untagged)],
                    eq(mv("y"), pl("c")),
                ),
            ),
        )),
    ]);
    expect(&thy, include_str!("fixtures/g5_e_vac.txt"));
}

#[test]
fn guard_forall_fusion_renders_fused() {
    // g5_a_nest_noimpl: All x. All #i. A(x)@#i - the report shows the FUSED
    // quantifier "∀ x #i. A( x ) @ #i" for both sub and whole.
    let thy = theory("T", vec![
        probe_rule(),
        lemma_all("L", forall(
            vec![var("x", SortHint::Untagged)],
            forall(
                vec![var("i", SortHint::Node)],
                act(fact("A", vec![mv("x")]), Term::Var(var("i", SortHint::Node))),
            ),
        )),
    ]);
    expect(&thy, include_str!("fixtures/g5_a_nest_noimpl.txt"));
}

#[test]
fn guard_exists_forall_body_unguarded() {
    // g5_e_fuse_all: Ex x. All #i. (A(x)@#i ==> C(x)@#i) - no cross-kind
    // fusion; the forall body guards nothing for the outer Ex.
    let thy = theory("T", vec![
        probe_rule(),
        lemma_ex("L", exists(
            vec![var("x", SortHint::Untagged)],
            forall(
                vec![var("i", SortHint::Node)],
                imp(
                    act(fact("A", vec![mv("x")]), Term::Var(var("i", SortHint::Node))),
                    act(fact("C", vec![mv("x")]), Term::Var(var("i", SortHint::Node))),
                ),
            ),
        )),
    ]);
    expect(&thy, include_str!("fixtures/g5_e_fuse_all.txt"));
}

#[test]
fn guard_equality_acceptances() {
    // Reference-accepted equality-guard shapes -> empty report.
    let node_i = || var("i", SortHint::Node);
    let cases: Vec<Formula> = vec![
        // g5_e_eqbare: Ex z. z = 'c'
        exists(vec![var("z", SortHint::Untagged)], eq(mv("z"), pl("c"))),
        // g5_e_eqinner: Ex z. h(z) = 'c'
        exists(vec![var("z", SortHint::Untagged)], eq(app("h", vec![mv("z")]), pl("c"))),
        // g5_e_eqpair: All k #i. A(k)@#i ==> Ex y r. k = <y, r>
        forall(
            vec![var("k", SortHint::Untagged), node_i()],
            imp(
                act(fact("A", vec![mv("k")]), Term::Var(node_i())),
                exists(
                    vec![var("y", SortHint::Untagged), var("r", SortHint::Untagged)],
                    eq(mv("k"), pair(vec![mv("y"), mv("r")])),
                ),
            ),
        ),
        // g5_e_eqmset: All x y #i. B(x,y)@#i ==> Ex z. (x++z) = y
        forall(
            vec![var("x", SortHint::Untagged), var("y", SortHint::Untagged), node_i()],
            imp(
                act(fact("B", vec![mv("x"), mv("y")]), Term::Var(node_i())),
                exists(
                    vec![var("z", SortHint::Untagged)],
                    eq(Term::BinOp(BinOp::Union, Box::new(mv("x")), Box::new(mv("z"))), mv("y")),
                ),
            ),
        ),
        // g5_e_eqchain: Ex z w. (z = 'c') & (w = h(z))
        exists(
            vec![var("z", SortHint::Untagged), var("w", SortHint::Untagged)],
            conj(eq(mv("z"), pl("c")), eq(mv("w"), app("h", vec![mv("z")]))),
        ),
        // g5_e_actorder: All k #j. C(k)@#j ==> Ex y w #i. (w = h(y)) & A(y)@#i
        forall(
            vec![var("k", SortHint::Untagged), var("j", SortHint::Node)],
            imp(
                act(fact("C", vec![mv("k")]), Term::Var(var("j", SortHint::Node))),
                exists(
                    vec![var("y", SortHint::Untagged), var("w", SortHint::Untagged), node_i()],
                    conj(
                        eq(mv("w"), app("h", vec![mv("y")])),
                        act(fact("A", vec![mv("y")]), Term::Var(node_i())),
                    ),
                ),
            ),
        ),
        // g5_e_pairboth: Ex y z. <y, z> = 'c'
        exists(
            vec![var("y", SortHint::Untagged), var("z", SortHint::Untagged)],
            eq(pair(vec![mv("y"), mv("z")]), pl("c")),
        ),
        // g5_e_selfnoop: Ex z. (z = 'c') & (z = z)
        exists(
            vec![var("z", SortHint::Untagged)],
            conj(eq(mv("z"), pl("c")), eq(mv("z"), mv("z"))),
        ),
        // g5_e_eqtemp: All #j x. A(x)@#j ==> Ex #i. #i = #j
        forall(
            vec![var("j", SortHint::Node), var("x", SortHint::Untagged)],
            imp(
                act(fact("A", vec![mv("x")]), Term::Var(var("j", SortHint::Node))),
                exists(
                    vec![node_i()],
                    eq(Term::Var(node_i()), Term::Var(var("j", SortHint::Node))),
                ),
            ),
        ),
        // g5_e_nest: Ex x. Ex #i. A(x)@#i  (existential fusion)
        exists(
            vec![var("x", SortHint::Untagged)],
            exists(vec![node_i()], act(fact("A", vec![mv("x")]), Term::Var(node_i()))),
        ),
        // g5_a_nest: All x. All #i. A(x)@#i ==> C(x)@#i  (forall fusion)
        forall(
            vec![var("x", SortHint::Untagged)],
            forall(
                vec![node_i()],
                imp(
                    act(fact("A", vec![mv("x")]), Term::Var(node_i())),
                    act(fact("C", vec![mv("x")]), Term::Var(node_i())),
                ),
            ),
        ),
        // g5_a_eqg2: All w x #i. ((w = h(x)) & A(x)@#i) ==> C(w)@#i
        forall(
            vec![var("w", SortHint::Untagged), var("x", SortHint::Untagged), node_i()],
            imp(
                conj(
                    eq(mv("w"), app("h", vec![mv("x")])),
                    act(fact("A", vec![mv("x")]), Term::Var(node_i())),
                ),
                act(fact("C", vec![mv("w")]), Term::Var(node_i())),
            ),
        ),
        // g5_a_eqonly: All w. (w = 'c') ==> (w = 'c')
        forall(
            vec![var("w", SortHint::Untagged)],
            imp(eq(mv("w"), pl("c")), eq(mv("w"), pl("c"))),
        ),
    ];
    for (i, f) in cases.into_iter().enumerate() {
        let thy = theory("T", vec![probe_rule(), lemma_all("L", f)]);
        let r = check_theory(&thy);
        assert!(r.is_empty(), "case {} reported: {}", i, render_report(&r));
    }
}

// ===========================================================================
// FAMILY 2 - Variable with mismatching sorts or capitalization
// ===========================================================================

#[test]
fn sorts_index_is_part_of_identity() {
    // s5_idx: $x.1 with x.2 -> silent (different indices, different vars).
    let thy = theory("T", vec![rule(
        "R1",
        vec![
            fact("In", vec![Term::Var(ivar("x", 1, SortHint::Pub))]),
            fact("In", vec![Term::Var(ivar("x", 2, SortHint::Untagged))]),
        ],
        vec![fact("M", vec![
            Term::Var(ivar("x", 1, SortHint::Pub)),
            Term::Var(ivar("x", 2, SortHint::Untagged)),
        ])],
        vec![],
    )]);
    expect_silent(&thy);
}

#[test]
fn sorts_same_index_clash_renders_index() {
    // s5_idxsame: $x.1 with x.1 -> "1. $x.1, x.1".
    let thy = theory("T", vec![rule(
        "R1",
        vec![
            fact("In", vec![Term::Var(ivar("x", 1, SortHint::Pub))]),
            fact("In", vec![Term::Var(ivar("x", 1, SortHint::Untagged))]),
        ],
        vec![fact("M", vec![
            Term::Var(ivar("x", 1, SortHint::Pub)),
            Term::Var(ivar("x", 1, SortHint::Untagged)),
        ])],
        vec![],
    )]);
    expect(&thy, include_str!("fixtures/s5_idxsame.txt"));
}

#[test]
fn sorts_capitalization_same_index() {
    // s5_capidx: $X.1 with $x.1 -> "1. $X.1, $x.1".
    let thy = theory("T", vec![rule(
        "R1",
        vec![
            fact("In", vec![Term::Var(ivar("X", 1, SortHint::Pub))]),
            fact("In", vec![Term::Var(ivar("x", 1, SortHint::Pub))]),
        ],
        vec![fact("M", vec![
            Term::Var(ivar("X", 1, SortHint::Pub)),
            Term::Var(ivar("x", 1, SortHint::Pub)),
        ])],
        vec![],
    )]);
    expect(&thy, include_str!("fixtures/s5_capidx.txt"));
}

#[test]
fn sorts_groups_sorted_with_separator_line() {
    // s5_groups: zz before aa in the rule; groups print sorted (aa first)
    // with a four-space separator line between the numbered groups.
    let thy = theory("T", vec![rule(
        "R1",
        vec![
            fact("In", vec![pub_("zz")]),
            fact("In", vec![mv("zz")]),
            fact("In", vec![pub_("aa")]),
            fact("In", vec![mv("aa")]),
        ],
        vec![fact("M", vec![mv("zz"), mv("aa")])],
        vec![],
    )]);
    expect(&thy, include_str!("fixtures/s5_groups.txt"));
}

#[test]
fn sorts_variant_order_is_sort_then_name() {
    // s5_crossname: ~X with $x -> "$x, ~X" (sort rank beats name order).
    let thy = theory("T", vec![rule(
        "R1",
        vec![
            fact("Fr", vec![fresh("X")]),
            fact("In", vec![pub_("x")]),
        ],
        vec![fact("M", vec![fresh("X"), pub_("x")])],
        vec![],
    )]);
    expect(&thy, include_str!("fixtures/s5_crossname.txt"));
}

#[test]
fn sorts_suffix_spelling_is_its_sigil_sort() {
    // s5_suffix2: x:pub with ~x -> "$x, ~x"; s5_suffix: x:pub with $x is the
    // SAME variable (silent).
    let suffix_pub = || Term::Var(var("x", SortHint::Suffix(SuffixSort::Pub)));
    let thy = theory("T", vec![rule(
        "R1",
        vec![fact("In", vec![suffix_pub()]), fact("Fr", vec![fresh("x")])],
        vec![fact("M", vec![suffix_pub(), fresh("x")])],
        vec![],
    )]);
    expect(&thy, include_str!("fixtures/s5_suffix2.txt"));

    let thy2 = theory("T", vec![rule(
        "R1",
        vec![fact("In", vec![suffix_pub()]), fact("In", vec![pub_("x")])],
        vec![fact("M", vec![suffix_pub(), pub_("x")])],
        vec![],
    )]);
    expect_silent(&thy2);
}

#[test]
fn sorts_four_variant_order() {
    // s5_all4b: $x, ~x, x, %x in one rule -> a single group listing the
    // variants in the order pub, fresh, msg, nat (the probe's oracle output
    // also carries an out-of-scope Message Derivation Checks block, so this
    // pins the check body rather than the whole report).
    let thy = theory("T", vec![rule(
        "R1",
        vec![
            fact("Fr", vec![fresh("x")]),
            fact("In", vec![pub_("x")]),
            fact("In", vec![mv("x")]),
            fact("In", vec![Term::Var(var("x", SortHint::Nat))]),
        ],
        vec![],
        vec![],
    )]);
    let r = checks::mismatching_sorts(&thy);
    assert_eq!(r.len(), 1);
    assert!(
        r[0].message.ends_with("  rule `R1': \n    1. $x, ~x, x, %x"),
        "got: {}",
        r[0].message
    );
}

// ===========================================================================
// FAMILY 3 - reserved-name normalization, list alignment, arity dedup,
//            subterm-convergence rendering
// ===========================================================================

#[test]
fn reserved_ku_normalizes_in_all_positions() {
    // t5_ku_all: [ Ku(x) ] --[ Ku(x) ]-> [ Ku(x) ] -> three entries, each
    // rendered as the canonical persistent !KU( x ); no lhs-not-rhs entry.
    let thy = theory("T", vec![rule(
        "r1",
        vec![fact("Ku", vec![mv("x")])],
        vec![fact("Ku", vec![mv("x")])],
        vec![fact("Ku", vec![mv("x")])],
    )]);
    expect(&thy, include_str!("fixtures/t5_ku_all.txt"));
}

#[test]
fn reserved_uppercase_io_normalizes() {
    // t5_up_inout: [ IN(x) ] --[ OUT(x) ]-> [ FR(x) ] -> premise silent
    // (IN = In is legal there), action reported as reserved `Out( x )`,
    // conclusion reported as special `Fr( x )`.
    let thy = theory("T", vec![rule(
        "r1",
        vec![fact("IN", vec![mv("x")])],
        vec![fact("OUT", vec![mv("x")])],
        vec![fact("FR", vec![mv("x")])],
    )]);
    expect(&thy, include_str!("fixtures/t5_up_inout.txt"));
}

#[test]
fn lhs_not_rhs_numbers_right_aligned() {
    // t5_align: ten LHS-only facts -> "   1." through "  10." (right-aligned
    // to the widest index).
    let mut items = vec![rule(
        "G",
        vec![fact("Fr", vec![fresh("z")])],
        vec![],
        (1..=10)
            .map(|i| fact(&format!("P{:02}", i), vec![fresh("z")]))
            .collect(),
    )];
    for i in 1..=10 {
        items.push(rule(
            &format!("C{:02}", i),
            vec![
                fact(&format!("P{:02}", i), vec![mv("x")]),
                fact(&format!("Q{:02}", i), vec![mv("x")]),
            ],
            vec![],
            vec![],
        ));
    }
    let thy = theory("T", items);
    expect(&thy, include_str!("fixtures/t5_align.txt"));
}

#[test]
fn lemma_arity_items_not_deduped_across_renders() {
    // t5_lemdup: the same lemma fact at two binder depths -> BOTH items
    // listed (`Bound 3,2,1` and `Bound 4,3,2`).
    let bt3 = |a: &str, b: &str, k: &str| {
        fact("Bt", vec![mv(a), mv(b), mv(k)])
    };
    let thy = theory("T", vec![
        rule(
            "R1",
            vec![fact("Fr", vec![fresh("x")])],
            vec![fact("Bt", vec![fresh("x")])],
            vec![fact("Out", vec![fresh("x")])],
        ),
        rule(
            "R2",
            vec![
                fact("Fr", vec![fresh("x")]),
                fact("Fr", vec![fresh("y")]),
                fact("Fr", vec![fresh("z")]),
            ],
            vec![fact("Bt", vec![fresh("x"), fresh("y"), fresh("z")])],
            vec![fact("Out", vec![fresh("x")])],
        ),
        lemma_ex("L", exists(
            vec![
                var("a", SortHint::Untagged),
                var("b", SortHint::Untagged),
                var("k", SortHint::Untagged),
                var("i", SortHint::Node),
            ],
            conj(
                act(bt3("a", "b", "k"), Term::Var(var("i", SortHint::Node))),
                exists(
                    vec![var("j", SortHint::Node)],
                    act(bt3("a", "b", "k"), Term::Var(var("j", SortHint::Node))),
                ),
            ),
        )),
    ]);
    expect(&thy, include_str!("fixtures/t5_lemdup.txt"));
}

// ---- Subterm Convergence Warning rendering --------------------------------

#[test]
fn subterm_flagged_equations_sorted() {
    // t5_sub_order2: f(z) = hh(z) before f(a) = hh(a) in source; the report
    // lists f(a) first (sorted by rendered equation).
    let thy = theory("T", vec![
        equations(vec![
            (app("f", vec![mv("z")]), app("hh", vec![mv("z")])),
            (app("f", vec![mv("a")]), app("hh", vec![mv("a")])),
        ]),
        rule(
            "R1",
            vec![fact("Fr", vec![fresh("x")])],
            vec![fact("A", vec![fresh("x")])],
            vec![fact("Out", vec![fresh("x")])],
        ),
    ]);
    expect(&thy, include_str!("fixtures/t5_sub_order2.txt"));
}

fn subterm_probe_theory(eqs: Vec<(Term, Term)>) -> Theory {
    theory("T", vec![
        equations(eqs),
        rule(
            "R1",
            vec![fact("Fr", vec![fresh("x")])],
            vec![fact("A", vec![fresh("x")])],
            vec![fact("Out", vec![fresh("x")])],
        ),
    ])
}

#[test]
fn subterm_equation_tuple_fill() {
    // t5_tup3: two elements fill the first line (trailing ", "), the third
    // wraps with the closing > glued.
    let g = "g0000000000000000012";
    let thy = subterm_probe_theory(vec![(
        app("f", vec![mv("x"), mv("y"), mv("z")]),
        pair(vec![
            app(g, vec![mv("x")]),
            app(g, vec![mv("y")]),
            app(g, vec![mv("z")]),
        ]),
    )]);
    expect(&thy, include_str!("fixtures/t5_tup3.txt"));
}

#[test]
fn subterm_equation_tuple_fill_wide_elements() {
    // t5_tup3b: the second element no longer fits the first line; the second
    // line takes elements two and three with > glued.
    let g = "g0000000000000000000000000000";
    let thy = subterm_probe_theory(vec![(
        app("f", vec![mv("x"), mv("y"), mv("z")]),
        pair(vec![
            app(g, vec![mv("x")]),
            app(g, vec![mv("y")]),
            app(g, vec![mv("z")]),
        ]),
    )]);
    expect(&thy, include_str!("fixtures/t5_tup3b.txt"));
}

#[test]
fn subterm_equation_closer_breaks_alone() {
    // t5_last36: the elements fit but the closing > exceeds the margin and
    // drops to the tuple's start column.
    let thy = subterm_probe_theory(vec![(
        app("f", vec![mv("x"), mv("y")]),
        pair(vec![
            app("g0000000000000000001", vec![mv("x")]),
            app("h00000000000000000000000000000000036", vec![mv("y")]),
        ]),
    )]);
    expect(&thy, include_str!("fixtures/t5_last36.txt"));
}

#[test]
fn subterm_equation_last_element_breaks_closer_glues() {
    // t5_last37: one char wider - the last element wraps and > glues to it.
    let thy = subterm_probe_theory(vec![(
        app("f", vec![mv("x"), mv("y")]),
        pair(vec![
            app("g0000000000000000001", vec![mv("x")]),
            app("h000000000000000000000000000000000037", vec![mv("y")]),
        ]),
    )]);
    expect(&thy, include_str!("fixtures/t5_last37.txt"));
}

/// Render one topic block (header + underline + body) the way the report does.
fn render_block(e: &report::WfError) -> String {
    format!("{}\n\n{}", underline_topic(&e.topic), e.message)
}

#[test]
fn subterm_ble_reference_block() {
    // The ble.spthy reference target: f4/f6/g2 flagged (sorted), f6 wraps at
    // the `=`; convergent and ground-RHS equations are not flagged.
    let aes = |k: Term, d: Term| app("aes_cmac", vec![k, d]);
    let thy = subterm_probe_theory(vec![
        (
            app("f4", vec![mv("u"), mv("v"), mv("x"), mv("z")]),
            aes(mv("x"), pair(vec![mv("u"), mv("v"), mv("z")])),
        ),
        (
            app("g2", vec![mv("u"), mv("v"), mv("x"), mv("y")]),
            aes(mv("x"), pair(vec![mv("u"), mv("v"), mv("y")])),
        ),
        (
            app("f6", vec![mv("w"), mv("n1"), mv("n2"), mv("r"), mv("iocap"), mv("a1"), mv("a2")]),
            aes(mv("w"), pair(vec![mv("n1"), mv("n2"), mv("r"), mv("iocap"), mv("a1"), mv("a2")])),
        ),
        // recover(split1(x), split2(x)) = x is subterm convergent -> silent.
        (
            app("recover", vec![app("split1", vec![mv("x")]), app("split2", vec![mv("x")])]),
            mv("x"),
        ),
    ]);
    let r = checks::subterm_convergence(&thy);
    assert_eq!(r.len(), 1);
    let want = include_str!("fixtures/t5_ble_subterm.txt");
    assert_eq!(render_block(&r[0]), want.trim_end_matches('\n'));
}

#[test]
fn subterm_mesh_reference_block() {
    // The mesh.spthy reference target: k1..k4 and s1 flagged (sorted; source
    // order starts with s1), the k2 tuple laid out across ten lines; the
    // ground-RHS aes_ccm_verify equation and the convergent aes_ccm_dec/enc
    // equations are not flagged.
    let aes = |k: Term, d: Term| app("aes_cmac", vec![k, d]);
    let c = || aes(app("s1", vec![cnst("smk2")]), mv("n"));
    let a1 = || aes(c(), pair(vec![mv("p"), cnst("nb_one")]));
    let a2 = || aes(c(), pair(vec![a1(), mv("p"), cnst("nb_two")]));
    let a3 = aes(c(), pair(vec![a2(), mv("p"), cnst("nb_three")]));
    let thy = subterm_probe_theory(vec![
        (app("s1", vec![mv("m")]), aes(cnst("null"), mv("m"))),
        (
            app("k1", vec![mv("n"), mv("salt"), mv("p")]),
            aes(aes(mv("salt"), mv("n")), mv("p")),
        ),
        (app("k2", vec![mv("n"), mv("p")]), pair(vec![a1(), a2(), a3])),
        (
            app("k3", vec![mv("n")]),
            aes(aes(app("s1", vec![cnst("smk3")]), mv("n")), cnst("id7")),
        ),
        (
            app("k4", vec![mv("n")]),
            aes(aes(app("s1", vec![cnst("smk4")]), mv("n")), cnst("id6")),
        ),
        (
            app("aes_ccm_dec", vec![mv("k"), mv("n"), app("aes_ccm_enc", vec![mv("k"), mv("n"), mv("m")])]),
            mv("m"),
        ),
        (
            app("aes_ccm_verify", vec![app("aes_ccm_enc", vec![mv("k"), mv("n"), mv("m")]), mv("k"), mv("n"), mv("m")]),
            cnst("true_val"),
        ),
    ]);
    let r = checks::subterm_convergence(&thy);
    assert_eq!(r.len(), 1);
    let want = include_str!("fixtures/t5_mesh_subterm.txt");
    assert_eq!(render_block(&r[0]), want.trim_end_matches('\n'));
}

// ===========================================================================
// Whole-theory reference targets (round5/targets)
// ===========================================================================

#[test]
fn issue515_reference_block() {
    // regression/trace/issue515.spthy: reserved names matched
    // case-insensitively (Ku/Kd report as !KU/!KD) plus the test2 special
    // facts; no lhs-not-rhs topic (K appears on a right-hand side, the other
    // builtins are excluded).
    let thy = theory("issue515", vec![
        rule("test",
            vec![fact("K", vec![mv("x")])],
            vec![fact("KU", vec![mv("x")])],
            vec![fact("KD", vec![mv("x")])]),
        rule("test2",
            vec![fact("Out", vec![mv("x")])],
            vec![fact("KD", vec![mv("x")])],
            vec![fact("In", vec![mv("x")])]),
        rule("test3",
            vec![fact("K", vec![mv("x")])],
            vec![fact("In", vec![mv("x")])],
            vec![fact("K", vec![mv("x")])]),
        rule("test4",
            vec![fact("Ku", vec![mv("x")])],
            vec![fact("Ku", vec![mv("x")])],
            vec![fact("Out", vec![mv("x")])]),
        rule("test5",
            vec![fact("Kd", vec![mv("x")])],
            vec![fact("Kd", vec![mv("x")])],
            vec![fact("Ku", vec![mv("x")])]),
    ]);
    expect(&thy, include_str!("fixtures/t5_issue515.txt"));
}

#[test]
fn axioms_and_induction_reference_block() {
    // loops/Axioms_and_Induction.spthy: only "Lemma annotations" (reuse on an
    // exists-trace lemma); the guarded restriction and lemmas are silent.
    let node_i = || var("i", SortHint::Node);
    let step_lemma = || forall(
        vec![var("x", SortHint::Untagged), node_i()],
        imp(
            act(fact("Step", vec![mv("x")]), Term::Var(node_i())),
            Formula::False,
        ),
    );
    let mut exists_test = Lemma {
        name: "Exists_test".into(),
        modulo: None,
        attributes: vec![LemmaAttr::Reuse],
        trace_quantifier: TraceQuantifier::ExistsTrace,
        formula: step_lemma(),
        proof: None,
        plaintext: String::new(),
    };
    exists_test.attributes = vec![LemmaAttr::Reuse];
    let thy = theory("Axioms_and_Induction", vec![
        rule("Start",
            vec![fact("Fr", vec![mv("x")])],
            vec![fact("Start", vec![mv("x")])],
            vec![fact("A", vec![mv("x")])]),
        rule("Step",
            vec![fact("A", vec![mv("x")])],
            vec![fact("Step", vec![mv("x")])],
            vec![fact("B", vec![mv("x")])]),
        rule("Stop",
            vec![fact("B", vec![mv("x")])],
            vec![fact("Stop", vec![mv("x")])],
            vec![]),
        TheoryItem::Restriction(Restriction {
            name: "Start_implies_Stop".into(),
            formula: forall(
                vec![var("x", SortHint::Untagged), node_i()],
                imp(
                    act(fact("Start", vec![mv("x")]), Term::Var(node_i())),
                    exists(
                        vec![var("j", SortHint::Node)],
                        act(fact("Stop", vec![mv("x")]), Term::Var(var("j", SortHint::Node))),
                    ),
                ),
            ),
            attributes: vec![],
        }),
        TheoryItem::Lemma(exists_test),
        lemma_q("NoStep_with_induction", TraceQuantifier::AllTraces, step_lemma()),
        lemma_q("NoStep_without_induction", TraceQuantifier::AllTraces, step_lemma()),
    ]);
    expect(&thy, include_str!("fixtures/t5_axioms_induction.txt"));
}

#[test]
fn sapic_lookup_rule_unbound_block() {
    // t5_lookup (mirrors the OCSPS/CertificateTransparency translated lookup
    // rules): the lookup variable is unbound and the state fact is LHS-only
    // with an edit-distance suggestion. (The oracle's trailing Message
    // Derivation Checks block is Maude-computed and out of scope.)
    let thy = theory("Tt5_lookup", vec![rule(
        "lookup_time_as_t_0_11121111",
        vec![fact("State_11121111", vec![fresh("lock8"), fresh("lock9"), mv("sk")])],
        vec![fact("IsIn", vec![pl("time"), fresh("t")])],
        vec![fact("State_111211111", vec![fresh("lock8"), fresh("lock9"), fresh("t"), mv("sk")])],
    )]);
    expect(&thy, include_str!("fixtures/t5_lookup.txt"));
}

#[test]
fn issue527_reference_block() {
    // regression/trace/issue527.spthy (minus the Maude-computed Message
    // Derivation Checks tail): pins index-aware sort grouping, first-rule
    // public-name attribution with sorted groups, case-insensitive reserved
    // facts, fact-capitalization groups, the un-deduplicated lemma arity
    // items and the K entry in the lhs-not-rhs list.
    let node = |n: &str| Term::Var(var(n, SortHint::Node));
    let bt = |a: Term, b: Term, k: Term| fact("B_TEST", vec![a, b, k]);
    let pfact = |name: &str, args: Vec<Term>| Fact {
        persistent: true,
        name: name.into(),
        args,
        annotations: vec![],
    };
    let lemma_formula = exists(
        vec![
            var("A", SortHint::Untagged),
            var("B", SortHint::Untagged),
            var("k", SortHint::Untagged),
            var("i", SortHint::Node),
        ],
        conj(
            conj(
                conj(
                    act(bt(mv("A"), mv("B"), mv("k")), node("i")),
                    exists(
                        vec![var("j", SortHint::Node)],
                        conj(
                            act(bt(mv("A"), mv("B"), mv("k")), node("j")),
                            Formula::Atom(Atom::Less(node("j"), node("i"))),
                        ),
                    ),
                ),
                Formula::Not(Box::new(exists(
                    vec![var("r", SortHint::Node)],
                    act(fact("Register_pk", vec![mv("A")]), node("r")),
                ))),
            ),
            Formula::Not(Box::new(exists(
                vec![var("a", SortHint::Node)],
                act(fact("Register_pk", vec![mv("B")]), node("a")),
            ))),
        ),
    );
    let mut register = Rule {
        name: "Register_pk".into(),
        modulo: None,
        attributes: vec![],
        let_block: vec![],
        premises: vec![fact("Fr", vec![fresh("ltk")])],
        actions: vec![],
        conclusions: vec![
            pfact("Ltk", vec![pub_("A"), mv("ltk")]),
            pfact("Pk", vec![pub_("A"), app("pk", vec![fresh("ltk")])]),
        ],
        embedded_restrictions: vec![],
        variants: vec![],
        left_right: None,
    };
    register.modulo = None;
    let mut one = Rule {
        name: "One".into(),
        modulo: None,
        attributes: vec![],
        let_block: vec![LetBinding {
            var: mv("m1"),
            value: pair(vec![pl("1"), pub_("A"), fresh("Na")]),
        }],
        premises: vec![],
        actions: vec![
            fact("OneResultingIn", vec![pl("second")]),
            fact("Fact", vec![]),
        ],
        conclusions: vec![
            fact("OneResultingIn", vec![pl("seconD")]),
            fact("Out", vec![mv("m1")]),
        ],
        embedded_restrictions: vec![],
        variants: vec![],
        left_right: None,
    };
    one.modulo = None;
    let mut four = Rule {
        name: "Four".into(),
        modulo: None,
        attributes: vec![],
        let_block: vec![LetBinding { var: mv("m"), value: pl("msg") }],
        premises: vec![],
        actions: vec![bt(pl("firSt"), pl("second"), mv("m1"))],
        conclusions: vec![
            fact("OneresltingIn", vec![pl("second")]),
            fact("Out", vec![pl("1")]),
        ],
        embedded_restrictions: vec![],
        variants: vec![],
        left_right: None,
    };
    four.modulo = None;
    let thy = theory("issue527", vec![
        TheoryItem::Rule(register),
        rule("Test_1",
            vec![
                fact("In", vec![app("aenc", vec![pair(vec![pl("1"), pub_("A"), mv("m")]), mv("pkB")])]),
                fact("F", vec![pub_("X")]),
                fact("Vars", vec![mv("rhs")]),
            ],
            vec![fact("Fr", vec![pub_("x")])],
            vec![fact("F", vec![pub_("x")]), fact("Test_1", vec![pub_("A")])]),
        TheoryItem::Rule(one),
        rule("Two",
            vec![],
            vec![fact("B_TEST", vec![pl("first")])],
            vec![fact("OneresultingIn", vec![pl("second")])]),
        rule("three",
            vec![],
            vec![fact("B_TEST", vec![pl("third")])],
            vec![fact("OneResltingIn", vec![pl("second")])]),
        TheoryItem::Rule(four),
        rule("test",
            vec![fact("K", vec![mv("x")])],
            vec![fact("KU", vec![mv("x")])],
            vec![fact("KD", vec![mv("x")])]),
        lemma_ex("AB_key_honst", lemma_formula),
    ]);
    expect(&thy, include_str!("fixtures/t5_issue527.txt"));
}
