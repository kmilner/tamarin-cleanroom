//! R5 — whole-theory FRAME parity (GAP 3).
//!
//! Verifies `render_theory` assembles the `theory <name> begin … end` echo
//! byte-for-byte across a diverse corpus. Like rounds 1–3 this is a
//! reconstruction test: a layout-INSENSITIVE echo parser turns each whole
//! capture back into the crate's `Theory` + `Signature` (discarding ALL
//! whitespace/layout, so the re-rendered bytes can only come from the renderer
//! under test), then `render_theory` must reproduce the capture exactly.
//!
//! The frame's contribution — item ORDER, header/footer, the inter-block blank
//! rhythm, and the 3-blank pre-`end` residue of the stripped wf/stamp comments
//! — is what this test pins; each interior block still renders through the
//! R1–R4 entry points (`render_signature_block`, `render_macros` /
//! `render_predicates`, `render_rule`, `render_restriction` / `render_lemma`).
//! Opaque top-level blocks (the `tactic:` region, `section{* … *}` comments)
//! are carried verbatim, as their interior layout is out of the erasure
//! surface (BEHAVIOR.md "Theory frame").

use pretty_clean::ast::*;
use pretty_clean::{
    render_lemma, render_macros, render_predicates, render_restriction, render_rule,
    render_signature_block, render_theory,
};

// ── focused per-gap unit tests (hand-built fixtures) ────────────────────────

fn msgv(name: &str) -> VarSpec {
    VarSpec { name: name.into(), idx: 0, sort: SortHint::Untagged, typ: None }
}
fn mac(name: &str, params: Vec<Term>, body: Term) -> Macro {
    Macro { name: name.into(), params, body }
}

#[test]
fn gap2_macros_block_always_breaks() {
    // GAP 2 (probe:r5_mac2) — the macros block ALWAYS breaks: the first macro
    // sits beside `macros: `, every subsequent macro on its own line at col 8,
    // even when the whole list would fit one line. Commas attach to the
    // preceding macro; the last carries none.
    let two = vec![
        mac("aa", vec![Term::Var(msgv("x"))], Term::App("h".into(), vec![Term::Var(msgv("x"))])),
        mac("bb", vec![Term::Var(msgv("x"))], Term::Var(msgv("x"))),
    ];
    assert_eq!(
        render_macros(&two),
        "macros: aa( x ) =  h(x),\n        bb( x ) =  x"
    );
    // A single macro is trivially one line (probe:r5_mac1, target:issue777).
    let one = vec![mac("onlyone", vec![Term::Var(msgv("x"))], Term::App("h".into(), vec![Term::Var(msgv("x"))]))];
    assert_eq!(render_macros(&one), "macros: onlyone( x ) =  h(x)");
}

#[test]
fn predicate_one_liner_and_no_spaces_around_iff() {
    // target:features_predicates_minimal / timepoints — `predicate: Name( args
    // )<=>formula`, no spaces around `<=>`, fact-style head, formula body.
    let p = Predicate {
        name: "True".into(),
        params: vec![msgv("x")],
        body: Formula::Atom(Atom::Eq(Term::Var(msgv("x")), Term::App("true".into(), vec![]))),
    };
    assert_eq!(render_predicates(std::slice::from_ref(&p)), "predicate: True( x )<=>x = true");

    // A group renders blank-line separated.
    let p2 = Predicate {
        name: "IsNormal".into(),
        params: vec![msgv("a")],
        body: Formula::Atom(Atom::Eq(Term::Var(msgv("a")), Term::App("NormalReq".into(), vec![]))),
    };
    assert_eq!(
        render_predicates(&[p.clone(), p2]),
        "predicate: True( x )<=>x = true\n\npredicate: IsNormal( a )<=>a = NormalReq"
    );
}

