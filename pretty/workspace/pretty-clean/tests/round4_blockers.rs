//! Round-4 probe-pinned unit tests for the three corpus-scale rule-rendering
//! blockers surfaced by the open-side full-corpus measurement.
//!
//!  * Blocker 1 — the SAPIC `process="…"` rule attribute: rendered
//!    DOUBLE-quoted, between `color` and `no_derivcheck`, value verbatim
//!    (probe:p_process, target:ct).
//!  * Blocker 2 — a multi-line last element makes the enclosing delimiter drop
//!    or not depending on the operator: a MULTISET-UNION `)` drops to its own
//!    line at the `(` column (like the round-3 AC fix), while an APPLICATION
//!    `)` stays JOINED as `>)` (probe:uniondrop / probe:appdrop).
//!  * Blocker 3 — a huge `variants (modulo AC)` block (C8/BP_IBS reach ~10 000
//!    lines) renders without stack overflow on a production-sized stack: the
//!    Doc build (`reduce_*`), render (`lay`) and DROP are all iterative.
//!
//! The byte-exact expectations for blockers 1–2 are sliced straight from the
//! probe captures (round2/targets/*.hs.txt); the fixtures below reconstruct the
//! AST the renderer must turn back into those bytes. The captures self-skip
//! once the crate moves at integration (the corpus gate takes over).

use pretty_clean::ast::*;
use pretty_clean::render_rule;

// ── fixture helpers ─────────────────────────────────────────────────────────

fn var(name: &str, idx: u64, sort: SortHint) -> Term {
    Term::Var(VarSpec { name: name.into(), idx, sort, typ: None })
}
fn msg(name: &str) -> Term {
    var(name, 0, SortHint::Untagged)
}
fn pubc(s: &str) -> Term {
    Term::PubLit(s.into())
}
fn app(f: &str, args: Vec<Term>) -> Term {
    Term::App(f.into(), args)
}
fn pair(elems: Vec<Term>) -> Term {
    Term::Pair(elems)
}
fn union(a: Term, b: Term) -> Term {
    Term::BinOp(BinOp::Union, Box::new(a), Box::new(b))
}
fn fact(name: &str, args: Vec<Term>) -> Fact {
    Fact { persistent: false, name: name.into(), args, annotations: vec![] }
}
fn erule(name: &str, prems: Vec<Fact>, acts: Vec<Fact>, concs: Vec<Fact>) -> Rule {
    Rule {
        name: name.into(),
        modulo: Some("E".into()),
        attributes: vec![],
        premises: prems,
        actions: acts,
        conclusions: concs,
        loop_breakers: vec![],
    }
}

/// The first rule block of a one-rule probe capture: `rule …` through the
/// trailing variants comment. `None` when the capture dir is gone.
fn probe_block(basename: &str) -> Option<String> {
    let cap = std::fs::read_to_string(format!("../../round2/targets/{basename}")).ok()?;
    let lines: Vec<&str> = cap.lines().collect();
    let start = lines.iter().position(|l| l.starts_with("rule "))?;
    let end = start
        + lines[start..]
            .iter()
            .position(|l| {
                let t = l.trim();
                t == "/* has exactly the trivial AC variant */" || t == "*/"
            })
            .expect("no variants comment end");
    Some(lines[start..=end].join("\n"))
}

// `<aa, bb, cc, dd, ee, ff>` and the shared multi-line `macf(…)` sub-term used
// by both blocker-2 probes: `macf` wraps because its last argument (the tuple
// `<'lbl', aa, bb, cccccccc, dd>`) sits on the second line, so `macf`'s own
// `)` attaches to that tuple (`…dd>)`), making the ENCLOSING tuple multi-line.
fn six_tuple() -> Term {
    pair(vec![msg("aa"), msg("bb"), msg("cc"), msg("dd"), msg("ee"), msg("ff")])
}
fn macf_term() -> Term {
    app(
        "macf",
        vec![
            app("firstf", vec![app("hashf", vec![six_tuple()])]),
            pair(vec![pubc("lbl"), msg("aa"), msg("bb"), msg("cccccccc"), msg("dd")]),
        ],
    )
}

