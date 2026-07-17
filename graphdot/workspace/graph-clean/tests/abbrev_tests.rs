//! Abbreviation + clustering tests, grounded in observed payloads
//! (BEHAVIOR.md §4, §5).

use graph_clean::abbrev::{
    legend_html, prefix_for_symbol, select, select_with, Abbreviator, TABLE_OPEN,
};
use graph_clean::model::*;
use graph_clean::term::Term;

/// Render the terms `select` picks, as `render_full` strings, for assertions.
fn picked(roots: &[Term]) -> Vec<String> {
    select(roots).iter().map(Term::render_full).collect()
}

/// Extract the inner HTML of the first `plain` legend node (bytes between
/// `label=<` and `>];`).
fn legend_inner(dot: &str) -> String {
    let anchor = dot.find("shape=\"plain\",label=<").unwrap();
    let start = anchor + dot[anchor..].find("label=<").unwrap() + "label=<".len();
    let end = start + dot[start..].find(">];").unwrap();
    dot[start..end].to_string()
}

#[test]
fn table_open_tag_is_65_bytes() {
    // The legend hang-indent equals this tag's byte length (BEHAVIOR.md §5a).
    assert_eq!(TABLE_OPEN.len(), 65);
}

#[test]
fn prefix_rule_matches_observed_names() {
    // (root symbol name, observed prefix) pairs taken from the corpus legend.
    let cases = [
        ("sign", "SI"),
        ("senc", "SE"),
        ("second", "SE"),
        ("hash", "HA"),
        ("h", "H"),
        ("h2", "H"),
        ("KDF", "KD"),
        ("kdf1", "KD"),
        ("pk", "PK"),
        ("aenc", "AE"),
        ("aead", "AE"),
        ("adec", "AD"),
        ("pmult", "PM"),
        ("plus", "PL"),
        ("pdec", "PD"),
        ("inv", "IN"),
        ("mac", "MA"),
        ("blind", "BL"),
        ("idsign", "ID"),
        // operator function names
        ("exp", "EX"),
        ("mult", "MU"),
        ("union", "UN"),
        ("xor", "XO"),
        // constants (unquoted names), non-letters skipped
        ("uninitialized", "UN"),
        ("F_status", "FS"),
        ("ho_req_ack", "HO"),
        ("no_message_state", "NO"),
        ("K_gNB_star", "KG"),
        // variables (undecorated names)
        ("AMF_UE_NGAP_ID", "AM"),
        ("cid_N26", "CI"),
        ("gNB_UE_ID", "GN"),
        ("commitmsg", "CO"),
        ("StateChannel", "ST"),
    ];
    for (name, expect) in cases {
        assert_eq!(prefix_for_symbol(name), expect, "prefix for {name:?}");
    }
}

#[test]
fn operator_terms_report_function_name_for_prefix() {
    assert_eq!(Term::mult(vec![Term::msg("a")]).root_symbol_name(), "mult");
    assert_eq!(Term::union(vec![Term::msg("a")]).root_symbol_name(), "union");
    assert_eq!(Term::xor(vec![Term::msg("a")]).root_symbol_name(), "xor");
    assert_eq!(Term::exp(Term::cst("g"), Term::msg("a")).root_symbol_name(), "exp");
    assert_eq!(prefix_for_symbol(&Term::mult(vec![]).root_symbol_name()), "MU");
}

#[test]
fn term_renders_observed_surface_syntax() {
    // H2 = h(<$I.1, $R.1, X, 'g'^~ekR, z, 'g'^(~lkR.1*~x.2)>)   (from multi_abbrev)
    let h2 = Term::app(
        "h",
        vec![Term::tuple(vec![
            Term::pubv("I.1"),
            Term::pubv("R.1"),
            Term::msg("X"),
            Term::exp(Term::cst("g"), Term::fresh("ekR")),
            Term::msg("z"),
            Term::exp(Term::cst("g"), Term::mult(vec![Term::fresh("lkR.1"), Term::fresh("x.2")])),
        ])],
    );
    assert_eq!(
        h2.render_full(),
        "h(<$I.1, $R.1, X, 'g'^~ekR, z, 'g'^(~lkR.1*~x.2)>)"
    );
    // (~ekR*x*~ekR.1)
    assert_eq!(
        Term::mult(vec![Term::fresh("ekR"), Term::msg("x"), Term::fresh("ekR.1")]).render_full(),
        "(~ekR*x*~ekR.1)"
    );
    // ('1'++'1'++z)
    assert_eq!(
        Term::union(vec![Term::cst("1"), Term::cst("1"), Term::msg("z")]).render_full(),
        "('1'++'1'++z)"
    );
}