#[test]
fn predicate_wrapping_body_wraps_at_margin_independent_of_header() {
    // target:dmn-basic — a wrapping predicate body wraps at column 1 (the
    // formula's own nesting from the margin), the SAME column regardless of the
    // header width: the body is rendered at absolute margin 0 and the header is
    // textually prepended to its first line. Build one ∃-body under two heads of
    // different lengths and assert the body columns match.
    let body = || {
        let mv = |n: &str| Term::Var(msgv(n));
        let nv = |n: &str| Term::Var(VarSpec { name: n.into(), idx: 0, sort: SortHint::Node, typ: None });
        let post = |a: Term, b: Term, tp: &str| {
            Formula::Atom(Atom::Action(
                Fact { persistent: false, name: "Post".into(), args: vec![a, b, mv("c")], annotations: vec![] },
                nv(tp),
            ))
        };
        let and = |a, b| Formula::And(Box::new(a), Box::new(b));
        let imp = |a, b| Formula::Implies(Box::new(a), Box::new(b));
        let eq = |a, b| Formula::Atom(Atom::Eq(a, b));
        let inner = imp(
            and(eq(mv("sa2"), Term::Pair(vec![mv("sid"), mv("s2")])), post(mv("sid"), mv("s1"), "i")),
            post(mv("sid"), mv("s2"), "j"),
        );
        Formula::Exists(
            vec![msgv("sid"), msgv("s1"), msgv("s2"), msgv("c"),
                 VarSpec { name: "i".into(), idx: 0, sort: SortHint::Node, typ: None },
                 VarSpec { name: "j".into(), idx: 0, sort: SortHint::Node, typ: None }],
            Box::new(inner),
        )
    };
    let short = Predicate { name: "P".into(), params: vec![msgv("sa2")], body: body() };
    let long = Predicate { name: "PredicateWithAVeryLongName".into(), params: vec![msgv("sa2")], body: body() };
    let body_col = |s: &str| {
        // The body is the first line NOT containing the `<=>` header.
        let line = s.lines().nth(1).unwrap();
        line.len() - line.trim_start().len()
    };
    let (rs, rl) = (render_predicates(std::slice::from_ref(&short)), render_predicates(std::slice::from_ref(&long)));
    assert!(rs.lines().count() > 1, "the fixture body must wrap");
    assert_eq!(body_col(&rs), body_col(&rl), "body column must be independent of header width");
    assert_eq!(body_col(&rs), 1, "wrapping body sits at column 1 (margin nest 1)");
}

#[test]
fn gap3_frame_glue_order_spacing_and_tail() {
    // GAP 3 — `render_theory` contributes ONLY the header/footer, item ORDER and
    // the blank-line rhythm; every block still renders through its R1–R4 entry
    // point. Assert render_theory == the sub-renderers joined by single blank
    // lines, signature first, with the 3-blank pre-`end` tail. Item types cover
    // heuristic, macros, a rule (trivial variant), a predicate group, a
    // restriction, a lemma, and an opaque verbatim block.
    let sig = Signature {
        builtins: vec![],
        functions: vec![],
        equations: vec![],
        convergent: false,
    };
    let rule = Rule {
        name: "R".into(),
        modulo: Some("E".into()),
        attributes: vec![],
        premises: vec![],
        actions: vec![],
        conclusions: vec![],
        loop_breakers: vec![],
    };
    let pred = Predicate {
        name: "True".into(),
        params: vec![msgv("x")],
        body: Formula::Atom(Atom::Eq(Term::Var(msgv("x")), Term::App("true".into(), vec![]))),
    };
    let restr = Restriction {
        name: "Ex".into(),
        formula: Formula::Exists(
            vec![VarSpec { name: "i".into(), idx: 0, sort: SortHint::Node, typ: None }],
            Box::new(Formula::Atom(Atom::Action(
                Fact { persistent: false, name: "A".into(), args: vec![], annotations: vec![] },
                Term::Var(VarSpec { name: "i".into(), idx: 0, sort: SortHint::Node, typ: None }),
            ))),
        ),
        expanded: Formula::Exists(
            vec![VarSpec { name: "i".into(), idx: 0, sort: SortHint::Node, typ: None }],
            Box::new(Formula::Atom(Atom::Action(
                Fact { persistent: false, name: "A".into(), args: vec![], annotations: vec![] },
                Term::Var(VarSpec { name: "i".into(), idx: 0, sort: SortHint::Node, typ: None }),
            ))),
        ),
    };
    let lemma = Lemma {
        name: "Bar".into(),
        attributes: vec![],
        trace_quantifier: TraceQuantifier::AllTraces,
        formula: Formula::True,
        proof: None,
    };
    let guarded = Guarded::Formula("\"\u{22a4}\"".into());
    let verbatim = "section{* a note *}".to_string();
    let mm = vec![mac("m", vec![Term::Var(msgv("x"))], Term::Var(msgv("x")))];

    let thy = Theory {
        name: "Demo".into(),
        items: vec![
            TheoryItem::Heuristic("p".into()),
            TheoryItem::Macros(mm.clone()),
            TheoryItem::Rule(rule.clone(), None),
            TheoryItem::Predicates(vec![pred.clone()]),
            TheoryItem::Verbatim(verbatim.clone()),
            TheoryItem::Restriction(restr.clone()),
            TheoryItem::Lemma(lemma.clone(), Some(guarded.clone())),
        ],
    };

    let expected = format!(
        "theory Demo\n\nbegin\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n{}\n\n\n\nend",
        render_signature_block(&sig),
        "heuristic: p",
        render_macros(&mm),
        render_rule(&rule, None),
        render_predicates(std::slice::from_ref(&pred)),
        verbatim,
        render_restriction(&restr),
        render_lemma(&lemma, Some(&guarded)),
    );
    assert_eq!(render_theory(&thy, &sig), expected);

    // Structural spot-checks on the frame itself.
    let out = render_theory(&thy, &sig);
    assert!(out.starts_with("theory Demo\n\nbegin\n\n"), "header rhythm");
    assert!(out.ends_with("by sorry\n\n\n\nend"), "3-blank pre-end tail");
    // Signature is first (immediately after `begin`), items keep source order.
    let sig_at = out.find("// Function signature").unwrap();
    let heur_at = out.find("heuristic: p").unwrap();
    let rule_at = out.find("rule (modulo E) R").unwrap();
    let lem_at = out.find("lemma Bar").unwrap();
    assert!(sig_at < heur_at && heur_at < rule_at && rule_at < lem_at, "item order");
}

