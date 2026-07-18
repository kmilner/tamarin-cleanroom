//! R1 tests — term core + signature block.
//!
//! Every expected string is byte-exact oracle output: either a fragment of a
//! probe capture (workspace/scratchpad/probes/<name>.out, provenance
//! `probe:<name>`) or of a round-1 reference block
//! (round1/targets/<file>.hs.txt, provenance `target:<file>`). One test is a
//! derived-shape regression and says so in its comment. Full-corpus truth is
//! `scripts/pretty_gate.sh` at integration, not this suite.

use pretty_clean::ast::*;
use pretty_clean::{render_signature_block, render_term};

// ── term fixture helpers ────────────────────────────────────────────────────

fn var(name: &str, idx: u64, sort: SortHint) -> Term {
    Term::Var(VarSpec {
        name: name.into(),
        idx,
        sort,
        typ: None,
    })
}

fn msg(name: &str) -> Term {
    var(name, 0, SortHint::Untagged)
}

fn fresh(name: &str) -> Term {
    var(name, 0, SortHint::Fresh)
}

fn pubv(name: &str) -> Term {
    var(name, 0, SortHint::Pub)
}

fn app(f: &str, args: Vec<Term>) -> Term {
    Term::App(f.into(), args)
}

fn bin(op: BinOp, a: Term, b: Term) -> Term {
    Term::BinOp(op, Box::new(a), Box::new(b))
}

fn exp(a: Term, b: Term) -> Term {
    bin(BinOp::Exp, a, b)
}

fn fdecl(name: &str, arity: usize, private: bool, destructor: bool) -> FunctionDecl {
    FunctionDecl {
        name: name.into(),
        arity,
        private,
        destructor,
    }
}

fn equation(lhs: Term, rhs: Term) -> Equation {
    Equation { lhs, rhs }
}

fn sig(
    builtins: &[&str],
    functions: Vec<FunctionDecl>,
    equations: Vec<Equation>,
) -> Signature {
    Signature {
        builtins: builtins.iter().map(|s| s.to_string()).collect(),
        functions,
        equations,
        convergent: false,
    }
}

// ── terms: variables and literals ───────────────────────────────────────────

#[test]
fn var_sigils_and_index() {
    // target:cav13_DH_example (~tid, $A), targets' lemma text (#i),
    // target:features_multiset_NumberSubtermTests (%x), builtin equations
    // (x.1) and AC-variant blocks (~x.7, XB.10).
    assert_eq!(render_term(&msg("x")), "x");
    assert_eq!(render_term(&fresh("tid")), "~tid");
    assert_eq!(render_term(&pubv("A")), "$A");
    assert_eq!(render_term(&var("i", 0, SortHint::Node)), "#i");
    assert_eq!(render_term(&var("x", 0, SortHint::Nat)), "%x");
    assert_eq!(render_term(&var("x", 1, SortHint::Untagged)), "x.1");
    assert_eq!(render_term(&var("x", 7, SortHint::Fresh)), "~x.7");
    assert_eq!(render_term(&var("XB", 10, SortHint::Untagged)), "XB.10");
}

#[test]
fn suffix_sorts_render_as_sigils() {
    // target:cav13_DH_example: source `x:fresh` echoes `~x` (BEHAVIOR.md
    // "SortHint::Suffix" entry).
    assert_eq!(
        render_term(&var("x", 0, SortHint::Suffix(SuffixSort::Fresh))),
        "~x"
    );
    assert_eq!(
        render_term(&var("A", 0, SortHint::Suffix(SuffixSort::Pub))),
        "$A"
    );
}

#[test]
fn literal_constants() {
    assert_eq!(render_term(&Term::PubLit("g".into())), "'g'"); // probe:t_gone
    assert_eq!(
        render_term(&Term::PubLit("hello_world".into())),
        "'hello_world'"
    ); // probe:t_pair
    assert_eq!(render_term(&Term::FreshLit("n".into())), "~'n'"); // probe:t_frlit
    assert_eq!(render_term(&Term::NatOne), "%1"); // probe:t_nat
    assert_eq!(render_term(&Term::NatLit("2".into())), "%2"); // probe:t_num2
    assert_eq!(render_term(&Term::Number(2)), "%2"); // probe:t_num2
    assert_eq!(render_term(&Term::NumberOne), "one"); // probe:t_one
    assert_eq!(render_term(&Term::DhNeutral), "DH_neutral"); // probe:t_gone
    assert_eq!(render_term(&app("zero", vec![])), "zero"); // probe:t_xor
    assert_eq!(render_term(&app("shk", vec![])), "shk"); // target:cav13_DH_example
}

