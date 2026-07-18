//! GENERATION tests (BEHAVIOR.md §6): build a [`System`] over the crate's own
//! input model and assert that [`generate`] emits a byte-exact DOT payload,
//! reproducing a real (live-probed) graph — id/port allocation, record groups,
//! fact rendering, the invtrapezium open-target node, and the edge vocabulary all
//! at once. This is a real generation fixture, not a serialization round-trip.

use graph_clean::dot::to_dot;
use graph_clean::generate::*;
use graph_clean::render::Fact;
use graph_clean::term::Term;

fn fresh(n: &str) -> Term {
    Term::fresh(n)
}
fn pubv(n: &str) -> Term {
    Term::pubv(n)
}

/// Reproduce `tests/fixtures/nsl_invtrap.dot` — a live NSLPK3 source-case graph
/// with two rule records, an invtrapezium open target, and two edges — purely by
/// building a `System` and generating it.
#[test]
fn generate_reproduces_nsl_invtrap_byte_exact() {
    let pk = |v: &str| Term::app("pk", vec![fresh(v)]);

    // Node 0: rule instance #vr.3 : Register_pk
    let register = RuleInstance::new("vr.3", "Register_pk", "#d5d897")
        .premises(vec![Fact::new("Fr", vec![fresh("ltkA.4")])])
        .conclusions(vec![
            Fact::new("!Ltk", vec![pubv("A.4"), fresh("ltkA.4")]),
            Fact::new("!Pk", vec![pubv("A.4"), pk("ltkA.4")]),
            Fact::new("Out", vec![pk("ltkA.4")]),
        ]);

    // Node 1: the Fresh source rule (no premises).
    let fresh_rule = RuleInstance::new("vf.5", "Fresh", "#a8a4eb")
        .conclusions(vec![Fact::new("Fr", vec![fresh("ltkA.4")])]);

    let sys = System {
        nodes: vec![
            GraphNode::Rule(register),
            GraphNode::Rule(fresh_rule),
            GraphNode::OpenTarget { node_var: "i".into(), premise_index: 0 },
        ],
        edges: vec![
            // conclusion 0 (!Ltk, port n2) of node 0 -> the open target (n9)
            SysEdge::new(EndRef::Conclusion(0, 0), EndRef::Whole(2), EdgeStyle::StructuralGray),
            // conclusion 0 (Fr, port n7) of node 1 -> premise 0 (port n0) of node 0
            SysEdge::new(EndRef::Conclusion(1, 0), EndRef::Premise(0, 0), EdgeStyle::Message),
        ],
        legend_html: None,
        legend_edges: Vec::new(),
    };

    let expected = include_str!("fixtures/nsl_invtrap.dot");
    assert_eq!(to_dot(&generate(&sys)), expected);
}

/// The generator omits an empty premise group (info always present) — the Fresh
/// rule renders `{info}|{concl}`, exactly as observed.
#[test]
fn empty_premise_group_is_dropped() {
    let sys = System {
        nodes: vec![GraphNode::Rule(
            RuleInstance::new("vf.5", "Fresh", "#a8a4eb")
                .conclusions(vec![Fact::new("Fr", vec![fresh("x")])]),
        )],
        ..System::default()
    };
    let dot = to_dot(&generate(&sys));
    // ports: info n0, concl n1, node n2 (no premise port allocated).
    assert!(dot.contains("n2[shape=\"record\",label=\"{{<n0> #vf.5 : Fresh}|{<n1> Fr( ~x )}}\""));
}

/// Knowledge / action / compressed ellipse kinds render with the observed labels
/// and colors, and the id allocator advances one per ellipse.
#[test]
fn ellipse_kinds_and_colors() {
    let sys = System {
        nodes: vec![
            GraphNode::Knowledge { term: "k".into(), temporal: "vk".into() },
            GraphNode::Action { fact: "Accept( Test, A, B, k )".into(), temporal: "i1".into() },
            GraphNode::Compressed { temporal: "vf.7".into(), rule: "isend".into() },
        ],
        ..System::default()
    };
    let dot = to_dot(&generate(&sys));
    assert!(dot.contains("n0[label=\"!KU( k ) @ #vk\",shape=\"ellipse\",color=\"gray\"];"));
    assert!(dot.contains(
        "n1[label=\"Accept( Test, A, B, k ) @ #i1\",shape=\"ellipse\",color=\"darkblue\"];"
    ));
    assert!(dot.contains("n2[label=\"#vf.7 : isend\",shape=\"ellipse\"];"));
}

