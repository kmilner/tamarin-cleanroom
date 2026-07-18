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

// ---------------------------------------------------------------------------
// Round 11: fill rounding, relief, nested occupancies, tuple numerators,
// tuple-opener hang, trigger_width override (QUERIES.log Session 11).

/// Build a plain `name( LONG, $aa, … )` argfact of an exact display width,
/// mirroring the round-8..11 probe generators.
fn probe_argfact(name: &str, flat: usize) -> String {
    let k = (flat - name.len() - 8) / 5;
    let l1 = flat - name.len() - 4 - 5 * k;
    let long = format!("$q{}", "a".repeat(l1 - 2));
    let mut elems = vec![long];
    let a = "abcdefghijklmnopqrstuvwxyz".as_bytes();
    for i in 0..k {
        elems.push(format!("${}{}", a[i / 26] as char, a[i % 26] as char));
    }
    let s = format!("{}( {} )", name, elems.join(", "));
    assert_eq!(s.chars().count(), flat);
    s
}

/// Probe GB_50_50 (round 11): equal both-wrap pairs allocate 43/43 — the
/// proportional share 43.5 rounds half-DOWN (half-up 44 is out of band).
#[test]
fn equal_pair_fill_rounds_half_down() {
    let cells = vec![probe_argfact("Naa", 50), probe_argfact("Wbb", 50)];
    assert_eq!(graph_clean::generate::group_widths(&cells), vec![43, 43]);
}

/// Probe IA_65_24 (round 11): beside a wrapping 65-wide sibling a 24-flat
/// target wraps (the model's relief pass charges the peel-only-broken 65 at
/// its full occupancy). The live IA_65_23 boundary — a 23-flat target saved
/// beside the same sibling — sits one column inside the documented ±1
/// coupled-`fits` residue and is intentionally NOT asserted here.
#[test]
fn relief_target_beside_wrapping_sibling() {
    let w24 = graph_clean::generate::group_widths(&vec![
        probe_argfact("Waa", 65),
        probe_argfact("Tbb", 24),
    ]);
    assert!(w24[0] < 65, "the 65-wide cell wraps");
    assert!(w24[1] < 24, "a 24-flat target wraps beside the same sibling");
}

/// Probe K1_37 / K1_38 (round 11): nested tuple-in-tuple occupancy is
/// `elems − 1` per nested node — a pair-of-pairs of flat `s` occupies
/// `s + 3 + 1 + 1`, so the 45-flat partner flips exactly between 37 and 38.
#[test]
fn nested_tuple_occupancy_flips_partner_at_38() {
    let pp = |flat: usize| {
        let l1 = flat - 26; // N( <<LONG, $aa>, <$ab, $ac>> )
        let s = format!("N( <<$q{}, $aa>, <$ab, $ac>> )", "a".repeat(l1 - 2));
        assert_eq!(s.chars().count(), flat);
        s
    };
    let w37 = graph_clean::generate::group_widths(&vec![probe_argfact("Faa", 45), pp(37)]);
    assert!(w37[0] >= 45, "beside a 37-wide pair-of-pairs the 45 partner stays flat");
    let w38 = graph_clean::generate::group_widths(&vec![probe_argfact("Faa", 45), pp(38)]);
    assert!(w38[0] < 45, "beside a 38-wide pair-of-pairs it wraps");
}

