//! R3 tests — restriction / lemma / formula rendering.
//!
//! Two layers, mirroring rounds 1–2:
//!  1. Probe-pinned unit tests: hand-written fixtures asserted against
//!     byte-exact oracle output (probe captures under
//!     workspace/scratchpad/probes/, provenance `probe:<name>`).
//!  2. Whole-block parity over the round-3 curated corpus captures
//!     (round3/targets/*.hs.txt): every restriction block and every lemma
//!     block of every file is parsed back into the crate's AST by a
//!     layout-insensitive echo parser (test harness only — it discards all
//!     whitespace, so layout can only come from the renderer) and
//!     re-rendered; the bytes must match the capture. Guarded-comment
//!     content and embedded proof scripts are opaque inputs and are carried
//!     verbatim (they are NOT produced by this crate).

use pretty_clean::ast::*;
use pretty_clean::{render_lemma, render_restriction};

// ── fixture helpers ─────────────────────────────────────────────────────────

fn var(name: &str, idx: u64, sort: SortHint) -> VarSpec {
    VarSpec {
        name: name.into(),
        idx,
        sort,
        typ: None,
    }
}

fn msgv(name: &str) -> VarSpec {
    var(name, 0, SortHint::Untagged)
}

fn nodev(name: &str) -> VarSpec {
    var(name, 0, SortHint::Node)
}

fn tvar(name: &str) -> Term {
    Term::Var(nodev(name))
}

fn mvar(name: &str) -> Term {
    Term::Var(msgv(name))
}

fn fact(name: &str, args: Vec<Term>) -> Fact {
    Fact {
        persistent: false,
        name: name.into(),
        args,
        annotations: vec![],
    }
}

fn action(name: &str, args: Vec<Term>, tp: &str) -> Formula {
    Formula::Atom(Atom::Action(fact(name, args), tvar(tp)))
}

fn forall(vs: Vec<VarSpec>, b: Formula) -> Formula {
    Formula::Forall(vs, Box::new(b))
}

fn exists(vs: Vec<VarSpec>, b: Formula) -> Formula {
    Formula::Exists(vs, Box::new(b))
}

fn implies(l: Formula, r: Formula) -> Formula {
    Formula::Implies(Box::new(l), Box::new(r))
}

fn and(l: Formula, r: Formula) -> Formula {
    Formula::And(Box::new(l), Box::new(r))
}

fn or(l: Formula, r: Formula) -> Formula {
    Formula::Or(Box::new(l), Box::new(r))
}

fn not(f: Formula) -> Formula {
    Formula::Not(Box::new(f))
}

fn eq(l: Term, r: Term) -> Formula {
    Formula::Atom(Atom::Eq(l, r))
}

fn less(l: Term, r: Term) -> Formula {
    Formula::Atom(Atom::Less(l, r))
}

fn restriction(name: &str, f: Formula) -> Restriction {
    Restriction {
        name: name.into(),
        formula: f,
    }
}

fn lemma(name: &str, tq: TraceQuantifier, f: Formula) -> Lemma {
    Lemma {
        name: name.into(),
        attributes: vec![],
        trace_quantifier: tq,
        formula: f,
        proof: None,
    }
}

// ── probe-pinned unit tests ─────────────────────────────────────────────────

