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

## Round 5 audit

Reviewer: similarity auditor (both-sides). Scope: this round's delta only, as
seen from clean-room HEAD `75807c0` — `git status/diff` restricted to `console/`.
Compared against upstream `src/Main/Mode/Batch.hs` (`ppSummary`/`ppRep`, `ppWf`),
`lib/theory/src/ClosedTheory.hs` (`prettyClosedSummary`), and
`lib/theory/src/Theory/Proof.hs` (`showProofStatus`). The round fills in the
summary-body content (`<result lines>` slot) and adds an incremental emitter:

- REWRITE `src/framing.rs`: `LemmaResult` gains kind-dependent falsified text via
  new `LemmaOutcome { name, kind, result, steps }` (`result_text`/`line`); new
  `WarningSummary { failed_checks, analysis_maybe_wrong }` (`render`, `WARNING_PREFIX`);
  `Summary` swaps its flat `entries: Vec<SummaryEntry>` for `warnings: Option<..>`
  + `lemmas: Vec<..>`, and `render_block` grows the two-section body assembly.
- NEW `src/emit.rs`: `Sink` trait, `StreamCollector`, `BatchEmitter`
  (`begin`/`progress`/`closed_phases`/`extra_progress`/`payload`/`record_summary`/
  `finish`), `drive_batch`; new re-exports in `lib.rs`.
- `tests/cli_tests.rs`: GAP-1 summary-content tests + GAP-2 streaming-equivalence
  tests; new `probes/round5/*.spthy` (self-authored) and `tests/fixtures/r5_*`
  golden captures.

**Findings: none (0).** No identifier constellation, internal-structure mirror,
non-boundary magic constant, or comment lineage survives filtration.

Items considered and filtered out:

- **Falsified/verdict wording** (`framing.rs` `LemmaOutcome::result_text`). The
  literals `verified` / `analysis incomplete` / `falsified - found trace`
  (all-traces) / `falsified - no trace found` (exists-trace) are byte-exact
  interop tokens the crate must reproduce; each is directly captured, not guessed
  (`BEHAVIOR.md` §12c; `QUERIES.log 22:05:00` r5_falsify_prove, `22:09:00`
  r5_exists_verified). Merger. Crucially the clean DECOMPOSITION is not the
  source's: upstream `showProofStatus` is an 8-arm flat match keyed on
  `(SystemTraceQuantifier, ProofStatus)` over a 6-value status enum
  (`TraceFound/CompleteProof/UnfinishableProof/IncompleteProof/UndeterminedProof/
  InvalidatedProof`); the clean collapses to a 3-value observed enum
  (`Verified/Falsified/AnalysisIncomplete`) and branches on `kind` only inside the
  `Falsified` arm. It omits the three un-probed statuses entirely — the signature
  of observed-only reconstruction, not transcription. No source identifier
  (`showProofStatus`, `ProofStatus`, `SystemTraceQuantifier`) is carried.

- **Warning heading + advisory** (`framing.rs` `WarningSummary::render`). The
  strings `WARNING: <N> wellformedness check failed!` (singular noun) and
  `The analysis results might be wrong!` are captured interop tokens
  (`BEHAVIOR.md` §12b; `QUERIES.log 22:06:00` r5_warn_lemma_prove). The `--prove`
  gating of the advisory is a black-box fact triangulated across four logged
  probes (`22:07:00` default = no advisory; `22:08:00` zero-lemma still emits;
  `22:08:30` zero-match still emits; `22:09:30` no-warning never emits), not read
  off the source's `[ … | thyLoadOptions.proveMode ]` guard — and the field is
  named `analysis_maybe_wrong` for the observable, not `proveMode`. The 11-space
  advisory indent is DERIVED as `WARNING_PREFIX.len()` (alignment relationship,
  §12b), not the source's hardcoded 9-space literal `"         The analysis…"`
  (upstream gets the other 2 from `nest 2`). Deriving the constant from the prefix
  rather than copying the magic literal is dispositive of non-copying.

