//! Round-5 corpus acceptance harness.
//!
//! `tests/corpus5/*.fml` holds every lemma and restriction formula printed by
//! the Haskell reference for the 71 round-5 gate-diff theories (captured from
//! oracle output; the reference reports NO formula-bundle topic for any of
//! them - their wf blocks contain no "Quantifier sorts", "Formula terms" or
//! " Formula guardedness" section). This test parses each printed formula
//! back into the wf-clean AST and asserts our formula bundle is silent too:
//! the guardedness DECISION (and the sort/term checks) must accept everything
//! the reference accepts.

use wf_clean::*;

// ---------------------------------------------------------------------------
// Parser for the oracle's pretty-printed (Unicode) formula syntax
// ---------------------------------------------------------------------------

struct P {
    cs: Vec<char>,
    i: usize,
    /// Nullary function symbols of the source theory (from its printed
    /// `functions:` block); bare occurrences are constants, not variables.
    nullary: std::collections::BTreeSet<String>,
}

#[derive(Debug)]
struct PErr(String);

type PR<T> = Result<T, PErr>;

impl P {
    fn new(s: &str, nullary: std::collections::BTreeSet<String>) -> Self {
        P {
            cs: s.chars().collect(),
            i: 0,
            nullary,
        }
    }
    fn ws(&mut self) {
        while self.i < self.cs.len() && self.cs[self.i].is_whitespace() {
            self.i += 1;
        }
    }
    fn peek(&mut self) -> Option<char> {
        self.ws();
        self.cs.get(self.i).copied()
    }
    fn eat(&mut self, c: char) -> PR<()> {
        if self.peek() == Some(c) {
            self.i += 1;
            Ok(())
        } else {
            Err(PErr(format!("expected {:?} at {}", c, self.i)))
        }
    }
    fn try_eat(&mut self, c: char) -> bool {
        if self.peek() == Some(c) {
            self.i += 1;
            true
        } else {
            false
        }
    }
    fn name(&mut self) -> PR<String> {
        self.ws();
        let start = self.i;
        while self.i < self.cs.len()
            && (self.cs[self.i].is_alphanumeric() || self.cs[self.i] == '_')
        {
            self.i += 1;
        }
        if self.i == start {
            return Err(PErr(format!("expected name at {}", self.i)));
        }
        Ok(self.cs[start..self.i].iter().collect())
    }
    /// `name` optionally followed by `.digits` (the printed variable index).
    fn name_idx(&mut self) -> PR<(String, u64)> {
        let n = self.name()?;
        let mut idx = 0u64;
        if self.cs.get(self.i) == Some(&'.')
            && self.cs.get(self.i + 1).map_or(false, |c| c.is_ascii_digit())
        {
            self.i += 1;
            let start = self.i;
            while self.i < self.cs.len() && self.cs[self.i].is_ascii_digit() {
                self.i += 1;
            }
            idx = self.cs[start..self.i]
                .iter()
                .collect::<String>()
                .parse()
                .unwrap();
        }
        Ok((n, idx))
    }

    fn var(&mut self) -> PR<VarSpec> {
        let sort = match self.peek() {
            Some('#') => {
                self.i += 1;
                SortHint::Node
            }
            Some('~') => {
                self.i += 1;
                SortHint::Fresh
            }
            Some('$') => {
                self.i += 1;
                SortHint::Pub
            }
            Some('%') => {
                self.i += 1;
                SortHint::Nat
            }
            _ => SortHint::Untagged,
        };
        let (name, idx) = self.name_idx()?;
        Ok(VarSpec {
            name,
            idx,
            sort,
            typ: None,
        })
    }