#[test]
fn restriction_wrapper_and_safety() {
    // probe:q_w1 r_eq — safety restriction: statement, "// safety formula",
    // blank line, expanded-formula comment.
    let r = restriction(
        "r_eq",
        forall(
            vec![msgv("x"), msgv("y"), nodev("i")],
            implies(
                action("B", vec![mvar("x"), mvar("y")], "i"),
                eq(mvar("x"), mvar("y")),
            ),
        ),
    );
    let expected = [
        "restriction r_eq:",
        "  \"\u{2200} x y #i. (B( x, y ) @ #i) \u{21d2} (x = y)\"",
        "  // safety formula",
        "",
        "  /*",
        "  expanded formula:",
        "  \"\u{2200} x y #i. (B( x, y ) @ #i) \u{21d2} (x = y)\"",
        "  */",
    ]
    .join("\n");
    assert_eq!(render_restriction(&r), expected);

    // probe:q_w1 r_alltr — an ∃ conclusion defeats the safety line.
    let r2 = restriction(
        "r_alltr",
        forall(
            vec![msgv("x"), nodev("i")],
            implies(
                action("A", vec![mvar("x")], "i"),
                exists(
                    vec![nodev("j")],
                    action("B", vec![mvar("x"), mvar("x")], "j"),
                ),
            ),
        ),
    );
    let expected2 = [
        "restriction r_alltr:",
        "  \"\u{2200} x #i. (A( x ) @ #i) \u{21d2} (\u{2203} #j. B( x, x ) @ #j)\"",
        "",
        "  /*",
        "  expanded formula:",
        "  \"\u{2200} x #i. (A( x ) @ #i) \u{21d2} (\u{2203} #j. B( x, x ) @ #j)\"",
        "  */",
    ]
    .join("\n");
    assert_eq!(render_restriction(&r2), expected2);

    // probe:q_s2 u4 — ¬∃ in the ANTECEDENT (an ∃ in NNF) defeats safety;
    // probe:q_s1 s8 — ¬∃ in the CONCLUSION (a ∀ in NNF) keeps it.
    let ex_past = exists(
        vec![nodev("j")],
        and(
            action("B", vec![mvar("x"), mvar("x")], "j"),
            less(tvar("j"), tvar("i")),
        ),
    );
    let u4 = forall(
        vec![msgv("x"), nodev("i")],
        implies(
            and(action("A", vec![mvar("x")], "i"), not(ex_past)),
            eq(mvar("x"), Term::PubLit("c".into())),
        ),
    );
    assert!(!render_restriction(&restriction("u", u4)).contains("safety"));
    let s8 = forall(
        vec![msgv("x"), nodev("i")],
        implies(
            action("A", vec![mvar("x")], "i"),
            not(exists(
                vec![nodev("j")],
                action("B", vec![mvar("x"), mvar("x")], "j"),
            )),
        ),
    );
    assert!(render_restriction(&restriction("s", s8)).contains("// safety formula"));
}

#[test]
fn lemma_wrapper_one_line_and_guarded() {
    // probe:q_w1 l_top / l_bot — one-line statements, ⊤/⊥ glyphs, guarded
    // comment header keyed by the trace quantifier, `by sorry` tail.
    let l = lemma("l_top", TraceQuantifier::AllTraces, Formula::True);
    let g = Guarded::Formula("\"\u{22a5}\"".into());
    let expected = [
        "lemma l_top:",
        "  all-traces \"\u{22a4}\"",
        "/*",
        "guarded formula characterizing all counter-examples:",
        "\"\u{22a5}\"",
        "*/",
        "by sorry",
    ]
    .join("\n");
    assert_eq!(render_lemma(&l, Some(&g)), expected);

    let l2 = lemma("l_bot", TraceQuantifier::ExistsTrace, Formula::False);
    let expected2 = [
        "lemma l_bot:",
        "  exists-trace \"\u{22a5}\"",
        "/*",
        "guarded formula characterizing all satisfying traces:",
        "\"\u{22a5}\"",
        "*/",
        "by sorry",
    ]
    .join("\n");
    assert_eq!(render_lemma(&l2, Some(&g)), expected2);
}

#[test]
fn lemma_attrs_source_order() {
    // probe:q_la1 la4 — attributes echo in source order, duplicates kept.
    let mut l = lemma(
        "la4",
        TraceQuantifier::AllTraces,
        forall(
            vec![msgv("x"), nodev("i")],
            implies(
                action("A", vec![mvar("x")], "i"),
                exists(
                    vec![nodev("j")],
                    action("B", vec![mvar("x"), mvar("x")], "j"),
                ),
            ),
        ),
    );
    l.attributes = vec![
        LemmaAttr::HideLemma("la1".into()),
        LemmaAttr::UseInduction,
        LemmaAttr::HideLemma("la3".into()),
        LemmaAttr::Reuse,
    ];
    let out = render_lemma(&l, None);
    assert_eq!(
        out.lines().next().unwrap(),
        "lemma la4 [hide_lemma=la1, use_induction, hide_lemma=la3, reuse]:"
    );
}