// -------------------------------------------------------------------------
// Round 5 — record-cell WRAP wired into generation (BEHAVIOR.md §3f).
// Each fixture is a live-probed single-node graph whose one wide conclusion cell
// breaks across the fill-width boundary; `generate` must reproduce it byte-exact,
// exercising the greedy fill + delimiter peel through the real record builder.
// -------------------------------------------------------------------------

/// A one-rule theory `E{n}: [Fr(~s)] --[E{n}()]-> [Out(<'a01'..'a{n}'>)]` graph:
/// build the System and compare to the captured `wrap_E{n}.dot`.
fn wrap_element_case(n: usize, fixture: &str) {
    let consts: Vec<Term> = (1..=n).map(|k| Term::cst(&format!("a{:02}", k))).collect();
    let rule = RuleInstance::new("i", &format!("E{}", n), "#d5d897")
        .premises(vec![Fact::new("Fr", vec![fresh("s")])])
        .actions(vec![Fact::new(&format!("E{}", n), vec![])])
        .conclusions(vec![Fact::new("Out", vec![Term::tuple(consts)])]);
    let sys = System { nodes: vec![GraphNode::Rule(rule)], ..System::default() };
    assert_eq!(to_dot(&generate(&sys)), fixture);
}

#[test]
fn wrap_e12_greedy_fill_and_paren_peel() {
    // 12 five-column elements: line0 packs 11 + trailing ", ", 'a12' wraps to the
    // <-column indent, the tuple '>' stays with it, the fact ')' peels to col 0.
    wrap_element_case(12, include_str!("fixtures/wrap_E12.dot"));
}

#[test]
fn wrap_e13_and_e14_continuation_packs_greedily() {
    // The continuation line packs greedily to the same width as the first line
    // (two elements for E13, three for E14) — the one-element-lookahead residual is
    // subsumed by the greedy fill.
    wrap_element_case(13, include_str!("fixtures/wrap_E13.dot"));
    wrap_element_case(14, include_str!("fixtures/wrap_E14.dot"));
}

/// A one-rule theory `W{p}: … [Out(<'a…a'(p), 'y'>)]` graph, exercising the tuple
/// `>` peel at the boundary.
fn wrap_width_case(p: usize, fixture: &str) {
    let atom = Term::cst(&"a".repeat(p));
    let rule = RuleInstance::new("i", &format!("W{}", p), "#d5d897")
        .premises(vec![Fact::new("Fr", vec![fresh("s")])])
        .actions(vec![Fact::new(&format!("W{}", p), vec![])])
        .conclusions(vec![Fact::new("Out", vec![Term::tuple(vec![atom, Term::cst("y")])])]);
    let sys = System { nodes: vec![GraphNode::Rule(rule)], ..System::default() };
    assert_eq!(to_dot(&generate(&sys)), fixture);
}

#[test]
fn wrap_w71_fits_at_87() {
    // Flat width 87 stays on one line (no `\l`): the boundary is inclusive.
    wrap_width_case(71, include_str!("fixtures/wrap_W71.dot"));
}

#[test]
fn wrap_w72_paren_peels_when_content_fits_but_close_does_not() {
    // The tuple fits on line0 (`'y'>`); the fact ` )` (col 88) does not, so `)` peels.
    wrap_width_case(72, include_str!("fixtures/wrap_W72.dot"));
}

#[test]
fn wrap_w74_tuple_close_peels_to_open_column() {
    // The last element fills to col 87, so the tuple `>` peels to the `<` column
    // (5) on its own line, then the fact `)` peels to col 0.
    wrap_width_case(74, include_str!("fixtures/wrap_W74.dot"));
}

// -------------------------------------------------------------------------
// Round 5 — role CLUSTER subgraphs (BEHAVIOR.md §4).
// -------------------------------------------------------------------------

