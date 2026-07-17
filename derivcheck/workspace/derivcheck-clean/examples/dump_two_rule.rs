// Emits the two-rule (POIDC_CMB) block so it can be byte-compared with the
// captured reference fixture.
use derivcheck_clean::ast::*;
use derivcheck_clean::*;
use std::collections::HashSet;

struct FlagByName(HashSet<String>);
impl DerivabilitySolver for FlagByName {
    fn check_rule(&self, p: &RuleProbe) -> Vec<Derivability> {
        p.variables
            .iter()
            .map(|v| if self.0.contains(&v.name) { Derivability::NotDerivable } else { Derivability::Derivable })
            .collect()
    }
}
fn mv(n: &str) -> Term { Term::Var(VarSpec { name: n.into(), idx: 0, sort: SortHint::Msg, typ: None }) }
fn f(n: &str, a: Vec<Term>) -> Fact { Fact { persistent: false, name: n.into(), args: a, annotations: vec![] } }
fn ap(n: &str, a: Vec<Term>) -> Term { Term::App(n.into(), a) }
fn rule(name: &str, prem: Vec<Fact>, conc: Vec<Fact>) -> Rule {
    Rule { name: name.into(), modulo: None, attributes: vec![], let_block: vec![], premises: prem,
        actions: vec![], conclusions: conc, embedded_restrictions: vec![], variants: vec![], left_right: None }
}
fn main() {
    let re_sign = rule("reSign",
        vec![f("In", vec![Term::Pair(vec![mv("sk1"), mv("r1")])]),
             f("In", vec![ap("sign", vec![mv("m"), mv("r2"), mv("sk2")])])],
        vec![f("Out", vec![ap("sign", vec![mv("m"), mv("r1"), mv("sk1")])])]);
    let rp = rule("RP_gets_idToken",
        vec![f("In", vec![ap("raenc", vec![mv("x"), mv("rndA"), mv("pkA")])])],
        vec![f("Out", vec![Term::PubLit("ok".into())])]);
    let thy = Theory { is_diff: false, name: "T".into(), configuration: None,
        items: vec![TheoryItem::Rule(re_sign), TheoryItem::Rule(rp)] };
    let solver = FlagByName(["m", "r2", "sk2", "pkA"].iter().map(|s| s.to_string()).collect());
    let report = message_derivation_checks(&thy, &solver, 5);
    print!("{}", report[0].message);
}
