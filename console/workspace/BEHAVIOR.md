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
- (RESOLVED in R5) `falsified` verdict string: directly captured — it is NOT a
  bare `falsified` but a kind-dependent `falsified - found trace` (all-traces) /
  `falsified - no trace found` (exists-trace). See §12c.
- (RESOLVED in R5) WARNING-line vs lemma-line ordering when both present: the
  warning section comes first, separated from the lemma lines by a `  ` line,
  and gains an advisory second line under `--prove`. See §12a–b.
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

## 12. Summary body content (Round 5)  [r5_* captures]

The `<result lines>` slot of §6/§10b (the summary-of-summaries body under the
`processing time:` line) has more structure than §6 recorded. Probed with
self-authored theories `probes/round5/*.spthy` (own fixtures, not the GPL
examples), split-stream, on both `--prove` and default runs.

### 12a. Body layout — an opening line plus up to two sections
After the `  processing time: <T>s` line the body is, byte-exactly:

    "  \n"                                   <- opening two-space line (ALWAYS present)
    <warning section, if any>                <- comes first
    "  \n" between the two sections          <- present iff BOTH sections present
    <lemma section, if any>

i.e. an opening `  ` line, then the present sections joined by another `  ` line.
- No warnings, no lemmas → body is just `  \n` `[r5_nolemma_default]`.
- Lemmas only → `  \n` + lemma lines `[split_batch_default / r5_exists_verified]`.
- Warning only → `  \n` + warning section `[round2_fresh_public_constant / r5_freshpub_prove]`.
- Warning + lemmas → `  \n` + warning + `  \n` + lemmas `[r5_warn_lemma_*]`.

### 12b. Warning section — one or two lines
    "  WARNING: <N> wellformedness check failed!\n"   (noun stays singular "check")
    "           The analysis results might be wrong!\n"   (SECOND line, conditional)
The advisory second line is left-padded 11 spaces — exactly the width of the
prefix `  WARNING: ` — so it aligns under the text after `WARNING: `. It is
emitted **iff the run is a proving run (`--prove`)**, independent of whether any
lemma was actually proved: a `--prove` run whose lemma prefix matches nothing
(all lemmas "analysis incomplete") still emits it `[r5_falsify_nomatch]`, and a
proving run with a warning but no lemmas emits it with no following separator
`[r5_freshpub_prove]`; a plain analyze run never does `[r5_warn_lemma_default]`.
A run with no warning never shows this line regardless of `--prove`
`[r5_nslpk3_prove_bound2, r5_nolemma_prove]`.

### 12c. Lemma verdict lines — `  <name> (<kind>): <verdict> (<N> steps)`
Observed `<verdict>` forms (the step count is the explored proof depth):
- `verified` — uniform for BOTH `all-traces` and `exists-trace`
  `[r5_exists_verified: can_ping/always_true both "verified"]`.
- `analysis incomplete` — default/unselected lemmas, AND `--prove` lemmas cut off
  by `--bound` (only the step count grows; NO extra line for the bound)
  `[r5_nslpk3_prove_bound2: nonce_secrecy "analysis incomplete (6 steps)"]`.
- `falsified` — carries a **kind-dependent suffix** `[r5_falsify_prove]`:
  - all-traces → `falsified - found trace` (a counter-example trace was produced),
  - exists-trace → `falsified - no trace found` (no witnessing trace exists).
  (Corrects the R2 guess that falsified prints as a bare `falsified`.)

### 12d. Between-theory joining (multi-file)  [r5_multi_warn_prove]
Each theory's block is preceded by a single `\n` (blank line) before its
`analyzed:` header — unchanged from §10b; confirmed with a lemmas-only theory
followed by a warning+lemma theory. The per-theory warning/lemma structure of
§12a–c is entirely local to each block.

Corresponding STDERR (unchanged framing, §10a): preamble once, then per theory
its five closed phases + any `[Saturating Sources] …` extra progress; the number
of saturating-sources lines is theory-specific (NSLPK3 emits four, the tiny
`WarnAndLemma`/`FreshPubConst` theories emit none) — treated as the opaque
per-theory `extra_progress` slot.