#[test]
fn guarded_conversion_failed_comment() {
    // probe:q_r1 — the failed-conversion comment variant: alternate header,
    // error text indented two columns.
    let l = lemma(
        "r1_forall_and",
        TraceQuantifier::AllTraces,
        forall(
            vec![msgv("x"), nodev("i")],
            and(
                action("A", vec![mvar("x")], "i"),
                action("A", vec![mvar("x")], "i"),
            ),
        ),
    );
    let err = [
        "universal quantifier without toplevel implication",
        "  \"\u{2200} x #i. (A( x ) @ #i) \u{2227} (A( x ) @ #i)\"",
        "in the formula",
        "  \"\u{2200} x #i. (A( x ) @ #i) \u{2227} (A( x ) @ #i)\"",
    ]
    .join("\n");
    let expected = [
        "lemma r1_forall_and:",
        "  all-traces \"\u{2200} x #i. (A( x ) @ #i) \u{2227} (A( x ) @ #i)\"",
        "/*",
        "conversion to guarded formula failed:",
        "  universal quantifier without toplevel implication",
        "    \"\u{2200} x #i. (A( x ) @ #i) \u{2227} (A( x ) @ #i)\"",
        "  in the formula",
        "    \"\u{2200} x #i. (A( x ) @ #i) \u{2227} (A( x ) @ #i)\"",
        "*/",
        "by sorry",
    ]
    .join("\n");
    assert_eq!(render_lemma(&l, Some(&Guarded::Failed(err))), expected);
}

#[test]
fn binder_list_wrap_and_body_indent() {
    // probe:q_l2 bw1 — binders fill-wrap aligned after `∀ `, body at
    // (quantifier origin + 1), fact args fill after `Name( ` with the `)`
    // dropping to the fact's column and `@ #i` beside it, `(⊤)` conclusion.
    let names = [
        "xlongvariablename01",
        "xlongvariablename02",
        "xlongvariablename03",
        "xlongvariablename04",
    ];
    let mut vs: Vec<VarSpec> = names.iter().map(|n| msgv(n)).collect();
    vs.push(nodev("i"));
    let l = lemma(
        "bw1",
        TraceQuantifier::AllTraces,
        forall(
            vs,
            implies(
                action("E", names.iter().map(|n| mvar(n)).collect(), "i"),
                Formula::True,
            ),
        ),
    );
    let expected = [
        "lemma bw1:",
        "  all-traces",
        "  \"\u{2200} xlongvariablename01 xlongvariablename02 xlongvariablename03",
        "     xlongvariablename04 #i.",
        "    (E( xlongvariablename01, xlongvariablename02, xlongvariablename03,",
        "        xlongvariablename04",
        "     ) @ #i) \u{21d2}",
        "    (\u{22a4})\"",
        "by sorry",
    ]
    .join("\n");
    assert_eq!(render_lemma(&l, None), expected);
}

#[test]
fn action_atom_breaks_inside_fact() {
    // probe:q_l5 m63 — the action atom is an hsep: at overflow the FACT
    // breaks internally and `) @ #i` stays joined; the closing quote counts
    // against the ribbon.
    let name = format!("F{}", "w".repeat(62));
    let l = lemma(
        "m63",
        TraceQuantifier::ExistsTrace,
        exists(
            vec![msgv("x"), nodev("i")],
            action(&name, vec![mvar("x")], "i"),
        ),
    );
    let expected = [
        "lemma m63:".to_string(),
        "  exists-trace".to_string(),
        "  \"\u{2203} x #i.".to_string(),
        format!("    {name}( x"),
        "    ) @ #i\"".to_string(),
        "by sorry".to_string(),
    ]
    .join("\n");
    assert_eq!(render_lemma(&l, None), expected);
}

