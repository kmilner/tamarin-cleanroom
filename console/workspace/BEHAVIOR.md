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
- (RESOLVED in R4) Under `--prove`, extra stderr progress lines DO appear after
  `Theory closed` — see §10a.
- Interactive-mode value validation (`--port`, `--interface`, `--image-format`):
  the server starts even on garbage values (`--port=x .` -> exit 124 server run;
  `--image-format=BOGUS .` -> server run), so there is NO CLI-level validation
  error to reproduce for those flags `[R4 interactive probes]`.

## 10. Stream separation (Round 4)  [split_* captures: <label>.out.txt / .err.txt]

The Round 1–3 oracle merged streams (`2>&1`). Re-probing with
`1>out 2>err` (probes/split_probe.sh) shows a clean, deterministic split. Each
stream is byte-complete on its own; the merged-mode interleaving of §4 was purely
a flush artifact of combining two FDs.

STDOUT carries:
- the opaque theory payload(s) `theory … end` `[split_batch_default.out]`,
- the whole "summary of summaries" block `[split_batch_default.out]`,
- the `--version`/`-V` banner (tamarin line + warranty + `Generated from:` block)
  `[split_version.out — banner ONLY; the merged version.tmpl of §4 wrongly bundled
  the maude preamble into stdout]`,
- every `--help` / per-mode help page `[split_help.out == help_global.txt]`,
- the two cmdargs VALIDATION envelopes `error: … \n\n<full help>`
  (no-input, bad-workdir) `[split_err_noinput.out == err_noinput_diff.txt,
  split_err_workdir.out == err_two_modes.txt]`.

STDERR carries:
- the 3-line maude preamble (§5) `[split_batch_default.err]`,
- the per-theory progress markers `[Theory <name>] <phase>` `[split_batch_default.err]`,
- the bare cmdargs one-liners: `Unknown flag`, `Ambiguous mode`,
  `Unhandled argument to flag` `[split_err_unknown.err == err_unknown_flag.txt,
  split_err_ambig.err == noargs.txt]`,
- all runtime errors: file-open failures and the app `error`/CallStack lines
  `[split_err_missing.err, split_err_bound.err]`.

`--parse-only` STDERR = one `Theory loaded` line per theory, no preamble; STDOUT =
concatenated payloads `[split_batch_parseonly.err/out, split_parseonly_multi]`.
`variants` STDOUT = variant payload; STDERR = preamble only `[split_variants.*]`.

### 10a. Multi-theory framing  [split_batch_multifile.*]
- STDERR: preamble once, then each theory's full 5-phase progress block in order.
- STDOUT: payloads concatenated directly (payload_k ends `end\n`, payload_{k+1}
  begins `theory …` with NO separator), then ONE combined summary listing every
  theory. Payloads are printed only for theories NOT written to an output file.

Under `--prove`, a theory's stderr progress block gains extra engine lines AFTER
`[Theory <name>] Theory closed`, verbatim `[split_batch_prove.err]`:

    [Saturating Sources] Step 1 (Max 5)
    [Saturating Sources] Done
    [Saturating Sources] Step 1 (Max 5)
    [Saturating Sources] Done

(`Max 5` = the `--saturation` limit.) A plain analyze run emits none of these. The
framing model treats these as an opaque per-theory `extra_progress` slot appended
to that theory's five closed phases; the engine supplies the exact text.

### 10b. summary-of-summaries exact layout  [split_batch_multifile.out, split_batch_output_single.out]
The summary section on STDOUT is:

    \n                                  <- separates payload from summary (always emitted)
    ==============================================================================   (78 '=')
    summary of summaries:
    <per theory, each preceded by a blank line:>
      analyzed: <PATH-as-passed-on-CLI>
      <blank>
      [  output:          <OUTPUT-PATH>]   <- only when an output file was written
        processing time: <T>s
      <2-space line "  ">
      <result lines, each "  <text>">
    <blank>
    ==============================================================================
    (final newline)

Key/value alignment inside the indented block: labels right-padded to the width of
`processing time:` (16), then one space, so values start at column 19 →
`  output:` + 10 spaces + value; `  processing time: ` + value. `analyzed:` is a
separate un-indented header line with a single space. `output:` is the label for
BOTH `-o FILE` and `-O DIR` (the latter writes `<inputstem>_analyzed.spthy`).
`analyzed:` echoes the path exactly as given on the command line (relative stays
relative) `[R4 relative-path probe]`.

## 11. Typed value validation (Round 4)

Validation fires ONLY when a value string is present (via `=value`, `--oj=…`, or a
short-attached `-b5`); a BARE value flag (`--bound`, `--stop-on-trace`, …) never
errors `[R4 bare-vs-valued sweep]`. An empty `=` value IS a present value and is
validated (`--bound=` → error, `--stop-on-trace=` → `… method: `). All these errors
go to STDERR, exit 1, and use the app `error` shape
`tamarin-prover: <msg>\nCallStack (from HasCallStack):\n  error, called at
src/Main/Mode/Batch.hs:162:33 in main:Main.Mode.Batch`. In a normal run they follow
the maude preamble; under `--parse-only` no preamble precedes them.

Integer flags → `<LABEL>: invalid bound given` `[R4 numeric sweep]`:
`--bound`→`bound`, `-c/--open-chains`→`OpenChainsLimit`,
`-s/--saturation`→`SaturationLimit`, `-d/--derivcheck-timeout`→`derivcheck-timeout`,
`--replication-bound`→`replication-bound`.
Accepted integer set = Haskell `readMaybe Int`: trims leading/trailing ASCII
whitespace, optional `-` (with optional whitespace before the digits), decimal /
`0x`hex / `0o`octal, silent overflow wrap; REJECTS `+`, `.`, `_`, trailing
non-whitespace, and empty. Negatives and 0 are accepted for ALL of them (the
"PositiveInteger" help annotation is not enforced at the CLI).

Enum flags:
- `--stop-on-trace`: value lowercased, valid ∈ {dfs,bfs,seqdfs,sorry,none};
  invalid → `unknown stop-on-trace method: <lowercased-value>` (echoes value).
- `--partial-evaluation`: case-insensitive, valid ∈ {summary,verbose};
  invalid → `partial-evaluation: unknown option` (no echo).
- `--output-module` (`-m`): case-SENSITIVE, valid ∈
  {spthytyped,spthy,msr,proverifequiv,proverif,deepsec};
  invalid → `output mode not supported.` (no echo).
`--heuristic` is NOT validated (any chars accepted).

Repeated valued flag → LAST occurrence wins / is the one validated
`[R4 repeated probe]`.

Multi-invalid precedence is a FIXED order, independent of CLI arg order
`[R4 precedence sweep, all pairwise relations consistent]`:
`stop-on-trace > bound > partial-evaluation > open-chains > saturation >
output-module > derivcheck-timeout > replication-bound`.

### 11a. Consumer-extension flags (NOT in the reference, NOT in help)
`--processors <positive int>`, `--maude-processes <positive int>`,
`--data-dir <path>` all yield `Unknown flag: --<name>` in the reference
`[R4 interop probe]`, confirming they are absent. The clean crate accepts them as
typed flags with its OWN validation (positive integer; any path); their rejection
texts are original (no reference to reproduce) and they are omitted from help.