// ── Blocker 1: the SAPIC process attribute ──────────────────────────────────

#[test]
fn blocker1_process_attribute_render() {
    // probe:p_process / target:ct — `process="in(x.1);"` renders DOUBLE-quoted
    // (unlike `role='…'`), positioned right after `color`, its snippet verbatim.
    // Source attribute order is scrambled to also pin the canonicalisation
    // (color < process < issapicrule < role).
    let mut r = erule("Init", vec![], vec![fact("Init", vec![])], vec![fact("State_", vec![])]);
    r.attributes = vec![
        RuleAttr::Role("Process".into()),
        RuleAttr::IsSapicRule,
        RuleAttr::Process("in(x.1);".into()),
        RuleAttr::Color("ffffff".into()),
    ];
    let expected = [
        "rule (modulo E) Init[color=#ffffff, process=\"in(x.1);\", issapicrule,",
        "                     role='Process']:",
        "   [ ] --[ Init( ) ]-> [ State_( ) ]",
        "",
        "  /* has exactly the trivial AC variant */",
    ]
    .join("\n");
    assert_eq!(render_rule(&r, None), expected);
    if let Some(cap) = probe_block("probe_process.hs.txt") {
        assert_eq!(render_rule(&r, None), cap, "vs probe_process capture");
    }
}

#[test]
fn blocker1_process_value_is_verbatim() {
    // probe:p_process — the process snippet carries spaces, single-quoted
    // constants, commas and `<>` VERBATIM inside the double quotes (no escaping
    // and none observable), and stays one unbreakable text token.
    let mut r = erule(
        "outxlbl_0_11",
        vec![fact("State_11", vec![var("x", 1, SortHint::Untagged)])],
        vec![],
        vec![
            fact("State_111", vec![var("x", 1, SortHint::Untagged)]),
            fact("Out", vec![pair(vec![var("x", 1, SortHint::Untagged), pubc("lbl")])]),
        ],
    );
    r.attributes = vec![
        RuleAttr::Color("ffffff".into()),
        RuleAttr::Process("out(<x.1, 'lbl'>);".into()),
        RuleAttr::IsSapicRule,
        RuleAttr::Role("Process".into()),
    ];
    let header = render_rule(&r, None);
    let first_two = header.lines().take(2).collect::<Vec<_>>().join("\n");
    assert_eq!(
        first_two,
        "rule (modulo E) outxlbl_0_11[color=#ffffff, process=\"out(<x.1, 'lbl'>);\",\n                             issapicrule, role='Process']:"
    );
}

#[test]
fn blocker1_canonical_attribute_order_full() {
    // Canonical order color < process < no_derivcheck < issapicrule < role,
    // pinned across probe:p_process (color/process/issapicrule/role) and
    // target:running-example (process/no_derivcheck/issapicrule). Source order
    // scrambled; the rendered attribute list (whitespace-collapsed) is
    // canonical.
    let mut r = erule("R", vec![fact("In", vec![msg("x")])], vec![], vec![fact("Out", vec![msg("x")])]);
    r.attributes = vec![
        RuleAttr::Role("r".into()),
        RuleAttr::NoDerivCheck,
        RuleAttr::IsSapicRule,
        RuleAttr::Process("p();".into()),
        RuleAttr::Color("abc123".into()),
    ];
    let rendered = render_rule(&r, None);
    let attrs = &rendered[rendered.find('[').unwrap()..rendered.find("]:").unwrap() + 1];
    let collapsed = attrs.split_whitespace().collect::<Vec<_>>().join(" ");
    assert_eq!(
        collapsed,
        "[color=#abc123, process=\"p();\", no_derivcheck, issapicrule, role='r']"
    );
}

// ── Blocker 2: union `)` drops, application `)` stays joined ─────────────────