    fn term(&mut self) -> PR<Term> {
        match self.peek() {
            Some('\'') => {
                self.i += 1;
                let start = self.i;
                while self.i < self.cs.len() && self.cs[self.i] != '\'' {
                    self.i += 1;
                }
                let s: String = self.cs[start..self.i].iter().collect();
                self.eat('\'')?;
                Ok(Term::PubLit(s))
            }
            Some('<') => {
                self.i += 1;
                let mut items = vec![self.term()?];
                while self.try_eat(',') {
                    items.push(self.term()?);
                }
                self.eat('>')?;
                Ok(Term::Pair(items))
            }
            Some('(') => {
                // parenthesised term with infix AC operators, e.g. `(x++z)`
                self.i += 1;
                let mut t = self.term()?;
                loop {
                    self.ws();
                    if self.cs.get(self.i) == Some(&'+') && self.cs.get(self.i + 1) == Some(&'+') {
                        self.i += 2;
                        let r = self.term()?;
                        t = Term::BinOp(BinOp::Union, Box::new(t), Box::new(r));
                    } else if self.cs.get(self.i) == Some(&'⊕') {
                        self.i += 1;
                        let r = self.term()?;
                        t = Term::BinOp(BinOp::Xor, Box::new(t), Box::new(r));
                    } else {
                        break;
                    }
                }
                self.eat(')')?;
                Ok(t)
            }
            Some('~') if self.cs.get(self.i + 1) == Some(&'\'') => {
                self.i += 2;
                let start = self.i;
                while self.i < self.cs.len() && self.cs[self.i] != '\'' {
                    self.i += 1;
                }
                let s: String = self.cs[start..self.i].iter().collect();
                self.eat('\'')?;
                Ok(Term::FreshLit(s))
            }
            Some(c) if c == '#' || c == '~' || c == '$' || c == '%' => {
                Ok(Term::Var(self.var()?))
            }
            _ => {
                let (name, idx) = self.name_idx()?;
                if idx == 0 && self.cs.get(self.i) == Some(&'(') {
                    self.i += 1;
                    let mut args = Vec::new();
                    if self.peek() != Some(')') {
                        args.push(self.term()?);
                        while self.try_eat(',') {
                            args.push(self.term()?);
                        }
                    }
                    self.eat(')')?;
                    Ok(Term::App(name, args))
                } else if idx == 0 && name == "DH_neutral" {
                    Ok(Term::DhNeutral)
                } else if idx == 0 && self.nullary.contains(&name) {
                    Ok(Term::App(name, vec![]))
                } else {
                    Ok(Term::Var(VarSpec {
                        name,
                        idx,
                        sort: SortHint::Untagged,
                        typ: None,
                    }))
                }
            }
        }
    }

    /// An atom starting at the current position: a fact/action, an equality,
    /// an ordering or a `last`. `persistent` facts carry a leading `!`.
    fn atom(&mut self) -> PR<Formula> {
        let persistent = self.try_eat('!');
        let t = self.term()?;
        match self.peek() {
            Some('@') => {
                self.i += 1;
                let tv = self.term()?;
                let fact = match t {
                    Term::App(name, args) => Fact {
                        persistent,
                        name,
                        args,
                        annotations: vec![],
                    },
                    Term::Var(v) => Fact {
                        persistent,
                        name: v.name,
                        args: vec![],
                        annotations: vec![],
                    },
                    other => return Err(PErr(format!("bad fact term {:?}", other))),
                };
                Ok(Formula::Atom(Atom::Action(fact, tv)))
            }
            Some('=') => {
                self.i += 1;
                let r = self.term()?;
                Ok(Formula::Atom(Atom::Eq(t, r)))
            }
            Some('<') => {
                self.i += 1;
                let r = self.term()?;
                Ok(Formula::Atom(Atom::Less(t, r)))
            }
            Some('⋖') => {
                self.i += 1;
                let r = self.term()?;
                Ok(Formula::Atom(Atom::LessMset(t, r)))
            }
            Some('⊏') => {
                self.i += 1;
                let r = self.term()?;
                Ok(Formula::Atom(Atom::Subterm(t, r)))
            }
            _ => match t {
                Term::App(name, args) if name == "last" && args.len() == 1 => {
                    Ok(Formula::Atom(Atom::Last(args.into_iter().next().unwrap())))
                }
                other => Err(PErr(format!("dangling term {:?}", other))),
            },
        }
    }

    fn unary(&mut self) -> PR<Formula> {
        match self.peek() {
            Some('∀') | Some('∃') => {
                let forall = self.cs[self.i] == '∀';
                self.i += 1;
                let mut vars = Vec::new();
                loop {
                    self.ws();
                    if self.cs.get(self.i) == Some(&'.') {
                        self.i += 1;
                        break;
                    }
                    vars.push(self.var()?);
                }
                let body = self.formula()?;
                Ok(if forall {
                    Formula::Forall(vars, Box::new(body))
                } else {
                    Formula::Exists(vars, Box::new(body))
                })
            }
            Some('¬') => {
                self.i += 1;
                Ok(Formula::Not(Box::new(self.unary()?)))
            }
            Some('⊥') => {
                self.i += 1;
                Ok(Formula::False)
            }
            Some('⊤') => {
                self.i += 1;
                Ok(Formula::True)
            }
            Some('(') => {
                // Either `(formula)` or an atom whose left term is
                // parenthesised (`(x++z) = y`): try the atom first.
                let save = self.i;
                if let Ok(a) = self.atom() {
                    return Ok(a);
                }
                self.i = save;
                self.eat('(')?;
                let f = self.formula()?;
                self.eat(')')?;
                Ok(f)
            }
            _ => self.atom(),
        }
    }

