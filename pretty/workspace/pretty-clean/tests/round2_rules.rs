//! R2 tests — rule rendering.
//!
//! Two layers:
//!  1. Probe-pinned unit tests: hand-written `Rule` fixtures asserted against
//!     byte-exact oracle output (probe captures under
//!     workspace/scratchpad/probes/, provenance `probe:<name>`).
//!  2. Whole-rule-block parity over the round-2 curated corpus captures
//!     (round2/targets/*.hs.txt): every rule block of every file is parsed
//!     back into the crate's AST by a layout-insensitive echo-parser (test
//!     harness only — it discards all whitespace, so layout can only come
//!     from the renderer) and re-rendered; the bytes must match the capture.

use pretty_clean::ast::*;
use pretty_clean::render_rule;

// ── fixture helpers ─────────────────────────────────────────────────────────

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

fn app(f: &str, args: Vec<Term>) -> Term {
    Term::App(f.into(), args)
}

fn xor(a: Term, b: Term) -> Term {
    Term::BinOp(BinOp::Xor, Box::new(a), Box::new(b))
}

fn fact(name: &str, args: Vec<Term>) -> Fact {
    Fact {
        persistent: false,
        name: name.into(),
        args,
        annotations: vec![],
    }
}

fn annotated(name: &str, args: Vec<Term>, annotations: Vec<FactAnnotation>) -> Fact {
    Fact {
        persistent: false,
        name: name.into(),
        args,
        annotations,
    }
}