// ── the layout-insensitive echo parser (test harness only) ──────────────────
//
// Merges the round-2 rule parser (terms, annotated facts, rule bodies,
// AC-variant comments) with the round-3 formula parser (∀ ∃ ⇒ ∧ ∨ ¬ atoms,
// restriction/lemma wrappers). It never consults layout — only token order.

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
            &self.rest()[..self.rest().len().min(70)]
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
            &self.rest()[..self.rest().len().min(50)]
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

    // ── terms ──
    fn term(&mut self) -> Term {
        let mut t = self.atom();
        while self.starts("^") {
            self.pos += 1;
            let rhs = self.atom();
            t = Term::BinOp(BinOp::Exp, Box::new(t), Box::new(rhs));
        }
        t
    }
    fn sorted_var(&mut self, sort: SortHint) -> Term {
        let (name, idx) = self.name_idx();
        Term::Var(VarSpec { name, idx, sort, typ: None })
    }
    fn atom(&mut self) -> Term {
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
                if self.rest().chars().next().is_some_and(|c| c.is_ascii_digit()) {
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
                    panic!("AC operator expected at …{:?}", &self.rest()[..self.rest().len().min(50)]);
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
                    self.pos += 1;
                    let mut args = vec![self.term()];
                    while self.eat(",") {
                        args.push(self.term());
                    }
                    self.expect(")");
                    Term::App(name, args)
                } else {
                    Term::Var(VarSpec { name, idx, sort: SortHint::Untagged, typ: None })
                }
            }
        }
    }

    // ── facts (with rule-fact annotations) ──
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
        let mut annotations = Vec::new();
        if self.rest().starts_with('[') {
            self.pos += 1;
            loop {
                self.ws();
                if self.eat("+") {
                    annotations.push(FactAnnotation::SolveFirst);
                } else if self.eat("-") {
                    annotations.push(FactAnnotation::SolveLast);
                } else if self.eat("no_precomp") {
                    annotations.push(FactAnnotation::NoSources);
                } else {
                    panic!("annotation expected");
                }
                if !self.eat(",") {
                    break;
                }
            }
            self.expect("]");
        }
        Fact { persistent, name, args, annotations }
    }
    fn fact_seq(&mut self, close: &str) -> Vec<Fact> {
        self.ws();
        let mut facts = Vec::new();
        if !self.starts(close) {
            facts.push(self.fact());
            while self.eat(",") {
                facts.push(self.fact());
            }
        }
        self.expect(close);
        facts
    }

    // ── rules ──
    fn rule_core(&mut self) -> Rule {
        self.expect("rule");
        self.ws();
        let modulo = if self.eat("(modulo") {
            let m = self.ident();
            self.expect(")");
            Some(m)
        } else {
            None
        };
        self.ws();
        let name: String = self
            .rest()
            .chars()
            .take_while(|c| !matches!(c, '[' | ':') && !c.is_whitespace())
            .collect();
        self.pos += name.len();
        let mut attributes = Vec::new();
        if self.rest().starts_with('[') {
            self.pos += 1;
            loop {
                self.ws();
                if self.eat("color=#") {
                    let v: String = self.rest().chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
                    self.pos += v.len();
                    attributes.push(RuleAttr::Color(v));
                } else if self.eat("process=\"") {
                    let end = self.rest().find('"').expect("unterminated process=");
                    attributes.push(RuleAttr::Process(self.rest()[..end].into()));
                    self.pos += end + 1;
                } else if self.eat("no_derivcheck") {
                    attributes.push(RuleAttr::NoDerivCheck);
                } else if self.eat("issapicrule") {
                    attributes.push(RuleAttr::IsSapicRule);
                } else if self.eat("role='") {
                    let end = self.rest().find('\'').unwrap();
                    attributes.push(RuleAttr::Role(self.rest()[..end].into()));
                    self.pos += end + 1;
                } else {
                    panic!("attribute expected at …{:?}", &self.rest()[..self.rest().len().min(50)]);
                }
                if !self.eat(",") {
                    break;
                }
            }
            self.expect("]");
        }
        self.expect(":");
        self.expect("[");
        let premises = self.fact_seq("]");
        self.ws();
        let actions = if self.eat("-->") {
            Vec::new()
        } else {
            self.expect("--[");
            self.fact_seq("]->")
        };
        self.expect("[");
        let conclusions = self.fact_seq("]");
        Rule { name, modulo, attributes, premises, actions, conclusions, loop_breakers: vec![] }
    }
    fn breaker_line(&mut self) -> Vec<usize> {
        self.expect("//");
        self.ws();
        assert!(self.eat("loop breakers:") || self.eat("loop breaker:"));
        self.expect("[");
        let mut ids = vec![self.digits().parse().unwrap()];
        while self.eat(",") {
            self.ws();
            ids.push(self.digits().parse().unwrap());
        }
        self.expect("]");
        ids
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
        VarSpec { name, idx, sort, typ: None }
    }
    fn quantifier(&mut self, glyph: &str) -> Formula {
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
                return ctor(Box::new(l), Box::new(r));
            }
        }
        l
    }
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
        if self.rest().starts_with('(') && !self.paren_is_term() {
            self.pos += 1;
            let f = self.formula();
            self.expect(")");
            return f;
        }
        self.formula_atom()
    }
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
        // Nullary fact `Name( )` at atom position is an action fact.
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
                Fact { persistent: false, name, args, annotations: vec![] },
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
        panic!("relation glyph expected at …{:?}", &self.rest()[..self.rest().len().min(50)]);
    }
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
                let v: String = self.rest().chars().take_while(|c| *c != ',' && *c != ']').collect();
                self.pos += v.len();
                attrs.push(LemmaAttr::Heuristic(v));
            } else {
                panic!("lemma attribute expected at …{:?}", &self.rest()[..self.rest().len().min(50)]);
            }
            if !self.eat(",") {
                break;
            }
        }
        attrs
    }
}

