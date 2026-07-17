# BEHAVIOR.md — Unit D: CLI/console text surface

Every statement below traces to an oracle observation captured under
`workspace/captures/` (file names in `[brackets]`) or to the SPEC. No
tamarin-prover internals were read; all behavior derived from black-box probing
of `oracle/hs_oracle.sh`.

Oracle note: `hs_oracle.sh <arg1> [rest...]` consumes `arg1` as the first
token then appends the rest, i.e. it runs `tamarin-prover <arg1> <rest...>`
with stderr merged into stdout. A true zero-argument invocation is therefore
NOT observable through this oracle (it always passes at least one token); the
closest observable is a single empty-string token (see Ambiguous mode).

## 1. Top-level shape

The binary is a multi-mode command (cmdargs-style). The first token selects a
**mode** by unique prefix match against the mode names; otherwise the default
(batch) mode is used and the token is treated as an input FILE.

Modes enumerated from help output `[help_global]`:
- (default / no token) = batch mode — positional `FILES`.
- `interactive` — "Start a web-server to construct proofs interactively." — positional `WORKDIR`.
- `variants` — "Compute the variants of the intruder rules for DH-exponentiation." — no positional.
- `test` — "Self-test the tamarin-prover installation." — positional `FILES`.

Mode prefix matching `[mode_prefix_int/_v/_t]`: `int`→interactive, `v`→variants,
`t`→test (any unambiguous non-empty prefix works). An empty token matches all
three → ambiguous `[noargs]`.

A flag appearing BEFORE a would-be mode token keeps the default batch mode; the
token then becomes a filename `[flag_before_mode]`: `-v interactive` →
default mode, "interactive" opened as a file → openFile error.

## 2. Exit codes

- `--help` / per-mode `--help`, `--version`/`-V`, successful batch run: exit 0.
- Every parse error and every runtime error observed: exit 1.

## 3. `--help` (global and per-mode)  [help_global, help_interactive, help_variants, help_test]

Each mode's help has the same skeleton:

    <mode-usage-line>
      <mode-description>.

    Commands:
      interactive  Start a web-server to construct proofs interactively.
      variants     Compute the variants of the intruder rules for DH-exponentiation.
      test         Self-test the tamarin-prover installation.

    Flags:
      <flag rows, two-column, right column wrapped to a fixed width>
    About:
      <help/version rows>

    <blank>
    ------------------------------------------------------------------------------
    To show help for different commands, type tamarin-prover [Command] --help.
    ------------------------------------------------------------------------------
    See 'https://github.com/tamarin-prover/tamarin-prover/blob/master/README.md'
    for usage instructions and pointers to examples.
    ------------------------------------------------------------------------------
    <blank>

Usage lines:
- global: `tamarin-prover [COMMAND] ... [OPTIONS] FILES`
- interactive: `interactive [COMMAND] ... [OPTIONS] WORKDIR`
- variants: `variants [COMMAND] ... [OPTIONS]`   (no positional)
- test: `test [COMMAND] ... [OPTIONS] FILES`

The Commands block is IDENTICAL in all four helps. The footer block (7 lines
ending with a trailing blank line) is identical in all four. Every help output
ends with `----\n\n` (footer rule + one trailing empty line).

The right-hand description column width differs by mode because cmdargs sizes
the left column to the widest flag in THAT mode: global uses a very wide left
column (widest flag `-m --output-module[=...]`), the other three narrower.
Because the exact wrapping/padding is compatibility output, the four help texts
are stored verbatim as fixtures and emitted as opaque strings — the clean code
does not re-derive column math.

Availability of About rows:
- default/global mode: `-? --help` AND `-V --version`.
- interactive / variants / test: only `-? --help` (no `--version`). Confirmed:
  `variants --version` → `Unknown flag: --version` `[version_from_mode]`.

### Flag inventory & value semantics (from help text)

Value markers: `[=X]` = value OPTIONAL; `=FILE` (no brackets) = value shown
required (but see note). Defaults appear in the description text.

