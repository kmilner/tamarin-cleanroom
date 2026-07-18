//! R1 — signature block: `builtins:` / `functions:` / `equations:`.
//!
//! Renders the declared signature as the theory echo's first section,
//! performing the OBSERVABLE closure over builtins (all oracle-pinned; see
//! workspace/BEHAVIOR.md "Signature section" for probe provenance):
//!
//!   * only diffie-hellman / bilinear-pairing / multiset / natural-numbers /
//!     xor keep a `builtins:` entry, in exactly that canonical order;
//!     bilinear-pairing induces diffie-hellman; every other builtin expands
//!     into function/equation entries instead;
//!   * the base pairing symbols `fst/1, pair/2, snd/1` and the two projection
//!     equations are always present (`dest-pairing` flips fst/snd to
//!     `[destructor]`);
//!   * functions dedup and sort byte-wise; attributes render
//!     `[private,constructor]` / `[private,destructor]` / `[destructor]` /
//!     nothing;
//!   * equations dedup exactly and sort byte-wise on their rendered text;
//!   * layout: builtins and functions are `text prefix <> fsep` fill lines;
//!     the equations block is all-or-nothing — one line when it fits the
//!     73-column ribbon, otherwise one equation per line at indent 4; an
//!     overlong equation drops `= rhs` to (equation indent − 2).

use crate::ast::{Equation, FunctionDecl, Signature, SortHint, Term, VarSpec};
use crate::doc::{
    beside_op, char, fsep, nest, punctuate, render_with, sep, text, vcat, Doc,
};
use crate::term::{self, RIBBON, WIDTH};

/// Header comment above the declarations (byte-exact from every capture).
const HEADER: &str = "// Function signature and definition of the equational theory E";

/// Full signature section: header comment, blank line, then the declaration
/// lines (`builtins:` only when it has surviving entries).
pub fn render(sig: &Signature) -> String {
    render_with(WIDTH, RIBBON, &block_doc(sig))
}

pub(crate) fn block_doc(sig: &Signature) -> Doc {
    let mut lines = vec![text(HEADER), text("")];
    let builtin_names = builtins_line_entries(&sig.builtins);
    if !builtin_names.is_empty() {
        lines.push(fill_line(
            "builtins: ",
            builtin_names.iter().map(|n| text(n)).collect(),
        ));
    }
    lines.push(fill_line(
        "functions: ",
        merged_function_items(sig).iter().map(|s| text(s)).collect(),
    ));
    lines.push(equations_doc(sig));
    vcat(lines)
}

/// `text prefix <> fsep (punctuate ',' items)`: comma attached to the
/// preceding item, fill space between items, continuation aligned after the
/// prefix (probe:b_all, probe:b_revealing-signing).
fn fill_line(prefix: &str, items: Vec<Doc>) -> Doc {
    beside_op(text(prefix), fsep(punctuate(&char(','), items)))
}

// ── builtins line ───────────────────────────────────────────────────────────

/// The five builtins with a line entry, in canonical order (probe:b_all).
const LINE_ORDER: [&str; 5] = [
    "diffie-hellman",
    "bilinear-pairing",
    "multiset",
    "natural-numbers",
    "xor",
];

fn builtins_line_entries(declared: &[String]) -> Vec<&'static str> {
    let mut present = [false; 5];
    for b in declared {
        match b.as_str() {
            "diffie-hellman" => present[0] = true,
            // bilinear-pairing induces diffie-hellman (probe:b_bilinear-pairing).
            "bilinear-pairing" => {
                present[0] = true;
                present[1] = true;
            }
            "multiset" => present[2] = true,
            "natural-numbers" => present[3] = true,
            "xor" => present[4] = true,
            _ => {}
        }
    }
    LINE_ORDER
        .iter()
        .zip(present)
        .filter_map(|(n, p)| p.then_some(*n))
        .collect()
}

// ── functions line ──────────────────────────────────────────────────────────

fn decl(name: &str, arity: usize, private: bool, destructor: bool) -> FunctionDecl {
    FunctionDecl {
        name: name.into(),
        arity,
        private,
        destructor,
    }
}