// ── block parsers ───────────────────────────────────────────────────────────

fn parse_rule_block(block: &str) -> (Rule, Option<AcVariants>) {
    let mut p = P::new(block);
    let mut r = p.rule_core();
    if p.starts("//") {
        r.loop_breakers = p.breaker_line();
    }
    p.expect("/*");
    if p.eat("has exactly the trivial AC variant") {
        p.expect("*/");
        return (r, None);
    }
    let mut ac = p.rule_core();
    let mut substitutions: Vec<Vec<(Term, Term)>> = Vec::new();
    if p.eat("variants (modulo AC)") {
        loop {
            if p.starts("*/") || p.starts("//") {
                break;
            }
            p.ws();
            if p.rest().chars().next().is_some_and(|c| c.is_ascii_digit()) {
                let n: usize = p.digits().parse().unwrap();
                p.expect(".");
                assert_eq!(n, substitutions.len() + 1, "group numbering");
                substitutions.push(Vec::new());
                continue;
            }
            let lhs = p.term();
            p.expect("=");
            let rhs = p.term();
            substitutions
                .last_mut()
                .expect("substitution before group index")
                .push((lhs, rhs));
        }
    }
    if p.starts("//") {
        ac.loop_breakers = p.breaker_line();
    }
    p.expect("*/");
    (r, Some(AcVariants { ac_rule: ac, substitutions }))
}