#[test]
fn relation_atom_break() {
    // probe:q_l4 tw1 — `=` attaches to the lhs line, the rhs drops to the
    // atom origin; the rhs tuple wraps by the R1 pair law (trailing ", ").
    let long = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let l = lemma(
        "tw1",
        TraceQuantifier::AllTraces,
        forall(
            vec![msgv("a"), msgv("b"), msgv("c"), nodev("i")],
            implies(
                action("E", vec![mvar("a"), mvar("b"), mvar("c"), mvar("c")], "i"),
                eq(
                    mvar("a"),
                    Term::Pair(vec![
                        mvar("b"),
                        mvar("c"),
                        Term::PubLit(long.into()),
                        mvar("b"),
                        mvar("c"),
                        mvar("b"),
                        mvar("c"),
                    ]),
                ),
            ),
        ),
    );
    let expected = [
        "lemma tw1:",
        "  all-traces",
        "  \"\u{2200} a b c #i.",
        "    (E( a, b, c, c ) @ #i) \u{21d2}",
        "    (a =",
        "     <b, c, 'aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa', b, ",
        "      c, b, c>)\"",
        "by sorry",
    ]
    .join("\n");
    assert_eq!(render_lemma(&l, None), expected);
}

#[test]
fn deep_connective_nesting() {
    // probe:q_p2 s13 — ¬/∨/∧ nesting with universal operand parens; the
    // statement keyword line breaks while the formula body fills.
    let cl = |s: &str| Term::PubLit(s.into());
    let f = exists(
        vec![msgv("x"), nodev("i")],
        and(
            action("A", vec![mvar("x")], "i"),
            and(
                not(exists(vec![nodev("j")], action("C", vec![], "j"))),
                or(eq(mvar("x"), cl("c")), not(eq(mvar("x"), cl("d")))),
            ),
        ),
    );
    let l = lemma("s13_deep", TraceQuantifier::ExistsTrace, f);
    let expected = [
        "lemma s13_deep:",
        "  exists-trace",
        "  \"\u{2203} x #i.",
        "    (A( x ) @ #i) \u{2227} ((\u{ac}(\u{2203} #j. C( ) @ #j)) \u{2227} ((x = 'c') \u{2228} (\u{ac}(x = 'd'))))\"",
        "by sorry",
    ]
    .join("\n");
    assert_eq!(render_lemma(&l, None), expected);
}

// ── the layout-insensitive echo parser (test harness only) ──────────────────
//
// Parses rendered restriction/lemma blocks back into the crate AST. It skips
// ALL whitespace between tokens, so the re-rendered layout can only come
// from the renderer under test, never from the capture. Guarded content and
// proof scripts are extracted line-wise as VERBATIM opaque inputs (they are
// inputs to the renderer, not its output).

struct P<'a> {
    s: &'a str,
    pos: usize,
}

impl<'a> P<'a> {
    fn new(s: &'a str) -> Self {
        P { s, pos: 0 }
    }