/// `name/arity` + attribute suffix (probe:f_attrs): `private` implies a
/// role keyword is shown too; a public destructor shows `[destructor]` alone;
/// a public constructor shows nothing.
fn function_item(d: &FunctionDecl) -> String {
    let mut s = format!("{}/{}", d.name, d.arity);
    if d.private || d.destructor {
        let mut attrs: Vec<&str> = Vec::new();
        if d.private {
            attrs.push("private");
        }
        attrs.push(if d.destructor { "destructor" } else { "constructor" });
        s.push('[');
        s.push_str(&attrs.join(","));
        s.push(']');
    }
    s
}

/// Base + builtin-expansion + user function symbols, deduped and byte-sorted.
/// Sorting the full item text equals sorting by name (`/` orders below every
/// identifier character), matching probe:f_sort.
fn merged_function_items(sig: &Signature) -> Vec<String> {
    let mut decls: Vec<FunctionDecl> = base_functions(&sig.builtins);
    for b in &sig.builtins {
        decls.extend(builtin_functions(b));
    }
    decls.extend(sig.functions.iter().cloned());
    let mut items: Vec<String> = decls.iter().map(function_item).collect();
    items.sort();
    items.dedup();
    items
}

/// Always-present pairing symbols; `dest-pairing` flips the projections to
/// destructors (probe:b_none, probe:b_dest-pairing).
fn base_functions(declared: &[String]) -> Vec<FunctionDecl> {
    let dest = declared.iter().any(|b| b == "dest-pairing");
    vec![
        decl("fst", 1, false, dest),
        decl("pair", 2, false, false),
        decl("snd", 1, false, dest),
    ]
}

/// Function symbols induced by one declared builtin (probes:b_*; compatibility
/// content learned from oracle output).
fn builtin_functions(name: &str) -> Vec<FunctionDecl> {
    match name {
        "hashing" => vec![decl("h", 1, false, false)],
        "asymmetric-encryption" | "dest-asymmetric-encryption" => vec![
            decl("adec", 2, false, name.starts_with("dest-")),
            decl("aenc", 2, false, false),
            decl("pk", 1, false, false),
        ],
        "signing" | "dest-signing" => vec![
            decl("pk", 1, false, false),
            decl("sign", 2, false, false),
            decl("true", 0, false, false),
            decl("verify", 3, false, name.starts_with("dest-")),
        ],
        "symmetric-encryption" | "dest-symmetric-encryption" => vec![
            decl("sdec", 2, false, name.starts_with("dest-")),
            decl("senc", 2, false, false),
        ],
        "revealing-signing" => vec![
            decl("getMessage", 1, false, false),
            decl("pk", 1, false, false),
            decl("revealSign", 2, false, false),
            decl("revealVerify", 3, false, false),
            decl("true", 0, false, false),
        ],
        "locations-report" => vec![
            decl("check_rep", 2, false, true),
            decl("get_rep", 1, false, true),
            decl("rep", 2, true, false),
            decl("report", 1, false, false),
        ],
        _ => Vec::new(),
    }
}

// ── equations block ─────────────────────────────────────────────────────────

/// `sep (header : map (nest 4) (punctuate ',' eqs))`: one line when it fits,
/// otherwise header alone and one equation per line at indent 4
/// (probe:e_mid, probe:e_conv, target:Tutorial).
fn equations_doc(sig: &Signature) -> Doc {
    let header = if sig.convergent {
        "equations [convergent]:"
    } else {
        "equations:"
    };
    let eqs = merged_equations(sig);
    let mut elems = vec![text(header)];
    for d in punctuate(&char(','), eqs.iter().map(equation_doc).collect()) {
        elems.push(nest(4, &d));
    }
    sep(elems)
}

/// `sep [lhs, nest (-2) ("= " <> rhs)]`: one line `lhs = rhs`; an overlong
/// equation drops `= rhs` to (equation indent − 2) (probe:e_long).
fn equation_doc(eq: &Equation) -> Doc {
    sep(vec![
        term::doc(&eq.lhs),
        nest(-2, &beside_op(text("= "), term::doc(&eq.rhs))),
    ])
}