/// Reproduce `tests/fixtures/cluster_process.dot` — a live SAPIC source-case graph
/// (`cluster_Process_Session_1`, one role record, white node fill / black font) —
/// purely by building a clustered `System`.
#[test]
fn generate_reproduces_live_sapic_cluster_byte_exact() {
    let m = |n: &str| Term::msg(n);
    let record = RuleInstance::new("t", "eventNewKeyhk", "#ffffff")
        .role("Process", "black")
        .cluster("Process_Session_1", "#36A5D84C")
        .premises(vec![Fact::new("State_111111", vec![m("h"), m("k")])])
        .actions(vec![Fact::new("NewKey", vec![m("h"), m("k")])])
        .conclusions(vec![Fact::new("State_1111111", vec![m("h"), m("k")])]);
    let sys = System { nodes: vec![GraphNode::Rule(record)], ..System::default() };
    assert_eq!(to_dot(&generate(&sys)), include_str!("fixtures/cluster_process.dot"));
}

/// Reproduce corpus `cluster_multi.dot` (79c16911ad179d51): four free ellipses
/// (compressed intruder rules + gray `!KU`), then ONE `cluster_User_Session_1`
/// record, then two red-dashed deduction edges — exercising the full free→cluster
/// →edge emission order and id allocation.
#[test]
fn generate_reproduces_multi_cluster_corpus_byte_exact() {
    let m = |n: &str| Term::msg(n);
    let record = RuleInstance::new("i", "eventExclusivelmrm", "#80406c")
        .role("User", "white")
        .cluster("User_Session_1", "#D036D84C")
        .premises(vec![Fact::new("State_111121111", vec![m("x"), m("y"), m("s"), m("sk")])])
        .actions(vec![Fact::new("Exclusive", vec![m("x"), m("y")])])
        .conclusions(vec![Fact::new("State_1111211111", vec![m("x"), m("y"), m("s"), m("sk")])]);
    let sys = System {
        nodes: vec![
            GraphNode::Compressed { temporal: "k1".into(), rule: "isend[K( x )]".into() },
            GraphNode::Compressed { temporal: "k2".into(), rule: "isend[K( y )]".into() },
            GraphNode::Knowledge { term: "x".into(), temporal: "vk".into() },
            GraphNode::Knowledge { term: "y".into(), temporal: "vk.1".into() },
            GraphNode::Rule(record),
        ],
        edges: vec![
            SysEdge::new(EndRef::Whole(2), EndRef::Whole(0), EdgeStyle::KnowledgeDeduction),
            SysEdge::new(EndRef::Whole(3), EndRef::Whole(1), EdgeStyle::KnowledgeDeduction),
        ],
        ..System::default()
    };
    assert_eq!(to_dot(&generate(&sys)), include_str!("fixtures/cluster_multi.dot"));
}

// -------------------------------------------------------------------------
// Round 5 — missing node kinds: the `#last` designated timepoint + bare
// timepoints (BEHAVIOR.md §3d/§6).
// -------------------------------------------------------------------------

/// Reproduce corpus `last_timepoint.dot` (24a119958f784d43): a compressed rule, two
/// darkblue actions, a gray `!KU`, and the `#last` timepoint ellipse fed by a
/// black-dashed before-edge.
#[test]
fn generate_reproduces_last_timepoint_byte_exact() {
    let sys = System {
        nodes: vec![
            GraphNode::Compressed { temporal: "j".into(), rule: "isend[K( s )]".into() },
            GraphNode::Action { fact: "Secret( 'KEY', A, s )".into(), temporal: "i".into() },
            GraphNode::Action { fact: "Reveal( K, X )".into(), temporal: "l".into() },
            GraphNode::Knowledge { term: "s".into(), temporal: "vk".into() },
            GraphNode::last(),
        ],
        edges: vec![
            SysEdge::new(EndRef::Whole(2), EndRef::Whole(4), EdgeStyle::TemporalBlack),
            SysEdge::new(EndRef::Whole(3), EndRef::Whole(0), EdgeStyle::KnowledgeDeduction),
        ],
        ..System::default()
    };
    assert_eq!(to_dot(&generate(&sys)), include_str!("fixtures/last_timepoint.dot"));
}