/// Probes TB4_47/TB4_48 (round 11, re-read round 12): a lone-4-tuple fact at
/// 47 stays flat beside a wrapping 45-argfact (relief with the bump-free
/// charge for ≥ 4-element receivers), 48 wraps; WIT_78 (round 11): the
/// mid-list 4-tuple fact fits at 78 beside `Fr( ~ni )`. (The WIT_79 wrap is
/// NOT asserted: it sits in the documented ΣC = 88 coupled-`fits` zone — the
/// round-12 LD4_68 probe observed the byte-identical shape class staying
/// FLAT at budget+1 beside a flat-20 sibling.)
#[test]
fn bonus_gated_on_last_tuple_arg() {
    let tup4 = |flat: usize| {
        let l1 = flat - 23;
        let s = format!("Tf( <$q{}, $aa, $ab, $ac> )", "a".repeat(l1 - 2));
        assert_eq!(s.chars().count(), flat);
        s
    };
    let w47 = graph_clean::generate::group_widths(&vec![tup4(47), probe_argfact("Fbb", 45)]);
    assert!(w47[0] >= 47, "a 47-flat lone-tuple fact stays flat (relief)");
    let w48 = graph_clean::generate::group_widths(&vec![tup4(48), probe_argfact("Fbb", 45)]);
    assert!(w48[0] < 48, "a 48-flat lone-tuple fact wraps");
    let wit = |flat: usize| {
        let core = "<'commit', ff($cf), ff($cg), $ch>";
        let l1 = flat - (6 + 5 + 2 + core.chars().count() + 2 + 7 + 2);
        let s = format!("St_I( ~id, $q{}, {}, w1($zz) )", "a".repeat(l1 - 2), core);
        assert_eq!(s.chars().count(), flat);
        s
    };
    let w78 = graph_clean::generate::group_widths(&vec![wit(78), "Fr( ~ni )".to_string()]);
    assert!(w78[0] >= 78, "mid-list 4-tuple slack covers 78 beside Fr( ~ni )");
}

/// Round-12 battery L (beside a floor-protected flat-20 sibling, bonus-free
/// budget 67): the pass-1 slack is `⌈elems/2⌉ − 1` for a tuple/union arg in
/// ANY position — pair 0 (LA2_68 wraps at budget+1), 3-tuple 1 (LA3_68 flat,
/// LA3_69 wraps), mid-list 4-tuple 1 (LD4_68 FLAT at budget+1, LD4_69
/// wraps), single-arg 3-tuple 1 (LC3_69 wraps at budget+2 — refuting the
/// old `⌊elems/2⌋ + 2` single-arg bonus), 6-tuple 2 (LA6_70 wraps at
/// budget+3), 3-union 1 (LE3_68 flat, LE3_69 wraps).
#[test]
fn any_arg_tuple_slack_battery_l() {
    let sib = || "Fzz( $q00aaaa, $aa )".to_string(); // flat 20, never wraps
    assert_eq!(sib().chars().count(), 20);
    let mtup = |flat: usize, tup: &str| {
        let fixed = 3 + 4 + 5 + 2 + tup.chars().count();
        let s = format!("Mtt( $q{}, $be, {} )", "a".repeat(flat - fixed - 2), tup);
        assert_eq!(s.chars().count(), flat);
        s
    };
    // pair last arg: slack 0 — 68 wraps at budget+1
    let w = graph_clean::generate::group_widths(&vec![sib(), mtup(68, "<$bo, $bp>")]);
    assert!(w[1] < 68, "LA2_68: a last-pair fact wraps at budget+1");
    // 3-tuple last arg: slack 1 — 68 flat, 69 wraps
    let w = graph_clean::generate::group_widths(&vec![sib(), mtup(68, "<$bo, $bp, $bq>")]);
    assert!(w[1] >= 68, "LA3_68: a last-3-tuple fact fits at budget+1");
    let w = graph_clean::generate::group_widths(&vec![sib(), mtup(69, "<$bo, $bp, $bq>")]);
    assert!(w[1] < 69, "LA3_69: it wraps at budget+2");
    // mid-list 4-tuple: slack 1 (any-position)
    let mid = |flat: usize| {
        let tup = "<$bo, $bp, $bq, $br>";
        let fixed = 3 + 4 + 2 + tup.chars().count() + 5;
        let s = format!("Dtt( $q{}, {}, $be )", "a".repeat(flat - fixed - 2), tup);
        assert_eq!(s.chars().count(), flat);
        s
    };
    let w = graph_clean::generate::group_widths(&vec![sib(), mid(68)]);
    assert!(w[1] >= 68, "LD4_68: a mid-list 4-tuple fact fits at budget+1");
    let w = graph_clean::generate::group_widths(&vec![sib(), mid(69)]);
    assert!(w[1] < 69, "LD4_69: it wraps at budget+2");
    // 6-tuple last arg: slack 2 — 70 wraps at budget+3
    let w = graph_clean::generate::group_widths(&vec![
        sib(),
        mtup(70, "<$bo, $bp, $bq, $br, $bs, $bt>"),
    ]);
    assert!(w[1] < 70, "LA6_70: a last-6-tuple fact wraps at budget+3");
}

