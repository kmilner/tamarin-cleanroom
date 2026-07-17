# Similarity audit — console / cli-clean

## Round 1 audit

Reviewer: similarity auditor (both-sides). Compared
`workspace/cli-clean/src/*` against HASKELL `src/Main/Console.hs`,
`src/Main/Environment.hs`, `src/Main.hs`, and the modes under
`src/Main/Mode/` (`Batch.hs`, `Interactive.hs`, `Test.hs`, `Intruder.hs`) plus
`src/Main/TheoryLoader.hs` (`theoryLoadFlags`).

**Findings: none (0).** No internal-decomposition mirror, non-observable name
match, echoed comment, or algorithmic-expression match survives filtration.

Items considered and filtered out (compatibility / observation-derived):

- Flag-table factoring `PROVE_COMMON` / `WITH_TOOLS` (modes.rs:58, 81) vs
  Haskell `theoryLoadFlags` (TheoryLoader.hs:86) and `toolFlags`
  (Environment.hs:30). Closest structural parallel: the clean shares the same
  20-flag proving block and 3-flag tool block between batch and interactive, in
  the same order and with the same batch split point
  (`PROVE_COMMON ++ [no-compress,parse-only,precompute-only] ++ outputFlags ++
  WITH_TOOLS`), mirroring `Batch.hs:53`. This SURVIVES the copying charge and
  is filtered out: the observed per-mode help fixtures (`fixtures/help_global.txt`,
  `fixtures/help_interactive.txt`) show the identical `prove…replication-bound`
  block and the `with-dot/json/maude` block as the maximal common contiguous
  prefix/suffix in both modes, so the shared-subset boundaries are directly
  visible in oracle output; the factoring is convergent and compatibility-motivated,
  and the clean names (`PROVE_COMMON`/`WITH_TOOLS`) are original, not the Haskell
  internal names.
- `FlagSpec { long, short, takes_value, aliases }` (modes.rs:43): an independent
  reduction of the public cmdargs `flagOpt/flagReq/flagNone` surface, not a mirror
  of a specific Haskell internal type.
- `app_error` CallStack string `src/Main/Mode/Batch.hs:162:33` (errors.rs:97):
  verbatim in captures/`err_bound_nonint.txt` and `err_output_module_bad.txt` —
  observed oracle output, compatibility content.
- Summary/framing assembly (framing.rs) builds the observed byte layout imperatively;
  it does not mirror Haskell `ppSummary`/`ppRep` `Pretty.vcat` combinator structure.
- Help/version text stored as opaque fixtures; the About-row split (version only in
  batch) is observable from the help pages.

Verdict: pass.

## Round 4 audit

Reviewer: similarity auditor (both-sides). Scope: this round's delta only, as
seen from clean-room HEAD `63ed8a9` — `git status/diff` restricted to `console/`.
The round adds the two-stream output model and typed value validation:

- NEW `src/stream.rs` (`Stream`, `Streams`); NEW `src/args.rs` (`parse_args`,
  `Args`, `Parsed`, the `StopOnTrace`/`PartialEval`/`OutputModule` enums,
  `read_haskell_int`, the fixed-order validation);
- rewrites of `src/framing.rs` (stream-split `frame_batch`/`frame_parse_only`/
  `frame_variants`, `Summary`/`BatchTheory`/`LoadedTheory`, `render_summary`),
  `src/errors.rs` (`ParseError{ text, stream, exit_code }`, the value-validation
  and consumer-extension renderers, `app_error` promoted to shared helper),
  `src/version.rs` (`frame_version` splits banner→stdout / maude preamble→stderr;
  `MAUDE_PATH` slot dropped from `version.tmpl`), `src/modes.rs`
  (`consumes_next`, `INTEROP_FLAGS`), `src/parse.rs` (`Options::last`/
  `occurrences`, index loop for `--flag value`), `src/lib.rs`, `src/main.rs`.