// ── terms: application, pair, diff ──────────────────────────────────────────

#[test]
fn application_spacing() {
    // target:cav13_DH_example: `mac(shk, <g^~x, $A, $B>)` — comma-space
    // between args, no padding inside the parens, nullary `shk`/`g` bare.
    let t = app(
        "mac",
        vec![
            app("shk", vec![]),
            Term::Pair(vec![
                exp(app("g", vec![]), fresh("x")),
                pubv("A"),
                pubv("B"),
            ]),
        ],
    );
    assert_eq!(render_term(&t), "mac(shk, <g^~x, $A, $B>)");
}

#[test]
fn pair_flattening() {
    // probe:t_pair: right-nested flattens, non-last nesting is kept.
    let xyz = Term::Pair(vec![msg("x"), msg("y"), msg("z")]);
    assert_eq!(render_term(&xyz), "<x, y, z>");
    let right_nested = Term::Pair(vec![
        msg("x"),
        Term::Pair(vec![msg("y"), msg("z")]),
    ]);
    assert_eq!(render_term(&right_nested), "<x, y, z>");
    let left_nested = Term::Pair(vec![
        Term::Pair(vec![msg("x"), msg("y")]),
        msg("z"),
    ]);
    assert_eq!(render_term(&left_nested), "<<x, y>, z>");
}

#[test]
fn diff_renders_in_application_form() {
    // probe:t_diff (run with --diff): `diff(x, y)`.
    let t = Term::Diff(Box::new(msg("x")), Box::new(msg("y")));
    assert_eq!(render_term(&t), "diff(x, y)");
}

#[test]
fn composite_from_nslpk3() {
    // target:classic_NSLPK3: `aenc(<'1', ~ni, $I>, pkR)`.
    let t = app(
        "aenc",
        vec![
            Term::Pair(vec![Term::PubLit("1".into()), fresh("ni"), pubv("I")]),
            msg("pkR"),
        ],
    );
    assert_eq!(render_term(&t), "aenc(<'1', ~ni, $I>, pkR)");
}

// ── terms: exponentiation ───────────────────────────────────────────────────

#[test]
fn exp_chains_render_flat_both_nestings() {
    // probe:t_exp2: `('g'^~x)^~y` and `'g'^(~x^~y)` both echo `'g'^~x^~y`.
    let g = Term::PubLit("g".into());
    let left = exp(exp(g.clone(), fresh("x")), fresh("y"));
    let right = exp(g.clone(), exp(fresh("x"), fresh("y")));
    assert_eq!(render_term(&left), "'g'^~x^~y");
    assert_eq!(render_term(&right), "'g'^~x^~y");
    // AlgApp("exp", …) is the same operator (interface alternate encoding).
    let alg = Term::AlgApp(
        "exp".into(),
        Box::new(g),
        Box::new(fresh("x")),
    );
    assert_eq!(render_term(&alg), "'g'^~x");
}

#[test]
fn exp_with_app_operands() {
    // target:sp14_Joux: `em(XB, XC)^~ekA`; target:cav13_DH_example variants:
    // `x.10^inv(~x.7)`.
    let t = exp(app("em", vec![msg("XB"), msg("XC")]), fresh("ekA"));
    assert_eq!(render_term(&t), "em(XB, XC)^~ekA");
    let t2 = exp(
        var("x", 10, SortHint::Untagged),
        app("inv", vec![var("x", 7, SortHint::Fresh)]),
    );
    assert_eq!(render_term(&t2), "x.10^inv(~x.7)");
}

// ── terms: AC operators ─────────────────────────────────────────────────────

#[test]
fn mult_parenthesized_and_flattened() {
    // probe:t_mult2: `'g'^(~x*~y*~z)`; target:cav13_DH_example variants:
    // `inv((~x.7*x.11))` — the mult parens are intrinsic, even as an argument.
    let m = bin(
        BinOp::Mult,
        bin(BinOp::Mult, fresh("x"), fresh("y")),
        fresh("z"),
    );
    let t = exp(Term::PubLit("g".into()), m);
    assert_eq!(render_term(&t), "'g'^(~x*~y*~z)");
    let inner = bin(
        BinOp::Mult,
        var("x", 7, SortHint::Fresh),
        var("x", 11, SortHint::Untagged),
    );
    assert_eq!(
        render_term(&app("inv", vec![inner])),
        "inv((~x.7*x.11))"
    );
}