fn parse_restriction_block(block: &str) -> Restriction {
    let stmt_end = block.find("\n\n").expect("restriction comment separator");
    let stmt = &block[..stmt_end];
    let mut p = P::new(stmt);
    p.expect("restriction");
    p.ws();
    let name: String = p.rest().chars().take_while(|c| *c != ':').collect();
    p.pos += name.len();
    p.expect(":");
    p.expect("\"");
    let formula = p.formula();
    p.expect("\"");
    let _ = p.eat("// safety formula");
    let comment = &block[stmt_end..];
    let ex_hdr = "expanded formula:";
    let ex_idx = comment.find(ex_hdr).expect("expanded formula header");
    let mut pe = P::new(&comment[ex_idx + ex_hdr.len()..]);
    pe.expect("\"");
    let expanded = pe.formula();
    pe.expect("\"");
    Restriction { name, formula, expanded }
}

fn parse_lemma_block(block: &str) -> (Lemma, Option<Guarded>) {
    let lines: Vec<&str> = block.lines().collect();
    let open = lines.iter().position(|l| *l == "/*").expect("guarded comment opener");
    let close = open + 1 + lines[open + 1..].iter().position(|l| *l == "*/").expect("guarded closer");
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
    let header = lines[open + 1];
    let content = lines[open + 2..close].join("\n");
    let guarded = if header == "conversion to guarded formula failed:" {
        let stripped: Vec<String> = lines[open + 2..close]
            .iter()
            .map(|l| l.strip_prefix("  ").unwrap_or(l).to_string())
            .collect();
        Guarded::Failed(stripped.join("\n"))
    } else {
        Guarded::Formula(content)
    };
    let tail = lines[close + 1..].join("\n");
    let proof = if tail == "by sorry" { None } else { Some(tail) };
    (Lemma { name, attributes, trace_quantifier, formula, proof }, Some(guarded))
}

fn parse_macros_block(block: &str) -> Vec<Macro> {
    let mut p = P::new(block);
    p.expect("macros:");
    let mut out = Vec::new();
    loop {
        let name = p.ident();
        p.expect("(");
        let mut params = Vec::new();
        p.ws();
        if !p.rest().starts_with(')') {
            params.push(p.term());
            while p.eat(",") {
                params.push(p.term());
            }
        }
        p.expect(")");
        p.expect("=");
        let body = p.term();
        out.push(Macro { name, params, body });
        if !p.eat(",") {
            break;
        }
    }
    out
}

fn parse_predicate_block(block: &str) -> Predicate {
    let mut p = P::new(block);
    p.expect("predicate:");
    let name = p.ident();
    p.expect("(");
    let mut params = Vec::new();
    p.ws();
    if !p.rest().starts_with(')') {
        params.push(p.binder());
        while p.eat(",") {
            params.push(p.binder());
        }
    }
    p.expect(")");
    p.expect("<=>");
    let body = p.formula();
    Predicate { name, params, body }
}

/// Reconstruct the `Signature` from the declaration lines. builtins parsed from
/// the `builtins:` line (they induce no functions/equations, so they can't
/// perturb the merge); functions/equations parsed WHOLE and handed to the
/// renderer, which re-adds the base pairing symbols + equations, dedups and
/// re-sorts — reproducing the echo (the R1 closure is exercised on real input).
fn parse_signature(decls: &str) -> Signature {
    let lines: Vec<&str> = decls.lines().collect();
    let mut builtins = Vec::new();
    let mut functions = Vec::new();
    let mut equations = Vec::new();
    let mut convergent = false;
    let mut i = 0;
    while i < lines.len() {
        // A section = a col-0 header line + its indented continuation lines.
        let mut sect = String::from(lines[i]);
        let mut j = i + 1;
        while j < lines.len() && lines[j].starts_with(char::is_whitespace) {
            sect.push(' ');
            sect.push_str(lines[j].trim_start());
            j += 1;
        }
        if let Some(rest) = sect.strip_prefix("builtins:") {
            builtins = rest
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        } else if let Some(rest) = sect.strip_prefix("functions:") {
            functions = parse_function_items(rest);
        } else if let Some(rest) = sect.strip_prefix("equations") {
            let rest = rest.trim_start();
            let rest = if let Some(r) = rest.strip_prefix("[convergent]") {
                convergent = true;
                r.trim_start()
            } else {
                rest
            };
            let rest = rest.strip_prefix(':').expect("equations header colon");
            equations = parse_equations(rest);
        } else {
            panic!("unknown signature section: {sect:?}");
        }
        i = j;
    }
    Signature { builtins, functions, equations, convergent }
}