    fn rest(&self) -> &'a str {
        &self.s[self.pos..]
    }

    fn ws(&mut self) {
        while let Some(c) = self.rest().chars().next() {
            if c.is_whitespace() {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
    }

    fn starts(&mut self, tok: &str) -> bool {
        self.ws();
        self.rest().starts_with(tok)
    }

    fn eat(&mut self, tok: &str) -> bool {
        if self.starts(tok) {
            self.pos += tok.len();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, tok: &str) {
        assert!(
            self.eat(tok),
            "expected {tok:?} at …{:?}",
            &self.rest()[..self.rest().len().min(80)]
        );
    }

    fn ident(&mut self) -> String {
        self.ws();
        let s: String = self
            .rest()
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        assert!(
            !s.is_empty(),
            "expected ident at …{:?}",
            &self.rest()[..self.rest().len().min(60)]
        );
        self.pos += s.len();
        s
    }

    fn digits(&mut self) -> String {
        let s: String = self
            .rest()
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        self.pos += s.len();
        s
    }

    /// `name` or `name.idx` as a (name, idx) pair.
    fn name_idx(&mut self) -> (String, u64) {
        let name = self.ident();
        if self.rest().starts_with('.')
            && self.s[self.pos + 1..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit())
        {
            self.pos += 1;
            let idx = self.digits().parse().unwrap();
            (name, idx)
        } else {
            (name, 0)
        }
    }

    fn quoted(&mut self) -> String {
        self.expect("'");
        let end = self.rest().find('\'').expect("unterminated quote");
        let s = self.rest()[..end].to_string();
        self.pos += end + 1;
        s
    }

    // ── terms (as in round-2, plus nullary-application awareness) ──

    fn term(&mut self) -> Term {
        let mut t = self.term_atom();
        while self.starts("^") {
            self.pos += 1;
            let rhs = self.term_atom();
            t = Term::BinOp(BinOp::Exp, Box::new(t), Box::new(rhs));
        }
        t
    }

    fn sorted_var(&mut self, sort: SortHint) -> Term {
        let (name, idx) = self.name_idx();
        Term::Var(VarSpec {
            name,
            idx,
            sort,
            typ: None,
        })
    }

    fn term_atom(&mut self) -> Term {
        self.ws();
        let c = self.rest().chars().next().expect("term expected");
        match c {
            '\'' => Term::PubLit(self.quoted()),
            '~' => {
                self.pos += 1;
                if self.rest().starts_with('\'') {
                    Term::FreshLit(self.quoted())
                } else {
                    self.sorted_var(SortHint::Fresh)
                }
            }
            '$' => {
                self.pos += 1;
                self.sorted_var(SortHint::Pub)
            }
            '#' => {
                self.pos += 1;
                self.sorted_var(SortHint::Node)
            }
            '%' => {
                self.pos += 1;
                if self
                    .rest()
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_ascii_digit())
                {
                    Term::NatLit(self.digits())
                } else {
                    self.sorted_var(SortHint::Nat)
                }
            }
            '<' => {
                self.pos += 1;
                let mut elems = vec![self.term()];
                while self.eat(",") {
                    elems.push(self.term());
                }
                self.expect(">");
                Term::Pair(elems)
            }
            '(' => {
                self.pos += 1;
                let first = self.term();
                let op = if self.starts("++") {
                    BinOp::Union
                } else if self.starts("%+") {
                    BinOp::NatPlus
                } else if self.starts("\u{2295}") {
                    BinOp::Xor
                } else if self.starts("*") {
                    BinOp::Mult
                } else {
                    panic!(
                        "AC operator expected at …{:?}",
                        &self.rest()[..self.rest().len().min(60)]
                    );
                };
                let glyph = match op {
                    BinOp::Union => "++",
                    BinOp::NatPlus => "%+",
                    BinOp::Xor => "\u{2295}",
                    BinOp::Mult => "*",
                    BinOp::Exp => unreachable!(),
                };
                let mut t = first;
                while self.eat(glyph) {
                    let rhs = self.term();
                    t = Term::BinOp(op, Box::new(t), Box::new(rhs));
                }
                self.expect(")");
                t
            }
            _ => {
                let (name, idx) = self.name_idx();
                if self.rest().starts_with('(') {
                    // Application: nullary symbols render bare, so an
                    // argument list is always non-empty here (a nullary
                    // `Name( )` at formula-atom position is a FACT and is
                    // handled by `formula_atom`).
                    self.pos += 1;
                    let mut args = vec![self.term()];
                    while self.eat(",") {
                        args.push(self.term());
                    }
                    self.expect(")");
                    Term::App(name, args)
                } else {
                    Term::Var(VarSpec {
                        name,
                        idx,
                        sort: SortHint::Untagged,
                        typ: None,
                    })
                }
            }
        }
    }

    // ── formulas ──

    fn binder(&mut self) -> VarSpec {
        self.ws();
        let sort = if self.eat("~") {
            SortHint::Fresh
        } else if self.eat("$") {
            SortHint::Pub
        } else if self.eat("#") {
            SortHint::Node
        } else if self.eat("%") {
            SortHint::Nat
        } else {
            SortHint::Untagged
        };
        let (name, idx) = self.name_idx();
        VarSpec {
            name,
            idx,
            sort,
            typ: None,
        }
    }

    fn quantifier(&mut self, glyph: &str) -> Formula {
        // Called with the glyph already known to be next.
        self.expect(glyph);
        let mut vs = Vec::new();
        loop {
            self.ws();
            if self.rest().starts_with('.') {
                self.pos += 1;
                break;
            }
            vs.push(self.binder());
        }
        let body = self.formula();
        if glyph == "\u{2200}" {
            Formula::Forall(vs, Box::new(body))
        } else {
            Formula::Exists(vs, Box::new(body))
        }
    }

    fn formula(&mut self) -> Formula {
        self.ws();
        if self.rest().starts_with('\u{2200}') {
            return self.quantifier("\u{2200}");
        }
        if self.rest().starts_with('\u{2203}') {
            return self.quantifier("\u{2203}");
        }
        let l = self.funit();
        self.ws();
        for (glyph, ctor) in [
            ("\u{2227}", Formula::And as fn(_, _) -> _),
            ("\u{2228}", Formula::Or as fn(_, _) -> _),
            ("\u{21d2}", Formula::Implies as fn(_, _) -> _),
            ("\u{21d4}", Formula::Iff as fn(_, _) -> _),
        ] {
            if self.eat(glyph) {
                let r = self.funit();
                // The statement printer parenthesizes every operand, so a
                // second connective at the same level cannot occur.
                self.ws();
                for g in ["\u{2227}", "\u{2228}", "\u{21d2}", "\u{21d4}"] {
                    assert!(
                        !self.rest().starts_with(g),
                        "unparenthesized connective chain at …{:?}",
                        &self.rest()[..self.rest().len().min(60)]
                    );
                }
                return ctor(Box::new(l), Box::new(r));
            }
        }
        l
    }

    /// One operand-position formula unit.
    fn funit(&mut self) -> Formula {
        self.ws();
        if self.eat("\u{22a4}") {
            return Formula::True;
        }
        if self.eat("\u{22a5}") {
            return Formula::False;
        }
        if self.eat("\u{ac}") {
            self.expect("(");
            let f = self.formula();
            self.expect(")");
            return Formula::Not(Box::new(f));
        }
        if self.rest().starts_with('\u{2200}') {
            return self.quantifier("\u{2200}");
        }
        if self.rest().starts_with('\u{2203}') {
            return self.quantifier("\u{2203}");
        }
        if self.rest().starts_with('(') {
            // Ambiguity: `(` opens either a formula group or a
            // self-parenthesized AC term (`(x++z) = y`). A formula group is
            // followed by a connective or a closer; a term paren is followed
            // by a relation glyph. Decide by matching the paren.
            if !self.paren_is_term() {
                self.pos += 1;
                let f = self.formula();
                self.expect(")");
                return f;
            }
        }
        self.formula_atom()
    }

    /// At a `(`: does this parenthesized span read as a TERM (its matching
    /// `)` is followed by a relation glyph) rather than a formula group?
    fn paren_is_term(&self) -> bool {
        let bytes = self.rest();
        let mut depth = 0usize;
        let mut it = bytes.char_indices();
        let close_end = loop {
            let Some((i, c)) = it.next() else { return false };
            match c {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        break i + 1;
                    }
                }
                _ => {}
            }
        };
        let after = bytes[close_end..].trim_start();
        after.starts_with('=')
            || after.starts_with('<')
            || after.starts_with('\u{228f}')
            || after.starts_with('@')
    }

    fn fact(&mut self) -> Fact {
        self.ws();
        let persistent = self.eat("!");
        let name = self.ident();
        self.expect("(");
        self.ws();
        let mut args = Vec::new();
        if !self.rest().starts_with(')') {
            args.push(self.term());
            while self.eat(",") {
                args.push(self.term());
            }
        }
        self.expect(")");
        Fact {
            persistent,
            name,
            args,
            annotations: vec![],
        }
    }

    fn formula_atom(&mut self) -> Formula {
        self.ws();
        if self.rest().starts_with("last(") {
            self.pos += "last(".len();
            let t = self.term();
            self.expect(")");
            return Formula::Atom(Atom::Last(t));
        }
        if self.rest().starts_with('!') {
            let f = self.fact();
            self.expect("@");
            let tp = self.term();
            return Formula::Atom(Atom::Action(f, tp));
        }
        // Nullary fact `Name( )` cannot be parsed as a term; detect the
        // `ident( )` shape up front.
        let save = self.pos;
        let mut is_nullary_fact = false;
        if self.rest().chars().next().is_some_and(|c| c.is_ascii_alphanumeric() || c == '_') {
            let _ = self.ident();
            if self.starts("(") {
                self.pos += 1;
                if self.starts(")") {
                    is_nullary_fact = true;
                }
            }
        }
        self.pos = save;
        if is_nullary_fact {
            let f = self.fact();
            self.expect("@");
            let tp = self.term();
            return Formula::Atom(Atom::Action(f, tp));
        }
        let t = self.term();
        self.ws();
        if self.eat("@") {
            let (name, args) = match t {
                Term::App(name, args) => (name, args),
                Term::Var(v) => {
                    assert_eq!(v.idx, 0, "action fact head with index");
                    (v.name, vec![])
                }
                other => panic!("action atom on non-fact term {other:?}"),
            };
            let tp = self.term();
            return Formula::Atom(Atom::Action(
                Fact {
                    persistent: false,
                    name,
                    args,
                    annotations: vec![],
                },
                tp,
            ));
        }
        if self.eat("=") {
            let r = self.term();
            return Formula::Atom(Atom::Eq(t, r));
        }
        if self.eat("\u{228f}") {
            let r = self.term();
            return Formula::Atom(Atom::Subterm(t, r));
        }
        if self.eat("<") {
            let r = self.term();
            return Formula::Atom(Atom::Less(t, r));
        }
        panic!(
            "relation glyph expected at …{:?}",
            &self.rest()[..self.rest().len().min(60)]
        );
    }

    // ── wrappers ──

    fn lemma_attrs(&mut self) -> Vec<LemmaAttr> {
        let mut attrs = Vec::new();
        loop {
            self.ws();
            if self.eat("sources") {
                attrs.push(LemmaAttr::Sources);
            } else if self.eat("reuse") {
                attrs.push(LemmaAttr::Reuse);
            } else if self.eat("use_induction") {
                attrs.push(LemmaAttr::UseInduction);
            } else if self.eat("hide_lemma=") {
                attrs.push(LemmaAttr::HideLemma(self.ident()));
            } else if self.eat("heuristic=") {
                let v: String = self
                    .rest()
                    .chars()
                    .take_while(|c| *c != ',' && *c != ']')
                    .collect();
                self.pos += v.len();
                attrs.push(LemmaAttr::Heuristic(v));
            } else {
                panic!(
                    "lemma attribute expected at …{:?}",
                    &self.rest()[..self.rest().len().min(60)]
                );
            }
            if !self.eat(",") {
                break;
            }
        }
        attrs
    }
}