#[test]
fn xor_glyph_and_flattening() {
    // probe:t_xor: all nestings echo `(x⊕y⊕z)`; target:features_xor_xor:
    // `(~x⊕~y)`.
    let l = bin(BinOp::Xor, bin(BinOp::Xor, msg("x"), msg("y")), msg("z"));
    let r = bin(BinOp::Xor, msg("x"), bin(BinOp::Xor, msg("y"), msg("z")));
    assert_eq!(render_term(&l), "(x\u{2295}y\u{2295}z)");
    assert_eq!(render_term(&r), "(x\u{2295}y\u{2295}z)");
    assert_eq!(
        render_term(&bin(BinOp::Xor, fresh("x"), fresh("y"))),
        "(~x\u{2295}~y)"
    );
}

#[test]
fn union_and_natplus() {
    // probe:t_uni: `(x++y++z)`; target:sp14_Joux: `($B++$C)`;
    // probe:t_nat: `(%x%+%y%+%z)`; target:NumberSubtermTests: `(%x%+%1)`.
    let u = bin(
        BinOp::Union,
        msg("x"),
        bin(BinOp::Union, msg("y"), msg("z")),
    );
    assert_eq!(render_term(&u), "(x++y++z)");
    assert_eq!(
        render_term(&bin(BinOp::Union, pubv("B"), pubv("C"))),
        "($B++$C)"
    );
    let n = bin(
        BinOp::NatPlus,
        bin(
            BinOp::NatPlus,
            var("x", 0, SortHint::Nat),
            var("y", 0, SortHint::Nat),
        ),
        var("z", 0, SortHint::Nat),
    );
    assert_eq!(render_term(&n), "(%x%+%y%+%z)");
    assert_eq!(
        render_term(&bin(BinOp::NatPlus, var("x", 0, SortHint::Nat), Term::NatOne)),
        "(%x%+%1)"
    );
}

#[test]
fn union_wide_breaks_between_elements() {
    // DERIVED-SHAPE regression: the construction (elements carry the trailing
    // operator, fill break between elements, continuation aligned after `(`)
    // is pinned by probe:t_uniwide, which observed the same break inside a
    // fact at indent 3. This asserts the construction stays stable at column 0.
    let names: Vec<String> = (1..=4).map(|i| format!("{}{i}", "a".repeat(17))).collect();
    let t = bin(
        BinOp::Union,
        bin(
            BinOp::Union,
            bin(BinOp::Union, msg(&names[0]), msg(&names[1])),
            msg(&names[2]),
        ),
        msg(&names[3]),
    );
    assert_eq!(
        render_term(&t),
        "(aaaaaaaaaaaaaaaaa1++aaaaaaaaaaaaaaaaa2++aaaaaaaaaaaaaaaaa3++\n aaaaaaaaaaaaaaaaa4)"
    );
}

// ── signature: per-behavior ─────────────────────────────────────────────────

#[test]
fn base_signature_block() {
    // probe:b_none — no builtins, no user declarations.
    let expected = "\
// Function signature and definition of the equational theory E

functions: fst/1, pair/2, snd/1
equations: fst(<x.1, x.2>) = x.1, snd(<x.1, x.2>) = x.2";
    assert_eq!(render_signature_block(&sig(&[], vec![], vec![])), expected);
}

#[test]
fn function_attribute_spellings() {
    // probe:f_attrs.
    let s = sig(
        &[],
        vec![
            fdecl("a", 1, true, false),
            fdecl("d", 2, false, true),
            fdecl("b", 1, true, true),
            fdecl("c", 3, false, false),
        ],
        vec![],
    );
    let expected = "\
// Function signature and definition of the equational theory E

functions: a/1[private,constructor], b/1[private,destructor], c/3,
           d/2[destructor], fst/1, pair/2, snd/1
equations: fst(<x.1, x.2>) = x.1, snd(<x.1, x.2>) = x.2";
    assert_eq!(render_signature_block(&s), expected);
}

