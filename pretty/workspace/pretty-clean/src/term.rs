//! R1 — TERM rendering (the deep core; every other sub-target reuses this).
//!
//! Every glyph, spacing and parenthesization rule here is oracle-pinned; see
//! workspace/BEHAVIOR.md "Term rendering" for the probe provenance of each
//! case. Summary of the observed surface:
//!
//!   * sort sigils `~ $ # %` + `.idx` suffix when the index is > 0;
//!   * public constants `'name'`, fresh constants `~'name'`, naturals `%n`,
//!     DH constants `one` / `DH_neutral`;
//!   * application `f(a, b)` — comma attached to the preceding argument, fill
//!     space between arguments, wrap aligned after `(`; arity 0 renders bare;
//!   * tuples `<a, b, c>` — `", "` attached to each element (a wrapped line
//!     keeps the trailing space), right-nested pairs flattened;
//!   * `^` chains render flat for BOTH association orders, no parens added;
//!   * AC operators `*`, `⊕`, `++`, `%+` — self-parenthesized `(a*b*c)`,
//!     flattened across both sides, no spaces, break between elements with
//!     the operator attached to the preceding element;
//!   * `diff(l, r)` in application form.

use crate::ast::{BinOp, SortHint, SuffixSort, Term, VarSpec};
use crate::doc::{
    beside_op, char, fcat, fsep, punctuate, render_with, text, Doc,
};

/// Theory-echo layout parameters (SPEC; every wrap observation in
/// BEHAVIOR.md reproduces at these settings).
pub(crate) const WIDTH: isize = 110;
pub(crate) const RIBBON: isize = 73;

/// Render one term at the echo's layout parameters, starting at column 0.
pub fn render(t: &Term) -> String {
    render_with(WIDTH, RIBBON, &doc(t))
}

/// The term's `Doc` (embedded by the signature/rule/formula renderers).
pub(crate) fn doc(t: &Term) -> Doc {
    match t {
        Term::Var(v) => text(&var_str(v)),
        Term::PubLit(s) => text(&format!("'{s}'")),
        Term::FreshLit(s) => text(&format!("~'{s}'")),
        Term::NatLit(s) => text(&format!("%{s}")),
        Term::Number(n) => text(&format!("%{n}")),
        Term::NumberOne => text("one"),
        Term::NatOne => text("%1"),
        Term::DhNeutral => text("DH_neutral"),
        Term::App(f, args) => {
            if args.is_empty() {
                text(f)
            } else {
                app_doc(f, args)
            }
        }
        Term::AlgApp(name, a, b) => {
            if name == "exp" {
                exp_doc(t)
            } else {
                app_doc(name, &[(**a).clone(), (**b).clone()])
            }
        }
        Term::Pair(elems) => pair_doc(elems),
        Term::Diff(l, r) => app_doc("diff", &[(**l).clone(), (**r).clone()]),
        Term::BinOp(BinOp::Exp, _, _) => exp_doc(t),
        Term::BinOp(op, _, _) => ac_doc(*op, t),
        // UNOBSERVABLE placeholder (BEHAVIOR.md): not oracle-pinned.
        Term::PatMatch(inner) => beside_op(text("="), doc(inner)),
    }
}

/// Sigil + name + `.idx` suffix (suffix only when idx > 0).
fn var_str(v: &VarSpec) -> String {
    let sigil = match v.sort {
        SortHint::Msg | SortHint::Untagged | SortHint::Suffix(SuffixSort::Msg) => "",
        SortHint::Pub | SortHint::Suffix(SuffixSort::Pub) => "$",
        SortHint::Fresh | SortHint::Suffix(SuffixSort::Fresh) => "~",
        SortHint::Node | SortHint::Suffix(SuffixSort::Node) => "#",
        SortHint::Nat | SortHint::Suffix(SuffixSort::Nat) => "%",
    };
    if v.idx > 0 {
        format!("{sigil}{}.{}", v.name, v.idx)
    } else {
        format!("{sigil}{}", v.name)
    }
}

/// `f(a1, a2, …)`: commas attach to the preceding argument, fill space
/// between arguments (fsep), wrap aligns after the `(`, `)` attaches to the
/// last argument (probe:t_wide W2, probe:e_long).
fn app_doc(f: &str, args: &[Term]) -> Doc {
    let arg_docs = punctuate(&char(','), args.iter().map(doc).collect());
    beside_op(
        beside_op(text(&format!("{f}(")), fsep(arg_docs)),
        char(')'),
    )
}

/// `<a, b, c>`: `", "` attaches to each non-last element (fcat — a wrapped
/// line keeps the trailing space, probe:t_wide W1), `<`/`>` attach outside.
/// A Pair in LAST position flattens into the enclosing tuple; other positions
/// keep their own delimiters (probe:t_pair).
fn pair_doc(elems: &[Term]) -> Doc {
    let mut flat: Vec<&Term> = Vec::new();
    collect_pair(elems, &mut flat);
    let docs = punctuate(&text(", "), flat.into_iter().map(doc).collect());
    beside_op(beside_op(char('<'), fcat(docs)), char('>'))
}

fn collect_pair<'a>(elems: &'a [Term], out: &mut Vec<&'a Term>) {
    for (i, e) in elems.iter().enumerate() {
        match e {
            Term::Pair(inner) if i + 1 == elems.len() => collect_pair(inner, out),
            _ => out.push(e),
        }
    }
}

/// `a^b^c`: chains flatten for BOTH association orders (probe:t_exp2), the
/// `^` attaches to the preceding operand, no spaces, no parens added.
fn exp_doc(t: &Term) -> Doc {
    let mut leaves: Vec<&Term> = Vec::new();
    collect_exp(t, &mut leaves);
    let docs = punctuate(&char('^'), leaves.into_iter().map(doc).collect());
    fcat(docs)
}

fn collect_exp<'a>(t: &'a Term, out: &mut Vec<&'a Term>) {
    match t {
        Term::BinOp(BinOp::Exp, a, b) => {
            collect_exp(a, out);
            collect_exp(b, out);
        }
        Term::AlgApp(name, a, b) if name == "exp" => {
            collect_exp(a, out);
            collect_exp(b, out);
        }
        _ => out.push(t),
    }
}

/// AC operator: `(a<op>b<op>c)` — self-parenthesized, flattened across both
/// sides, operator attached to the preceding element, no spaces, fcat break
/// between elements (probes:t_xor, t_uni, t_nat, t_mult2, t_uniwide).
fn ac_doc(op: BinOp, t: &Term) -> Doc {
    let glyph = match op {
        BinOp::Mult => "*",
        BinOp::Union => "++",
        BinOp::Xor => "\u{2295}",
        BinOp::NatPlus => "%+",
        BinOp::Exp => unreachable!("Exp renders via exp_doc"),
    };
    let mut leaves: Vec<&Term> = Vec::new();
    collect_ac(op, t, &mut leaves);
    let docs = punctuate(&text(glyph), leaves.into_iter().map(doc).collect());
    beside_op(beside_op(char('('), fcat(docs)), char(')'))
}

fn collect_ac<'a>(op: BinOp, t: &'a Term, out: &mut Vec<&'a Term>) {
    match t {
        Term::BinOp(o, a, b) if *o == op => {
            collect_ac(op, a, out);
            collect_ac(op, b, out);
        }
        _ => out.push(t),
    }
}