/// A bare timepoint variable renders as an uncolored `#var` ellipse (observed
/// `#i`, `#decrypt`, `#t1`, …).
#[test]
fn bare_timepoint_ellipse() {
    let sys = System {
        nodes: vec![GraphNode::Temporal { var: "decrypt".into() }],
        ..System::default()
    };
    assert!(to_dot(&generate(&sys)).contains("n0[label=\"#decrypt\",shape=\"ellipse\"];"));
}

// -------------------------------------------------------------------------
// Round 5 — pre-rendered-cell interop entry (RawRule).
// -------------------------------------------------------------------------

/// The pre-rendered path reproduces the Term-based path exactly: building the live
/// SAPIC cluster from PRE-RENDERED cell strings yields the same bytes.
#[test]
fn raw_rule_matches_term_path_byte_exact() {
    let record = RawRule::new("#t : eventNewKeyhk[NewKey( h, k )]", "#ffffff")
        .role("Process", "black")
        .cluster("Process_Session_1", "#36A5D84C")
        .premises(vec!["State_111111( h, k )".into()])
        .conclusions(vec!["State_1111111( h, k )".into()]);
    let sys = System { nodes: vec![GraphNode::RawRule(record)], ..System::default() };
    assert_eq!(to_dot(&generate(&sys)), include_str!("fixtures/cluster_process.dot"));
}

/// The wrap + escape pipeline applies to pre-rendered cell strings too: feeding the
/// E12 conclusion as a raw string reproduces the wrapped `wrap_E12.dot`.
#[test]
fn raw_rule_wraps_and_escapes_prerendered_cells() {
    let out: String = {
        let elems: Vec<String> = (1..=12).map(|k| format!("'a{:02}'", k)).collect();
        format!("Out( <{}> )", elems.join(", "))
    };
    let record = RawRule::new("#i : E12[E12( )]", "#d5d897")
        .premises(vec!["Fr( ~s )".into()])
        .conclusions(vec![out]);
    let sys = System { nodes: vec![GraphNode::RawRule(record)], ..System::default() };
    assert_eq!(to_dot(&generate(&sys)), include_str!("fixtures/wrap_E12.dot"));
}

/// Reproduce the conclusion-group FILL of the live `Wide` record byte-exact
/// (BEHAVIOR.md §3f, round 7). The conclusion group `[Ack 25, Big 68, Out 11]`
/// exercises the smallest-flat-first fill-budget allocation: `Ack` wraps and is
/// allocated 20 (breaking after `~n.4`), `Out` fits, and `Big` — placed last —
/// gets fill budget `87 − 20 − 11 = 56`, packing eight tuple elements on line 0
/// (not the seven a flat-sum budget of 51 would give). The record line is the one
/// captured live in `tests/fixtures/wide_group.dot`.
#[test]
fn wide_conclusion_group_fill_byte_exact() {
    let record = RawRule::new("#vr.3 : Wide[Made( ~n.4 )]", "#d5d897")
        .premises(vec![
            "In( <x1.4, x2.4, x3.4, x4.4, x5.4, x6.4, x7.4, x8.4, x9.4, x10.4> )".into(),
            "Fr( ~n.4 )".into(),
        ])
        .conclusions(vec![
            "Ack( ~n.4, <x1.4, x2.4> )".into(),
            "Big( <x1.4, x2.4, x3.4, x4.4, x5.4, x6.4, x7.4, x8.4, x9.4, x10.4> )".into(),
            "Out( x1.4 )".into(),
        ]);
    let sys = System { nodes: vec![GraphNode::RawRule(record)], ..System::default() };
    assert_eq!(to_dot(&generate(&sys)), include_str!("fixtures/wide_record.dot"));
}

/// End-to-end abbreviation: a system with a legend emits the `{ rank="sink"; … }`
/// block and the invis edge after it (observed order).
#[test]
fn legend_sink_block_and_invis_edges() {
    use graph_clean::abbrev::Abbreviator;
    let mut ab = Abbreviator::new();
    ab.add(Term::exp(Term::cst("g"), fresh("lkR.4"))); // EX1 = 'g'^~lkR.4
    let sys = System {
        nodes: vec![GraphNode::Knowledge { term: "EX1".into(), temporal: "vk".into() }],
        edges: Vec::new(),
        legend_html: Some(ab.legend_html()),
        legend_edges: vec![(EndRef::Whole(0), ())],
    };
    let dot = to_dot(&generate(&sys));
    assert!(dot.contains("{\nrank=\"sink\";\n"));
    assert!(dot.contains("EX1</FONT>"));
    // invis edge from the KU ellipse (n0) to the legend node, after the sink block.
    let sink = dot.find("rank=\"sink\"").unwrap();
    let invis = dot.find("[style=\"invis\"];").unwrap();
    assert!(invis > sink, "invis edge must follow the sink block");
}