#[test]
fn function_sort_is_ascii_case_sensitive() {
    // probe:f_sort.
    let s = sig(
        &[],
        vec![
            fdecl("Bb", 1, false, false),
            fdecl("aa", 1, false, false),
            fdecl("Zz", 1, false, false),
            fdecl("a1", 2, false, false),
            fdecl("cA", 0, false, false),
        ],
        vec![],
    );
    assert!(render_signature_block(&s)
        .contains("functions: Bb/1, Zz/1, a1/2, aa/1, cA/0, fst/1, pair/2, snd/1"));
}

#[test]
fn function_dedup_user_vs_builtin() {
    // probe:f_dedup — `builtins: hashing` + `functions: h/1, k/1, k/1`.
    let s = sig(
        &["hashing"],
        vec![
            fdecl("h", 1, false, false),
            fdecl("k", 1, false, false),
            fdecl("k", 1, false, false),
        ],
        vec![],
    );
    assert!(render_signature_block(&s).contains("functions: fst/1, h/1, k/1, pair/2, snd/1"));
}

#[test]
fn builtins_dedup_and_canonical_pair() {
    // probe:b_dupline — `builtins: xor, xor, multiset` echoes canonically.
    let s = sig(&["xor", "xor", "multiset"], vec![], vec![]);
    assert!(render_signature_block(&s).contains("builtins: multiset, xor"));
}

#[test]
fn bilinear_pairing_induces_diffie_hellman() {
    // probe:b_bilinear-pairing.
    let s = sig(&["bilinear-pairing"], vec![], vec![]);
    assert!(render_signature_block(&s).contains("builtins: diffie-hellman, bilinear-pairing"));
}

#[test]
fn dest_pairing_flips_projections() {
    // probe:b_dest-pairing.
    let s = sig(&["dest-pairing"], vec![], vec![]);
    assert!(render_signature_block(&s)
        .contains("functions: fst/1[destructor], pair/2, snd/1[destructor]"));
}

#[test]
fn signing_expansion_block() {
    // probe:b_signing.
    let expected = "\
// Function signature and definition of the equational theory E

functions: fst/1, pair/2, pk/1, sign/2, snd/1, true/0, verify/3
equations:
    fst(<x.1, x.2>) = x.1,
    snd(<x.1, x.2>) = x.2,
    verify(sign(x.1, x.2), x.1, pk(x.2)) = true";
    assert_eq!(
        render_signature_block(&sig(&["signing"], vec![], vec![])),
        expected
    );
}

#[test]
fn dest_signing_expansion_block() {
    // probe:b_dest-signing — note the functions-line wrap before the
    // destructor-attributed item.
    let expected = "\
// Function signature and definition of the equational theory E

functions: fst/1, pair/2, pk/1, sign/2, snd/1, true/0,
           verify/3[destructor]
equations:
    fst(<x.1, x.2>) = x.1,
    snd(<x.1, x.2>) = x.2,
    verify(sign(x.1, x.2), x.1, pk(x.2)) = true";
    assert_eq!(
        render_signature_block(&sig(&["dest-signing"], vec![], vec![])),
        expected
    );
}

#[test]
fn revealing_signing_expansion_block() {
    // probe:b_revealing-signing — functions fill-wrap at column 11.
    let expected = "\
// Function signature and definition of the equational theory E

functions: fst/1, getMessage/1, pair/2, pk/1, revealSign/2,
           revealVerify/3, snd/1, true/0
equations:
    fst(<x.1, x.2>) = x.1,
    getMessage(revealSign(x.1, x.2)) = x.1,
    revealVerify(revealSign(x.1, x.2), x.1, pk(x.2)) = true,
    snd(<x.1, x.2>) = x.2";
    assert_eq!(
        render_signature_block(&sig(&["revealing-signing"], vec![], vec![])),
        expected
    );
}

#[test]
fn locations_report_expansion_block() {
    // probe:b_locrep.
    let expected = "\
// Function signature and definition of the equational theory E

functions: check_rep/2[destructor], fst/1, get_rep/1[destructor], pair/2,
           rep/2[private,constructor], report/1, snd/1
equations:
    check_rep(rep(x.1, x.2), x.2) = x.1,
    fst(<x.1, x.2>) = x.1,
    get_rep(rep(x.1, x.2)) = x.1,
    snd(<x.1, x.2>) = x.2";
    assert_eq!(
        render_signature_block(&sig(&["locations-report"], vec![], vec![])),
        expected
    );
}