// ── block extraction & parsing ──────────────────────────────────────────────

/// Slice a capture into its restriction and lemma blocks (verbatim lines).
fn item_blocks(capture: &str) -> (Vec<String>, Vec<String>) {
    let lines: Vec<&str> = capture.lines().collect();
    let mut restrictions = Vec::new();
    let mut lemmas = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].starts_with("restriction ") {
            let start = i;
            while lines[i] != "  */" {
                i += 1;
            }
            restrictions.push(lines[start..=i].join("\n"));
        } else if lines[i].starts_with("lemma ") {
            let start = i;
            // Through the guarded comment's closing `*/` at column 0.
            while lines[i] != "*/" {
                i += 1;
            }
            // Tail: a single `by …` line, or an embedded proof script that
            // runs through its column-0 `qed`.
            i += 1;
            if lines[i].starts_with("by ") {
                // single-line tail
            } else {
                while lines[i] != "qed" {
                    i += 1;
                }
            }
            lemmas.push(lines[start..=i].join("\n"));
        }
        i += 1;
    }
    (restrictions, lemmas)
}

fn parse_restriction(block: &str) -> Restriction {
    // Statement part: everything before the blank line ahead of the
    // expanded-formula comment.
    let stmt_end = block.find("\n\n").expect("restriction comment separator");
    let stmt = &block[..stmt_end];
    let mut p = P::new(stmt);
    p.expect("restriction");
    p.ws();
    let name: String = p.rest().chars().take_while(|c| *c != ':').collect();
    p.pos += name.len();
    p.expect(":");
    p.expect("\"");
    let f = p.formula();
    p.expect("\"");
    let had_safety = p.eat("// safety formula");
    p.ws();
    assert!(p.rest().is_empty(), "unparsed statement: {:?}", p.rest());
    let r = Restriction { name, formula: f };
    // The renderer derives the safety line from the formula; cross-check the
    // classification against the capture here so a parity PASS can't hide a
    // misclassification compensated elsewhere.
    assert_eq!(
        pretty_clean::render_restriction(&r).contains("\n  // safety formula"),
        had_safety,
        "safety classification for {}",
        r.name
    );
    r
}

