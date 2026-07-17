# Unit E: macro expansion

Build crate macro-clean in workspace/ implementing .spthy macro semantics as
observed: theories in oracle/examples/ use `macros:`; the oracle's
pretty-printed output shows their EXPANSION into rules. Characterize: macro
definition syntax (incl. arity, nesting, let-style substitution), call-site
expansion semantics, name capture/collision behavior (probe edge cases:
recursive use, shadowing, arity mismatch errors), and where expansion is
visible vs preserved in output. Input model: the theory AST in
../wellformedness/interface/ast_types.rs (includes macro fields — study it).
Deliverable: expand(theory) -> theory (expanded) with byte-parity fixtures
(oracle output on macro theory == oracle output on your pre-expanded
equivalent where observable, plus direct expansion checks). BEHAVIOR.md +
QUERIES.log + tests; REPORT.md + 10-line summary.
