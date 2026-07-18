//! R1 test scaffold — term core + signature block.
//!
//! Pattern: construct an AST value as Rust constructors, assert its render
//! equals the exact snippet you observed from `oracle/pretty_oracle.sh`. The
//! `#[ignore]` markers keep the baseline green; REMOVE each one as you
//! implement that sub-target, and spot-verify the expected string against the
//! oracle. Full-file integration truth is `scripts/pretty_gate.sh`, not these.

use pretty_clean::ast::*;

// Compiles today: proves the AST model is usable. No renderer call.
#[test]
fn ast_model_constructs() {
    let t = Term::AlgApp(
        "exp".into(),
        Box::new(Term::Var(VarSpec { name: "g".into(), idx: 0, sort: SortHint::Untagged, typ: None })),
        Box::new(Term::Var(VarSpec { name: "x".into(), idx: 0, sort: SortHint::Fresh, typ: None })),
    );
    assert!(matches!(t, Term::AlgApp(..)));
}

#[test]
#[ignore = "R1: implement term::render, then un-ignore and confirm vs oracle"]
fn exp_renders_as_caret() {
    // Oracle: `g^~x` for exp(g, ~x). Confirm the exact bytes before asserting.
    let t = Term::AlgApp(
        "exp".into(),
        Box::new(Term::Var(VarSpec { name: "g".into(), idx: 0, sort: SortHint::Untagged, typ: None })),
        Box::new(Term::Var(VarSpec { name: "x".into(), idx: 0, sort: SortHint::Fresh, typ: None })),
    );
    assert_eq!(pretty_clean::term::render(&t), "g^~x");
}

#[test]
#[ignore = "R1: implement signature::render, then un-ignore and confirm vs oracle"]
fn signature_functions_line() {
    // Oracle DH_example: `functions: fst/1, g/0, mac/2, pair/2, shk/0[private,constructor], snd/1`
    let sig = Signature {
        builtins: vec![],
        functions: vec![FunctionDecl { name: "fst".into(), arity: 1, private: false, constructor: true }],
        equations: vec![],
    };
    let out = pretty_clean::signature::render(&sig);
    assert!(out.contains("functions: fst/1"));
}
