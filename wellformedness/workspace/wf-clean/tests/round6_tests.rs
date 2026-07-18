//! Round-6 byte-parity tests for the final corpus residuals.
//!
//! Each fixture is an OBSERVED oracle WARNING topic block (extracted from the
//! round-6 reference targets, cross-checked with the constructed probes
//! lhs1/lhs2/lhs3/pc1/pc2 in round6/probes). We build the matching AST by hand
//! and assert byte-for-byte reproduction of the affected topic block.

use wf_clean::ast::*;
use wf_clean::*;

// ---- AST builders ---------------------------------------------------------

fn var(name: &str, sort: SortHint) -> VarSpec {
    VarSpec { name: name.into(), idx: 0, sort, typ: None }
}
fn mv(name: &str) -> Term { Term::Var(var(name, SortHint::Msg)) }
fn pl(name: &str) -> Term { Term::PubLit(name.into()) }

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

fn theory(name: &str, items: Vec<TheoryItem>) -> Theory {
    Theory { is_diff: false, name: name.into(), configuration: None, items }
}

/// Render one topic block (header + underline + blank line + body) the way the
/// full report renders it, so a fixture can pin a single topic in isolation.
fn render_block(e: &report::WfError) -> String {
    format!("{}\n\n{}", underline_topic(&e.topic), e.message)
}

fn expect_block(e: &report::WfError, fixture: &str) {
    assert_eq!(render_block(e), fixture.trim_end_matches('\n'));
}

// ===========================================================================
// FAMILY pubcap - Public constants with mismatching capitalization
// ===========================================================================

#[test]
fn centralizedmonitor_pubcap_block() {
    // The CentralizedMonitor.spthy reference: the translated rule "Init" uses
    // the public constants 'c' and 'C' (same lowercase key, same first rule),
    // which collapse to one "rule "Init":  name 'C', 'c'" entry (ASCII sort
    // puts 'C' before 'c'). Mirrors probe pc1.
    let thy = theory("CentralizedMonitor", vec![rule(
        "Init",
        vec![],
        vec![fact("A", vec![pl("c"), pl("C")])],
        vec![],
    )]);
    let r = checks::public_names_report(&thy);
    assert_eq!(r.len(), 1);
    expect_block(&r[0], include_str!("fixtures/t6_centralizedmonitor_pubcap.txt"));
}

// ===========================================================================
// FAMILY listfmt/factusage - Facts occur in the LHS but not in any RHS
// ===========================================================================

#[test]
fn ble_lhs_not_rhs_per_rule_entries() {
    // The ble.spthy reference: ten entries, one per (rule, factName) premise
    // occurrence - RespChooseKeysize is listed separately for each of its four
    // rules, InitChooseKeysize for each of its four. Ten entries force the
    // index to right-align to the widest ("   1." three leading spaces, "  10."
    // two). No RHS facts exist, so no "Perhaps you want to use" suggestions
    // fire. Mirrors the ble target and probes lhs1/lhs2.
    let arity4 = || vec![mv("a"), mv("b"), mv("c"), mv("d")];
    let arity2 = || vec![mv("a"), mv("b")];
    let mut items = vec![
        rule("Oracle_f4", vec![fact("LowEntropyf4", arity4())], vec![], vec![]),
        rule("Oracle_passkey", vec![fact("LowEntropy", vec![mv("a")])], vec![], vec![]),
    ];
    for v in ["SS", "SW", "WS", "WW"] {
        items.push(rule(
            &format!("RespSelectKeysize{}", v),
            vec![fact("RespChooseKeysize", arity2())],
            vec![],
            vec![],
        ));
    }
    for v in ["SS", "SW", "WS", "WW"] {
        items.push(rule(
            &format!("InitSelectKeysize{}", v),
            vec![fact("InitChooseKeysize", arity2())],
            vec![],
            vec![],
        ));
    }
    let thy = theory("ble", items);
    let r = checks::fact_lhs_occur_no_rhs(&thy);
    assert_eq!(r.len(), 1);
    expect_block(&r[0], include_str!("fixtures/t6_ble_lhs.txt"));
}

#[test]
fn lhs_not_rhs_no_dedup_source_order() {
    // A fact reused as an LHS-only premise across non-adjacent rules yields one
    // entry per rule, kept in source order (NOT grouped by fact name); a fact
    // repeated inside one rule's premises yields one entry per occurrence
    // (probes lhs1/lhs2).
    let thy = theory("T", vec![
        rule("A", vec![fact("AA", vec![mv("x")]), fact("BB", vec![mv("x")])], vec![], vec![]),
        rule("B", vec![fact("CC", vec![mv("x")])], vec![], vec![]),
        rule("C", vec![fact("AA", vec![mv("y")])], vec![], vec![]),
        rule("D", vec![fact("DD", vec![mv("x")]), fact("DD", vec![mv("x")])], vec![], vec![]),
    ]);
    let r = checks::fact_lhs_occur_no_rhs(&thy);
    assert_eq!(r.len(), 1);
    // Six numbered entries (single-digit index, two-space margin), separated by
    // a line of exactly two spaces; source order, no dedup.
    let want = [("A", "AA"), ("A", "BB"), ("B", "CC"), ("C", "AA"), ("D", "DD"), ("D", "DD")]
        .iter()
        .enumerate()
        .map(|(i, (rl, nm))| {
            format!(
                "  {}. in rule \"{}\":  factName `{}' arity: 1 multiplicity: Linear",
                i + 1,
                rl,
                nm
            )
        })
        .collect::<Vec<_>>()
        .join("\n  \n");
    assert_eq!(r[0].message, want);
}

#[test]
fn lhs_not_rhs_rhs_identity_suppresses_all_occurrences() {
    // A fact identity present on some RHS suppresses every one of its LHS
    // occurrences, even across rules (probe lhs3): AA is a conclusion of rule
    // Z, so neither of its LHS uses (rules A, C) is listed - only BB, whose
    // nearest RHS fact AA (edit distance 2 <= 3) is suggested.
    let thy = theory("T", vec![
        rule("Z", vec![], vec![], vec![fact("AA", vec![mv("s")])]),
        rule("A", vec![fact("AA", vec![mv("x")]), fact("BB", vec![mv("x")])], vec![], vec![]),
        rule("C", vec![fact("AA", vec![mv("y")])], vec![], vec![]),
    ]);
    let r = checks::fact_lhs_occur_no_rhs(&thy);
    assert_eq!(r.len(), 1);
    assert_eq!(
        r[0].message,
        "  1. in rule \"A\":  factName `BB' arity: 1 multiplicity: Linear. \
Perhaps you want to use the fact in rule \"Z\":  factName `AA' arity: 1 multiplicity: Linear"
    );
}