#[test]
fn abbreviator_numbers_and_nests() {
    // Register MU1 first, then EX1 = 'g'^MU1 must reference it (nesting, §5b).
    let mut ab = Abbreviator::new();
    let mu = Term::mult(vec![Term::fresh("ekR"), Term::msg("x"), Term::fresh("ekR.1")]);
    let ex = Term::exp(Term::cst("g"), mu.clone());
    assert_eq!(ab.add(mu), "MU1");
    assert_eq!(ab.add(ex), "EX1");
    // second, structurally-identical mult dedups to MU1 (no MU2).
    let mu2 = Term::mult(vec![Term::fresh("ekR"), Term::msg("x"), Term::fresh("ekR.1")]);
    assert_eq!(ab.add(mu2), "MU1");
    let rows = ab.rows();
    assert_eq!(rows, vec![
        ("MU1".to_string(), "(~ekR*x*~ekR.1)".to_string()),
        ("EX1".to_string(), "'g'^MU1".to_string()),
    ]);
}

#[test]
fn per_prefix_counter_increments() {
    let mut ab = Abbreviator::new();
    assert_eq!(ab.add(Term::app("sign", vec![Term::fresh("a")])), "SI1");
    assert_eq!(ab.add(Term::app("sign", vec![Term::fresh("b")])), "SI2");
    assert_eq!(ab.add(Term::app("senc", vec![Term::fresh("c")])), "SE1");
    assert_eq!(ab.add(Term::app("sign", vec![Term::fresh("d")])), "SI3");
}

#[test]
fn legend_html_single_row_matches_fixture() {
    let dot = include_str!("fixtures/simple_abbrev.dot");
    let expected = legend_inner(dot);
    // SI1 = sign(<'2', $I, $R, hki, 'g'^~ekR>, ~ltkI)   (raw; legend escapes < >)
    let rows = vec![(
        "SI1".to_string(),
        "sign(<'2', $I, $R, hki, 'g'^~ekR>, ~ltkI)".to_string(),
    )];
    assert_eq!(legend_html(&rows), expected);
}

#[test]
fn legend_html_multi_row_matches_fixture() {
    let dot = include_str!("fixtures/multi_abbrev.dot");
    let expected = legend_inner(dot);
    // Rows in the fixture's order (order itself is a documented gap, §5b);
    // this asserts the RENDERING is byte-exact: escaping + 65-space hang indent.
    let rows: Vec<(String, String)> = [
        ("EX2", "'g'^(~ekR*x)"),
        ("EX3", "'g'^~ekR.1"),
        ("EX4", "'g'^~lkR.1"),
        ("H2", "h(<$I.1, $R.1, X, 'g'^~ekR, z, 'g'^(~lkR.1*~x.2)>)"),
        ("MU1", "(~ekR*x*~ekR.1)"),
        ("EX1", "'g'^MU1"),
        ("H1", "h(<$I, $R, EX2, EX3, EX1, 'g'^(~lkR*~x.1)>)"),
        ("MU2", "(x*~ekR.1)"),
    ]
    .iter()
    .map(|(a, b)| (a.to_string(), b.to_string()))
    .collect();
    assert_eq!(legend_html(&rows), expected);
}

#[test]
fn legend_via_terms_and_abbreviator_matches_fixture() {
    // Reproduce the simple_abbrev legend end-to-end from a Term model + the
    // Abbreviator (naming + rendering), not from a hand-written row list.
    let dot = include_str!("fixtures/simple_abbrev.dot");
    let expected = legend_inner(dot);
    let si = Term::app(
        "sign",
        vec![
            Term::tuple(vec![
                Term::cst("2"),
                Term::pubv("I"),
                Term::pubv("R"),
                Term::msg("hki"),
                Term::exp(Term::cst("g"), Term::fresh("ekR")),
            ]),
            Term::fresh("ltkI"),
        ],
    );
    let mut ab = Abbreviator::new();
    assert_eq!(ab.add(si), "SI1");
    assert_eq!(ab.legend_html(), expected);
}

#[test]
fn cluster_trigger_follows_role() {
    // role="Undefined" everywhere -> Simple.
    let mut g = Graph::new(Header::Compact);
    g.push(Stmt::Node(Node::record(
        "n5",
        Record {
            columns: vec![vec![Cell::new("n0", "Fr( ~x )")]],
            fillcolor: "#d5d897".into(),
            fontcolor: "black".into(),
            role: Role::undefined(),
        },
    )));
    assert_eq!(g.infer_header(), Header::Simple);

    // a defined role anywhere -> Compact.
    g.push(Stmt::Cluster(Cluster {
        label: "Initiator_Session_1".into(),
        color: "#4936D84C".into(),
        body: vec![Stmt::Node(Node::record(
            "n7",
            Record {
                columns: vec![vec![Cell::new("n4", "State( ~k )")]],
                fillcolor: "#80404f".into(),
                fontcolor: "white".into(),
                role: Role("Initiator".into()),
            },
        ))],
    }));
    assert_eq!(g.infer_header(), Header::Compact);
}