fn parse_function_items(s: &str) -> Vec<FunctionDecl> {
    let mut p = P::new(s);
    let mut out = Vec::new();
    loop {
        p.ws();
        if p.rest().is_empty() {
            break;
        }
        let name: String = p.rest().chars().take_while(|c| *c != '/').collect();
        p.pos += name.len();
        p.expect("/");
        let arity: usize = p.digits().parse().unwrap();
        let (mut private, mut destructor) = (false, false);
        if p.rest().starts_with('[') {
            let end = p.rest().find(']').expect("unterminated attr");
            let attrs = &p.rest()[1..end];
            private = attrs.contains("private");
            destructor = attrs.contains("destructor");
            p.pos += end + 1;
        }
        out.push(FunctionDecl { name, arity, private, destructor });
        if !p.eat(",") {
            break;
        }
    }
    out
}

fn parse_equations(s: &str) -> Vec<Equation> {
    let mut p = P::new(s);
    let mut out = Vec::new();
    loop {
        p.ws();
        if p.rest().is_empty() {
            break;
        }
        let lhs = p.term();
        p.expect("=");
        let rhs = p.term();
        out.push(Equation { lhs, rhs });
        if !p.eat(",") {
            break;
        }
    }
    out
}

// ── the master splitter: whole echo → (name, Signature, items) ──────────────

fn is_blank(l: &str) -> bool {
    l.is_empty()
}

/// Split one whole extracted echo into `(name, Signature, Vec<TheoryItem>)`.
fn parse_theory(capture: &str) -> (String, Signature, Vec<TheoryItem>) {
    let body = capture.strip_suffix('\n').unwrap_or(capture);
    let lines: Vec<&str> = body.lines().collect();
    assert!(lines[0].starts_with("theory "), "no theory header");
    let name = lines[0]["theory ".len()..].trim().to_string();
    assert_eq!(lines[1], "", "blank after theory name");
    assert_eq!(lines[2], "begin", "begin line");
    assert_eq!(lines[3], "", "blank after begin");
    let n = lines.len();
    // Trailing frame: `<3 blank lines> end`.
    assert_eq!(lines[n - 1], "end", "end line");
    assert!(
        is_blank(lines[n - 2]) && is_blank(lines[n - 3]) && is_blank(lines[n - 4]),
        "expected 3 blank lines before end"
    );
    let inner = &lines[4..n - 4];

    // Signature: `// header`, blank, then the declaration run up to a blank.
    assert!(inner[0].starts_with("// Function signature"), "signature header");
    assert_eq!(inner[1], "", "blank after signature header");
    let mut k = 2;
    while k < inner.len() && !is_blank(inner[k]) {
        k += 1;
    }
    let sig = parse_signature(&inner[2..k].join("\n"));

    let mut items: Vec<TheoryItem> = Vec::new();
    let mut i = k;
    while i < inner.len() {
        if is_blank(inner[i]) {
            i += 1;
            continue;
        }
        let line = inner[i];
        if line.starts_with("macros:") {
            let (block, next) = take_until_blank(inner, i);
            items.push(TheoryItem::Macros(parse_macros_block(&block)));
            i = next;
        } else if line.starts_with("predicate:") {
            // Group the whole contiguous run of predicate blocks into one item.
            let mut preds = Vec::new();
            while i < inner.len() && inner[i].starts_with("predicate:") {
                let (block, next) = take_until_blank(inner, i);
                preds.push(parse_predicate_block(&block));
                i = next;
                while i < inner.len() && is_blank(inner[i]) {
                    i += 1;
                }
            }
            items.push(TheoryItem::Predicates(preds));
        } else if let Some(v) = line.strip_prefix("heuristic:") {
            items.push(TheoryItem::Heuristic(v.trim().to_string()));
            i += 1;
        } else if line.starts_with("tactic:")
            || line.starts_with("section{*")
            || line.starts_with("subsection{*")
            || line.starts_with("text{*")
            || line.starts_with("options")
        {
            // Opaque top-level block: everything up to the next blank line.
            let (block, next) = take_until_blank(inner, i);
            items.push(TheoryItem::Verbatim(block));
            i = next;
        } else if line.starts_with("rule ") {
            let (block, next) = take_rule_block(inner, i);
            let (r, v) = parse_rule_block(&block);
            items.push(TheoryItem::Rule(r, v));
            i = next;
        } else if line.starts_with("restriction ") {
            let (block, next) = take_until_line(inner, i, "  */");
            items.push(TheoryItem::Restriction(parse_restriction_block(&block)));
            i = next;
        } else if line.starts_with("lemma ") {
            let (block, next) = take_lemma_block(inner, i);
            let (l, g) = parse_lemma_block(&block);
            items.push(TheoryItem::Lemma(l, g));
            i = next;
        } else if line.starts_with("/*") {
            // A top-level formal comment carried verbatim: the theory-level
            // `/* looping facts with injective instances: … */` note (single
            // line) or a multi-line `/* … */` user comment (through its col-0
            // `*/`).
            let (block, next) = if line.trim_end().ends_with("*/") {
                (line.to_string(), i + 1)
            } else {
                take_until_line(inner, i, "*/")
            };
            items.push(TheoryItem::Verbatim(block));
            i = next;
        } else {
            panic!("unrecognised top-level block at line {i}: {line:?}");
        }
    }
    (name, sig, items)
}