#[test]
fn all_builtins_block() {
    // probe:b_all — canonical builtin order (source order scrambled),
    // cross-builtin dedup (single pk/1, true/0), builtins-line wrap at
    // column 10, functions-line wrap at column 11, equations one-per-line.
    let s = sig(
        &[
            "xor",
            "revealing-signing",
            "natural-numbers",
            "symmetric-encryption",
            "multiset",
            "signing",
            "hashing",
            "diffie-hellman",
            "asymmetric-encryption",
            "bilinear-pairing",
        ],
        vec![],
        vec![],
    );
    let expected = "\
// Function signature and definition of the equational theory E

builtins: diffie-hellman, bilinear-pairing, multiset, natural-numbers,
          xor
functions: adec/2, aenc/2, fst/1, getMessage/1, h/1, pair/2, pk/1,
           revealSign/2, revealVerify/3, sdec/2, senc/2, sign/2, snd/1, true/0,
           verify/3
equations:
    adec(aenc(x.1, pk(x.2)), x.2) = x.1,
    fst(<x.1, x.2>) = x.1,
    getMessage(revealSign(x.1, x.2)) = x.1,
    revealVerify(revealSign(x.1, x.2), x.1, pk(x.2)) = true,
    sdec(senc(x.1, x.2), x.2) = x.1,
    snd(<x.1, x.2>) = x.2,
    verify(sign(x.1, x.2), x.1, pk(x.2)) = true";
    assert_eq!(render_signature_block(&s), expected);
}

#[test]
fn equations_one_line_at_exactly_73_columns() {
    // probe:e_mid — the one-line form is exactly 73 columns and is kept.
    let s = sig(
        &[],
        vec![fdecl("pr", 2, false, false), fdecl("fa", 1, false, false)],
        vec![equation(
            app("fa", vec![app("pr", vec![msg("x"), msg("y")])]),
            msg("x"),
        )],
    );
    assert!(render_signature_block(&s).contains(
        "equations: fa(pr(x, y)) = x, fst(<x.1, x.2>) = x.1, snd(<x.1, x.2>) = x.2"
    ));
}

#[test]
fn convergent_header_and_one_equation_per_line() {
    // probe:e_conv — `[convergent]` preserved; broken block is one equation
    // per line even though all three would fit joined at indent 4.
    let s = Signature {
        builtins: vec![],
        functions: vec![fdecl("enc", 2, false, false), fdecl("dec", 2, false, false)],
        equations: vec![equation(
            app(
                "dec",
                vec![app("enc", vec![msg("x"), msg("y")]), msg("y")],
            ),
            msg("x"),
        )],
        convergent: true,
    };
    let expected = "\
// Function signature and definition of the equational theory E

functions: dec/2, enc/2, fst/1, pair/2, snd/1
equations [convergent]:
    dec(enc(x, y), y) = x,
    fst(<x.1, x.2>) = x.1,
    snd(<x.1, x.2>) = x.2";
    assert_eq!(render_signature_block(&s), expected);
}

#[test]
fn equations_same_head_sort_no_alpha_dedup() {
    // probe:e_adedup — user `fst(<a, b>) = a` is kept alongside the builtin
    // fst equation and sorts before it (byte order on rendered text).
    let s = sig(
        &[],
        vec![],
        vec![equation(
            app("fst", vec![Term::Pair(vec![msg("a"), msg("b")])]),
            msg("a"),
        )],
    );
    assert!(render_signature_block(&s).contains(
        "equations: fst(<a, b>) = a, fst(<x.1, x.2>) = x.1, snd(<x.1, x.2>) = x.2"
    ));
}

#[test]
fn exact_duplicate_equations_dedup() {
    // probe:e_dup.
    let user_eq = equation(
        app("fa", vec![app("pr", vec![msg("x"), msg("y")])]),
        msg("x"),
    );
    let s = sig(
        &[],
        vec![fdecl("pr", 2, false, false), fdecl("fa", 1, false, false)],
        vec![user_eq.clone(), user_eq],
    );
    assert!(render_signature_block(&s).contains(
        "equations: fa(pr(x, y)) = x, fst(<x.1, x.2>) = x.1, snd(<x.1, x.2>) = x.2"
    ));
}