fn parse_lemma(block: &str) -> (Lemma, Option<Guarded>) {
    let lines: Vec<&str> = block.lines().collect();
    // The guarded comment opens at the first column-0 `/*`.
    let open = lines
        .iter()
        .position(|l| *l == "/*")
        .expect("guarded comment opener");
    let close = open + 1 + lines[open + 1..]
        .iter()
        .position(|l| *l == "*/")
        .expect("guarded comment closer");
    let head = lines[..open].join("\n");
    let mut p = P::new(&head);
    p.expect("lemma");
    p.ws();
    let name: String = p
        .rest()
        .chars()
        .take_while(|c| !c.is_whitespace() && *c != '[' && *c != ':')
        .collect();
    p.pos += name.len();
    p.ws();
    let attributes = if p.eat("[") {
        let a = p.lemma_attrs();
        p.expect("]");
        a
    } else {
        Vec::new()
    };
    p.expect(":");
    let trace_quantifier = if p.eat("all-traces") {
        TraceQuantifier::AllTraces
    } else {
        p.expect("exists-trace");
        TraceQuantifier::ExistsTrace
    };
    p.expect("\"");
    let formula = p.formula();
    p.expect("\"");
    p.ws();
    assert!(p.rest().is_empty(), "unparsed lemma head: {:?}", p.rest());

    // Guarded comment interior: header line + verbatim content.
    let header = lines[open + 1];
    let content = lines[open + 2..close].join("\n");
    let guarded = if header == "conversion to guarded formula failed:" {
        let stripped: Vec<String> = lines[open + 2..close]
            .iter()
            .map(|l| l.strip_prefix("  ").unwrap_or(l).to_string())
            .collect();
        Guarded::Failed(stripped.join("\n"))
    } else {
        let expected_header = match trace_quantifier {
            TraceQuantifier::AllTraces => "guarded formula characterizing all counter-examples:",
            TraceQuantifier::ExistsTrace => "guarded formula characterizing all satisfying traces:",
        };
        assert_eq!(header, expected_header, "guarded header for {name}");
        Guarded::Formula(content)
    };

    // Tail: `by sorry` or a verbatim embedded proof.
    let tail = lines[close + 1..].join("\n");
    let proof = if tail == "by sorry" { None } else { Some(tail) };
    (
        Lemma {
            name,
            attributes,
            trace_quantifier,
            formula,
            proof,
        },
        Some(guarded),
    )
}

