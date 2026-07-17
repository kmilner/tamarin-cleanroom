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