fn rule(name: &str, prems: Vec<Fact>, acts: Vec<Fact>, concs: Vec<Fact>) -> Rule {
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

// ── probe-pinned unit tests ─────────────────────────────────────────────────

#[test]
fn one_line_body_and_trivial_comment() {
    // probe:p_lb2 rule S — one-line body, trivial-variant comment.
    let r = rule(
        "S",
        vec![fact("Fr", vec![fresh("a")])],
        vec![],
        vec![fact("A", vec![fresh("a")]), fact("B", vec![fresh("a")])],
    );
    let expected = [
        "rule (modulo E) S:",
        "   [ Fr( ~a ) ] --> [ A( ~a ), B( ~a ) ]",
        "",
        "  /* has exactly the trivial AC variant */",
    ]
    .join("\n");
    assert_eq!(render_rule(&r, None), expected);
}

#[test]
fn plural_loop_breakers() {
    // probe:p_lb2 rule L2 — `// loop breakers: [0,1]`, comma without space.
    let mut r = rule(
        "L2",
        vec![
            fact("A", vec![msg("x")]),
            fact("B", vec![msg("y")]),
            fact("In", vec![msg("z")]),
        ],
        vec![],
        vec![
            fact("A", vec![Term::Pair(vec![msg("x"), msg("z")])]),
            fact("B", vec![Term::Pair(vec![msg("y"), msg("z")])]),
        ],
    );
    r.loop_breakers = vec![0, 1];
    let expected = [
        "rule (modulo E) L2:",
        "   [ A( x ), B( y ), In( z ) ] --> [ A( <x, z> ), B( <y, z> ) ]",
        "",
        "  // loop breakers: [0,1]",
        "  /* has exactly the trivial AC variant */",
    ]
    .join("\n");
    assert_eq!(render_rule(&r, None), expected);
}

#[test]
fn fact_annotations_canonical_order() {
    // probe:p_fann — `[no_precomp,+]` echoes `[+, no_precomp]`; the wide
    // premise list forces the three-row body with the arrow out-dented.
    let r = rule(
        "F",
        vec![
            annotated("Fa", vec![msg("x")], vec![FactAnnotation::SolveFirst]),
            annotated("Fb", vec![msg("y")], vec![FactAnnotation::SolveLast]),
            annotated(
                "Fc",
                vec![msg("z")],
                vec![FactAnnotation::NoSources, FactAnnotation::SolveFirst],
            ),
            fact("In", vec![msg("w")]),
        ],
        vec![],
        vec![fact(
            "Out",
            vec![Term::Pair(vec![msg("x"), msg("y"), msg("z"), msg("w")])],
        )],
    );
    let expected = [
        "rule (modulo E) F:",
        "   [ Fa( x )[+], Fb( y )[-], Fc( z )[+, no_precomp], In( w ) ]",
        "  -->",
        "   [ Out( <x, y, z, w> ) ]",
        "",
        "  /* has exactly the trivial AC variant */",
    ]
    .join("\n");
    assert_eq!(render_rule(&r, None), expected);
}

#[test]
fn rule_attributes_canonical_order_and_wrap() {
    // probe:p_rattr — canonical order color/no_derivcheck/issapicrule/role,
    // `color=#hex` unquoted, `role='…'` quoted; the long list fill-wraps
    // aligned after the `[`.
    let mut r1 = rule(
        "R1",
        vec![fact("In", vec![msg("x")])],
        vec![],
        vec![fact("Out", vec![msg("x")])],
    );
    r1.attributes = vec![
        RuleAttr::Role("myrole".into()),
        RuleAttr::NoDerivCheck,
        RuleAttr::Color("a1b2c3".into()),
    ];
    let expected1 = [
        "rule (modulo E) R1[color=#a1b2c3, no_derivcheck, role='myrole']:",
        "   [ In( x ) ] --> [ Out( x ) ]",
        "",
        "  /* has exactly the trivial AC variant */",
    ]
    .join("\n");
    assert_eq!(render_rule(&r1, None), expected1);

    let mut r3 = rule(
        "R3",
        vec![fact("In", vec![msg("x")])],
        vec![],
        vec![fact("Out", vec![msg("x")])],
    );
    r3.attributes = vec![
        RuleAttr::IsSapicRule,
        RuleAttr::Color("ff0000".into()),
        RuleAttr::Role("r3longroleAAAAAAAAAAAA".into()),
        RuleAttr::NoDerivCheck,
    ];
    let expected3 = [
        "rule (modulo E) R3[color=#ff0000, no_derivcheck, issapicrule,",
        "                   role='r3longroleAAAAAAAAAAAA']:",
        "   [ In( x ) ] --> [ Out( x ) ]",
        "",
        "  /* has exactly the trivial AC variant */",
    ]
    .join("\n");
    assert_eq!(render_rule(&r3, None), expected3);
}

#[test]
fn variants_substitution_layout() {
    // probe:p_var1 — a substitution lhs shorter than 6 columns is padded to
    // column 6; a longer lhs pushes `= rhs` to its own line at column 6;
    // groups are separated by a line of bare indent (four spaces).
    let long = "longvariablenameone";
    let r = rule(
        "V",
        vec![
            fact("Fr", vec![fresh(long)]),
            fact("Fr", vec![fresh("lv2")]),
        ],
        vec![],
        vec![fact("Out", vec![xor(fresh(long), fresh("lv2"))])],
    );
    let ac = rule(
        "V",
        vec![
            fact("Fr", vec![fresh(long)]),
            fact("Fr", vec![fresh("lv2")]),
        ],
        vec![],
        vec![fact("Out", vec![msg("z")])],
    );
    let ac = Rule {
        modulo: Some("AC".into()),
        ..ac
    };
    let v = AcVariants {
        ac_rule: ac,
        substitutions: vec![
            vec![
                (fresh(long), var(long, 4, SortHint::Fresh)),
                (fresh("lv2"), var("lv2", 4, SortHint::Fresh)),
                (
                    msg("z"),
                    xor(var(long, 4, SortHint::Fresh), var("lv2", 4, SortHint::Fresh)),
                ),
            ],
            vec![
                (fresh(long), var("x", 4, SortHint::Fresh)),
                (fresh("lv2"), var("x", 4, SortHint::Fresh)),
                (msg("z"), app("zero", vec![])),
            ],
        ],
    };
    let expected = [
        "rule (modulo E) V:",
        "   [ Fr( ~longvariablenameone ), Fr( ~lv2 ) ]",
        "  -->",
        "   [ Out( (~longvariablenameone\u{2295}~lv2) ) ]",
        "",
        "  /*",
        "  rule (modulo AC) V:",
        "     [ Fr( ~longvariablenameone ), Fr( ~lv2 ) ] --> [ Out( z ) ]",
        "    variants (modulo AC)",
        "    1. ~longvariablenameone",
        "             = ~longvariablenameone.4",
        "       ~lv2  = ~lv2.4",
        "       z     = (~longvariablenameone.4\u{2295}~lv2.4)",
        "    ",
        "    2. ~longvariablenameone",
        "             = ~x.4",
        "       ~lv2  = ~x.4",
        "       z     = zero",
        "  */",
    ]
    .join("\n");
    assert_eq!(render_rule(&r, Some(&v)), expected);
}

#[test]
fn loop_breaker_inside_variants_comment() {
    // probe:p_lbvar — the E-side breaker sits between the body and the
    // comment; the AC rule's breaker renders INSIDE the comment after the
    // variants list, at the comment nesting.
    let mut r = rule(
        "LV",
        vec![fact("A", vec![msg("x")]), fact("Fr", vec![fresh("y")])],
        vec![],
        vec![fact("A", vec![xor(fresh("y"), msg("x"))])],
    );
    r.loop_breakers = vec![0];
    let mut ac = rule(
        "LV",
        vec![fact("A", vec![msg("x")]), fact("Fr", vec![fresh("y")])],
        vec![],
        vec![fact("A", vec![msg("z")])],
    );
    ac.modulo = Some("AC".into());
    ac.loop_breakers = vec![0];
    let v = AcVariants {
        ac_rule: ac,
        substitutions: vec![
            vec![
                (fresh("y"), var("y", 4, SortHint::Fresh)),
                (msg("x"), var("x", 4, SortHint::Untagged)),
                (
                    msg("z"),
                    xor(var("y", 4, SortHint::Fresh), var("x", 4, SortHint::Untagged)),
                ),
            ],
            vec![
                (fresh("y"), var("y", 4, SortHint::Fresh)),
                (msg("x"), app("zero", vec![])),
                (msg("z"), var("y", 4, SortHint::Fresh)),
            ],
        ],
    };
    let expected = [
        "rule (modulo E) LV:",
        "   [ A( x ), Fr( ~y ) ] --> [ A( (~y\u{2295}x) ) ]",
        "",
        "  // loop breaker: [0]",
        "  /*",
        "  rule (modulo AC) LV:",
        "     [ A( x ), Fr( ~y ) ] --> [ A( z ) ]",
        "    variants (modulo AC)",
        "    1. ~y    = ~y.4",
        "       x     = x.4",
        "       z     = (~y.4\u{2295}x.4)",
        "    ",
        "    2. ~y    = ~y.4",
        "       x     = zero",
        "       z     = ~y.4",
        "    // loop breaker: [0]",
        "  */",
    ]
    .join("\n");
    assert_eq!(render_rule(&r, Some(&v)), expected);
}

#[test]
fn persistent_facts_and_empty_args() {
    // target:NSLPK3 Register_pk (persistent `!` prefix);
    // target:mesh ProvisionerSendProvInvite (`Name( )` nullary fact).
    let mut ltk = fact("Ltk", vec![var("A", 0, SortHint::Pub), fresh("ltkA")]);
    ltk.persistent = true;
    assert_eq!(
        pretty_clean::render_fact(&ltk),
        "!Ltk( $A, ~ltkA )"
    );
    assert_eq!(
        pretty_clean::render_fact(&fact("ProvisionerStartProvisioning", vec![])),
        "ProvisionerStartProvisioning( )"
    );
}

#[test]
fn macros_block_layout() {
    // probe:p_mac1 — all-or-nothing item list aligned after `macros: `
    // (m2 gets its own line though it would fit beside m1), fact-style heads
    // with the `)` attached to the last param line, and `) =  body` spacing;
    // target:issue777 — the one-line single-macro form.
    let mac = |name: &str, params: Vec<Term>, body: Term| Macro {
        name: name.into(),
        params,
        body,
    };
    let long_pair = Term::Pair(vec![
        msg("xlongvariablename1"),
        msg("ylongvariablename2"),
        msg("xlongvariablename1"),
    ]);
    let ms = vec![
        mac(
            "m1",
            vec![msg("x"), msg("y")],
            app("h", vec![Term::Pair(vec![msg("x"), msg("y")])]),
        ),
        mac("m2", vec![], app("h", vec![Term::PubLit("c".into())])),
        mac(
            "mlongernameAAAAAAAAAAAAAAAA",
            vec![msg("xlongvariablename1"), msg("ylongvariablename2")],
            app("h", vec![long_pair]),
        ),
    ];
    let expected = [
        "macros: m1( x, y ) =  h(<x, y>),",
        "        m2( ) =  h('c'),",
        "        mlongernameAAAAAAAAAAAAAAAA( xlongvariablename1,",
        "                                     ylongvariablename2 ) =  h(<xlongvariablename1, ylongvariablename2, ",
        "                                                                xlongvariablename1>)",
    ]
    .join("\n");
    assert_eq!(pretty_clean::macros::render_macros(&ms), expected);

    // target:issue777.
    let pk = mac(
        "pk",
        vec![msg("x")],
        Term::BinOp(
            BinOp::Exp,
            Box::new(Term::PubLit("g".into())),
            Box::new(msg("x")),
        ),
    );
    assert_eq!(
        pretty_clean::macros::render_macros(&[pk]),
        "macros: pk( x ) =  'g'^x"
    );
}

// ── whole-rule-block parity over the round-2 curated captures ───────────────

/// Slice a capture into its rule blocks: each spans the `rule …` header
/// through the trailing variants comment (including the interior blank line
/// and any loop-breaker line).
fn rule_blocks(capture: &str) -> Vec<String> {
    let lines: Vec<&str> = capture.lines().collect();
    let mut blocks = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if !lines[i].starts_with("rule ") {
            i += 1;
            continue;
        }
        let start = i;
        // Body: up to the first blank line.
        while i < lines.len() && !lines[i].is_empty() {
            i += 1;
        }
        // Optional loop-breaker annotation line(s).
        let mut j = i + 1;
        while j < lines.len() && lines[j].trim_start().starts_with("// loop breaker") {
            j += 1;
        }
        // The variants comment: one-liner or a /* … */ span.
        let end = if lines[j].trim() == "/* has exactly the trivial AC variant */" {
            j
        } else {
            assert_eq!(lines[j].trim(), "/*", "unexpected comment opener");
            let mut k = j + 1;
            while lines[k].trim() != "*/" {
                k += 1;
            }
            k
        };
        blocks.push(lines[start..=end].join("\n"));
        i = end + 1;
    }
    blocks
}

