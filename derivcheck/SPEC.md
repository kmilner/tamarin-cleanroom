# Unit G: message derivation checks

Build crate derivcheck-clean in workspace/. The oracle emits wf topics
"Message Derivation Checks" / "Derivation Checks" (appear AFTER Subterm
Convergence in report order) for theories where lemma/rule variables are not
derivable by the intruder — oracle/examples/ has private-function theories
that trigger them; also probe --derivcheck-timeout if help shows it.
ARCHITECTURE REQUIREMENT: the underlying "can the intruder derive X from Y"
decision is made by the prover's solver, which you do NOT reimplement. Your
crate must be parameterized over a caller-supplied decision callback (design
a small trait, e.g. fn check(&probe) -> Outcome with whatever probe payload
your observed semantics need). You implement: which probes get constructed
for a theory (characterize by observation: which variables/lemmas/rules the
warnings mention), the warning DECISION logic around callback outcomes
(timeouts, successes), and byte-exact message/report text. Input model:
../wellformedness/interface/ast_types.rs. BEHAVIOR.md + QUERIES.log + tests
(callback stubbed in tests); REPORT.md + 10-line summary.