// ── whole-block parity over the round-3 curated captures ────────────────────

#[test]
fn parity_formula_blocks_match_captures() {
    // Deep formula nesting recurses; run on a wide stack like round 2.
    std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(parity_formula_blocks_impl)
        .unwrap()
        .join()
        .unwrap()
}

fn parity_formula_blocks_impl() {
    // Sealed-workspace location of the curated round-3 captures; self-skips
    // once the crate moves at integration (the corpus gate takes over).
    let dir = std::path::Path::new("../../round3/targets");
    if !dir.is_dir() {
        return;
    }
    let mut checked_r = 0;
    let mut checked_l = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("txt") {
            continue;
        }
        let capture = std::fs::read_to_string(&path).unwrap();
        let (restrictions, lemmas) = item_blocks(&capture);
        for block in &restrictions {
            let r = parse_restriction(block);
            assert_eq!(
                &render_restriction(&r),
                block,
                "restriction-block divergence: {} ({})",
                path.display(),
                r.name
            );
            checked_r += 1;
        }
        for block in &lemmas {
            let (l, g) = parse_lemma(block);
            assert_eq!(
                &render_lemma(&l, g.as_ref()),
                block,
                "lemma-block divergence: {} ({})",
                path.display(),
                l.name
            );
            checked_l += 1;
        }
    }
    println!("parity: {checked_r} restriction blocks / {checked_l} lemma blocks byte-checked");
    assert!(
        checked_r > 40 && checked_l > 100,
        "expected many blocks, got {checked_r} restrictions / {checked_l} lemmas"
    );
}