Compared against HASKELL `src/Main/Mode/Batch.hs` (`run`, `ppRep`/summary
assembly, the `error e` site), `src/Main/TheoryLoader.hs`
(`mkTheoryLoadOptions`, `stopOnTrace`, `parseIntArg`), `lib/theory/.../Module.hs`
(the `OutputMode` type and its `Show`), and the R4 evidence in
`workspace/BEHAVIOR.md` §10–§11a, `workspace/QUERIES.log` (Round-4 block),
`workspace/captures/split_*` and `vv_*`, and the new `probes/split_probe.sh`.

**Findings: none (0).** No protectable-expression copy survives filtration.

### Items examined and filtered out

- **App-`error` label strings** `bound`, `OpenChainsLimit`, `SaturationLimit`,
  `derivcheck-timeout`, `replication-bound` (`errors.rs` renderers; `args.rs`
  `build_args` label args). Despite `OpenChainsLimit`/`SaturationLimit` reading
  like internal camelCase, each appears **verbatim** in observed stderr
  (`captures/vv_openchains_x.err.txt`, `vv_saturation_x.err.txt`, etc.:
  `tamarin-prover: OpenChainsLimit: invalid bound given` …). Reproducing them is
  compelled compatibility/merger. The clean code even keeps the two roles
  distinct — it looks flags up by the observable **CLI** name (`o.last("open-chains")`)
  and uses `"OpenChainsLimit"` only where the byte-stream shows it (the error
  text) — whereas the source keys its `findArg` on `"OpenChainsLimit"`. Not a
  copy of the internal arg key.

- **CallStack `src/Main/Mode/Batch.hs:162:33`** (`app_error`). Verbatim in every
  `vv_*.err.txt`; the genuine `error e` site is `Batch.hs` ~161–162, so all
  `ArgumentError`s legitimately surface with that one call site. Observed
  compatibility content (already accepted R1); R4 only promotes it to a shared
  helper.

- **Enum value sets and case-rules.** `StopOnTrace{dfs,bfs,seqdfs,sorry,none}`,
  `PartialEval{summary,verbose}` (case-insensitive), `OutputModule`
  {spthytyped,spthy,msr,proverifequiv,proverif,deepsec} (case-SENSITIVE) are the
  accepted CLI value strings, observable via the help pages and the R4 case
  sweep (`QUERIES.log 16:45:00`). The Rust **variant identifiers** are
  PascalCase of those observable strings, NOT the Haskell constructors — the
  divergences prove independent derivation: `Verbose` vs source `Tracing`;
  `Proverif`/`ProverifEquiv` vs source `ModuleProVerif`/`ModuleProVerifEquivalence`;
  `Deepsec` vs source `ModuleDeepSec`. The type name `OutputModule` derives from
  the flag `--output-module`, not the source type `OutputMode`/`ModuleType`.

- **Defaults `OPEN_CHAINS_DEFAULT=10`, `SATURATION_DEFAULT=5`,
  `DERIVCHECK_TIMEOUT_DEFAULT=5`.** Not lifted non-observably from
  `defaultTheoryLoadOptions`: each is printed on the boundary — help pages show
  "(default 10)", "(default 5)", "(default 5)" (`fixtures/help_global.txt`,
  `help_interactive.txt`), and saturation=5 is independently pinned by the
  `--prove` capture `[Saturating Sources] … (Max 5)` (`split_batch_prove.err`).
  `replicationBound`'s source default 3 is deliberately NOT encoded (modeled as
  `Option`, pass-through), consistent with observation-only sourcing.

- **Fixed multi-invalid precedence order**
  `stop-on-trace > bound > partial-evaluation > open-chains > saturation >
  output-module > derivcheck-timeout > replication-bound` (`args.rs` `build_args`
  sequencing; `BEHAVIOR.md` §11). Traces to logged probe `QUERIES.log 16:47:00`
  ("fixed validation order … all pairwise relations consistent"). Crucially it is
  NOT a transcription of the source's arrangement: `mkTheoryLoadOptions`'s
  `Either`-applicative chain fixes precedence by textual position and places
  `outputModule` (pos ~13) BEFORE `openchain`/`saturation` (pos ~17/18), i.e.
  source order is `… partial-evaluation > output-module > open-chains > saturation
  > derivcheck …`. The clean order instead places `output-module` AFTER
  `saturation`. The divergence at exactly this link is dispositive evidence of
  probe-driven, non-copied origin. (Advisory, non-similarity: that same
  divergence suggests the pairwise sweep may not have exercised
  output-module × {open-chains, saturation}; the reference, being `Either`, should
  surface the output-module error first. Worth a confirming probe, but it does not
  bear on copyright and does not affect this verdict.)