/// Lines `[start..=last-nonblank]` up to (excluding) the next blank; returns the
/// joined block and the index just past it.
fn take_until_blank(inner: &[&str], start: usize) -> (String, usize) {
    let mut j = start;
    while j < inner.len() && !is_blank(inner[j]) {
        j += 1;
    }
    (inner[start..j].join("\n"), j)
}

/// Lines `[start..=<line matching `marker`>]` inclusive.
fn take_until_line(inner: &[&str], start: usize, marker: &str) -> (String, usize) {
    let mut j = start;
    while inner[j] != marker {
        j += 1;
    }
    (inner[start..=j].join("\n"), j + 1)
}

/// A rule block: header+body, blank, optional loop-breaker line(s), then the
/// variants comment (one-liner or a `/* … */` span).
fn take_rule_block(inner: &[&str], start: usize) -> (String, usize) {
    let mut i = start;
    while i < inner.len() && !is_blank(inner[i]) {
        i += 1;
    }
    let mut j = i + 1;
    while j < inner.len() && inner[j].trim_start().starts_with("// loop breaker") {
        j += 1;
    }
    let end = if inner[j].trim() == "/* has exactly the trivial AC variant */" {
        j
    } else {
        assert_eq!(inner[j].trim(), "/*", "unexpected rule comment opener");
        let mut kk = j + 1;
        while inner[kk].trim() != "*/" {
            kk += 1;
        }
        kk
    };
    (inner[start..=end].join("\n"), end + 1)
}

/// A lemma block: statement, guarded comment (through its col-0 `*/`), then the
/// tail (`by …` one line, or an embedded proof through `qed`).
fn take_lemma_block(inner: &[&str], start: usize) -> (String, usize) {
    let mut i = start;
    while inner[i] != "*/" {
        i += 1;
    }
    i += 1;
    if inner[i].starts_with("by ") {
        // one-line tail
    } else {
        while inner[i] != "qed" {
            i += 1;
        }
    }
    (inner[start..=i].join("\n"), i + 1)
}

// ── the whole-echo parity test ──────────────────────────────────────────────