## 13. Streaming emission contract (Round 5, INTEROP)

The assembled framing of §6/§10 hands back complete `Streams` at the end. A
consumer that flushes output *as the prover runs* needs to drive emission
event-by-event. The clean crate adds an incremental surface (`src/emit.rs`) that
is a pure re-ordering of the SAME byte content — no new observable strings, so no
new oracle facts; it is an internal API shape, verified by equivalence against
the assembled model rather than against the oracle:
- `Sink::emit(stream, text)` — the consumer's byte destination; the consumer
  chooses flush timing (flush-per-call = true line streaming; buffering = coalesced).
- `BatchEmitter` — `begin` (preamble→stderr) / `progress`,`closed_phases`,
  `extra_progress` (→stderr) / `payload` (→stdout) / `record_summary` (buffered) /
  `finish` (combined summary→stdout). The summary is the one unavoidable trailing
  barrier (it spans every theory), matching the reference which prints it last.
- Equivalence property: for identical inputs the emitter's per-stream bytes equal
  `frame_batch`'s. The cross-stream interleaving differs in wall-clock time (the
  point of streaming) but each stream's byte sequence is identical, because on
  each stream both paths emit in the same order: stderr = preamble then per-theory
  progress; stdout = payloads in order then the single summary. Proven by tests
  `streaming_matches_assembled_on_shared_inputs` and
  `streaming_reproduces_captured_multifile_streams`.

## 14. Completed lemma-verdict taxonomy (Round 6)  [r6_* captures]

Round 5 recorded three regular-lemma verdict phrases. Round 6 drove every lemma
into every reachable terminal state to close the taxonomy. Probe theories are
self-authored in `probes/round6/*.spthy`; diff summary captures are copied to
`tests/fixtures/r6_diff_*.out.txt`.

### 14a. The verdict-phrase set is exactly four (regular lemmas)
`<verdict>` in `<name> (<kind>): <verdict> (<N> steps)` is one of:
- `verified` — uniform for both kinds (§12c).
- `falsified - found trace` — all-traces (§12c).
- `falsified - no trace found` — exists-trace (§12c).
- `analysis incomplete` — unselected, `--prove` pattern not matching (§12b/R5
  `r5_falsify_nomatch`), or cut off by `--bound` (§12c). No further phrase appears
  under `--stop-on-trace=SORRY|NONE` `[r6_sot_sorry, r6_sot_none]`, reloading a
  partially/fully embedded proof `[r6_analysed_reload: only verified/falsified]`,
  or for a `[sources]`-attributed lemma `[r6_partialdecon]`. The `[sources]`
  attribute is invisible in the summary — such a lemma prints with the identical
  line form (`secret (all-traces): falsified - found trace (4 steps)`).