- **`read_haskell_int`** (`args.rs`). Emulates GHC base's `readMaybe :: Maybe Int`
  (ws-trim, sign with optional interior ws, `0x`/`0o` radix, overflow-wrap) — a
  public-library semantic, not tamarin expression — and its accept/reject
  boundary is the observable target (`QUERIES.log 16:45:30`; test inputs
  `-5,0,5," 5","5 ","- 5",0x10,010,\t5,5\n` mirror the probe). Merger:
  byte-compatibility on the numeric boundary dictates the algorithm. The name and
  doc reference "Haskell/`readMaybe`" descriptively; no tamarin identifier or
  comment is carried over. (Advisory: the `0o` octal branch is not among the
  probed/tested inputs — `010` is present but is decimal in both `read` and the
  clean code, so it does not exercise octal; this is a public-library boundary
  detail, no similarity concern, minor traceability completeness only.)

- **Stream routing** (`stream.rs`, and stream tags throughout). Which text lands
  on stdout vs stderr is a probed black-box fact (`BEHAVIOR.md` §10, the
  `split_*` `.out.txt`/`.err.txt` capture pairs from `split_probe.sh`), not a
  source-structural feature. `Streams{out,err}` is a trivial original two-buffer
  model.

- **Summary framing** (`framing.rs` `Summary::render_block`, `render_summary`).
  Built imperatively from the captured byte layout (`BEHAVIOR.md` §10b;
  `split_batch_multifile.out`, `split_batch_output_single.out`), computing column
  alignment dynamically (pad labels to the width of `processing time:`). This does
  NOT mirror Haskell `ppRep`'s `Pretty.vcat`/`nest 2` list, which hardcodes
  `"output:          "` (fixed 10 spaces) and carries two `Pretty.text ""` after
  the analyzed header where the clean output shows one blank — again a
  bytes-first, non-source derivation. `BatchTheory`/`LoadedTheory` and the
  per-theory `extra_progress` slot are the clean-room's own stream-oriented model.

- **`Args` struct / field organization.** A union of the whole observable CLI
  flag surface (incl. `output`, `output-json/dot`, `with-*`, `port`, `interface`,
  `image-format`, `quiet`, `no-compress`, `debug`, `no-logging` — none of which
  are `TheoryLoadOptions` fields), grouped by the clean-room's own categories.
  Field order of the validated block follows the PROBED precedence (with the
  output-module divergence above), not the source record order. Not a mirror of
  `TheoryLoadOptions`; note it keeps `prove`/`lemma`/`defines` as separate vectors
  rather than the source's `findArg "prove" ++ findArg "lemma"` merge.

- **Consumer-extension flags** `--processors`, `--maude-processes`, `--data-dir`
  (`modes.rs` `INTEROP_FLAGS`/`extf`/`consumes_next`; `errors.rs`
  `bad_positive_int`/`missing_value`; `args.rs` `positive_int_opt`). Confirmed
  ABSENT from the reference (`QUERIES.log 16:46:00`: each yields `Unknown flag`),
  net-new original surface with original rejection text, omitted from help. No
  reference expression to copy; nothing derived from tamarin.

- **Version stream split** (`version.rs` `frame_version`; `version.tmpl` losing
  the `MAUDE_PATH`/preamble lines). Backed by `split_version`/`split_shortV`
  captures and `QUERIES.log 16:55:00`. Banner text is the project's own
  compatibility banner (opaque fixture, accepted R1).

Every behavioral claim in the new code traces to a Round-4 logged probe or a
captured fixture; the two advisory notes above are correctness/traceability-
completeness margin remarks, not copyright findings, and neither is a copied
protectable expression.

Verdict: pass.