/// A diverse curated set spanning all block types: builtins variety (dh / xor /
/// multiset / bilinear-pairing / private / signing / hashing), macros, rules
/// with trivial + non-trivial AC variants, loop breakers, rule attributes /
/// SAPIC process attrs, fact annotations, restrictions (safety + expanded),
/// lemmas (all-traces + exists-trace + attributes + embedded proof + failed
/// conversion), predicates, heuristic, and opaque tactic / section blocks.
const CURATED: &[(&str, &str)] = &[
    ("round1", "features_multiset_minimal_multiset.spthy.hs.txt"),
    ("round1", "classic_NSLPK3.spthy.hs.txt"),
    ("round1", "features_xor_xor.spthy.hs.txt"),
    ("round1", "features_xor_xorplusdh.spthy.hs.txt"),
    ("round1", "features_multiset_NumberSubtermTests.spthy.hs.txt"),
    ("round1", "cav13_DH_example.spthy.hs.txt"),
    ("round1", "regression_trace_issue777.spthy.hs.txt"),
    ("round1", "features_private_function_symbols_test.spthy.hs.txt"),
    ("round1", "sp14_Joux.spthy.hs.txt"),
    ("round1", "Tutorial.spthy.hs.txt"),
    ("round2", "loops_Minimal_Loop_Example.spthy.hs.txt"),
    ("round2", "csf18-xor_CH07.spthy.hs.txt"),
    ("round2", "regression_trace_seqdfsneeded.spthy.hs.txt"),
    ("round2", "regression_trace_issue713-ruleattributes.spthy.hs.txt"),
    ("round2", "running-example.hs.txt"),
    ("round2", "ct.hs.txt"),
    ("round3", "features_predicates_minimal.spthy.hs.txt"),
    ("round3", "sapic_fast_feature-predicates_timepoints.spthy.hs.txt"),
    ("round3", "related_work_YubiSecure_KS_STM12_Yubikey.spthy.hs.txt"),
    ("round3", "thesis-LaraSchmid-evoting_chapter5_HumanErrors_AuthenticationProtocols_Cronto_EA.spthy.hs.txt"),
    ("round3", "post17_needham_schroeder_symmetric_cbc.spthy.hs.txt"),
    ("round3", "sapic_fast_GJM-contract_contract.spthy.hs.txt"),
    ("round3", "accountability_csf21-acc-unbounded_mixnets_basic_dmn-basic.spthy.hs.txt"),
    ("round3", "accountability_masters-thesis-morio_CentralizedMonitor.spthy.hs.txt"),
    ("round3", "thesis-LaraSchmid-evoting_chapter5_HumanErrors_HPagree.spthy.hs.txt"),
    ("round3", "esorics23-bluetooth_models_ble.spthy.hs.txt"),
    ("round3", "thesis-SvenHammann-POIDC_OIDC_Implicit.spthy.hs.txt"),
    ("round3", "csf18-alethea_alethea_votingphase_malS.spthy.hs.txt"),
    ("round3", "ccs18-5G_5G-AKA-bindingChannel_5G_AKA.spthy.hs.txt"),
];

#[test]
fn whole_echo_frame_parity() {
    // Some captures carry ~10 000-line variant blocks and deep formula nesting;
    // render on a generous stack like rounds 2–3.
    std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(whole_echo_frame_parity_impl)
        .unwrap()
        .join()
        .unwrap();
}

fn whole_echo_frame_parity_impl() {
    let base = std::path::Path::new("../..");
    // Self-skip once the crate moves at integration (the corpus gate takes over).
    if !base.join("round1/targets").is_dir() {
        return;
    }
    let mut passed = 0;
    let mut failures: Vec<String> = Vec::new();
    for (round, file) in CURATED {
        let path = base.join(round).join("targets").join(file);
        let capture = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => {
                failures.push(format!("MISSING {round}/{file}"));
                continue;
            }
        };
        let expected = capture.strip_suffix('\n').unwrap_or(&capture);
        let result = std::panic::catch_unwind(|| {
            let (name, sig, items) = parse_theory(&capture);
            let thy = Theory { name, items };
            render_theory(&thy, &sig)
        });
        match result {
            Ok(rendered) if rendered == expected => passed += 1,
            Ok(rendered) => {
                let at = rendered
                    .lines()
                    .zip(expected.lines())
                    .position(|(a, b)| a != b)
                    .unwrap_or(0);
                let g = expected.lines().nth(at).unwrap_or("<eof>");
                let got = rendered.lines().nth(at).unwrap_or("<eof>");
                failures.push(format!(
                    "DIFF {round}/{file} @line {at}\n    want: {g:?}\n    got:  {got:?}"
                ));
            }
            Err(e) => {
                let msg = e
                    .downcast_ref::<String>()
                    .cloned()
                    .or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string()))
                    .unwrap_or_else(|| "panic".into());
                failures.push(format!("PANIC {round}/{file}: {msg}"));
            }
        }
    }
    println!("whole-echo frame parity: {passed}/{} files byte-match", CURATED.len());
    assert!(
        failures.is_empty(),
        "whole-echo frame divergences ({} of {}):\n{}",
        failures.len(),
        CURATED.len(),
        failures.join("\n")
    );
    assert!(passed >= 15, "need >=15 diverse whole-echo files, got {passed}");
}