Lemma SELECTION never omits a line: a lemma the run did not prove — because
`--prove=PREFIX`/`--lemma=NAME` did not select it — still appears, as `analysis
incomplete`, never as a "skipped"/absent line `[r6_twolemma_proveone,
r6_twolemma_lemmafilter]`. `--prove=first` on a two-lemma theory prints `first`
verified and `second (exists-trace): analysis incomplete (1 steps)`. There is thus
no distinct summary form for an unselected/filtered lemma. A `--prove`/`--lemma`
PREFIX that matches no lemma is itself a wellformedness check ("... does not
correspond to a specified lemma in the theory") and so feeds the whole-theory
WARNING count-line slot (§12b), not a new per-lemma form `[r6_twolemma_lemmafilter:
count 1 + advisory]` — reinforcing §14e.

Partial deconstructions / open chains do NOT add a summary-of-summaries line: a
theory that exercises them prints exactly one verdict line per lemma
`[r6_openchains_default, r6_woolam_noautosrc]` (they only add opaque stderr
`[Open Chains] …` progress and, when they trip a wellformedness rule, feed the
existing WARNING count-line slot).

### 14b. `--diff` adds three prefixed line forms  [r6_diff_* captures]
Under `--diff` the summary body's lemma section gains side-prefixed lines. Byte
layout (each still carries the `  ` body indent of §12a):

    "  RHS :  <name> (<kind>): <verdict> (<N> steps)\n"   projected ordinary lemma
    "  LHS :  <name> (<kind>): <verdict> (<N> steps)\n"   projected ordinary lemma
    "  DiffLemma:  <name> : <verdict> (<N> steps)\n"      observational equivalence

- `RHS`/`LHS` prefix = `RHS` + ` :  ` (space, colon, two spaces); the remainder is
  the *identical* regular line form of §12c (the `<kind>` renders normally,
  including `exists-trace` `[r6_diff_two_lemmas: can_send]`).
  A projected line carries the SAME kind-dependent verdict set as a whole-theory
  lemma — earlier captures only exercised `verified`; the remaining projected
  verdicts were driven out explicitly:
  - projected falsified, both kinds: `RHS/LHS :  leaked (all-traces): falsified -
    found trace` and `RHS/LHS :  impossible (exists-trace): falsified - no trace
    found` `[r6_diff_false]`;
  - projected `analysis incomplete`: a `--diff` run WITHOUT `--prove` leaves each
    ordinary lemma's `RHS`/`LHS` lines (and the `DiffLemma`) at `analysis
    incomplete` `[r6_diff_lemma_noprove]`.
  The two sides are computed INDEPENDENTLY, not mirrored: when the two projected
  systems genuinely differ, one side's line can read `verified` while the other
  reads `falsified` for the same lemma (`RHS :  secret (all-traces): verified` vs
  `LHS :  secret (all-traces): falsified - found trace`) `[r6_diff_asym]`. `LHS`
  is the first `diff(L,R)` argument (the left system), `RHS` the second (the right
  system): here `diff(~m, senc(~m,~k))` gives LHS `Out(~m)` (secrecy false) and
  RHS `Out(senc(~m,~k))` (secrecy true).
- `DiffLemma` prefix = `DiffLemma` + `:  ` (colon, two spaces — NO space before the
  colon, unlike RHS/LHS). Its content is `<name> : <verdict>` (a single space each
  side of the colon) with NO `(<kind>)`: an observational-equivalence lemma has no
  trace-kind. Its `falsified` always reads `- found trace` (a distinguishing
  attack) `[r6_diff_n5n6: 8 steps; r6_diff_warn: 7 steps]`; `verified`
  `[r6_diff_equiv_true: 31 steps]`; `analysis incomplete` `[r6_diff_default]`.
- The auto-generated diff lemma is always named `Observational_equivalence`.

### 14c. Diff summary ordering  [r6_diff_with_lemma, r6_diff_two_lemmas, r6_diff_warn]
Within one theory's body the lemma section is, in order:
1. for EACH ordinary lemma, its `RHS` line then its `LHS` line (the pair is
   grouped per lemma: `RHS l1, LHS l1, RHS l2, LHS l2, …`), then
2. the single `DiffLemma:` line, last.
The surrounding body structure of §12a is unchanged: opening `  ` line, then the
warning section first (with the §12b `--prove` advisory), then a `  ` separator,
then this lemma section `[r6_diff_warn]`.

### 14d. Diff STDERR (documented, opaque — NOT byte-modelled)
Diff runs prefix the per-theory progress with a bare `diffLemma\n` line right
after the maude preamble, and the `[Saturating Sources] …` / `[Open Chains] …`
engine markers interleave *within* the five phases (not only after `Theory
closed`, unlike §10a) and in run-varying multiplicity `[r6_diff_* .err]`. Because
that interleaving is engine-timing-driven, the Round-6 fixtures assert only the
STDOUT summary section (via `render_summary`); diff stderr is treated as an opaque
progress stream, same policy as the `extra_progress` slot.

### 14e. Lemma errors are NOT per-lemma verdicts
No probe produced an "error" verdict *line* in the summary. A malformed lemma
fails a wellformedness check → the whole-theory WARNING count-line slot (§12b), and
a solver/oracle failure aborts the entire run after `Theory closed` with a runtime
stderr line and empty stdout — e.g. a missing heuristic oracle:
`tamarin-prover: ./oracle: readCreateProcess: posix_spawnp: does not exist (No such
file or directory)` (exit 1) `[r6_oracle_err]`. These belong to the §8c runtime-error
surface, not the summary taxonomy.

### 14f. Clean-crate model
`framing::LemmaOutcome` gains a `side: LemmaSide` field (`Whole` | `Rhs` | `Lhs` |
`Diff`); `Whole` keeps the pre-R6 rendering byte-for-byte. `Diff` renders the
`DiffLemma:` form and ignores `kind` (set to `AllTraces` by the `diff_lemma`
constructor). `Summary.lemmas` stays a single ordered `Vec` — a diff theory lists
its `Rhs`/`Lhs`/`Diff` entries in the §14c order and `render_block` prints them in
sequence. The verdict phrase moved to a shared `verdict_phrase(result, kind)`, so a
projected `Rhs`/`Lhs` line reuses it with that side's own `kind` — the projected
falsified/incomplete forms of the amended §14b are therefore rendered by the same
code path as whole-theory lemmas, now pinned against observed captures rather than
inferred. Byte-parity tests: `diff_summary_*` (eight captures: the original five
plus `r6_diff_false` projected-falsified-both-kinds, `r6_diff_lemma_noprove`
projected-incomplete, `r6_diff_asym` independent-sides) +
`diff_summary_line_bytes_are_exact`, and
`frame_batch_multi_warn_then_two_lemmas_reproduces_both_streams` re-verifies the
multi-lemma/multi-theory warning-vs-lemma interleaving (`r6_multi_warn_two`).

## 15. A fourth non-diff verdict phrase (Round 7)  [r7_* captures]

Rounds 5–6 recorded the non-diff verdict set as exactly three phrases (verified /
falsified-{found trace|no trace found} / analysis incomplete). Round 7 found a
FOURTH, driven out with the GPL example `csf23-subterms/YellowTest.spthy`
(observation input only; a subterm theory whose proofs reach a terminal
`UNFINISHABLE` step). Probe theories self-authored in `probes/round7/*.spthy`;
split-stream captures via `probes/split_probe.sh` → `captures/r7_*.{out,err}.txt`,
copied to `tests/fixtures/`.

### 15a. `analysis cannot be finished (reducible operators in subterms)`
`--prove` on YellowTest reports two of its lemmas as
`[r7_yellow_prove]`:

    YellowRed (exists-trace): analysis cannot be finished (reducible operators in subterms) (4 steps)
    YellowGreen (all-traces): analysis cannot be finished (reducible operators in subterms) (4 steps)

(the other two lemmas of the same theory close normally: `GreenYellow
(exists-trace): verified (3 steps)` and `RedYellow (all-traces): falsified - found
trace (3 steps)`). Properties of this fourth phrase:
- It is UNIFORM across trace-kinds — both an `exists-trace` and an `all-traces`
  lemma print the identical string with NO falsified-style `- found trace` /
  `- no trace found` suffix (like `verified` and `analysis incomplete`, unlike
  `falsified`). The phrase is a fixed canned string; the parenthetical reason
  `(reducible operators in subterms)` did not vary across the observed lemmas
  (stays plural regardless of count, matching the singular-"check" convention of
  the WARNING count line, §12b). No OTHER `analysis cannot be finished (<reason>)`
  variant surfaced — the only feature that produced it in the whole example
  corpus is `csf23-subterms` (four files); FreshOrderingTest / ParserTests only
  ever `verified`, and it is the theory pretty-print's `by UNFINISHABLE //
  reducible operator in subterm` proof step surfacing in the summary.
- It carries NO accompanying warning or advisory: the YellowTest `--prove`
  summary has no WARNING section and no `The analysis results might be wrong!`
  line, and stderr is the plain 5 phases + `[Saturating Sources] Done`
  `[r7_yellow_prove.err]`. So this state does not feed the might-be-wrong advisory
  of §12b (that advisory tracks a wellformedness-check failure under `--prove`,
  not proof incompleteness).
- It is a PROVEN-terminal state: it appears only when the lemma is actually
  proved. A DEFAULT (no `--prove`) run of the same theory reports all four lemmas
  as `analysis incomplete (1 steps)` `[r7_yellow_default]` — the phrase requires
  the proof to run and hit the wall.

### 15b. Bound exhaustion is DISTINCT (still `analysis incomplete`)
`--prove --bound=2` on the SAME theory cuts every proof short BEFORE it reaches
the `UNFINISHABLE` step, so all four lemmas read `analysis incomplete (4 steps)`
`[r7_yellow_bound]` — never the reducible-operators phrase. This directly answers
the Round-7 question: a bounded/unfinished-by-bound analysis and a
reducible-operators wall are DIFFERENT verdicts even for the identical lemmas
(the bound one reuses the existing `analysis incomplete`, §12c). `--stop-on-trace=
SORRY` does NOT change the phrasing: it still shows the reducible-operators phrase
for the wall lemmas and verified/falsified for the rest `[r7_yellow_sotsorry]`
(consistent with §14a `r6_sot_*`).

### 15c. It composes with the `--diff` side prefixes and both kinds
`--diff --prove` on `csf23-subterms/YellowDiffTest.spthy` (whose `diff(...)`
makes only the LHS projection reducible) yields `[r7_yellowdiff_prove]`:

    RHS :  GreenYellow (exists-trace): verified (3 steps)
    LHS :  GreenYellow (exists-trace): analysis cannot be finished (reducible operators in subterms) (3 steps)
    RHS :  RedYellow (all-traces): falsified - found trace (3 steps)
    LHS :  RedYellow (all-traces): analysis cannot be finished (reducible operators in subterms) (3 steps)
    RHS :  YellowRed (exists-trace): falsified - no trace found (3 steps)
    LHS :  YellowRed (exists-trace): analysis cannot be finished (reducible operators in subterms) (3 steps)
    RHS :  YellowGreen (all-traces): verified (3 steps)
    LHS :  YellowGreen (all-traces): analysis cannot be finished (reducible operators in subterms) (3 steps)
    DiffLemma:  Observational_equivalence : falsified - found trace (9 steps)

So the phrase is rendered by the SAME shared verdict path (§14f `verdict_phrase`)
as a whole-theory lemma: it slots behind the `RHS :  `/`LHS :  ` prefix
unchanged, for both trace-kinds, exactly like every other verdict; the sides are
independent (RHS closes, LHS hits the wall).

### 15d. Other candidate states re-confirmed to add NO new phrase
- `[sources]`/typing lemmas: with `--auto-sources`, the auto-generated
  `AUTO_typing (all-traces): verified (12 steps)` renders as a plain lemma line —
  no special "sources" summary line `[r7_openchains_autosrc]`. A theory whose
  partial deconstructions never resolve just proof-searches forever under a plain
  `--prove` (killed by timeout, exit 124, empty stdout — a resource condition, not
  a summary phrase) `[r7_openchains_prove]`. Confirms §14a: partial
  deconstructions / open chains add no summary-of-summaries line.
- Induction: a lemma marked `[use_induction]` (exists-trace or all-traces) proves
  normally to `verified`/`falsified` with no distinct phrase and no error
  `[r7_induction_prove]`.
- Lemma errors / solver failures / `--prove=<pattern>` non-match: unchanged from
  §14e / §12b — they abort with a runtime stderr line or feed the WARNING
  count-line slot, never a per-lemma verdict.

### 15e. Clean-crate model
`framing::LemmaResult` gains a fourth variant `AnalysisCannotBeFinished`;
`verdict_phrase` maps it to the canned
`analysis cannot be finished (reducible operators in subterms)` for both kinds, so
`Whole`/`Rhs`/`Lhs`/`Diff` rendering picks it up automatically (no per-side code).
Byte-parity tests: `summary_reducible_operators_whole_theory_both_kinds`
(split-stream, both kinds, `r7_yellow_prove`),
`summary_bound_exhaustion_is_incomplete_not_unfinishable` (the distinct bound case,
`r7_yellow_bound`), `diff_summary_projected_reducible_operators_both_sides_and_kinds`
(`r7_yellowdiff_prove`), and `reducible_operators_line_bytes_are_exact` (capture-
independent phrase bytes, whole + projected). All prior fixtures/tests stay green
(59 total).