#[test]
fn blocker2_union_paren_drops_below_tuple() {
    // probe:uniondrop — the multiset-union `)` drops to its own line at the
    // `(` column when the last union element (a tuple) is multi-line; the
    // tuple's `>` drops too, so `>` and `)` sit on SEPARATE lines.
    let last = pair(vec![pubc("2"), msg("dd"), macf_term()]);
    let u = union(pair(vec![pubc("1"), msg("shortval111")]), last);
    let r = erule(
        "PU",
        vec![fact("In", vec![six_tuple()])],
        vec![],
        vec![fact("FactUUUUUUUUUUUUUU", vec![u])],
    );
    let expected = [
        "rule (modulo E) PU:",
        "   [ In( <aa, bb, cc, dd, ee, ff> ) ]",
        "  -->",
        "   [",
        "   FactUUUUUUUUUUUUUU( (<'1', shortval111>++",
        "                        <'2', dd, ",
        "                         macf(firstf(hashf(<aa, bb, cc, dd, ee, ff>)),",
        "                              <'lbl', aa, bb, cccccccc, dd>)",
        "                        >",
        "                       )",
        "   )",
        "   ]",
        "",
        "  /* has exactly the trivial AC variant */",
    ]
    .join("\n");
    assert_eq!(render_rule(&r, None), expected);
    if let Some(cap) = probe_block("probe_uniondrop.hs.txt") {
        assert_eq!(render_rule(&r, None), cap, "vs probe_uniondrop capture");
    }
}

#[test]
fn blocker2_application_paren_stays_joined() {
    // probe:appdrop — the SAME multi-line tuple as the last argument of an
    // APPLICATION keeps the application's `)` JOINED to the tuple's `>` (`>)`),
    // unlike the union case. Confirms `app_doc` must NOT adopt the drop.
    let last = pair(vec![pubc("2"), msg("dd"), macf_term()]);
    let call = app("someop", vec![pair(vec![pubc("1"), msg("longvariablename111")]), last]);
    let r = erule(
        "Probe",
        vec![fact("In", vec![six_tuple()])],
        vec![],
        vec![fact("MyFactLongName", vec![call])],
    );
    let expected = [
        "rule (modulo E) Probe:",
        "   [ In( <aa, bb, cc, dd, ee, ff> ) ]",
        "  -->",
        "   [",
        "   MyFactLongName( someop(<'1', longvariablename111>,",
        "                          <'2', dd, ",
        "                           macf(firstf(hashf(<aa, bb, cc, dd, ee, ff>)),",
        "                                <'lbl', aa, bb, cccccccc, dd>)",
        "                          >)",
        "   )",
        "   ]",
        "",
        "  /* has exactly the trivial AC variant */",
    ]
    .join("\n");
    assert_eq!(render_rule(&r, None), expected);
    if let Some(cap) = probe_block("probe_appdrop.hs.txt") {
        assert_eq!(render_rule(&r, None), cap, "vs probe_appdrop capture");
    }
}

// ── Blocker 3: no stack overflow on huge variant blocks ─────────────────────

#[test]
fn blocker3_huge_variants_render_on_small_stack() {
    // A ~10 000-line `variants (modulo AC)` block (bigger than C8's) must
    // render on a SMALL 2 MB stack — far under the production 8 MB — proving
    // the iterative Doc build / render / drop, not a wide-stack workaround.
    std::thread::Builder::new()
        .stack_size(2 * 1024 * 1024)
        .spawn(|| {
            let base = erule(
                "Big",
                vec![fact("Fr", vec![var("a", 0, SortHint::Fresh)])],
                vec![],
                vec![fact("Out", vec![var("a", 0, SortHint::Fresh)])],
            );
            let ac = Rule { modulo: Some("AC".into()), ..base.clone() };
            let groups: Vec<Vec<(Term, Term)>> = (0..12000)
                .map(|_| {
                    vec![
                        (var("a", 0, SortHint::Fresh), var("a", 4, SortHint::Fresh)),
                        (var("b", 0, SortHint::Fresh), var("b", 4, SortHint::Fresh)),
                    ]
                })
                .collect();
            let v = AcVariants { ac_rule: ac, substitutions: groups };
            let out = render_rule(&base, Some(&v));
            // Complete render: opens and closes the comment, all groups present.
            assert!(out.starts_with("rule (modulo E) Big:"));
            assert!(out.trim_end().ends_with("*/"));
            assert!(out.contains("12000. "));
        })
        .unwrap()
        .join()
        .unwrap();
}