/// The caller-supplied width interface (round 10): with no overrides the
/// result is byte-identical to the estimate path; a supplied occupancy widens
/// or narrows SIBLING budgets exactly as the display-text estimate would have;
/// a supplied fill numerator moves the wrapped cell's own share.
#[test]
fn supplied_cell_widths_override_estimates() {
    use graph_clean::generate::{group_widths, group_widths_with, CellWidths};
    // [Faa 45, Sib 40]: T = 85 <= 87, nothing wraps under estimates.
    let cells: Vec<String> = vec![
        "Faa( $aa, $ab, $ac, $ad, $ae, $af, $ag, $ah )".into(),
        format!("Sib( '{}' )", "a".repeat(33)),
    ];
    // No-override call sites are byte-identical (regression gate).
    assert_eq!(group_widths_with(&cells, &[]), group_widths(&cells));
    assert_eq!(group_widths_with(&cells, &[None, None]), group_widths(&cells));
    let flat0 = 45usize;
    assert!(group_widths(&cells)[0] >= flat0, "estimate path: Faa stays flat");
    // Supplying the sib's internal occupancy (say its UN-abbreviated width
    // renders at 60 columns) shrinks Faa's budget below its flat: it wraps.
    let ov = vec![None, Some(CellWidths::occupancy(60))];
    let w = group_widths_with(&cells, &ov);
    assert!(w[0] < flat0, "supplied sibling occupancy must trigger the wrap");
    // The wrapping cell's own fill share follows the supplied numerator too.
    let ov2 = vec![
        Some(CellWidths { fill_width: Some(80), ..Default::default() }),
        Some(CellWidths::occupancy(60)),
    ];
    let w2 = group_widths_with(&cells, &ov2);
    assert!(w2[0] > w[0], "a larger supplied fill numerator widens the share");
    // And a supplied bonus lifts the cell's own trigger budget.
    let ov3 = vec![
        Some(CellWidths { bonus: Some(30), ..Default::default() }),
        Some(CellWidths::occupancy(60)),
    ];
    let w3 = group_widths_with(&cells, &ov3);
    assert!(w3[0] >= flat0, "a supplied bonus can keep the cell flat");
}

/// The RawRule width plumbing: overrides reach the record's cells, and an
/// absent override vector reproduces the estimate path byte-exactly.
#[test]
fn raw_rule_supplied_widths_reach_cells() {
    use graph_clean::generate::CellWidths;
    let base = || {
        RawRule::new("#i : R[]", "#ffffff").conclusions(vec![
            "Faa( $aa, $ab, $ac, $ad, $ae, $af, $ag, $ah )".into(),
            format!("Sib( '{}' )", "a".repeat(33)),
        ])
    };
    let plain = System { nodes: vec![GraphNode::RawRule(base())], ..System::default() };
    let with_none = System {
        nodes: vec![GraphNode::RawRule(base().conclusion_widths(vec![None, None]))],
        ..System::default()
    };
    assert_eq!(to_dot(&generate(&plain)), to_dot(&generate(&with_none)));
    let with_occ = System {
        nodes: vec![GraphNode::RawRule(
            base().conclusion_widths(vec![None, Some(CellWidths::occupancy(60))]),
        )],
        ..System::default()
    };
    let dot_plain = to_dot(&generate(&plain));
    let dot_occ = to_dot(&generate(&with_occ));
    assert_ne!(dot_plain, dot_occ, "the supplied occupancy must change the layout");
    assert!(dot_occ.contains("Faa( $aa,"), "Faa wraps under the supplied occupancy");
    assert!(dot_occ.contains("\\l"), "wrapped cell present");
}