// ── the layout-insensitive echo parser (test harness only) ──────────────────
//
// Parses a rendered rule block back into the crate AST. It skips ALL
// whitespace between tokens, so the re-rendered layout can only come from
// the renderer under test, never from the capture.

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
            &self.rest()[..self.rest().len().min(60)]
        );
    }

    fn ident(&mut self) -> String {
        self.ws();
        let s: String = self
            .rest()
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        assert!(!s.is_empty(), "expected ident at …{:?}", &self.rest()[..self.rest().len().min(40)]);
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
            && self.s[self.pos + 1..].chars().next().is_some_and(|c| c.is_ascii_digit())
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
        Term::Var(VarSpec {
            name,
            idx,
            sort,
            typ: None,
        })
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
                    panic!("AC operator expected at …{:?}", &self.rest()[..self.rest().len().min(40)]);
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
                    // Application (nullary symbols render bare, so `(` here
                    // always introduces at least one argument).
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

    // ── facts ──

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
        // Annotations attach with no whitespace after the paren.
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
        Fact {
            persistent,
            name,
            args,
            annotations,
        }
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
                    let v: String = self
                        .rest()
                        .chars()
                        .take_while(|c| c.is_ascii_alphanumeric())
                        .collect();
                    self.pos += v.len();
                    attributes.push(RuleAttr::Color(v));
                } else if self.eat("no_derivcheck") {
                    attributes.push(RuleAttr::NoDerivCheck);
                } else if self.eat("issapicrule") {
                    attributes.push(RuleAttr::IsSapicRule);
                } else if self.eat("role='") {
                    let end = self.rest().find('\'').unwrap();
                    attributes.push(RuleAttr::Role(self.rest()[..end].into()));
                    self.pos += end + 1;
                } else {
                    panic!("attribute expected at …{:?}", &self.rest()[..self.rest().len().min(40)]);
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
        Rule {
            name,
            modulo,
            attributes,
            premises,
            actions,
            conclusions,
            loop_breakers: vec![],
        }
    }

    /// `// loop breaker: [0]` / `// loop breakers: [0,1]`.
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
}