- **Summary body layout** (`framing.rs` `Summary::render_block`). The
  opening `"  \n"` line + present-sections-joined-by-`"  \n"` (warning first,
  then lemmas) is reconstructed imperatively from the captured 4-case truth table
  (`BEHAVIOR.md` §12a; captures r5_nolemma_default / r5_exists_verified /
  r5_freshpub_prove / r5_warn_lemma_*). It does NOT mirror the source's
  Pretty-combinator mechanics — `ppRep`'s `nest 2 $ vcat [ …, "", ppWf $--$
  lemmas ]`, where the `  ` separators are `Pretty.text ""` under `nest 2` and the
  section short-circuits fall out of `emptyDoc`/`$--$`. Bytes-first, not
  structure-first. `LemmaOutcome::line`'s `<name> (<kind>): <verdict> (<N> steps)`
  is the observed row format (upstream `prettyClosedSummary`'s
  `<-> analysisType <> colon <-> … <-> parens (integer siz <-> "steps")`); the
  `(N steps)` / `steps` token is interop content, and the clean carries the step
  count as an opaque input rather than replicating the `foldProof proofStepSummary`
  step-counting fold. No source comment (the `prettyClosedSummary` "hacky
  construction" / "Just annotation invariant" block) is echoed.

- **Incremental emitter** (`src/emit.rs`). No upstream counterpart: `Batch.hs`
  emits with `mapM_ (putStrLn . renderDoc) docs` then one `putStrLn … ppSummary`
  — no streaming/`Sink` architecture. The `Sink`/`BatchEmitter`/`StreamCollector`/
  `drive_batch` surface and its `begin/progress/payload/record_summary/finish`
  lifecycle are original Rust API, reusing the crate's own `framing` primitives
  (`progress_line`, `render_summary`, `maude_preamble`). Introduces no new
  observable strings — a pure per-stream re-ordering verified by equivalence
  against `frame_batch` (`BEHAVIOR.md` §13), so the absence of a dedicated oracle
  probe is correct, not a provenance gap. No identifier or structure is derived
  from tamarin.

- **Round-5 probes/fixtures.** `probes/round5/{falsify,warn_and_lemma,
  exists_verified,nolemma_clean}.spthy` are self-authored (own theory names
  `FalsifyMe`/`WarnAndLemma`/`ExistsVerified`/`NoLemmaClean`, own comments),
  matching the `BEHAVIOR.md` §12 claim "own fixtures, not the GPL examples." The
  `r5_nslpk3_*` / `r5_multi_*` captures use `classic/NSLPK3.spthy` as probe INPUT
  only; the `tests/fixtures/r5_*.out.txt/.err.txt` are the reference program's own
  emitted bytes (golden oracle artifacts, consistent with prior rounds), not
  authored expression in the clean crate. Probe-derived — filtered.

Every behavioral claim added this round traces to a logged Round-5 probe
(`QUERIES.log 22:05:00–22:11:00`) or a captured fixture; the streaming layer is a
no-new-oracle-facts internal API validated by byte-equivalence tests. Nothing in
the delta is a copied protectable expression.

Verdict: pass.

## Round 6 audit

Scope: this round's working-tree delta under `console/`, against clean-room HEAD
`e76455a`. Sources changed: `framing.rs` (+94/-27 net), `lib.rs` (re-export),
`cli-clean/tests/cli_tests.rs`, plus `BEHAVIOR.md` §14 / `QUERIES.log` Round-6
block and the `probes/round6/*`, `captures/r6_*`, `tests/fixtures/r6_diff_*`
artifacts. Behavioural theme: the `--diff` lemma-verdict taxonomy — the
side-prefixed (`RHS`/`LHS`) projected-lemma lines and the trailing observational-
equivalence (`DiffLemma:`) line, plus closure probes (sources/open-chains,
stop-on-trace, oracle-error) that added NO new source. Upstream comparands:
`prettyClosedDiffSummary` (`lib/theory/src/ClosedTheory.hs:492–545`),
`showDiffProofStatus` (`Theory/Proof.hs:1115–1121`), `Side` (`Constraint/System.hs
:329`), `unprovenDiffLemma "Observational_equivalence"` (`OpenTheory.hs:558–559`),
`ppSummary` (`Main/Mode/Batch.hs:127–151`).

- **`LemmaSide` enum + `LemmaOutcome` constructors** (`framing.rs`). The clean
  crate collapses whole-run, projected-side and diff-lemma rows into ONE
  `LemmaOutcome` carrying a `side: LemmaSide` (`Whole|Rhs|Lhs|Diff`), with three
  named constructors (`whole`/`projected`/`diff_lemma`) and one `Vec<LemmaOutcome>`
  ordered by the caller. This is NOT the source's decomposition: upstream renders
  diff summaries from two distinct list-comprehensions inside
  `prettyClosedDiffSummary` — `lemmaSummaries` over `EitherLemmaItem (s, lem)` and
  `diffLemmaSummaries` over `DiffLemmaItem lem`, joined `(vcat …) $$ (vcat …)` — and
  keeps non-diff rendering in an entirely separate `prettyClosedSummary`. The clean
  invents a tagged-union abstraction that has no upstream analogue; `Whole`/`Diff`
  are not source constructor names, and `Rhs`/`Lhs` are idiomatic Rust casing of the
  behaviour token, not the source's all-caps `RHS`/`LHS` `Side` constructors. No
  source identifier (`EitherLemmaItem`, `DiffLemmaItem`, `prettyClosedDiffSummary`,
  `lDiffName`, `Side`) is carried. Original structure.

- **`RHS :  ` / `LHS :  ` prefixes** (`LemmaOutcome::line`). The three-token layout
  `"RHS" + " :  " + <normal row>` is a byte-exact interop constant, directly
  captured (`BEHAVIOR.md` §14b; `QUERIES.log 00:43:00` r6_diff_with_lemma,
  `00:44:00` r6_diff_two_lemmas; fixtures `r6_diff_with_lemma/two_lemmas/warn`).
  The literal derives from upstream `text (show s) <-> text ": "` where
  `show LHS/RHS` yields the tokens and `<->` inserts the interstitial space — but
  the clean writes the observed bytes (`format!("RHS :  {core}")`) rather than
  replaying the Pretty `<->`-spacing mechanics; bytes-first, not combinator-first,
  same signature as the Round-5 body-layout reasoning. The projected remainder is
  the pre-existing non-diff row (§12c), unchanged, `<kind>` rendering normally
  including `exists-trace` (r6_diff_two_lemmas `can_send`). Merger token, filtered.

- **`DiffLemma:  <name> : <verdict> (<N> steps)`** (`LemmaOutcome::line`, `Diff`
  arm). Byte-exact interop line, captured (`BEHAVIOR.md` §14b; `QUERIES.log
  00:40:00` r6_diff_n5n6, `00:41:00` r6_diff_equiv_true, `00:42:00` r6_diff_default;
  fixtures). The `DiffLemma:  ` prefix (NO space before colon, two after) vs the
  `RHS :  ` shape (space before colon) is an observed distinction, logged verbatim,
  not inferred from source. Structurally the clean does NOT copy the source's split
  into a second status function: upstream keys diff verdicts off `showDiffProofStatus`
  (a separate 6-arm match: `TraceFound/CompleteProof/UnfinishableProof/IncompleteProof
  /UndeterminedProof/InvalidatedProof`); the clean instead REUSES its own 3-value
  `verdict_phrase(result, TraceKind::AllTraces)`, exploiting that a diff lemma has
  no trace-kind and its `falsified` always reads `- found trace`. Reusing the
  existing observed-only 3-arm phrase map (omitting the three un-probed statuses)
  is the fingerprint of reconstruction, not transcription of the dual-function source.

- **`verdict_phrase` extraction** (`framing.rs`). The `result_text` *method* became a
  free `fn verdict_phrase(result, kind)` so the `Diff` arm can call it with a fixed
  kind. The phrase strings themselves (`verified` / `analysis incomplete` /
  `falsified - found trace` / `falsified - no trace found`) are UNCHANGED from
  Round 5 — no new interop token is introduced by the extraction, and the doc-comment
  cross-references were retargeted (`result_text` → `verdict_phrase`) with no source
  comment carried. Internal refactor; carries no new provenance.

- **`Observational_equivalence` name.** The auto-generated diff-lemma name is a
  captured OUTPUT value (upstream `unprovenDiffLemma "Observational_equivalence" []`),
  logged at `QUERIES.log 00:40:00` and asserted in `BEHAVIOR.md` §14b ("always
  named `Observational_equivalence`"). It is NOT baked into `framing.rs` — the
  `diff_lemma` constructor takes `name: impl Into<String>`, so the string appears
  only as observed data in tests/fixtures, exactly like any lemma name flowing
  through the crate. Boundary datum, filtered.

- **Diff summary ordering** (§14c → single ordered `Vec`). The observed order —
  per ordinary lemma its `RHS` then `LHS`, all pairs, then the single `DiffLemma`
  last — is triangulated from captures (`QUERIES.log 00:44:00` r6_diff_two_lemmas
  established the per-lemma grouping `RHS l1, LHS l1, RHS l2, LHS l2`; r6_diff_warn
  confirmed warning-section-first). The clean encodes this as caller-supplied `Vec`
  order printed in sequence by `render_block`, NOT as the source's
  `(vcat lemmaSummaries) $$ (vcat diffLemmaSummaries)` with the RHS/LHS interleave
  falling out of `EitherLemmaItem` item layout. Behaviour reproduced by a
  port-original mechanism.

- **No comment lineage.** Upstream's `prettyClosedDiffSummary` carries a long
  `Just`-annotation-invariant note and the `TODO: The whole consruction seems a bit
  hacky` block, repeated verbatim across all three comprehensions; none of that is
  echoed. The clean's comments describe the observed byte layout and its own enum
  only.

- **Round-6 probes/fixtures.** `probes/round6/{diff_equiv_true,diff_with_lemma,
  diff_two_lemmas,diff_warn,open_chains,partialdecon}.spthy` are self-authored (own
  theory names `DiffTwoLemmas`/`DiffWarn`/… , own `diff(~m,~m)` / `diff(~k,~'foo')`
  constructions), matching the §14 "own fixtures" claim. `r6_diff_lrmismatch` reuses
  `round2/diff_left_right_mismatch.spthy` and `r6_analysed_reload` an eccDAA
  `.analysed.spthy` as probe INPUT only; `tests/fixtures/r6_diff_*.out.txt` are the
  reference program's own emitted bytes (golden oracle artifacts), not authored
  expression in the clean crate. Probe-derived — filtered.

Closure-only probes (r6_sot_*, r6_openchains_*, r6_partialdecon, r6_oracle_err,
r6_analysed_reload) added NO summary-taxonomy line form — each logged a negative
result (no new verdict phrase; `[sources]` invisible; partial-deconstruction adds no
summary line; oracle failure is a §8c runtime-error abort, empty stdout) and touched
no source. Every behavioural claim added this round traces to a logged Round-6 probe
(`QUERIES.log 00:40:00–00:49:00`) or a captured fixture, and the only new source is a
byte-exact reproduction of the `--diff` line forms via a tagged-union abstraction that
does not exist upstream. Nothing in the delta is a copied protectable expression.

VERDICT: pass

## Round 6 (cont.) audit

Delta audited: the **working-tree changes** in `console/` on top of committed HEAD
`ce83726` (the prior Round-6 section above is already landed). Scope, from
`git -C /home/kamilner/tamarin-cleanroom status/diff -- console/`:

- **`cli-clean/tests/cli_tests.rs`** (+107): four new tests —
  `diff_summary_projected_falsified_both_kinds`,
  `diff_summary_projected_incomplete_without_prove`,
  `diff_summary_projected_sides_are_independent`,
  `frame_batch_multi_warn_then_two_lemmas_reproduces_both_streams`.
- **`workspace/BEHAVIOR.md`** (+37): §14b/§14e amendments (projected verdict set,
  lemma-selection never omits a line, side-independence).
- **`workspace/QUERIES.log`** (+9): Round-6-cont probe log `14:30–14:35`.
- **New self-authored probes** `probes/round6/{diff_asym,diff_false,two_lemma}.spthy`;
  new fixtures/captures `r6_diff_{asym,false,lemma_noprove}`, `r6_multi_warn_two`,
  and closure captures `r6_twolemma_{prove,proveone,lemmafilter}`.
- **No production source changed.** `git diff -- console/workspace/cli-clean/src/`
  is empty; `framing.rs`/`lib.rs`/`version.rs` are untouched this round. There is
  therefore no new code surface on which protectable expression could land — the
  delta is tests, self-authored input theories, oracle-derived fixtures, and prose.

Both-sides comparison ran against `lib/theory/src/Theory/Proof.hs`
(`showProofStatus`/`showDiffProofStatus`, 1104–1121), `lib/theory/src/ClosedTheory.hs`
(the `lemmaSummaries`/`diffLemmaSummaries` `<->`-combinator render, 505–545),
`src/Main/Mode/Batch.hs` (`summary of summaries`, 138), and
`lib/theory/src/Theory/Tools/Wellformedness.hs` (1163/1183). Findings:

- **No new source ⇒ no new copied structure.** Every identifier the four tests
  touch (`assert_diff_summary`, `rhs`, `lhs`, `diff_lemma`, `frame_batch`,
  `BatchTheory`, `Summary`, `MaudeInfo`, `slots`, `summary_start`, `warn`, `lemma`,
  `s`, `TraceKind`, `LemmaResult`) is the clean crate's own pre-existing API landed
  and audited in earlier rounds (`src/framing.rs`, `src/version.rs`); confirmed the
  committed test file already defined/imported all of them. The tests add call sites
  and data, not logic.

- **Verdict-phrase strings — boundary interop, unchanged.** `"verified"`,
  `"falsified - found trace"`, `"falsified - no trace found"`, `"analysis incomplete"`
  are verbatim `showProofStatus`/`showDiffProofStatus` outputs (Proof.hs 1105–1110),
  but they are byte-exact output tokens the console exists to reproduce; they entered
  the clean in Round 5 (`verdict_phrase`) and are untouched here. This round adds
  **no new phrase** — and, decisively, the three still-unprobed statuses
  (`"analysis cannot be finished (reducible operators in subterms)"`,
  `"analysis undetermined"`, `"proof has been invalidated"`, Proof.hs 1111–1113 /
  1118–1120) remain **absent** from the clean. The projected RHS/LHS lines are shown
  (tests + fixtures) to reuse the *same* observed-only 3-value phrase map for their
  own `kind`, rather than transcribing upstream's dual 6-arm `showProofStatus` +
  6-arm `showDiffProofStatus` pair. Reproducing only the probed subset off one shared
  reused mapping is the fingerprint of reconstruction, not of copying the source's
  two exhaustive status functions. Filtered.

- **RHS/LHS/DiffLemma line forms — unchanged Round-5/6 bytes-first design.** Upstream
  emits `text (show s) <-> text ": " <-> … <-> showProofStatus …` and
  `text "DiffLemma: " <-> …` via Pretty `<->` spacing (ClosedTheory.hs 510–541); the
  clean writes the observed bytes directly (`format!("RHS :  {core}")`, prior round).
  No combinator lineage, and nothing in this delta modifies that path — the new tests
  merely pin previously-unobserved *verdict* values (falsified/both-kinds, incomplete,
  asymmetric) through it. Filtered.

- **Probes are self-authored; input syntax is merger.** `diff_asym.spthy` /
  `diff_false.spthy` / `two_lemma.spthy` carry own theory names (`DiffAsym`,
  `DiffFalse`, `TwoLemma`), own rule/lemma names (`Send`/`Leak`/`A`,
  `secret`/`leaked`/`impossible`/`first`/`second`) and own `diff()` constructions
  (`diff(~m, senc(~m,~k))`, `diff(~m,~m)`). The `.spthy` surface syntax
  (`builtins:`, `rule`, `--[ ]->`, `lemma`, `exists-trace`) is the tool's required
  input language — merger, dictated by the need to drive tamarin. No resemblance to
  any upstream `examples/` theory. Filtered.

- **Fixtures are oracle golden bytes.** `r6_diff_{asym,false,lemma_noprove}.out.txt`
  and `r6_multi_warn_two.{out,err}.txt` each carry the full HS proof body plus the
  `Generated from: Tamarin version 1.13.0 … Git revision 0234f6a1…` trailer and the
  `summary of summaries:` block — i.e. the reference prover's own emitted bytes via
  `oracle/hs_oracle.sh`, not authored expression. All four new tests reproduce these
  bytes through the *unchanged* clean code path and **pass** (verified by running
  `cargo test diff_summary_projected` and `…multi_warn_then_two`: 4/4 ok). Their step
  counts (`3/2/67/1/44`) and lemma names are boundary observations copied from the
  captures, cross-referenced in `QUERIES.log 14:30–14:35`. Probe-/boundary-derived,
  filtered.

- **Doc echo of a wellformedness string — observed output, not source.**
  `BEHAVIOR.md` describes the `--lemma`/`--prove` mismatch as tripping a check
  ("… does not correspond to a specified lemma in the theory"). Upstream's literal is
  `"do(es) not correspond to a specified lemma in the theory"`
  (Wellformedness.hs 1163/1183). The clean's text is (a) a *paraphrase* ("does" vs the
  source's `"do(es)"`), (b) a description of a **warning string surfaced in tamarin's
  own OUTPUT** — a boundary datum captured in `r6_twolemma_lemmafilter`, not internal
  source expression — and (c) present only in a behaviour note, embedded in **no**
  clean source file. Boundary observation in prose; filtered. (The clean correctly
  routes this to the existing §12b WARNING count-line slot rather than inventing a
  per-lemma "skipped" form, matching observed behaviour, not the source's WF plumbing.)

- **Side-independence / LHS=left, RHS=right.** The `diff_asym` claim that `LHS` is the
  first `diff()` argument and `RHS` the second, and that the two sides compute
  independently, is established by capture (`r6_diff_asym`: `RHS secret verified` vs
  `LHS secret falsified`), logged at `QUERIES.log 14:32:00`. Behavioural fact read off
  the oracle, not lifted from source. Filtered.

Every behavioural claim added this round traces to a logged Round-6-cont probe
(`QUERIES.log 14:30:00–14:35:00`) or a captured golden fixture. The delta introduces
no production source, no new interop token, no upstream identifier constellation, no
combinator/structure lineage, and no comment lineage (the `TODO: The whole consruction
seems a bit hacky` note and the `Just`-annotation-invariant block repeated across
ClosedTheory.hs's comprehensions are, as before, not echoed). Nothing in the delta is a
copied protectable expression.

VERDICT: pass