Default/batch mode flags:
- `--prove[=LEMMAPREFIX*|LEMMANAME]`     optional value, repeatable
- `--lemma[=LEMMAPREFIX*|LEMMANAME]`     optional value, repeatable
- `--stop-on-trace[=DFS|BFS|SEQDFS|SORRY|NONE]`  optional, default DFS
- `-b --bound[=INT]`                     optional
- `--heuristic[=(C|I|O|P|S|c|i|o|p|s|{.})+]`  optional, default 's'
- `--partial-evaluation[=SUMMARY|VERBOSE]`   optional
- `-D --defines[=STRING]`                optional
- `--diff`                               no value
- `--quit-on-warning`                    no value
- `--auto-sources`                       no value
- `--oraclename[=FILE]`                  optional; default './theory_filename.oracle', fallback './oracle'
- `--oracle-only`                        no value
- `--quiet`                              no value
- `-v --verbose`                         no value
- `-c --open-chains[=PositiveInteger]`   optional, default 10
- `-s --saturation[=PositiveInteger]`    optional, default 5
- `-d --derivcheck-timeout[=INT]`        optional, default 5 (0 deactivates)
- `--no-reuse`                           no value
- `--no-restrictions`                    no value
- `--replication-bound[=INT]`            optional
- `--no-compress`                        no value
- `--parse-only`                         no value
- `--precompute-only`                    no value
- `-o --output[=FILE]`                   optional
- `-O --Output[=DIR]`                    optional
- `-m --output-module[=spthytyped|spthy|msr|proverifequiv|proverif|deepsec]`  optional enum
- `--output-json=FILE` (alias `--oj`)    value shown required; NO parse error when omitted [oj_alias_noval,outputjson_noval]
- `--output-dot=FILE`  (alias `--od`)    value shown required
- `--with-dot[=FILE]`                    optional
- `--with-json[=FILE]`                   optional
- `--with-maude[=FILE]`                  optional
- About: `-? --help`, `-V --version`

interactive mode adds: `-p --port[=PORT]`, `-i --interface[=INTERFACE]`,
`--image-format[=PNG|SVG]` (default SVG), `--debug`, `--no-logging`; plus the
proving/output subset (--prove … --with-maude minus the file-output-module
flags: no `--no-compress/--parse-only/--precompute-only/-o/-O/-m/--output-json/
--output-dot`). About: only `-? --help`.

variants mode: only `-O --Output[=DIR]`; About `-? --help`.

test mode: only `--with-dot/--with-json/--with-maude`; About `-? --help`.

## 4. `--version` and `-V`  [version, short_version]

`--version` and `-V` produce identical output (only valid in default mode).
The maude readiness check runs concurrently and its lines INTERLEAVE with the
version banner in a fixed, reproducible order:

    maude tool: '<MAUDE_PATH>'
     checking version: <BANNER line 1: tamarin-prover <VER>, (C) <authors>, <YEARS>>
    <blank>
    This program comes with ABSOLUTELY NO WARRANTY. It is free software, and you
    are welcome to redistribute it according to its LICENSE, see
    'https://github.com/tamarin-prover/tamarin-prover/blob/master/LICENSE'.
    <blank>
    <MAUDE_VER>. OK.
     checking installation: OK.
    Generated from:
    Tamarin version <VER>
    Maude version <MAUDE_VER>
    Git revision: <GITREV> (with uncommited changes), branch: <BRANCH>
    Compiled at: <TIMESTAMP>

The banner text (` checking version: ` prefix + tamarin line + warranty block)
is emitted between the maude check's "checking version:" and its "<ver>. OK."
because the maude subprocess is being awaited. Dynamic/environment slots:
`<MAUDE_PATH>` (default 'maude'), `<VER>` (observed 1.13.0), `<YEARS>`
(2010-2023), `<MAUDE_VER>` (3.5.1), `<GITREV>` + dirty-suffix + `<BRANCH>`,
`<TIMESTAMP>`. These are build/runtime metadata, injected into the template.

## 5. maude preamble (framing prelude)

Batch/variants runs that CLOSE a theory (need maude) print a 3-line prelude
`[batch_nslpk3_default]`:

    maude tool: '<MAUDE_PATH>'
     checking version: <MAUDE_VER>. OK.
     checking installation: OK.

`--parse-only` does NOT print the preamble `[batch_nslpk3_parseonly]` (parsing
needs no maude). Runtime file-open errors are printed AFTER this preamble.

## 6. Batch-mode framing around a processed theory

Full default run `[batch_nslpk3_default]`:

    <maude preamble (3 lines)>
    [Theory <NAME>] Theory loaded
    [Theory <NAME>] Theory translated
    [Theory <NAME>] Derivation checks started
    [Theory <NAME>] Derivation checks ended
    [Theory <NAME>] Theory closed
    <OPAQUE THEORY PAYLOAD, from `theory <NAME>` through `end`>
    <blank>
    ==============================================================================
    summary of summaries:
    <blank>
    analyzed: <ABSOLUTE_FILE_PATH>
    <blank>
      processing time: <T>s
      <-- literal 2-space line
    <RESULT LINES>
    <blank>
    ==============================================================================

