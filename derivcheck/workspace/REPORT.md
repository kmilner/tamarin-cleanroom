# REPORT — Unit G: Message Derivation Checks (clean-room)

## Scope
Clean-room reimplementation of the `Message Derivation Checks` wellformedness
topic emitted by the reference tamarin-prover binary. Everything below is derived
solely from black-box oracle observation (`oracle/hs_oracle.sh`), the captured
example outputs, and the SPEC. No file under `/home/kamilner/tamarin-rs/` was
read; the only oracle interaction was executing the wrapper script (see
`workspace/QUERIES.log`). The parsed-theory input model is mirrored from the
provided interface header `../wellformedness/interface/ast_types.rs`.

## Deliverable
Crate `workspace/derivcheck-clean/`:
- `src/ast.rs` — mirror of the interface header (type surface only), so the crate
  compiles/tests standalone; replaced by the real AST module at integration.
- `src/lib.rs` — the check, the callback trait, the decision logic, the renderer,
  and 15 unit tests (solver stubbed). `cargo test` -> 15 passed, clean build.

## Architecture (callback abstraction)
The "can the intruder derive X from Y" decision is the prover's solver and is NOT
reimplemented. It is supplied by the caller through:

    pub struct DerivProbe<'a> {              // one question per (rule, variable)
        pub rule_name: &'a str,
        pub rule: &'a Rule,                  // full context (Y = premises, etc.)
        pub variable: &'a VarSpec,           // X
        pub premises: &'a [Fact],
        pub timeout_secs: u64,
    }
    pub enum Derivability { Derivable, NotDerivable, TimedOut }
    pub trait DerivabilitySolver { fn check(&self, probe: &DerivProbe) -> Derivability; }

This crate owns everything except the derivability verdict:
- Probe construction. `candidate_variables(rule)` = every distinct variable
  occurring in the rule's premises/actions/conclusions/let-bindings (a safe
  superset; the solver resolves public / `Fr`-bound / state-bound variables to
  `Derivable`). Rules are visited in theory order; `IntrRule` items and rules
  carrying `[no_derivcheck]` are excluded.
- Decision logic. `timeout_secs == 0` deactivates the check (empty report, solver
  never consulted). `Derivable` -> skip; `NotDerivable` -> collect; `TimedOut` ->
  per `TimeoutPolicy` (default `SuppressRule`).
- Presentation. Sort/dedup the flagged variables and render the byte-exact block.

Entry points: `message_derivation_checks(thy, solver, timeout_secs)` and the
policy-explicit `message_derivation_checks_with(...)`; plus `underline_topic`,
`topics`, `render_derivation_report`, `render_variable`, `candidate_variables`,
`sort_variables`.

## Observed behavior encoded (full detail in BEHAVIOR.md)
- Topic string `Message Derivation Checks`, underlined by 25 `=` (= title length),
  emitted after `Subterm Convergence Warning` in report order.
- Block: header, blank line, a fixed intro paragraph (2-space indent), then one
  block per failing rule "Rule <name>: \nFailed to derive Variable(s): <vars>"
  (note the trailing space after the colon), blocks separated by blank lines.
- Rules in theory (source) order; derivable rules omitted; `[no_derivcheck]`
  suppresses a rule.
- Reported variables sorted by (sort-rank, name lexicographic, index numeric)
  with Fresh < Msg; public names never appear; duplicates collapse. Rendering:
  fresh `~name`, public `$name`, message `name`, nonzero index `name.idx`.
- `--derivcheck-timeout=INT` (default 5, `=0` deactivates; the `=` is required —
  `-d 1` mis-parses `1` as a filename).

## Byte-parity evidence
The two-rule test string was diffed against the real POIDC_CMB capture:
297 bytes each, `cmp` identical. The single-rule and two-rule tests assert the
renderer's output equals those byte-exact strings, so the crate reproduces the
oracle block byte-for-byte for the observed cases.

## Residuals / limits
- Per-rule timeout output text is unobserved. The derivation check is robustly
  fast (even an 8-term xor rule finishes it quickly; the slowness there is later
  precomputation), so a genuine per-rule timeout could not be forced with
  practical inputs. The `TimedOut` outcome is modeled and the default policy
  (`SuppressRule`, fail-open) follows the only timeout-related observation
  (`=0` => no warnings). The alternative (`SkipVariable`) is a single enum arm.
- Sort-rank beyond Fresh < Msg and Nat/Node/Pub rendering prefixes were not
  exercised because such variables are never reported (always derivable / not
  message terms). They are placed last / rendered conservatively, flagged in
  comments.
- The crate does not emit the driver's phase-log lines
  (`[Theory X] Derivation checks started/ended`) or compute the
  `WARNING: N wellformedness check failed!` count — those belong to the
  wellformedness driver, not this topic.

## Integration contract
`message_derivation_checks` returns `Vec<WfError>` sharing topic
`Message Derivation Checks`: first entry = the intro paragraph, then one entry per
failing rule. A report writer renders a topic as
`underline_topic(topic) + "\n" + entries.join("\n\n")`, which
`render_derivation_report` implements for this topic (verified byte-exact). At
integration, thin adapters map these into the shared report and wire the real
solver as the `DerivabilitySolver`.
