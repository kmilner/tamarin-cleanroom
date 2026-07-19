//! Layout-insensitive echo parser (test harness only), shared by the R6 web
//! acceptance sweep. Copied from the round-5 inline parser (which is left
//! untouched) so the R1–R5 suites are not perturbed; it consults only token
//! order, never layout, so it parses the web mode's width-100 plain text (spans
//! stripped, entities unescaped) into the same model the batch echo yields.

#![allow(dead_code)]

use pretty_clean::ast::*;

pub struct P<'a> {
    pub s: &'a str,
    pub pos: usize,
}

impl<'a> P<'a> {
    pub fn new(s: &'a str) -> Self {
        P { s, pos: 0 }
    }
    pub fn rest(&self) -> &'a str {
        &self.s[self.pos..]
    }
    pub fn ws(&mut self) {
        while let Some(c) = self.rest().chars().next() {
            if c.is_whitespace() {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
    }
    pub fn starts(&mut self, tok: &str) -> bool {
        self.ws();
        self.rest().starts_with(tok)
    }
    pub fn eat(&mut self, tok: &str) -> bool {
        if self.starts(tok) {
            self.pos += tok.len();
            true
        } else {
            false
        }
    }
    pub fn expect(&mut self, tok: &str) {
        assert!(
            self.eat(tok),
            "expected {tok:?} at …{:?}",
            &self.rest()[..self.rest().len().min(70)]
        );
    }
    pub fn ident(&mut self) -> String {
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
    pub fn digits(&mut self) -> String {
        let s: String = self
            .rest()
            .chars()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        self.pos += s.len();
        s
    }
    pub fn name_idx(&mut self) -> (String, u64) {
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
    pub fn quoted(&mut self) -> String {
        self.expect("'");
        let end = self.rest().find('\'').expect("unterminated quote");
        let s = self.rest()[..end].to_string();
        self.pos += end + 1;
        s
    }

    pub fn term(&mut self) -> Term {
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

    pub fn fact(&mut self) -> Fact {
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

    pub fn rule_core(&mut self) -> Rule {
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
    pub fn formula(&mut self) -> Formula {
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
}

// ── block parsers ───────────────────────────────────────────────────────────

/// A bare rule (header + body only) — the construction/deconstruction form.
pub fn parse_bare_rule(block: &str) -> Rule {
    let mut p = P::new(block);
    let r = p.rule_core();
    p.ws();
    assert!(p.rest().is_empty(), "trailing text after bare rule: {:?}", p.rest());
    r
}

pub fn parse_rule_block(block: &str) -> (Rule, Option<AcVariants>) {
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

pub fn parse_restriction_block(block: &str) -> Restriction {
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

/// Reconstruct the `Signature` from the web message-pane body (the declaration
/// lines only — no batch header comment).
pub fn parse_signature(decls: &str) -> Signature {
    let lines: Vec<&str> = decls.lines().collect();
    let mut builtins = Vec::new();
    let mut functions = Vec::new();
    let mut equations = Vec::new();
    let mut convergent = false;
    let mut i = 0;
    while i < lines.len() {
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
    // The `builtins:` line never lists `dest-pairing` (it induces no operators),
    // so recover it from the shown functions: a destructor `fst`/`snd` means the
    // theory used dest-pairing, and the renderer must flip its base pairing
    // symbols to destructors (else it re-adds constructor `fst/1` alongside the
    // shown `fst/1[destructor]`). This is a reconstruction detail; the real
    // adapter carries the builtin set directly.
    if functions
        .iter()
        .any(|f| f.destructor && (f.name == "fst" || f.name == "snd") && f.arity == 1)
    {
        builtins.push("dest-pairing".to_string());
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