/// One-line text of an equation — the dedup/sort key (probe:e_adedup pins
/// byte order on this rendered text; probe:e_dup pins exact-dedup).
fn equation_key(eq: &Equation) -> String {
    render_with(isize::MAX / 2, isize::MAX / 2, &equation_doc(eq))
}

/// Base + builtin-expansion + user equations, deduped and byte-sorted on
/// their rendered text.
fn merged_equations(sig: &Signature) -> Vec<Equation> {
    let mut eqs = base_equations();
    for b in &sig.builtins {
        eqs.extend(builtin_equations(b));
    }
    eqs.extend(sig.equations.iter().cloned());
    let mut keyed: Vec<(String, Equation)> =
        eqs.into_iter().map(|e| (equation_key(&e), e)).collect();
    keyed.sort_by(|a, b| a.0.cmp(&b.0));
    keyed.dedup_by(|a, b| a.0 == b.0);
    keyed.into_iter().map(|(_, e)| e).collect()
}

/// `x.<i>` — the variable spelling in builtin-induced equations
/// (probe:b_none and every builtin expansion probe).
fn xv(i: u64) -> Term {
    Term::Var(VarSpec {
        name: "x".into(),
        idx: i,
        sort: SortHint::Untagged,
        typ: None,
    })
}

fn app(f: &str, args: Vec<Term>) -> Term {
    Term::App(f.into(), args)
}

fn eq(lhs: Term, rhs: Term) -> Equation {
    Equation { lhs, rhs }
}

/// `fst(<x.1, x.2>) = x.1`, `snd(<x.1, x.2>) = x.2` (probe:b_none).
fn base_equations() -> Vec<Equation> {
    vec![
        eq(app("fst", vec![Term::Pair(vec![xv(1), xv(2)])]), xv(1)),
        eq(app("snd", vec![Term::Pair(vec![xv(1), xv(2)])]), xv(2)),
    ]
}

/// Equations induced by one declared builtin (probes:b_*).
fn builtin_equations(name: &str) -> Vec<Equation> {
    match name {
        // adec(aenc(x.1, pk(x.2)), x.2) = x.1
        "asymmetric-encryption" | "dest-asymmetric-encryption" => vec![eq(
            app(
                "adec",
                vec![
                    app("aenc", vec![xv(1), app("pk", vec![xv(2)])]),
                    xv(2),
                ],
            ),
            xv(1),
        )],
        // verify(sign(x.1, x.2), x.1, pk(x.2)) = true
        "signing" | "dest-signing" => vec![eq(
            app(
                "verify",
                vec![
                    app("sign", vec![xv(1), xv(2)]),
                    xv(1),
                    app("pk", vec![xv(2)]),
                ],
            ),
            app("true", vec![]),
        )],
        // sdec(senc(x.1, x.2), x.2) = x.1
        "symmetric-encryption" | "dest-symmetric-encryption" => vec![eq(
            app(
                "sdec",
                vec![app("senc", vec![xv(1), xv(2)]), xv(2)],
            ),
            xv(1),
        )],
        // getMessage(revealSign(x.1, x.2)) = x.1
        // revealVerify(revealSign(x.1, x.2), x.1, pk(x.2)) = true
        "revealing-signing" => vec![
            eq(
                app("getMessage", vec![app("revealSign", vec![xv(1), xv(2)])]),
                xv(1),
            ),
            eq(
                app(
                    "revealVerify",
                    vec![
                        app("revealSign", vec![xv(1), xv(2)]),
                        xv(1),
                        app("pk", vec![xv(2)]),
                    ],
                ),
                app("true", vec![]),
            ),
        ],
        // check_rep(rep(x.1, x.2), x.2) = x.1 ; get_rep(rep(x.1, x.2)) = x.1
        "locations-report" => vec![
            eq(
                app(
                    "check_rep",
                    vec![app("rep", vec![xv(1), xv(2)]), xv(2)],
                ),
                xv(1),
            ),
            eq(app("get_rep", vec![app("rep", vec![xv(1), xv(2)])]), xv(1)),
        ],
        _ => Vec::new(),
    }
}
