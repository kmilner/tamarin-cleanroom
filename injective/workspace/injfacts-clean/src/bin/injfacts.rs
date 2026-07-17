//! Validation harness. Reads a compact description of a theory's rules on stdin
//! and prints the injective fact tags (one `name/arity` per line, sorted).
//!
//! This binary only *reconstructs* `Rule`/`Fact`/`Term` values so the real
//! library function can be exercised on rules parsed from the oracle's own
//! normalized rule pretty-print. It carries no injectivity logic.
//!
//! Input grammar (one token stream per line):
//!   RULE                 -- start a new rule
//!   P <fact>             -- a premise fact
//!   C <fact>             -- a conclusion fact
//! where <fact> = [!]name arg1 arg2 ...   (whitespace separated, no inner spaces)
//! and a token starting with '~' is a Fresh variable; anything else is treated
//! as a non-variable literal (only first-argument freshness / equality matter).

use injfacts_clean::ast::{Fact, Rule, SortHint, Term, VarSpec};
use injfacts_clean::injective_fact_instances;
use std::io::{self, Read};

fn token_to_term(tok: &str) -> Term {
    if let Some(rest) = tok.strip_prefix('~') {
        Term::Var(VarSpec {
            name: rest.to_string(),
            idx: 0,
            sort: SortHint::Fresh,
            typ: None,
        })
    } else {
        Term::PubLit(tok.to_string())
    }
}

fn parse_fact(fields: &[&str]) -> Fact {
    let raw = fields[0];
    let (persistent, name) = match raw.strip_prefix('!') {
        Some(n) => (true, n.to_string()),
        None => (false, raw.to_string()),
    };
    let args = fields[1..].iter().map(|t| token_to_term(t)).collect();
    Fact {
        persistent,
        name,
        args,
        annotations: vec![],
    }
}

fn empty_rule() -> Rule {
    Rule {
        name: String::new(),
        modulo: None,
        attributes: vec![],
        let_block: vec![],
        premises: vec![],
        actions: vec![],
        conclusions: vec![],
        embedded_restrictions: vec![],
        variants: vec![],
        left_right: None,
    }
}

fn main() {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input).unwrap();

    let mut rules: Vec<Rule> = Vec::new();
    let mut cur: Option<Rule> = None;
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split_whitespace().collect();
        match fields[0] {
            "RULE" => {
                if let Some(r) = cur.take() {
                    rules.push(r);
                }
                cur = Some(empty_rule());
            }
            "P" => {
                if let Some(r) = cur.as_mut() {
                    r.premises.push(parse_fact(&fields[1..]));
                }
            }
            "C" => {
                if let Some(r) = cur.as_mut() {
                    r.conclusions.push(parse_fact(&fields[1..]));
                }
            }
            _ => {}
        }
    }
    if let Some(r) = cur.take() {
        rules.push(r);
    }

    let result = injective_fact_instances(&rules);
    for (name, arity) in result {
        println!("{}/{}", name, arity);
    }
}
