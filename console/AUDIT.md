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