- `====` rule is exactly 78 `=`. Output ends `====\n` (no trailing content).
- `--prove` uses the SAME 5 progress phases `[batch_nslpk3_prove]`.
- `--parse-only` prints ONLY `[Theory <NAME>] Theory loaded`, then payload; NO
  preamble, NO summary block `[batch_nslpk3_parseonly]`.
- The "Generated from:" version comment and any `WARNING:` block are part of the
  OPAQUE theory payload (inside `theory … end`), not framing.

RESULT LINE forms (2-space indent):
- `  <lemma> (all-traces): analysis incomplete (<N> steps)`  [default, unselected]
- `  <lemma> (all-traces): verified (<N> steps)`             [proved true]  `[batch_nslpk3_prove]`
- `  <lemma> (exists-trace): <result> (<N> steps)`           [exists-trace kind]
- (form `falsified (<N> steps)` inferred as the counterpart to `verified`; not
  directly captured — see LIMITATIONS.)
- `  WARNING: <N> wellformedness check failed!`  [when checks failed; note the
  singular "check"] `[round2_multiplication_in_rule_lhs]`

When a theory has warnings but no lemmas, the WARNING line is the sole result
line. Ordering of the WARNING line vs lemma lines when BOTH are present is not
observed (LIMITATIONS).

`<T>` (processing time) and `<ABSOLUTE_FILE_PATH>` are dynamic slots.

## 7. variants mode run  [mode_variants_run]

`variants` (no file): maude preamble, then the pretty-printed intruder
variant rules (opaque payload), NO summary block.

## 8. Error text taxonomy

### 8a. cmdargs parse errors — bare one-liner, exit 1, no help:
- Unknown long flag `[err_unknown_flag]`:  `Unknown flag: --foobar`
- Unknown short flag `[err_unknown_short]`: `Unknown flag: -Z` (note: `-h` is
  NOT help; help is `-?`/`--help`) `[short_help]`
- Value given to a valueless flag `[help_eq]`: `Unhandled argument to flag, none expected: --help=foo`
- Ambiguous mode `[noargs]`: `Ambiguous mode '', could be any of: interactive variants test`

### 8b. cmdargs validation errors — message + blank + FULL help of the active mode, exit 1:
- No input file (flags otherwise valid) `[err_stopontrace_bad, err_noinput_diff, err_noinput_prove]`:
      error: no input files given
      <blank>
      <global help body>
  Envelope = `error: no input files given\n\n` + exact global help.
- Bad interactive WORKDIR `[err_two_modes, interactive_two_pos]`:
      error: directory '<DIR>' does not exist.
      <blank>
      <interactive help body>
  (`interactive test` → 'test' is the WORKDIR; `interactive aa bb` → names 'bb'.)

### 8c. runtime errors — printed AFTER the maude preamble, exit 1:
File open (IOException, no CallStack):
- missing `[err_missing_file, token_notfile]`:  `tamarin-prover: <path>: openFile: does not exist (No such file or directory)`
- unreadable `[err_unreadable]`:                 `tamarin-prover: <path>: openFile: permission denied (Permission denied)`
- directory `[err_isdir]`:                       `tamarin-prover: <path>: openFile: inappropriate type (is a directory)`

Application `error` with Haskell CallStack (all cite the same call site):
- bad bound `[err_bound_nonint]`:
      tamarin-prover: bound: invalid bound given
      CallStack (from HasCallStack):
        error, called at src/Main/Mode/Batch.hs:162:33 in main:Main.Mode.Batch
- bad output module `[err_output_module_bad]`:
      tamarin-prover: output mode not supported.
      CallStack (from HasCallStack):
        error, called at src/Main/Mode/Batch.hs:162:33 in main:Main.Mode.Batch

These runtime strings (incl. the CallStack source path) are compatibility
output taken verbatim from the oracle; they are emitted by the runtime layer
after parse(), so the clean crate exposes them as render helpers rather than
from parse().

## 9. LIMITATIONS / unobserved (documented, not guessed in code)
- True zero-argument behavior: not observable through this oracle.
- `falsified (<N> steps)` exact string: inferred counterpart of `verified`; not
  directly captured (no small example produced it within probe budget).
- WARNING-line vs lemma-line ordering in the summary when both present.
- Whether `--output-json`/`--output-dot` error at output time when value omitted
  (only parse-time / --parse-only observed, where no error occurs).
- Additional per-proof progress lines under heavy `--prove` (none seen on NSLPK3).