/// Parse one extracted rule block into (Rule, Option<AcVariants>).
fn parse_block(block: &str) -> (Rule, Option<AcVariants>) {
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
            // A group index (`12.`) or the next substitution's lhs.
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
    (
        r,
        Some(AcVariants {
            ac_rule: ac,
            substitutions,
        }),
    )
}

#[test]
fn parity_rule_blocks_match_captures() {
    // The Doc engine recurses deeply on very large variant comments (Joux
    // has a 160-variant block), so run on a wide stack.
    std::thread::Builder::new()
        .stack_size(512 * 1024 * 1024)
        .spawn(parity_rule_blocks_impl)
        .unwrap()
        .join()
        .unwrap();
}

fn parity_rule_blocks_impl() {
    // Sealed-workspace location of the curated round-2 captures; self-skips
    // once the crate moves at integration (the corpus gate takes over).
    let dir = std::path::Path::new("../../round2/targets");
    if !dir.is_dir() {
        return;
    }
    let mut checked = 0;
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("txt") {
            continue;
        }
        let capture = std::fs::read_to_string(&path).unwrap();
        for (i, block) in rule_blocks(&capture).iter().enumerate() {
            let (r, v) = parse_block(block);
            let rendered = render_rule(&r, v.as_ref());
            assert_eq!(
                &rendered,
                block,
                "rule-block divergence: {} block #{i} ({})",
                path.display(),
                r.name
            );
            checked += 1;
        }
    }
    assert!(checked > 50, "expected many rule blocks, got {checked}");
}