#[test]
fn overlong_equation_breaks_at_equals() {
    // probe:e_long — functions fill-wrap, argument wrap aligned after the
    // open paren, and `= rhs` dropped to (equation indent − 2).
    let s = sig(
        &[],
        vec![
            fdecl("unwrapAAAAAAAAAAAAAAAAAAAA", 1, false, false),
            fdecl("wrapAAAAAAAAAAAAAAAAAAAAAA", 2, false, false),
        ],
        vec![equation(
            app(
                "unwrapAAAAAAAAAAAAAAAAAAAA",
                vec![app(
                    "wrapAAAAAAAAAAAAAAAAAAAAAA",
                    vec![msg("xlongvariablename1"), msg("ylongvariablename2")],
                )],
            ),
            msg("xlongvariablename1"),
        )],
    );
    let expected = "\
// Function signature and definition of the equational theory E

functions: fst/1, pair/2, snd/1, unwrapAAAAAAAAAAAAAAAAAAAA/1,
           wrapAAAAAAAAAAAAAAAAAAAAAA/2
equations:
    fst(<x.1, x.2>) = x.1,
    snd(<x.1, x.2>) = x.2,
    unwrapAAAAAAAAAAAAAAAAAAAA(wrapAAAAAAAAAAAAAAAAAAAAAA(xlongvariablename1,
                                                          ylongvariablename2))
  = xlongvariablename1";
    assert_eq!(render_signature_block(&s), expected);
}

// ── whole-signature-block parity: the 10 round-1 files ──────────────────────
// Fixtures are the SOURCE declarations of each file (readable inputs);
// expected strings are the corresponding lines of round1/targets/<f>.hs.txt.
// `parity_blocks_match_capture_files` additionally re-extracts each block
// straight from the capture file and byte-compares, guarding the literals
// against transcription slips (it self-skips once the capture dir is gone).

/// The 10 fixtures, paired with their capture-file basename.
fn round1_fixtures() -> Vec<(&'static str, Signature)> {
    vec![
        (
            "cav13_DH_example.spthy.hs.txt",
            sig(
                &["diffie-hellman"],
                vec![
                    fdecl("mac", 2, false, false),
                    fdecl("g", 0, false, false),
                    fdecl("shk", 0, true, false),
                ],
                vec![],
            ),
        ),
        (
            "classic_NSLPK3.spthy.hs.txt",
            sig(&["asymmetric-encryption"], vec![], vec![]),
        ),
        (
            "features_multiset_minimal_multiset.spthy.hs.txt",
            sig(&["multiset"], vec![], vec![]),
        ),
        (
            "features_multiset_NumberSubtermTests.spthy.hs.txt",
            sig(
                &[
                    "natural-numbers",
                    "multiset",
                    "diffie-hellman",
                    "xor",
                    "bilinear-pairing",
                    "hashing",
                ],
                vec![
                    fdecl("mypair", 2, false, false),
                    fdecl("myfst", 1, false, false),
                    fdecl("mysnd", 1, false, false),
                ],
                vec![
                    equation(
                        app("myfst", vec![app("mypair", vec![msg("a"), msg("b")])]),
                        msg("a"),
                    ),
                    equation(
                        app("mysnd", vec![app("mypair", vec![msg("a"), msg("b")])]),
                        msg("b"),
                    ),
                ],
            ),
        ),
        (
            "features_private_function_symbols_test.spthy.hs.txt",
            sig(&[], vec![fdecl("f", 0, true, false)], vec![]),
        ),
        ("features_xor_xor.spthy.hs.txt", sig(&["xor"], vec![], vec![])),
        (
            "features_xor_xorplusdh.spthy.hs.txt",
            sig(&["xor", "diffie-hellman"], vec![], vec![]),
        ),
        (
            "regression_trace_issue777.spthy.hs.txt",
            sig(&["diffie-hellman"], vec![], vec![]),
        ),
        (
            "sp14_Joux.spthy.hs.txt",
            sig(&["bilinear-pairing", "signing", "multiset"], vec![], vec![]),
        ),
        (
            "Tutorial.spthy.hs.txt",
            sig(
                &[],
                vec![
                    fdecl("h", 1, false, false),
                    fdecl("aenc", 2, false, false),
                    fdecl("adec", 2, false, false),
                    fdecl("pk", 1, false, false),
                ],
                vec![equation(
                    app(
                        "adec",
                        vec![
                            app("aenc", vec![msg("m"), app("pk", vec![msg("k")])]),
                            msg("k"),
                        ],
                    ),
                    msg("m"),
                )],
            ),
        ),
    ]
}