// ---------------------------------------------------------------------------
// Selection rule (BEHAVIOR.md §5c). Each test mirrors a controlled black-box
// probe of the server (see QUERIES.log): a crafted rule whose one graph node
// carries specific terms, so which term is abbreviated is directly observed.
// ---------------------------------------------------------------------------

#[test]
fn select_length_boundary_is_ten() {
    // Probe: Keep(~x, '12345678', '12345678', '1234567', '1234567')
    //   => '12345678' (len 10, x2) abbreviated; '1234567' (len 9, x2) not.
    let c10 = Term::cst("12345678"); // renders '12345678' == 10 chars
    let c9 = Term::cst("1234567"); //  renders '1234567'  ==  9 chars
    assert_eq!(c10.render_len(), 10);
    assert_eq!(c9.render_len(), 9);
    let roots = vec![c10.clone(), c10.clone(), c9.clone(), c9.clone()];
    let got = picked(&roots);
    assert_eq!(got, vec!["'12345678'".to_string()]); // only the 10-char one
}

#[test]
fn select_requires_two_occurrences_even_for_long_terms() {
    // Probe: 'aaaa…aaaa' (42 chars, x1) NOT abbreviated; 'shared1234' (12, x2) is.
    let long_once = Term::cst(&"a".repeat(40)); // renders with quotes == 42 chars
    let shared = Term::cst("shared1234");
    assert!(long_once.render_len() > 10);
    let roots = vec![long_once, shared.clone(), shared.clone()];
    assert_eq!(picked(&roots), vec!["'shared1234'".to_string()]);
}

#[test]
fn select_never_abbreviates_a_tuple() {
    // Probe: Keep(~x, <'aa','bb','cc'>, <'aa','bb','cc'>, h('longarg123'), h('longarg123'))
    //   => the length-18 tuple (x2) is NOT abbreviated; h('longarg123') and its
    //   inner 'longarg123' ARE (bottom-up nesting).
    let tup = Term::tuple(vec![Term::cst("aa"), Term::cst("bb"), Term::cst("cc")]);
    assert!(tup.render_len() >= 10 && tup.is_tuple());
    let h = Term::app("h", vec![Term::cst("longarg123")]);
    let roots = vec![tup.clone(), tup.clone(), h.clone(), h.clone()];
    let got = picked(&roots);
    // tuple excluded; both h(...) and the nested 'longarg123' selected (bottom-up
    // => shorter 'longarg123' before h('longarg123')).
    assert_eq!(
        got,
        vec!["'longarg123'".to_string(), "h('longarg123')".to_string()]
    );
    assert!(!got.iter().any(|s| s.starts_with('<')), "no tuple abbreviated");
}

#[test]
fn select_then_name_reproduces_probe_legend() {
    // End-to-end: feed the tuple probe's terms through select + Abbreviator and
    // reproduce the observed legend rows  LO1='longarg123' ; H1='h(LO1)'.
    let tup = Term::tuple(vec![Term::cst("aa"), Term::cst("bb"), Term::cst("cc")]);
    let h = Term::app("h", vec![Term::cst("longarg123")]);
    let roots = vec![tup.clone(), tup, h.clone(), h];
    let mut ab = Abbreviator::new();
    for t in select(&roots) {
        ab.add(t);
    }
    assert_eq!(
        ab.rows(),
        vec![
            ("LO1".to_string(), "'longarg123'".to_string()),
            ("H1".to_string(), "h(LO1)".to_string()),
        ]
    );
}

#[test]
fn select_counts_nested_occurrences() {
    // A long atom that appears only *inside* two copies of an outer term still
    // reaches occurrence 2 and is abbreviated (nested counting).
    let inner = Term::cst("longarg123"); // len 12
    let outer = Term::app("h", vec![inner.clone()]);
    let roots = vec![outer.clone(), outer]; // inner occurs twice, only nested
    let got = picked(&roots);
    assert!(got.contains(&"'longarg123'".to_string()));
    assert!(got.contains(&"h('longarg123')".to_string()));
}

#[test]
fn select_with_custom_thresholds() {
    // The thresholds are parameterisable for probing.
    let t = Term::cst("abcdef"); // len 8
    let roots = vec![t.clone(), t.clone()];
    assert!(select(&roots).is_empty()); // 8 < 10 default
    assert_eq!(select_with(&roots, 8, 2).len(), 1); // lowered length gate
    assert!(select_with(&roots, 8, 3).is_empty()); // raised occ gate
}
