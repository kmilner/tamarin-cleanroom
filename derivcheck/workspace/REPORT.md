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

---

# Round 4 — closing five derivation-check gaps

All five gaps were pinned by fresh black-box probes (authored `probes4/*.spthy`, run through
`oracle/hs_oracle.sh`; every observation logged in QUERIES.log under `ROUND 4`) and the crate
`workspace/derivcheck-clean` was updated to match. `cargo test` → 28 passed, warning-clean; the
two-rule block is byte-identical (297 bytes, `cmp`) to the captured reference fixture
`probes4/fixture_two_rule.txt` (verified by `examples/dump_two_rule.rs`).

## GAP 1 — output contract (one value, heading included)
`message_derivation_checks` now returns the WHOLE topic as a SINGLE `WfError` whose `message` is
the complete block INCLUDING the underlined `Message Derivation Checks` heading + 25-`=`
underline (previously it returned intro + per-rule fragments assembled by a separate renderer).
The consuming report renderer adds no per-topic heading; topic blocks are joined by one blank
line, which the reference confirms (blank line between the Subterm and Derivation blocks). The
returned message is byte-exact and carries no trailing newline. `render_derivation_report` now
just returns that single message.

## GAP 2 — multi-variable ordering (key was wrong)
Discriminating probes (the prior fixtures all used index 0 and could not tell the keys apart)
establish the true total order: **`(index numeric, sort-rank Fresh<Msg<Nat, name byte/ASCII)`**
— index is PRIMARY, not the sort. Evidence: `In(h(~a.2)),In(h(b))` → `b, ~a.2` (idx-0 msg before
idx-2 fresh); `In(h(<z.1,a.2,~z.2,~a.1,m>))` → `m, ~a.1, z.1, ~z.2, a.2`; `In(h(<apple,Zebra>))`
→ `Zebra, apple` (ASCII, uppercase first). The old `(sort_rank, name, idx)` key is replaced by
`(idx, sort_rank, name)` in `var_order_key`; `sort_rank` gains `Nat=2`. Trailing digits without
a `.` stay in the name at index 0 (`x2`,`v10`).

## GAP 3 — candidate scope
Probed which classes can EVER be flagged. Candidate set = every free variable of the expanded
rule across premises, actions AND conclusions: an action-only var (`z`) and a conclusion-only var
(`k`) are both flagged. Public (`$p`) is a candidate but the solver always resolves it derivable
(never flagged; the unit does not pre-filter). Natural-number vars (`%x`) CAN be flagged and are
rendered `%`. Temporal/node vars are a parse error in message-term position — never candidates. A
name matching a declared nullary function is a constant (`Term::App(name, [])`), contributing no
variable — handled by the term walk. Candidate enumeration was widened to include actions and
conclusions accordingly.

## GAP 4 — pre-expansion
Derivability is decided AFTER macro and `let` expansion; the reported names are the post-expansion
inner names (`let y = h(w) in In(y)` → `w`; `let y = w in In(h(y))` → `w`, never `y`;
`macros mac(x)=h(x); In(mac(w))` → `w`). The unit now expands the theory-level `macros:` table and
the rule's `let` bindings into the rule's fact terms (`expand_rule`, `expand_term`, `subst_vars`,
`let_substitution`) before enumerating candidates; let-bound LHS names are eliminated and never
become candidates.

## GAP 5 — solver interface shape (batched)
`DerivabilitySolver` is now batched: `check_rule(&RuleProbe)` receives one rule with ALL its
candidate variables and returns one verdict per variable — one saturation per rule, no wasteful
per-variable re-saturation. The per-variable path is retained as a thin adapter: `PerVariableSolver`
+ `PerVariable<S>` wrapper. A recording-stub test asserts the solver is consulted exactly once per
rule with the full candidate batch; fixtures/tests were updated to the batched trait.
