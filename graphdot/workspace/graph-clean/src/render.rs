//! Record-cell CONTENT rendering (BEHAVIOR.md §3a, §3f): turning facts / rule
//! instances into the flat, un-wrapped text that a record cell holds, plus the
//! graphviz record-label escaping. The line WRAPPING of that text (the `\l` /
//! `&nbsp;` layout) is done by the faithful HughesPJ engine in [`crate::pretty`],
//! driven from [`crate::doclayout`]; this module produces only the flat text and
//! the escaping.
//!
//! What is byte-exact here (verified against the corpus):
//!   * **Escaping** of record-label metacharacters `< > { } |`.
//!   * **Fact spacing** `Name( a, b )` (a space after `(` and before `)`) versus
//!     **function spacing** `f(a, b)` (no inner spaces) — mined over the corpus;
//!     a fact / relation symbol pads, an ordinary function application does not.
//!   * **Info-cell** shape `#<temporal> : <RuleName>` optionally followed by
//!     `[<action>, …]`.

use crate::term::Term;

/// Escape the metacharacters that are special inside a graphviz record label:
/// `< > { } |` each get a leading backslash. Everything else (including single
/// quotes, `~ $ ^ * ⊕`, spaces) is literal. Observed in every record cell.
pub fn escape_record(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' | '>' | '{' | '}' | '|' => {
                out.push('\\');
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

/// A fact / relation atom occupying one premise or conclusion cell, e.g.
/// `!SessionKey( <…>, $A, H1 )`. `name` is the fact symbol (may start with `!`
/// for a persistent fact); `args` are its term arguments.
#[derive(Clone, Debug)]
pub struct Fact {
    pub name: String,
    pub args: Vec<Term>,
}

impl Fact {
    pub fn new(name: &str, args: Vec<Term>) -> Self {
        Fact { name: name.to_string(), args }
    }

    /// Flat (un-wrapped, un-escaped) surface rendering with **fact spacing**:
    /// `Name( a, b )`, and `Name( )` when it has no arguments. Argument terms use
    /// the ordinary term rendering (functions `f(a, b)` without inner spaces,
    /// tuples `<a, b>`). This matches every un-wrapped fact cell in the corpus.
    pub fn render_flat(&self) -> String {
        let inner: Vec<String> = self.args.iter().map(Term::render_full).collect();
        pad(&self.name, &inner)
    }

    /// Flat rendering with abbreviations applied to registered sub-terms
    /// (`table` maps a term's full rendering to its abbreviation name).
    pub fn render_flat_abbrev(&self, table: &std::collections::HashMap<String, String>) -> String {
        let inner: Vec<String> = self.args.iter().map(|t| t.render_abbrev(table)).collect();
        pad(&self.name, &inner)
    }
}

/// Fact spacing `Name( a, b )`, collapsing to `Name( )` for zero arguments
/// (observed: `NoA( )`, `!Semistate_1( )` — a single space between the parens).
fn pad(name: &str, args: &[String]) -> String {
    if args.is_empty() {
        format!("{}( )", name)
    } else {
        format!("{}( {} )", name, args.join(", "))
    }
}

/// The info cell of a rule instance: `#<temporal> : <rule>` plus, when the rule
/// has action facts, `[<action>, …]`. Matches the observed info-cell shape
/// (`#i : I_Complete[Complete( … ), …]`, or a bare `#vf.5 : Fresh`).
pub fn render_info(temporal: &str, rule: &str, actions: &[Fact]) -> String {
    if actions.is_empty() {
        format!("#{} : {}", temporal, rule)
    } else {
        let acts: Vec<String> = actions.iter().map(Fact::render_flat).collect();
        format!("#{} : {}[{}]", temporal, rule, acts.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_record_metachars() {
        assert_eq!(escape_record("<$A, $A, 'g'^~ex>"), "\\<$A, $A, 'g'^~ex\\>");
        assert_eq!(escape_record("a|b{c}"), "a\\|b\\{c\\}");
        assert_eq!(escape_record("no meta 'g'^~x"), "no meta 'g'^~x");
    }

    #[test]
    fn fact_flat_spacing_matches_corpus() {
        // Fr( ~ex )  — a fact pads inside its parens.
        assert_eq!(Fact::new("Fr", vec![Term::fresh("ex")]).render_flat(), "Fr( ~ex )");
        // zero-arg fact: NoA( )
        assert_eq!(Fact::new("NoA", vec![]).render_flat(), "NoA( )");
        // Out( pk(~ltkA) ) — outer fact pads; inner function pk(...) does NOT.
        let f = Fact::new("Out", vec![Term::app("pk", vec![Term::fresh("ltkA")])]);
        assert_eq!(f.render_flat(), "Out( pk(~ltkA) )");
        // !SessionKey( <$A, $A, 'g'^~ex>, $A, ~ex )
        let f = Fact::new(
            "!SessionKey",
            vec![
                Term::tuple(vec![Term::pubv("A"), Term::pubv("A"), Term::exp(Term::cst("g"), Term::fresh("ex"))]),
                Term::pubv("A"),
                Term::fresh("ex"),
            ],
        );
        assert_eq!(f.render_flat(), "!SessionKey( <$A, $A, 'g'^~ex>, $A, ~ex )");
    }

    #[test]
    fn info_cell_shape() {
        assert_eq!(render_info("vf.5", "Fresh", &[]), "#vf.5 : Fresh");
        let acts = vec![Fact::new("RegKey", vec![Term::pubv("R.4")])];
        assert_eq!(render_info("vr.9", "generate_ltk", &acts), "#vr.9 : generate_ltk[RegKey( $R.4 )]");
    }
}