    fn formula(&mut self) -> PR<Formula> {
        let mut f = self.unary()?;
        loop {
            match self.peek() {
                Some('∧') => {
                    self.i += 1;
                    let r = self.unary()?;
                    f = Formula::And(Box::new(f), Box::new(r));
                }
                Some('∨') => {
                    self.i += 1;
                    let r = self.unary()?;
                    f = Formula::Or(Box::new(f), Box::new(r));
                }
                Some('⇒') => {
                    self.i += 1;
                    let r = self.formula()?;
                    f = Formula::Implies(Box::new(f), Box::new(r));
                }
                Some('⇔') => {
                    self.i += 1;
                    let r = self.formula()?;
                    f = Formula::Iff(Box::new(f), Box::new(r));
                }
                _ => break,
            }
        }
        Ok(f)
    }
}

fn parse_formula(s: &str, nullary: std::collections::BTreeSet<String>) -> Result<Formula, String> {
    let mut p = P::new(s, nullary);
    let f = p.formula().map_err(|e| e.0)?;
    p.ws();
    if p.i != p.cs.len() {
        return Err(format!(
            "trailing input at {}: {:?}",
            p.i,
            p.cs[p.i..].iter().take(30).collect::<String>()
        ));
    }
    Ok(f)
}

fn lemma_item(name: &str, f: Formula) -> TheoryItem {
    TheoryItem::Lemma(Lemma {
        name: name.to_string(),
        modulo: None,
        attributes: vec![],
        trace_quantifier: TraceQuantifier::AllTraces,
        formula: f,
        proof: None,
        plaintext: String::new(),
    })
}

fn restriction_item(name: &str, f: Formula) -> TheoryItem {
    TheoryItem::Restriction(Restriction {
        name: name.to_string(),
        formula: f,
        attributes: vec![],
    })
}

/// Every reference-accepted corpus formula must produce an empty formula
/// bundle (no Quantifier sorts, no Formula terms, no Formula guardedness).
#[test]
fn corpus5_reference_accepted_formulas_are_accepted() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus5");
    let mut entries: Vec<_> = std::fs::read_dir(dir)
        .expect("tests/corpus5 missing")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().map_or(false, |x| x == "fml"))
        .collect();
    entries.sort();
    assert!(
        entries.len() > 600,
        "expected the full corpus capture, found {}",
        entries.len()
    );
    let mut parse_failures = Vec::new();
    let mut report_failures = Vec::new();
    for path in &entries {
        let fname = path.file_name().unwrap().to_string_lossy().to_string();
        let text = std::fs::read_to_string(path).unwrap();
        let text = text.trim();
        let base = fname.split("__").next().unwrap().to_string();
        let fns_path = format!("{}/{}.fns", dir, base);
        let nullary: std::collections::BTreeSet<String> = std::fs::read_to_string(&fns_path)
            .map(|t| t.lines().map(|l| l.trim().to_string()).filter(|l| !l.is_empty()).collect())
            .unwrap_or_default();
        let f = match parse_formula(text, nullary) {
            Ok(f) => f,
            Err(e) => {
                parse_failures.push(format!("{}: {}", fname, e));
                continue;
            }
        };
        let is_lemma = fname.contains("__lemma__");
        let item = if is_lemma {
            lemma_item("L", f)
        } else {
            restriction_item("R", f)
        };
        let thy = Theory {
            is_diff: false,
            name: "corpus".into(),
            configuration: None,
            items: vec![item],
        };
        let r = wf_clean::checks::formula_reports(&thy, &std::collections::BTreeSet::new());
        if !r.is_empty() {
            report_failures.push(format!(
                "{}: [{}] {}",
                fname,
                r[0].topic,
                r[0].message.lines().take(3).collect::<Vec<_>>().join(" | ")
            ));
        }
    }
    assert!(
        parse_failures.is_empty(),
        "{} formulas failed to parse:\n{}",
        parse_failures.len(),
        parse_failures.join("\n")
    );
    assert!(
        report_failures.is_empty(),
        "{} reference-accepted formulas were reported:\n{}",
        report_failures.len(),
        report_failures.join("\n")
    );
}