/// Signature block of a capture: the `// Function signature …` header, the
/// blank line after it, then the contiguous non-blank declaration lines.
fn extract_block(capture: &str) -> Option<String> {
    let lines: Vec<&str> = capture.lines().collect();
    let start = lines
        .iter()
        .position(|l| l.starts_with("// Function signature"))?;
    let mut end = start + 2; // header + blank line
    while end < lines.len() && !lines[end].is_empty() {
        end += 1;
    }
    Some(lines[start..end].join("\n"))
}

#[test]
fn parity_blocks_match_capture_files() {
    // Sealed-workspace location of the pre-materialized oracle captures;
    // self-skips when absent (e.g. after the crate moves at integration).
    let dir = std::path::Path::new("../../round1/targets");
    if !dir.is_dir() {
        return;
    }
    for (file, fixture) in round1_fixtures() {
        let capture = std::fs::read_to_string(dir.join(file))
            .unwrap_or_else(|e| panic!("reading {file}: {e}"));
        let expected = extract_block(&capture)
            .unwrap_or_else(|| panic!("no signature block found in {file}"));
        assert_eq!(
            render_signature_block(&fixture),
            expected,
            "signature-block divergence vs capture {file}"
        );
    }
}

#[test]
fn parity_cav13_dh_example() {
    // source: builtins: diffie-hellman; functions: mac/2, g/0, shk/0 [private]
    let s = sig(
        &["diffie-hellman"],
        vec![
            fdecl("mac", 2, false, false),
            fdecl("g", 0, false, false),
            fdecl("shk", 0, true, false),
        ],
        vec![],
    );
    let expected = "\
// Function signature and definition of the equational theory E

builtins: diffie-hellman
functions: fst/1, g/0, mac/2, pair/2, shk/0[private,constructor], snd/1
equations: fst(<x.1, x.2>) = x.1, snd(<x.1, x.2>) = x.2";
    assert_eq!(render_signature_block(&s), expected); // target:cav13_DH_example
}

#[test]
fn parity_classic_nslpk3() {
    // source: builtins: asymmetric-encryption
    let s = sig(&["asymmetric-encryption"], vec![], vec![]);
    let expected = "\
// Function signature and definition of the equational theory E

functions: adec/2, aenc/2, fst/1, pair/2, pk/1, snd/1
equations:
    adec(aenc(x.1, pk(x.2)), x.2) = x.1,
    fst(<x.1, x.2>) = x.1,
    snd(<x.1, x.2>) = x.2";
    assert_eq!(render_signature_block(&s), expected); // target:classic_NSLPK3
}

#[test]
fn parity_minimal_multiset() {
    let s = sig(&["multiset"], vec![], vec![]);
    let expected = "\
// Function signature and definition of the equational theory E

builtins: multiset
functions: fst/1, pair/2, snd/1
equations: fst(<x.1, x.2>) = x.1, snd(<x.1, x.2>) = x.2";
    // target:features_multiset_minimal_multiset
    assert_eq!(render_signature_block(&s), expected);
}

#[test]
fn parity_number_subterm_tests() {
    // source: builtins: natural-numbers, multiset, diffie-hellman, xor,
    //         bilinear-pairing, hashing
    //         functions: mypair/2, myfst/1, mysnd/1
    //         equations: myfst(mypair(a,b))=a, mysnd(mypair(a,b))=b
    let s = sig(
        &[
            "natural-numbers",
            "multiset",
            "diffie-hellman",
            "xor",
            "bilinear-pairing",
            "hashing",
        ],
        vec![
            fdecl("mypair", 2, false, false),
            fdecl("myfst", 1, false, false),
            fdecl("mysnd", 1, false, false),
        ],
        vec![
            equation(
                app("myfst", vec![app("mypair", vec![msg("a"), msg("b")])]),
                msg("a"),
            ),
            equation(
                app("mysnd", vec![app("mypair", vec![msg("a"), msg("b")])]),
                msg("b"),
            ),
        ],
    );
    let expected = "\
// Function signature and definition of the equational theory E

builtins: diffie-hellman, bilinear-pairing, multiset, natural-numbers,
          xor
functions: fst/1, h/1, myfst/1, mypair/2, mysnd/1, pair/2, snd/1
equations:
    fst(<x.1, x.2>) = x.1,
    myfst(mypair(a, b)) = a,
    mysnd(mypair(a, b)) = b,
    snd(<x.1, x.2>) = x.2";
    // target:features_multiset_NumberSubtermTests
    assert_eq!(render_signature_block(&s), expected);
}