/// Round-12 battery M (MA5/MA6): the relief charge for a wrapping sibling
/// follows the sibling's UNROUNDED fill quotient with a +1/3 bump —
/// `hd(q + 1/3)` — for receivers without a ≥ 4-element tuple arg. Beside a
/// 54-wide sibling (q = 49.45 → charge 50) a 37-flat mid-pair target is
/// saved and a 38-flat one wraps; beside 59 (q = 54.03 → charge 54) a
/// 33-flat one is saved and 34 wraps.
#[test]
fn relief_charge_bump_battery_m() {
    let midpair = |flat: usize, tag: &str| {
        let fixed = 2 + 4 + 2 + 8 + 4;
        let s = format!("Tc( $q{}{}, <$a, $b>, $c )", tag, "a".repeat(flat - fixed - 2 - tag.len()));
        assert_eq!(s.chars().count(), flat);
        s
    };
    let w = graph_clean::generate::group_widths(&vec![probe_argfact("Wbb", 54), midpair(37, "z")]);
    assert!(w[1] >= 37, "MA5_54_37: saved at the hd(q+1/3) boundary");
    let w = graph_clean::generate::group_widths(&vec![probe_argfact("Wbb", 54), midpair(38, "z")]);
    assert!(w[1] < 38, "MA5_54_38: one past the boundary wraps");
    let w = graph_clean::generate::group_widths(&vec![probe_argfact("Wbb", 59), midpair(33, "z")]);
    assert!(w[1] >= 33, "MA6_59_33: saved");
    let w = graph_clean::generate::group_widths(&vec![probe_argfact("Wbb", 59), midpair(34, "z")]);
    assert!(w[1] < 34, "MA6_59_34: wraps");
}

/// Probe K3_40_6_60 (round 11): a 6-tuple receiver's fill numerator carries
/// the tuple surcharge (`flat + 7`), giving ribbon 38 beside a 60-argfact.
#[test]
fn tuple_receiver_fill_numerator() {
    let recv = "Trr( <$q31aa, $aa, $ab, $ac, $ad, $ae> )".to_string();
    assert_eq!(recv.chars().count(), 40);
    let w = graph_clean::generate::group_widths(&vec![recv, probe_argfact("Sbb", 60)]);
    assert_eq!(w[0], 38);
}

/// Probes K4_tuple2 / K4_tupfunc (round 11): the TUPLE opener hangs — when
/// the first element does not fit beside the `<`, the `<` stays at the end of
/// the line and the elements start on the next line at the fill column (also
/// inside a function argument). Byte-exact against the captures.
#[test]
fn tuple_opener_hang_byte_fixtures() {
    use graph_clean::doclayout::wrap_cell_dot;
    let long95 = format!("$q93{}", "a".repeat(91));
    let cell = format!("Tzz( <{}, $hh> )", long95);
    let expect = format!(
        "Tzz( \\<\\l{n}{}, \\l{n}$hh\\>\\l)\\l",
        long95,
        n = "&nbsp;".repeat(6)
    );
    assert_eq!(wrap_cell_dot(&cell, 87), expect);
    let long84 = format!("$q90{}", "a".repeat(80));
    let cell2 = format!("Qzz( w1(<{}, $ha, $hb, $hc>) )", long84);
    let expect2 = format!(
        "Qzz( w1(\\<\\l{n}{}, \\l{n}$ha, $hb, $hc\\>)\\l)\\l",
        long84,
        n = "&nbsp;".repeat(9)
    );
    assert_eq!(wrap_cell_dot(&cell2, 87), expect2);
}

/// The round-11 `trigger_width` override: a caller-supplied self width enters
/// the wrap-trigger comparison (a cell whose display fits can be made to
/// wrap), and an absent override stays byte-identical.
#[test]
fn supplied_trigger_width_overrides_self_width() {
    use graph_clean::generate::{group_widths, group_widths_with, CellWidths};
    let cells = vec![probe_argfact("Faa", 45), probe_argfact("Sbb", 40)];
    assert_eq!(group_widths_with(&cells, &[None, None]), group_widths(&cells));
    assert!(group_widths(&cells)[0] >= 45, "estimate path: row total 85 fits");
    let ov = vec![Some(CellWidths { trigger_width: Some(95), ..Default::default() }), None];
    let w = group_widths_with(&cells, &ov);
    assert!(w[0] < 45, "a supplied 95-column self width makes the cell wrap");
}