#[test]
fn parity_private_function_symbols_test() {
    // source: functions: f/0 [private]
    let s = sig(&[], vec![fdecl("f", 0, true, false)], vec![]);
    let expected = "\
// Function signature and definition of the equational theory E

functions: f/0[private,constructor], fst/1, pair/2, snd/1
equations: fst(<x.1, x.2>) = x.1, snd(<x.1, x.2>) = x.2";
    // target:features_private_function_symbols_test
    assert_eq!(render_signature_block(&s), expected);
}

#[test]
fn parity_xor() {
    let s = sig(&["xor"], vec![], vec![]);
    let expected = "\
// Function signature and definition of the equational theory E

builtins: xor
functions: fst/1, pair/2, snd/1
equations: fst(<x.1, x.2>) = x.1, snd(<x.1, x.2>) = x.2";
    assert_eq!(render_signature_block(&s), expected); // target:features_xor_xor
}

#[test]
fn parity_xorplusdh() {
    // source: builtins: xor, diffie-hellman (echo reorders canonically)
    let s = sig(&["xor", "diffie-hellman"], vec![], vec![]);
    let expected = "\
// Function signature and definition of the equational theory E

builtins: diffie-hellman, xor
functions: fst/1, pair/2, snd/1
equations: fst(<x.1, x.2>) = x.1, snd(<x.1, x.2>) = x.2";
    // target:features_xor_xorplusdh
    assert_eq!(render_signature_block(&s), expected);
}

#[test]
fn parity_issue777() {
    // source: builtins: diffie-hellman (the macros block is outside the
    // signature section).
    let s = sig(&["diffie-hellman"], vec![], vec![]);
    let expected = "\
// Function signature and definition of the equational theory E

builtins: diffie-hellman
functions: fst/1, pair/2, snd/1
equations: fst(<x.1, x.2>) = x.1, snd(<x.1, x.2>) = x.2";
    // target:regression_trace_issue777
    assert_eq!(render_signature_block(&s), expected);
}

#[test]
fn parity_joux() {
    // source: builtins: bilinear-pairing, signing, multiset
    let s = sig(&["bilinear-pairing", "signing", "multiset"], vec![], vec![]);
    let expected = "\
// Function signature and definition of the equational theory E

builtins: diffie-hellman, bilinear-pairing, multiset
functions: fst/1, pair/2, pk/1, sign/2, snd/1, true/0, verify/3
equations:
    fst(<x.1, x.2>) = x.1,
    snd(<x.1, x.2>) = x.2,
    verify(sign(x.1, x.2), x.1, pk(x.2)) = true";
    assert_eq!(render_signature_block(&s), expected); // target:sp14_Joux
}

#[test]
fn parity_tutorial() {
    // source: functions: h/1, aenc/2, adec/2, pk/1
    //         equations: adec(aenc(m, pk(k)), k) = m
    let s = sig(
        &[],
        vec![
            fdecl("h", 1, false, false),
            fdecl("aenc", 2, false, false),
            fdecl("adec", 2, false, false),
            fdecl("pk", 1, false, false),
        ],
        vec![equation(
            app(
                "adec",
                vec![
                    app("aenc", vec![msg("m"), app("pk", vec![msg("k")])]),
                    msg("k"),
                ],
            ),
            msg("m"),
        )],
    );
    let expected = "\
// Function signature and definition of the equational theory E

functions: adec/2, aenc/2, fst/1, h/1, pair/2, pk/1, snd/1
equations:
    adec(aenc(m, pk(k)), k) = m,
    fst(<x.1, x.2>) = x.1,
    snd(<x.1, x.2>) = x.2";
    assert_eq!(render_signature_block(&s), expected); // target:Tutorial
}
